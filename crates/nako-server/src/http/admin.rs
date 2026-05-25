use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
};
use nako_api::{
    admin::{
        ADMIN_API_VERSION, AdminAcquisitionIntakeCandidateDiagnostic,
        AdminAcquisitionIntakeCandidateListResponse, AdminArtworkConfigDiagnostics,
        AdminAuthConfigDiagnostics, AdminCatalogGovernanceItem,
        AdminCatalogGovernanceItemListResponse, AdminConfigPlaybackDiagnostics,
        AdminConfigStagingDiagnostics, AdminDatabaseBackendCapabilitiesDiagnostics,
        AdminDatabaseConfigDiagnostics, AdminGeneratedArtifactProposal,
        AdminGeneratedArtifactProposalListResponse, AdminGeneratedArtifactReviewPlanResponse,
        AdminGeneratedArtifactReviewRequest, AdminGeneratedArtifactReviewResponse,
        AdminJobCancelRequestResponse, AdminJobListItem, AdminJobListResponse,
        AdminLibraryConfigDiagnostics, AdminMetadataConfigDiagnostics,
        AdminMetadataProviderConfigDiagnostics, AdminMetadataRuntimeConfigDiagnostics,
        AdminNetworkAccessDiagnostics, AdminNetworkExposureMode,
        AdminNetworkExternalEndpointDiagnostics, AdminNetworkReadinessCheck,
        AdminNetworkReadinessCheckName, AdminNetworkReadinessDiagnostics,
        AdminNetworkReadinessReason, AdminOriginPolicyDiagnostics, AdminOutboxEventListItem,
        AdminOutboxEventListResponse, AdminOverviewMetadataProviderSummary,
        AdminOverviewMetadataSummary, AdminOverviewResponse, AdminOverviewRuntimeSummary,
        AdminOverviewStartupSummary, AdminOverviewStatus, AdminOverviewStorageBackendSummary,
        AdminOverviewStorageSummary, AdminPlaybackFfmpegDiagnostics,
        AdminPlaybackHardwareCapability, AdminPlaybackHardwareCapabilityReason,
        AdminPlaybackHardwareDeviceInitialization, AdminPlaybackHardwareDeviceInitializationStatus,
        AdminPlaybackHardwareDiagnostics, AdminPlaybackHardwareEncoderDiscovery,
        AdminPlaybackHardwareEncoderDiscoveryStatus, AdminPlaybackHardwareSmokeProbe,
        AdminPlaybackHardwareSmokeProbeStatus, AdminPlaybackReadinessCheck,
        AdminPlaybackReadinessCheckName, AdminPlaybackReadinessDiagnostics,
        AdminPlaybackReadinessReason, AdminPlaybackRemoteBudgetDiagnostics,
        AdminPlaybackRemuxRuntimeDiagnostics, AdminPlaybackRuntimeDiagnosticsResponse,
        AdminPlaybackRuntimeStatus, AdminPlaybackSessionListItem, AdminPlaybackSessionListResponse,
        AdminPlaybackStagingDiagnostics, AdminPlaybackSupportEvidenceResponse,
        AdminPlaybackSupportHardwareCapabilityEvidence, AdminPlaybackSupportHardwareEvidence,
        AdminPlaybackSupportRedactionEvidence, AdminPlaybackSupportRuntimeEvidence,
        AdminPlaybackSupportSessionEvidence, AdminPlaybackSupportSourceEvidence,
        AdminPlaybackSupportSubject, AdminPlaybackTranscodeBudgetDiagnostics,
        AdminRuntimeConfigDiagnostics, AdminServerConfigDiagnosticsResponse,
        AdminStorageStagingDiagnosticsResponse, AdminStorageStagingRecord,
        AdminStorageStagingSummary, AdminTranscodeConfigDiagnostics, AdminTrustedProxyDiagnostics,
        AdminTunnelProviderDiagnostics, AdminTunnelProviderKind, AdminVfsCacheSummary,
        AdminWatchFolderDiscoveryFailure, AdminWatchFolderDiscoveryRequest,
        AdminWatchFolderDiscoveryResponse, StorageBackendDiagnosticsResponse, StorageBackendKind,
        StorageBackendRuntimeStateScope, StorageBackendStatus,
    },
    metadata_diagnostics::{MetadataProviderDiagnosticStatus, MetadataProviderDiagnosticsResponse},
    public_client::{API_VERSION, page_info_from_request},
};
use nako_core::{
    ArtworkCandidateId, AutomationArtifactId, ImageKind, JobId, ManagedArtworkArtifactId,
    ManagedArtworkIngestId, MediaItemId, NakoError, PageRequest,
};
use nako_db::DatabaseBackendCapabilities;
use nako_transcode::{
    HardwareAccelerationCapability, HardwareDeviceInitializationStatus,
    HardwareEncoderDiscoveryStatus, HardwareSmokeProbeStatus, hardware_acceleration_readiness,
};
use nako_vfs::StorageUri;
use serde::Deserialize;

use crate::{
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
            "/admin/v1/automation/generated-artifacts/{artifact_id}/review-plan",
            post(plan_admin_generated_artifact_review),
        )
        .route(
            "/admin/v1/automation/generated-artifacts/{artifact_id}/review",
            post(review_admin_generated_artifact),
        )
        .route(
            "/admin/v1/catalog/governance/items",
            get(list_admin_catalog_governance_items),
        )
        .route("/admin/v1/events", get(list_admin_outbox_events))
        .route("/admin/v1/jobs", get(list_admin_jobs))
        .route("/admin/v1/jobs/{job_id}/cancel", post(cancel_admin_job))
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
        .route("/admin/v1/storage/staging", get(list_admin_storage_staging))
        .route("/admin/v1/system/config", get(get_admin_system_config))
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

pub(super) async fn get_admin_overview(State(app): State<NakoApp>) -> Json<AdminOverviewResponse> {
    let storage = app.storage().list_storage_backend_diagnostics().await;
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

    Json(AdminOverviewResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        status,
        storage,
        metadata,
        runtime,
        startup,
    })
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
            hardware_policy: config.transcode.hardware_policy(),
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
    let vfs_cache = app
        .storage()
        .summarize_vfs_cache(crate::app::current_time_ms()?)
        .await?;

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
    let sessions = app.playback().list_transcode_sessions(filter, page).await?;
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
    let has_ffmpeg_probe_error = capabilities.iter().any(|capability| {
        matches!(
            capability.reason_code,
            AdminPlaybackHardwareCapabilityReason::ProbeError
        )
    });
    let available_gpu_capabilities = capabilities
        .iter()
        .filter(|capability| capability.accelerator.is_gpu() && capability.available)
        .count();
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
    let readiness = playback_readiness_diagnostics(
        has_ffmpeg_probe_error,
        hardware_acceleration_readiness(
            playback.hardware_policy,
            &playback.hardware_selection,
            &playback.hardware_report,
        ),
        playback.hardware_selection.fallback_used,
        playback.transcode_budget,
        transcode_budget,
        &remote_playback,
        &staging,
    );

    AdminPlaybackRuntimeDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        readiness,
        ffmpeg: AdminPlaybackFfmpegDiagnostics {
            probe_status: if has_ffmpeg_probe_error {
                AdminPlaybackRuntimeStatus::Degraded
            } else {
                AdminPlaybackRuntimeStatus::Ready
            },
            has_probe_error: has_ffmpeg_probe_error,
            hardware_capability_count: usize_to_u32(capabilities.len()),
            available_gpu_capabilities: usize_to_u32(available_gpu_capabilities),
        },
        hardware: AdminPlaybackHardwareDiagnostics {
            policy: playback.hardware_policy,
            selection: playback.hardware_selection,
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
        remote_playback,
        staging,
    }
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
        ffmpeg: runtime.ffmpeg,
        hardware: AdminPlaybackSupportHardwareEvidence {
            policy: runtime.hardware.policy,
            selected_acceleration: runtime.hardware.selection.acceleration,
            fallback_used: runtime.hardware.selection.fallback_used,
            capability_count: usize_to_u32(runtime.hardware.capabilities.len()),
            unavailable_capabilities,
        },
        transcode: runtime.transcode,
        remux: runtime.remux,
        remote_playback: runtime.remote_playback,
        staging: runtime.staging,
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
        accelerator: capability.accelerator,
        available: capability.available,
        reason_code: hardware_capability_reason(capability),
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
    hardware_readiness: nako_transcode::HardwareAccelerationReadiness,
    fallback_used: bool,
    configured_budget: nako_transcode::TranscodeResourceBudget,
    effective_budget: nako_transcode::TranscodeResourceBudget,
    remote_playback: &AdminPlaybackRemoteBudgetDiagnostics,
    staging: &AdminPlaybackStagingDiagnostics,
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
        if fallback_used {
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
    ])
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
