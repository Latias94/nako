use nako_core::{
    AdminSettingsEffect, AdminSettingsSource, ExternalProvider, JobId, LibraryId, LibraryPreset,
};
use serde::{Deserialize, Serialize};

use crate::metadata_diagnostics::MetadataProviderDiagnosticStatus;

pub const ADMIN_API_VERSION: &str = "v1";

mod access;
mod automation;
mod catalog_governance;
mod incident_bundle;
mod intake;
mod library;
mod managed_artwork;
mod metadata_candidate_review;
mod network;
mod operations;
mod playback;
mod storage;
pub use access::*;
pub use automation::*;
pub use catalog_governance::*;
pub use incident_bundle::*;
pub use intake::*;
pub use library::*;
pub use managed_artwork::*;
pub use metadata_candidate_review::*;
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

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminHardwareAcceleration {
    #[default]
    None,
    Vaapi,
    Nvenc,
    QuickSync,
    Amf,
    VideoToolbox,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminHardwareAccelerationFallback {
    #[default]
    Cpu,
    Fail,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminHardwareAccelerationPolicy {
    pub requested: AdminHardwareAcceleration,
    pub fallback: AdminHardwareAccelerationFallback,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminTranscodeConfigDiagnostics {
    pub hardware_policy: AdminHardwareAccelerationPolicy,
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
    pub operator_readiness: AdminOperatorReadinessSummary,
    pub storage: AdminOverviewStorageSummary,
    pub catalog: AdminOverviewCatalogSummary,
    pub metadata: AdminOverviewMetadataSummary,
    pub runtime: AdminOverviewRuntimeSummary,
    pub source_fingerprint_hash: AdminOverviewSourceFingerprintHashSummary,
    pub startup: AdminOverviewStartupSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub summary: AdminOperatorReadinessSummary,
    pub details: AdminOperatorReadinessDetails,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessDetails {
    pub setup: AdminOperatorReadinessSetupDetail,
    pub media_library_scan: AdminOperatorReadinessMediaLibraryScanDetail,
    pub playback: AdminOperatorReadinessPlaybackDetail,
    pub durable_jobs: AdminOperatorReadinessDurableJobsDetail,
    pub storage: AdminOperatorReadinessStorageDetail,
    pub network: AdminOperatorReadinessNetworkDetail,
    pub backup: AdminOperatorReadinessBackupDetail,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessSetupDetail {
    pub auth_enabled: bool,
    pub token_reference_configured: bool,
    pub exposure_mode: AdminNetworkExposureMode,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessMediaLibraryScanDetail {
    pub configured_libraries: u32,
    pub library_scan: AdminOperatorReadinessLibraryScanPosture,
    pub source_fingerprint_hash: AdminOverviewSourceFingerprintHashSummary,
    pub watch_folder_runtime: AdminOverviewWatchFolderRuntimeSummary,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessLibraryScanPosture {
    pub configured_libraries: u32,
    pub pending_libraries: u32,
    pub failed_libraries: u32,
    pub never_completed_libraries: u32,
    pub succeeded_libraries: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessPlaybackDetail {
    pub readiness: AdminPlaybackReadinessDiagnostics,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessDurableJobsDetail {
    pub queue_pressure: Vec<AdminJobQueuePressureSummary>,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessStorageDetail {
    pub summary: AdminOverviewStorageSummary,
    pub vfs_cache_repair: Option<AdminOperatorReadinessVfsCacheRepairPressure>,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessVfsCacheRepairPressure {
    pub total_unresolved_targets: u32,
    pub primary_classification: AdminVfsCacheRepairClassification,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessNetworkDetail {
    pub readiness: AdminNetworkReadinessDiagnostics,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessBackupDetail {
    pub durable_database_configured: bool,
    pub check: AdminOperatorReadinessCheck,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOverviewStatus {
    Healthy,
    Degraded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessSummary {
    pub status: AdminOperatorReadinessStatus,
    pub checks: Vec<AdminOperatorReadinessCheck>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessCheck {
    pub area: AdminOperatorReadinessArea,
    pub status: AdminOperatorReadinessStatus,
    pub reason: AdminOperatorReadinessReason,
    pub source_reason: Option<String>,
    pub attention_count: u32,
    pub action: Option<AdminOperatorReadinessAction>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOperatorReadinessAction {
    pub route_key: String,
    pub route_path: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOperatorReadinessStatus {
    Ready,
    Degraded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOperatorReadinessArea {
    Setup,
    MediaLibraryScan,
    DurableJobs,
    Playback,
    Storage,
    Network,
    Backup,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminOperatorReadinessReason {
    AuthConfigured,
    AuthTokenReferenceMissing,
    AuthDisabledLocalOnly,
    AuthDisabledRemoteExposure,
    MediaLibraryConfigured,
    NoMediaLibraryConfigured,
    ScanWorkPending,
    ScanRepairPressure,
    WatchFolderRuntimeCoverageGap,
    DurableJobsReady,
    DurableJobsPressure,
    PlaybackReady,
    PlaybackDegraded,
    PlaybackUnavailable,
    StorageReady,
    StorageDegraded,
    StorageUnavailable,
    VfsCacheRepairPressure,
    NetworkReady,
    NetworkDegraded,
    NetworkUnavailable,
    BackupRunbookAvailable,
    BackupNeedsDurableDatabase,
}

impl AdminOperatorReadinessSummary {
    #[must_use]
    pub fn from_checks(checks: Vec<AdminOperatorReadinessCheck>) -> Self {
        let status = if checks
            .iter()
            .any(|check| check.status == AdminOperatorReadinessStatus::Unavailable)
        {
            AdminOperatorReadinessStatus::Unavailable
        } else if checks
            .iter()
            .any(|check| check.status == AdminOperatorReadinessStatus::Degraded)
        {
            AdminOperatorReadinessStatus::Degraded
        } else {
            AdminOperatorReadinessStatus::Ready
        };

        Self { status, checks }
    }
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewCatalogSummary {
    pub governed_items: u32,
    pub unknown_kind_items: u32,
    pub low_confidence_items: u32,
    pub items_with_duplicate_relationships: u32,
    pub items_missing_accepted_provider_mapping: u32,
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

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewSourceFingerprintHashSummary {
    pub total_sources: u64,
    pub fingerprinted_sources: u64,
    pub content_hash_sources: u64,
    pub queued_jobs: u64,
    pub running_jobs: u64,
    pub succeeded_jobs: u64,
    pub failed_jobs: u64,
    pub cancelled_jobs: u64,
    pub claimable_jobs: u64,
    pub delayed_retry_jobs: u64,
    pub oldest_queued_at: Option<String>,
    pub next_retry_at: Option<String>,
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
    pub watch_folder_runtimes_started: u32,
    pub watch_folder_runtime: AdminOverviewWatchFolderRuntimeSummary,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminOverviewWatchFolderRuntimeSummary {
    pub configured_libraries: u32,
    pub realtime_enabled_libraries: u32,
    pub started_libraries: u32,
    pub skipped_libraries: u32,
    pub diagnostics: Vec<AdminWatchFolderRuntimeCoverageDiagnostic>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderRuntimeCoverageDiagnostic {
    pub library_id: LibraryId,
    pub library_name: String,
    pub root_scheme: Option<String>,
    pub root_ref_redacted: String,
    pub status: AdminWatchFolderRuntimeCoverageStatus,
    pub safe_reason: String,
    pub last_tick: Option<AdminWatchFolderRuntimeTickDiagnostic>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminWatchFolderRuntimeCoverageStatus {
    Started,
    Disabled,
    UnsupportedRoot,
    MissingRoot,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderRuntimeTickDiagnostic {
    pub monitored: bool,
    pub status: AdminWatchFolderRuntimeOutcomeStatus,
    pub ready_candidates: u64,
    pub inspecting_candidates: u64,
    pub blocked_candidates: u64,
    pub recorded_candidates: u64,
    pub newly_ready_candidates: u64,
    pub observed_candidates: u64,
    pub suppressed_candidates: u64,
    pub active_suppressions: u64,
    pub failure_count: u64,
    pub enqueue_scan: bool,
    pub enqueue_reason: AdminWatchFolderIntakeEnqueueReason,
    pub scan_admission_status: AdminWatchFolderScanAdmissionStatus,
    pub scan_job_id: Option<JobId>,
    pub reused_existing_scan: bool,
    pub backoff_required: bool,
    pub discovery_failures: Vec<AdminWatchFolderRuntimeFailureDiagnostic>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminWatchFolderScanAdmissionStatus {
    NotAdmitted,
    Enqueued,
    ReusedQueued,
    ReusedRunning,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminWatchFolderRuntimeFailureDiagnostic {
    pub ref_redacted: String,
    pub safe_message: String,
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
            operator_readiness: AdminOperatorReadinessSummary::from_checks(vec![
                AdminOperatorReadinessCheck {
                    area: AdminOperatorReadinessArea::Setup,
                    status: AdminOperatorReadinessStatus::Ready,
                    reason: AdminOperatorReadinessReason::AuthConfigured,
                    source_reason: None,
                    attention_count: 0,
                    action: None,
                },
                AdminOperatorReadinessCheck {
                    area: AdminOperatorReadinessArea::Storage,
                    status: AdminOperatorReadinessStatus::Degraded,
                    reason: AdminOperatorReadinessReason::StorageDegraded,
                    source_reason: Some("degraded_backend".to_owned()),
                    attention_count: 1,
                    action: Some(AdminOperatorReadinessAction {
                        route_key: "storageVfsCacheRepairTargets".to_owned(),
                        route_path: "/admin/v1/storage/vfs-cache/repair/targets".to_owned(),
                    }),
                },
                AdminOperatorReadinessCheck {
                    area: AdminOperatorReadinessArea::DurableJobs,
                    status: AdminOperatorReadinessStatus::Degraded,
                    reason: AdminOperatorReadinessReason::DurableJobsPressure,
                    source_reason: Some("queued_work".to_owned()),
                    attention_count: 2,
                    action: Some(AdminOperatorReadinessAction {
                        route_key: "jobs".to_owned(),
                        route_path: "/admin/v1/jobs".to_owned(),
                    }),
                },
            ]),
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
            catalog: AdminOverviewCatalogSummary {
                governed_items: 2,
                unknown_kind_items: 1,
                low_confidence_items: 1,
                items_with_duplicate_relationships: 1,
                items_missing_accepted_provider_mapping: 2,
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
            source_fingerprint_hash: AdminOverviewSourceFingerprintHashSummary {
                total_sources: 3,
                fingerprinted_sources: 2,
                content_hash_sources: 1,
                queued_jobs: 4,
                running_jobs: 1,
                succeeded_jobs: 8,
                failed_jobs: 1,
                cancelled_jobs: 0,
                claimable_jobs: 2,
                delayed_retry_jobs: 1,
                oldest_queued_at: Some("2026-06-05T00:00:00.000Z".to_owned()),
                next_retry_at: Some("2026-06-05T00:05:00.000Z".to_owned()),
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
                watch_folder_runtimes_started: 0,
                watch_folder_runtime: AdminOverviewWatchFolderRuntimeSummary {
                    configured_libraries: 2,
                    realtime_enabled_libraries: 1,
                    started_libraries: 1,
                    skipped_libraries: 1,
                    diagnostics: vec![AdminWatchFolderRuntimeCoverageDiagnostic {
                        library_id,
                        library_name: "Movies".to_owned(),
                        root_scheme: Some("local".to_owned()),
                        root_ref_redacted: "local://<redacted>".to_owned(),
                        status: AdminWatchFolderRuntimeCoverageStatus::Started,
                        safe_reason: "local watch-folder runtime started".to_owned(),
                        last_tick: Some(AdminWatchFolderRuntimeTickDiagnostic {
                            monitored: true,
                            status: AdminWatchFolderRuntimeOutcomeStatus::Degraded,
                            ready_candidates: 2,
                            inspecting_candidates: 1,
                            blocked_candidates: 0,
                            recorded_candidates: 3,
                            newly_ready_candidates: 2,
                            observed_candidates: 4,
                            suppressed_candidates: 1,
                            active_suppressions: 1,
                            failure_count: 1,
                            enqueue_scan: true,
                            enqueue_reason:
                                AdminWatchFolderIntakeEnqueueReason::NewStableCandidates,
                            scan_admission_status:
                                AdminWatchFolderScanAdmissionStatus::ReusedRunning,
                            scan_job_id: Some(JobId::new()),
                            reused_existing_scan: true,
                            backoff_required: true,
                            discovery_failures: vec![AdminWatchFolderRuntimeFailureDiagnostic {
                                ref_redacted: "local://<redacted>".to_owned(),
                                safe_message: "storage error: Io".to_owned(),
                            }],
                        }),
                    }],
                },
            },
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["admin_api_version"], "v1");
        assert_eq!(value["public_api_version"], API_VERSION);
        assert_eq!(value["status"], "healthy");
        assert_eq!(value["operator_readiness"]["status"], "degraded");
        assert_eq!(value["operator_readiness"]["checks"][0]["area"], "setup");
        assert_eq!(
            value["operator_readiness"]["checks"][0]["reason"],
            "auth_configured"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][1]["source_reason"],
            "degraded_backend"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][1]["action"]["route_key"],
            "storageVfsCacheRepairTargets"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][2]["area"],
            "durable_jobs"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][2]["reason"],
            "durable_jobs_pressure"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][2]["action"]["route_key"],
            "jobs"
        );
        assert_eq!(
            value["operator_readiness"]["checks"][2]["action"]["route_path"],
            "/admin/v1/jobs"
        );
        assert_eq!(value["storage"]["ready_backends"], 1);
        assert_eq!(value["storage"]["backends"][0]["status"], "ready");
        assert_eq!(value["catalog"]["governed_items"], 2);
        assert_eq!(value["catalog"]["items_with_duplicate_relationships"], 1);
        assert_eq!(value["metadata"]["providers"][0]["provider"], "tmdb");
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["status"],
            "started"
        );
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["root_ref_redacted"],
            "local://<redacted>"
        );
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["last_tick"]["enqueue_reason"],
            "new_stable_candidates"
        );
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["last_tick"]["status"],
            "degraded"
        );
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["last_tick"]["scan_admission_status"],
            "reused_running"
        );
        assert_eq!(
            value["startup"]["watch_folder_runtime"]["diagnostics"][0]["last_tick"]["discovery_failures"]
                [0]["ref_redacted"],
            "local://<redacted>"
        );
        assert_eq!(value["source_fingerprint_hash"]["fingerprinted_sources"], 2);
        assert_eq!(value["source_fingerprint_hash"]["content_hash_sources"], 1);
        assert_eq!(value["source_fingerprint_hash"]["claimable_jobs"], 2);
        assert_eq!(
            value["source_fingerprint_hash"]["oldest_queued_at"],
            "2026-06-05T00:00:00.000Z"
        );
        assert!(!body.contains("secret"));
        assert!(!body.contains("token"));
        assert!(!body.contains("root_uri"));
        assert!(!body.contains("output_path"));
        assert!(!body.contains("ProviderRawResponse"));
        assert!(!body.contains("source:v1:content_hash"));
        assert!(!body.contains("locator"));
        assert!(!body.contains("uri_redacted"));
        assert!(!body.contains("local:///"));
        assert!(!body.contains("C:\\"));
    }
}
