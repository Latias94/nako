use nako_addon_client::{
    AddonClientError, AddonEventCallFailure, AddonEventCallRequest, ReqwestAddonTransport,
    call_addon_event_with_outcome,
};
use nako_addon_protocol::{
    AddonAuth, AddonEventSubscriptionDeclaration, AddonScope, validate_manifest,
};
use nako_api::extension::{
    AddonEventDeliveryAttemptsResponse, AddonEventDispatchEventSummary, AddonEventDispatchResponse,
};
use nako_core::{
    AddonEventDeliveryAttemptId, AddonEventDeliveryRepository, AddonEventDeliveryStatus,
    AddonRegistrationRecord, AddonRepository, AddonRoutingDeclarationKind, AddonRoutingPlanStatus,
    AddonRoutingPlanTarget, AddonStatus, EventId, EventOutboxRepository, NakoError,
    NewAddonEventDeliveryAttempt, OutboxEventRecord, Result, SecretString,
};
use time::{Duration as TimeDuration, OffsetDateTime, format_description::well_known::Rfc3339};
use tokio::task::JoinSet;
use tracing::warn;

use super::{
    AddonAppService, declaration_scopes_granted, resolve_outbound_task_dispatch_secret,
    stored_granted_scopes,
};

impl AddonAppService {
    pub async fn list_addon_event_delivery_attempts(
        &self,
        event_id: EventId,
    ) -> Result<AddonEventDeliveryAttemptsResponse> {
        self.get_outbox_event_or_not_found(event_id).await?;
        let attempts = self
            .store
            .list_addon_event_delivery_attempts(event_id)
            .await?;

        Ok(AddonEventDeliveryAttemptsResponse { event_id, attempts })
    }

    pub async fn deliver_addon_events_for_event(
        &self,
        event_id: EventId,
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
        let mut errors = Vec::new();

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

                let permit = self.permits.clone().acquire_owned().await.map_err(|err| {
                    NakoError::Provider {
                        provider: "addon_event".to_owned(),
                        message: format!("addon event resource budget was closed: {err}"),
                    }
                })?;
                attempted_subscriptions += 1;
                let service = self.clone();
                let event = event.clone();
                let addon = addon.clone();
                let granted_scopes = granted_scopes.clone();
                workers.spawn(async move {
                    let _permit = permit;
                    service
                        .deliver_addon_event_subscription(
                            event,
                            addon,
                            subscription,
                            granted_scopes,
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
                    AddonEventDeliveryOutcome::AlreadySucceeded => skipped_subscriptions += 1,
                },
                Ok(Err(err)) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "addon event delivery failed before attempt completion"
                    );
                    errors.push(err.to_string());
                }
                Err(err) => {
                    failed += 1;
                    warn!(
                        event_id = %event.id,
                        error = %err,
                        "addon event delivery worker join failed"
                    );
                    errors.push(format!("addon event delivery worker join failed: {err}"));
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
            attempts,
            errors,
        })
    }

    async fn deliver_addon_event_subscription(
        &self,
        event: OutboxEventRecord,
        addon: AddonRegistrationRecord,
        subscription: AddonEventSubscriptionDeclaration,
        granted_scopes: Vec<AddonScope>,
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

        let max_attempts = manifest.default_max_attempts.unwrap_or(3);
        let existing = self
            .store
            .list_addon_event_delivery_attempts_for_addon(addon.id, event.id, &subscription.id)
            .await?;
        if existing
            .iter()
            .any(|attempt| attempt.status == AddonEventDeliveryStatus::Succeeded)
        {
            return Ok(AddonEventDeliveryOutcome::AlreadySucceeded);
        }
        let attempt_number = existing
            .iter()
            .map(|attempt| attempt.attempt_number)
            .max()
            .unwrap_or(0)
            + 1;
        if attempt_number > max_attempts {
            return Err(NakoError::Conflict {
                message: format!(
                    "addon {} event subscription {} exhausted attempts for event {}",
                    addon.id, subscription.id, event.id
                ),
            });
        }

        let attempt = self
            .store
            .create_addon_event_delivery_attempt(NewAddonEventDeliveryAttempt {
                id: AddonEventDeliveryAttemptId::new(),
                addon_id: addon.id,
                event_id: event.id,
                declaration_id: subscription.id.clone(),
                attempt_number,
            })
            .await?;
        let payload =
            serde_json::from_str::<serde_json::Value>(&event.payload_json).map_err(|err| {
                NakoError::InvalidInput {
                    message: format!("failed to parse outbox event payload JSON: {err}"),
                }
            })?;
        let outbound_secret = resolve_addon_event_outbound_secret(&addon, manifest.auth)?;
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
                attempt: attempt_number,
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
                    next_addon_event_retry_at(attempt_number, max_attempts)?
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

enum AddonEventDeliveryOutcome {
    Attempt(nako_core::AddonEventDeliveryAttemptRecord),
    AlreadySucceeded,
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
