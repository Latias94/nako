use nako_addon_protocol::{
    AddonConfigurationSchema, AddonEntryPointKind, AddonHealthStatus, AddonInstallDescriptor,
    AddonInstallGuide, AddonManifest, AddonResource, AddonScope, AddonTaskDeclaration,
};
use nako_core::{
    AddonGrantRecord, AddonId, AddonPermission, AddonRegistrationRecord,
    AddonRoutingDeclarationKind, AddonRoutingPlanRecord, AddonRoutingPlanStatus,
    AddonRoutingPlanTarget, AddonSideEffectApplyStatus, AddonSideEffectId, AddonSideEffectRecord,
    AddonSideEffectTarget, AddonSideEffectTargetKind, AddonSideEffectValidationStatus, AddonStatus,
    AddonTokenId, AddonTokenRecord, AddonTokenStatus, AutomationArtifactId, AutomationArtifactKind,
    AutomationArtifactRecord, AutomationArtifactStatus, AutomationCapability, AutomationJobInput,
    AutomationProviderConfigRecord, AutomationProviderId, AutomationProviderStatus, EventId, JobId,
    JobKind, LibraryId, MediaItemId, MediaSourceId, OutboxEventRecord,
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
