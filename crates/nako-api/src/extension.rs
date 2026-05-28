use std::collections::BTreeMap;

use nako_addon_protocol::{
    AddonConfigurationSchema, AddonEntryPointKind, AddonHealthStatus, AddonInstallDescriptor,
    AddonInstallGuide, AddonManifest, AddonResource, AddonResourceLinkCheckStatus,
    AddonResourceLinkType, AddonResourceSearchIntent, AddonResourceSearchProviderExecution,
    AddonResourceSearchProviderFinality, AddonResourceSearchProviderStatus, AddonRuntimeKind,
    AddonScope, AddonSubtitleFormat, AddonSubtitleProviderExecution, AddonSubtitleProviderStatus,
    AddonTaskDeclaration,
};
use nako_core::{
    ADDON_TASK_RUN_PROGRESS_SCHEMA, ADDON_TASK_RUN_RESULT_SCHEMA, AddonEventDeliveryAttemptRecord,
    AddonEventDeliveryStatus, AddonGrantRecord, AddonId, AddonPermission, AddonRegistrationRecord,
    AddonRoutingDeclarationKind, AddonRoutingPlanRecord, AddonRoutingPlanStatus,
    AddonRoutingPlanTarget, AddonSideEffectApplyStatus, AddonSideEffectId, AddonSideEffectRecord,
    AddonSideEffectTarget, AddonSideEffectTargetKind, AddonSideEffectValidationStatus, AddonStatus,
    AddonTaskRunLeaseGuard, AddonTaskRunRecord, AddonTokenId, AddonTokenRecord, AddonTokenStatus,
    AutomationArtifactId, AutomationArtifactKind, AutomationArtifactRecord,
    AutomationArtifactStatus, AutomationCapability, AutomationJobInput,
    AutomationProviderConfigRecord, AutomationProviderId, AutomationProviderStatus,
    DomainEventKind, DomainEventSubject, EventId, JobId, JobKind, JobStatus, JobWorkerId,
    LibraryId, MediaItemId, MediaSourceId, OutboxEventRecord, OutboxEventStatus,
    WebhookDeliveryAttemptRecord, WebhookEndpointId, WebhookEndpointRecord, WebhookEndpointStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpsertWebhookEndpointRequest {
    pub id: Option<WebhookEndpointId>,
    pub name: String,
    pub url: String,
    pub secret_env: Option<String>,
    pub subscribed_event_kinds: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub max_attempts: Option<u32>,
    pub status: WebhookEndpointStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookEndpointResponse {
    pub endpoint: WebhookEndpointRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookEndpointsResponse {
    pub endpoints: Vec<WebhookEndpointRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDeliveryAttemptsResponse {
    pub event_id: EventId,
    pub attempts: Vec<WebhookDeliveryAttemptRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct WebhookDispatchResponse {
    pub event: OutboxEventRecord,
    pub attempted_endpoints: u32,
    pub delivered: u32,
    pub failed: u32,
    pub skipped_endpoints: u32,
    pub attempts: Vec<WebhookDeliveryAttemptRecord>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventDeliveryAttemptsResponse {
    pub event_id: EventId,
    pub attempts: Vec<AddonEventDeliveryAttemptRecord>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonEventSchedulerWorkStatus {
    Due,
    RetryDue,
    WaitingRetry,
    AlreadySucceeded,
    Exhausted,
    InFlight,
    Deferred,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventDispatchEventSummary {
    pub id: EventId,
    pub kind: DomainEventKind,
    pub subject: DomainEventSubject,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_id: Option<LibraryId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<MediaSourceId>,
    pub status: OutboxEventStatus,
    pub attempts: u32,
    pub occurred_at: String,
    pub updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_attempt_at: Option<String>,
}

impl AddonEventDispatchEventSummary {
    #[must_use]
    pub fn from_record(record: &OutboxEventRecord) -> Self {
        Self {
            id: record.id,
            kind: record.kind,
            subject: record.subject,
            library_id: record.library_id,
            source_id: record.source_id,
            status: record.status,
            attempts: record.attempts,
            occurred_at: record.occurred_at.clone(),
            updated_at: record.updated_at.clone(),
            next_attempt_at: record.next_attempt_at.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventDispatchResponse {
    pub event: AddonEventDispatchEventSummary,
    pub attempted_subscriptions: u32,
    pub delivered: u32,
    pub failed: u32,
    pub skipped_subscriptions: u32,
    pub attempts: Vec<AddonEventDeliveryAttemptRecord>,
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReplayAddonEventRequest {
    pub reason_code: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventReplayResponse {
    pub reason_code: String,
    pub dispatch: AddonEventDispatchResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventSchedulerWorkItem {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub manifest_version: String,
    pub declaration_id: String,
    pub event_kind: String,
    pub status: AddonEventSchedulerWorkStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_reason_code: Option<String>,
    pub routing_plan_status: AddonRoutingPlanStatus,
    pub routing_plan_target: AddonRoutingPlanTarget,
    pub attempt_count: u32,
    pub next_attempt_number: u32,
    pub max_attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_attempt_status: Option<AddonEventDeliveryStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_retry_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lease_expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonEventSchedulerWorkResponse {
    pub event: AddonEventDispatchEventSummary,
    pub due_work_count: usize,
    pub blocked_work_count: usize,
    pub work: Vec<AddonEventSchedulerWorkItem>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpsertAutomationProviderRequest {
    pub id: Option<AutomationProviderId>,
    pub name: String,
    pub base_url: String,
    pub secret_env: Option<String>,
    pub capabilities: Vec<AutomationCapability>,
    pub timeout_ms: Option<u64>,
    pub max_attempts: Option<u32>,
    pub status: AutomationProviderStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProviderResponse {
    pub provider: AutomationProviderConfigRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationProvidersResponse {
    pub providers: Vec<AutomationProviderConfigRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnqueueAutomationJobRequest {
    pub provider_id: AutomationProviderId,
    pub capability: AutomationCapability,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub prompt: serde_json::Value,
    pub idempotency_key: String,
}

impl EnqueueAutomationJobRequest {
    pub fn into_job_input(self) -> Result<AutomationJobInput, serde_json::Error> {
        Ok(AutomationJobInput {
            provider_id: self.provider_id,
            capability: self.capability,
            library_id: self.library_id,
            item_id: self.item_id,
            source_id: self.source_id,
            prompt_json: serde_json::to_string(&self.prompt)?,
            idempotency_key: self.idempotency_key,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AutomationArtifactsResponse {
    pub artifacts: Vec<AutomationArtifactRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RegisterAddonRequest {
    pub id: Option<AddonId>,
    pub manifest: AddonManifest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outbound_task_dispatch_secret_env: Option<String>,
    #[serde(default)]
    pub granted_scopes: Vec<AddonScope>,
    #[serde(default)]
    pub status: Option<AddonStatus>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRegistrationSummary {
    pub id: AddonId,
    pub manifest_id: String,
    pub name: String,
    pub version: String,
    pub protocol_version: String,
    pub base_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outbound_task_dispatch_secret_env: Option<String>,
    pub granted_scopes: Vec<String>,
    pub status: AddonStatus,
    pub created_at: String,
    pub updated_at: String,
}

impl AdminAddonRegistrationSummary {
    #[must_use]
    pub fn from_record(record: &AddonRegistrationRecord) -> Self {
        Self {
            id: record.id,
            manifest_id: record.manifest_id.clone(),
            name: record.name.clone(),
            version: record.version.clone(),
            protocol_version: record.protocol_version.clone(),
            base_url: record.base_url.clone(),
            outbound_task_dispatch_secret_env: record.outbound_task_dispatch_secret_env.clone(),
            granted_scopes: record.granted_scopes.clone(),
            status: record.status,
            created_at: record.created_at.clone(),
            updated_at: record.updated_at.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRegistrationDetail {
    pub summary: AdminAddonRegistrationSummary,
    pub manifest: AddonManifest,
}

impl AdminAddonRegistrationDetail {
    pub fn from_record(record: &AddonRegistrationRecord) -> Result<Self, serde_json::Error> {
        Ok(Self {
            summary: AdminAddonRegistrationSummary::from_record(record),
            manifest: serde_json::from_str(&record.manifest_json)?,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRegistrationResponse {
    pub addon: AdminAddonRegistrationDetail,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonLifecycleIntent {
    Install,
    Update,
    Remove,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonManagerPlanRequest {
    pub intent: AdminAddonLifecycleIntent,
    pub operator_confirmed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonManagerPlanResponse {
    pub addon_id: AddonId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<AdminAddonLifecycleIntent>,
    pub operator_confirmed: bool,
    pub source: AdminAddonRegistrationDetail,
    pub health_check: AdminAddonHealthCheckResponse,
    pub tokens: AddonTokensResponse,
    pub grants: AddonGrantsResponse,
    pub install_guide: AdminAddonInstallGuideResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRegistrationsResponse {
    pub addons: Vec<AdminAddonRegistrationSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UpdateAddonStatusRequest {
    pub status: AddonStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuidePreviewRequest {
    pub descriptor: AddonInstallDescriptor,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuidePreviewResponse {
    pub guide: AddonInstallGuide,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonSourceCatalogSourceKind {
    BuiltinOfficial,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSourceCatalogSourcesResponse {
    pub sources: Vec<AdminAddonSourceCatalogSource>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSourceCatalogSource {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub kind: AdminAddonSourceCatalogSourceKind,
    pub entry_count: usize,
    pub provides_package_signing: bool,
    pub provides_process_supervision: bool,
    pub provides_provider_breadth: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSourceCatalogEntriesResponse {
    pub source_id: String,
    pub entries: Vec<AdminAddonSourceCatalogEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSourceCatalogEntry {
    pub source_id: String,
    pub entry_id: String,
    pub manifest_id: String,
    pub addon_name: String,
    pub addon_version: String,
    pub protocol_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub runtime_kind: AddonRuntimeKind,
    pub resources: Vec<AddonResource>,
    pub scopes: Vec<AddonScope>,
    pub tasks: Vec<String>,
    pub package_signing_verified: bool,
    pub lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSourceCatalogResolveResponse {
    pub source_id: String,
    pub entry: AdminAddonSourceCatalogEntry,
    pub descriptor: AddonInstallDescriptor,
    pub install_guide: AddonInstallGuide,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonHealthCheckStatus {
    Reachable,
    Degraded,
    Unhealthy,
    Unreachable,
    ProtocolMismatch,
    InvalidManifest,
}

impl From<AddonHealthStatus> for AdminAddonHealthCheckStatus {
    fn from(value: AddonHealthStatus) -> Self {
        match value {
            AddonHealthStatus::Ok => Self::Reachable,
            AddonHealthStatus::Degraded => Self::Degraded,
            AddonHealthStatus::Unhealthy => Self::Unhealthy,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonHealthCheckResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub status: AdminAddonHealthCheckStatus,
    pub latency_ms: u128,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addon_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_count: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol_checked_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRuntimeReadinessResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub readiness: AdminAddonRuntimeReadinessDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRuntimeReadinessDiagnostics {
    pub status: AdminAddonRuntimeReadinessStatus,
    pub reason: AdminAddonRuntimeReadinessReason,
    pub checks: Vec<AdminAddonRuntimeReadinessCheck>,
}

impl AdminAddonRuntimeReadinessDiagnostics {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminAddonRuntimeReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminAddonRuntimeReadinessStatus::Unavailable)
        {
            AdminAddonRuntimeReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminAddonRuntimeReadinessStatus::Degraded)
        {
            AdminAddonRuntimeReadinessStatus::Degraded
        } else {
            AdminAddonRuntimeReadinessStatus::Ready
        };
        let reason = checks
            .iter()
            .find(|check| check.status == status)
            .map_or(AdminAddonRuntimeReadinessReason::Ready, |check| {
                check.reason
            });

        Self {
            status,
            reason,
            checks,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonRuntimeReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonRuntimeReadinessReason {
    Ready,
    Unavailable,
    ManifestMismatch,
    ProtocolMismatch,
    MissingGrant,
    MissingSecretReference,
    NetworkPolicyBlocked,
    SidecarDegraded,
    SidecarUnhealthy,
    UnsafeResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRuntimeReadinessCheck {
    pub name: AdminAddonRuntimeReadinessCheckName,
    pub status: AdminAddonRuntimeReadinessStatus,
    pub reason: AdminAddonRuntimeReadinessReason,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

impl AdminAddonRuntimeReadinessCheck {
    #[must_use]
    pub fn ready(
        name: AdminAddonRuntimeReadinessCheckName,
        reason: AdminAddonRuntimeReadinessReason,
    ) -> Self {
        Self {
            name,
            status: AdminAddonRuntimeReadinessStatus::Ready,
            reason,
            safe_error_code: None,
        }
    }

    #[must_use]
    pub fn degraded(
        name: AdminAddonRuntimeReadinessCheckName,
        reason: AdminAddonRuntimeReadinessReason,
        safe_error_code: &'static str,
    ) -> Self {
        Self {
            name,
            status: AdminAddonRuntimeReadinessStatus::Degraded,
            reason,
            safe_error_code: Some(safe_error_code.to_owned()),
        }
    }

    #[must_use]
    pub fn unavailable(
        name: AdminAddonRuntimeReadinessCheckName,
        reason: AdminAddonRuntimeReadinessReason,
        safe_error_code: &'static str,
    ) -> Self {
        Self {
            name,
            status: AdminAddonRuntimeReadinessStatus::Unavailable,
            reason,
            safe_error_code: Some(safe_error_code.to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonRuntimeReadinessCheckName {
    Reachability,
    Protocol,
    Manifest,
    Grants,
    SecretReferences,
    Network,
    Safety,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSurfacesResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub entry_points: Vec<AdminAddonEntryPointSurface>,
    pub hosted_pages: Vec<AdminAddonHostedPageSurface>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration_schema: Option<AdminAddonConfigurationSchemaSurface>,
    pub secret_reference_fields: Vec<AdminAddonSecretReferenceFieldSurface>,
    pub tasks: Vec<AdminAddonTaskSurface>,
    pub event_subscriptions: Vec<AdminAddonEventSubscriptionSurface>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRoutingPlansResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub manifest_version: String,
    pub manifest_fingerprint: String,
    pub executable: usize,
    pub deferred: usize,
    pub plans: Vec<AdminAddonRoutingPlan>,
}

impl AdminAddonRoutingPlansResponse {
    #[must_use]
    pub fn from_records(
        addon_id: AddonId,
        manifest_id: String,
        manifest_version: String,
        manifest_fingerprint: String,
        records: Vec<AddonRoutingPlanRecord>,
    ) -> Self {
        let executable = records
            .iter()
            .filter(|plan| plan.status == AddonRoutingPlanStatus::Executable)
            .count();
        let deferred = records
            .iter()
            .filter(|plan| plan.status == AddonRoutingPlanStatus::Deferred)
            .count();

        Self {
            addon_id,
            manifest_id,
            manifest_version,
            manifest_fingerprint,
            executable,
            deferred,
            plans: records
                .into_iter()
                .map(AdminAddonRoutingPlan::from_record)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonRoutingPlan {
    pub declaration_kind: AddonRoutingDeclarationKind,
    pub declaration_id: String,
    pub status: AddonRoutingPlanStatus,
    pub target: AddonRoutingPlanTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_reason_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_kind: Option<JobKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_kind: Option<String>,
    pub required_scope_count: usize,
    pub filter_configured: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
}

impl AdminAddonRoutingPlan {
    #[must_use]
    pub fn from_record(record: AddonRoutingPlanRecord) -> Self {
        #[derive(Deserialize)]
        struct RoutingPlanSummary {
            #[serde(default)]
            required_scope_count: usize,
            #[serde(default)]
            filter_configured: bool,
            #[serde(default)]
            timeout_ms: Option<u64>,
            #[serde(default)]
            max_attempts: Option<u32>,
        }

        let summary = serde_json::from_str::<RoutingPlanSummary>(&record.plan_json).ok();

        Self {
            declaration_kind: record.declaration_kind,
            declaration_id: record.declaration_id,
            status: record.status,
            target: record.target,
            safe_reason_code: record.safe_reason_code,
            job_kind: record.job_kind,
            event_kind: record.event_kind,
            required_scope_count: summary
                .as_ref()
                .map_or(0, |summary| summary.required_scope_count),
            filter_configured: summary
                .as_ref()
                .is_some_and(|summary| summary.filter_configured),
            timeout_ms: summary.as_ref().and_then(|summary| summary.timeout_ms),
            max_attempts: summary.as_ref().and_then(|summary| summary.max_attempts),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonEntryPointSurface {
    pub id: String,
    pub kind: AddonEntryPointKind,
    pub label: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hosted_page_id: Option<String>,
    pub required_scopes: Vec<AddonScope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonHostedPageSurface {
    pub id: String,
    pub title: String,
    pub path: String,
    pub url: String,
    pub required_scopes: Vec<AddonScope>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonConfigurationSchemaSurface {
    pub schema_id: String,
    pub schema: serde_json::Value,
}

impl From<AddonConfigurationSchema> for AdminAddonConfigurationSchemaSurface {
    fn from(value: AddonConfigurationSchema) -> Self {
        Self {
            schema_id: value.schema_id,
            schema: value.schema,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSecretReferenceFieldSurface {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonTaskSurface {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required_scopes: Vec<AddonScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_attempts: Option<u32>,
}

impl From<AddonTaskDeclaration> for AdminAddonTaskSurface {
    fn from(value: AddonTaskDeclaration) -> Self {
        Self {
            id: value.id,
            name: value.name,
            path: value.path,
            description: value.description,
            required_scopes: value.required_scopes,
            timeout_ms: value.timeout_ms,
            max_attempts: value.max_attempts,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonEventSubscriptionSurface {
    pub id: String,
    pub event_kind: String,
    pub path: String,
    pub required_scopes: Vec<AddonScope>,
    pub filters: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceCallDiagnosticRequest {
    pub resource: AddonResource,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonResourceCallDiagnosticStatus {
    Succeeded,
    MissingResource,
    MissingGrant,
    AuthorizationGap,
    Unreachable,
    ProtocolMismatch,
    RetryableHttpFailure,
    HttpFailure,
    UnsafeResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceCallDiagnosticResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub resource: AddonResource,
    pub status: AdminAddonResourceCallDiagnosticStatus,
    pub latency_ms: u128,
    pub attempts: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchDiagnosticRequest {
    pub query: String,
    pub intent: AddonResourceSearchIntent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_types: Vec<AddonResourceLinkType>,
    #[serde(default)]
    pub refresh: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub context: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchProviderDiagnostic {
    pub provider_id: String,
    pub status: AddonResourceSearchProviderStatus,
    pub result_count: usize,
    pub finality: AddonResourceSearchProviderFinality,
    pub has_safe_message: bool,
}

impl From<AddonResourceSearchProviderExecution> for AdminAddonResourceSearchProviderDiagnostic {
    fn from(value: AddonResourceSearchProviderExecution) -> Self {
        Self {
            provider_id: value.provider_id,
            status: value.status,
            result_count: value.result_count,
            finality: value.finality,
            has_safe_message: value.safe_message.is_some(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchDiagnosticResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub status: AdminAddonResourceCallDiagnosticStatus,
    pub latency_ms: u128,
    pub attempts: u32,
    pub limit: usize,
    pub total: usize,
    pub result_count: usize,
    pub link_count: usize,
    pub merged_link_count: usize,
    pub provider_executions: Vec<AdminAddonResourceSearchProviderDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchRequest {
    pub query: String,
    pub intent: AddonResourceSearchIntent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub link_types: Vec<AddonResourceLinkType>,
    #[serde(default)]
    pub refresh: bool,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub context: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchResultSummary {
    pub result_ref_fingerprint: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    pub score: u16,
    pub links: Vec<AdminAddonResourceSearchLinkSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchLinkSummary {
    pub selection_id: String,
    pub link_type: AddonResourceLinkType,
    pub source: String,
    pub source_ref_redacted: String,
    pub has_password: bool,
    pub has_note: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub search_id: String,
    pub status: AdminAddonResourceCallDiagnosticStatus,
    pub latency_ms: u128,
    pub attempts: u32,
    pub limit: usize,
    pub total: usize,
    pub result_count: usize,
    pub results: Vec<AdminAddonResourceSearchResultSummary>,
    pub provider_executions: Vec<AdminAddonResourceSearchProviderDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchSelectionRequest {
    pub target_library_id: LibraryId,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceSearchSelectionResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub search_id: String,
    pub selection_id: String,
    pub candidate: AddonAcquisitionCandidateSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleSearchRequest {
    pub query: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub languages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleProviderDiagnostic {
    pub provider_id: String,
    pub status: AddonSubtitleProviderStatus,
    pub result_count: usize,
    pub has_safe_message: bool,
}

impl From<AddonSubtitleProviderExecution> for AdminAddonSubtitleProviderDiagnostic {
    fn from(value: AddonSubtitleProviderExecution) -> Self {
        Self {
            provider_id: value.provider_id,
            status: value.status,
            result_count: value.result_count,
            has_safe_message: value.safe_message.is_some(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAddonSubtitleDeliveryKind {
    Inline,
    DownloadUrl,
    ArtifactRef,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleCandidateSummary {
    pub selection_id: String,
    pub candidate_ref_fingerprint: String,
    pub title: String,
    pub language: String,
    pub format: AddonSubtitleFormat,
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    pub score: u16,
    pub delivery_kind: AdminAddonSubtitleDeliveryKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleSearchResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub search_id: String,
    pub status: AdminAddonResourceCallDiagnosticStatus,
    pub latency_ms: u128,
    pub attempts: u32,
    pub limit: usize,
    pub total: usize,
    pub result_count: usize,
    pub subtitles: Vec<AdminAddonSubtitleCandidateSummary>,
    pub provider_executions: Vec<AdminAddonSubtitleProviderDiagnostic>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAddonSubtitleSelectionRequest {}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleSelectedReference {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub search_id: String,
    pub selection_id: String,
    pub candidate_ref_fingerprint: String,
    pub delivery_kind: AdminAddonSubtitleDeliveryKind,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleSelectionResponse {
    pub selected_ref: AdminAddonSubtitleSelectedReference,
    pub candidate: AdminAddonSubtitleCandidateSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSubtitleSidecarRole {
    Default,
    Forced,
    Sdh,
    Commentary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSubtitleImportConflictPolicy {
    CreateMissing,
    ReplaceExisting,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSubtitleImportBackupPolicy {
    None,
    ExistingFileKeepLatest,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSubtitleImportPlanStatus {
    Ready,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminSubtitleImportPlanReason {
    Ready,
    MediaSourceMatchesItem,
    CandidateLanguageMismatch,
    CandidateFormatMismatch,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAddonSubtitleImportPlanRequest {
    pub media_item_id: MediaItemId,
    pub media_source_id: MediaSourceId,
    pub language: String,
    pub format: AddonSubtitleFormat,
    pub sidecar_role: AdminSubtitleSidecarRole,
    pub conflict_policy: AdminSubtitleImportConflictPolicy,
    pub backup_policy: AdminSubtitleImportBackupPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSubtitleImportTargetSummary {
    pub library_id: LibraryId,
    pub media_item_id: MediaItemId,
    pub media_source_id: MediaSourceId,
    pub item_title: String,
    pub media_file_name: String,
    pub source_ref_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSubtitleSidecarPlan {
    pub file_name: String,
    pub language: String,
    pub format: AddonSubtitleFormat,
    pub role: AdminSubtitleSidecarRole,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSubtitleImportPlan {
    pub idempotency_key: String,
    pub status: AdminSubtitleImportPlanStatus,
    pub reasons: Vec<AdminSubtitleImportPlanReason>,
    pub target: AdminSubtitleImportTargetSummary,
    pub sidecar: AdminSubtitleSidecarPlan,
    pub conflict_policy: AdminSubtitleImportConflictPolicy,
    pub backup_policy: AdminSubtitleImportBackupPolicy,
    pub preview_only: bool,
    pub writes_library: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonSubtitleImportPlanResponse {
    pub selected_ref: AdminAddonSubtitleSelectedReference,
    pub candidate: AdminAddonSubtitleCandidateSummary,
    pub plan: AdminSubtitleImportPlan,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdminAddonResourceLinkCheckRequest {
    #[serde(default)]
    pub refresh: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonResourceLinkCheckResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub search_id: String,
    pub selection_id: String,
    pub status: AdminAddonResourceCallDiagnosticStatus,
    pub latency_ms: u128,
    pub attempts: u32,
    pub link_type: AddonResourceLinkType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_status: Option<AddonResourceLinkCheckStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checked_at_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requires_password: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    pub has_safe_message: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub safe_facts: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub safe_error_code: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuideResponse {
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub addon_name: String,
    pub addon_version: String,
    pub protocol_version: String,
    pub base_url: String,
    pub status: AddonStatus,
    pub docker_compose: AdminAddonInstallGuideSnippet,
    pub systemd: AdminAddonInstallGuideSnippet,
    pub secret_references: Vec<AdminAddonInstallGuideSecretReference>,
    pub health_check_steps: Vec<AdminAddonInstallGuideStep>,
    pub registration_verification_steps: Vec<AdminAddonInstallGuideStep>,
    pub lifecycle_boundary: AdminAddonInstallGuideLifecycleBoundary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuideSnippet {
    pub title: String,
    pub filename: String,
    pub content: String,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuideSecretReference {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub env_var: String,
    pub placeholder: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuideStep {
    pub title: String,
    pub command: String,
    pub expected_result: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAddonInstallGuideLifecycleBoundary {
    pub nako_manages_containers: bool,
    pub nako_manages_processes: bool,
    pub nako_manages_packages: bool,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct IssueAddonTokenRequest {
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokenSummary {
    pub id: AddonTokenId,
    pub addon_id: AddonId,
    pub label: String,
    pub token_prefix: String,
    pub status: AddonTokenStatus,
    pub created_at: String,
    pub rotated_at: Option<String>,
    pub revoked_at: Option<String>,
    pub last_used_at: Option<String>,
}

impl AddonTokenSummary {
    #[must_use]
    pub fn from_record(record: AddonTokenRecord) -> Self {
        Self {
            id: record.id,
            addon_id: record.addon_id,
            label: record.label,
            token_prefix: record.token_prefix,
            status: record.status,
            created_at: record.created_at,
            rotated_at: record.rotated_at,
            revoked_at: record.revoked_at,
            last_used_at: record.last_used_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokenResponse {
    pub token: AddonTokenSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokensResponse {
    pub tokens: Vec<AddonTokenSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokenIssuedResponse {
    pub token: AddonTokenSummary,
    pub raw_token: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTokenRotationResponse {
    pub rotated: AddonTokenSummary,
    pub token: AddonTokenSummary,
    pub raw_token: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReplaceAddonGrantsRequest {
    #[serde(default)]
    pub grants: Vec<AddonGrantAssignment>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct AddonGrantAssignment {
    pub permission: AddonPermission,
    #[serde(default)]
    pub library_id: Option<LibraryId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonGrantsResponse {
    pub grants: Vec<AddonGrantRecord>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAccessCheckRequest {
    pub permission: AddonPermission,
    #[serde(default)]
    pub library_id: Option<LibraryId>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAccessCheckResponse {
    pub addon_id: AddonId,
    pub token_id: AddonTokenId,
    pub permission: AddonPermission,
    pub library_id: Option<LibraryId>,
    pub allowed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateAddonTaskRunRequest {
    pub declaration_id: String,
    pub idempotency_key: String,
    #[serde(default)]
    pub dispatch: AddonTaskRunDispatchMode,
    #[serde(default)]
    pub library_id: Option<LibraryId>,
    #[serde(default)]
    pub source_id: Option<MediaSourceId>,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AddonTaskRunDispatchMode {
    #[default]
    SidecarClaim,
    Direct,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RetryAddonTaskRunRequest {
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunResponse {
    pub run: AddonTaskRunSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunsResponse {
    pub runs: Vec<AddonTaskRunSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClaimAddonTaskRunRequest {
    pub worker_id: Option<JobWorkerId>,
    #[serde(default)]
    pub declaration_id: Option<String>,
    #[serde(default = "default_addon_task_run_lease_duration_ms")]
    pub lease_duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ClaimAddonTaskRunResponse {
    pub run: Option<AddonTaskRunLease>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunLease {
    pub run: AddonTaskRunSummary,
    pub guard: AddonTaskRunLeaseGuard,
    pub lease_expires_at: String,
    pub cancel_requested_at: Option<String>,
    pub input: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReportAddonTaskRunProgressRequest {
    pub guard: AddonTaskRunLeaseGuard,
    #[serde(default = "default_addon_task_run_lease_duration_ms")]
    pub lease_duration_ms: u64,
    pub stage: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percent: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default)]
    pub metrics: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompleteAddonTaskRunRequest {
    pub guard: AddonTaskRunLeaseGuard,
    #[serde(default)]
    pub output: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FailAddonTaskRunRequest {
    pub guard: AddonTaskRunLeaseGuard,
    pub safe_error_code: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    #[serde(default)]
    pub output: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CancelAddonTaskRunRequest {
    pub guard: AddonTaskRunLeaseGuard,
    #[serde(default)]
    pub output: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonTaskRunSummary {
    pub job_id: JobId,
    pub addon_id: AddonId,
    pub manifest_id: String,
    pub manifest_version: String,
    pub manifest_fingerprint: String,
    pub declaration_id: String,
    pub declaration_name: String,
    pub declaration_path: String,
    pub status: JobStatus,
    pub resource_class: String,
    pub library_id: Option<LibraryId>,
    pub source_id: Option<MediaSourceId>,
    pub attempt: u32,
    pub max_attempts: Option<u32>,
    pub retry_of_job_id: Option<JobId>,
    pub retryable: bool,
    pub has_input: bool,
    pub progress: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub safe_error_code: Option<String>,
    pub cancel_requested_at: Option<String>,
    pub queued_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

impl AddonTaskRunSummary {
    #[must_use]
    pub fn from_record(record: AddonTaskRunRecord) -> Self {
        let retryable = record.job.status == JobStatus::Failed
            && record
                .max_attempts
                .is_none_or(|max_attempts| record.attempt < max_attempts);

        Self {
            job_id: record.job.id,
            addon_id: record.addon_id,
            manifest_id: record.manifest_id,
            manifest_version: record.manifest_version,
            manifest_fingerprint: record.manifest_fingerprint.to_string(),
            declaration_id: record.declaration_id,
            declaration_name: record.declaration_name,
            declaration_path: record.declaration_path,
            status: record.job.status,
            resource_class: record.job.resource_class,
            library_id: record.job.library_id,
            source_id: record.job.source_id,
            attempt: record.attempt,
            max_attempts: record.max_attempts,
            retry_of_job_id: record.retry_of_job_id,
            retryable,
            has_input: !record.input_json.trim().is_empty(),
            progress: record
                .progress_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            result: record
                .result_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            safe_error_code: record.safe_error_code,
            cancel_requested_at: record.cancel_requested_at,
            queued_at: record.job.queued_at,
            started_at: record.job.started_at,
            completed_at: record.job.completed_at,
            updated_at: record.updated_at,
        }
    }
}

pub fn addon_task_run_progress_json(
    stage: String,
    percent: Option<u8>,
    message: Option<String>,
    metrics: serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "schema": ADDON_TASK_RUN_PROGRESS_SCHEMA,
        "stage": stage,
        "percent": percent,
        "message": message,
        "metrics": metrics,
    })
}

pub fn addon_task_run_result_json(
    status: &'static str,
    output: serde_json::Value,
    safe_error_code: Option<&str>,
    retry_after_ms: Option<u64>,
) -> serde_json::Value {
    serde_json::json!({
        "schema": ADDON_TASK_RUN_RESULT_SCHEMA,
        "status": status,
        "output": output,
        "safe_error_code": safe_error_code,
        "retry_after_ms": retry_after_ms,
    })
}

const fn default_addon_task_run_lease_duration_ms() -> u64 {
    30_000
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitAddonSideEffectRequest {
    pub permission: AddonPermission,
    pub library_id: LibraryId,
    pub target: AddonSideEffectTargetRequest,
    pub idempotency_key: String,
    pub provenance: serde_json::Value,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectTargetRequest {
    pub kind: AddonSideEffectTargetKind,
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectTargetSummary {
    pub kind: AddonSideEffectTargetKind,
    pub id: String,
}

impl AddonSideEffectTargetSummary {
    #[must_use]
    pub fn from_target(target: AddonSideEffectTarget) -> Self {
        Self {
            kind: target.kind,
            id: target.id,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectSummary {
    pub id: AddonSideEffectId,
    pub addon_id: AddonId,
    pub token_id: AddonTokenId,
    pub permission: AddonPermission,
    pub library_id: LibraryId,
    pub target: AddonSideEffectTargetSummary,
    pub idempotency_key: String,
    pub validation_status: AddonSideEffectValidationStatus,
    pub safe_error_code: Option<String>,
    pub apply_status: AddonSideEffectApplyStatus,
    pub apply_error_code: Option<String>,
    pub applied_item_id: Option<MediaItemId>,
    pub applied_source: Option<String>,
    pub apply_report: Option<serde_json::Value>,
    pub applied_at: Option<String>,
    pub created_at: String,
}

impl AddonSideEffectSummary {
    #[must_use]
    pub fn from_record(record: AddonSideEffectRecord) -> Self {
        Self {
            id: record.id,
            addon_id: record.addon_id,
            token_id: record.token_id,
            permission: record.permission,
            library_id: record.library_id,
            target: AddonSideEffectTargetSummary::from_target(record.target),
            idempotency_key: record.idempotency_key,
            validation_status: record.validation_status,
            safe_error_code: record.safe_error_code,
            apply_status: record.apply_status,
            apply_error_code: record.apply_error_code,
            applied_item_id: record.applied_item_id,
            applied_source: record.applied_source,
            apply_report: record
                .apply_report_json
                .and_then(|value| serde_json::from_str(&value).ok()),
            applied_at: record.applied_at,
            created_at: record.created_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectResponse {
    pub side_effect: AddonSideEffectSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitAddonGeneratedArtifactRequest {
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub idempotency_key: String,
    pub prompt: serde_json::Value,
    pub payload: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonGeneratedArtifactSummary {
    pub artifact_id: AutomationArtifactId,
    pub provider_id: AutomationProviderId,
    pub job_id: JobId,
    pub capability: AutomationCapability,
    pub kind: AutomationArtifactKind,
    pub library_id: Option<LibraryId>,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub status: AutomationArtifactStatus,
    pub writes_canonical_metadata: bool,
    pub writes_sidecar: bool,
    pub writes_library_files: bool,
    pub creates_media_source: bool,
    pub creates_managed_import: bool,
}

impl AddonGeneratedArtifactSummary {
    #[must_use]
    pub fn from_record(record: AutomationArtifactRecord) -> Self {
        Self {
            artifact_id: record.id,
            provider_id: record.provider_id,
            job_id: record.job_id,
            capability: record.capability,
            kind: record.kind,
            library_id: record.library_id,
            item_id: record.item_id,
            source_id: record.source_id,
            status: record.status,
            writes_canonical_metadata: false,
            writes_sidecar: false,
            writes_library_files: false,
            creates_media_source: false,
            creates_managed_import: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonGeneratedArtifactResponse {
    pub artifact: AddonGeneratedArtifactSummary,
    pub idempotent_replay: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SubmitAddonAcquisitionCandidateRequest {
    pub target_library_id: LibraryId,
    pub source_key: String,
    pub source_uri: String,
    pub display_name: Option<String>,
    pub intended_locator: Option<String>,
    pub size_bytes: Option<u64>,
    pub fingerprint: Option<String>,
    pub state: Option<nako_core::AcquisitionIntakeCandidateState>,
    pub diagnostics: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAcquisitionCandidateSummary {
    pub id: nako_core::AcquisitionIntakeCandidateId,
    pub target_library_id: LibraryId,
    pub state: nako_core::AcquisitionIntakeCandidateState,
    pub source_kind: String,
    pub source_scheme: Option<String>,
    pub source_ref_redacted: String,
    pub source_key_fingerprint: String,
    pub has_display_name: bool,
    pub has_intended_locator: bool,
    pub size_bytes: Option<u64>,
    pub has_fingerprint: bool,
    pub has_diagnostics: bool,
    pub managed_import_artifact_id: Option<nako_core::ManagedImportArtifactId>,
    pub writes_library: bool,
    pub creates_media_source: bool,
    pub creates_managed_import: bool,
    pub promotion_apply: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonAcquisitionCandidateResponse {
    pub candidate: AddonAcquisitionCandidateSummary,
    pub idempotent_replay: bool,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn admin_resource_search_product_response_uses_opaque_link_refs() {
        let response = AdminAddonResourceSearchResponse {
            addon_id: AddonId::new(),
            manifest_id: "example.resource-search".to_owned(),
            search_id: "search_opaque_1".to_owned(),
            status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
            latency_ms: 12,
            attempts: 1,
            limit: 20,
            total: 1,
            result_count: 1,
            results: vec![AdminAddonResourceSearchResultSummary {
                result_ref_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
                title: "Display Title".to_owned(),
                content: Some("Display content".to_owned()),
                source: "pansou".to_owned(),
                tags: vec!["display-tag".to_owned()],
                score: 930,
                links: vec![AdminAddonResourceSearchLinkSummary {
                    selection_id: "sel_opaque_1".to_owned(),
                    link_type: AddonResourceLinkType::Quark,
                    source: "quark".to_owned(),
                    source_ref_redacted: "https://<redacted>".to_owned(),
                    has_password: true,
                    has_note: true,
                }],
            }],
            provider_executions: vec![AdminAddonResourceSearchProviderDiagnostic {
                provider_id: "pansou".to_owned(),
                status: AddonResourceSearchProviderStatus::Ok,
                result_count: 1,
                finality: AddonResourceSearchProviderFinality::Complete,
                has_safe_message: false,
            }],
            http_status: Some(200),
            safe_error_code: None,
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["search_id"], "search_opaque_1");
        assert_eq!(
            value["results"][0]["links"][0]["selection_id"],
            "sel_opaque_1"
        );
        assert_eq!(
            value["results"][0]["links"][0]["source_ref_redacted"],
            "https://<redacted>"
        );
        assert!(!body.contains("https://pan.quark.cn"));
        assert!(!body.contains("normalized_url"));
        assert!(!body.contains("\"url\""));
        assert!(!body.contains("secret-code"));
        assert!(!body.contains("request_context"));
        assert!(!body.contains("provider exception"));
    }

    #[test]
    fn admin_resource_link_check_response_uses_safe_facts_only() {
        let response = AdminAddonResourceLinkCheckResponse {
            addon_id: AddonId::new(),
            manifest_id: "example.resource-search".to_owned(),
            search_id: "search_opaque_1".to_owned(),
            selection_id: "sel_opaque_1".to_owned(),
            status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
            latency_ms: 14,
            attempts: 1,
            link_type: AddonResourceLinkType::Quark,
            check_status: Some(AddonResourceLinkCheckStatus::Reachable),
            checked_at_ms: Some(1_779_814_400_000),
            requires_password: Some(false),
            retryable: Some(false),
            retry_after_ms: None,
            has_safe_message: true,
            safe_facts: BTreeMap::from([("http_status_class".to_owned(), "2xx".to_owned())]),
            http_status: Some(200),
            safe_error_code: None,
        };

        let request = AdminAddonResourceLinkCheckRequest { refresh: true };
        let request_value = serde_json::to_value(&request).unwrap();
        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(request_value, serde_json::json!({ "refresh": true }));
        assert_eq!(value["search_id"], "search_opaque_1");
        assert_eq!(value["selection_id"], "sel_opaque_1");
        assert_eq!(value["link_type"], "quark");
        assert_eq!(value["check_status"], "reachable");
        assert_eq!(value["safe_facts"]["http_status_class"], "2xx");

        for forbidden in [
            "https://pan.quark.cn",
            "normalized_url",
            "\"url\"",
            "secret-code",
            "private-note",
            "source_uri",
            "request_context",
        ] {
            assert!(
                !body.contains(forbidden),
                "link-check product response leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn admin_subtitle_search_product_response_uses_opaque_candidate_refs() {
        let addon_id = AddonId::new();
        let response = AdminAddonSubtitleSearchResponse {
            addon_id,
            manifest_id: "example.subtitle-provider".to_owned(),
            search_id: "sub_opaque_1".to_owned(),
            status: AdminAddonResourceCallDiagnosticStatus::Succeeded,
            latency_ms: 12,
            attempts: 1,
            limit: 10,
            total: 1,
            result_count: 1,
            subtitles: vec![AdminAddonSubtitleCandidateSummary {
                selection_id: "sel_opaque_1".to_owned(),
                candidate_ref_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
                title: "Display Subtitle".to_owned(),
                language: "en".to_owned(),
                format: AddonSubtitleFormat::Vtt,
                source: "fixture".to_owned(),
                release: Some("WEB-DL".to_owned()),
                score: 920,
                delivery_kind: AdminAddonSubtitleDeliveryKind::Inline,
            }],
            provider_executions: vec![AdminAddonSubtitleProviderDiagnostic {
                provider_id: "fixture".to_owned(),
                status: AddonSubtitleProviderStatus::Ok,
                result_count: 1,
                has_safe_message: false,
            }],
            http_status: Some(200),
            safe_error_code: None,
        };
        let selection = AdminAddonSubtitleSelectionResponse {
            selected_ref: AdminAddonSubtitleSelectedReference {
                addon_id,
                manifest_id: response.manifest_id.clone(),
                search_id: response.search_id.clone(),
                selection_id: response.subtitles[0].selection_id.clone(),
                candidate_ref_fingerprint: response.subtitles[0].candidate_ref_fingerprint.clone(),
                delivery_kind: AdminAddonSubtitleDeliveryKind::Inline,
            },
            candidate: response.subtitles[0].clone(),
        };

        let request = AdminAddonSubtitleSelectionRequest::default();
        let request_value = serde_json::to_value(&request).unwrap();
        let value = serde_json::to_value(&response).unwrap();
        let selection_value = serde_json::to_value(&selection).unwrap();
        let body = format!("{value}{selection_value}");

        assert_eq!(request_value, serde_json::json!({}));
        assert_eq!(value["search_id"], "sub_opaque_1");
        assert_eq!(value["subtitles"][0]["selection_id"], "sel_opaque_1");
        assert_eq!(value["subtitles"][0]["delivery_kind"], "inline");
        assert_eq!(
            selection_value["selected_ref"]["candidate_ref_fingerprint"],
            "sha256:0123456789abcdef0123456789abcdef"
        );

        for forbidden in [
            "WEBVTT",
            "secret subtitle text",
            "https://subtitle.example",
            "download?token",
            "artifact-secret-id",
            "source_locator",
            "local:///secret",
            "\"url\"",
            "\"text\"",
            "\"artifact_id\"",
        ] {
            assert!(
                !body.contains(forbidden),
                "subtitle response leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn admin_subtitle_import_plan_response_uses_safe_preview_fields() {
        let addon_id = AddonId::new();
        let library_id = LibraryId::new();
        let item_id = MediaItemId::new();
        let source_id = MediaSourceId::new();
        let response = AdminAddonSubtitleImportPlanResponse {
            selected_ref: AdminAddonSubtitleSelectedReference {
                addon_id,
                manifest_id: "example.subtitle-provider".to_owned(),
                search_id: "sub_opaque_1".to_owned(),
                selection_id: "sel_opaque_1".to_owned(),
                candidate_ref_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
                delivery_kind: AdminAddonSubtitleDeliveryKind::DownloadUrl,
            },
            candidate: AdminAddonSubtitleCandidateSummary {
                selection_id: "sel_opaque_1".to_owned(),
                candidate_ref_fingerprint: "sha256:0123456789abcdef0123456789abcdef".to_owned(),
                title: "Display Subtitle".to_owned(),
                language: "en".to_owned(),
                format: AddonSubtitleFormat::Srt,
                source: "fixture".to_owned(),
                release: None,
                score: 900,
                delivery_kind: AdminAddonSubtitleDeliveryKind::DownloadUrl,
            },
            plan: AdminSubtitleImportPlan {
                idempotency_key: "sip_opaque_1".to_owned(),
                status: AdminSubtitleImportPlanStatus::Ready,
                reasons: vec![
                    AdminSubtitleImportPlanReason::MediaSourceMatchesItem,
                    AdminSubtitleImportPlanReason::Ready,
                ],
                target: AdminSubtitleImportTargetSummary {
                    library_id,
                    media_item_id: item_id,
                    media_source_id: source_id,
                    item_title: "Demo".to_owned(),
                    media_file_name: "demo.mkv".to_owned(),
                    source_ref_fingerprint: "sha256:abcdefabcdefabcdefabcdefabcdefab".to_owned(),
                },
                sidecar: AdminSubtitleSidecarPlan {
                    file_name: "demo.en.srt".to_owned(),
                    language: "en".to_owned(),
                    format: AddonSubtitleFormat::Srt,
                    role: AdminSubtitleSidecarRole::Default,
                },
                conflict_policy: AdminSubtitleImportConflictPolicy::CreateMissing,
                backup_policy: AdminSubtitleImportBackupPolicy::None,
                preview_only: true,
                writes_library: false,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["plan"]["preview_only"], true);
        assert_eq!(value["plan"]["writes_library"], false);
        assert_eq!(value["plan"]["sidecar"]["file_name"], "demo.en.srt");
        assert_eq!(value["plan"]["target"]["media_file_name"], "demo.mkv");

        for forbidden in [
            "WEBVTT",
            "secret subtitle text",
            "https://subtitle.example",
            "artifact-secret-id",
            "source_locator",
            "local:///",
            "C:\\",
            "\"url\"",
            "\"text\"",
            "\"artifact_id\"",
            "backup_uri",
        ] {
            assert!(
                !body.contains(forbidden),
                "subtitle import plan leaked forbidden term: {forbidden}"
            );
        }
    }

    #[test]
    fn addon_runtime_access_check_wire_shape_matches_public_protocol() {
        let library_id = LibraryId::from_str("018f0000-0000-7000-8000-000000000003").unwrap();

        let api_request = serde_json::to_value(AddonAccessCheckRequest {
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
        })
        .unwrap();
        let protocol_request = serde_json::to_value(nako_addon_protocol::AddonAccessCheckRequest {
            permission: nako_addon_protocol::AddonPermission::MetadataWrite,
            library_id: Some(library_id.to_string()),
        })
        .unwrap();

        assert_eq!(api_request, protocol_request);

        let addon_id = AddonId::from_str("018f0000-0000-7000-8000-000000000002").unwrap();
        let token_id = AddonTokenId::from_str("018f0000-0000-7000-8000-000000000004").unwrap();

        let api_response = serde_json::to_value(AddonAccessCheckResponse {
            addon_id,
            token_id,
            permission: AddonPermission::MetadataWrite,
            library_id: Some(library_id),
            allowed: true,
        })
        .unwrap();
        let protocol_response =
            serde_json::to_value(nako_addon_protocol::AddonAccessCheckResponse {
                addon_id: addon_id.to_string(),
                token_id: token_id.to_string(),
                permission: nako_addon_protocol::AddonPermission::MetadataWrite,
                library_id: Some(library_id.to_string()),
                allowed: true,
            })
            .unwrap();

        assert_eq!(api_response, protocol_response);
    }

    #[test]
    fn addon_runtime_side_effect_wire_shape_matches_public_protocol() {
        let library_id = LibraryId::from_str("018f0000-0000-7000-8000-000000000003").unwrap();
        let source_id = MediaSourceId::from_str("018f0000-0000-7000-8000-000000000005").unwrap();

        let api_request = serde_json::to_value(SubmitAddonSideEffectRequest {
            permission: AddonPermission::MetadataWrite,
            library_id,
            target: AddonSideEffectTargetRequest {
                kind: AddonSideEffectTargetKind::MediaSource,
                id: source_id.to_string(),
            },
            idempotency_key: "metadata-demo-1".to_owned(),
            provenance: serde_json::json!({
                "origin": "reference-addon",
                "request_id": "request-1"
            }),
            payload: serde_json::json!({
                "title": "Demo From Addon",
                "genres": ["Addon Genre"]
            }),
        })
        .unwrap();
        let protocol_request =
            serde_json::to_value(nako_addon_protocol::SubmitAddonSideEffectRequest {
                permission: nako_addon_protocol::AddonPermission::MetadataWrite,
                library_id: library_id.to_string(),
                target: nako_addon_protocol::AddonSideEffectTarget {
                    kind: nako_addon_protocol::AddonSideEffectTargetKind::MediaSource,
                    id: source_id.to_string(),
                },
                idempotency_key: "metadata-demo-1".to_owned(),
                provenance: serde_json::json!({
                    "origin": "reference-addon",
                    "request_id": "request-1"
                }),
                payload: serde_json::json!({
                    "title": "Demo From Addon",
                    "genres": ["Addon Genre"]
                }),
            })
            .unwrap();

        assert_eq!(api_request, protocol_request);

        let side_effect_id =
            AddonSideEffectId::from_str("018f0000-0000-7000-8000-000000000006").unwrap();
        let addon_id = AddonId::from_str("018f0000-0000-7000-8000-000000000002").unwrap();
        let token_id = AddonTokenId::from_str("018f0000-0000-7000-8000-000000000004").unwrap();
        let item_id = MediaItemId::from_str("018f0000-0000-7000-8000-000000000007").unwrap();

        let api_response = serde_json::to_value(AddonSideEffectResponse {
            side_effect: AddonSideEffectSummary {
                id: side_effect_id,
                addon_id,
                token_id,
                permission: AddonPermission::MetadataWrite,
                library_id,
                target: AddonSideEffectTargetSummary {
                    kind: AddonSideEffectTargetKind::MediaSource,
                    id: source_id.to_string(),
                },
                idempotency_key: "metadata-demo-1".to_owned(),
                validation_status: AddonSideEffectValidationStatus::Accepted,
                safe_error_code: None,
                apply_status: AddonSideEffectApplyStatus::Applied,
                apply_error_code: None,
                applied_item_id: Some(item_id),
                applied_source: Some(format!("addon:{addon_id}")),
                apply_report: Some(serde_json::json!({
                    "projected_items": 1
                })),
                applied_at: Some("2026-05-18T12:00:00.000Z".to_owned()),
                created_at: "2026-05-18T12:00:00.000Z".to_owned(),
            },
            idempotent_replay: false,
        })
        .unwrap();
        let protocol_response =
            serde_json::to_value(nako_addon_protocol::AddonSideEffectResponse {
                side_effect: nako_addon_protocol::AddonSideEffectSummary {
                    id: side_effect_id.to_string(),
                    addon_id: Some(addon_id.to_string()),
                    token_id: Some(token_id.to_string()),
                    permission: nako_addon_protocol::AddonPermission::MetadataWrite,
                    library_id: library_id.to_string(),
                    target: nako_addon_protocol::AddonSideEffectTarget {
                        kind: nako_addon_protocol::AddonSideEffectTargetKind::MediaSource,
                        id: source_id.to_string(),
                    },
                    idempotency_key: "metadata-demo-1".to_owned(),
                    validation_status: "accepted".to_owned(),
                    safe_error_code: None,
                    apply_status: "applied".to_owned(),
                    apply_error_code: None,
                    applied_item_id: Some(item_id.to_string()),
                    applied_source: Some(format!("addon:{addon_id}")),
                    apply_report: Some(serde_json::json!({
                        "projected_items": 1
                    })),
                    applied_at: Some("2026-05-18T12:00:00.000Z".to_owned()),
                    created_at: Some("2026-05-18T12:00:00.000Z".to_owned()),
                },
                idempotent_replay: false,
            })
            .unwrap();

        assert_eq!(api_response, protocol_response);
    }
}
