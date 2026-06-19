use std::{sync::Arc, time::Duration as StdDuration};

use nako_addon_client::{
    AddonClientError, AddonEventCallFailure, AddonEventCallRequest, ReqwestAddonTransport,
    call_addon_event_with_outcome,
};
use nako_addon_protocol::{
    AddonAuth, AddonEventSubscriptionDeclaration, AddonScope, validate_manifest,
};
use nako_api::extension::{
    AddonEventDeliveryAttemptSummary, AddonEventDeliveryAttemptsResponse,
    AddonEventDispatchEventSummary, AddonEventDispatchResponse, AddonEventReplayResponse,
    AddonEventSchedulerWorkItem, AddonEventSchedulerWorkResponse, AddonEventSchedulerWorkStatus,
    ReplayAddonEventRequest,
};
use nako_core::{
    AddonEventDeliveryAttemptId, AddonEventDeliveryRepository, AddonEventDeliveryStatus,
    AddonEventSchedulerWorkRecord, AddonRegistrationRecord, AddonRepository,
    AddonRoutingDeclarationKind, AddonRoutingPlanStatus, AddonRoutingPlanTarget, AddonStatus,
    ClaimAddonEventDeliveryAttempt, EventId, EventOutboxRepository, NakoError,
    OutboxEventListFilter, OutboxEventRecord, OutboxEventStatus, PageRequest, Result, SecretString,
};
use time::{Duration as TimeDuration, OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::{sync::Semaphore, task::JoinSet};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use crate::config::AddonEventSchedulerConfig;

use super::{
    AddonAppService, helpers::resolve_outbound_task_dispatch_secret,
    helpers::stored_granted_scopes, routing::declaration_scopes_granted,
};

const ADDON_EVENT_DELIVERY_LEASE_SECONDS: i64 = 300;

impl AddonAppService {
    pub(crate) fn start_addon_event_scheduler(&self, config: AddonEventSchedulerConfig) -> bool {
        if !config.enabled {
            return false;
        }

        let service = self.clone();
        let shutdown = self.runtime.shutdown_token();
        self.runtime.spawn(
            "addon_event_scheduler",
            "addon.event.scheduler",
            async move {
                run_addon_event_scheduler_loop(service, config, shutdown).await;
            },
        );

        true
    }

    pub(crate) async fn run_addon_event_scheduler_tick(
        &self,
        config: AddonEventSchedulerConfig,
    ) -> Result<()> {
        let events = self
            .store
            .list_outbox_events(
                OutboxEventListFilter {
                    status: Some(OutboxEventStatus::Pending),
                    ..OutboxEventListFilter::default()
                },
                PageRequest::new(config.batch_size.max(1), 0).clamped(),
            )
            .await?;
        let event_permits = Arc::new(Semaphore::new(config.concurrency.max(1)));
        let mut workers = JoinSet::new();

        for event in events {
            let work = self.list_addon_event_scheduler_work(event.id).await?;
            if work.due_work_count == 0 {
                continue;
            }

            let service = self.clone();
            let event_id = event.id;
            let event_permits = Arc::clone(&event_permits);
            workers.spawn(async move {
                let _permit =
                    event_permits
                        .acquire_owned()
                        .await
                        .map_err(|err| NakoError::Provider {
                            provider: "addon_event_scheduler".to_owned(),
                            message: format!(
                                "addon event scheduler concurrency budget was closed: {err}"
                            ),
                        })?;
                service
                    .deliver_addon_events_for_event(event_id)
                    .await
                    .map(|_| ())
            });
        }

        while let Some(result) = workers.join_next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    warn!(
                        error = %err,
                        "addon event scheduler failed to deliver due event"
                    );
                }
                Err(err) => {
                    warn!(
                        error = %err,
                        "addon event scheduler worker join failed"
                    );
                }
            }
        }

        Ok(())
    }

    pub async fn list_addon_event_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<AddonEventDeliveryAttemptsResponse> {
        self.get_outbox_event_or_not_found(event_id).await?;
        let attempts = self
            .store
            .list_addon_event_delivery_attempts(event_id)
            .await?
            .into_iter()
            .map(AddonEventDeliveryAttemptSummary::from)
            .collect();

        Ok(AddonEventDeliveryAttemptsResponse { event_id, attempts })
    }

    pub async fn deliver_addon_events_for_event(
        &self,
        event_id: EventId,
    ) -> Result<AddonEventDispatchResponse> {
        self.dispatch_addon_events_for_event(event_id, AddonEventDeliveryMode::Normal)
            .await
    }

    pub async fn replay_addon_events_for_event(
        &self,
        event_id: EventId,
        request: ReplayAddonEventRequest,
    ) -> Result<AddonEventReplayResponse> {
        let reason_code = validate_replay_reason_code(&request.reason_code)?;
        let dispatch = self
            .dispatch_addon_events_for_event(
                event_id,
                AddonEventDeliveryMode::ForcedReplay {
                    reason_code: reason_code.clone(),
                },
            )
            .await?;

        Ok(AddonEventReplayResponse {
            reason_code,
            dispatch,
        })
    }

    async fn dispatch_addon_events_for_event(
        &self,
        event_id: EventId,
        mode: AddonEventDeliveryMode,
    ) -> Result<AddonEventDispatchResponse> {
        let event = self.get_outbox_event_or_not_found(event_id).await?;
        let addons = self
            .store
            .list_addon_registrations(Some(AddonStatus::Enabled))
            .await?;
        let mut workers = JoinSet::new();
        let mut attempted_subscriptions = 0_u32;
        let mut skipped_subscriptions = 0_u32;
        let mut delivered = 0_u32;
        let mut failed = 0_u32;
        let mut attempts = Vec::new();
        let mut error_count = 0_u32;

        for addon in addons {
            let plans = self.store.list_addon_routing_plans(addon.id).await?;
            let candidate_plans = plans
                .into_iter()
                .filter(|plan| {
                    plan.declaration_kind == AddonRoutingDeclarationKind::EventSubscription
                        && plan.event_kind.as_deref() == Some(event.kind.as_str())
                })
                .collect::<Vec<_>>();

            if candidate_plans.is_empty() {
                skipped_subscriptions += 1;
                continue;
            }

            let manifest = self.stored_manifest(&addon)?;
            validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
                message: err.to_string(),
            })?;
            let granted_scopes = stored_granted_scopes(&addon)?;
            for plan in candidate_plans {
                let Some(subscription) = manifest
                    .event_subscriptions
                    .iter()
                    .find(|subscription| subscription.id == plan.declaration_id)
                    .cloned()
                else {
                    skipped_subscriptions += 1;
                    continue;
                };
                if plan.status != AddonRoutingPlanStatus::Executable
                    || plan.target != AddonRoutingPlanTarget::EventOutbox
                {
                    skipped_subscriptions += 1;
                    continue;
                }
                if !declaration_scopes_granted(&subscription.required_scopes, &granted_scopes) {
                    skipped_subscriptions += 1;
                    continue;
                }
                if !addon_event_subscription_filter_matches(&event, &subscription.filters) {
                    skipped_subscriptions += 1;
                    continue;
                }

                attempted_subscriptions += 1;
                let service = self.clone();
                let event = event.clone();
                let addon = addon.clone();
                let granted_scopes = granted_scopes.clone();
                let mode = mode.clone();
                workers.spawn(async move {
                    service
                        .deliver_addon_event_subscription(
                            event,
                            addon,
                            subscription,
                            granted_scopes,
                            mode,
                        )
                        .await
                });
            }
        }

        while let Some(result) = workers.join_next().await {
            match result {
                Ok(Ok(outcome)) => match outcome {
                    AddonEventDeliveryOutcome::Attempt(attempt) => {
                        match attempt.status {
                            AddonEventDeliveryStatus::Succeeded => delivered += 1,
                            AddonEventDeliveryStatus::Failed => failed += 1,
                            AddonEventDeliveryStatus::Pending
                            | AddonEventDeliveryStatus::Running => {}
                        }
                        attempts.push(attempt);
                    }
                    AddonEventDeliveryOutcome::Skipped => skipped_subscriptions += 1,
                },
                Ok(Err(err)) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "addon event delivery failed before attempt completion"
                    );
                    error_count = error_count.saturating_add(1);
                }
                Err(err) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "addon event delivery worker join failed"
                    );
                    error_count = error_count.saturating_add(1);
                }
            }
        }
        attempts.sort_by_key(|attempt| {
            (
                attempt.addon_id,
                attempt.declaration_id.clone(),
                attempt.attempt_number,
            )
        });

        Ok(AddonEventDispatchResponse {
            event: AddonEventDispatchEventSummary::from_record(&event),
            attempted_subscriptions,
            delivered,
            failed,
            skipped_subscriptions,
            attempts: attempts
                .into_iter()
                .map(AddonEventDeliveryAttemptSummary::from)
                .collect(),
            error_count,
        })
    }

    pub async fn list_addon_event_scheduler_work(
        &self,
        event_id: EventId,
    ) -> Result<AddonEventSchedulerWorkResponse> {
        let event = self.get_outbox_event_or_not_found(event_id).await?;
        let work_records = self.store.list_addon_event_scheduler_work(event_id).await?;
        let now =
            OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("failed to format addon event scheduler timestamp: {err}"),
                })?;
        let mut work = Vec::with_capacity(work_records.len());
        for record in work_records {
            work.push(
                self.addon_event_scheduler_work_item(&event, record, now.as_str())
                    .await?,
            );
        }
        let due_work_count = work
            .iter()
            .filter(|item| {
                matches!(
                    item.status,
                    AddonEventSchedulerWorkStatus::Due | AddonEventSchedulerWorkStatus::RetryDue
                )
            })
            .count();
        let blocked_work_count = work
            .iter()
            .filter(|item| {
                matches!(
                    item.status,
                    AddonEventSchedulerWorkStatus::Deferred
                        | AddonEventSchedulerWorkStatus::Exhausted
                        | AddonEventSchedulerWorkStatus::InFlight
                )
            })
            .count();

        Ok(AddonEventSchedulerWorkResponse {
            event: AddonEventDispatchEventSummary::from_record(&event),
            due_work_count,
            blocked_work_count,
            work,
        })
    }

    async fn addon_event_scheduler_work_item(
        &self,
        event: &OutboxEventRecord,
        record: AddonEventSchedulerWorkRecord,
        now: &str,
    ) -> Result<AddonEventSchedulerWorkItem> {
        let addon = self
            .store
            .get_addon_registration(record.addon_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "addon_registration",
                id: record.addon_id.to_string(),
            })?;
        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;
        let max_attempts = manifest.default_max_attempts.unwrap_or(3);
        let granted_scopes = stored_granted_scopes(&addon)?;
        let subscription = manifest
            .event_subscriptions
            .iter()
            .find(|subscription| subscription.id == record.declaration_id);

        let (status, safe_reason_code) = addon_event_scheduler_work_status(
            event,
            &record,
            max_attempts,
            subscription,
            &granted_scopes,
            now,
        );

        Ok(AddonEventSchedulerWorkItem {
            addon_id: record.addon_id,
            manifest_id: addon.manifest_id,
            manifest_version: addon.version,
            declaration_id: record.declaration_id,
            event_kind: record.event_kind,
            status,
            safe_reason_code,
            routing_plan_status: record.routing_plan_status,
            routing_plan_target: record.routing_plan_target,
            attempt_count: record.attempt_count,
            next_attempt_number: record.next_attempt_number,
            max_attempts,
            latest_attempt_status: record.latest_attempt_status,
            latest_http_status: record.latest_http_status,
            next_retry_at: record.latest_next_retry_at,
            lease_expires_at: record.latest_lease_expires_at,
        })
    }

    async fn deliver_addon_event_subscription(
        &self,
        event: OutboxEventRecord,
        addon: AddonRegistrationRecord,
        subscription: AddonEventSubscriptionDeclaration,
        granted_scopes: Vec<AddonScope>,
        mode: AddonEventDeliveryMode,
    ) -> Result<AddonEventDeliveryOutcome> {
        if addon.status != AddonStatus::Enabled {
            return Err(NakoError::Conflict {
                message: format!("addon registration {} is not enabled", addon.id),
            });
        }
        if subscription.event_kind != event.kind.as_str() {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "addon event subscription {} expects {} but event {} is {}",
                    subscription.id,
                    subscription.event_kind,
                    event.id,
                    event.kind.as_str()
                ),
            });
        }

        let manifest = self.stored_manifest(&addon)?;
        validate_manifest(&manifest).map_err(|err| NakoError::InvalidInput {
            message: err.to_string(),
        })?;
        if !declaration_scopes_granted(&subscription.required_scopes, &granted_scopes) {
            return Err(NakoError::Forbidden {
                message: format!(
                    "addon {} is missing grants required for event subscription {}",
                    addon.id, subscription.id
                ),
            });
        }
        if !addon_event_subscription_filter_matches(&event, &subscription.filters) {
            return Ok(AddonEventDeliveryOutcome::Skipped);
        }

        let max_attempts = manifest.default_max_attempts.unwrap_or(3);
        let Some(attempt) = self
            .store
            .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
                id: AddonEventDeliveryAttemptId::new(),
                addon_id: addon.id,
                event_id: event.id,
                declaration_id: subscription.id.clone(),
                max_attempts,
                now: addon_event_timestamp_now()?,
                lease_expires_at: addon_event_delivery_lease_expires_at()?,
                forced_replay: mode.is_forced_replay(),
                replay_reason_code: mode.replay_reason_code().map(ToOwned::to_owned),
            })
            .await?
        else {
            return Ok(AddonEventDeliveryOutcome::Skipped);
        };
        let payload =
            serde_json::from_str::<serde_json::Value>(&event.payload_json).map_err(|err| {
                NakoError::InvalidInput {
                    message: format!("failed to parse outbox event payload JSON: {err}"),
                }
            })?;
        let outbound_secret = resolve_addon_event_outbound_secret(&addon, manifest.auth)?;
        let permit =
            self.permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|err| NakoError::Provider {
                    provider: "addon_event".to_owned(),
                    message: format!("addon event resource budget was closed: {err}"),
                })?;
        let _permit = permit;
        let outcome = call_addon_event_with_outcome(
            &ReqwestAddonTransport::default(),
            &manifest,
            &granted_scopes,
            AddonEventCallRequest {
                subscription_id: subscription.id,
                event_id: event.id.to_string(),
                event_kind: event.kind.as_str().to_owned(),
                subject_kind: event.subject.kind().to_owned(),
                subject_id: event.subject.id(),
                occurred_at: event.occurred_at,
                attempt: attempt.attempt_number,
                payload,
            },
            outbound_secret
                .as_ref()
                .map(nako_core::SecretString::expose_secret),
        )
        .await;

        match outcome {
            Ok(outcome) => self
                .store
                .set_addon_event_delivery_attempt_result(
                    attempt.id,
                    AddonEventDeliveryStatus::Succeeded,
                    Some(outcome.http_status),
                    None,
                    None,
                )
                .await
                .map(AddonEventDeliveryOutcome::Attempt),
            Err(failure) => {
                let next_retry_at = if addon_event_client_error_is_retryable(&failure.error) {
                    next_addon_event_retry_at(attempt.attempt_number, max_attempts)?
                } else {
                    None
                };
                self.store
                    .set_addon_event_delivery_attempt_result(
                        attempt.id,
                        AddonEventDeliveryStatus::Failed,
                        failure.error.http_status(),
                        Some(addon_event_failure_error(&failure)),
                        next_retry_at,
                    )
                    .await
                    .map(AddonEventDeliveryOutcome::Attempt)
            }
        }
    }

    async fn get_outbox_event_or_not_found(&self, event_id: EventId) -> Result<OutboxEventRecord> {
        self.store
            .get_outbox_event(event_id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "outbox_event",
                id: event_id.to_string(),
            })
    }
}

async fn run_addon_event_scheduler_loop(
    service: AddonAppService,
    config: AddonEventSchedulerConfig,
    shutdown: CancellationToken,
) {
    while !shutdown.is_cancelled() {
        let sleep_for = match service.run_addon_event_scheduler_tick(config).await {
            Ok(()) => StdDuration::from_millis(config.interval_ms.max(1)),
            Err(err) => {
                warn!(
                    error = %err,
                    "addon event scheduler tick failed"
                );
                StdDuration::from_millis(config.error_backoff_ms.max(1))
            }
        };

        tokio::select! {
            () = shutdown.cancelled() => break,
            () = tokio::time::sleep(sleep_for) => {}
        }
    }
}

#[derive(Clone, Debug)]
enum AddonEventDeliveryMode {
    Normal,
    ForcedReplay { reason_code: String },
}

impl AddonEventDeliveryMode {
    fn is_forced_replay(&self) -> bool {
        matches!(self, Self::ForcedReplay { .. })
    }

    fn replay_reason_code(&self) -> Option<&str> {
        match self {
            Self::Normal => None,
            Self::ForcedReplay { reason_code } => Some(reason_code),
        }
    }
}

fn validate_replay_reason_code(raw: &str) -> Result<String> {
    let reason_code = raw.trim();
    if reason_code.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "addon event replay reason_code must not be empty".to_owned(),
        });
    }
    if reason_code.len() > 128 {
        return Err(NakoError::InvalidInput {
            message: "addon event replay reason_code must be at most 128 bytes".to_owned(),
        });
    }
    if reason_code.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: "addon event replay reason_code must not contain control characters"
                .to_owned(),
        });
    }

    Ok(reason_code.to_owned())
}

fn addon_event_scheduler_work_status(
    event: &OutboxEventRecord,
    record: &AddonEventSchedulerWorkRecord,
    max_attempts: u32,
    subscription: Option<&AddonEventSubscriptionDeclaration>,
    granted_scopes: &[AddonScope],
    now: &str,
) -> (AddonEventSchedulerWorkStatus, Option<String>) {
    if event.status != OutboxEventStatus::Pending {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            Some("event_not_pending".to_owned()),
        );
    }
    if record.routing_plan_status != AddonRoutingPlanStatus::Executable
        || record.routing_plan_target != AddonRoutingPlanTarget::EventOutbox
    {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            record
                .routing_plan_safe_reason_code
                .clone()
                .or_else(|| Some("routing_plan_deferred".to_owned())),
        );
    }
    let Some(subscription) = subscription else {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            Some("event_subscription_not_declared".to_owned()),
        );
    };
    if subscription.event_kind != event.kind.as_str() {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            Some("event_kind_mismatch".to_owned()),
        );
    }
    if !declaration_scopes_granted(&subscription.required_scopes, granted_scopes) {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            Some("missing_grant".to_owned()),
        );
    }
    if !addon_event_subscription_filter_matches(event, &subscription.filters) {
        return (
            AddonEventSchedulerWorkStatus::Deferred,
            Some("filter_not_matched".to_owned()),
        );
    }
    if record.has_succeeded {
        return (AddonEventSchedulerWorkStatus::AlreadySucceeded, None);
    }
    if addon_event_latest_attempt_is_active_in_flight(record, now) {
        return (
            AddonEventSchedulerWorkStatus::InFlight,
            Some("delivery_in_flight".to_owned()),
        );
    }
    if record.attempt_count >= max_attempts {
        return (
            AddonEventSchedulerWorkStatus::Exhausted,
            Some("attempts_exhausted".to_owned()),
        );
    }
    if record.attempt_count == 0 {
        return (AddonEventSchedulerWorkStatus::Due, None);
    }
    if addon_event_latest_attempt_is_expired_in_flight(record, now) {
        return (AddonEventSchedulerWorkStatus::RetryDue, None);
    }
    match record.latest_next_retry_at.as_deref() {
        Some(next_retry_at) if next_retry_at > now => {
            return (
                AddonEventSchedulerWorkStatus::WaitingRetry,
                Some("retry_not_due".to_owned()),
            );
        }
        Some(_) => return (AddonEventSchedulerWorkStatus::RetryDue, None),
        None => {
            return (
                AddonEventSchedulerWorkStatus::Deferred,
                Some("retry_not_scheduled".to_owned()),
            );
        }
    }
}

fn addon_event_subscription_filter_matches(
    event: &OutboxEventRecord,
    filters: &serde_json::Value,
) -> bool {
    let Some(filters) = filters.as_object() else {
        return filters.is_null();
    };
    if filters.is_empty() {
        return true;
    }

    filters
        .iter()
        .all(|(field, expected)| addon_event_filter_field_matches(event, field, expected))
}

fn addon_event_filter_field_matches(
    event: &OutboxEventRecord,
    field: &str,
    expected: &serde_json::Value,
) -> bool {
    let actual = match field {
        "event_kind" | "kind" => Some(event.kind.as_str().to_owned()),
        "subject_kind" | "subject.kind" => Some(event.subject.kind().to_owned()),
        "subject_id" | "subject.id" => Some(event.subject.id()),
        "library_id" => event.library_id.map(|id| id.to_string()),
        "source_id" => event.source_id.map(|id| id.to_string()),
        _ => return false,
    };

    addon_event_filter_value_matches(actual.as_deref(), expected)
}

fn addon_event_filter_value_matches(actual: Option<&str>, expected: &serde_json::Value) -> bool {
    match expected {
        serde_json::Value::Null => actual.is_none(),
        serde_json::Value::String(expected) => actual == Some(expected.as_str()),
        serde_json::Value::Array(expected_values) => expected_values
            .iter()
            .any(|expected| addon_event_filter_value_matches(actual, expected)),
        serde_json::Value::Bool(expected) => {
            actual == Some(if *expected { "true" } else { "false" })
        }
        serde_json::Value::Number(expected) => {
            let expected = expected.to_string();
            actual == Some(expected.as_str())
        }
        serde_json::Value::Object(_) => false,
    }
}

fn addon_event_latest_attempt_is_active_in_flight(
    record: &AddonEventSchedulerWorkRecord,
    now: &str,
) -> bool {
    if !matches!(
        record.latest_attempt_status,
        Some(AddonEventDeliveryStatus::Pending | AddonEventDeliveryStatus::Running)
    ) {
        return false;
    }

    match record.latest_lease_expires_at.as_deref() {
        Some(lease_expires_at) => lease_expires_at > now,
        None => true,
    }
}

fn addon_event_latest_attempt_is_expired_in_flight(
    record: &AddonEventSchedulerWorkRecord,
    now: &str,
) -> bool {
    matches!(
        record.latest_attempt_status,
        Some(AddonEventDeliveryStatus::Pending | AddonEventDeliveryStatus::Running)
    ) && record
        .latest_lease_expires_at
        .as_deref()
        .is_some_and(|lease_expires_at| lease_expires_at <= now)
}

enum AddonEventDeliveryOutcome {
    Attempt(nako_core::AddonEventDeliveryAttemptRecord),
    Skipped,
}

fn addon_event_timestamp_now() -> Result<String> {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to format addon event timestamp: {err}"),
        })
}

fn addon_event_delivery_lease_expires_at() -> Result<String> {
    let expires_at =
        OffsetDateTime::now_utc() + TimeDuration::seconds(ADDON_EVENT_DELIVERY_LEASE_SECONDS);
    expires_at
        .format(&Rfc3339)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to format addon event delivery lease timestamp: {err}"),
        })
}

fn resolve_addon_event_outbound_secret(
    addon: &AddonRegistrationRecord,
    auth: AddonAuth,
) -> Result<Option<SecretString>> {
    match auth {
        AddonAuth::None => Ok(None),
        AddonAuth::Bearer | AddonAuth::SharedSecret => resolve_outbound_task_dispatch_secret(addon),
    }
}

fn addon_event_client_error_is_retryable(error: &AddonClientError) -> bool {
    match error {
        AddonClientError::Http { .. } => true,
        AddonClientError::HttpStatus { retryable, .. } => *retryable,
        AddonClientError::Protocol(_)
        | AddonClientError::InvalidRequest { .. }
        | AddonClientError::InvalidResponse { .. }
        | AddonClientError::UnsafeRequestBody => false,
    }
}

fn addon_event_failure_error(failure: &AddonEventCallFailure) -> String {
    serde_json::json!({
        "safe_error_code": safe_addon_event_client_error_code(&failure.error),
        "attempts": failure.attempts,
        "error_kind": failure.error.kind(),
        "http_status": failure.error.http_status(),
        "retryable": addon_event_client_error_is_retryable(&failure.error),
    })
    .to_string()
}

fn safe_addon_event_client_error_code(error: &AddonClientError) -> &'static str {
    match error {
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::MissingAuthToken {
            ..
        }) => "authorization_gap",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::EventSubscriptionNotDeclared { .. },
        ) => "event_subscription_not_declared",
        AddonClientError::Protocol(
            nako_addon_protocol::AddonManifestError::MissingDeclaredScopeForDeclaration { .. },
        ) => "missing_grant",
        AddonClientError::Protocol(nako_addon_protocol::AddonManifestError::InvalidEnvelope {
            ..
        }) => "unsafe_response",
        AddonClientError::Protocol(_) => "protocol_mismatch",
        AddonClientError::InvalidRequest { .. } => "invalid_request",
        AddonClientError::InvalidResponse { .. } => "invalid_response",
        AddonClientError::UnsafeRequestBody => "unsafe_request_body",
        AddonClientError::HttpStatus {
            retryable: true, ..
        } => "retryable_http_failure",
        AddonClientError::HttpStatus {
            retryable: false, ..
        } => "http_failure",
        AddonClientError::Http { .. } => "sidecar_unreachable",
    }
}

fn next_addon_event_retry_at(attempt_number: u32, max_attempts: u32) -> Result<Option<String>> {
    if attempt_number >= max_attempts {
        return Ok(None);
    }

    let exponent = attempt_number.saturating_sub(1).min(6);
    let delay_seconds = 30_i64 * 2_i64.pow(exponent);
    let retry_at = OffsetDateTime::now_utc() + TimeDuration::seconds(delay_seconds);
    retry_at
        .format(&Rfc3339)
        .map(Some)
        .map_err(|err| NakoError::InvalidInput {
            message: format!("failed to format addon event retry timestamp: {err}"),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addon_event_failure_error_is_redaction_safe_json() {
        let error = addon_event_failure_error(&AddonEventCallFailure {
            error: AddonClientError::HttpStatus {
                status: 503,
                retryable: true,
            },
            attempts: 1,
        });

        assert!(error.contains("retryable_http_failure"));
        assert!(error.contains("503"));
        assert!(!error.contains("Bearer"));
        assert!(!error.contains("nako_at_"));
    }
}
