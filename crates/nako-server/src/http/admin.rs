use std::collections::HashSet;

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
        AdminInvitationListResponse, AdminInvitationRecord, AdminInvitationResponse,
        AdminJobCancelRequestResponse, AdminJobListItem, AdminJobListResponse,
        AdminLibraryAccessLevel, AdminLibraryAccessPolicyDeleteResponse,
        AdminLibraryAccessPolicyListResponse, AdminLibraryAccessPolicyRecord,
        AdminLibraryAccessPolicyResponse, AdminLibraryAccessReason, AdminLibraryAccessSummary,
        AdminLibraryAccessSummaryEntry, AdminLibraryConfigDiagnostics,
        AdminMetadataConfigDiagnostics, AdminMetadataProviderConfigDiagnostics,
        AdminMetadataRuntimeConfigDiagnostics, AdminNetworkAccessDiagnostics,
        AdminNetworkExposureMode, AdminNetworkExternalEndpointDiagnostics,
        AdminNetworkReadinessCheck, AdminNetworkReadinessCheckName,
        AdminNetworkReadinessDiagnostics, AdminNetworkReadinessReason,
        AdminOriginPolicyDiagnostics, AdminOutboxEventListItem, AdminOutboxEventListResponse,
        AdminOverviewMetadataProviderSummary, AdminOverviewMetadataSummary, AdminOverviewResponse,
        AdminOverviewRuntimeSummary, AdminOverviewStartupSummary, AdminOverviewStatus,
        AdminOverviewStorageBackendSummary, AdminOverviewStorageSummary,
        AdminPlaybackArtifactLifecycleDiagnostics, AdminPlaybackFfmpegDiagnostics,
        AdminPlaybackHardwareCapability, AdminPlaybackHardwareCapabilityReason,
        AdminPlaybackHardwareDeviceInitialization, AdminPlaybackHardwareDeviceInitializationStatus,
        AdminPlaybackHardwareDiagnostics, AdminPlaybackHardwareEncoderDiscovery,
        AdminPlaybackHardwareEncoderDiscoveryStatus, AdminPlaybackHardwareSmokeProbe,
        AdminPlaybackHardwareSmokeProbeStatus, AdminPlaybackHardwareStageCapability,
        AdminPlaybackPolicyDiagnostics, AdminPlaybackReadinessCheck,
        AdminPlaybackReadinessCheckName, AdminPlaybackReadinessDiagnostics,
        AdminPlaybackReadinessReason, AdminPlaybackRemoteBudgetDiagnostics,
        AdminPlaybackRemuxRuntimeDiagnostics, AdminPlaybackResourceClass,
        AdminPlaybackResourceClassPressure, AdminPlaybackResourceEnforcement,
        AdminPlaybackResourcePressureDiagnostics, AdminPlaybackRuntimeDiagnosticsResponse,
        AdminPlaybackRuntimeStatus, AdminPlaybackSessionListItem, AdminPlaybackSessionListResponse,
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
        AdminSetLocalPasswordRequest, AdminStorageBackendHealthDiagnostic,
        AdminStorageBackendHealthDiagnosticsResponse, AdminStorageBackendHealthResetResponse,
        AdminStorageStagingDiagnosticsResponse, AdminStorageStagingRecord,
        AdminStorageStagingSummary, AdminTranscodeConfigDiagnostics,
        AdminTranscodePipelineReadiness, AdminTranscodePipelineReadinessStatus,
        AdminTrustedProxyDiagnostics, AdminTunnelProviderDiagnostics, AdminTunnelProviderKind,
        AdminUpdateLibraryMetadataProfileRequest, AdminUpdateMetadataRawCacheSettingsRequest,
        AdminUpdatePlaybackRuntimeSettingsRequest, AdminUpdateUserStatusRequest,
        AdminUpsertLibraryAccessPolicyRequest, AdminVfsCacheSummary,
        AdminWatchFolderDiscoveryFailure, AdminWatchFolderDiscoveryRequest,
        AdminWatchFolderDiscoveryResponse, JobResponse, StorageBackendDiagnosticsResponse,
        StorageBackendKind, StorageBackendRuntimeStateScope, StorageBackendStatus,
    },
    metadata_diagnostics::{MetadataProviderDiagnosticStatus, MetadataProviderDiagnosticsResponse},
    public_client::{API_VERSION, ClientErrorCode, ErrorResponse, page_info_from_request},
};
use nako_core::{
    ArtworkCandidateId, AutomationArtifactId, GeneratedArtifactMetadataApplyOutcomeId,
    GeneratedArtifactMetadataApplyRecoveryAttention, GeneratedArtifactMetadataApplyRecoveryFilter,
    GeneratedArtifactMetadataBulkApplyBatchId, ImageKind, JobId, LibraryAccessPolicy,
    LibraryAccessPolicyFilter, LibraryAccessPolicyScope, LibraryId, ManagedArtworkArtifactId,
    ManagedArtworkIngestId, MediaItemId, MetadataCandidateReviewId, NakoError, PageRequest,
    PlaybackTargetKind, PlaybackTargetTransportAuth, ProviderMappingId, RendererSessionRecord,
    RendererSessionState, RoleAssignment, User, UserId, UserInvitationId, UserPrincipalId,
    UserRole, UserStatus,
};
use nako_db::DatabaseBackendCapabilities;
use nako_transcode::{
    HardwareAccelerationCapability, HardwareDeviceInitializationStatus,
    HardwareEncoderDiscoveryStatus, HardwareSmokeProbeStatus, TranscodeRuntimeInventoryStatus,
};
use nako_vfs::StorageUri;
use serde::Deserialize;

use crate::{
    api_mapping::{
        admin_hardware_acceleration, admin_hardware_pipeline_stage, admin_hardware_policy,
        admin_transcode_pipeline_readiness,
    },
    app::{NakoApp, RuntimeSupervisorDiagnostics},
    config::{
        LocalLibraryConfig, MetadataProviderConfig, MetadataProviderRuntimeConfig,
        NetworkAccessConfig, NetworkExposureMode as ConfigNetworkExposureMode,
        TunnelProviderConfig, TunnelProviderKind as ConfigTunnelProviderKind,
    },
};

use super::{
    error::ApiResult,
    query::{
        AcquisitionIntakeCandidateListQuery, ArtworkArtifactLifecycleQuery,
        ArtworkArtifactRemediationQuery, ArtworkArtifactStorageDriftQuery, ArtworkGalleryQuery,
        CatalogGovernanceItemsQuery, JobListQuery, OutboxEventListQuery, PageQuery,
        PlaybackSessionListQuery, PlaybackSupportEvidenceQuery, StorageStagingQuery,
    },
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/admin/v1/overview", get(get_admin_overview))
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
            "/admin/v1/metadata/candidate-reviews/{review_id}",
            get(get_admin_metadata_candidate_review),
        )
        .route("/admin/v1/events", get(list_admin_outbox_events))
        .route("/admin/v1/jobs", get(list_admin_jobs))
        .route("/admin/v1/jobs/{job_id}/cancel", post(cancel_admin_job))
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

    Ok(Json(AdminWatchFolderDiscoveryResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        target_library_id: diagnostic.target_library_id,
        root_scheme: diagnostic.root_scheme,
        root_ref_redacted: diagnostic.root_uri_redacted,
        ready_candidates: diagnostic.ready_candidates,
        blocked_candidates: diagnostic.blocked_candidates,
        incomplete_candidates: diagnostic.incomplete_candidates,
        unsupported_candidates: diagnostic.unsupported_candidates,
        recorded_candidates: diagnostic.recorded_candidates,
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
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.library_scan().enqueue_library_scan(library_id).await?;

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
    Query(query): Query<ArtworkArtifactLifecycleQuery>,
) -> ApiResult<impl IntoResponse> {
    let (_filter, page) = query.into_filter_and_page()?;
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

pub(super) async fn get_admin_overview(State(app): State<NakoApp>) -> ApiResult<impl IntoResponse> {
    let storage = app.storage().list_storage_backend_diagnostics().await;
    let catalog = app.catalog().catalog_governance_summary().await?;
    let metadata = app.metadata().list_metadata_provider_diagnostics();
    let runtime = app.runtime_diagnostics();
    let startup = app.startup_report().clone();

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
    };
    let status = overview_status(&storage, &metadata, &runtime);

    Ok(Json(AdminOverviewResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        status,
        storage,
        catalog,
        metadata,
        runtime,
        startup,
    }))
}

pub(super) async fn get_admin_system_config(
    State(app): State<NakoApp>,
) -> Json<AdminServerConfigDiagnosticsResponse> {
    let config = app.config();

    Json(AdminServerConfigDiagnosticsResponse {
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
    })
}

pub(super) async fn get_admin_access_summary(
    State(app): State<NakoApp>,
    Extension(principal): Extension<UserPrincipalId>,
) -> ApiResult<impl IntoResponse> {
    let config = app.config();
    let user = app.get_user_by_principal(&principal).await?;
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
            principal_id: principal.to_string(),
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
    let startup = app.startup_report().clone();
    let process_cached_backends = usize_to_u32(app.storage().process_cached_backend_count().await);
    let used_manifest_bytes = app.storage().sum_staging_manifest_bytes().await?;
    let now_ms = crate::app::current_time_ms()?;
    let cleanup_pressure = app
        .storage()
        .summarize_staging_cleanup_pressure(now_ms)
        .await?;
    let vfs_cache = app.storage().summarize_vfs_cache(now_ms).await?;

    Ok(Json(AdminStorageStagingDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        summary: AdminStorageStagingSummary {
            configured_max_bytes: app.config().staging.max_bytes,
            used_manifest_bytes,
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
            },
        },
        records,
        page: page_info_from_request(page, returned),
    }))
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
    let returned = jobs.len();
    let jobs = jobs.into_iter().map(AdminJobListItem::from_job).collect();

    Ok(Json(AdminJobListResponse {
        jobs,
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

    Ok(Json(AdminPlaybackSupportEvidenceResponse {
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
    }))
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
        crate::app::playback::PlaybackResourceEnforcement::NotYetEnforced => {
            AdminPlaybackResourceEnforcement::NotYetEnforced
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
