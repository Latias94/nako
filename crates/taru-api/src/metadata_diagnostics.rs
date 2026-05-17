use serde::{Deserialize, Serialize};
use taru_client_protocol::PageInfo;
use taru_core::{
    ExternalProvider, LibraryId, MediaItemId, MediaKind, MetadataProfile,
    MetadataProviderAttemptRecord, MetadataRefreshMode, ProviderRawResponse,
    ProviderRawResponseCleanup,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptsResponse {
    pub item_id: MediaItemId,
    pub attempts: Vec<MetadataProviderAttemptDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderAttemptDiagnostic {
    #[serde(flatten)]
    pub attempt: MetadataProviderAttemptRecord,
    pub retryable: bool,
}

impl MetadataProviderAttemptDiagnostic {
    #[must_use]
    pub fn from_record(attempt: MetadataProviderAttemptRecord) -> Self {
        Self {
            retryable: attempt.is_retryable(),
            attempt,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRawResponsesResponse {
    pub item_id: MediaItemId,
    pub responses: Vec<ProviderRawResponse>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderDiagnosticsResponse {
    pub providers: Vec<MetadataProviderDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderDiagnostic {
    pub provider: ExternalProvider,
    pub status: MetadataProviderDiagnosticStatus,
    pub provider_name: Option<String>,
    pub reason: Option<String>,
    pub runtime: MetadataProviderRuntimeDiagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderDiagnosticStatus {
    Available,
    Disabled,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataProviderRuntimeDiagnostic {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub min_interval_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
    pub proxy_configured: bool,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_backoff_ms: u64,
    pub circuit_open: bool,
    pub circuit_open_until_ms: Option<u64>,
    pub consecutive_failures: u64,
    pub last_error: Option<String>,
    pub last_rate_limit_wait_ms: u64,
    pub state_scope: MetadataProviderRuntimeStateScope,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProviderRuntimeStateScope {
    ProcessLocal,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct EnqueueMetadataMaintenanceRequest {
    pub library_id: Option<LibraryId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_ids: Vec<MediaItemId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<ExternalProvider>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub item_kinds: Vec<MediaKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile: Option<MetadataProfile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_mode: Option<MetadataRefreshMode>,
    #[serde(default)]
    pub force: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanResponse {
    pub request: EnqueueMetadataMaintenanceRequest,
    pub planned_items: u32,
    pub skipped_items: u32,
    pub items: Vec<MetadataMaintenancePlanItem>,
    pub errors: Vec<MetadataMaintenancePlanError>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanItem {
    pub item_id: MediaItemId,
    pub library_id: Option<LibraryId>,
    pub kind: MediaKind,
    pub title: String,
    pub providers: Vec<ExternalProvider>,
    pub language: Option<String>,
    pub refresh_mode: MetadataRefreshMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataMaintenancePlanError {
    pub item_id: MediaItemId,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MetadataRawCleanupResponse {
    pub cleanup: ProviderRawResponseCleanup,
}
