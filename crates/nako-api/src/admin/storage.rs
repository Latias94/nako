use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryId, StagingManifestId, StagingManifestRecord, StagingPurpose, StagingState,
    StorageBackendHealthRecord, StorageBackendHealthStatus, StorageCircuitBreakerState,
    StorageFailureClass,
};
use serde::{Deserialize, Serialize};

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
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminStorageStagingRecord {
    pub id: StagingManifestId,
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
