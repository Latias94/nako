use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryId, StagingManifestId, StagingManifestRecord, StagingPurpose, StagingState,
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
    pub cleanup_on_startup: bool,
    pub retention_ms: u64,
    pub startup_deleted_records: u32,
    pub startup_deleted_files: u32,
    pub cleanup_candidate_records: u32,
    pub cleanup_candidate_bytes: u64,
    pub process_cached_backends: u32,
    pub vfs_cache: AdminVfsCacheSummary,
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
