use serde::{Deserialize, Serialize};
use taru_addon_protocol::{AddonManifest, AddonScope};
use taru_core::{
    AddonGrantRecord, AddonId, AddonPermission, AddonRegistrationRecord, AddonSideEffectId,
    AddonSideEffectRecord, AddonSideEffectTarget, AddonSideEffectTargetKind,
    AddonSideEffectValidationStatus, AddonStatus, AddonTokenId, AddonTokenRecord, AddonTokenStatus,
    AutomationArtifactRecord, AutomationCapability, AutomationJobInput,
    AutomationProviderConfigRecord, AutomationProviderId, AutomationProviderStatus, EventId,
    LibraryId, MediaItemId, MediaSourceId, OutboxEventRecord, WebhookDeliveryAttemptRecord,
    WebhookEndpointId, WebhookEndpointRecord, WebhookEndpointStatus,
};

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
pub struct AddonRegistrationResponse {
    pub addon: AddonRegistrationRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonRegistrationsResponse {
    pub addons: Vec<AddonRegistrationRecord>,
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
            created_at: record.created_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AddonSideEffectResponse {
    pub side_effect: AddonSideEffectSummary,
    pub idempotent_replay: bool,
}
