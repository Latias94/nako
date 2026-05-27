use nako_core::{
    AdminSettingsEffect, AdminSettingsSource, ExternalProvider, LibraryId, LibraryPreset,
};
use nako_transcode::HardwareAccelerationPolicy;
use serde::{Deserialize, Serialize};

use crate::metadata_diagnostics::MetadataProviderDiagnosticStatus;

pub const ADMIN_API_VERSION: &str = "v1";

mod access;
mod automation;
mod catalog_governance;
mod intake;
mod library;
mod managed_artwork;
mod network;
mod operations;
mod playback;
mod storage;
pub use access::*;
pub use automation::*;
pub use catalog_governance::*;
pub use intake::*;
pub use library::*;
pub use managed_artwork::*;
pub use network::*;
pub use operations::*;
pub use playback::*;
pub use storage::*;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminServerConfigDiagnosticsResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub auth: AdminAuthConfigDiagnostics,
    pub network: AdminNetworkAccessDiagnostics,
    pub database: AdminDatabaseConfigDiagnostics,
    pub runtime: AdminRuntimeConfigDiagnostics,
    pub libraries: Vec<AdminLibraryConfigDiagnostics>,
    pub metadata: AdminMetadataConfigDiagnostics,
    pub transcode: AdminTranscodeConfigDiagnostics,
    pub staging: AdminConfigStagingDiagnostics,
    pub playback: AdminConfigPlaybackDiagnostics,
    pub artwork: AdminArtworkConfigDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessSummaryResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub mode: AdminAccessMode,
    pub principal: AdminAccessPrincipalSummary,
    pub auth: AdminAccessAuthSummary,
    pub readiness: AdminAccessCapabilitySummary,
    pub library_access: AdminLibraryAccessSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAccessMode {
    SingleAdmin,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessPrincipalSummary {
    pub principal_id: String,
    pub display_name: String,
    pub principal_kind: AdminAccessPrincipalKind,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAccessPrincipalKind {
    LocalAdmin,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessAuthSummary {
    pub enabled: bool,
    pub token_reference_configured: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessCapabilitySummary {
    pub single_admin_mode: AdminAccessCapabilityState,
    pub user_accounts: AdminAccessCapabilityState,
    pub roles: AdminAccessCapabilityState,
    pub library_access_policy: AdminAccessCapabilityState,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminAccessCapabilityState {
    Active,
    Planned,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessSummary {
    pub configured_libraries: u32,
    pub libraries: Vec<AdminLibraryAccessSummaryEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessSummaryEntry {
    pub library_id: LibraryId,
    pub library_name: String,
    pub preset: LibraryPreset,
    pub backend_kind: StorageBackendKind,
    pub access: AdminLibraryAccessLevel,
    pub reason: AdminLibraryAccessReason,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminLibraryAccessLevel {
    Manage,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminLibraryAccessReason {
    SingleAdminMode,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAuthConfigDiagnostics {
    pub enabled: bool,
    pub token_env: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminDatabaseConfigDiagnostics {
    pub configured_backend_kind: String,
    pub active_backend_kind: String,
    pub url_scheme: String,
    pub runtime_supported: bool,
    pub migrated_on_startup: bool,
    pub capabilities: AdminDatabaseBackendCapabilitiesDiagnostics,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminDatabaseBackendCapabilitiesDiagnostics {
    pub lifecycle: bool,
    pub libraries: bool,
    pub jobs: bool,
    pub job_leases: bool,
    pub media: bool,
    pub scan_commits: bool,
    pub metadata: bool,
    pub catalog: bool,
    pub playback_sessions: bool,
    pub playback_state: bool,
    pub transcode_sessions: bool,
    pub event_outbox: bool,
    pub addons: bool,
    pub automation: bool,
    pub managed_artwork: bool,
    pub vfs_cache: bool,
    pub webhooks: bool,
    pub search_index: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminRuntimeConfigDiagnostics {
    pub listen_addr: String,
    pub scan_concurrency: usize,
    pub probe_concurrency: usize,
    pub metadata_concurrency: usize,
    pub remux_concurrency: usize,
    pub webhook_concurrency: usize,
    pub remux_timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryConfigDiagnostics {
    pub id: LibraryId,
    pub name: String,
    pub preset: LibraryPreset,
    pub backend_kind: StorageBackendKind,
    pub root_scheme: String,
    pub has_webdav_password_env: bool,
    pub webdav_timeout_ms: Option<u64>,
    pub webdav_max_attempts: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataConfigDiagnostics {
    pub raw_cache_retention_ms: u64,
    pub raw_cache_cleanup_on_startup: bool,
    pub raw_cache_cleanup_interval_ms: u64,
    pub runtime: AdminMetadataRuntimeConfigDiagnostics,
    pub maintenance_policies: u32,
    pub providers: Vec<AdminMetadataProviderConfigDiagnostics>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataRuntimeConfigDiagnostics {
    pub timeout_ms: u64,
    pub max_attempts: u32,
    pub min_interval_ms: u64,
    pub concurrency: usize,
    pub user_agent: String,
    pub has_proxy: bool,
    pub circuit_breaker_failures: u32,
    pub circuit_breaker_backoff_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataProviderConfigDiagnostics {
    pub provider: ExternalProvider,
    pub enabled: bool,
    pub token_env: Option<String>,
    pub api_key_env: Option<String>,
    pub has_api_base_url: bool,
    pub has_image_base_url: bool,
    pub language: Option<String>,
    pub include_adult: bool,
    pub header_count: u32,
    pub secret_header_count: u32,
    pub has_provider_runtime_override: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpdateMetadataRawCacheSettingsRequest {
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminMetadataRawCacheSettingsResponse {
    pub admin_api_version: String,
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
    pub source: AdminSettingsSource,
    pub effect: AdminSettingsEffect,
    pub updated_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTranscodeConfigDiagnostics {
    pub hardware_policy: HardwareAccelerationPolicy,
    pub cpu_concurrency: usize,
    pub gpu_concurrency: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminConfigStagingDiagnostics {
    pub max_bytes: u64,
    pub retention_ms: u64,
    pub cleanup_on_startup: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminConfigPlaybackDiagnostics {
    pub remote_stream_concurrency: usize,
    pub remote_stage_concurrency: usize,
    pub transcode_artifact_retention_ms: u64,
    pub transcode_artifact_cleanup_on_startup: bool,
    pub hls_segment_cleanup_enabled: bool,
    pub hls_segment_keep_ms: u64,
    pub transcode_throttle_enabled: bool,
    pub transcode_throttle_delay_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminArtworkConfigDiagnostics {
    pub artifact_root_configured: bool,
    pub fetch_timeout_ms: u64,
    pub fetch_max_attempts: u32,
    pub fetch_max_bytes: u64,
    pub fetch_concurrency: usize,
    pub ingest_worker_enabled: bool,
    pub ingest_worker_idle_ms: u64,
    pub fetch_user_agent: String,
    pub has_fetch_proxy: bool,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub status: AdminOverviewStatus,
    pub storage: AdminOverviewStorageSummary,
    pub metadata: AdminOverviewMetadataSummary,
    pub runtime: AdminOverviewRuntimeSummary,
    pub startup: AdminOverviewStartupSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOverviewStatus {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStorageSummary {
    pub total_backends: u32,
    pub ready_backends: u32,
    pub degraded_backends: u32,
    pub unavailable_backends: u32,
    pub backends: Vec<AdminOverviewStorageBackendSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStorageBackendSummary {
    pub library_id: LibraryId,
    pub library_name: String,
    pub backend_kind: StorageBackendKind,
    pub status: StorageBackendStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewMetadataSummary {
    pub total_providers: u32,
    pub available_providers: u32,
    pub disabled_providers: u32,
    pub unavailable_providers: u32,
    pub providers: Vec<AdminOverviewMetadataProviderSummary>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewMetadataProviderSummary {
    pub provider: ExternalProvider,
    pub status: MetadataProviderDiagnosticStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewRuntimeSummary {
    pub active_tasks: u32,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub succeeded_jobs: u64,
    pub cancelled_jobs: u64,
    pub failed_jobs: u64,
    pub shutdown_requested: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewStartupSummary {
    pub configured_libraries: u32,
    pub recovered_transcode_sessions: u64,
    pub recovered_jobs: u64,
    pub staging_deleted_records: u32,
    pub staging_deleted_files: u32,
    pub metadata_raw_cache_deleted: u64,
    pub metadata_lifecycle_tasks_started: u32,
    pub artwork_ingest_worker_started: bool,
    pub addon_event_scheduler_started: bool,
}

#[cfg(test)]
mod tests {
    use crate::{
        metadata_diagnostics::MetadataProviderDiagnosticStatus, public_client::API_VERSION,
    };

    use super::*;

    #[test]
    fn admin_overview_response_serializes_safe_summary_fields() {
        let library_id = LibraryId::new();
        let response = AdminOverviewResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            status: AdminOverviewStatus::Healthy,
            storage: AdminOverviewStorageSummary {
                total_backends: 1,
                ready_backends: 1,
                degraded_backends: 0,
                unavailable_backends: 0,
                backends: vec![AdminOverviewStorageBackendSummary {
                    library_id,
                    library_name: "Movies".to_owned(),
                    backend_kind: StorageBackendKind::Local,
                    status: StorageBackendStatus::Ready,
                }],
            },
            metadata: AdminOverviewMetadataSummary {
                total_providers: 1,
                available_providers: 1,
                disabled_providers: 0,
                unavailable_providers: 0,
                providers: vec![AdminOverviewMetadataProviderSummary {
                    provider: nako_core::ExternalProvider::Tmdb,
                    status: MetadataProviderDiagnosticStatus::Available,
                }],
            },
            runtime: AdminOverviewRuntimeSummary {
                active_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                succeeded_jobs: 0,
                cancelled_jobs: 0,
                failed_jobs: 0,
                shutdown_requested: false,
            },
            startup: AdminOverviewStartupSummary {
                configured_libraries: 1,
                recovered_transcode_sessions: 0,
                recovered_jobs: 0,
                staging_deleted_records: 0,
                staging_deleted_files: 0,
                metadata_raw_cache_deleted: 0,
                metadata_lifecycle_tasks_started: 0,
                artwork_ingest_worker_started: false,
                addon_event_scheduler_started: false,
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["public_api_version"], API_VERSION);
        assert_eq!(value["status"], "healthy");
        assert_eq!(value["storage"]["ready_backends"], 1);
        assert_eq!(value["storage"]["backends"][0]["status"], "ready");
        assert_eq!(value["metadata"]["providers"][0]["provider"], "tmdb");
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ProviderRawResponse"));
    }
}
