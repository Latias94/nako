use std::collections::{HashMap, HashSet};

use axum::{
    Extension, Json, Router,
    extract::Request,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post, put},
};
use nako_api::{
    admin::{
        ADMIN_API_VERSION, AdminAccessAuthSummary, AdminAccessCapabilityState,
        AdminAccessCapabilitySummary, AdminAccessMode, AdminAccessPrincipalKind,
        AdminAccessPrincipalSummary, AdminAccessSummaryResponse, AdminAccessUserListResponse,
        AdminAccessUserRecord, AdminAccessUserResponse, AdminAcquisitionIntakeCandidateDiagnostic,
        AdminAcquisitionIntakeCandidateListResponse, AdminArtworkConfigDiagnostics,
        AdminAuthConfigDiagnostics, AdminCatalogGovernanceItem,
        AdminCatalogGovernanceItemListResponse, AdminCatalogGovernanceProviderMappingReviewRequest,
        AdminConfigPlaybackDiagnostics, AdminConfigStagingDiagnostics,
        AdminCreateInvitationRequest, AdminCreateInvitationResponse, AdminCreateUserRequest,
        AdminDatabaseBackendCapabilitiesDiagnostics, AdminDatabaseConfigDiagnostics,
        AdminGeneratedArtifactMetadataApplyOutcomeListResponse,
        AdminGeneratedArtifactMetadataApplyOutcomeResponse,
        AdminGeneratedArtifactMetadataApplyPlanResponse,
        AdminGeneratedArtifactMetadataApplyRecoveryResponse,
        AdminGeneratedArtifactMetadataApplyRequest, AdminGeneratedArtifactMetadataApplyResponse,
        AdminGeneratedArtifactMetadataBulkApplyBatchResponse,
        AdminGeneratedArtifactMetadataBulkApplyPlanRequest,
        AdminGeneratedArtifactMetadataBulkApplyPlanResponse,
        AdminGeneratedArtifactMetadataBulkApplyRequest, AdminGeneratedArtifactProposal,
        AdminGeneratedArtifactProposalListResponse, AdminGeneratedArtifactReviewPlanResponse,
        AdminGeneratedArtifactReviewRequest, AdminGeneratedArtifactReviewResponse,
        AdminIncidentBundleArtifactSummary, AdminIncidentBundleDatabasePosture,
        AdminIncidentBundleFormat, AdminIncidentBundleJobQueuePosture,
        AdminIncidentBundleLibraryPosture, AdminIncidentBundleMetadataPosture,
        AdminIncidentBundleNetworkPosture, AdminIncidentBundlePlaybackPosture,
        AdminIncidentBundleRedactionSummary, AdminIncidentBundleResponse,
        AdminIncidentBundleRuntimePosture, AdminIncidentBundleStoragePosture,
        AdminIncidentBundleSystemPosture, AdminInvitationListResponse, AdminInvitationRecord,
        AdminInvitationResponse, AdminJobCancelRequestResponse, AdminJobListItem,
        AdminJobListResponse, AdminLibraryAccessLevel, AdminLibraryAccessPolicyDeleteResponse,
        AdminLibraryAccessPolicyListResponse, AdminLibraryAccessPolicyRecord,
        AdminLibraryAccessPolicyResponse, AdminLibraryAccessReason, AdminLibraryAccessSummary,
        AdminLibraryAccessSummaryEntry, AdminLibraryConfigDiagnostics,
        AdminMetadataCandidateReviewApplyRequest, AdminMetadataCandidateReviewBatchApplyRequest,
        AdminMetadataCandidateReviewBatchCreateRequest,
        AdminMetadataCandidateReviewBatchPlanRequest, AdminMetadataCandidateReviewBatchResponse,
        AdminMetadataCandidateReviewRelatedHierarchyApplyRequest,
        AdminMetadataCandidateReviewRelatedHierarchyPlanRequest, AdminMetadataConfigDiagnostics,
        AdminMetadataProviderConfigDiagnostics, AdminMetadataRuntimeConfigDiagnostics,
        AdminNetworkAccessDiagnostics, AdminNetworkExposureMode,
        AdminNetworkExternalEndpointDiagnostics, AdminNetworkReadinessCheck,
        AdminNetworkReadinessCheckName, AdminNetworkReadinessDiagnostics,
        AdminNetworkReadinessReason, AdminNetworkReadinessStatus, AdminOperatorReadinessAction,
        AdminOperatorReadinessArea, AdminOperatorReadinessCheck, AdminOperatorReadinessReason,
        AdminOperatorReadinessStatus, AdminOperatorReadinessSummary, AdminOriginPolicyDiagnostics,
        AdminOutboxEventListItem, AdminOutboxEventListResponse,
        AdminOverviewMetadataProviderSummary, AdminOverviewMetadataSummary, AdminOverviewResponse,
        AdminOverviewRuntimeSummary, AdminOverviewSourceFingerprintHashSummary,
        AdminOverviewStartupSummary, AdminOverviewStatus, AdminOverviewStorageBackendSummary,
        AdminOverviewStorageSummary, AdminOverviewWatchFolderRuntimeSummary,
        AdminPlaybackArtifactLifecycleDiagnostics, AdminPlaybackFfmpegDiagnostics,
        AdminPlaybackHardwareCapability, AdminPlaybackHardwareCapabilityReason,
        AdminPlaybackHardwareDeviceInitialization, AdminPlaybackHardwareDeviceInitializationStatus,
        AdminPlaybackHardwareDiagnostics, AdminPlaybackHardwareEncoderDiscovery,
        AdminPlaybackHardwareEncoderDiscoveryStatus, AdminPlaybackHardwareSmokeProbe,
        AdminPlaybackHardwareSmokeProbeStatus, AdminPlaybackHardwareStageCapability,
        AdminPlaybackPolicyDiagnostics, AdminPlaybackReadinessCheck,
        AdminPlaybackReadinessCheckName, AdminPlaybackReadinessDiagnostics,
        AdminPlaybackReadinessReason, AdminPlaybackReadinessStatus,
        AdminPlaybackRemoteBudgetDiagnostics, AdminPlaybackRemuxRuntimeDiagnostics,
        AdminPlaybackResourceClass, AdminPlaybackResourceClassPressure,
        AdminPlaybackResourceEnforcement, AdminPlaybackResourcePressureDiagnostics,
        AdminPlaybackRuntimeDiagnosticsResponse, AdminPlaybackRuntimeStatus,
        AdminPlaybackSessionListItem, AdminPlaybackSessionListResponse,
        AdminPlaybackStagingDiagnostics, AdminPlaybackSupportEvidenceResponse,
        AdminPlaybackSupportHardwareCapabilityEvidence, AdminPlaybackSupportHardwareEvidence,
        AdminPlaybackSupportRedactionEvidence, AdminPlaybackSupportRuntimeEvidence,
        AdminPlaybackSupportSessionEvidence, AdminPlaybackSupportSourceEvidence,
        AdminPlaybackSupportSubject, AdminPlaybackThrottleDiagnostics,
        AdminPlaybackTranscodeBudgetDiagnostics, AdminRendererAdapterDiagnostics,
        AdminRendererAdapterKind, AdminRendererAdapterReason, AdminRendererAdapterStatus,
        AdminRendererControlPlane, AdminRendererDiscoveryMode, AdminRendererMediaTransport,
        AdminRendererReadinessDiagnostics, AdminRendererRuntimeDiagnosticsResponse,
        AdminRendererSessionDiagnostics, AdminRendererSessionSummary, AdminReplaceUserRolesRequest,
        AdminRuntimeConfigDiagnostics, AdminServerConfigDiagnosticsResponse,
        AdminSetLocalPasswordRequest, AdminSourceDuplicateReconciliationApplyRequest,
        AdminSourceDuplicateReconciliationApplyResponse,
        AdminSourceDuplicateReconciliationPlanResponse, AdminSourceFingerprintHashEnqueueRequest,
        AdminSourceFingerprintHashMode, AdminSourceFingerprintHashRetryRequest,
        AdminStorageBackendHealthDiagnostic, AdminStorageBackendHealthDiagnosticsResponse,
        AdminStorageBackendHealthResetResponse, AdminStorageStagingCleanupPurposeStateSummary,
        AdminStorageStagingDiagnosticsResponse, AdminStorageStagingPolicySlice,
        AdminStorageStagingPressureStatus, AdminStorageStagingPressureSummary,
        AdminStorageStagingPurposeStateSummary, AdminStorageStagingRecord,
        AdminStorageStagingSummary, AdminTranscodeConfigDiagnostics,
        AdminTranscodePipelineReadiness, AdminTranscodePipelineReadinessStatus,
        AdminTrustedProxyDiagnostics, AdminTunnelProviderDiagnostics, AdminTunnelProviderKind,
        AdminUpdateLibraryMetadataProfileRequest, AdminUpdateMetadataRawCacheSettingsRequest,
        AdminUpdatePlaybackRuntimeSettingsRequest, AdminUpdateUserStatusRequest,
        AdminUpsertLibraryAccessPolicyRequest, AdminVfsCacheRefreshResponse,
        AdminVfsCacheRepairAction, AdminVfsCacheRepairActionBoundary,
        AdminVfsCacheRepairActionPlan, AdminVfsCacheRepairActionPlanReason,
        AdminVfsCacheRepairActionPlanResponse, AdminVfsCacheRepairActionPlanStatus,
        AdminVfsCacheRepairActionReadiness, AdminVfsCacheRepairAutomationBlockReason,
        AdminVfsCacheRepairAutomationBlockedTarget, AdminVfsCacheRepairAutomationBoundary,
        AdminVfsCacheRepairAutomationEligibleTarget, AdminVfsCacheRepairAutomationEnqueueRequest,
        AdminVfsCacheRepairAutomationEnqueueResponse, AdminVfsCacheRepairAutomationJob,
        AdminVfsCacheRepairAutomationPlanResponse, AdminVfsCacheRepairAutomationPolicyReport,
        AdminVfsCacheRepairAutomationPolicyRequest, AdminVfsCacheRepairCacheState,
        AdminVfsCacheRepairClassification, AdminVfsCacheRepairClassificationCount,
        AdminVfsCacheRepairDiagnostic, AdminVfsCacheRepairEnqueueOutcome,
        AdminVfsCacheRepairEnqueueRequest, AdminVfsCacheRepairEnqueueResponse,
        AdminVfsCacheRepairExecutableAction, AdminVfsCacheRepairExecuteResponse,
        AdminVfsCacheRepairJobSummary, AdminVfsCacheRepairRemediationActionGroup,
        AdminVfsCacheRepairRemediationPlanBoundary, AdminVfsCacheRepairRemediationPlanResponse,
        AdminVfsCacheRepairRetryRequest, AdminVfsCacheRepairTarget,
        AdminVfsCacheRepairTargetListResponse, AdminVfsCacheRepairTargetPreviewResponse,
        AdminVfsCacheSummary, AdminWatchFolderDiscoveryFailure, AdminWatchFolderDiscoveryRequest,
        AdminWatchFolderDiscoveryResponse, AdminWatchFolderIntakeEnqueueReason,
        AdminWatchFolderRuntimeCoverageDiagnostic, AdminWatchFolderRuntimeCoverageStatus,
        AdminWatchFolderRuntimeFailureDiagnostic, AdminWatchFolderRuntimeTickDiagnostic,
        AdminWatchFolderScanAdmissionStatus, AdminWatchFolderSuppression, JobResponse,
        StorageBackendDiagnosticsResponse, StorageBackendKind, StorageBackendRuntimeStateScope,
        StorageBackendStatus,
    },
    metadata_diagnostics::{MetadataProviderDiagnosticStatus, MetadataProviderDiagnosticsResponse},
    public_client::{API_VERSION, ClientErrorCode, ErrorResponse, page_info_from_request},
};
use nako_core::{
    ArtworkCandidateId, AutomationArtifactId, ExternalProvider,
    GeneratedArtifactMetadataApplyOutcomeId, GeneratedArtifactMetadataApplyRecoveryAttention,
    GeneratedArtifactMetadataApplyRecoveryFilter, GeneratedArtifactMetadataBulkApplyBatchId,
    ImageKind, JobId, LibraryAccessPolicy, LibraryAccessPolicyFilter, LibraryAccessPolicyScope,
    LibraryId, ManagedArtworkArtifactId, ManagedArtworkIngestId, MediaItemId, MediaSourceId,
    MetadataCandidateReviewBatchId, MetadataCandidateReviewId, MetadataCandidateReviewQueueFilter,
    MetadataCandidateReviewStatus, NakoError, PageRequest, PlaybackTargetKind,
    PlaybackTargetTransportAuth, ProviderMappingId, RendererSessionRecord, RendererSessionState,
    RoleAssignment, TranscodeSessionId, User, UserId, UserInvitationId, UserPrincipalId, UserRole,
    UserStatus,
};
use nako_db::DatabaseBackendCapabilities;
use nako_library::{
    SourceFingerprintHashMode, WatchFolderIntakeEnqueueReason, WatchFolderIntakePlanInput,
    plan_watch_folder_intake,
};
use nako_transcode::{
    HardwareAccelerationCapability, HardwareDeviceInitializationStatus,
    HardwareEncoderDiscoveryStatus, HardwareSmokeProbeStatus, TranscodeRuntimeInventoryStatus,
};
use nako_vfs::{ObjectCacheState, StorageUri, VfsCacheRepairAction, VfsCacheRepairClassification};
use serde::{Deserialize, Serialize};

use crate::{
    api_mapping::{
        admin_hardware_acceleration, admin_hardware_pipeline_stage, admin_hardware_policy,
        admin_transcode_pipeline_readiness,
    },
    app::{
        EnqueueSourceFingerprintHashRequest, EnqueueVfsCacheRepairTargetOutcome,
        LibraryScanTraceContext, NakoApp, RetrySourceFingerprintHashRequest,
        RetryVfsCacheRepairJobRequest, RuntimeSupervisorDiagnostics,
        SourceDuplicateReconciliationApplyRequest as AppSourceDuplicateReconciliationApplyRequest,
        SourceDuplicateReconciliationPlanRequest, StagingBudgetPolicySlice,
        StagingCleanupPurposeStateSummary, StagingPurposeStateSummary,
        StorageStagingPressureStatus, VfsCacheRepairActionBoundary, VfsCacheRepairActionPlanReason,
        VfsCacheRepairActionPlanReport, VfsCacheRepairActionPlanStatus,
        VfsCacheRepairAutomationBlockReason as AppVfsCacheRepairAutomationBlockReason,
        VfsCacheRepairAutomationEnqueueOutcome as AppVfsCacheRepairAutomationEnqueueOutcome,
        VfsCacheRepairAutomationEnqueueReport, VfsCacheRepairAutomationJobReport,
        VfsCacheRepairAutomationPolicy, VfsCacheRepairAutomationPolicyReport,
        VfsCacheRepairCommandOutput, VfsCacheRepairExecutableRoute,
        VfsCacheRepairJobSummary as AppVfsCacheRepairJobSummary, VfsCacheRepairReadinessPressure,
        VfsCacheRepairRefreshActionReport, VfsCacheRepairRemediationActionGroupReport,
        VfsCacheRepairRemediationClassificationCountReport,
        VfsCacheRepairRemediationPlanBoundary as AppVfsCacheRepairRemediationPlanBoundary,
        VfsCacheRepairRemediationPlanReport, VfsCacheRepairTargetPreviewReport,
        VfsCacheRepairTargetReport, WatchFolderRuntimeCoverageDiagnostic,
        WatchFolderRuntimeCoverageReport, WatchFolderRuntimeCoverageStatus,
        WatchFolderRuntimeTickDiagnostic,
        WatchFolderScanAdmissionStatus as AppWatchFolderScanAdmissionStatus,
        storage_staging_pressure_status as app_storage_staging_pressure_status,
    },
    config::{
        LocalLibraryConfig, MetadataProviderConfig, MetadataProviderRuntimeConfig,
        NetworkAccessConfig, NetworkExposureMode as ConfigNetworkExposureMode,
        TunnelProviderConfig, TunnelProviderKind as ConfigTunnelProviderKind,
    },
};

const STORAGE_VFS_CACHE_REPAIR_REFRESH_CACHE_ROUTE_KEY: &str = "storageVfsCacheRepairRefreshCache";
const STORAGE_VFS_CACHE_REPAIR_REFRESH_CACHE_ROUTE_PATH: &str =
    "/admin/v1/storage/vfs-cache/repair/refresh-cache";
const STORAGE_VFS_CACHE_REPAIR_TARGET_REFRESH_CACHE_ROUTE_KEY: &str =
    "storageVfsCacheRepairTargetRefreshCache";
const STORAGE_VFS_CACHE_REPAIR_TARGET_REFRESH_CACHE_ROUTE_PATH: &str =
    "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache";
const ADMIN_JOBS_ROUTE_KEY: &str = "jobs";
const ADMIN_JOBS_ROUTE_PATH: &str = "/admin/v1/jobs";
const ADMIN_PLAYBACK_RUNTIME_ROUTE_KEY: &str = "playbackRuntime";
const ADMIN_PLAYBACK_RUNTIME_ROUTE_PATH: &str = "/admin/v1/playback/runtime";
const ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_KEY: &str = "storageVfsCacheRepairTargets";
const ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_PATH: &str = "/admin/v1/storage/vfs-cache/repair/targets";
const ADMIN_SYSTEM_CONFIG_ROUTE_KEY: &str = "systemConfig";
const ADMIN_SYSTEM_CONFIG_ROUTE_PATH: &str = "/admin/v1/system/config";

use super::{
    error::ApiResult,
    query::{
        AcquisitionIntakeCandidateListQuery, ArtworkArtifactCleanupQuery,
        ArtworkArtifactLifecycleQuery, ArtworkArtifactRemediationQuery,
        ArtworkArtifactStorageDriftQuery, ArtworkGalleryQuery, CatalogGovernanceItemsQuery,
        JobListQuery, OutboxEventListQuery, PageQuery, PlaybackSessionListQuery,
        PlaybackSupportEvidenceQuery, StorageStagingQuery, parse_u32_filter, parse_u64_filter,
    },
    trace_context::HttpTraceContext,
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/admin/v1/overview", get(get_admin_overview))
        .route(
            "/admin/v1/diagnostics/incident-bundle",
            get(get_admin_incident_bundle),
        )
        .route(
            "/admin/v1/acquisition/intake/candidates",
            get(list_admin_acquisition_intake_candidates),
        )
        .route(
            "/admin/v1/acquisition/intake/watch-folder-discovery",
            post(discover_admin_watch_folder_candidates),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/proposals",
            get(list_admin_generated_artifact_proposals),
        )
        .route(
            "/admin/v1/automation/generated-artifact-apply-outcomes",
            get(list_admin_generated_artifact_metadata_apply_outcomes),
        )
        .route(
            "/admin/v1/automation/generated-artifact-apply-outcomes/{outcome_id}",
            get(get_admin_generated_artifact_metadata_apply_outcome),
        )
        .route(
            "/admin/v1/automation/generated-artifact-apply-recovery",
            get(list_admin_generated_artifact_metadata_apply_recovery),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/metadata-apply-plan",
            post(plan_admin_generated_artifact_metadata_bulk_apply),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/metadata-apply-batches",
            post(create_admin_generated_artifact_metadata_bulk_apply_batch),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/metadata-apply-batches/{batch_id}",
            get(get_admin_generated_artifact_metadata_bulk_apply_batch),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan",
            post(plan_admin_generated_artifact_review),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/{artifact_id}/review",
            post(review_admin_generated_artifact),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply-plan",
            post(plan_admin_generated_artifact_metadata_apply),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/{artifact_id}/metadata-apply",
            post(apply_admin_generated_artifact_metadata),
        )
        .route(
            "/admin/v1/catalog/governance/items",
            get(list_admin_catalog_governance_items),
        )
        .route(
            "/admin/v1/catalog/governance/items/{item_id}",
            get(get_admin_catalog_governance_item_detail),
        )
        .route(
            "/admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review-plan",
            post(plan_admin_catalog_governance_provider_mapping_review),
        )
        .route(
            "/admin/v1/catalog/governance/items/{item_id}/provider-mappings/{mapping_id}/review",
            post(review_admin_catalog_governance_provider_mapping),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews",
            get(list_admin_metadata_candidate_reviews),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/batch-application-plan",
            post(plan_admin_metadata_candidate_review_batch_application),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/batch-apply",
            post(apply_admin_metadata_candidate_review_batch),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/batches",
            post(create_admin_metadata_candidate_review_batch),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/batches/{batch_id}",
            get(get_admin_metadata_candidate_review_batch),
        )
        .route(
            "/admin/v1/metadata/items/{item_id}/candidate-reviews",
            get(list_admin_metadata_candidate_reviews_for_item),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/{review_id}",
            get(get_admin_metadata_candidate_review),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/{review_id}/apply",
            post(apply_admin_metadata_candidate_review),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/application-plan",
            post(plan_admin_metadata_candidate_review_related_hierarchy),
        )
        .route(
            "/admin/v1/metadata/candidate-reviews/{review_id}/related-hierarchy/apply",
            post(apply_admin_metadata_candidate_review_related_hierarchy),
        )
        .route("/admin/v1/events", get(list_admin_outbox_events))
        .route("/admin/v1/jobs", get(list_admin_jobs))
        .route("/admin/v1/jobs/{job_id}/cancel", post(cancel_admin_job))
        .route(
            "/admin/v1/source-fingerprint-hashes",
            post(enqueue_admin_source_fingerprint_hash),
        )
        .route(
            "/admin/v1/source-fingerprint-hashes/jobs/{job_id}/retry",
            post(retry_admin_source_fingerprint_hash_job),
        )
        .route(
            "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-plan",
            get(get_admin_source_duplicate_reconciliation_plan),
        )
        .route(
            "/admin/v1/libraries/{library_id}/sources/{source_id}/duplicate-reconciliation-apply",
            post(apply_admin_source_duplicate_reconciliation),
        )
        .route("/admin/v1/access/summary", get(get_admin_access_summary))
        .route(
            "/admin/v1/access/users",
            get(list_admin_access_users).post(create_admin_access_user),
        )
        .route(
            "/admin/v1/access/invitations",
            get(list_admin_access_invitations).post(create_admin_access_invitation),
        )
        .route(
            "/admin/v1/access/invitations/{invitation_id}/revoke",
            post(revoke_admin_access_invitation),
        )
        .route(
            "/admin/v1/access/users/{user_id}/roles",
            put(replace_admin_access_user_roles),
        )
        .route(
            "/admin/v1/access/users/{user_id}/status",
            patch(update_admin_access_user_status),
        )
        .route(
            "/admin/v1/access/users/{user_id}/local-password",
            put(set_admin_access_user_local_password).delete(delete_admin_access_user_local_password),
        )
        .route(
            "/admin/v1/access/library-policies",
            get(list_admin_library_access_policies)
                .put(upsert_admin_library_access_policy)
                .delete(delete_admin_library_access_policy),
        )
        .route(
            "/admin/v1/libraries/{library_id}/metadata-profile",
            get(get_admin_library_metadata_profile).put(update_admin_library_metadata_profile),
        )
        .route(
            "/admin/v1/libraries/{library_id}/scan",
            post(scan_admin_library),
        )
        .route(
            "/admin/v1/libraries/{library_id}/nfo/import",
            post(import_admin_library_nfo),
        )
        .route(
            "/admin/v1/libraries/{library_id}/nfo/export",
            post(export_admin_library_nfo),
        )
        .route(
            "/admin/v1/artwork/candidates/{candidate_id}/accept",
            post(accept_admin_artwork_candidate),
        )
        .route(
            "/admin/v1/artwork/ingests/process-next",
            post(process_next_admin_artwork_ingest),
        )
        .route(
            "/admin/v1/artwork/ingests/{ingest_id}/requeue",
            post(requeue_admin_artwork_ingest),
        )
        .route(
            "/admin/v1/artwork/artifacts/{artifact_id}/publish",
            post(publish_admin_artwork_artifact),
        )
        .route(
            "/admin/v1/items/{item_id}/artwork",
            get(get_admin_item_artwork_gallery),
        )
        .route(
            "/admin/v1/items/{item_id}/artwork/{kind}/select",
            post(select_admin_item_artwork),
        )
        .route(
            "/admin/v1/items/{item_id}/artwork/{kind}/selection",
            delete(unpublish_admin_item_artwork),
        )
        .route(
            "/admin/v1/artwork/artifacts/lifecycle",
            get(get_admin_artwork_artifact_lifecycle),
        )
        .route(
            "/admin/v1/artwork/artifacts/storage-drift",
            get(get_admin_artwork_artifact_storage_drift),
        )
        .route(
            "/admin/v1/artwork/artifacts/remediation-plan",
            get(get_admin_artwork_artifact_remediation_plan),
        )
        .route(
            "/admin/v1/artwork/artifacts/remediate-stray-files",
            post(remediate_admin_artwork_artifact_stray_files),
        )
        .route(
            "/admin/v1/artwork/artifacts/cleanup",
            post(cleanup_admin_artwork_artifacts),
        )
        .route("/admin/v1/storage/backends", get(list_admin_storage_backends))
        .route(
            "/admin/v1/storage/backends/{backend_key}/circuit-breaker/reset",
            post(reset_admin_storage_backend_circuit_breaker),
        )
        .route("/admin/v1/storage/staging", get(list_admin_storage_staging))
        .route(
            "/admin/v1/storage/vfs-cache/repair/remediation-plan",
            get(get_admin_vfs_cache_repair_remediation_plan),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/automation/plan",
            post(plan_admin_vfs_cache_repair_automation),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/automation/jobs",
            post(enqueue_admin_vfs_cache_repair_automation),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/targets",
            get(list_admin_vfs_cache_repair_targets),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/preview",
            get(get_admin_vfs_cache_repair_target_preview),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/refresh-cache",
            post(refresh_admin_vfs_cache_repair_target),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/targets/{target_ref}/jobs",
            post(enqueue_admin_vfs_cache_repair_target),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/jobs/{job_id}/execute",
            post(execute_admin_vfs_cache_repair_job),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/jobs/{job_id}/retry",
            post(retry_admin_vfs_cache_repair_job),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/action-plan",
            get(get_admin_vfs_cache_repair_action_plan),
        )
        .route(
            "/admin/v1/storage/vfs-cache/repair/refresh-cache",
            post(refresh_admin_vfs_cache),
        )
        .route("/admin/v1/system/config", get(get_admin_system_config))
        .route(
            "/admin/v1/settings/metadata/raw-cache",
            get(get_admin_metadata_raw_cache_settings).put(update_admin_metadata_raw_cache_settings),
        )
        .route(
            "/admin/v1/settings/playback/runtime",
            get(get_admin_playback_runtime_settings).put(update_admin_playback_runtime_settings),
        )
        .route(
            "/admin/v1/playback/runtime",
            get(get_admin_playback_runtime),
        )
        .route(
            "/admin/v1/playback/support",
            get(get_admin_playback_support_evidence),
        )
        .route(
            "/admin/v1/playback/sessions",
            get(list_admin_playback_sessions),
        )
        .route(
            "/admin/v1/playback/renderers",
            get(get_admin_playback_renderers),
        )
        .route_layer(middleware::from_fn(require_admin_principal))
}

async fn require_admin_principal(
    Extension(principal): Extension<nako_core::AuthenticatedPrincipal>,
    request: Request,
    next: Next,
) -> Response {
    if principal.is_administrator() {
        return next.run(request).await;
    }

    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse::new(
            ClientErrorCode::Forbidden,
            "administrator role is required",
        )),
    )
        .into_response()
}

pub(super) async fn accept_admin_artwork_candidate(
    State(app): State<NakoApp>,
    Path(candidate_id): Path<ArtworkCandidateId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().accept_candidate(candidate_id).await?))
}

pub(super) async fn process_next_admin_artwork_ingest(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().process_next().await?))
}

pub(super) async fn requeue_admin_artwork_ingest(
    State(app): State<NakoApp>,
    Path(ingest_id): Path<ManagedArtworkIngestId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().requeue_ingest(ingest_id).await?))
}

pub(super) async fn publish_admin_artwork_artifact(
    State(app): State<NakoApp>,
    Path(artifact_id): Path<ManagedArtworkArtifactId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().publish_artifact(artifact_id).await?))
}

pub(super) async fn get_admin_item_artwork_gallery(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
    Query(query): Query<ArtworkGalleryQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.artwork()
            .item_gallery(item_id, query.into_page()?)
            .await?,
    ))
}

pub(super) async fn select_admin_item_artwork(
    State(app): State<NakoApp>,
    Path((item_id, kind)): Path<(MediaItemId, String)>,
    Json(request): Json<SelectAdminItemArtworkRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.artwork()
            .select_item_artwork(
                item_id,
                parse_admin_artwork_kind(&kind)?,
                request.artifact_id,
            )
            .await?,
    ))
}

pub(super) async fn unpublish_admin_item_artwork(
    State(app): State<NakoApp>,
    Path((item_id, kind)): Path<(MediaItemId, String)>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.artwork()
            .unpublish_item_artwork(item_id, parse_admin_artwork_kind(&kind)?)
            .await?,
    ))
}

pub(super) async fn get_admin_artwork_artifact_lifecycle(
    State(app): State<NakoApp>,
    Query(query): Query<ArtworkArtifactLifecycleQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    Ok(Json(
        app.artwork()
            .artifact_lifecycle_diagnostics(filter, page)
            .await?,
    ))
}

pub(super) async fn list_admin_acquisition_intake_candidates(
    State(app): State<NakoApp>,
    Query(query): Query<AcquisitionIntakeCandidateListQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let diagnostics = app
        .acquisition_intake()
        .list_candidates(filter, page)
        .await?;
    let candidates = diagnostics
        .candidates
        .into_iter()
        .map(admin_acquisition_intake_candidate)
        .collect();

    Ok(Json(AdminAcquisitionIntakeCandidateListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        candidates,
        page: page_info_from_request(
            PageRequest::new(diagnostics.limit, diagnostics.offset),
            diagnostics.returned,
        ),
    }))
}

pub(super) async fn discover_admin_watch_folder_candidates(
    State(app): State<NakoApp>,
    Json(request): Json<AdminWatchFolderDiscoveryRequest>,
) -> ApiResult<impl IntoResponse> {
    let diagnostic = app
        .acquisition_intake()
        .discover_watch_folder_candidates(
            crate::app::acquisition_intake::DiscoverWatchFolderCandidatesRequest {
                target_library_id: request.target_library_id,
                root_uri: request.root_uri.map(parse_admin_storage_uri).transpose()?,
                max_depth: request.max_depth,
            },
        )
        .await?;
    let intake_plan = plan_watch_folder_intake(WatchFolderIntakePlanInput {
        ready_candidates: diagnostic.ready_candidates,
        inspecting_candidates: diagnostic.inspecting_candidates,
        blocked_candidates: diagnostic.blocked_candidates,
        recorded_candidates: diagnostic.recorded_candidates,
        newly_ready_candidates: diagnostic.newly_ready_candidates,
        suppressed_candidates: diagnostic.suppressed_candidates,
        active_suppressions: diagnostic.active_suppressions.len() as u64,
        failure_count: diagnostic.failures.len() as u64,
    });

    Ok(Json(AdminWatchFolderDiscoveryResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        target_library_id: diagnostic.target_library_id,
        root_scheme: diagnostic.root_scheme,
        root_ref_redacted: diagnostic.root_uri_redacted,
        ready_candidates: diagnostic.ready_candidates,
        inspecting_candidates: diagnostic.inspecting_candidates,
        blocked_candidates: diagnostic.blocked_candidates,
        incomplete_candidates: diagnostic.incomplete_candidates,
        unsupported_candidates: diagnostic.unsupported_candidates,
        suppressed_candidates: diagnostic.suppressed_candidates,
        recorded_candidates: diagnostic.recorded_candidates,
        newly_ready_candidates: diagnostic.newly_ready_candidates,
        enqueue_scan: intake_plan.summary.enqueue_scan,
        enqueue_reason: admin_watch_folder_intake_enqueue_reason(intake_plan.enqueue.reason),
        active_suppressions: diagnostic
            .active_suppressions
            .into_iter()
            .map(admin_watch_folder_suppression)
            .collect(),
        failures: diagnostic
            .failures
            .into_iter()
            .map(|failure| AdminWatchFolderDiscoveryFailure {
                ref_redacted: failure.uri_redacted,
                safe_message: failure.safe_message,
            })
            .collect(),
        writes_library: diagnostic.writes_library,
        managed_import_artifacts_created: diagnostic.managed_import_artifacts_created,
        promotion_apply: diagnostic.promotion_apply,
    }))
}

fn admin_watch_folder_intake_enqueue_reason(
    reason: WatchFolderIntakeEnqueueReason,
) -> AdminWatchFolderIntakeEnqueueReason {
    match reason {
        WatchFolderIntakeEnqueueReason::NewStableCandidates => {
            AdminWatchFolderIntakeEnqueueReason::NewStableCandidates
        }
        WatchFolderIntakeEnqueueReason::WaitingForStability => {
            AdminWatchFolderIntakeEnqueueReason::WaitingForStability
        }
        WatchFolderIntakeEnqueueReason::SuppressedCandidates => {
            AdminWatchFolderIntakeEnqueueReason::SuppressedCandidates
        }
        WatchFolderIntakeEnqueueReason::BlockedCandidates => {
            AdminWatchFolderIntakeEnqueueReason::BlockedCandidates
        }
        WatchFolderIntakeEnqueueReason::DiscoveryFailures => {
            AdminWatchFolderIntakeEnqueueReason::DiscoveryFailures
        }
        WatchFolderIntakeEnqueueReason::NoNewStableCandidates => {
            AdminWatchFolderIntakeEnqueueReason::NoNewStableCandidates
        }
    }
}

fn admin_watch_folder_suppression(
    suppression: crate::app::PlannedWatchFolderWriteSuppressionDiagnostic,
) -> AdminWatchFolderSuppression {
    AdminWatchFolderSuppression {
        target_library_id: suppression.target_library_id,
        scope_scheme: suppression.scope_scheme,
        scope_ref_redacted: suppression.scope_ref_redacted,
        owner: suppression.owner,
        reason: suppression.reason,
        expires_at_ms: suppression.expires_at_ms,
        completion: match suppression.completion {
            crate::app::PlannedWatchFolderWriteCompletion::SuppressOnly => {
                "suppress_only".to_owned()
            }
            crate::app::PlannedWatchFolderWriteCompletion::ReconcileScope => {
                "reconcile_scope".to_owned()
            }
        },
    }
}

pub(super) async fn list_admin_generated_artifact_proposals(
    State(app): State<NakoApp>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.try_into()?;
    let proposals = app
        .automation()
        .list_generated_artifact_proposals(page)
        .await?;
    let returned = proposals.len();
    let proposals = proposals
        .into_iter()
        .map(AdminGeneratedArtifactProposal::from_proposal)
        .collect();

    Ok(Json(AdminGeneratedArtifactProposalListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        proposals,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn list_admin_generated_artifact_metadata_apply_outcomes(
    State(app): State<NakoApp>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.try_into()?;
    let outcomes = app
        .automation()
        .list_generated_artifact_metadata_apply_outcomes(page)
        .await?;
    let returned = outcomes.len();
    let outcomes = outcomes
        .into_iter()
        .map(nako_api::admin::AdminGeneratedArtifactMetadataApplyOutcome::from_record)
        .collect();

    Ok(Json(
        AdminGeneratedArtifactMetadataApplyOutcomeListResponse {
            admin_api_version: ADMIN_API_VERSION.to_owned(),
            public_api_version: API_VERSION.to_owned(),
            outcomes,
            page: page_info_from_request(page, returned),
        },
    ))
}

pub(super) async fn get_admin_generated_artifact_metadata_apply_outcome(
    State(app): State<NakoApp>,
    Path(outcome_id): Path<GeneratedArtifactMetadataApplyOutcomeId>,
) -> ApiResult<impl IntoResponse> {
    let outcome = app
        .automation()
        .get_generated_artifact_metadata_apply_outcome(outcome_id)
        .await?;

    Ok(Json(AdminGeneratedArtifactMetadataApplyOutcomeResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        outcome: nako_api::admin::AdminGeneratedArtifactMetadataApplyOutcome::from_record(outcome),
    }))
}

pub(super) async fn list_admin_generated_artifact_metadata_apply_recovery(
    State(app): State<NakoApp>,
    Query(query): Query<AdminGeneratedArtifactApplyRecoveryQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let entries = app
        .automation()
        .list_generated_artifact_metadata_apply_recovery_entries(filter, page)
        .await?;
    let returned = entries.len();

    Ok(Json(
        AdminGeneratedArtifactMetadataApplyRecoveryResponse::from_entries(
            entries,
            page_info_from_request(page, returned),
        ),
    ))
}

pub(super) async fn plan_admin_generated_artifact_review(
    State(app): State<NakoApp>,
    Path(artifact_id): Path<AutomationArtifactId>,
    Json(request): Json<AdminGeneratedArtifactReviewRequest>,
) -> ApiResult<impl IntoResponse> {
    let plan = app
        .automation()
        .plan_generated_artifact_review(artifact_id, request.decision)
        .await?;

    Ok(Json(AdminGeneratedArtifactReviewPlanResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        plan: nako_api::admin::AdminGeneratedArtifactAcceptancePlan::from_plan(plan),
    }))
}

pub(super) async fn review_admin_generated_artifact(
    State(app): State<NakoApp>,
    Path(artifact_id): Path<AutomationArtifactId>,
    Json(request): Json<AdminGeneratedArtifactReviewRequest>,
) -> ApiResult<impl IntoResponse> {
    let result = app
        .automation()
        .review_generated_artifact(artifact_id, request.decision)
        .await?;

    Ok(Json(AdminGeneratedArtifactReviewResponse::from_result(
        result,
    )))
}

pub(super) async fn plan_admin_generated_artifact_metadata_apply(
    State(app): State<NakoApp>,
    Path(artifact_id): Path<AutomationArtifactId>,
) -> ApiResult<impl IntoResponse> {
    let plan = app
        .automation()
        .plan_generated_artifact_metadata_apply(artifact_id)
        .await?;

    Ok(Json(AdminGeneratedArtifactMetadataApplyPlanResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        plan: nako_api::admin::AdminGeneratedArtifactMetadataApplyPlan::from_plan(plan),
    }))
}

pub(super) async fn plan_admin_generated_artifact_metadata_bulk_apply(
    State(app): State<NakoApp>,
    Json(request): Json<AdminGeneratedArtifactMetadataBulkApplyPlanRequest>,
) -> ApiResult<impl IntoResponse> {
    let plan = app
        .automation()
        .plan_generated_artifact_metadata_bulk_apply(
            nako_core::GeneratedArtifactMetadataBulkApplyPlanRequest {
                artifact_ids: request.artifact_ids,
            },
        )
        .await?;

    Ok(Json(
        AdminGeneratedArtifactMetadataBulkApplyPlanResponse::from_plan(plan),
    ))
}

pub(super) async fn create_admin_generated_artifact_metadata_bulk_apply_batch(
    State(app): State<NakoApp>,
    Json(request): Json<AdminGeneratedArtifactMetadataBulkApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    let batch = app
        .automation()
        .create_generated_artifact_metadata_bulk_apply_batch(
            nako_core::GeneratedArtifactMetadataBulkApplyBatchRequest {
                artifact_ids: request.artifact_ids,
                idempotency_key: request.idempotency_key,
            },
        )
        .await?;

    Ok(Json(
        AdminGeneratedArtifactMetadataBulkApplyBatchResponse::from_batch(batch),
    ))
}

pub(super) async fn get_admin_generated_artifact_metadata_bulk_apply_batch(
    State(app): State<NakoApp>,
    Path(batch_id): Path<GeneratedArtifactMetadataBulkApplyBatchId>,
) -> ApiResult<impl IntoResponse> {
    let batch = app
        .automation()
        .get_generated_artifact_metadata_bulk_apply_batch(batch_id)
        .await?;

    Ok(Json(
        AdminGeneratedArtifactMetadataBulkApplyBatchResponse::from_batch(batch),
    ))
}

pub(super) async fn apply_admin_generated_artifact_metadata(
    State(app): State<NakoApp>,
    Path(artifact_id): Path<AutomationArtifactId>,
    Json(request): Json<AdminGeneratedArtifactMetadataApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    let result = app
        .automation()
        .apply_generated_artifact_metadata(nako_core::GeneratedArtifactMetadataApplyRequest {
            artifact_id,
            idempotency_key: request.idempotency_key,
        })
        .await?;

    Ok(Json(
        AdminGeneratedArtifactMetadataApplyResponse::from_result(result),
    ))
}

pub(super) async fn get_admin_library_metadata_profile(
    State(app): State<NakoApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library().get_admin_metadata_profile(library_id).await?,
    ))
}

pub(super) async fn update_admin_library_metadata_profile(
    State(app): State<NakoApp>,
    Path(library_id): Path<LibraryId>,
    Json(request): Json<AdminUpdateLibraryMetadataProfileRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .update_admin_metadata_profile(library_id, request)
            .await?,
    ))
}

pub(super) async fn scan_admin_library(
    State(app): State<NakoApp>,
    Extension(http_trace_context): Extension<HttpTraceContext>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let trace_context = LibraryScanTraceContext::from_request_id(http_trace_context.request_id())?;
    let job = app
        .library_scan()
        .enqueue_library_scan_with_trace_context(library_id, trace_context)
        .await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

pub(super) async fn import_admin_library_nfo(
    State(app): State<NakoApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.nfo().enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

pub(super) async fn export_admin_library_nfo(
    State(app): State<NakoApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.nfo().enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

fn admin_acquisition_intake_candidate(
    diagnostic: crate::app::acquisition_intake::AcquisitionIntakeCandidateDiagnostic,
) -> AdminAcquisitionIntakeCandidateDiagnostic {
    AdminAcquisitionIntakeCandidateDiagnostic {
        id: diagnostic.id,
        target_library_id: diagnostic.target_library_id,
        source_kind: diagnostic.source_kind,
        custom_source_kind: diagnostic.custom_source_kind,
        source_scheme: diagnostic.source_scheme,
        source_ref_redacted: diagnostic.source_uri_redacted,
        source_key_fingerprint: diagnostic.source_key_fingerprint,
        has_display_name: diagnostic.has_display_name,
        has_intended_locator: diagnostic.has_intended_locator,
        size_bytes: diagnostic.size_bytes,
        has_fingerprint: diagnostic.has_fingerprint,
        managed_import_artifact_id: diagnostic.managed_import_artifact_id,
        state: diagnostic.state,
        has_diagnostics: diagnostic.has_diagnostics,
        first_seen_at_ms: diagnostic.first_seen_at_ms,
        last_seen_at_ms: diagnostic.last_seen_at_ms,
        created_at_ms: diagnostic.created_at_ms,
        updated_at_ms: diagnostic.updated_at_ms,
    }
}

fn parse_admin_storage_uri(value: String) -> Result<StorageUri, NakoError> {
    StorageUri::parse(value).map_err(|_err| NakoError::InvalidInput {
        message: "invalid root_uri; expected a storage URI with a scheme".to_owned(),
    })
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct SelectAdminItemArtworkRequest {
    pub(super) artifact_id: ManagedArtworkArtifactId,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct AdminAccessUsersQuery {
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct AdminGeneratedArtifactApplyRecoveryQuery {
    pub(super) attention: Option<GeneratedArtifactMetadataApplyRecoveryAttention>,
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

impl AdminGeneratedArtifactApplyRecoveryQuery {
    fn into_filter_and_page(
        self,
    ) -> Result<(GeneratedArtifactMetadataApplyRecoveryFilter, PageRequest), NakoError> {
        Ok((
            GeneratedArtifactMetadataApplyRecoveryFilter {
                attention: self.attention,
            },
            self.page.try_into()?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
pub(super) struct AdminLibraryAccessPolicyListQuery {
    pub(super) user_id: Option<UserId>,
    pub(super) role: Option<UserRole>,
    pub(super) library_id: Option<LibraryId>,
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

impl AdminLibraryAccessPolicyListQuery {
    fn into_filter_and_page(self) -> Result<(LibraryAccessPolicyFilter, PageRequest), NakoError> {
        Ok((
            LibraryAccessPolicyFilter {
                user_id: self.user_id,
                role: self.role,
                library_id: self.library_id,
            },
            self.page.try_into()?,
        ))
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub(super) struct AdminLibraryAccessPolicyDeleteQuery {
    pub(super) user_id: Option<UserId>,
    pub(super) role: Option<UserRole>,
    pub(super) library_id: LibraryId,
}

impl AdminLibraryAccessPolicyDeleteQuery {
    fn into_scope_and_library(self) -> Result<(LibraryAccessPolicyScope, LibraryId), NakoError> {
        let scope = match (self.user_id, self.role) {
            (Some(user_id), None) => LibraryAccessPolicyScope::User(user_id),
            (None, Some(role)) => LibraryAccessPolicyScope::Role(role),
            (None, None) => {
                return Err(NakoError::InvalidInput {
                    message: "either user_id or role is required".to_owned(),
                });
            }
            (Some(_), Some(_)) => {
                return Err(NakoError::InvalidInput {
                    message: "user_id and role filters are mutually exclusive".to_owned(),
                });
            }
        };

        Ok((scope, self.library_id))
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct AdminMetadataCandidateReviewQueueQuery {
    pub(super) status: Option<MetadataCandidateReviewStatus>,
    pub(super) provider: Option<ExternalProvider>,
    pub(super) limit: Option<String>,
    pub(super) offset: Option<String>,
}

impl AdminMetadataCandidateReviewQueueQuery {
    fn into_filter_and_page(
        self,
    ) -> Result<(MetadataCandidateReviewQueueFilter, PageRequest), NakoError> {
        let page = PageQuery {
            limit: self
                .limit
                .map(|value| parse_u32_filter("limit", value))
                .transpose()?,
            offset: self
                .offset
                .map(|value| parse_u64_filter("offset", value))
                .transpose()?,
        };

        Ok((
            MetadataCandidateReviewQueueFilter {
                status: self.status,
                provider: self.provider,
            },
            page.try_into()?,
        ))
    }
}

async fn admin_access_user_record(
    app: &NakoApp,
    user: User,
) -> Result<AdminAccessUserRecord, NakoError> {
    let roles = app
        .list_role_assignments(user.id)
        .await?
        .into_iter()
        .map(|assignment| assignment.role)
        .collect();
    let bootstrap = is_bootstrap_admin_user(&user);
    let local_password_configured = app.get_local_credential_by_user(user.id).await?.is_some();

    Ok(AdminAccessUserRecord::from_user(
        user,
        roles,
        bootstrap,
        local_password_configured,
    ))
}

async fn admin_access_user_or_not_found(app: &NakoApp, user_id: UserId) -> Result<User, NakoError> {
    app.get_user(user_id)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "user",
            id: user_id.to_string(),
        })
}

fn role_assignments_for_user(
    user_id: UserId,
    roles: &[UserRole],
    granted_at_ms: i64,
) -> Vec<RoleAssignment> {
    roles
        .iter()
        .copied()
        .map(|role| RoleAssignment {
            user_id,
            role,
            granted_at_ms,
        })
        .collect()
}

fn validate_admin_access_roles(roles: &[UserRole]) -> Result<(), NakoError> {
    let mut unique = HashSet::new();
    for role in roles {
        if !unique.insert(*role) {
            return Err(NakoError::InvalidInput {
                message: format!("duplicate Role in request: {}", role.as_str()),
            });
        }
    }

    Ok(())
}

fn validate_access_user_text(field: &str, value: String) -> Result<String, NakoError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("{field} cannot be empty"),
        });
    }
    if value.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: format!("{field} cannot contain control characters"),
        });
    }

    Ok(value)
}

fn is_bootstrap_admin_user(user: &User) -> bool {
    user.principal_id == UserPrincipalId::local_admin()
        && user.id == nako_core::bootstrap_admin_user_id()
}

fn parse_admin_artwork_kind(value: &str) -> Result<ImageKind, NakoError> {
    match value {
        "poster" => Ok(ImageKind::Poster),
        "backdrop" => Ok(ImageKind::Backdrop),
        "logo" => Ok(ImageKind::Logo),
        "thumbnail" => Ok(ImageKind::Thumbnail),
        "banner" => Ok(ImageKind::Banner),
        _ => Err(NakoError::InvalidInput {
            message: format!("unsupported artwork kind path segment: {value}"),
        }),
    }
}

pub(super) async fn get_admin_artwork_artifact_storage_drift(
    State(app): State<NakoApp>,
    Query(query): Query<ArtworkArtifactStorageDriftQuery>,
) -> ApiResult<impl IntoResponse> {
    let (page, file_scan_limit) = query.into_page_and_file_scan_limit()?;
    Ok(Json(
        app.artwork()
            .artifact_storage_drift_diagnostics(page, file_scan_limit)
            .await?,
    ))
}

pub(super) async fn get_admin_artwork_artifact_remediation_plan(
    State(app): State<NakoApp>,
    Query(query): Query<ArtworkArtifactRemediationQuery>,
) -> ApiResult<impl IntoResponse> {
    let (page, file_scan_limit) = query.into_page_and_file_scan_limit()?;
    Ok(Json(
        app.artwork()
            .artifact_remediation_plan(page, file_scan_limit)
            .await?,
    ))
}

pub(super) async fn remediate_admin_artwork_artifact_stray_files(
    State(app): State<NakoApp>,
    Query(query): Query<ArtworkArtifactRemediationQuery>,
) -> ApiResult<impl IntoResponse> {
    let file_scan_limit = query.into_confirmed_file_scan_limit()?;
    Ok(Json(
        app.artwork()
            .cleanup_untracked_artifact_files(file_scan_limit)
            .await?,
    ))
}

pub(super) async fn cleanup_admin_artwork_artifacts(
    State(app): State<NakoApp>,
    Query(query): Query<ArtworkArtifactCleanupQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = query.into_confirmed_page()?;
    Ok(Json(
        app.artwork().cleanup_unselected_artifacts(page).await?,
    ))
}

pub(super) async fn list_admin_catalog_governance_items(
    State(app): State<NakoApp>,
    Query(query): Query<CatalogGovernanceItemsQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let max_confidence_milli = filter.max_confidence_milli;
    let items = app
        .catalog()
        .list_catalog_governance_items(filter, page)
        .await?;
    let returned = items.len();
    let items = items
        .into_iter()
        .map(|item| AdminCatalogGovernanceItem::from_record(item, max_confidence_milli))
        .collect();

    Ok(Json(AdminCatalogGovernanceItemListResponse {
        items,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn get_admin_catalog_governance_item_detail(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .get_catalog_governance_item_detail(item_id)
            .await?,
    ))
}

pub(super) async fn plan_admin_catalog_governance_provider_mapping_review(
    State(app): State<NakoApp>,
    Path((item_id, mapping_id)): Path<(MediaItemId, ProviderMappingId)>,
    Json(request): Json<AdminCatalogGovernanceProviderMappingReviewRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .plan_catalog_governance_provider_mapping_review(item_id, mapping_id, request.decision)
            .await?,
    ))
}

pub(super) async fn review_admin_catalog_governance_provider_mapping(
    State(app): State<NakoApp>,
    Path((item_id, mapping_id)): Path<(MediaItemId, ProviderMappingId)>,
    Json(request): Json<AdminCatalogGovernanceProviderMappingReviewRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .review_catalog_governance_provider_mapping(item_id, mapping_id, request.decision)
            .await?,
    ))
}

pub(super) async fn get_admin_metadata_candidate_review(
    State(app): State<NakoApp>,
    Path(review_id): Path<MetadataCandidateReviewId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .get_admin_metadata_candidate_review(review_id)
            .await?,
    ))
}

pub(super) async fn list_admin_metadata_candidate_reviews_for_item(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.try_into()?;

    Ok(Json(
        app.metadata()
            .list_admin_metadata_candidate_reviews_for_item(item_id, page)
            .await?,
    ))
}

pub(super) async fn list_admin_metadata_candidate_reviews(
    State(app): State<NakoApp>,
    Query(query): Query<AdminMetadataCandidateReviewQueueQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;

    Ok(Json(
        app.metadata()
            .list_admin_metadata_candidate_reviews(filter, page)
            .await?,
    ))
}

pub(super) async fn plan_admin_metadata_candidate_review_batch_application(
    State(app): State<NakoApp>,
    Json(request): Json<AdminMetadataCandidateReviewBatchPlanRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .plan_admin_metadata_candidate_review_batch_application(request)
            .await?,
    ))
}

pub(super) async fn apply_admin_metadata_candidate_review_batch(
    State(app): State<NakoApp>,
    Json(request): Json<AdminMetadataCandidateReviewBatchApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .apply_admin_metadata_candidate_review_batch(request)
            .await?,
    ))
}

pub(super) async fn create_admin_metadata_candidate_review_batch(
    State(app): State<NakoApp>,
    Json(request): Json<AdminMetadataCandidateReviewBatchCreateRequest>,
) -> ApiResult<impl IntoResponse> {
    let batch = app
        .metadata()
        .create_admin_metadata_candidate_review_batch(request)
        .await?;

    Ok(Json(AdminMetadataCandidateReviewBatchResponse::from_batch(
        batch,
    )))
}

pub(super) async fn get_admin_metadata_candidate_review_batch(
    State(app): State<NakoApp>,
    Path(batch_id): Path<MetadataCandidateReviewBatchId>,
) -> ApiResult<impl IntoResponse> {
    let batch = app
        .metadata()
        .get_admin_metadata_candidate_review_batch(batch_id)
        .await?;

    Ok(Json(AdminMetadataCandidateReviewBatchResponse::from_batch(
        batch,
    )))
}

pub(super) async fn apply_admin_metadata_candidate_review(
    State(app): State<NakoApp>,
    Path(review_id): Path<MetadataCandidateReviewId>,
    Json(request): Json<AdminMetadataCandidateReviewApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .apply_admin_metadata_candidate_review(review_id, request)
            .await?,
    ))
}

pub(super) async fn plan_admin_metadata_candidate_review_related_hierarchy(
    State(app): State<NakoApp>,
    Path(review_id): Path<MetadataCandidateReviewId>,
    Json(request): Json<AdminMetadataCandidateReviewRelatedHierarchyPlanRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .plan_admin_metadata_candidate_review_related_hierarchy(review_id, request)
            .await?,
    ))
}

pub(super) async fn apply_admin_metadata_candidate_review_related_hierarchy(
    State(app): State<NakoApp>,
    Path(review_id): Path<MetadataCandidateReviewId>,
    Json(request): Json<AdminMetadataCandidateReviewRelatedHierarchyApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .apply_admin_metadata_candidate_review_related_hierarchy(review_id, request)
            .await?,
    ))
}

pub(super) async fn get_admin_overview(State(app): State<NakoApp>) -> ApiResult<impl IntoResponse> {
    Ok(Json(admin_overview_response(&app).await?))
}

async fn admin_overview_response(app: &NakoApp) -> ApiResult<AdminOverviewResponse> {
    let storage = app.storage().list_storage_backend_diagnostics().await;
    let catalog = app.catalog().catalog_governance_summary().await?;
    let metadata = app.metadata().list_metadata_provider_diagnostics();
    let runtime = app.runtime_diagnostics();
    let source_fingerprint_hash = app.source_hash().admin_overview_summary().await?;
    let vfs_cache_repair_pressure = app.storage().vfs_cache_repair_readiness_pressure().await?;
    let network_readiness = network_readiness_diagnostics(app.config());
    let playback_readiness = admin_playback_runtime_diagnostics(&app).await.readiness;
    let startup = app.startup_report().clone();
    let latest_watch_folder_ticks = app.watch_folder_runtime().latest_tick_diagnostics().await;

    let storage = storage_summary(storage);
    let metadata = metadata_summary(metadata);
    let runtime = runtime_summary(runtime);
    let startup = AdminOverviewStartupSummary {
        configured_libraries: usize_to_u32(startup.configured_libraries),
        recovered_transcode_sessions: startup.recovered_transcode_sessions,
        recovered_jobs: startup.recovered_jobs,
        staging_deleted_records: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_records)),
        staging_deleted_files: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_files)),
        metadata_raw_cache_deleted: startup.metadata_raw_cache_deleted,
        metadata_lifecycle_tasks_started: usize_to_u32(startup.metadata_lifecycle_tasks_started),
        artwork_ingest_worker_started: startup.artwork_ingest_worker_started,
        addon_event_scheduler_started: startup.addon_event_scheduler_started,
        watch_folder_runtimes_started: usize_to_u32(startup.watch_folder_runtimes_started),
        watch_folder_runtime: admin_watch_folder_runtime_summary(
            startup.watch_folder_runtime_coverage,
            latest_watch_folder_ticks,
        ),
    };
    let operator_readiness = operator_readiness_summary(
        app.config(),
        &storage,
        &runtime,
        &source_fingerprint_hash,
        vfs_cache_repair_pressure.as_ref(),
        &startup,
        network_readiness,
        playback_readiness,
    );
    let status = overview_status(&storage, &metadata, &runtime);

    Ok(AdminOverviewResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        status,
        operator_readiness,
        storage,
        catalog,
        metadata,
        runtime,
        source_fingerprint_hash,
        startup,
    })
}

pub(super) async fn get_admin_incident_bundle(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    let overview = admin_overview_response(&app).await?;
    let system_config = admin_system_config_response(&app);
    let playback_runtime = admin_playback_runtime_diagnostics(&app).await;
    let playback_support = admin_playback_support_evidence(&app, None, None).await?;
    let storage = admin_storage_staging_summary(&app).await?;
    let vfs_cache_repair_action_plan = admin_vfs_cache_repair_action_plan(
        app.storage().plan_latest_vfs_cache_repair_action().await?,
    );
    let queue_pressure = app
        .job_queue_pressure_diagnostics()
        .await?
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(AdminIncidentBundleResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        generated_at_ms: crate::app::current_time_ms()?,
        artifact: AdminIncidentBundleArtifactSummary {
            format: AdminIncidentBundleFormat::JsonOnly,
            zip_archive_included: false,
            upload_transport_included: false,
            unbounded_logs_included: false,
        },
        overview,
        system: incident_bundle_system_posture(&system_config),
        network: incident_bundle_network_posture(&system_config.network),
        playback: AdminIncidentBundlePlaybackPosture {
            runtime: playback_runtime,
            support_evidence: playback_support,
        },
        storage: AdminIncidentBundleStoragePosture {
            staging: storage,
            vfs_cache_repair_action_plan,
        },
        jobs: AdminIncidentBundleJobQueuePosture { queue_pressure },
        redaction: AdminIncidentBundleRedactionSummary::complete(),
    }))
}

fn incident_bundle_system_posture(
    config: &AdminServerConfigDiagnosticsResponse,
) -> AdminIncidentBundleSystemPosture {
    let local_count = config
        .libraries
        .iter()
        .filter(|library| library.backend_kind == StorageBackendKind::Local)
        .count();
    let webdav_count = config
        .libraries
        .iter()
        .filter(|library| library.backend_kind == StorageBackendKind::WebDav)
        .count();
    let enabled_provider_count = config
        .metadata
        .providers
        .iter()
        .filter(|provider| provider.enabled)
        .count();
    let providers_with_secret_reference_count = config
        .metadata
        .providers
        .iter()
        .filter(|provider| {
            provider.token_env.is_some()
                || provider.api_key_env.is_some()
                || provider.secret_header_count > 0
        })
        .count();
    let providers_with_runtime_override_count = config
        .metadata
        .providers
        .iter()
        .filter(|provider| provider.has_provider_runtime_override)
        .count();

    AdminIncidentBundleSystemPosture {
        auth_enabled: config.auth.enabled,
        database: AdminIncidentBundleDatabasePosture {
            configured_backend_kind: config.database.configured_backend_kind.clone(),
            active_backend_kind: config.database.active_backend_kind.clone(),
            url_scheme: config.database.url_scheme.clone(),
            runtime_supported: config.database.runtime_supported,
            migrated_on_startup: config.database.migrated_on_startup,
        },
        runtime: AdminIncidentBundleRuntimePosture {
            scan_concurrency: config.runtime.scan_concurrency,
            probe_concurrency: config.runtime.probe_concurrency,
            metadata_concurrency: config.runtime.metadata_concurrency,
            remux_concurrency: config.runtime.remux_concurrency,
            webhook_concurrency: config.runtime.webhook_concurrency,
        },
        libraries: AdminIncidentBundleLibraryPosture {
            configured_count: usize_to_u32(config.libraries.len()),
            local_count: usize_to_u32(local_count),
            webdav_count: usize_to_u32(webdav_count),
        },
        metadata: AdminIncidentBundleMetadataPosture {
            provider_count: usize_to_u32(config.metadata.providers.len()),
            enabled_provider_count: usize_to_u32(enabled_provider_count),
            disabled_provider_count: usize_to_u32(
                config
                    .metadata
                    .providers
                    .len()
                    .saturating_sub(enabled_provider_count),
            ),
            providers_with_secret_reference_count: usize_to_u32(
                providers_with_secret_reference_count,
            ),
            providers_with_runtime_override_count: usize_to_u32(
                providers_with_runtime_override_count,
            ),
        },
    }
}

fn incident_bundle_network_posture(
    network: &AdminNetworkAccessDiagnostics,
) -> AdminIncidentBundleNetworkPosture {
    let tunnel_providers_with_endpoint_count = network
        .tunnel_providers
        .iter()
        .filter(|provider| provider.endpoint_configured)
        .count();
    let tunnel_providers_with_token_reference_count = network
        .tunnel_providers
        .iter()
        .filter(|provider| provider.token_env.is_some())
        .count();

    AdminIncidentBundleNetworkPosture {
        exposure_mode: network.exposure_mode,
        readiness: network.readiness.clone(),
        external_endpoint_configured: network.external_endpoint.configured,
        external_endpoint_scheme: network.external_endpoint.scheme.clone(),
        trusted_proxy_headers_enabled: network.trusted_proxy.headers_enabled,
        trusted_proxy_source_count: network.trusted_proxy.source_count,
        allowed_origin_count: network.origins.allowed_origin_count,
        tunnel_provider_count: usize_to_u32(network.tunnel_providers.len()),
        tunnel_providers_with_endpoint_count: usize_to_u32(tunnel_providers_with_endpoint_count),
        tunnel_providers_with_token_reference_count: usize_to_u32(
            tunnel_providers_with_token_reference_count,
        ),
    }
}

pub(super) async fn get_admin_system_config(
    State(app): State<NakoApp>,
) -> Json<AdminServerConfigDiagnosticsResponse> {
    Json(admin_system_config_response(&app))
}

fn admin_system_config_response(app: &NakoApp) -> AdminServerConfigDiagnosticsResponse {
    let config = app.config();

    AdminServerConfigDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        auth: AdminAuthConfigDiagnostics {
            enabled: config.auth.enabled,
            token_env: config.auth.token_env.clone(),
        },
        network: network_access_diagnostics(config),
        database: database_config_diagnostics(
            config,
            app.database_diagnostics(),
            app.startup_report().database_migrated,
        ),
        runtime: AdminRuntimeConfigDiagnostics {
            listen_addr: config.listen_addr.to_string(),
            scan_concurrency: config.scan_concurrency,
            probe_concurrency: config.probe_concurrency,
            metadata_concurrency: config.metadata_concurrency,
            remux_concurrency: config.remux_concurrency,
            webhook_concurrency: config.webhook_concurrency,
            remux_timeout_ms: config.remux_timeout_ms,
        },
        libraries: config
            .libraries
            .iter()
            .map(library_config_diagnostics)
            .collect(),
        metadata: AdminMetadataConfigDiagnostics {
            raw_cache_retention_ms: config.metadata.raw_cache_retention_ms,
            raw_cache_cleanup_on_startup: config.metadata.maintenance.raw_cache_cleanup_on_startup,
            raw_cache_cleanup_interval_ms: config
                .metadata
                .maintenance
                .raw_cache_cleanup_interval_ms,
            runtime: metadata_runtime_config_diagnostics(&config.metadata.runtime),
            maintenance_policies: usize_to_u32(config.metadata.maintenance.policies.len()),
            providers: config
                .metadata
                .providers
                .iter()
                .map(metadata_provider_config_diagnostics)
                .collect(),
        },
        transcode: AdminTranscodeConfigDiagnostics {
            hardware_policy: admin_hardware_policy(config.transcode.hardware_policy()),
            cpu_concurrency: config.transcode.cpu_concurrency,
            gpu_concurrency: config.transcode.gpu_concurrency,
        },
        staging: AdminConfigStagingDiagnostics {
            max_bytes: config.staging.max_bytes,
            retention_ms: config.staging.retention_ms,
            cleanup_on_startup: config.staging.cleanup_on_startup,
        },
        playback: AdminConfigPlaybackDiagnostics {
            remote_stream_concurrency: config.playback.remote_stream_concurrency,
            remote_stage_concurrency: config.playback.remote_stage_concurrency,
            transcode_artifact_retention_ms: config.playback.transcode_artifact_retention_ms,
            transcode_artifact_cleanup_on_startup: config
                .playback
                .transcode_artifact_cleanup_on_startup,
            hls_segment_cleanup_enabled: config.playback.hls_segment_cleanup_enabled,
            hls_segment_keep_ms: config.playback.hls_segment_keep_ms,
            transcode_throttle_enabled: config.playback.transcode_throttle_enabled,
            transcode_throttle_delay_ms: config.playback.transcode_throttle_delay_ms,
        },
        artwork: AdminArtworkConfigDiagnostics {
            artifact_root_configured: !config.artwork.artifact_root.as_os_str().is_empty(),
            fetch_timeout_ms: config.artwork.fetch_timeout_ms,
            fetch_max_attempts: config.artwork.fetch_max_attempts,
            fetch_max_bytes: config.artwork.fetch_max_bytes,
            fetch_concurrency: config.artwork.fetch_concurrency,
            ingest_worker_enabled: config.artwork.ingest_worker_enabled,
            ingest_worker_idle_ms: config.artwork.ingest_worker_idle_ms,
            fetch_user_agent: config.artwork.fetch_user_agent.clone(),
            has_fetch_proxy: config
                .artwork
                .fetch_proxy
                .as_ref()
                .is_some_and(|proxy| !proxy.is_blank()),
            max_width: config.artwork.max_width,
            max_height: config.artwork.max_height,
        },
    }
}

pub(super) async fn get_admin_access_summary(
    State(app): State<NakoApp>,
    Extension(principal): Extension<nako_core::AuthenticatedPrincipal>,
) -> ApiResult<impl IntoResponse> {
    let config = app.config();
    let user = app.get_user_by_principal(&principal.principal_id).await?;
    let libraries = config
        .libraries
        .iter()
        .map(|library| {
            let diagnostics = library_config_diagnostics(library);
            AdminLibraryAccessSummaryEntry {
                library_id: diagnostics.id,
                library_name: diagnostics.name,
                preset: diagnostics.preset,
                backend_kind: diagnostics.backend_kind,
                access: AdminLibraryAccessLevel::Manage,
                reason: AdminLibraryAccessReason::SingleAdminMode,
            }
        })
        .collect::<Vec<_>>();

    Ok(Json(AdminAccessSummaryResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        mode: AdminAccessMode::SingleAdmin,
        principal: AdminAccessPrincipalSummary {
            principal_id: principal.principal_id.to_string(),
            display_name: user
                .as_ref()
                .map(|user| user.display_name.clone())
                .unwrap_or_else(|| "Local administrator".to_owned()),
            principal_kind: AdminAccessPrincipalKind::LocalAdmin,
        },
        auth: AdminAccessAuthSummary {
            enabled: config.auth.enabled,
            token_reference_configured: config.auth.token_env.is_some(),
        },
        readiness: AdminAccessCapabilitySummary {
            single_admin_mode: AdminAccessCapabilityState::Active,
            user_accounts: AdminAccessCapabilityState::Active,
            roles: AdminAccessCapabilityState::Active,
            library_access_policy: AdminAccessCapabilityState::Active,
        },
        library_access: AdminLibraryAccessSummary {
            configured_libraries: usize_to_u32(libraries.len()),
            libraries,
        },
    }))
}

pub(super) async fn list_admin_access_users(
    State(app): State<NakoApp>,
    Query(query): Query<AdminAccessUsersQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.page.try_into()?;
    let users = app.list_users(page).await?;
    let returned = users.len();
    let mut records = Vec::with_capacity(users.len());

    for user in users {
        records.push(admin_access_user_record(&app, user).await?);
    }

    Ok(Json(AdminAccessUserListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        users: records,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn create_admin_access_user(
    State(app): State<NakoApp>,
    Json(request): Json<AdminCreateUserRequest>,
) -> ApiResult<impl IntoResponse> {
    validate_admin_access_roles(&request.roles)?;
    let now_ms = crate::app::current_time_ms()?;
    let user_id = UserId::new();
    let user = User {
        id: user_id,
        principal_id: UserPrincipalId::new(format!("local-user:{user_id}"))?,
        username: validate_access_user_text("username", request.username)?,
        display_name: validate_access_user_text("display_name", request.display_name)?,
        status: UserStatus::Active,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    let assignments = role_assignments_for_user(user.id, &request.roles, now_ms);

    app.upsert_user(&user).await?;
    app.replace_role_assignments(user.id, &assignments).await?;

    Ok(Json(AdminAccessUserResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        user: admin_access_user_record(&app, user).await?,
    }))
}

pub(super) async fn list_admin_access_invitations(
    State(app): State<NakoApp>,
    Query(query): Query<AdminAccessUsersQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.page.try_into()?;
    let invitations = app.list_user_invitations(page).await?;
    let returned = invitations.len();
    let invitations = invitations
        .into_iter()
        .map(AdminInvitationRecord::from)
        .collect();

    Ok(Json(AdminInvitationListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        invitations,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn create_admin_access_invitation(
    State(app): State<NakoApp>,
    Extension(principal): Extension<nako_core::AuthenticatedPrincipal>,
    Json(request): Json<AdminCreateInvitationRequest>,
) -> ApiResult<impl IntoResponse> {
    validate_admin_access_roles(&request.roles)?;
    let issued = app
        .create_user_invitation(
            principal.user_id,
            request.email_or_username,
            request.roles,
            request.expires_in_ms,
        )
        .await?;

    Ok(Json(AdminCreateInvitationResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        invitation: AdminInvitationRecord::from(issued.invitation),
        token: issued.token,
    }))
}

pub(super) async fn revoke_admin_access_invitation(
    State(app): State<NakoApp>,
    Path(invitation_id): Path<UserInvitationId>,
) -> ApiResult<impl IntoResponse> {
    let invitation = app.revoke_user_invitation(invitation_id).await?;

    Ok(Json(AdminInvitationResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        invitation: AdminInvitationRecord::from(invitation),
    }))
}

pub(super) async fn replace_admin_access_user_roles(
    State(app): State<NakoApp>,
    Path(user_id): Path<UserId>,
    Json(request): Json<AdminReplaceUserRolesRequest>,
) -> ApiResult<impl IntoResponse> {
    validate_admin_access_roles(&request.roles)?;
    let user = admin_access_user_or_not_found(&app, user_id).await?;
    if is_bootstrap_admin_user(&user) && !request.roles.contains(&UserRole::Administrator) {
        return Err(NakoError::InvalidInput {
            message: "bootstrap administrator must retain the administrator role".to_owned(),
        }
        .into());
    }

    let assignments =
        role_assignments_for_user(user.id, &request.roles, crate::app::current_time_ms()?);
    app.replace_role_assignments(user.id, &assignments).await?;

    Ok(Json(AdminAccessUserResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        user: admin_access_user_record(&app, user).await?,
    }))
}

pub(super) async fn update_admin_access_user_status(
    State(app): State<NakoApp>,
    Path(user_id): Path<UserId>,
    Json(request): Json<AdminUpdateUserStatusRequest>,
) -> ApiResult<impl IntoResponse> {
    let mut user = admin_access_user_or_not_found(&app, user_id).await?;
    if is_bootstrap_admin_user(&user) && request.status != UserStatus::Active {
        return Err(NakoError::InvalidInput {
            message: "bootstrap administrator cannot be disabled".to_owned(),
        }
        .into());
    }

    user.status = request.status;
    user.updated_at_ms = crate::app::current_time_ms()?;
    app.upsert_user(&user).await?;

    Ok(Json(AdminAccessUserResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        user: admin_access_user_record(&app, user).await?,
    }))
}

pub(super) async fn set_admin_access_user_local_password(
    State(app): State<NakoApp>,
    Path(user_id): Path<UserId>,
    Json(request): Json<AdminSetLocalPasswordRequest>,
) -> ApiResult<impl IntoResponse> {
    app.set_local_password(user_id, &request.password).await?;

    Ok(Json(nako_api::admin::AdminLocalPasswordResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        user_id,
        local_password_configured: true,
    }))
}

pub(super) async fn delete_admin_access_user_local_password(
    State(app): State<NakoApp>,
    Path(user_id): Path<UserId>,
) -> ApiResult<impl IntoResponse> {
    let _ = admin_access_user_or_not_found(&app, user_id).await?;
    app.delete_local_password(user_id).await?;

    Ok(Json(nako_api::admin::AdminLocalPasswordResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        user_id,
        local_password_configured: false,
    }))
}

pub(super) async fn list_admin_library_access_policies(
    State(app): State<NakoApp>,
    Query(query): Query<AdminLibraryAccessPolicyListQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let policies = app.list_library_access_policies(filter, page).await?;
    let returned = policies.len();
    let policies = policies
        .into_iter()
        .map(AdminLibraryAccessPolicyRecord::from)
        .collect();

    Ok(Json(AdminLibraryAccessPolicyListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        policies,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn upsert_admin_library_access_policy(
    State(app): State<NakoApp>,
    Json(request): Json<AdminUpsertLibraryAccessPolicyRequest>,
) -> ApiResult<impl IntoResponse> {
    let now_ms = crate::app::current_time_ms()?;
    let policy = LibraryAccessPolicy {
        scope: request.scope.into(),
        library_id: request.library_id,
        access: request.access,
        created_at_ms: now_ms,
        updated_at_ms: now_ms,
    };
    app.upsert_library_access_policy(&policy).await?;

    Ok(Json(AdminLibraryAccessPolicyResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        policy: AdminLibraryAccessPolicyRecord::from(policy),
    }))
}

pub(super) async fn delete_admin_library_access_policy(
    State(app): State<NakoApp>,
    Query(query): Query<AdminLibraryAccessPolicyDeleteQuery>,
) -> ApiResult<impl IntoResponse> {
    let (scope, library_id) = query.into_scope_and_library()?;
    app.delete_library_access_policy(scope, library_id).await?;

    Ok(Json(AdminLibraryAccessPolicyDeleteResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        deleted: true,
    }))
}

pub(super) async fn get_admin_metadata_raw_cache_settings(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_admin_metadata_raw_cache_settings().await?))
}

pub(super) async fn update_admin_metadata_raw_cache_settings(
    State(app): State<NakoApp>,
    Json(request): Json<AdminUpdateMetadataRawCacheSettingsRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.update_admin_metadata_raw_cache_settings(request)
            .await?,
    ))
}

pub(super) async fn get_admin_playback_runtime_settings(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_admin_playback_runtime_settings().await?))
}

pub(super) async fn update_admin_playback_runtime_settings(
    State(app): State<NakoApp>,
    Json(request): Json<AdminUpdatePlaybackRuntimeSettingsRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.update_admin_playback_runtime_settings(request).await?,
    ))
}

pub(super) async fn list_admin_storage_backends(
    State(app): State<NakoApp>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = query.try_into()?;
    let backends = app
        .storage()
        .list_storage_backend_health(page)
        .await?
        .into_iter();
    let returned = backends.len();
    let backends = backends
        .map(AdminStorageBackendHealthDiagnostic::from_record)
        .collect();

    Ok(Json(AdminStorageBackendHealthDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        backends,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn reset_admin_storage_backend_circuit_breaker(
    State(app): State<NakoApp>,
    Path(backend_key): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let reset_at_ms = crate::app::current_time_ms()?;
    let record = app
        .storage()
        .reset_storage_backend_health(&backend_key, reset_at_ms)
        .await?
        .ok_or_else(|| NakoError::NotFound {
            entity: "storage_backend_health",
            id: backend_key,
        })?;
    let reset_at_ms = record.updated_at_ms;
    let backend = AdminStorageBackendHealthDiagnostic::from_record(record);

    Ok(Json(AdminStorageBackendHealthResetResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        backend,
        reset_at_ms,
    }))
}

pub(super) async fn list_admin_storage_staging(
    State(app): State<NakoApp>,
    Query(query): Query<StorageStagingQuery>,
) -> ApiResult<impl IntoResponse> {
    let (purpose, state, page) = query.into_filter_and_page()?;
    let records = app
        .storage()
        .list_staging_manifest_records(purpose, state, page)
        .await?;
    let returned = records.len();
    let records = records
        .into_iter()
        .map(AdminStorageStagingRecord::from_record)
        .collect();
    let summary = admin_storage_staging_summary(&app).await?;

    Ok(Json(AdminStorageStagingDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        summary,
        records,
        page: page_info_from_request(page, returned),
    }))
}

async fn admin_storage_staging_summary(app: &NakoApp) -> ApiResult<AdminStorageStagingSummary> {
    let startup = app.startup_report().clone();
    let process_cached_backends = usize_to_u32(app.storage().process_cached_backend_count().await);
    let used_manifest_bytes = app.storage().sum_staging_manifest_bytes().await?;
    let now_ms = crate::app::current_time_ms()?;
    let cleanup_pressure = app
        .storage()
        .summarize_staging_cleanup_pressure(now_ms)
        .await?;
    let manifest_pressure = app.storage().summarize_staging_manifest_pressure().await?;
    let policy_slices = app
        .storage()
        .summarize_staging_budget_policy()
        .await?
        .into_iter()
        .map(storage_staging_policy_slice)
        .collect();
    let purpose_state_summaries = manifest_pressure
        .purpose_state_summaries
        .iter()
        .copied()
        .map(storage_staging_purpose_state_summary)
        .collect();
    let cleanup_purpose_state_summaries = cleanup_pressure
        .cleanup_purpose_state_summaries
        .iter()
        .copied()
        .map(storage_staging_cleanup_purpose_state_summary)
        .collect();
    let vfs_cache = app.storage().summarize_vfs_cache(now_ms).await?;
    let vfs_cache_repair = app
        .storage()
        .latest_vfs_cache_repair_diagnostic()
        .await?
        .map(admin_vfs_cache_repair_diagnostic);

    Ok(AdminStorageStagingSummary {
        configured_max_bytes: app.config().staging.max_bytes,
        used_manifest_bytes,
        pressure: storage_staging_pressure_summary(
            app.config().staging.max_bytes,
            used_manifest_bytes,
            manifest_pressure.total_records,
            manifest_pressure.in_flight_records,
            manifest_pressure.failed_records,
            manifest_pressure.unknown_size_records,
            manifest_pressure.active_leases,
            manifest_pressure.ffmpeg_input_records,
            manifest_pressure.probe_input_records,
        ),
        policy_slices,
        purpose_state_summaries,
        cleanup_purpose_state_summaries,
        cleanup_on_startup: app.config().staging.cleanup_on_startup,
        retention_ms: app.config().staging.retention_ms,
        startup_deleted_records: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_records)),
        startup_deleted_files: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_files)),
        cleanup_candidate_records: usize_to_u32(cleanup_pressure.cleanup_candidate_records),
        cleanup_candidate_bytes: cleanup_pressure.cleanup_candidate_bytes,
        process_cached_backends,
        vfs_cache: AdminVfsCacheSummary {
            object_count: vfs_cache.object_count,
            listing_count: vfs_cache.listing_count,
            failure_count: vfs_cache.failure_count,
            stale_object_count: vfs_cache.stale_object_count,
            stale_listing_count: vfs_cache.stale_listing_count,
            last_failure_at_ms: vfs_cache.last_failure_at_ms,
            repair: vfs_cache_repair,
        },
    })
}

pub(super) async fn refresh_admin_vfs_cache(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    let report = app.storage().refresh_latest_vfs_cache_repair().await?;

    Ok(Json(admin_vfs_cache_refresh_response(report)))
}

pub(super) async fn get_admin_vfs_cache_repair_remediation_plan(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    let report = app.storage().plan_vfs_cache_repair_remediation().await?;

    Ok(Json(admin_vfs_cache_repair_remediation_plan(report)))
}

pub(super) async fn plan_admin_vfs_cache_repair_automation(
    State(app): State<NakoApp>,
    Json(request): Json<AdminVfsCacheRepairAutomationPolicyRequest>,
) -> ApiResult<impl IntoResponse> {
    let report = app
        .storage()
        .plan_vfs_cache_repair_automation(VfsCacheRepairAutomationPolicy {
            enabled: request.enabled,
        })
        .await?;

    Ok(Json(admin_vfs_cache_repair_automation_plan(report)))
}

pub(super) async fn enqueue_admin_vfs_cache_repair_automation(
    State(app): State<NakoApp>,
    Json(request): Json<AdminVfsCacheRepairAutomationEnqueueRequest>,
) -> ApiResult<impl IntoResponse> {
    let report = app
        .storage()
        .enqueue_vfs_cache_repair_automation(
            VfsCacheRepairAutomationPolicy {
                enabled: request.enabled,
            },
            request.priority.map(Into::into),
        )
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(admin_vfs_cache_repair_automation_enqueue(report)),
    ))
}

pub(super) async fn list_admin_vfs_cache_repair_targets(
    State(app): State<NakoApp>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.try_into()?;
    let targets = app.storage().list_vfs_cache_repair_targets(page).await?;
    let returned = targets.len();

    Ok(Json(AdminVfsCacheRepairTargetListResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        targets: targets
            .into_iter()
            .map(admin_vfs_cache_repair_target)
            .collect(),
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn get_admin_vfs_cache_repair_target_preview(
    State(app): State<NakoApp>,
    Path(target_ref): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let report = app
        .storage()
        .preview_vfs_cache_repair_target(&target_ref)
        .await?;

    Ok(Json(admin_vfs_cache_repair_target_preview(report)))
}

pub(super) async fn refresh_admin_vfs_cache_repair_target(
    State(app): State<NakoApp>,
    Path(target_ref): Path<String>,
) -> ApiResult<impl IntoResponse> {
    let report = app
        .storage()
        .refresh_vfs_cache_repair_target(&target_ref)
        .await?;

    Ok(Json(admin_vfs_cache_refresh_response(report)))
}

pub(super) async fn enqueue_admin_vfs_cache_repair_target(
    State(app): State<NakoApp>,
    Path(target_ref): Path<String>,
    Json(request): Json<AdminVfsCacheRepairEnqueueRequest>,
) -> ApiResult<impl IntoResponse> {
    let outcome = app
        .storage()
        .enqueue_vfs_cache_repair_target(&target_ref, request.priority.map(Into::into))
        .await?;

    Ok((
        StatusCode::ACCEPTED,
        Json(admin_vfs_cache_repair_enqueue_response(outcome)),
    ))
}

pub(super) async fn execute_admin_vfs_cache_repair_job(
    State(app): State<NakoApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<impl IntoResponse> {
    let output = app.storage().execute_vfs_cache_repair_job(job_id).await?;

    Ok(Json(admin_vfs_cache_repair_execute_response(output)))
}

pub(super) async fn retry_admin_vfs_cache_repair_job(
    State(app): State<NakoApp>,
    Path(job_id): Path<JobId>,
    Json(request): Json<AdminVfsCacheRepairRetryRequest>,
) -> ApiResult<impl IntoResponse> {
    let job = app
        .storage()
        .retry_vfs_cache_repair_job(RetryVfsCacheRepairJobRequest {
            job_id,
            max_attempts: request.max_attempts,
            next_attempt_at: request.next_attempt_at,
        })
        .await?;

    Ok((StatusCode::ACCEPTED, Json(AdminJobListItem::from_job(job))))
}

pub(super) async fn get_admin_vfs_cache_repair_action_plan(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    let report = app.storage().plan_latest_vfs_cache_repair_action().await?;

    Ok(Json(AdminVfsCacheRepairActionPlanResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        plan: admin_vfs_cache_repair_action_plan(report),
    }))
}

fn admin_vfs_cache_refresh_response(
    report: VfsCacheRepairRefreshActionReport,
) -> AdminVfsCacheRefreshResponse {
    AdminVfsCacheRefreshResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        action: admin_vfs_cache_repair_action(report.action),
        operation: report.operation,
        refreshed: report.refresh.operation == report.operation,
        repair: admin_vfs_cache_repair_diagnostic(report.repair),
    }
}

fn admin_vfs_cache_repair_enqueue_response(
    outcome: EnqueueVfsCacheRepairTargetOutcome,
) -> AdminVfsCacheRepairEnqueueResponse {
    let (outcome, job) = match outcome {
        EnqueueVfsCacheRepairTargetOutcome::Enqueued(job) => {
            (AdminVfsCacheRepairEnqueueOutcome::Enqueued, job)
        }
        EnqueueVfsCacheRepairTargetOutcome::AlreadyQueued(job) => {
            (AdminVfsCacheRepairEnqueueOutcome::AlreadyQueued, job)
        }
    };

    AdminVfsCacheRepairEnqueueResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        outcome,
        job: AdminJobListItem::from_job(job),
    }
}

fn admin_vfs_cache_repair_execute_response(
    output: VfsCacheRepairCommandOutput,
) -> AdminVfsCacheRepairExecuteResponse {
    AdminVfsCacheRepairExecuteResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        job: AdminJobListItem::from_job(output.job),
        summary: admin_vfs_cache_repair_job_summary(output.summary),
    }
}

fn admin_vfs_cache_repair_automation_plan(
    report: VfsCacheRepairAutomationPolicyReport,
) -> AdminVfsCacheRepairAutomationPlanResponse {
    AdminVfsCacheRepairAutomationPlanResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        policy: admin_vfs_cache_repair_automation_policy(report),
    }
}

fn admin_vfs_cache_repair_automation_enqueue(
    report: VfsCacheRepairAutomationEnqueueReport,
) -> AdminVfsCacheRepairAutomationEnqueueResponse {
    AdminVfsCacheRepairAutomationEnqueueResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        policy: admin_vfs_cache_repair_automation_policy(report.policy_report),
        jobs: report
            .jobs
            .into_iter()
            .map(admin_vfs_cache_repair_automation_job)
            .collect(),
        enqueued_count: report.enqueued_count,
        already_queued_count: report.already_queued_count,
    }
}

fn admin_vfs_cache_repair_automation_policy(
    report: VfsCacheRepairAutomationPolicyReport,
) -> AdminVfsCacheRepairAutomationPolicyReport {
    AdminVfsCacheRepairAutomationPolicyReport {
        enabled: report.policy.enabled,
        total_unresolved_targets: report.total_unresolved_targets,
        eligible_targets: report
            .eligible_targets
            .into_iter()
            .map(|target| AdminVfsCacheRepairAutomationEligibleTarget {
                target: admin_vfs_cache_repair_target(target.target),
            })
            .collect(),
        blocked_targets: report
            .blocked_targets
            .into_iter()
            .map(|target| AdminVfsCacheRepairAutomationBlockedTarget {
                target: admin_vfs_cache_repair_target(target.target),
                reason: admin_vfs_cache_repair_automation_block_reason(target.reason),
            })
            .collect(),
        boundary: AdminVfsCacheRepairAutomationBoundary {
            reads_repair_targets: report.boundary.reads_repair_targets,
            may_start_durable_jobs: report.boundary.may_start_durable_jobs,
            refreshes_vfs_cache: report.boundary.refreshes_vfs_cache,
            changes_backend_configuration: report.boundary.changes_backend_configuration,
            deletes_cache_entries: report.boundary.deletes_cache_entries,
            writes_library_files: report.boundary.writes_library_files,
        },
    }
}

fn admin_vfs_cache_repair_automation_block_reason(
    reason: AppVfsCacheRepairAutomationBlockReason,
) -> AdminVfsCacheRepairAutomationBlockReason {
    match reason {
        AppVfsCacheRepairAutomationBlockReason::PolicyDisabled => {
            AdminVfsCacheRepairAutomationBlockReason::PolicyDisabled
        }
        AppVfsCacheRepairAutomationBlockReason::BackendConfigurationRequired => {
            AdminVfsCacheRepairAutomationBlockReason::BackendConfigurationRequired
        }
        AppVfsCacheRepairAutomationBlockReason::ManualFailureInspectionRequired => {
            AdminVfsCacheRepairAutomationBlockReason::ManualFailureInspectionRequired
        }
        AppVfsCacheRepairAutomationBlockReason::NoActionRequired => {
            AdminVfsCacheRepairAutomationBlockReason::NoActionRequired
        }
    }
}

fn admin_vfs_cache_repair_automation_job(
    report: VfsCacheRepairAutomationJobReport,
) -> AdminVfsCacheRepairAutomationJob {
    AdminVfsCacheRepairAutomationJob {
        outcome: admin_vfs_cache_repair_automation_enqueue_outcome(report.outcome),
        job_id: report.job_id,
        status: report.status,
        priority: report.priority,
        resource_class: report.resource_class,
        library_id: report.library_id,
        source_id: report.source_id,
    }
}

fn admin_vfs_cache_repair_automation_enqueue_outcome(
    outcome: AppVfsCacheRepairAutomationEnqueueOutcome,
) -> AdminVfsCacheRepairEnqueueOutcome {
    match outcome {
        AppVfsCacheRepairAutomationEnqueueOutcome::Enqueued => {
            AdminVfsCacheRepairEnqueueOutcome::Enqueued
        }
        AppVfsCacheRepairAutomationEnqueueOutcome::AlreadyQueued => {
            AdminVfsCacheRepairEnqueueOutcome::AlreadyQueued
        }
    }
}

fn admin_vfs_cache_repair_job_summary(
    summary: AppVfsCacheRepairJobSummary,
) -> AdminVfsCacheRepairJobSummary {
    AdminVfsCacheRepairJobSummary {
        action: admin_vfs_cache_repair_action(summary.action),
        source_scheme: summary.source_scheme,
        operation: summary.operation,
        classification: admin_vfs_cache_repair_classification(summary.classification),
        failure_class: summary.failure_class,
        failed_at_ms: summary.failed_at_ms,
        failure_count: summary.failure_count,
        refreshed_cache_state: summary
            .refreshed_cache_state
            .map(admin_vfs_cache_repair_cache_state),
    }
}

fn admin_vfs_cache_repair_cache_state(state: ObjectCacheState) -> AdminVfsCacheRepairCacheState {
    match state {
        ObjectCacheState::Fresh => AdminVfsCacheRepairCacheState::Fresh,
        ObjectCacheState::StaleFallback => AdminVfsCacheRepairCacheState::StaleFallback,
    }
}

fn admin_vfs_cache_repair_target_preview(
    report: VfsCacheRepairTargetPreviewReport,
) -> AdminVfsCacheRepairTargetPreviewResponse {
    AdminVfsCacheRepairTargetPreviewResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        target: admin_vfs_cache_repair_target(report.target),
        plan: admin_vfs_cache_repair_action_plan(report.plan),
    }
}

fn admin_vfs_cache_repair_target(target: VfsCacheRepairTargetReport) -> AdminVfsCacheRepairTarget {
    AdminVfsCacheRepairTarget {
        target_ref: target.target_ref,
        scheme: target.scheme,
        operation: target.operation,
        failed_at_ms: target.failed_at_ms,
        failure_count: target.failure_count,
        classification: admin_vfs_cache_repair_classification(target.repair.classification),
        recommended_action: admin_vfs_cache_repair_action(target.repair.recommended_action),
        failure_class: target.repair.failure_class,
        retryable: target.repair.retryable,
        safe_message: target.repair.safe_message,
    }
}

fn admin_vfs_cache_repair_remediation_plan(
    report: VfsCacheRepairRemediationPlanReport,
) -> AdminVfsCacheRepairRemediationPlanResponse {
    AdminVfsCacheRepairRemediationPlanResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        total_unresolved_targets: report.total_unresolved_targets,
        action_groups: report
            .action_groups
            .into_iter()
            .map(admin_vfs_cache_repair_remediation_action_group)
            .collect(),
        classification_counts: report
            .classification_counts
            .into_iter()
            .map(admin_vfs_cache_repair_classification_count)
            .collect(),
        boundary: admin_vfs_cache_repair_remediation_plan_boundary(report.boundary),
    }
}

fn admin_vfs_cache_repair_remediation_action_group(
    group: VfsCacheRepairRemediationActionGroupReport,
) -> AdminVfsCacheRepairRemediationActionGroup {
    let executable_action = (group.status == VfsCacheRepairActionPlanStatus::Executable
        && group.action == VfsCacheRepairAction::RefreshCache)
        .then(|| group.executable_route)
        .flatten()
        .map(admin_vfs_cache_refresh_executable_action);

    AdminVfsCacheRepairRemediationActionGroup {
        action: admin_vfs_cache_repair_action(group.action),
        count: group.count,
        status: admin_vfs_cache_repair_action_plan_status(group.status),
        readiness: AdminVfsCacheRepairActionReadiness {
            status: admin_vfs_cache_repair_action_plan_status(group.status),
            api_executable: group.api_executable,
            reasons: group
                .reasons
                .into_iter()
                .map(admin_vfs_cache_repair_action_plan_reason)
                .collect(),
        },
        boundary: admin_vfs_cache_repair_action_boundary(group.boundary),
        executable_action,
        sample_targets: group
            .sample_targets
            .into_iter()
            .map(admin_vfs_cache_repair_target)
            .collect(),
    }
}

fn admin_vfs_cache_repair_classification_count(
    count: VfsCacheRepairRemediationClassificationCountReport,
) -> AdminVfsCacheRepairClassificationCount {
    AdminVfsCacheRepairClassificationCount {
        classification: admin_vfs_cache_repair_classification(count.classification),
        count: count.count,
    }
}

fn admin_vfs_cache_repair_remediation_plan_boundary(
    boundary: AppVfsCacheRepairRemediationPlanBoundary,
) -> AdminVfsCacheRepairRemediationPlanBoundary {
    AdminVfsCacheRepairRemediationPlanBoundary {
        read_only: boundary.read_only,
        refreshes_vfs_cache: boundary.refreshes_vfs_cache,
        changes_backend_configuration: boundary.changes_backend_configuration,
        deletes_cache_entries: boundary.deletes_cache_entries,
        writes_library_files: boundary.writes_library_files,
        starts_durable_job: boundary.starts_durable_job,
    }
}

fn admin_vfs_cache_repair_action_plan(
    report: VfsCacheRepairActionPlanReport,
) -> AdminVfsCacheRepairActionPlan {
    let executable_action = (report.status == VfsCacheRepairActionPlanStatus::Executable
        && report.action == VfsCacheRepairAction::RefreshCache)
        .then(|| report.executable_route)
        .flatten()
        .map(admin_vfs_cache_refresh_executable_action);

    AdminVfsCacheRepairActionPlan {
        status: admin_vfs_cache_repair_action_plan_status(report.status),
        action: admin_vfs_cache_repair_action(report.action),
        readiness: AdminVfsCacheRepairActionReadiness {
            status: admin_vfs_cache_repair_action_plan_status(report.status),
            api_executable: report.api_executable,
            reasons: report
                .reasons
                .into_iter()
                .map(admin_vfs_cache_repair_action_plan_reason)
                .collect(),
        },
        boundary: admin_vfs_cache_repair_action_boundary(report.boundary),
        executable_action,
        repair: report.repair.map(admin_vfs_cache_repair_diagnostic),
    }
}

fn admin_vfs_cache_refresh_executable_action(
    route: VfsCacheRepairExecutableRoute,
) -> AdminVfsCacheRepairExecutableAction {
    let (route_key, route_path) = match route {
        VfsCacheRepairExecutableRoute::LatestRefreshCache => (
            STORAGE_VFS_CACHE_REPAIR_REFRESH_CACHE_ROUTE_KEY,
            STORAGE_VFS_CACHE_REPAIR_REFRESH_CACHE_ROUTE_PATH,
        ),
        VfsCacheRepairExecutableRoute::TargetRefreshCache => (
            STORAGE_VFS_CACHE_REPAIR_TARGET_REFRESH_CACHE_ROUTE_KEY,
            STORAGE_VFS_CACHE_REPAIR_TARGET_REFRESH_CACHE_ROUTE_PATH,
        ),
    };

    AdminVfsCacheRepairExecutableAction {
        method: "POST".to_owned(),
        route_key: route_key.to_owned(),
        route_path: route_path.to_owned(),
    }
}

fn admin_vfs_cache_repair_action_plan_status(
    status: VfsCacheRepairActionPlanStatus,
) -> AdminVfsCacheRepairActionPlanStatus {
    match status {
        VfsCacheRepairActionPlanStatus::NoAction => AdminVfsCacheRepairActionPlanStatus::NoAction,
        VfsCacheRepairActionPlanStatus::Executable => {
            AdminVfsCacheRepairActionPlanStatus::Executable
        }
        VfsCacheRepairActionPlanStatus::PlanOnly => AdminVfsCacheRepairActionPlanStatus::PlanOnly,
    }
}

fn admin_vfs_cache_repair_action_plan_reason(
    reason: VfsCacheRepairActionPlanReason,
) -> AdminVfsCacheRepairActionPlanReason {
    match reason {
        VfsCacheRepairActionPlanReason::NoRepairDiagnostic => {
            AdminVfsCacheRepairActionPlanReason::NoRepairDiagnostic
        }
        VfsCacheRepairActionPlanReason::NoActionRequired => {
            AdminVfsCacheRepairActionPlanReason::NoActionRequired
        }
        VfsCacheRepairActionPlanReason::RefreshCacheExecutable => {
            AdminVfsCacheRepairActionPlanReason::RefreshCacheExecutable
        }
        VfsCacheRepairActionPlanReason::BackendConfigurationRequired => {
            AdminVfsCacheRepairActionPlanReason::BackendConfigurationRequired
        }
        VfsCacheRepairActionPlanReason::ManualFailureInspectionRequired => {
            AdminVfsCacheRepairActionPlanReason::ManualFailureInspectionRequired
        }
    }
}

fn admin_vfs_cache_repair_action_boundary(
    boundary: VfsCacheRepairActionBoundary,
) -> AdminVfsCacheRepairActionBoundary {
    AdminVfsCacheRepairActionBoundary {
        refreshes_vfs_cache: boundary.refreshes_vfs_cache,
        changes_backend_configuration: boundary.changes_backend_configuration,
        requires_manual_failure_inspection: boundary.requires_manual_failure_inspection,
        deletes_cache_entries: boundary.deletes_cache_entries,
        writes_library_files: boundary.writes_library_files,
        starts_durable_job: boundary.starts_durable_job,
    }
}

fn admin_vfs_cache_repair_diagnostic(
    diagnostic: nako_vfs::VfsCacheRepairDiagnostic,
) -> AdminVfsCacheRepairDiagnostic {
    AdminVfsCacheRepairDiagnostic {
        classification: admin_vfs_cache_repair_classification(diagnostic.classification),
        recommended_action: admin_vfs_cache_repair_action(diagnostic.recommended_action),
        operation: diagnostic.operation,
        failure_class: diagnostic.failure_class,
        retryable: diagnostic.retryable,
        failed_at_ms: diagnostic.failed_at_ms,
        failure_count: diagnostic.failure_count,
        safe_message: diagnostic.safe_message,
        operator_action: diagnostic.operator_action,
    }
}

fn admin_vfs_cache_repair_action(action: VfsCacheRepairAction) -> AdminVfsCacheRepairAction {
    match action {
        VfsCacheRepairAction::None => AdminVfsCacheRepairAction::None,
        VfsCacheRepairAction::RefreshCache => AdminVfsCacheRepairAction::RefreshCache,
        VfsCacheRepairAction::FixBackendConfiguration => {
            AdminVfsCacheRepairAction::FixBackendConfiguration
        }
        VfsCacheRepairAction::InspectFailure => AdminVfsCacheRepairAction::InspectFailure,
    }
}

fn admin_vfs_cache_repair_classification(
    classification: VfsCacheRepairClassification,
) -> AdminVfsCacheRepairClassification {
    match classification {
        VfsCacheRepairClassification::Healthy => AdminVfsCacheRepairClassification::Healthy,
        VfsCacheRepairClassification::RepairableStaleFallback => {
            AdminVfsCacheRepairClassification::RepairableStaleFallback
        }
        VfsCacheRepairClassification::RetryableRefreshFailure => {
            AdminVfsCacheRepairClassification::RetryableRefreshFailure
        }
        VfsCacheRepairClassification::OperatorActionRequired => {
            AdminVfsCacheRepairClassification::OperatorActionRequired
        }
        VfsCacheRepairClassification::UnknownFailure => {
            AdminVfsCacheRepairClassification::UnknownFailure
        }
    }
}

fn storage_staging_policy_slice(slice: StagingBudgetPolicySlice) -> AdminStorageStagingPolicySlice {
    AdminStorageStagingPolicySlice {
        backend_key: slice.backend_key,
        library_id: slice.library_id,
        library_name: slice.library_name,
        backend_kind: slice.backend_kind,
        source_scheme: slice.source_scheme,
        configured_max_bytes: slice.configured_max_bytes,
        used_manifest_bytes: slice.used_manifest_bytes,
        pressure: storage_staging_pressure_summary(
            slice.configured_max_bytes,
            slice.used_manifest_bytes,
            slice.manifest_pressure.total_records,
            slice.manifest_pressure.in_flight_records,
            slice.manifest_pressure.failed_records,
            slice.manifest_pressure.unknown_size_records,
            slice.manifest_pressure.active_leases,
            slice.manifest_pressure.ffmpeg_input_records,
            slice.manifest_pressure.probe_input_records,
        ),
    }
}

fn storage_staging_purpose_state_summary(
    summary: StagingPurposeStateSummary,
) -> AdminStorageStagingPurposeStateSummary {
    AdminStorageStagingPurposeStateSummary {
        purpose: summary.purpose,
        state: summary.state,
        record_count: summary.record_count,
        used_manifest_bytes: summary.used_manifest_bytes,
        active_leases: summary.active_leases,
        unknown_size_records: summary.unknown_size_records,
    }
}

fn storage_staging_cleanup_purpose_state_summary(
    summary: StagingCleanupPurposeStateSummary,
) -> AdminStorageStagingCleanupPurposeStateSummary {
    AdminStorageStagingCleanupPurposeStateSummary {
        purpose: summary.purpose,
        state: summary.state,
        record_count: summary.record_count,
        cleanup_candidate_bytes: summary.cleanup_candidate_bytes,
        active_leases: summary.active_leases,
        unknown_size_records: summary.unknown_size_records,
    }
}

fn storage_staging_pressure_summary(
    configured_max_bytes: u64,
    used_manifest_bytes: u64,
    total_records: usize,
    in_flight_records: usize,
    failed_records: usize,
    unknown_size_records: usize,
    active_leases: u64,
    ffmpeg_input_records: usize,
    probe_input_records: usize,
) -> AdminStorageStagingPressureSummary {
    AdminStorageStagingPressureSummary {
        status: storage_staging_pressure_status(configured_max_bytes, used_manifest_bytes),
        used_ratio_milli: storage_staging_used_ratio_milli(
            configured_max_bytes,
            used_manifest_bytes,
        ),
        total_records: usize_to_u32(total_records),
        in_flight_records: usize_to_u32(in_flight_records),
        failed_records: usize_to_u32(failed_records),
        unknown_size_records: usize_to_u32(unknown_size_records),
        active_leases: u64_to_u32(active_leases),
        ffmpeg_input_records: usize_to_u32(ffmpeg_input_records),
        probe_input_records: usize_to_u32(probe_input_records),
    }
}

fn storage_staging_pressure_status(
    configured_max_bytes: u64,
    used_manifest_bytes: u64,
) -> AdminStorageStagingPressureStatus {
    match app_storage_staging_pressure_status(configured_max_bytes, used_manifest_bytes) {
        StorageStagingPressureStatus::Disabled => AdminStorageStagingPressureStatus::Disabled,
        StorageStagingPressureStatus::Healthy => AdminStorageStagingPressureStatus::Healthy,
        StorageStagingPressureStatus::Elevated => AdminStorageStagingPressureStatus::Elevated,
        StorageStagingPressureStatus::Critical => AdminStorageStagingPressureStatus::Critical,
        StorageStagingPressureStatus::Exhausted => AdminStorageStagingPressureStatus::Exhausted,
    }
}

fn storage_staging_used_ratio_milli(
    configured_max_bytes: u64,
    used_manifest_bytes: u64,
) -> Option<u32> {
    if configured_max_bytes == 0 {
        return None;
    }

    let ratio = u128::from(used_manifest_bytes)
        .saturating_mul(1_000)
        .saturating_div(u128::from(configured_max_bytes));

    Some(ratio.min(u128::from(u32::MAX)) as u32)
}

fn network_access_diagnostics(
    config: &crate::config::NakoServerConfig,
) -> AdminNetworkAccessDiagnostics {
    let network = &config.network;

    AdminNetworkAccessDiagnostics {
        exposure_mode: admin_network_exposure_mode(network.exposure_mode),
        readiness: network_readiness_diagnostics(config),
        external_endpoint: endpoint_diagnostics(network.external_base_url.as_deref()),
        trusted_proxy: AdminTrustedProxyDiagnostics {
            headers_enabled: network.trusted_proxy_headers,
            source_count: usize_to_u32(network.trusted_proxy_sources.len()),
        },
        origins: AdminOriginPolicyDiagnostics {
            allowed_origin_count: usize_to_u32(network.allowed_origins.len()),
            configured: !network.allowed_origins.is_empty(),
        },
        tunnel_providers: network
            .tunnel_providers
            .iter()
            .map(tunnel_provider_diagnostics)
            .collect(),
    }
}

fn network_readiness_diagnostics(
    config: &crate::config::NakoServerConfig,
) -> AdminNetworkReadinessDiagnostics {
    let network = &config.network;
    AdminNetworkReadinessDiagnostics::from_checks(vec![
        exposure_mode_readiness_check(network),
        auth_readiness_check(config),
        external_endpoint_readiness_check(network),
        trusted_proxy_readiness_check(network),
        origin_policy_readiness_check(network),
        tunnel_provider_readiness_check(network),
    ])
}

fn exposure_mode_readiness_check(network: &NetworkAccessConfig) -> AdminNetworkReadinessCheck {
    match network.exposure_mode {
        ConfigNetworkExposureMode::LocalOnly => AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::ExposureMode,
            AdminNetworkReadinessReason::LocalOnly,
        ),
        ConfigNetworkExposureMode::PrivateNetwork
        | ConfigNetworkExposureMode::ReverseProxy
        | ConfigNetworkExposureMode::TunnelProvider => AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::ExposureMode,
            AdminNetworkReadinessReason::Ready,
        ),
    }
}

fn auth_readiness_check(config: &crate::config::NakoServerConfig) -> AdminNetworkReadinessCheck {
    if matches!(
        config.network.exposure_mode,
        ConfigNetworkExposureMode::LocalOnly
    ) {
        return AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::Auth,
            AdminNetworkReadinessReason::LocalOnly,
        );
    }

    if config.auth.enabled {
        AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::Auth,
            AdminNetworkReadinessReason::Ready,
        )
    } else {
        AdminNetworkReadinessCheck::unavailable(
            AdminNetworkReadinessCheckName::Auth,
            AdminNetworkReadinessReason::AuthDisabled,
        )
    }
}

fn external_endpoint_readiness_check(network: &NetworkAccessConfig) -> AdminNetworkReadinessCheck {
    match network.exposure_mode {
        ConfigNetworkExposureMode::ReverseProxy | ConfigNetworkExposureMode::TunnelProvider => {
            if is_https_endpoint(network.external_base_url.as_deref()) {
                AdminNetworkReadinessCheck::ready(
                    AdminNetworkReadinessCheckName::ExternalEndpoint,
                    AdminNetworkReadinessReason::Ready,
                )
            } else {
                AdminNetworkReadinessCheck::unavailable(
                    AdminNetworkReadinessCheckName::ExternalEndpoint,
                    AdminNetworkReadinessReason::MissingExternalBaseUrl,
                )
            }
        }
        ConfigNetworkExposureMode::LocalOnly | ConfigNetworkExposureMode::PrivateNetwork => {
            AdminNetworkReadinessCheck::ready(
                AdminNetworkReadinessCheckName::ExternalEndpoint,
                AdminNetworkReadinessReason::Ready,
            )
        }
    }
}

fn trusted_proxy_readiness_check(network: &NetworkAccessConfig) -> AdminNetworkReadinessCheck {
    if network.trusted_proxy_headers && network.trusted_proxy_sources.is_empty() {
        AdminNetworkReadinessCheck::unavailable(
            AdminNetworkReadinessCheckName::TrustedProxy,
            AdminNetworkReadinessReason::MissingTrustedProxySources,
        )
    } else {
        AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::TrustedProxy,
            AdminNetworkReadinessReason::Ready,
        )
    }
}

fn origin_policy_readiness_check(network: &NetworkAccessConfig) -> AdminNetworkReadinessCheck {
    if matches!(network.exposure_mode, ConfigNetworkExposureMode::LocalOnly)
        || !network.allowed_origins.is_empty()
    {
        AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::OriginPolicy,
            AdminNetworkReadinessReason::Ready,
        )
    } else {
        AdminNetworkReadinessCheck::degraded(
            AdminNetworkReadinessCheckName::OriginPolicy,
            AdminNetworkReadinessReason::BrowserOriginsNotConfigured,
        )
    }
}

fn tunnel_provider_readiness_check(network: &NetworkAccessConfig) -> AdminNetworkReadinessCheck {
    if !matches!(
        network.exposure_mode,
        ConfigNetworkExposureMode::TunnelProvider
    ) {
        return AdminNetworkReadinessCheck::ready(
            AdminNetworkReadinessCheckName::TunnelProvider,
            AdminNetworkReadinessReason::Ready,
        );
    }

    if network.tunnel_providers.is_empty() {
        return AdminNetworkReadinessCheck::unavailable(
            AdminNetworkReadinessCheckName::TunnelProvider,
            AdminNetworkReadinessReason::MissingTunnelProvider,
        );
    }

    if network
        .tunnel_providers
        .iter()
        .any(|provider| !tunnel_provider_token_present(provider))
    {
        return AdminNetworkReadinessCheck::unavailable(
            AdminNetworkReadinessCheckName::TunnelProvider,
            AdminNetworkReadinessReason::MissingTunnelToken,
        );
    }

    AdminNetworkReadinessCheck::ready(
        AdminNetworkReadinessCheckName::TunnelProvider,
        AdminNetworkReadinessReason::Ready,
    )
}

fn endpoint_diagnostics(value: Option<&str>) -> AdminNetworkExternalEndpointDiagnostics {
    AdminNetworkExternalEndpointDiagnostics {
        configured: value.is_some_and(|value| !value.trim().is_empty()),
        scheme: endpoint_scheme(value),
        host_fingerprint: endpoint_host(value).map(|host| fingerprint_key(&host)),
    }
}

fn tunnel_provider_diagnostics(provider: &TunnelProviderConfig) -> AdminTunnelProviderDiagnostics {
    let endpoint = endpoint_diagnostics(provider.public_url.as_deref());

    AdminTunnelProviderDiagnostics {
        id: provider.id.clone(),
        kind: admin_tunnel_provider_kind(provider.kind),
        endpoint_configured: endpoint.configured,
        endpoint_scheme: endpoint.scheme,
        endpoint_host_fingerprint: endpoint.host_fingerprint,
        token_env: provider.token_env.clone(),
        token_present: tunnel_provider_token_present(provider),
    }
}

fn tunnel_provider_token_present(provider: &TunnelProviderConfig) -> bool {
    provider
        .token_env
        .as_deref()
        .and_then(|env_name| std::env::var(env_name).ok())
        .is_some_and(|value| !value.trim().is_empty())
}

fn admin_network_exposure_mode(mode: ConfigNetworkExposureMode) -> AdminNetworkExposureMode {
    match mode {
        ConfigNetworkExposureMode::LocalOnly => AdminNetworkExposureMode::LocalOnly,
        ConfigNetworkExposureMode::PrivateNetwork => AdminNetworkExposureMode::PrivateNetwork,
        ConfigNetworkExposureMode::ReverseProxy => AdminNetworkExposureMode::ReverseProxy,
        ConfigNetworkExposureMode::TunnelProvider => AdminNetworkExposureMode::TunnelProvider,
    }
}

fn admin_tunnel_provider_kind(kind: ConfigTunnelProviderKind) -> AdminTunnelProviderKind {
    match kind {
        ConfigTunnelProviderKind::External => AdminTunnelProviderKind::External,
        ConfigTunnelProviderKind::CloudflareTunnel => AdminTunnelProviderKind::CloudflareTunnel,
        ConfigTunnelProviderKind::TailscaleFunnel => AdminTunnelProviderKind::TailscaleFunnel,
        ConfigTunnelProviderKind::Ngrok => AdminTunnelProviderKind::Ngrok,
    }
}

fn is_https_endpoint(value: Option<&str>) -> bool {
    endpoint_scheme(value).as_deref() == Some("https")
}

fn endpoint_scheme(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let (scheme, rest) = value.split_once("://")?;
    if rest.trim().is_empty() {
        return None;
    }
    let scheme = scheme.to_ascii_lowercase();
    if scheme == "http" || scheme == "https" {
        Some(scheme)
    } else {
        None
    }
}

fn endpoint_host(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    let (_, rest) = value.split_once("://")?;
    let authority = rest
        .split(['/', '?', '#'])
        .next()
        .map(str::trim)
        .filter(|authority| !authority.is_empty())?;
    let host_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let host = host_port
        .strip_prefix('[')
        .and_then(|rest| rest.split_once(']').map(|(host, _)| host))
        .or_else(|| host_port.split(':').next())
        .map(str::trim)
        .filter(|host| !host.is_empty())?;

    Some(host.to_ascii_lowercase())
}

fn fingerprint_key(value: &str) -> String {
    use sha2::{Digest, Sha256};

    let digest = Sha256::digest(value.as_bytes());
    let prefix = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("sha256:{prefix}")
}

fn database_config_diagnostics(
    config: &crate::config::NakoServerConfig,
    database: crate::app::DatabaseDiagnostics,
    migrated_on_startup: bool,
) -> AdminDatabaseConfigDiagnostics {
    let configured_backend = config.database_backend;

    AdminDatabaseConfigDiagnostics {
        configured_backend_kind: configured_backend.as_str().to_owned(),
        active_backend_kind: database.backend_kind.as_str().to_owned(),
        url_scheme: database_url_scheme_from_config(config),
        runtime_supported: configured_backend == database.backend_kind,
        migrated_on_startup,
        capabilities: database_backend_capabilities_diagnostics(database.capabilities),
    }
}

fn database_backend_capabilities_diagnostics(
    capabilities: DatabaseBackendCapabilities,
) -> AdminDatabaseBackendCapabilitiesDiagnostics {
    AdminDatabaseBackendCapabilitiesDiagnostics {
        lifecycle: capabilities.lifecycle,
        libraries: capabilities.libraries,
        jobs: capabilities.jobs,
        job_leases: capabilities.job_leases,
        media: capabilities.media,
        scan_commits: capabilities.scan_commits,
        metadata: capabilities.metadata,
        catalog: capabilities.catalog,
        playback_sessions: capabilities.playback_sessions,
        playback_state: capabilities.playback_state,
        transcode_sessions: capabilities.transcode_sessions,
        event_outbox: capabilities.event_outbox,
        addons: capabilities.addons,
        automation: capabilities.automation,
        managed_artwork: capabilities.managed_artwork,
        vfs_cache: capabilities.vfs_cache,
        webhooks: capabilities.webhooks,
        search_index: capabilities.search_index,
    }
}

fn database_url_scheme_from_config(config: &crate::config::NakoServerConfig) -> String {
    if let Some(env_name) = config.database_url_env.as_deref() {
        return std::env::var(env_name)
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| database_url_scheme(&value))
            .unwrap_or_else(|| "env".to_owned());
    }

    database_url_scheme(&config.database_url)
}

fn database_url_scheme(database_url: &str) -> String {
    let scheme = database_url
        .split_once("://")
        .or_else(|| database_url.split_once("::"))
        .map(|(scheme, _)| scheme)
        .filter(|scheme| !scheme.trim().is_empty())
        .unwrap_or("unknown");

    scheme.to_ascii_lowercase()
}

fn library_config_diagnostics(config: &LocalLibraryConfig) -> AdminLibraryConfigDiagnostics {
    let (
        backend_kind,
        root_scheme,
        has_webdav_password_env,
        webdav_timeout_ms,
        webdav_max_attempts,
    ) = match config.webdav.as_ref() {
        Some(webdav) => (
            StorageBackendKind::WebDav,
            "webdav".to_owned(),
            webdav.password_env.is_some(),
            Some(webdav.timeout_ms),
            Some(webdav.max_attempts),
        ),
        None => (
            StorageBackendKind::Local,
            "local".to_owned(),
            false,
            None,
            None,
        ),
    };

    AdminLibraryConfigDiagnostics {
        id: config.id,
        name: config.name.clone(),
        preset: config.preset,
        backend_kind,
        root_scheme,
        has_webdav_password_env,
        webdav_timeout_ms,
        webdav_max_attempts,
    }
}

fn metadata_runtime_config_diagnostics(
    config: &MetadataProviderRuntimeConfig,
) -> AdminMetadataRuntimeConfigDiagnostics {
    AdminMetadataRuntimeConfigDiagnostics {
        timeout_ms: config.timeout_ms,
        max_attempts: config.max_attempts,
        min_interval_ms: config.min_interval_ms,
        concurrency: config.concurrency,
        user_agent: config.user_agent.clone(),
        has_proxy: config.proxy.is_some(),
        circuit_breaker_failures: config.circuit_breaker_failures,
        circuit_breaker_backoff_ms: config.circuit_breaker_backoff_ms,
    }
}

fn metadata_provider_config_diagnostics(
    config: &MetadataProviderConfig,
) -> AdminMetadataProviderConfigDiagnostics {
    AdminMetadataProviderConfigDiagnostics {
        provider: config.provider.clone(),
        enabled: config.enabled,
        token_env: config.token_env.clone(),
        api_key_env: config.api_key_env.clone(),
        has_api_base_url: config.api_base_url.is_some(),
        has_image_base_url: config.image_base_url.is_some(),
        language: config.language.clone(),
        include_adult: config.include_adult,
        header_count: usize_to_u32(config.headers.len()),
        secret_header_count: usize_to_u32(
            config
                .headers
                .iter()
                .filter(|header| header.value.is_some() || header.value_env.is_some())
                .count(),
        ),
        has_provider_runtime_override: config.runtime.is_some(),
    }
}

pub(super) async fn list_admin_outbox_events(
    State(app): State<NakoApp>,
    Query(query): Query<OutboxEventListQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let events = app.webhooks().list_outbox_events(filter, page).await?;
    let returned = events.len();
    let events = events
        .into_iter()
        .map(AdminOutboxEventListItem::from_record)
        .collect();

    Ok(Json(AdminOutboxEventListResponse {
        events,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn list_admin_jobs(
    State(app): State<NakoApp>,
    Query(query): Query<JobListQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let jobs = app.jobs().list_jobs(filter, page).await?;
    let queue_pressure = app.job_queue_pressure_diagnostics().await?;
    let returned = jobs.len();
    let jobs = jobs.into_iter().map(AdminJobListItem::from_job).collect();
    let queue_pressure = queue_pressure.into_iter().map(Into::into).collect();

    Ok(Json(AdminJobListResponse {
        jobs,
        queue_pressure,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn cancel_admin_job(
    State(app): State<NakoApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<impl IntoResponse> {
    let cancellation = app.jobs().request_job_cancellation(job_id).await?;

    Ok(Json(AdminJobCancelRequestResponse::from_record(
        cancellation,
    )))
}

pub(super) async fn enqueue_admin_source_fingerprint_hash(
    State(app): State<NakoApp>,
    Extension(http_trace_context): Extension<HttpTraceContext>,
    Json(request): Json<AdminSourceFingerprintHashEnqueueRequest>,
) -> ApiResult<impl IntoResponse> {
    let mode = admin_source_fingerprint_hash_mode(request.mode, request.partial_prefix_bytes)?;
    let trace_context =
        crate::app::DurableJobTraceContext::from_request_id(http_trace_context.request_id())?;
    let job = app
        .source_hash()
        .enqueue_source_fingerprint_hash_with_trace_context(
            EnqueueSourceFingerprintHashRequest {
                library_id: request.library_id,
                source_id: request.source_id,
                mode,
                priority: request.priority.map(Into::into),
            },
            Some(&trace_context),
        )
        .await?;

    Ok((StatusCode::ACCEPTED, Json(AdminJobListItem::from_job(job))))
}

pub(super) async fn retry_admin_source_fingerprint_hash_job(
    State(app): State<NakoApp>,
    Path(job_id): Path<JobId>,
    Extension(http_trace_context): Extension<HttpTraceContext>,
    Json(request): Json<AdminSourceFingerprintHashRetryRequest>,
) -> ApiResult<impl IntoResponse> {
    tracing::debug!(
        request_id = %http_trace_context.request_id(),
        job_id = %job_id,
        "retrying source fingerprint hash job"
    );
    let job = app
        .source_hash()
        .retry_source_fingerprint_hash_job(RetrySourceFingerprintHashRequest {
            job_id,
            max_attempts: request.max_attempts,
            next_attempt_at: request.next_attempt_at,
        })
        .await?;

    Ok((StatusCode::ACCEPTED, Json(AdminJobListItem::from_job(job))))
}

pub(super) async fn get_admin_source_duplicate_reconciliation_plan(
    State(app): State<NakoApp>,
    Path((library_id, source_id)): Path<(LibraryId, MediaSourceId)>,
    Query(query): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page: PageRequest = query.try_into()?;
    let plan = app
        .source_duplicate_reconciliation()
        .plan_source_duplicate_reconciliation(SourceDuplicateReconciliationPlanRequest {
            library_id,
            source_id,
            page,
        })
        .await?;
    let returned = plan.candidates.len();

    Ok(Json(
        AdminSourceDuplicateReconciliationPlanResponse::from_plan(
            plan,
            page_info_from_request(page, returned),
        ),
    ))
}

pub(super) async fn apply_admin_source_duplicate_reconciliation(
    State(app): State<NakoApp>,
    Path((library_id, source_id)): Path<(LibraryId, MediaSourceId)>,
    Json(request): Json<AdminSourceDuplicateReconciliationApplyRequest>,
) -> ApiResult<impl IntoResponse> {
    let result = app
        .source_duplicate_reconciliation()
        .apply_source_duplicate_reconciliation(AppSourceDuplicateReconciliationApplyRequest {
            library_id,
            source_id,
            duplicate_source_id: request.duplicate_source_id,
            expected_action: request.expected_action.into(),
        })
        .await?;

    Ok(Json(
        AdminSourceDuplicateReconciliationApplyResponse::from_result(result),
    ))
}

fn admin_source_fingerprint_hash_mode(
    mode: AdminSourceFingerprintHashMode,
    partial_prefix_bytes: Option<u64>,
) -> Result<SourceFingerprintHashMode, NakoError> {
    match mode {
        AdminSourceFingerprintHashMode::Full => {
            if partial_prefix_bytes.is_some() {
                return Err(NakoError::InvalidInput {
                    message:
                        "partial_prefix_bytes is only valid for partial source fingerprint hash mode"
                            .to_owned(),
                });
            }

            Ok(SourceFingerprintHashMode::Full)
        }
        AdminSourceFingerprintHashMode::Partial => {
            let prefix_bytes = partial_prefix_bytes.ok_or_else(|| NakoError::InvalidInput {
                message: "partial source fingerprint hash mode requires partial_prefix_bytes"
                    .to_owned(),
            })?;

            nako_library::validate_source_fingerprint_hash_partial_prefix_bytes(
                prefix_bytes,
                "partial source fingerprint hash prefix must be greater than zero",
            )?;

            Ok(SourceFingerprintHashMode::Partial { prefix_bytes })
        }
    }
}

pub(super) async fn list_admin_playback_sessions(
    State(app): State<NakoApp>,
    Query(query): Query<PlaybackSessionListQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    let sessions = app.playback().list_playback_sessions(filter, page).await?;
    let returned = sessions.len();
    let sessions = sessions
        .into_iter()
        .map(AdminPlaybackSessionListItem::from_record)
        .collect();

    Ok(Json(AdminPlaybackSessionListResponse {
        sessions,
        page: page_info_from_request(page, returned),
    }))
}

pub(super) async fn get_admin_playback_runtime(
    State(app): State<NakoApp>,
) -> Json<AdminPlaybackRuntimeDiagnosticsResponse> {
    Json(admin_playback_runtime_diagnostics(&app).await)
}

pub(super) async fn get_admin_playback_renderers(
    State(app): State<NakoApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let sessions = app
        .renderer()
        .list_renderer_sessions_for_admin(page)
        .await?;
    let returned = sessions.len();
    let diagnostics = admin_renderer_runtime_diagnostics(sessions, page, returned);

    Ok(Json(diagnostics))
}

pub(super) async fn get_admin_playback_support_evidence(
    State(app): State<NakoApp>,
    Query(query): Query<PlaybackSupportEvidenceQuery>,
) -> ApiResult<impl IntoResponse> {
    let (session_id, source_id) = query.into_context()?;
    Ok(Json(
        admin_playback_support_evidence(&app, session_id, source_id).await?,
    ))
}

async fn admin_playback_support_evidence(
    app: &NakoApp,
    session_id: Option<TranscodeSessionId>,
    source_id: Option<MediaSourceId>,
) -> ApiResult<AdminPlaybackSupportEvidenceResponse> {
    let context = app
        .playback()
        .support_evidence_context(crate::app::playback::PlaybackSupportEvidenceRequest {
            session_id,
            source_id,
        })
        .await?;
    let runtime = admin_playback_runtime_diagnostics(&app).await;
    let subject_source_id = context
        .session
        .as_ref()
        .map(|session| session.source_id)
        .or_else(|| context.source.as_ref().map(|source| source.id))
        .or(source_id);

    Ok(AdminPlaybackSupportEvidenceResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        subject: AdminPlaybackSupportSubject {
            session_id,
            source_id: subject_source_id,
        },
        session: context
            .session
            .map(AdminPlaybackSupportSessionEvidence::from_record),
        source: context
            .source
            .map(AdminPlaybackSupportSourceEvidence::from_record),
        runtime: playback_support_runtime_evidence(runtime),
        redaction: AdminPlaybackSupportRedactionEvidence {
            paths_redacted: true,
            source_references_redacted: true,
            ffmpeg_commands_redacted: true,
            stderr_redacted: true,
            credentials_redacted: true,
        },
    })
}

fn admin_watch_folder_runtime_summary(
    report: WatchFolderRuntimeCoverageReport,
    latest_ticks: HashMap<LibraryId, WatchFolderRuntimeTickDiagnostic>,
) -> AdminOverviewWatchFolderRuntimeSummary {
    AdminOverviewWatchFolderRuntimeSummary {
        configured_libraries: usize_to_u32(report.diagnostics.len()),
        realtime_enabled_libraries: usize_to_u32(report.realtime_enabled_libraries()),
        started_libraries: usize_to_u32(report.started_libraries()),
        skipped_libraries: usize_to_u32(report.skipped_libraries()),
        diagnostics: report
            .diagnostics
            .into_iter()
            .map(|diagnostic| {
                admin_watch_folder_runtime_coverage_diagnostic(diagnostic, &latest_ticks)
            })
            .collect(),
    }
}

fn admin_watch_folder_runtime_coverage_diagnostic(
    diagnostic: WatchFolderRuntimeCoverageDiagnostic,
    latest_ticks: &HashMap<LibraryId, WatchFolderRuntimeTickDiagnostic>,
) -> AdminWatchFolderRuntimeCoverageDiagnostic {
    let last_tick = latest_ticks
        .get(&diagnostic.library_id)
        .map(admin_watch_folder_runtime_tick_diagnostic);

    AdminWatchFolderRuntimeCoverageDiagnostic {
        library_id: diagnostic.library_id,
        library_name: diagnostic.library_name,
        root_scheme: diagnostic.root_scheme,
        root_ref_redacted: diagnostic.root_ref_redacted,
        status: admin_watch_folder_runtime_coverage_status(diagnostic.status),
        safe_reason: diagnostic.safe_reason,
        last_tick,
    }
}

fn admin_watch_folder_runtime_tick_diagnostic(
    diagnostic: &WatchFolderRuntimeTickDiagnostic,
) -> AdminWatchFolderRuntimeTickDiagnostic {
    AdminWatchFolderRuntimeTickDiagnostic {
        monitored: diagnostic.monitored,
        ready_candidates: diagnostic.intake_plan.discover.ready_candidates,
        inspecting_candidates: diagnostic.intake_plan.discover.inspecting_candidates,
        blocked_candidates: diagnostic.intake_plan.discover.blocked_candidates,
        recorded_candidates: diagnostic.intake_plan.discover.recorded_candidates,
        newly_ready_candidates: diagnostic.intake_plan.discover.newly_ready_candidates,
        observed_candidates: diagnostic.intake_plan.summary.observed_candidates,
        suppressed_candidates: diagnostic.intake_plan.summary.suppressed_candidates,
        active_suppressions: diagnostic.intake_plan.suppression.active_suppressions,
        failure_count: diagnostic.intake_plan.summary.failure_count,
        enqueue_scan: diagnostic.intake_plan.summary.enqueue_scan,
        enqueue_reason: admin_watch_folder_intake_enqueue_reason(
            diagnostic.intake_plan.enqueue.reason,
        ),
        scan_admission_status: admin_watch_folder_scan_admission_status(
            diagnostic.scan_admission_status,
        ),
        scan_job_id: diagnostic.scan_job_id,
        reused_existing_scan: diagnostic.reused_existing_scan,
        backoff_required: diagnostic.backoff_required,
        discovery_failures: diagnostic
            .discovery_failures
            .iter()
            .map(|failure| AdminWatchFolderRuntimeFailureDiagnostic {
                ref_redacted: failure.uri_redacted.clone(),
                safe_message: failure.safe_message.clone(),
            })
            .collect(),
    }
}

fn admin_watch_folder_scan_admission_status(
    status: AppWatchFolderScanAdmissionStatus,
) -> AdminWatchFolderScanAdmissionStatus {
    match status {
        AppWatchFolderScanAdmissionStatus::NotAdmitted => {
            AdminWatchFolderScanAdmissionStatus::NotAdmitted
        }
        AppWatchFolderScanAdmissionStatus::Enqueued => {
            AdminWatchFolderScanAdmissionStatus::Enqueued
        }
        AppWatchFolderScanAdmissionStatus::ReusedQueued => {
            AdminWatchFolderScanAdmissionStatus::ReusedQueued
        }
        AppWatchFolderScanAdmissionStatus::ReusedRunning => {
            AdminWatchFolderScanAdmissionStatus::ReusedRunning
        }
    }
}

fn admin_watch_folder_runtime_coverage_status(
    status: WatchFolderRuntimeCoverageStatus,
) -> AdminWatchFolderRuntimeCoverageStatus {
    match status {
        WatchFolderRuntimeCoverageStatus::Started => AdminWatchFolderRuntimeCoverageStatus::Started,
        WatchFolderRuntimeCoverageStatus::Disabled => {
            AdminWatchFolderRuntimeCoverageStatus::Disabled
        }
        WatchFolderRuntimeCoverageStatus::UnsupportedRoot => {
            AdminWatchFolderRuntimeCoverageStatus::UnsupportedRoot
        }
        WatchFolderRuntimeCoverageStatus::MissingRoot => {
            AdminWatchFolderRuntimeCoverageStatus::MissingRoot
        }
    }
}

fn storage_summary(diagnostics: StorageBackendDiagnosticsResponse) -> AdminOverviewStorageSummary {
    let mut ready_backends = 0;
    let mut degraded_backends = 0;
    let mut unavailable_backends = 0;
    let backends = diagnostics
        .backends
        .into_iter()
        .map(|backend| {
            match backend.status {
                StorageBackendStatus::Ready => ready_backends += 1,
                StorageBackendStatus::Degraded => degraded_backends += 1,
                StorageBackendStatus::Unavailable => unavailable_backends += 1,
            }

            AdminOverviewStorageBackendSummary {
                library_id: backend.library_id,
                library_name: backend.library_name,
                backend_kind: backend.backend_kind,
                status: backend.status,
            }
        })
        .collect::<Vec<_>>();

    AdminOverviewStorageSummary {
        total_backends: usize_to_u32(backends.len()),
        ready_backends,
        degraded_backends,
        unavailable_backends,
        backends,
    }
}

fn metadata_summary(
    diagnostics: MetadataProviderDiagnosticsResponse,
) -> AdminOverviewMetadataSummary {
    let mut available_providers = 0;
    let mut disabled_providers = 0;
    let mut unavailable_providers = 0;
    let providers = diagnostics
        .providers
        .into_iter()
        .map(|provider| {
            match provider.status {
                MetadataProviderDiagnosticStatus::Available => available_providers += 1,
                MetadataProviderDiagnosticStatus::Disabled => disabled_providers += 1,
                MetadataProviderDiagnosticStatus::Unavailable => unavailable_providers += 1,
            }

            AdminOverviewMetadataProviderSummary {
                provider: provider.provider,
                status: provider.status,
            }
        })
        .collect::<Vec<_>>();

    AdminOverviewMetadataSummary {
        total_providers: usize_to_u32(providers.len()),
        available_providers,
        disabled_providers,
        unavailable_providers,
        providers,
    }
}

fn runtime_summary(diagnostics: RuntimeSupervisorDiagnostics) -> AdminOverviewRuntimeSummary {
    AdminOverviewRuntimeSummary {
        active_tasks: usize_to_u32(diagnostics.active_tasks),
        completed_tasks: diagnostics.completed_tasks,
        failed_tasks: diagnostics.failed_tasks,
        succeeded_jobs: diagnostics.succeeded_jobs,
        cancelled_jobs: diagnostics.cancelled_jobs,
        failed_jobs: diagnostics.failed_jobs,
        shutdown_requested: diagnostics.shutdown_requested,
    }
}

async fn admin_playback_runtime_diagnostics(
    app: &NakoApp,
) -> AdminPlaybackRuntimeDiagnosticsResponse {
    let playback = app.playback().runtime_diagnostics();
    let storage = app.storage().list_storage_backend_diagnostics().await;
    let startup = app.startup_report().clone();

    let capabilities = playback
        .hardware_report
        .capabilities
        .iter()
        .map(hardware_capability_diagnostic)
        .collect::<Vec<_>>();
    let transcode_budget = playback.transcode_budget.bounded();

    let remote_playback = remote_budget_summary(
        storage,
        playback.remote_stream_concurrency,
        playback.remote_stage_concurrency,
    );
    let staging = AdminPlaybackStagingDiagnostics {
        max_bytes: playback.staging_max_bytes,
        retention_ms: playback.staging_retention_ms,
        cleanup_on_startup: playback.staging_cleanup_on_startup,
        startup_deleted_records: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_records)),
        startup_deleted_files: startup
            .staging_cleanup
            .as_ref()
            .map_or(0, |cleanup| usize_to_u32(cleanup.deleted_files)),
    };
    let artifact_lifecycle = AdminPlaybackArtifactLifecycleDiagnostics {
        transcode_artifact_retention_ms: playback.transcode_artifact_retention_ms,
        transcode_artifact_cleanup_on_startup: playback.transcode_artifact_cleanup_on_startup,
        hls_segment_cleanup_enabled: playback.hls_segment_cleanup_enabled,
        hls_segment_keep_ms: playback.hls_segment_keep_ms,
        startup_examined_artifacts: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.examined_artifacts),
        startup_deleted_artifacts: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.deleted_artifacts),
        startup_deleted_files: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.deleted_files),
        startup_deleted_directories: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.deleted_directories),
        startup_deleted_bytes: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.deleted_bytes),
        startup_skipped_security: startup
            .playback_artifact_cleanup
            .as_ref()
            .map_or(0, |cleanup| cleanup.skipped_security),
    };
    let throttle = AdminPlaybackThrottleDiagnostics {
        enabled: playback.transcode_throttle_enabled,
        delay_ms: playback.transcode_throttle_delay_ms,
    };
    let policy = AdminPlaybackPolicyDiagnostics::ready();
    let hls_pipeline_readiness =
        admin_transcode_pipeline_readiness(playback.hls_pipeline_readiness);
    let readiness = playback_readiness_diagnostics(
        playback.runtime_inventory.has_probe_error,
        hls_pipeline_readiness,
        playback.hls_pipeline_readiness.fallback_used,
        playback.transcode_budget,
        transcode_budget,
        &remote_playback,
        &staging,
        &artifact_lifecycle,
        &throttle,
    );

    AdminPlaybackRuntimeDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        readiness,
        policy,
        ffmpeg: AdminPlaybackFfmpegDiagnostics {
            probe_status: playback_runtime_status(playback.runtime_inventory.probe_status),
            has_probe_error: playback.runtime_inventory.has_probe_error,
            hardware_capability_count: playback.runtime_inventory.hardware_capability_count,
            available_gpu_capabilities: playback.runtime_inventory.available_gpu_capabilities,
        },
        hardware: AdminPlaybackHardwareDiagnostics {
            policy: admin_hardware_policy(playback.hardware_policy),
            pipeline: hls_pipeline_readiness,
            capabilities,
        },
        transcode: AdminPlaybackTranscodeBudgetDiagnostics {
            configured_cpu_slots: playback.transcode_budget.cpu_slots,
            configured_gpu_slots: playback.transcode_budget.gpu_slots,
            effective_cpu_slots: transcode_budget.cpu_slots,
            effective_gpu_slots: transcode_budget.gpu_slots,
            selected_hls_slots: playback.selected_hls_slots,
        },
        remux: AdminPlaybackRemuxRuntimeDiagnostics {
            max_concurrent_sessions: playback.remux_concurrency,
            timeout_ms: playback.remux_timeout_ms,
        },
        resource_pressure: admin_playback_resource_pressure(
            &playback.resource_pressure,
            &remote_playback,
        ),
        remote_playback,
        staging,
        artifact_lifecycle,
        throttle,
    }
}

fn admin_playback_resource_pressure(
    pressure: &crate::app::playback::PlaybackRuntimeResourcePressure,
    remote_playback: &AdminPlaybackRemoteBudgetDiagnostics,
) -> AdminPlaybackResourcePressureDiagnostics {
    AdminPlaybackResourcePressureDiagnostics {
        classes: pressure
            .classes
            .iter()
            .map(|pressure| admin_playback_resource_class_pressure(pressure, remote_playback))
            .collect(),
    }
}

fn admin_playback_resource_class_pressure(
    pressure: &crate::app::playback::PlaybackRuntimeResourceClassPressure,
    remote_playback: &AdminPlaybackRemoteBudgetDiagnostics,
) -> AdminPlaybackResourceClassPressure {
    let mut pressure = AdminPlaybackResourceClassPressure {
        class: admin_playback_resource_class(pressure.class),
        enforcement: admin_playback_resource_enforcement(pressure.enforcement),
        configured_capacity: pressure.configured_capacity,
        available_permits: pressure.available_permits,
        in_use_permits: pressure.in_use_permits,
    };

    match pressure.class {
        AdminPlaybackResourceClass::RemoteStream => {
            pressure.configured_capacity = Some(remote_playback.stream_permits_max);
            pressure.available_permits = Some(remote_playback.stream_permits_available);
            pressure.in_use_permits = Some(
                remote_playback
                    .stream_permits_max
                    .saturating_sub(remote_playback.stream_permits_available),
            );
        }
        AdminPlaybackResourceClass::RemoteStage => {
            pressure.configured_capacity = Some(remote_playback.stage_permits_max);
            pressure.available_permits = Some(remote_playback.stage_permits_available);
            pressure.in_use_permits = Some(
                remote_playback
                    .stage_permits_max
                    .saturating_sub(remote_playback.stage_permits_available),
            );
        }
        AdminPlaybackResourceClass::RemuxProcess
        | AdminPlaybackResourceClass::CpuTranscode
        | AdminPlaybackResourceClass::GpuTranscode
        | AdminPlaybackResourceClass::HlsArtifactIo => {}
    }

    pressure
}

fn admin_playback_resource_class(
    class: crate::app::playback::PlaybackResourceClass,
) -> AdminPlaybackResourceClass {
    match class {
        crate::app::playback::PlaybackResourceClass::RemoteStream => {
            AdminPlaybackResourceClass::RemoteStream
        }
        crate::app::playback::PlaybackResourceClass::RemoteStage => {
            AdminPlaybackResourceClass::RemoteStage
        }
        crate::app::playback::PlaybackResourceClass::RemuxProcess => {
            AdminPlaybackResourceClass::RemuxProcess
        }
        crate::app::playback::PlaybackResourceClass::CpuTranscode => {
            AdminPlaybackResourceClass::CpuTranscode
        }
        crate::app::playback::PlaybackResourceClass::GpuTranscode => {
            AdminPlaybackResourceClass::GpuTranscode
        }
        crate::app::playback::PlaybackResourceClass::HlsArtifactIo => {
            AdminPlaybackResourceClass::HlsArtifactIo
        }
    }
}

fn admin_playback_resource_enforcement(
    enforcement: crate::app::playback::PlaybackResourceEnforcement,
) -> AdminPlaybackResourceEnforcement {
    match enforcement {
        crate::app::playback::PlaybackResourceEnforcement::HostOwned => {
            AdminPlaybackResourceEnforcement::HostOwned
        }
        crate::app::playback::PlaybackResourceEnforcement::AdmissionPermit => {
            AdminPlaybackResourceEnforcement::AdmissionPermit
        }
    }
}

fn playback_runtime_status(status: TranscodeRuntimeInventoryStatus) -> AdminPlaybackRuntimeStatus {
    match status {
        TranscodeRuntimeInventoryStatus::Ready => AdminPlaybackRuntimeStatus::Ready,
        TranscodeRuntimeInventoryStatus::Degraded => AdminPlaybackRuntimeStatus::Degraded,
    }
}

fn admin_renderer_runtime_diagnostics(
    sessions: Vec<RendererSessionRecord>,
    page: PageRequest,
    returned: usize,
) -> AdminRendererRuntimeDiagnosticsResponse {
    let now_ms = crate::app::current_time_ms().unwrap_or(i64::MAX);
    let sessions = sessions
        .into_iter()
        .map(|session| AdminRendererSessionDiagnostics::from_record(session, now_ms))
        .collect::<Vec<_>>();
    let summary = renderer_session_summary(&sessions);

    AdminRendererRuntimeDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        readiness: AdminRendererReadinessDiagnostics::ready(),
        summary,
        adapters: renderer_adapter_diagnostics(),
        sessions,
        page: page_info_from_request(page, returned),
    }
}

fn renderer_session_summary(
    sessions: &[AdminRendererSessionDiagnostics],
) -> AdminRendererSessionSummary {
    AdminRendererSessionSummary {
        returned_sessions: usize_to_u32(sessions.len()),
        online_sessions: usize_to_u32(
            sessions
                .iter()
                .filter(|session| session.state == RendererSessionState::Online)
                .count(),
        ),
        offline_sessions: usize_to_u32(
            sessions
                .iter()
                .filter(|session| session.state == RendererSessionState::Offline)
                .count(),
        ),
        revoked_sessions: usize_to_u32(
            sessions
                .iter()
                .filter(|session| session.state == RendererSessionState::Revoked)
                .count(),
        ),
        expired_sessions: usize_to_u32(sessions.iter().filter(|session| session.expired).count()),
        active_playback_sessions: usize_to_u32(
            sessions
                .iter()
                .filter(|session| session.active_playback_session_id.is_some())
                .count(),
        ),
    }
}

fn renderer_adapter_diagnostics() -> Vec<AdminRendererAdapterDiagnostics> {
    vec![
        AdminRendererAdapterDiagnostics {
            adapter: AdminRendererAdapterKind::NakoRemoteClient,
            target_kind: PlaybackTargetKind::NakoRemoteClient,
            status: AdminRendererAdapterStatus::Ready,
            reason: AdminRendererAdapterReason::NakoRemoteClientReady,
            control_plane: AdminRendererControlPlane::PublicClientPolling,
            discovery: AdminRendererDiscoveryMode::ClientRegistration,
            media_transport: AdminRendererMediaTransport::AuthenticatedNakoClientStream,
            transport_auth: PlaybackTargetTransportAuth::Bearer,
        },
        AdminRendererAdapterDiagnostics {
            adapter: AdminRendererAdapterKind::NakoRemoteClientCastSafeTransport,
            target_kind: PlaybackTargetKind::NakoRemoteClient,
            status: AdminRendererAdapterStatus::Ready,
            reason: AdminRendererAdapterReason::CastSafeTransportReady,
            control_plane: AdminRendererControlPlane::PublicClientPolling,
            discovery: AdminRendererDiscoveryMode::ClientRegistration,
            media_transport: AdminRendererMediaTransport::CastSafeUrl,
            transport_auth: PlaybackTargetTransportAuth::CastTicket,
        },
        AdminRendererAdapterDiagnostics {
            adapter: AdminRendererAdapterKind::Chromecast,
            target_kind: PlaybackTargetKind::Chromecast,
            status: AdminRendererAdapterStatus::Planned,
            reason: AdminRendererAdapterReason::ChromecastAdapterPlanned,
            control_plane: AdminRendererControlPlane::AdapterProcess,
            discovery: AdminRendererDiscoveryMode::LocalNetworkDiscovery,
            media_transport: AdminRendererMediaTransport::CastSafeUrl,
            transport_auth: PlaybackTargetTransportAuth::CastTicket,
        },
        AdminRendererAdapterDiagnostics {
            adapter: AdminRendererAdapterKind::DlnaRenderer,
            target_kind: PlaybackTargetKind::DlnaRenderer,
            status: AdminRendererAdapterStatus::Planned,
            reason: AdminRendererAdapterReason::DlnaAdapterPlanned,
            control_plane: AdminRendererControlPlane::AdapterProcess,
            discovery: AdminRendererDiscoveryMode::LocalNetworkDiscovery,
            media_transport: AdminRendererMediaTransport::CastSafeUrl,
            transport_auth: PlaybackTargetTransportAuth::CastTicket,
        },
        AdminRendererAdapterDiagnostics {
            adapter: AdminRendererAdapterKind::Airplay,
            target_kind: PlaybackTargetKind::Airplay,
            status: AdminRendererAdapterStatus::Planned,
            reason: AdminRendererAdapterReason::AirplayAdapterPlanned,
            control_plane: AdminRendererControlPlane::AdapterProcess,
            discovery: AdminRendererDiscoveryMode::PlatformDiscovery,
            media_transport: AdminRendererMediaTransport::NativeProtocolStream,
            transport_auth: PlaybackTargetTransportAuth::CastTicket,
        },
    ]
}

fn playback_support_runtime_evidence(
    runtime: AdminPlaybackRuntimeDiagnosticsResponse,
) -> AdminPlaybackSupportRuntimeEvidence {
    let unavailable_capabilities = runtime
        .hardware
        .capabilities
        .iter()
        .filter(|capability| !capability.available)
        .map(
            |capability| AdminPlaybackSupportHardwareCapabilityEvidence {
                accelerator: capability.accelerator,
                reason_code: capability.reason_code,
                encoder_discovery_status: capability.encoder_discovery.status,
                device_initialization_status: capability.device_initialization.status,
                smoke_probe_status: capability.smoke_probe.status,
            },
        )
        .collect();

    AdminPlaybackSupportRuntimeEvidence {
        readiness: runtime.readiness,
        policy: runtime.policy,
        ffmpeg: runtime.ffmpeg,
        hardware: AdminPlaybackSupportHardwareEvidence {
            policy: runtime.hardware.policy,
            selected_acceleration: runtime.hardware.pipeline.selected,
            fallback_used: runtime.hardware.pipeline.fallback_used,
            capability_count: usize_to_u32(runtime.hardware.capabilities.len()),
            unavailable_capabilities,
        },
        transcode: runtime.transcode,
        remux: runtime.remux,
        remote_playback: runtime.remote_playback,
        staging: runtime.staging,
        artifact_lifecycle: runtime.artifact_lifecycle,
        throttle: runtime.throttle,
    }
}

fn operator_readiness_summary(
    config: &crate::config::NakoServerConfig,
    storage: &AdminOverviewStorageSummary,
    runtime: &AdminOverviewRuntimeSummary,
    source_fingerprint_hash: &AdminOverviewSourceFingerprintHashSummary,
    vfs_cache_repair_pressure: Option<&VfsCacheRepairReadinessPressure>,
    startup: &AdminOverviewStartupSummary,
    network: AdminNetworkReadinessDiagnostics,
    playback: AdminPlaybackReadinessDiagnostics,
) -> AdminOperatorReadinessSummary {
    AdminOperatorReadinessSummary::from_checks(vec![
        setup_readiness_check(config),
        media_library_scan_readiness_check(startup, runtime, source_fingerprint_hash),
        playback_readiness_check(playback),
        storage_readiness_check(storage, vfs_cache_repair_pressure),
        network_readiness_check(network),
        backup_readiness_check(config),
    ])
}

fn setup_readiness_check(config: &crate::config::NakoServerConfig) -> AdminOperatorReadinessCheck {
    if config.auth.enabled
        && config
            .auth
            .token_env
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
    {
        return operator_check(
            AdminOperatorReadinessArea::Setup,
            AdminOperatorReadinessStatus::Ready,
            AdminOperatorReadinessReason::AuthConfigured,
            None,
            0,
            None,
        );
    }

    if config.auth.enabled {
        return operator_check(
            AdminOperatorReadinessArea::Setup,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::AuthTokenReferenceMissing,
            None,
            1,
            Some(operator_action(
                ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
                ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
            )),
        );
    }

    let (status, reason) = if matches!(
        config.network.exposure_mode,
        ConfigNetworkExposureMode::LocalOnly
    ) {
        (
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::AuthDisabledLocalOnly,
        )
    } else {
        (
            AdminOperatorReadinessStatus::Unavailable,
            AdminOperatorReadinessReason::AuthDisabledRemoteExposure,
        )
    };

    operator_check(
        AdminOperatorReadinessArea::Setup,
        status,
        reason,
        None,
        1,
        Some(operator_action(
            ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
            ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
        )),
    )
}

fn media_library_scan_readiness_check(
    startup: &AdminOverviewStartupSummary,
    runtime: &AdminOverviewRuntimeSummary,
    source_fingerprint_hash: &AdminOverviewSourceFingerprintHashSummary,
) -> AdminOperatorReadinessCheck {
    if startup.configured_libraries == 0 {
        return operator_check(
            AdminOperatorReadinessArea::MediaLibraryScan,
            AdminOperatorReadinessStatus::Unavailable,
            AdminOperatorReadinessReason::NoMediaLibraryConfigured,
            None,
            1,
            Some(operator_action(
                ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
                ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
            )),
        );
    }

    // Runtime job failures may already include Source Fingerprint hash failures.
    // Use the larger failed-job count so manually persisted failures still
    // degrade readiness without double-counting scheduler-recorded failures.
    let repair_pressure = runtime
        .failed_jobs
        .max(source_fingerprint_hash.failed_jobs)
        .saturating_add(runtime.failed_tasks);
    if repair_pressure > 0 {
        return operator_check(
            AdminOperatorReadinessArea::MediaLibraryScan,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::ScanRepairPressure,
            Some("failed_work".to_owned()),
            u64_to_u32(repair_pressure),
            Some(operator_action(ADMIN_JOBS_ROUTE_KEY, ADMIN_JOBS_ROUTE_PATH)),
        );
    }

    let pending_work = source_fingerprint_hash
        .queued_jobs
        .saturating_add(source_fingerprint_hash.running_jobs)
        .saturating_add(source_fingerprint_hash.delayed_retry_jobs);
    if pending_work > 0 {
        return operator_check(
            AdminOperatorReadinessArea::MediaLibraryScan,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::ScanWorkPending,
            Some("queued_work".to_owned()),
            u64_to_u32(pending_work),
            Some(operator_action(ADMIN_JOBS_ROUTE_KEY, ADMIN_JOBS_ROUTE_PATH)),
        );
    }

    if let Some((source_reason, attention_count)) =
        watch_folder_runtime_coverage_gap(&startup.watch_folder_runtime)
    {
        return operator_check(
            AdminOperatorReadinessArea::MediaLibraryScan,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::WatchFolderRuntimeCoverageGap,
            Some(source_reason),
            attention_count,
            Some(operator_action(
                ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
                ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
            )),
        );
    }

    operator_check(
        AdminOperatorReadinessArea::MediaLibraryScan,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::MediaLibraryConfigured,
        None,
        0,
        None,
    )
}

fn watch_folder_runtime_coverage_gap(
    runtime: &AdminOverviewWatchFolderRuntimeSummary,
) -> Option<(String, u32)> {
    let unsupported_roots = runtime
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.status == AdminWatchFolderRuntimeCoverageStatus::UnsupportedRoot
        })
        .count();
    let missing_roots = runtime
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.status == AdminWatchFolderRuntimeCoverageStatus::MissingRoot
        })
        .count();
    let gap_count = unsupported_roots.saturating_add(missing_roots);

    if gap_count == 0 {
        return None;
    }

    let source_reason = if unsupported_roots > 0 {
        "unsupported_root"
    } else {
        "missing_root"
    };

    Some((source_reason.to_owned(), usize_to_u32(gap_count)))
}

fn playback_readiness_check(
    playback: AdminPlaybackReadinessDiagnostics,
) -> AdminOperatorReadinessCheck {
    let status = operator_playback_status(playback.status);
    let reason = match playback.status {
        AdminPlaybackReadinessStatus::Ready => AdminOperatorReadinessReason::PlaybackReady,
        AdminPlaybackReadinessStatus::Degraded => AdminOperatorReadinessReason::PlaybackDegraded,
        AdminPlaybackReadinessStatus::Unavailable => {
            AdminOperatorReadinessReason::PlaybackUnavailable
        }
    };
    let attention_count = usize_to_u32(
        playback
            .checks
            .iter()
            .filter(|check| check.status != AdminPlaybackReadinessStatus::Ready)
            .count(),
    );

    operator_check(
        AdminOperatorReadinessArea::Playback,
        status,
        reason,
        enum_code(playback.reason),
        attention_count,
        (status != AdminOperatorReadinessStatus::Ready).then(|| {
            operator_action(
                ADMIN_PLAYBACK_RUNTIME_ROUTE_KEY,
                ADMIN_PLAYBACK_RUNTIME_ROUTE_PATH,
            )
        }),
    )
}

fn storage_readiness_check(
    storage: &AdminOverviewStorageSummary,
    vfs_cache_repair_pressure: Option<&VfsCacheRepairReadinessPressure>,
) -> AdminOperatorReadinessCheck {
    let attention_count = storage
        .degraded_backends
        .saturating_add(storage.unavailable_backends);

    if storage.unavailable_backends > 0 {
        return operator_check(
            AdminOperatorReadinessArea::Storage,
            AdminOperatorReadinessStatus::Unavailable,
            AdminOperatorReadinessReason::StorageUnavailable,
            Some("unavailable_backends".to_owned()),
            attention_count,
            Some(operator_action(
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_KEY,
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_PATH,
            )),
        );
    }

    if storage.degraded_backends > 0 {
        return operator_check(
            AdminOperatorReadinessArea::Storage,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::StorageDegraded,
            Some("degraded_backends".to_owned()),
            attention_count,
            Some(operator_action(
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_KEY,
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_PATH,
            )),
        );
    }

    if let Some(pressure) = vfs_cache_repair_pressure {
        return operator_check(
            AdminOperatorReadinessArea::Storage,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::VfsCacheRepairPressure,
            enum_code(admin_vfs_cache_repair_classification(
                pressure.primary_classification,
            )),
            pressure.total_unresolved_targets,
            Some(operator_action(
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_KEY,
                ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_PATH,
            )),
        );
    }

    operator_check(
        AdminOperatorReadinessArea::Storage,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::StorageReady,
        None,
        0,
        None,
    )
}

fn network_readiness_check(
    network: AdminNetworkReadinessDiagnostics,
) -> AdminOperatorReadinessCheck {
    let status = operator_network_status(network.status);
    let reason = match network.status {
        AdminNetworkReadinessStatus::Ready => AdminOperatorReadinessReason::NetworkReady,
        AdminNetworkReadinessStatus::Degraded => AdminOperatorReadinessReason::NetworkDegraded,
        AdminNetworkReadinessStatus::Unavailable => {
            AdminOperatorReadinessReason::NetworkUnavailable
        }
    };
    let attention_count = usize_to_u32(
        network
            .checks
            .iter()
            .filter(|check| check.status != AdminNetworkReadinessStatus::Ready)
            .count(),
    );

    operator_check(
        AdminOperatorReadinessArea::Network,
        status,
        reason,
        enum_code(network.reason),
        attention_count,
        (status != AdminOperatorReadinessStatus::Ready).then(|| {
            operator_action(
                ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
                ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
            )
        }),
    )
}

fn backup_readiness_check(config: &crate::config::NakoServerConfig) -> AdminOperatorReadinessCheck {
    if config
        .database_url
        .trim()
        .eq_ignore_ascii_case("sqlite::memory:")
    {
        return operator_check(
            AdminOperatorReadinessArea::Backup,
            AdminOperatorReadinessStatus::Degraded,
            AdminOperatorReadinessReason::BackupNeedsDurableDatabase,
            Some("ephemeral_database".to_owned()),
            1,
            Some(operator_action(
                ADMIN_SYSTEM_CONFIG_ROUTE_KEY,
                ADMIN_SYSTEM_CONFIG_ROUTE_PATH,
            )),
        );
    }

    operator_check(
        AdminOperatorReadinessArea::Backup,
        AdminOperatorReadinessStatus::Ready,
        AdminOperatorReadinessReason::BackupRunbookAvailable,
        Some("backup_restore_runbook".to_owned()),
        0,
        None,
    )
}

fn operator_check(
    area: AdminOperatorReadinessArea,
    status: AdminOperatorReadinessStatus,
    reason: AdminOperatorReadinessReason,
    source_reason: Option<String>,
    attention_count: u32,
    action: Option<AdminOperatorReadinessAction>,
) -> AdminOperatorReadinessCheck {
    AdminOperatorReadinessCheck {
        area,
        status,
        reason,
        source_reason,
        attention_count,
        action,
    }
}

fn operator_action(route_key: &str, route_path: &str) -> AdminOperatorReadinessAction {
    AdminOperatorReadinessAction {
        route_key: route_key.to_owned(),
        route_path: route_path.to_owned(),
    }
}

fn operator_playback_status(status: AdminPlaybackReadinessStatus) -> AdminOperatorReadinessStatus {
    match status {
        AdminPlaybackReadinessStatus::Ready => AdminOperatorReadinessStatus::Ready,
        AdminPlaybackReadinessStatus::Degraded => AdminOperatorReadinessStatus::Degraded,
        AdminPlaybackReadinessStatus::Unavailable => AdminOperatorReadinessStatus::Unavailable,
    }
}

fn operator_network_status(status: AdminNetworkReadinessStatus) -> AdminOperatorReadinessStatus {
    match status {
        AdminNetworkReadinessStatus::Ready => AdminOperatorReadinessStatus::Ready,
        AdminNetworkReadinessStatus::Degraded => AdminOperatorReadinessStatus::Degraded,
        AdminNetworkReadinessStatus::Unavailable => AdminOperatorReadinessStatus::Unavailable,
    }
}

fn enum_code(value: impl Serialize) -> Option<String> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
}

fn overview_status(
    storage: &AdminOverviewStorageSummary,
    metadata: &AdminOverviewMetadataSummary,
    runtime: &AdminOverviewRuntimeSummary,
) -> AdminOverviewStatus {
    if storage.degraded_backends > 0
        || storage.unavailable_backends > 0
        || metadata.unavailable_providers > 0
        || runtime.failed_tasks > 0
        || runtime.failed_jobs > 0
        || runtime.shutdown_requested
    {
        AdminOverviewStatus::Degraded
    } else {
        AdminOverviewStatus::Healthy
    }
}

fn hardware_capability_diagnostic(
    capability: &HardwareAccelerationCapability,
) -> AdminPlaybackHardwareCapability {
    AdminPlaybackHardwareCapability {
        accelerator: admin_hardware_acceleration(capability.accelerator),
        available: capability.available,
        reason_code: hardware_capability_reason(capability),
        stage_capabilities: capability
            .stage_capabilities
            .iter()
            .map(|stage| AdminPlaybackHardwareStageCapability {
                stage: admin_hardware_pipeline_stage(stage.stage),
                available: stage.available,
                required: stage.required,
                feature: stage.feature.clone(),
                discovery_status: hardware_encoder_discovery_status(stage.discovery_status),
                has_detail: stage.detail.is_some(),
            })
            .collect(),
        encoder_discovery: AdminPlaybackHardwareEncoderDiscovery {
            status: hardware_encoder_discovery_status(capability.encoder_discovery.status),
            encoder: capability.encoder_discovery.encoder.clone(),
            has_detail: capability.encoder_discovery.detail.is_some(),
        },
        device_initialization: AdminPlaybackHardwareDeviceInitialization {
            status: hardware_device_initialization_status(capability.device_initialization.status),
            operator_check: capability.device_initialization.operator_check.clone(),
            has_detail: capability.device_initialization.detail.is_some(),
        },
        smoke_probe: AdminPlaybackHardwareSmokeProbe {
            status: hardware_smoke_probe_status(capability.smoke_probe.status),
            operator_check: capability.smoke_probe.operator_check.clone(),
            has_detail: capability.smoke_probe.detail.is_some(),
        },
    }
}

fn hardware_capability_reason(
    capability: &HardwareAccelerationCapability,
) -> AdminPlaybackHardwareCapabilityReason {
    if capability.available {
        return AdminPlaybackHardwareCapabilityReason::Available;
    }

    match capability.encoder_discovery.status {
        HardwareEncoderDiscoveryStatus::Missing => {
            return AdminPlaybackHardwareCapabilityReason::EncoderNotListed;
        }
        HardwareEncoderDiscoveryStatus::ProbeError => {
            return AdminPlaybackHardwareCapabilityReason::ProbeError;
        }
        HardwareEncoderDiscoveryStatus::NotRequired
        | HardwareEncoderDiscoveryStatus::Listed
        | HardwareEncoderDiscoveryStatus::Static => {}
    }

    if capability.device_initialization.status == HardwareDeviceInitializationStatus::Failed {
        return AdminPlaybackHardwareCapabilityReason::DeviceInitializationFailed;
    }

    if capability.smoke_probe.status == HardwareSmokeProbeStatus::Failed {
        return AdminPlaybackHardwareCapabilityReason::SmokeProbeFailed;
    }

    AdminPlaybackHardwareCapabilityReason::ProbeError
}

fn hardware_encoder_discovery_status(
    status: HardwareEncoderDiscoveryStatus,
) -> AdminPlaybackHardwareEncoderDiscoveryStatus {
    match status {
        HardwareEncoderDiscoveryStatus::NotRequired => {
            AdminPlaybackHardwareEncoderDiscoveryStatus::NotRequired
        }
        HardwareEncoderDiscoveryStatus::Listed => {
            AdminPlaybackHardwareEncoderDiscoveryStatus::Listed
        }
        HardwareEncoderDiscoveryStatus::Missing => {
            AdminPlaybackHardwareEncoderDiscoveryStatus::Missing
        }
        HardwareEncoderDiscoveryStatus::ProbeError => {
            AdminPlaybackHardwareEncoderDiscoveryStatus::ProbeError
        }
        HardwareEncoderDiscoveryStatus::Static => {
            AdminPlaybackHardwareEncoderDiscoveryStatus::Static
        }
    }
}

fn hardware_device_initialization_status(
    status: HardwareDeviceInitializationStatus,
) -> AdminPlaybackHardwareDeviceInitializationStatus {
    match status {
        HardwareDeviceInitializationStatus::NotRequired => {
            AdminPlaybackHardwareDeviceInitializationStatus::NotRequired
        }
        HardwareDeviceInitializationStatus::NotRun => {
            AdminPlaybackHardwareDeviceInitializationStatus::NotRun
        }
        HardwareDeviceInitializationStatus::Passed => {
            AdminPlaybackHardwareDeviceInitializationStatus::Passed
        }
        HardwareDeviceInitializationStatus::Failed => {
            AdminPlaybackHardwareDeviceInitializationStatus::Failed
        }
    }
}

fn hardware_smoke_probe_status(
    status: HardwareSmokeProbeStatus,
) -> AdminPlaybackHardwareSmokeProbeStatus {
    match status {
        HardwareSmokeProbeStatus::NotRequired => AdminPlaybackHardwareSmokeProbeStatus::NotRequired,
        HardwareSmokeProbeStatus::NotRun => AdminPlaybackHardwareSmokeProbeStatus::NotRun,
        HardwareSmokeProbeStatus::Passed => AdminPlaybackHardwareSmokeProbeStatus::Passed,
        HardwareSmokeProbeStatus::Failed => AdminPlaybackHardwareSmokeProbeStatus::Failed,
    }
}

fn remote_budget_summary(
    diagnostics: StorageBackendDiagnosticsResponse,
    configured_stream_permits: usize,
    configured_stage_permits: usize,
) -> AdminPlaybackRemoteBudgetDiagnostics {
    let backend_count = diagnostics.backends.len();
    let mut stream_permits_available = 0;
    let mut stream_permits_max = 0;
    let mut stage_permits_available = 0;
    let mut stage_permits_max = 0;

    for backend in diagnostics.backends {
        stream_permits_available += backend.registry.stream_permits_available;
        stream_permits_max += backend.registry.stream_permits_max;
        stage_permits_available += backend.registry.stage_permits_available;
        stage_permits_max += backend.registry.stage_permits_max;
    }

    AdminPlaybackRemoteBudgetDiagnostics {
        backend_count: usize_to_u32(backend_count),
        stream_permits_available,
        stream_permits_max: stream_permits_max.max(configured_stream_permits),
        stage_permits_available,
        stage_permits_max: stage_permits_max.max(configured_stage_permits),
        state_scope: StorageBackendRuntimeStateScope::ProcessLocal,
    }
}

fn playback_readiness_diagnostics(
    has_probe_error: bool,
    hardware_readiness: AdminTranscodePipelineReadiness,
    fallback_used: bool,
    configured_budget: nako_transcode::TranscodeResourceBudget,
    effective_budget: nako_transcode::TranscodeResourceBudget,
    remote_playback: &AdminPlaybackRemoteBudgetDiagnostics,
    staging: &AdminPlaybackStagingDiagnostics,
    _artifact_lifecycle: &AdminPlaybackArtifactLifecycleDiagnostics,
    _throttle: &AdminPlaybackThrottleDiagnostics,
) -> AdminPlaybackReadinessDiagnostics {
    AdminPlaybackReadinessDiagnostics::from_checks(vec![
        if has_probe_error {
            AdminPlaybackReadinessCheck::degraded(
                AdminPlaybackReadinessCheckName::FfmpegProbe,
                AdminPlaybackReadinessReason::ProbeError,
            )
        } else {
            AdminPlaybackReadinessCheck::ready(
                AdminPlaybackReadinessCheckName::FfmpegProbe,
                AdminPlaybackReadinessReason::FfmpegProbeReady,
            )
        },
        AdminPlaybackReadinessCheck::from_hardware(hardware_readiness),
        if hardware_readiness.status == AdminTranscodePipelineReadinessStatus::Unavailable {
            AdminPlaybackReadinessCheck::unavailable(
                AdminPlaybackReadinessCheckName::SelectedFallback,
                hardware_readiness.reason.into(),
            )
        } else if fallback_used {
            AdminPlaybackReadinessCheck::degraded(
                AdminPlaybackReadinessCheckName::SelectedFallback,
                AdminPlaybackReadinessReason::CpuFallbackActive,
            )
        } else {
            AdminPlaybackReadinessCheck::ready(
                AdminPlaybackReadinessCheckName::SelectedFallback,
                AdminPlaybackReadinessReason::SelectedAccelerationReady,
            )
        },
        if configured_budget.cpu_slots == effective_budget.cpu_slots
            && configured_budget.gpu_slots == effective_budget.gpu_slots
        {
            AdminPlaybackReadinessCheck::ready(
                AdminPlaybackReadinessCheckName::TranscodeBudget,
                AdminPlaybackReadinessReason::TranscodeBudgetReady,
            )
        } else {
            AdminPlaybackReadinessCheck::degraded(
                AdminPlaybackReadinessCheckName::TranscodeBudget,
                AdminPlaybackReadinessReason::TranscodeBudgetClamped,
            )
        },
        if remote_playback.stream_permits_max > 0 && remote_playback.stage_permits_max > 0 {
            AdminPlaybackReadinessCheck::ready(
                AdminPlaybackReadinessCheckName::RemotePlaybackBudget,
                AdminPlaybackReadinessReason::RemotePlaybackBudgetReady,
            )
        } else {
            AdminPlaybackReadinessCheck::degraded(
                AdminPlaybackReadinessCheckName::RemotePlaybackBudget,
                AdminPlaybackReadinessReason::RemotePlaybackBudgetClamped,
            )
        },
        AdminPlaybackReadinessCheck::ready(
            AdminPlaybackReadinessCheckName::PlaybackPolicy,
            AdminPlaybackReadinessReason::PlaybackPolicyReady,
        ),
        if staging.max_bytes > 0 {
            AdminPlaybackReadinessCheck::ready(
                AdminPlaybackReadinessCheckName::Staging,
                AdminPlaybackReadinessReason::StagingReady,
            )
        } else {
            AdminPlaybackReadinessCheck::unavailable(
                AdminPlaybackReadinessCheckName::Staging,
                AdminPlaybackReadinessReason::StagingBudgetDisabled,
            )
        },
        AdminPlaybackReadinessCheck::ready(
            AdminPlaybackReadinessCheckName::ArtifactLifecycle,
            AdminPlaybackReadinessReason::ArtifactLifecycleReady,
        ),
        AdminPlaybackReadinessCheck::ready(
            AdminPlaybackReadinessCheckName::TranscodeThrottle,
            AdminPlaybackReadinessReason::TranscodeThrottleReady,
        ),
    ])
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn u64_to_u32(value: u64) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_staging_pressure_status_uses_redacted_thresholds() {
        assert_eq!(
            storage_staging_pressure_status(0, 10),
            AdminStorageStagingPressureStatus::Disabled
        );
        assert_eq!(
            storage_staging_pressure_status(1_000, 749),
            AdminStorageStagingPressureStatus::Healthy
        );
        assert_eq!(
            storage_staging_pressure_status(1_000, 750),
            AdminStorageStagingPressureStatus::Elevated
        );
        assert_eq!(
            storage_staging_pressure_status(1_000, 900),
            AdminStorageStagingPressureStatus::Critical
        );
        assert_eq!(
            storage_staging_pressure_status(1_000, 1_000),
            AdminStorageStagingPressureStatus::Exhausted
        );
        assert_eq!(storage_staging_used_ratio_milli(0, 10), None);
        assert_eq!(storage_staging_used_ratio_milli(1_000, 1_250), Some(1_250));
    }

    #[test]
    fn media_library_scan_readiness_reports_watch_folder_runtime_coverage_gap() {
        let startup = startup_summary_with_watch_folder_diagnostics(vec![
            AdminWatchFolderRuntimeCoverageDiagnostic {
                library_id: LibraryId::new(),
                library_name: "Remote Movies".to_owned(),
                root_scheme: Some("webdav".to_owned()),
                root_ref_redacted: "webdav://<redacted>".to_owned(),
                status: AdminWatchFolderRuntimeCoverageStatus::UnsupportedRoot,
                safe_reason: "watch-folder runtime requires a local root".to_owned(),
                last_tick: None,
            },
            AdminWatchFolderRuntimeCoverageDiagnostic {
                library_id: LibraryId::new(),
                library_name: "Broken Movies".to_owned(),
                root_scheme: None,
                root_ref_redacted: "<redacted>".to_owned(),
                status: AdminWatchFolderRuntimeCoverageStatus::MissingRoot,
                safe_reason: "library has no parseable root".to_owned(),
                last_tick: None,
            },
        ]);
        let check = media_library_scan_readiness_check(
            &startup,
            &empty_runtime_summary(),
            &AdminOverviewSourceFingerprintHashSummary::default(),
        );
        let body = serde_json::to_string(&check).unwrap();

        assert_eq!(check.area, AdminOperatorReadinessArea::MediaLibraryScan);
        assert_eq!(check.status, AdminOperatorReadinessStatus::Degraded);
        assert_eq!(
            check.reason,
            AdminOperatorReadinessReason::WatchFolderRuntimeCoverageGap
        );
        assert_eq!(check.source_reason.as_deref(), Some("unsupported_root"));
        assert_eq!(check.attention_count, 2);
        assert_eq!(
            check
                .action
                .as_ref()
                .map(|action| action.route_key.as_str()),
            Some(ADMIN_SYSTEM_CONFIG_ROUTE_KEY)
        );
        assert!(!body.contains("webdav:///"));
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
    }

    #[test]
    fn media_library_scan_readiness_ignores_disabled_watch_folder_runtime() {
        let startup = startup_summary_with_watch_folder_diagnostics(vec![
            AdminWatchFolderRuntimeCoverageDiagnostic {
                library_id: LibraryId::new(),
                library_name: "Local Movies".to_owned(),
                root_scheme: Some("local".to_owned()),
                root_ref_redacted: "local://<redacted>".to_owned(),
                status: AdminWatchFolderRuntimeCoverageStatus::Disabled,
                safe_reason: "realtime monitoring is disabled".to_owned(),
                last_tick: None,
            },
        ]);
        let check = media_library_scan_readiness_check(
            &startup,
            &empty_runtime_summary(),
            &AdminOverviewSourceFingerprintHashSummary::default(),
        );

        assert_eq!(check.area, AdminOperatorReadinessArea::MediaLibraryScan);
        assert_eq!(check.status, AdminOperatorReadinessStatus::Ready);
        assert_eq!(
            check.reason,
            AdminOperatorReadinessReason::MediaLibraryConfigured
        );
        assert_eq!(check.attention_count, 0);
    }

    #[test]
    fn storage_readiness_reports_vfs_cache_repair_pressure() {
        let pressure = VfsCacheRepairReadinessPressure {
            primary_classification: VfsCacheRepairClassification::RetryableRefreshFailure,
            total_unresolved_targets: 2,
        };
        let check = storage_readiness_check(&ready_storage_summary(), Some(&pressure));
        let body = serde_json::to_string(&check).unwrap();

        assert_eq!(check.area, AdminOperatorReadinessArea::Storage);
        assert_eq!(check.status, AdminOperatorReadinessStatus::Degraded);
        assert_eq!(
            check.reason,
            AdminOperatorReadinessReason::VfsCacheRepairPressure
        );
        assert_eq!(
            check.source_reason.as_deref(),
            Some("retryable_refresh_failure")
        );
        assert_eq!(check.attention_count, 2);
        assert_eq!(
            check
                .action
                .as_ref()
                .map(|action| action.route_key.as_str()),
            Some(ADMIN_STORAGE_REPAIR_TARGETS_ROUTE_KEY)
        );
        assert!(!body.contains("local:///"));
        assert!(!body.contains("token"));
        assert!(!body.contains("password"));
    }

    #[test]
    fn storage_readiness_ignores_absent_vfs_cache_repair_pressure() {
        let check = storage_readiness_check(&ready_storage_summary(), None);

        assert_eq!(check.area, AdminOperatorReadinessArea::Storage);
        assert_eq!(check.status, AdminOperatorReadinessStatus::Ready);
        assert_eq!(check.reason, AdminOperatorReadinessReason::StorageReady);
        assert_eq!(check.attention_count, 0);
    }

    fn ready_storage_summary() -> AdminOverviewStorageSummary {
        AdminOverviewStorageSummary {
            total_backends: 1,
            ready_backends: 1,
            degraded_backends: 0,
            unavailable_backends: 0,
            backends: Vec::new(),
        }
    }

    fn startup_summary_with_watch_folder_diagnostics(
        diagnostics: Vec<AdminWatchFolderRuntimeCoverageDiagnostic>,
    ) -> AdminOverviewStartupSummary {
        let realtime_enabled_libraries = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.status != AdminWatchFolderRuntimeCoverageStatus::Disabled
            })
            .count();
        let started_libraries = diagnostics
            .iter()
            .filter(|diagnostic| {
                diagnostic.status == AdminWatchFolderRuntimeCoverageStatus::Started
            })
            .count();
        let configured_libraries = usize_to_u32(diagnostics.len());

        AdminOverviewStartupSummary {
            configured_libraries,
            recovered_transcode_sessions: 0,
            recovered_jobs: 0,
            staging_deleted_records: 0,
            staging_deleted_files: 0,
            metadata_raw_cache_deleted: 0,
            metadata_lifecycle_tasks_started: 0,
            artwork_ingest_worker_started: false,
            addon_event_scheduler_started: false,
            watch_folder_runtimes_started: usize_to_u32(started_libraries),
            watch_folder_runtime: AdminOverviewWatchFolderRuntimeSummary {
                configured_libraries,
                realtime_enabled_libraries: usize_to_u32(realtime_enabled_libraries),
                started_libraries: usize_to_u32(started_libraries),
                skipped_libraries: configured_libraries
                    .saturating_sub(usize_to_u32(started_libraries)),
                diagnostics,
            },
        }
    }

    fn empty_runtime_summary() -> AdminOverviewRuntimeSummary {
        AdminOverviewRuntimeSummary {
            active_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            succeeded_jobs: 0,
            cancelled_jobs: 0,
            failed_jobs: 0,
            shutdown_requested: false,
        }
    }
}
