use nako_client_protocol::PageInfo;
#[cfg(test)]
use nako_core::StagingAttribution;
use nako_core::{
    LibraryId, StagingAttributionKind, StagingManifestId, StagingManifestRecord, StagingPurpose,
    StagingState, StorageBackendHealthRecord, StorageBackendHealthStatus,
    StorageCircuitBreakerState, StorageFailureClass, VfsCacheOperation,
};
use serde::{Deserialize, Serialize};

use super::operations::{AdminJobListItem, AdminJobPriority};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub summary: AdminStorageStagingSummary,
    pub records: Vec<AdminStorageStagingRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingSummary {
    pub configured_max_bytes: u64,
    pub used_manifest_bytes: u64,
    pub pressure: AdminStorageStagingPressureSummary,
    #[serde(default)]
    pub policy_slices: Vec<AdminStorageStagingPolicySlice>,
    pub cleanup_on_startup: bool,
    pub retention_ms: u64,
    pub startup_deleted_records: u32,
    pub startup_deleted_files: u32,
    pub cleanup_candidate_records: u32,
    pub cleanup_candidate_bytes: u64,
    pub process_cached_backends: u32,
    pub vfs_cache: AdminVfsCacheSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminStorageStagingPressureStatus {
    Disabled,
    Healthy,
    Elevated,
    Critical,
    Exhausted,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingPressureSummary {
    pub status: AdminStorageStagingPressureStatus,
    pub used_ratio_milli: Option<u32>,
    pub total_records: u32,
    pub in_flight_records: u32,
    pub failed_records: u32,
    pub unknown_size_records: u32,
    pub active_leases: u32,
    pub ffmpeg_input_records: u32,
    pub probe_input_records: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingPolicySlice {
    pub backend_key: String,
    pub library_id: Option<LibraryId>,
    pub library_name: Option<String>,
    pub backend_kind: Option<StorageBackendKind>,
    pub source_scheme: String,
    pub configured_max_bytes: u64,
    pub used_manifest_bytes: u64,
    pub pressure: AdminStorageStagingPressureSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheSummary {
    pub object_count: u64,
    pub listing_count: u64,
    pub failure_count: u64,
    pub stale_object_count: u64,
    pub stale_listing_count: u64,
    pub last_failure_at_ms: Option<i64>,
    #[serde(default)]
    pub repair: Option<AdminVfsCacheRepairDiagnostic>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairClassification {
    Healthy,
    RepairableStaleFallback,
    RetryableRefreshFailure,
    OperatorActionRequired,
    UnknownFailure,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairAction {
    None,
    RefreshCache,
    FixBackendConfiguration,
    InspectFailure,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairActionPlanStatus {
    NoAction,
    Executable,
    PlanOnly,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairActionPlanReason {
    NoRepairDiagnostic,
    NoActionRequired,
    RefreshCacheExecutable,
    TargetScopedExecutionUnavailable,
    BackendConfigurationRequired,
    ManualFailureInspectionRequired,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairActionReadiness {
    pub status: AdminVfsCacheRepairActionPlanStatus,
    pub api_executable: bool,
    pub reasons: Vec<AdminVfsCacheRepairActionPlanReason>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairActionBoundary {
    pub refreshes_vfs_cache: bool,
    pub changes_backend_configuration: bool,
    pub requires_manual_failure_inspection: bool,
    pub deletes_cache_entries: bool,
    pub writes_library_files: bool,
    pub starts_durable_job: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairExecutableAction {
    pub method: String,
    pub route_key: String,
    pub route_path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairActionPlan {
    pub status: AdminVfsCacheRepairActionPlanStatus,
    pub action: AdminVfsCacheRepairAction,
    pub readiness: AdminVfsCacheRepairActionReadiness,
    pub boundary: AdminVfsCacheRepairActionBoundary,
    pub executable_action: Option<AdminVfsCacheRepairExecutableAction>,
    pub repair: Option<AdminVfsCacheRepairDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairDiagnostic {
    pub classification: AdminVfsCacheRepairClassification,
    pub recommended_action: AdminVfsCacheRepairAction,
    pub operation: Option<VfsCacheOperation>,
    pub failure_class: Option<StorageFailureClass>,
    pub retryable: bool,
    pub failed_at_ms: Option<i64>,
    pub failure_count: Option<u32>,
    pub safe_message: Option<String>,
    pub operator_action: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRefreshResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub action: AdminVfsCacheRepairAction,
    pub operation: VfsCacheOperation,
    pub refreshed: bool,
    pub repair: AdminVfsCacheRepairDiagnostic,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairActionPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub plan: AdminVfsCacheRepairActionPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairTarget {
    pub target_ref: String,
    pub scheme: String,
    pub operation: VfsCacheOperation,
    pub failed_at_ms: i64,
    pub failure_count: u32,
    pub classification: AdminVfsCacheRepairClassification,
    pub recommended_action: AdminVfsCacheRepairAction,
    pub failure_class: Option<StorageFailureClass>,
    pub retryable: bool,
    pub safe_message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairTargetListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub targets: Vec<AdminVfsCacheRepairTarget>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairTargetPreviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub target: AdminVfsCacheRepairTarget,
    pub plan: AdminVfsCacheRepairActionPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairEnqueueRequest {
    pub priority: Option<AdminJobPriority>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairEnqueueOutcome {
    Enqueued,
    AlreadyQueued,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairEnqueueResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub outcome: AdminVfsCacheRepairEnqueueOutcome,
    pub job: AdminJobListItem,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminVfsCacheRepairCacheState {
    Fresh,
    StaleFallback,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairJobSummary {
    pub action: AdminVfsCacheRepairAction,
    pub source_scheme: String,
    pub operation: VfsCacheOperation,
    pub classification: AdminVfsCacheRepairClassification,
    pub failure_class: Option<StorageFailureClass>,
    pub failed_at_ms: i64,
    pub failure_count: u32,
    pub refreshed_cache_state: Option<AdminVfsCacheRepairCacheState>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairExecuteResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub job: AdminJobListItem,
    pub summary: AdminVfsCacheRepairJobSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairRemediationPlanResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub total_unresolved_targets: u32,
    pub action_groups: Vec<AdminVfsCacheRepairRemediationActionGroup>,
    pub classification_counts: Vec<AdminVfsCacheRepairClassificationCount>,
    pub boundary: AdminVfsCacheRepairRemediationPlanBoundary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairRemediationActionGroup {
    pub action: AdminVfsCacheRepairAction,
    pub count: u32,
    pub status: AdminVfsCacheRepairActionPlanStatus,
    pub readiness: AdminVfsCacheRepairActionReadiness,
    pub boundary: AdminVfsCacheRepairActionBoundary,
    pub executable_action: Option<AdminVfsCacheRepairExecutableAction>,
    pub sample_targets: Vec<AdminVfsCacheRepairTarget>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairClassificationCount {
    pub classification: AdminVfsCacheRepairClassification,
    pub count: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminVfsCacheRepairRemediationPlanBoundary {
    pub read_only: bool,
    pub refreshes_vfs_cache: bool,
    pub changes_backend_configuration: bool,
    pub deletes_cache_entries: bool,
    pub writes_library_files: bool,
    pub starts_durable_job: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingRecord {
    pub id: StagingManifestId,
    pub attribution_kind: StagingAttributionKind,
    pub attribution_library_id: Option<LibraryId>,
    pub source_scheme: String,
    pub purpose: StagingPurpose,
    pub state: StagingState,
    pub size_bytes: Option<u64>,
    pub has_etag: bool,
    pub has_fingerprint: bool,
    pub active_leases: u32,
    pub has_validation_error: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub last_accessed_at_ms: i64,
    pub expires_at_ms: Option<i64>,
}

impl AdminStorageStagingRecord {
    #[must_use]
    pub fn from_record(record: StagingManifestRecord) -> Self {
        Self {
            id: record.id,
            attribution_kind: record.attribution.kind(),
            attribution_library_id: record.attribution.library_id(),
            source_scheme: record.source_scheme,
            purpose: record.purpose,
            state: record.state,
            size_bytes: record.size_bytes,
            has_etag: record.etag.is_some(),
            has_fingerprint: record.fingerprint.is_some(),
            active_leases: record.active_leases,
            has_validation_error: record.validation_error.is_some(),
            created_at_ms: record.created_at_ms,
            updated_at_ms: record.updated_at_ms,
            last_accessed_at_ms: record.last_accessed_at_ms,
            expires_at_ms: record.expires_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageBackendHealthDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub backends: Vec<AdminStorageBackendHealthDiagnostic>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageBackendHealthResetResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub backend: AdminStorageBackendHealthDiagnostic,
    pub reset_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageBackendHealthDiagnostic {
    pub backend_key: String,
    pub library_id: Option<LibraryId>,
    pub scheme: String,
    pub status: StorageBackendHealthStatus,
    pub circuit_breaker_state: StorageCircuitBreakerState,
    pub consecutive_failures: u32,
    pub last_success_at_ms: Option<i64>,
    pub last_failure_at_ms: Option<i64>,
    pub last_failure_class: Option<StorageFailureClass>,
    pub last_failure_safe_message: Option<String>,
    pub circuit_opened_at_ms: Option<i64>,
    pub backoff_until_ms: Option<i64>,
    pub updated_at_ms: i64,
}

impl AdminStorageBackendHealthDiagnostic {
    #[must_use]
    pub fn from_record(record: StorageBackendHealthRecord) -> Self {
        Self {
            backend_key: record.backend_key,
            library_id: record.library_id,
            scheme: record.scheme,
            status: record.status,
            circuit_breaker_state: record.circuit_breaker_state,
            consecutive_failures: record.consecutive_failures,
            last_success_at_ms: record.last_success_at_ms,
            last_failure_at_ms: record.last_failure_at_ms,
            last_failure_class: record.last_failure_class,
            last_failure_safe_message: record.last_failure_safe_message,
            circuit_opened_at_ms: record.circuit_opened_at_ms,
            backoff_until_ms: record.backoff_until_ms,
            updated_at_ms: record.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnosticsResponse {
    pub backends: Vec<StorageBackendDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendDiagnostic {
    pub library_id: LibraryId,
    pub library_name: String,
    pub root_uri: String,
    pub backend_kind: StorageBackendKind,
    pub scheme: String,
    pub status: StorageBackendStatus,
    pub reason: Option<String>,
    pub registry: StorageBackendRegistryDiagnostic,
    pub health: StorageBackendHealthDiagnostic,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendKind {
    Local,
    #[serde(rename = "webdav")]
    WebDav,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendRegistryDiagnostic {
    pub cached: bool,
    pub stream_permits_available: usize,
    pub stream_permits_max: usize,
    pub stage_permits_available: usize,
    pub stage_permits_max: usize,
    pub state_scope: StorageBackendRuntimeStateScope,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StorageBackendHealthDiagnostic {
    pub consecutive_errors: u64,
    pub last_success_at_ms: Option<i64>,
    pub last_error_at_ms: Option<i64>,
    pub last_error_class: Option<StorageFailureClass>,
    pub backoff_until_ms: Option<i64>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageBackendRuntimeStateScope {
    ProcessLocal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_storage_staging_record_redacts_paths_source_uri_and_errors() {
        let record = StagingManifestRecord {
            id: StagingManifestId::new(),
            attribution: StagingAttribution::attributed(LibraryId::new()),
            source_uri: "webdav:///Movies/Private/Demo.mkv".to_owned(),
            source_scheme: "webdav".to_owned(),
            purpose: StagingPurpose::FfmpegInput,
            local_path: "F:\\Nako\\secret-cache\\inputs\\Demo.mkv".to_owned(),
            size_bytes: Some(42),
            etag: Some("etag-secret".to_owned()),
            fingerprint: Some("fingerprint-secret".to_owned()),
            state: StagingState::Failed,
            created_at_ms: 1_000,
            updated_at_ms: 1_100,
            last_accessed_at_ms: 1_200,
            expires_at_ms: Some(1_300),
            active_leases: 0,
            validation_error: Some("failed at F:\\Nako\\secret-cache".to_owned()),
        };

        let item = AdminStorageStagingRecord::from_record(record);
        let body = serde_json::to_string(&item).unwrap();

        assert_eq!(item.source_scheme, "webdav");
        assert_eq!(item.purpose, StagingPurpose::FfmpegInput);
        assert_eq!(item.state, StagingState::Failed);
        assert_eq!(item.attribution_kind, StagingAttributionKind::Attributed);
        assert!(item.attribution_library_id.is_some());
        assert_eq!(item.size_bytes, Some(42));
        assert!(item.has_etag);
        assert!(item.has_fingerprint);
        assert!(item.has_validation_error);
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("secret-cache"));
        assert!(!body.contains("etag-secret"));
        assert!(!body.contains("fingerprint-secret"));
        assert!(!body.contains("failed at"));
    }

    #[test]
    fn admin_storage_staging_policy_slice_redacts_source_identity() {
        let library_id = LibraryId::new();
        let slice = AdminStorageStagingPolicySlice {
            backend_key: format!("library:{library_id}:webdav"),
            library_id: Some(library_id),
            library_name: Some("Remote Movies".to_owned()),
            backend_kind: Some(StorageBackendKind::WebDav),
            source_scheme: "webdav".to_owned(),
            configured_max_bytes: 100,
            used_manifest_bytes: 95,
            pressure: AdminStorageStagingPressureSummary {
                status: AdminStorageStagingPressureStatus::Critical,
                used_ratio_milli: Some(950),
                total_records: 1,
                in_flight_records: 1,
                failed_records: 0,
                unknown_size_records: 0,
                active_leases: 1,
                ffmpeg_input_records: 0,
                probe_input_records: 1,
            },
        };

        let body = serde_json::to_string(&slice).unwrap();

        assert_eq!(slice.library_id, Some(library_id));
        assert_eq!(slice.backend_kind, Some(StorageBackendKind::WebDav));
        assert_eq!(slice.source_scheme, "webdav");
        assert_eq!(
            slice.pressure.status,
            AdminStorageStagingPressureStatus::Critical
        );
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
        assert!(!body.contains("fingerprint"));
    }

    #[test]
    fn admin_storage_backend_health_diagnostic_redacts_raw_backend_details() {
        let library_id = LibraryId::new();
        let diagnostic =
            AdminStorageBackendHealthDiagnostic::from_record(StorageBackendHealthRecord {
                backend_key: format!("library:{library_id}:webdav"),
                library_id: Some(library_id),
                scheme: "webdav".to_owned(),
                status: StorageBackendHealthStatus::Unavailable,
                circuit_breaker_state: StorageCircuitBreakerState::Open,
                consecutive_failures: 3,
                last_success_at_ms: Some(500),
                last_failure_at_ms: Some(1_000),
                last_failure_class: Some(StorageFailureClass::Timeout),
                last_failure_safe_message: Some("storage backend timed out".to_owned()),
                circuit_opened_at_ms: Some(1_000),
                backoff_until_ms: Some(1_500),
                updated_at_ms: 1_000,
            });
        let body = serde_json::to_string(&diagnostic).unwrap();

        assert_eq!(
            diagnostic.backend_key,
            format!("library:{library_id}:webdav")
        );
        assert_eq!(diagnostic.library_id, Some(library_id));
        assert_eq!(diagnostic.scheme, "webdav");
        assert_eq!(diagnostic.status, StorageBackendHealthStatus::Unavailable);
        assert_eq!(
            diagnostic.circuit_breaker_state,
            StorageCircuitBreakerState::Open
        );
        assert_eq!(
            diagnostic.last_failure_safe_message.as_deref(),
            Some("storage backend timed out")
        );
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Private"));
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
    }

    #[test]
    fn admin_vfs_cache_summary_serializes_redacted_repair_preview() {
        let summary = AdminVfsCacheSummary {
            object_count: 10,
            listing_count: 4,
            failure_count: 1,
            stale_object_count: 3,
            stale_listing_count: 1,
            last_failure_at_ms: Some(1_000),
            repair: Some(AdminVfsCacheRepairDiagnostic {
                classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
                recommended_action: AdminVfsCacheRepairAction::RefreshCache,
                operation: Some(VfsCacheOperation::List),
                failure_class: Some(StorageFailureClass::Unavailable),
                retryable: true,
                failed_at_ms: Some(1_000),
                failure_count: Some(2),
                safe_message: Some("storage backend unavailable".to_owned()),
                operator_action: "cache refresh failed with a retryable storage failure".to_owned(),
            }),
        };

        let value = serde_json::to_value(&summary).unwrap();
        let body = value.to_string();

        assert_eq!(
            value["repair"]["classification"],
            "retryable_refresh_failure"
        );
        assert_eq!(value["repair"]["recommended_action"], "refresh_cache");
        assert_eq!(value["repair"]["operation"], "list");
        assert_eq!(value["repair"]["failure_class"], "unavailable");
        assert_eq!(value["repair"]["retryable"], true);
        assert_eq!(value["repair"]["failed_at_ms"], 1000);
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
        assert!(!body.contains("Movies/Demo.mkv"));
    }

    #[test]
    fn admin_vfs_cache_refresh_response_serializes_redacted_action_result() {
        let response = AdminVfsCacheRefreshResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            action: AdminVfsCacheRepairAction::RefreshCache,
            operation: VfsCacheOperation::Stat,
            refreshed: true,
            repair: AdminVfsCacheRepairDiagnostic {
                classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
                recommended_action: AdminVfsCacheRepairAction::RefreshCache,
                operation: Some(VfsCacheOperation::Stat),
                failure_class: Some(StorageFailureClass::Unavailable),
                retryable: true,
                failed_at_ms: Some(1_000),
                failure_count: Some(2),
                safe_message: Some("storage backend unavailable".to_owned()),
                operator_action: "cache refresh failed with a retryable storage failure".to_owned(),
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["action"], "refresh_cache");
        assert_eq!(value["operation"], "stat");
        assert_eq!(value["refreshed"], true);
        assert_eq!(value["repair"]["recommended_action"], "refresh_cache");
        assert_eq!(
            value["repair"]["safe_message"],
            "storage backend unavailable"
        );
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Movies/Demo.mkv"));
        assert!(!body.contains("cache-etag-secret"));
        assert!(!body.contains("cache-fingerprint-secret"));
        assert!(!body.contains("token=secret"));
    }

    #[test]
    fn admin_vfs_cache_repair_action_plan_serializes_redacted_boundaries() {
        let repair = AdminVfsCacheRepairDiagnostic {
            classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
            recommended_action: AdminVfsCacheRepairAction::RefreshCache,
            operation: Some(VfsCacheOperation::Stat),
            failure_class: Some(StorageFailureClass::Unavailable),
            retryable: true,
            failed_at_ms: Some(1_000),
            failure_count: Some(2),
            safe_message: Some("storage backend unavailable".to_owned()),
            operator_action: "cache refresh failed with a retryable storage failure".to_owned(),
        };
        let response = AdminVfsCacheRepairActionPlanResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            plan: AdminVfsCacheRepairActionPlan {
                status: AdminVfsCacheRepairActionPlanStatus::Executable,
                action: AdminVfsCacheRepairAction::RefreshCache,
                readiness: AdminVfsCacheRepairActionReadiness {
                    status: AdminVfsCacheRepairActionPlanStatus::Executable,
                    api_executable: true,
                    reasons: vec![AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable],
                },
                boundary: AdminVfsCacheRepairActionBoundary {
                    refreshes_vfs_cache: true,
                    changes_backend_configuration: false,
                    requires_manual_failure_inspection: false,
                    deletes_cache_entries: false,
                    writes_library_files: false,
                    starts_durable_job: false,
                },
                executable_action: Some(AdminVfsCacheRepairExecutableAction {
                    method: "POST".to_owned(),
                    route_key: "storageVfsCacheRepairRefreshCache".to_owned(),
                    route_path: "/admin/v1/storage/vfs-cache/repair/refresh-cache".to_owned(),
                }),
                repair: Some(repair),
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["plan"]["status"], "executable");
        assert_eq!(value["plan"]["action"], "refresh_cache");
        assert_eq!(value["plan"]["readiness"]["api_executable"], true);
        assert_eq!(
            value["plan"]["readiness"]["reasons"][0],
            "refresh_cache_executable"
        );
        assert_eq!(value["plan"]["boundary"]["refreshes_vfs_cache"], true);
        assert_eq!(
            value["plan"]["executable_action"]["route_key"],
            "storageVfsCacheRepairRefreshCache"
        );
        assert_eq!(
            value["plan"]["executable_action"]["route_path"],
            "/admin/v1/storage/vfs-cache/repair/refresh-cache"
        );
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Movies/Demo.mkv"));
        assert!(!body.contains("cache-etag-secret"));
        assert!(!body.contains("cache-fingerprint-secret"));
        assert!(!body.contains("token=secret"));

        let plan_only = AdminVfsCacheRepairActionPlan {
            status: AdminVfsCacheRepairActionPlanStatus::PlanOnly,
            action: AdminVfsCacheRepairAction::FixBackendConfiguration,
            readiness: AdminVfsCacheRepairActionReadiness {
                status: AdminVfsCacheRepairActionPlanStatus::PlanOnly,
                api_executable: false,
                reasons: vec![AdminVfsCacheRepairActionPlanReason::BackendConfigurationRequired],
            },
            boundary: AdminVfsCacheRepairActionBoundary {
                refreshes_vfs_cache: false,
                changes_backend_configuration: true,
                requires_manual_failure_inspection: false,
                deletes_cache_entries: false,
                writes_library_files: false,
                starts_durable_job: false,
            },
            executable_action: None,
            repair: None,
        };
        let value = serde_json::to_value(&plan_only).unwrap();

        assert_eq!(value["status"], "plan_only");
        assert_eq!(value["action"], "fix_backend_configuration");
        assert_eq!(value["readiness"]["api_executable"], false);
        assert_eq!(
            value["readiness"]["reasons"][0],
            "backend_configuration_required"
        );
        assert_eq!(value["boundary"]["changes_backend_configuration"], true);
        assert_eq!(value["executable_action"], serde_json::Value::Null);
    }

    #[test]
    fn admin_vfs_cache_repair_remediation_plan_serializes_redacted_read_only_boundary() {
        let refresh_target = AdminVfsCacheRepairTarget {
            target_ref: "vfsrt_0123456789abcdef0123456789abcdef".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 1_000,
            failure_count: 3,
            classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
            recommended_action: AdminVfsCacheRepairAction::RefreshCache,
            failure_class: Some(StorageFailureClass::Unavailable),
            retryable: true,
            safe_message: Some("storage backend unavailable".to_owned()),
        };
        let operator_target = AdminVfsCacheRepairTarget {
            target_ref: "vfsrt_fedcba9876543210fedcba9876543210".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 1_100,
            failure_count: 1,
            classification: AdminVfsCacheRepairClassification::OperatorActionRequired,
            recommended_action: AdminVfsCacheRepairAction::FixBackendConfiguration,
            failure_class: Some(StorageFailureClass::Timeout),
            retryable: false,
            safe_message: Some("storage backend timed out".to_owned()),
        };
        let response = AdminVfsCacheRepairRemediationPlanResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            total_unresolved_targets: 2,
            action_groups: vec![
                AdminVfsCacheRepairRemediationActionGroup {
                    action: AdminVfsCacheRepairAction::RefreshCache,
                    count: 1,
                    status: AdminVfsCacheRepairActionPlanStatus::Executable,
                    readiness: AdminVfsCacheRepairActionReadiness {
                        status: AdminVfsCacheRepairActionPlanStatus::Executable,
                        api_executable: true,
                        reasons: vec![AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable],
                    },
                    boundary: AdminVfsCacheRepairActionBoundary {
                        refreshes_vfs_cache: true,
                        changes_backend_configuration: false,
                        requires_manual_failure_inspection: false,
                        deletes_cache_entries: false,
                        writes_library_files: false,
                        starts_durable_job: false,
                    },
                    executable_action: Some(AdminVfsCacheRepairExecutableAction {
                        method: "POST".to_owned(),
                        route_key: "storageVfsCacheRepairTargetRefreshCache".to_owned(),
                        route_path:
                            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
                                .to_owned(),
                    }),
                    sample_targets: vec![refresh_target],
                },
                AdminVfsCacheRepairRemediationActionGroup {
                    action: AdminVfsCacheRepairAction::FixBackendConfiguration,
                    count: 1,
                    status: AdminVfsCacheRepairActionPlanStatus::PlanOnly,
                    readiness: AdminVfsCacheRepairActionReadiness {
                        status: AdminVfsCacheRepairActionPlanStatus::PlanOnly,
                        api_executable: false,
                        reasons: vec![
                            AdminVfsCacheRepairActionPlanReason::BackendConfigurationRequired,
                        ],
                    },
                    boundary: AdminVfsCacheRepairActionBoundary {
                        refreshes_vfs_cache: false,
                        changes_backend_configuration: true,
                        requires_manual_failure_inspection: false,
                        deletes_cache_entries: false,
                        writes_library_files: false,
                        starts_durable_job: false,
                    },
                    executable_action: None,
                    sample_targets: vec![operator_target],
                },
            ],
            classification_counts: vec![
                AdminVfsCacheRepairClassificationCount {
                    classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
                    count: 1,
                },
                AdminVfsCacheRepairClassificationCount {
                    classification: AdminVfsCacheRepairClassification::OperatorActionRequired,
                    count: 1,
                },
            ],
            boundary: AdminVfsCacheRepairRemediationPlanBoundary {
                read_only: true,
                refreshes_vfs_cache: false,
                changes_backend_configuration: false,
                deletes_cache_entries: false,
                writes_library_files: false,
                starts_durable_job: false,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["total_unresolved_targets"], 2);
        assert_eq!(value["boundary"]["read_only"], true);
        assert_eq!(value["boundary"]["refreshes_vfs_cache"], false);
        assert_eq!(value["boundary"]["changes_backend_configuration"], false);
        assert_eq!(value["boundary"]["deletes_cache_entries"], false);
        assert_eq!(value["boundary"]["writes_library_files"], false);
        assert_eq!(value["boundary"]["starts_durable_job"], false);
        assert_eq!(value["action_groups"][0]["action"], "refresh_cache");
        assert_eq!(value["action_groups"][0]["status"], "executable");
        assert_eq!(
            value["action_groups"][0]["readiness"]["reasons"][0],
            "refresh_cache_executable"
        );
        assert_eq!(
            value["action_groups"][0]["executable_action"]["route_path"],
            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
        );
        assert_eq!(
            value["action_groups"][1]["action"],
            "fix_backend_configuration"
        );
        assert_eq!(value["action_groups"][1]["status"], "plan_only");
        assert_eq!(
            value["action_groups"][1]["executable_action"],
            serde_json::Value::Null
        );
        assert_eq!(
            value["classification_counts"][0]["classification"],
            "retryable_refresh_failure"
        );
        assert_eq!(
            value["classification_counts"][1]["classification"],
            "operator_action_required"
        );

        assert!(!body.contains("source_uri"));
        assert!(!body.contains("source_locator"));
        assert!(!body.contains("cache_uri"));
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Movies/Demo.mkv"));
        assert!(!body.contains("F:\\Nako\\secret-cache"));
        assert!(!body.contains("cache-etag-secret"));
        assert!(!body.contains("cache-fingerprint-secret"));
        assert!(!body.contains("token=secret"));
        assert!(!body.contains("bearer-token-secret"));
    }

    #[test]
    fn admin_vfs_cache_repair_target_preview_serializes_redacted_scope() {
        let target = AdminVfsCacheRepairTarget {
            target_ref: "vfsrt_0123456789abcdef0123456789abcdef".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 1_000,
            failure_count: 3,
            classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
            recommended_action: AdminVfsCacheRepairAction::RefreshCache,
            failure_class: Some(StorageFailureClass::Unavailable),
            retryable: true,
            safe_message: Some("storage backend unavailable".to_owned()),
        };
        let plan = AdminVfsCacheRepairActionPlan {
            status: AdminVfsCacheRepairActionPlanStatus::Executable,
            action: AdminVfsCacheRepairAction::RefreshCache,
            readiness: AdminVfsCacheRepairActionReadiness {
                status: AdminVfsCacheRepairActionPlanStatus::Executable,
                api_executable: true,
                reasons: vec![AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable],
            },
            boundary: AdminVfsCacheRepairActionBoundary {
                refreshes_vfs_cache: true,
                changes_backend_configuration: false,
                requires_manual_failure_inspection: false,
                deletes_cache_entries: false,
                writes_library_files: false,
                starts_durable_job: false,
            },
            executable_action: Some(AdminVfsCacheRepairExecutableAction {
                method: "POST".to_owned(),
                route_key: "storageVfsCacheRepairTargetRefreshCache".to_owned(),
                route_path: "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
                    .to_owned(),
            }),
            repair: Some(AdminVfsCacheRepairDiagnostic {
                classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
                recommended_action: AdminVfsCacheRepairAction::RefreshCache,
                operation: Some(VfsCacheOperation::List),
                failure_class: Some(StorageFailureClass::Unavailable),
                retryable: true,
                failed_at_ms: Some(1_000),
                failure_count: Some(3),
                safe_message: Some("storage backend unavailable".to_owned()),
                operator_action: "cache refresh failed with a retryable storage failure".to_owned(),
            }),
        };
        let list = AdminVfsCacheRepairTargetListResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            targets: vec![target.clone()],
            page: PageInfo {
                limit: 50,
                offset: 0,
                returned: 1,
            },
        };
        let preview = AdminVfsCacheRepairTargetPreviewResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            target,
            plan,
        };

        let list_value = serde_json::to_value(&list).unwrap();
        let preview_value = serde_json::to_value(&preview).unwrap();
        let body = format!("{list_value}{preview_value}");

        assert_eq!(
            list_value["targets"][0]["target_ref"],
            "vfsrt_0123456789abcdef0123456789abcdef"
        );
        assert_eq!(list_value["targets"][0]["scheme"], "webdav");
        assert_eq!(list_value["targets"][0]["operation"], "list");
        assert_eq!(preview_value["plan"]["status"], "executable");
        assert_eq!(preview_value["plan"]["readiness"]["api_executable"], true);
        assert_eq!(
            preview_value["plan"]["readiness"]["reasons"][0],
            "refresh_cache_executable"
        );
        assert_eq!(
            preview_value["plan"]["executable_action"]["route_key"],
            "storageVfsCacheRepairTargetRefreshCache"
        );
        assert_eq!(
            preview_value["plan"]["executable_action"]["route_path"],
            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache"
        );
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("source_locator"));
        assert!(!body.contains("cache_uri"));
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Movies/Demo.mkv"));
        assert!(!body.contains("cache-etag-secret"));
        assert!(!body.contains("cache-fingerprint-secret"));
        assert!(!body.contains("token=secret"));
    }

    #[test]
    fn admin_vfs_cache_repair_job_commands_serialize_safe_job_facts_and_summary() {
        let job_id = nako_core::JobId::new();
        let library_id = LibraryId::new();
        let job = AdminJobListItem {
            id: job_id,
            kind: nako_core::JobKind::VfsCacheRepair,
            status: nako_core::JobStatus::Queued,
            resource_class: "storage.vfs.cache_repair".to_owned(),
            library_id: Some(library_id),
            source_id: None,
            has_input: true,
            has_summary: false,
            has_error: false,
            queued_at: "2026-06-07T00:00:00Z".to_owned(),
            started_at: None,
            completed_at: None,
        };
        let enqueue = AdminVfsCacheRepairEnqueueResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            outcome: AdminVfsCacheRepairEnqueueOutcome::Enqueued,
            job: job.clone(),
        };
        let execute = AdminVfsCacheRepairExecuteResponse {
            admin_api_version: "v1".to_owned(),
            public_api_version: "2025-01-01".to_owned(),
            job: AdminJobListItem {
                status: nako_core::JobStatus::Succeeded,
                has_summary: true,
                ..job
            },
            summary: AdminVfsCacheRepairJobSummary {
                action: AdminVfsCacheRepairAction::RefreshCache,
                source_scheme: "webdav".to_owned(),
                operation: VfsCacheOperation::Stat,
                classification: AdminVfsCacheRepairClassification::RetryableRefreshFailure,
                failure_class: Some(StorageFailureClass::Unavailable),
                failed_at_ms: 1_000,
                failure_count: 2,
                refreshed_cache_state: Some(AdminVfsCacheRepairCacheState::Fresh),
            },
        };

        let enqueue_value = serde_json::to_value(&enqueue).unwrap();
        let execute_value = serde_json::to_value(&execute).unwrap();
        let body = format!("{enqueue_value}{execute_value}");

        assert_eq!(enqueue_value["outcome"], "enqueued");
        assert_eq!(enqueue_value["job"]["id"], job_id.to_string());
        assert_eq!(enqueue_value["job"]["kind"], "vfs_cache_repair");
        assert_eq!(enqueue_value["job"]["has_input"], true);
        assert_eq!(execute_value["job"]["status"], "succeeded");
        assert_eq!(execute_value["summary"]["action"], "refresh_cache");
        assert_eq!(execute_value["summary"]["source_scheme"], "webdav");
        assert_eq!(execute_value["summary"]["operation"], "stat");
        assert_eq!(
            execute_value["summary"]["classification"],
            "retryable_refresh_failure"
        );
        assert_eq!(execute_value["summary"]["refreshed_cache_state"], "fresh");
        assert!(!body.contains("source_uri"));
        assert!(!body.contains("storage_uri"));
        assert!(!body.contains("cache_uri"));
        assert!(!body.contains("local_path"));
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("Movies/Demo.mkv"));
        assert!(!body.contains("backend.example"));
        assert!(!body.contains("token=secret"));
        assert!(!body.contains("input_json"));
        assert!(!body.contains("summary_json"));
        assert!(!body.contains("uri_digest"));
        assert!(!body.contains("raw backend"));
    }

    #[test]
    fn storage_backend_diagnostics_keeps_runtime_state_summarized() {
        let library_id = LibraryId::new();
        let response = StorageBackendDiagnosticsResponse {
            backends: vec![StorageBackendDiagnostic {
                library_id,
                library_name: "Movies".to_owned(),
                root_uri: "local://<redacted>".to_owned(),
                backend_kind: StorageBackendKind::Local,
                scheme: "local".to_owned(),
                status: StorageBackendStatus::Ready,
                reason: None,
                registry: StorageBackendRegistryDiagnostic {
                    cached: true,
                    stream_permits_available: 8,
                    stream_permits_max: 8,
                    stage_permits_available: 2,
                    stage_permits_max: 2,
                    state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
                },
                health: StorageBackendHealthDiagnostic {
                    consecutive_errors: 0,
                    last_success_at_ms: Some(1_000),
                    last_error_at_ms: None,
                    last_error_class: None,
                    backoff_until_ms: None,
                },
            }],
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["backends"][0]["library_id"], library_id.to_string());
        assert_eq!(value["backends"][0]["backend_kind"], "local");
        assert_eq!(value["backends"][0]["status"], "ready");
        assert_eq!(
            value["backends"][0]["registry"]["state_scope"],
            "process_local"
        );
        assert_eq!(value["backends"][0]["health"]["consecutive_errors"], 0);
        assert_eq!(
            value["backends"][0]["health"]["last_error_class"],
            serde_json::Value::Null
        );
        assert_eq!(
            value["backends"][0]["health"]["backoff_until_ms"],
            serde_json::Value::Null
        );
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
        assert!(!body.contains("webdav:///"));
    }
}
