use serde::{Deserialize, Serialize};

use super::{
    AdminJobQueuePressureSummary, AdminNetworkExposureMode, AdminNetworkReadinessDiagnostics,
    AdminOverviewResponse, AdminPlaybackRuntimeDiagnosticsResponse,
    AdminPlaybackSupportEvidenceResponse, AdminStorageStagingSummary,
    AdminVfsCacheRepairActionPlan,
};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub generated_at_ms: i64,
    pub artifact: AdminIncidentBundleArtifactSummary,
    pub overview: AdminOverviewResponse,
    pub system: AdminIncidentBundleSystemPosture,
    pub network: AdminIncidentBundleNetworkPosture,
    pub playback: AdminIncidentBundlePlaybackPosture,
    pub storage: AdminIncidentBundleStoragePosture,
    pub jobs: AdminIncidentBundleJobQueuePosture,
    pub redaction: AdminIncidentBundleRedactionSummary,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleArtifactSummary {
    pub format: AdminIncidentBundleFormat,
    pub zip_archive_included: bool,
    pub upload_transport_included: bool,
    pub unbounded_logs_included: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminIncidentBundleFormat {
    JsonOnly,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleSystemPosture {
    pub auth_enabled: bool,
    pub database: AdminIncidentBundleDatabasePosture,
    pub runtime: AdminIncidentBundleRuntimePosture,
    pub libraries: AdminIncidentBundleLibraryPosture,
    pub metadata: AdminIncidentBundleMetadataPosture,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleDatabasePosture {
    pub configured_backend_kind: String,
    pub active_backend_kind: String,
    pub url_scheme: String,
    pub runtime_supported: bool,
    pub migrated_on_startup: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleRuntimePosture {
    pub scan_concurrency: usize,
    pub probe_concurrency: usize,
    pub metadata_concurrency: usize,
    pub remux_concurrency: usize,
    pub webhook_concurrency: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleLibraryPosture {
    pub configured_count: u32,
    pub local_count: u32,
    pub webdav_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleMetadataPosture {
    pub provider_count: u32,
    pub enabled_provider_count: u32,
    pub disabled_provider_count: u32,
    pub providers_with_secret_reference_count: u32,
    pub providers_with_runtime_override_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleNetworkPosture {
    pub exposure_mode: AdminNetworkExposureMode,
    pub readiness: AdminNetworkReadinessDiagnostics,
    pub external_endpoint_configured: bool,
    pub external_endpoint_scheme: Option<String>,
    pub trusted_proxy_headers_enabled: bool,
    pub trusted_proxy_source_count: u32,
    pub allowed_origin_count: u32,
    pub tunnel_provider_count: u32,
    pub tunnel_providers_with_endpoint_count: u32,
    pub tunnel_providers_with_token_reference_count: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundlePlaybackPosture {
    pub runtime: AdminPlaybackRuntimeDiagnosticsResponse,
    pub support_evidence: AdminPlaybackSupportEvidenceResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleStoragePosture {
    pub staging: AdminStorageStagingSummary,
    pub vfs_cache_repair_action_plan: AdminVfsCacheRepairActionPlan,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleJobQueuePosture {
    pub queue_pressure: Vec<AdminJobQueuePressureSummary>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminIncidentBundleRedactionSummary {
    pub status: AdminIncidentBundleRedactionStatus,
    pub raw_paths_redacted: bool,
    pub locators_redacted: bool,
    pub tokens_redacted: bool,
    pub credentials_redacted: bool,
    pub ffmpeg_command_lines_redacted: bool,
    pub provider_payloads_redacted: bool,
    pub backend_urls_redacted: bool,
    pub query_strings_redacted: bool,
    pub raw_job_payloads_redacted: bool,
    pub unbounded_logs_redacted: bool,
}

impl AdminIncidentBundleRedactionSummary {
    #[must_use]
    pub const fn complete() -> Self {
        Self {
            status: AdminIncidentBundleRedactionStatus::Complete,
            raw_paths_redacted: true,
            locators_redacted: true,
            tokens_redacted: true,
            credentials_redacted: true,
            ffmpeg_command_lines_redacted: true,
            provider_payloads_redacted: true,
            backend_urls_redacted: true,
            query_strings_redacted: true,
            raw_job_payloads_redacted: true,
            unbounded_logs_redacted: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdminIncidentBundleRedactionStatus {
    Complete,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admin::{
        ADMIN_API_VERSION, AdminHardwareAcceleration, AdminHardwareAccelerationFallback,
        AdminHardwareAccelerationPolicy, AdminIncidentBundleDatabasePosture,
        AdminIncidentBundleJobQueuePosture, AdminIncidentBundleLibraryPosture,
        AdminIncidentBundleMetadataPosture, AdminIncidentBundleNetworkPosture,
        AdminIncidentBundlePlaybackPosture, AdminIncidentBundleRuntimePosture,
        AdminIncidentBundleStoragePosture, AdminJobQueuePressureSummary,
        AdminNetworkReadinessCheck, AdminNetworkReadinessCheckName,
        AdminNetworkReadinessDiagnostics, AdminNetworkReadinessReason,
        AdminOperatorReadinessSummary, AdminOverviewCatalogSummary, AdminOverviewMetadataSummary,
        AdminOverviewResponse, AdminOverviewRuntimeSummary,
        AdminOverviewSourceFingerprintHashSummary, AdminOverviewStartupSummary,
        AdminOverviewStatus, AdminOverviewStorageSummary, AdminOverviewWatchFolderRuntimeSummary,
        AdminPlaybackArtifactLifecycleDiagnostics, AdminPlaybackFfmpegDiagnostics,
        AdminPlaybackHardwareDiagnostics, AdminPlaybackPolicyDiagnostics,
        AdminPlaybackReadinessCheck, AdminPlaybackReadinessCheckName,
        AdminPlaybackReadinessDiagnostics, AdminPlaybackReadinessReason,
        AdminPlaybackRemoteBudgetDiagnostics, AdminPlaybackRemuxRuntimeDiagnostics,
        AdminPlaybackResourcePressureDiagnostics, AdminPlaybackRuntimeDiagnosticsResponse,
        AdminPlaybackRuntimeStatus, AdminPlaybackStagingDiagnostics,
        AdminPlaybackSupportEvidenceResponse, AdminPlaybackSupportHardwareEvidence,
        AdminPlaybackSupportRedactionEvidence, AdminPlaybackSupportRuntimeEvidence,
        AdminPlaybackSupportSubject, AdminPlaybackThrottleDiagnostics,
        AdminPlaybackTranscodeBudgetDiagnostics, AdminStorageStagingPressureStatus,
        AdminStorageStagingPressureSummary, AdminStorageStagingSummary,
        AdminTranscodePipelineReadiness, AdminTranscodePipelineReadinessReason,
        AdminTranscodePipelineReadinessStatus, AdminVfsCacheRepairAction,
        AdminVfsCacheRepairActionBoundary, AdminVfsCacheRepairActionPlan,
        AdminVfsCacheRepairActionPlanReason, AdminVfsCacheRepairActionPlanStatus,
        AdminVfsCacheRepairActionReadiness, AdminVfsCacheSummary, StorageBackendRuntimeStateScope,
    };
    use crate::public_client::API_VERSION;
    use nako_core::{JobKind, JobStatus};

    #[test]
    fn incident_bundle_redaction_summary_serializes_all_sensitive_families() {
        let value = serde_json::to_value(AdminIncidentBundleRedactionSummary::complete()).unwrap();

        assert_eq!(value["status"], "complete");
        assert_eq!(value["raw_paths_redacted"], true);
        assert_eq!(value["locators_redacted"], true);
        assert_eq!(value["tokens_redacted"], true);
        assert_eq!(value["credentials_redacted"], true);
        assert_eq!(value["ffmpeg_command_lines_redacted"], true);
        assert_eq!(value["provider_payloads_redacted"], true);
        assert_eq!(value["backend_urls_redacted"], true);
        assert_eq!(value["query_strings_redacted"], true);
        assert_eq!(value["raw_job_payloads_redacted"], true);
        assert_eq!(value["unbounded_logs_redacted"], true);
    }

    #[test]
    fn incident_bundle_response_serializes_support_artifact_without_sensitive_families() {
        let response = AdminIncidentBundleResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            generated_at_ms: 1_779_667_200_000,
            artifact: AdminIncidentBundleArtifactSummary {
                format: AdminIncidentBundleFormat::JsonOnly,
                zip_archive_included: false,
                upload_transport_included: false,
                unbounded_logs_included: false,
            },
            overview: AdminOverviewResponse {
                admin_api_version: ADMIN_API_VERSION.to_owned(),
                public_api_version: API_VERSION.to_owned(),
                status: AdminOverviewStatus::Healthy,
                operator_readiness: AdminOperatorReadinessSummary::from_checks(Vec::new()),
                storage: AdminOverviewStorageSummary {
                    total_backends: 1,
                    ready_backends: 1,
                    degraded_backends: 0,
                    unavailable_backends: 0,
                    backends: Vec::new(),
                },
                catalog: AdminOverviewCatalogSummary::default(),
                metadata: AdminOverviewMetadataSummary {
                    total_providers: 1,
                    available_providers: 1,
                    disabled_providers: 0,
                    unavailable_providers: 0,
                    providers: Vec::new(),
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
                source_fingerprint_hash: AdminOverviewSourceFingerprintHashSummary::default(),
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
                        configured_libraries: 1,
                        realtime_enabled_libraries: 0,
                        started_libraries: 0,
                        skipped_libraries: 0,
                        diagnostics: Vec::new(),
                    },
                },
            },
            system: AdminIncidentBundleSystemPosture {
                auth_enabled: true,
                database: AdminIncidentBundleDatabasePosture {
                    configured_backend_kind: "sqlite".to_owned(),
                    active_backend_kind: "sqlite".to_owned(),
                    url_scheme: "sqlite".to_owned(),
                    runtime_supported: true,
                    migrated_on_startup: true,
                },
                runtime: AdminIncidentBundleRuntimePosture {
                    scan_concurrency: 2,
                    probe_concurrency: 2,
                    metadata_concurrency: 2,
                    remux_concurrency: 1,
                    webhook_concurrency: 2,
                },
                libraries: AdminIncidentBundleLibraryPosture {
                    configured_count: 2,
                    local_count: 1,
                    webdav_count: 1,
                },
                metadata: AdminIncidentBundleMetadataPosture {
                    provider_count: 2,
                    enabled_provider_count: 1,
                    disabled_provider_count: 1,
                    providers_with_secret_reference_count: 1,
                    providers_with_runtime_override_count: 0,
                },
            },
            network: AdminIncidentBundleNetworkPosture {
                exposure_mode: AdminNetworkExposureMode::TunnelProvider,
                readiness: AdminNetworkReadinessDiagnostics::from_checks(vec![
                    AdminNetworkReadinessCheck::ready(
                        AdminNetworkReadinessCheckName::ExternalEndpoint,
                        AdminNetworkReadinessReason::Ready,
                    ),
                ]),
                external_endpoint_configured: true,
                external_endpoint_scheme: Some("https".to_owned()),
                trusted_proxy_headers_enabled: true,
                trusted_proxy_source_count: 1,
                allowed_origin_count: 1,
                tunnel_provider_count: 1,
                tunnel_providers_with_endpoint_count: 1,
                tunnel_providers_with_token_reference_count: 1,
            },
            playback: AdminIncidentBundlePlaybackPosture {
                runtime: empty_runtime(),
                support_evidence: empty_support(),
            },
            storage: AdminIncidentBundleStoragePosture {
                staging: AdminStorageStagingSummary {
                    configured_max_bytes: 1024,
                    used_manifest_bytes: 128,
                    pressure: AdminStorageStagingPressureSummary {
                        status: AdminStorageStagingPressureStatus::Healthy,
                        used_ratio_milli: Some(125),
                        total_records: 0,
                        in_flight_records: 0,
                        failed_records: 0,
                        unknown_size_records: 0,
                        active_leases: 0,
                        ffmpeg_input_records: 0,
                        probe_input_records: 0,
                    },
                    policy_slices: Vec::new(),
                    purpose_state_summaries: Vec::new(),
                    cleanup_purpose_state_summaries: Vec::new(),
                    cleanup_on_startup: false,
                    retention_ms: 0,
                    startup_deleted_records: 0,
                    startup_deleted_files: 0,
                    cleanup_candidate_records: 0,
                    cleanup_candidate_bytes: 0,
                    process_cached_backends: 0,
                    vfs_cache: AdminVfsCacheSummary {
                        object_count: 0,
                        listing_count: 0,
                        failure_count: 0,
                        stale_object_count: 0,
                        stale_listing_count: 0,
                        last_failure_at_ms: None,
                        repair: None,
                    },
                },
                vfs_cache_repair_action_plan: AdminVfsCacheRepairActionPlan {
                    status: AdminVfsCacheRepairActionPlanStatus::NoAction,
                    action: AdminVfsCacheRepairAction::None,
                    readiness: AdminVfsCacheRepairActionReadiness {
                        status: AdminVfsCacheRepairActionPlanStatus::NoAction,
                        api_executable: false,
                        reasons: vec![AdminVfsCacheRepairActionPlanReason::NoRepairDiagnostic],
                    },
                    boundary: AdminVfsCacheRepairActionBoundary {
                        refreshes_vfs_cache: false,
                        changes_backend_configuration: false,
                        requires_manual_failure_inspection: false,
                        deletes_cache_entries: false,
                        writes_library_files: false,
                        starts_durable_job: false,
                    },
                    executable_action: None,
                    repair: None,
                },
            },
            jobs: AdminIncidentBundleJobQueuePosture {
                queue_pressure: vec![AdminJobQueuePressureSummary {
                    kind: JobKind::LibraryScan,
                    status: JobStatus::Queued,
                    resource_class: "disk.scan".to_owned(),
                    count: 1,
                    claimable_count: 1,
                    delayed_retry_count: 0,
                    oldest_queued_at: None,
                    next_attempt_at: None,
                }],
            },
            redaction: AdminIncidentBundleRedactionSummary::complete(),
        };

        let value = serde_json::to_value(&response).unwrap();
        let body = value.to_string();

        assert_eq!(value["artifact"]["format"], "json_only");
        assert_eq!(value["system"]["libraries"]["configured_count"], 2);
        assert_eq!(
            value["network"]["external_endpoint_scheme"],
            serde_json::json!("https")
        );
        assert_eq!(value["redaction"]["tokens_redacted"], true);

        for forbidden in [
            "token_env",
            "api_key_env",
            "credential",
            "password",
            "external_base_url",
            "trusted_proxy_sources",
            "allowed_origins",
            "public_url",
            "backend_url",
            "query_string",
            "source_uri",
            "source_locator",
            "local_path",
            "output_path",
            "input_json",
            "summary_json",
            "payload_json",
            "raw_provider_response",
            "provider_payload",
            "ffmpeg_command",
        ] {
            assert!(
                !body.contains(forbidden),
                "incident bundle leaked forbidden term: {forbidden}"
            );
        }
    }

    fn empty_runtime() -> AdminPlaybackRuntimeDiagnosticsResponse {
        AdminPlaybackRuntimeDiagnosticsResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            readiness: AdminPlaybackReadinessDiagnostics::from_checks(vec![
                AdminPlaybackReadinessCheck::ready(
                    AdminPlaybackReadinessCheckName::PlaybackPolicy,
                    AdminPlaybackReadinessReason::PlaybackPolicyReady,
                ),
            ]),
            policy: AdminPlaybackPolicyDiagnostics::ready(),
            profile_presets: Vec::new(),
            ffmpeg: AdminPlaybackFfmpegDiagnostics {
                probe_status: AdminPlaybackRuntimeStatus::Ready,
                has_probe_error: false,
                hardware_capability_count: 0,
                available_gpu_capabilities: 0,
            },
            hardware: AdminPlaybackHardwareDiagnostics {
                policy: AdminHardwareAccelerationPolicy {
                    requested: AdminHardwareAcceleration::None,
                    fallback: AdminHardwareAccelerationFallback::Cpu,
                },
                pipeline: AdminTranscodePipelineReadiness {
                    status: AdminTranscodePipelineReadinessStatus::Ready,
                    reason: AdminTranscodePipelineReadinessReason::CpuRequested,
                    requested: AdminHardwareAcceleration::None,
                    selected: AdminHardwareAcceleration::None,
                    fallback_used: false,
                },
                capabilities: Vec::new(),
            },
            transcode: AdminPlaybackTranscodeBudgetDiagnostics {
                configured_cpu_slots: 1,
                configured_gpu_slots: 0,
                effective_cpu_slots: 1,
                effective_gpu_slots: 0,
                selected_hls_slots: 1,
            },
            remux: AdminPlaybackRemuxRuntimeDiagnostics {
                max_concurrent_sessions: 1,
                timeout_ms: 30_000,
            },
            resource_pressure: AdminPlaybackResourcePressureDiagnostics {
                classes: Vec::new(),
            },
            remote_playback: AdminPlaybackRemoteBudgetDiagnostics {
                backend_count: 0,
                stream_permits_available: 1,
                stream_permits_max: 1,
                stage_permits_available: 1,
                stage_permits_max: 1,
                state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
            },
            staging: AdminPlaybackStagingDiagnostics {
                max_bytes: 1024,
                retention_ms: 60_000,
                cleanup_on_startup: false,
                startup_deleted_records: 0,
                startup_deleted_files: 0,
            },
            artifact_lifecycle: AdminPlaybackArtifactLifecycleDiagnostics {
                transcode_artifact_retention_ms: 60_000,
                transcode_artifact_cleanup_on_startup: false,
                hls_segment_cleanup_enabled: true,
                hls_segment_keep_ms: 60_000,
                startup_examined_artifacts: 0,
                startup_deleted_artifacts: 0,
                startup_deleted_files: 0,
                startup_deleted_directories: 0,
                startup_deleted_bytes: 0,
                startup_skipped_security: 0,
            },
            throttle: AdminPlaybackThrottleDiagnostics {
                enabled: false,
                delay_ms: 0,
            },
        }
    }

    fn empty_support() -> AdminPlaybackSupportEvidenceResponse {
        let runtime = empty_runtime();

        AdminPlaybackSupportEvidenceResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            subject: AdminPlaybackSupportSubject {
                session_id: None,
                source_id: None,
            },
            session: None,
            source: None,
            runtime: AdminPlaybackSupportRuntimeEvidence {
                readiness: runtime.readiness,
                policy: runtime.policy,
                ffmpeg: runtime.ffmpeg,
                hardware: AdminPlaybackSupportHardwareEvidence {
                    policy: runtime.hardware.policy,
                    selected_acceleration: runtime.hardware.pipeline.selected,
                    fallback_used: runtime.hardware.pipeline.fallback_used,
                    capability_count: 0,
                    unavailable_capabilities: Vec::new(),
                },
                transcode: runtime.transcode,
                remux: runtime.remux,
                remote_playback: runtime.remote_playback,
                staging: runtime.staging,
                artifact_lifecycle: runtime.artifact_lifecycle,
                throttle: runtime.throttle,
            },
            redaction: AdminPlaybackSupportRedactionEvidence {
                paths_redacted: true,
                source_references_redacted: true,
                ffmpeg_commands_redacted: true,
                stderr_redacted: true,
                credentials_redacted: true,
            },
        }
    }
}
