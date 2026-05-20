use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde::Deserialize;
use taru_api::{
    admin::{
        ADMIN_API_VERSION, AdminArtworkConfigDiagnostics, AdminAuthConfigDiagnostics,
        AdminCatalogGovernanceItem, AdminCatalogGovernanceItemListResponse,
        AdminConfigPlaybackDiagnostics, AdminConfigStagingDiagnostics,
        AdminDatabaseBackendCapabilitiesDiagnostics, AdminDatabaseConfigDiagnostics,
        AdminJobCancelRequestResponse, AdminJobListItem, AdminJobListResponse,
        AdminLibraryConfigDiagnostics, AdminMetadataConfigDiagnostics,
        AdminMetadataProviderConfigDiagnostics, AdminMetadataRuntimeConfigDiagnostics,
        AdminOutboxEventListItem, AdminOutboxEventListResponse,
        AdminOverviewMetadataProviderSummary, AdminOverviewMetadataSummary, AdminOverviewResponse,
        AdminOverviewRuntimeSummary, AdminOverviewStartupSummary, AdminOverviewStatus,
        AdminOverviewStorageBackendSummary, AdminOverviewStorageSummary,
        AdminPlaybackFfmpegDiagnostics, AdminPlaybackHardwareCapability,
        AdminPlaybackHardwareCapabilityEvidence, AdminPlaybackHardwareCapabilityReason,
        AdminPlaybackHardwareDiagnostics, AdminPlaybackHardwareSmokeProbe,
        AdminPlaybackHardwareSmokeProbeStatus, AdminPlaybackRemoteBudgetDiagnostics,
        AdminPlaybackRemuxRuntimeDiagnostics, AdminPlaybackRuntimeDiagnosticsResponse,
        AdminPlaybackRuntimeStatus, AdminPlaybackSessionListItem, AdminPlaybackSessionListResponse,
        AdminPlaybackStagingDiagnostics, AdminPlaybackTranscodeBudgetDiagnostics,
        AdminRuntimeConfigDiagnostics, AdminServerConfigDiagnosticsResponse,
        AdminStorageStagingDiagnosticsResponse, AdminStorageStagingRecord,
        AdminStorageStagingSummary, AdminTranscodeConfigDiagnostics, AdminVfsCacheSummary,
        StorageBackendDiagnosticsResponse, StorageBackendKind, StorageBackendRuntimeStateScope,
        StorageBackendStatus,
    },
    metadata_diagnostics::{MetadataProviderDiagnosticStatus, MetadataProviderDiagnosticsResponse},
    public_client::{API_VERSION, page_info_from_request},
};
use taru_core::{
    ArtworkCandidateId, ImageKind, JobId, ManagedArtworkArtifactId, ManagedArtworkIngestId,
    MediaItemId, TaruError,
};
use taru_db::{DatabaseBackendCapabilities, TaruDatabase};
use taru_transcode::{
    HardwareAccelerationCapability, HardwareCapabilityEvidence, HardwareSmokeProbeStatus,
};

use crate::{
    app::{RuntimeSupervisorDiagnostics, TaruApp},
    config::{LocalLibraryConfig, MetadataProviderConfig, MetadataProviderRuntimeConfig},
};

use super::{
    error::ApiResult,
    query::{
        ArtworkArtifactLifecycleQuery, ArtworkArtifactRemediationQuery,
        ArtworkArtifactStorageDriftQuery, ArtworkGalleryQuery, CatalogGovernanceItemsQuery,
        JobListQuery, OutboxEventListQuery, PlaybackSessionListQuery, StorageStagingQuery,
    },
};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route("/admin/v1/overview", get(get_admin_overview))
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
            "/admin/v1/playback/sessions",
            get(list_admin_playback_sessions),
        )
}

pub(super) async fn accept_admin_artwork_candidate(
    State(app): State<TaruApp>,
    Path(candidate_id): Path<ArtworkCandidateId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().accept_candidate(candidate_id).await?))
}

pub(super) async fn process_next_admin_artwork_ingest(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().process_next().await?))
}

pub(super) async fn requeue_admin_artwork_ingest(
    State(app): State<TaruApp>,
    Path(ingest_id): Path<ManagedArtworkIngestId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().requeue_ingest(ingest_id).await?))
}

pub(super) async fn publish_admin_artwork_artifact(
    State(app): State<TaruApp>,
    Path(artifact_id): Path<ManagedArtworkArtifactId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.artwork().publish_artifact(artifact_id).await?))
}

pub(super) async fn get_admin_item_artwork_gallery(
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
    Path((item_id, kind)): Path<(MediaItemId, String)>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.artwork()
            .unpublish_item_artwork(item_id, parse_admin_artwork_kind(&kind)?)
            .await?,
    ))
}

pub(super) async fn get_admin_artwork_artifact_lifecycle(
    State(app): State<TaruApp>,
    Query(query): Query<ArtworkArtifactLifecycleQuery>,
) -> ApiResult<impl IntoResponse> {
    let (filter, page) = query.into_filter_and_page()?;
    Ok(Json(
        app.artwork()
            .artifact_lifecycle_diagnostics(filter, page)
            .await?,
    ))
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct SelectAdminItemArtworkRequest {
    pub(super) artifact_id: ManagedArtworkArtifactId,
}

fn parse_admin_artwork_kind(value: &str) -> Result<ImageKind, TaruError> {
    match value {
        "poster" => Ok(ImageKind::Poster),
        "backdrop" => Ok(ImageKind::Backdrop),
        "logo" => Ok(ImageKind::Logo),
        "thumbnail" => Ok(ImageKind::Thumbnail),
        "banner" => Ok(ImageKind::Banner),
        _ => Err(TaruError::InvalidInput {
            message: format!("unsupported artwork kind path segment: {value}"),
        }),
    }
}

pub(super) async fn get_admin_artwork_artifact_storage_drift(
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
    Query(query): Query<ArtworkArtifactLifecycleQuery>,
) -> ApiResult<impl IntoResponse> {
    let (_filter, page) = query.into_filter_and_page()?;
    Ok(Json(
        app.artwork().cleanup_unselected_artifacts(page).await?,
    ))
}

pub(super) async fn list_admin_catalog_governance_items(
    State(app): State<TaruApp>,
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

pub(super) async fn get_admin_overview(State(app): State<TaruApp>) -> Json<AdminOverviewResponse> {
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
    State(app): State<TaruApp>,
) -> Json<AdminServerConfigDiagnosticsResponse> {
    let config = app.config();

    Json(AdminServerConfigDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        auth: AdminAuthConfigDiagnostics {
            enabled: config.auth.enabled,
            token_env: config.auth.token_env.clone(),
        },
        database: database_config_diagnostics(
            config,
            app.store(),
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
    State(app): State<TaruApp>,
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

fn database_config_diagnostics(
    config: &crate::config::TaruServerConfig,
    store: &TaruDatabase,
    migrated_on_startup: bool,
) -> AdminDatabaseConfigDiagnostics {
    let configured_backend = config.database_backend;
    let active_backend = store.backend_kind();

    AdminDatabaseConfigDiagnostics {
        configured_backend_kind: configured_backend.as_str().to_owned(),
        active_backend_kind: active_backend.as_str().to_owned(),
        url_scheme: database_url_scheme(&config.database_url),
        runtime_supported: configured_backend == active_backend,
        migrated_on_startup,
        capabilities: database_backend_capabilities_diagnostics(store.capabilities()),
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
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
    Path(job_id): Path<JobId>,
) -> ApiResult<impl IntoResponse> {
    let cancellation = app.jobs().request_job_cancellation(job_id).await?;

    Ok(Json(AdminJobCancelRequestResponse::from_record(
        cancellation,
    )))
}

pub(super) async fn list_admin_playback_sessions(
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
) -> Json<AdminPlaybackRuntimeDiagnosticsResponse> {
    let playback = app.playback().runtime_diagnostics();
    let storage = app.storage().list_storage_backend_diagnostics().await;
    let startup = app.startup_report().clone();

    let capabilities = playback
        .hardware_report
        .capabilities
        .iter()
        .map(hardware_capability_diagnostic)
        .collect::<Vec<_>>();
    let has_probe_error = capabilities.iter().any(|capability| {
        capability.reason_code == AdminPlaybackHardwareCapabilityReason::ProbeError
    });
    let available_gpu_capabilities = capabilities
        .iter()
        .filter(|capability| capability.accelerator.is_gpu() && capability.available)
        .count();
    let transcode_budget = playback.transcode_budget.bounded();

    Json(AdminPlaybackRuntimeDiagnosticsResponse {
        admin_api_version: ADMIN_API_VERSION.to_owned(),
        public_api_version: API_VERSION.to_owned(),
        ffmpeg: AdminPlaybackFfmpegDiagnostics {
            probe_status: if has_probe_error {
                AdminPlaybackRuntimeStatus::Degraded
            } else {
                AdminPlaybackRuntimeStatus::Ready
            },
            has_probe_error,
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
        remote_playback: remote_budget_summary(
            storage,
            playback.remote_stream_concurrency,
            playback.remote_stage_concurrency,
        ),
        staging: AdminPlaybackStagingDiagnostics {
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
        },
    })
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
        evidence: hardware_capability_evidence(capability.evidence),
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
    match capability.evidence {
        HardwareCapabilityEvidence::CpuAlwaysAvailable
        | HardwareCapabilityEvidence::FfmpegEncoderListed
        | HardwareCapabilityEvidence::StaticDetector
            if capability.available =>
        {
            AdminPlaybackHardwareCapabilityReason::Available
        }
        HardwareCapabilityEvidence::FfmpegEncoderMissing => {
            AdminPlaybackHardwareCapabilityReason::EncoderNotListed
        }
        HardwareCapabilityEvidence::FfmpegProbeError
        | HardwareCapabilityEvidence::CpuAlwaysAvailable
        | HardwareCapabilityEvidence::FfmpegEncoderListed
        | HardwareCapabilityEvidence::StaticDetector => {
            AdminPlaybackHardwareCapabilityReason::ProbeError
        }
    }
}

fn hardware_capability_evidence(
    evidence: HardwareCapabilityEvidence,
) -> AdminPlaybackHardwareCapabilityEvidence {
    match evidence {
        HardwareCapabilityEvidence::CpuAlwaysAvailable => {
            AdminPlaybackHardwareCapabilityEvidence::CpuAlwaysAvailable
        }
        HardwareCapabilityEvidence::FfmpegEncoderListed => {
            AdminPlaybackHardwareCapabilityEvidence::FfmpegEncoderListed
        }
        HardwareCapabilityEvidence::FfmpegEncoderMissing => {
            AdminPlaybackHardwareCapabilityEvidence::FfmpegEncoderMissing
        }
        HardwareCapabilityEvidence::FfmpegProbeError => {
            AdminPlaybackHardwareCapabilityEvidence::FfmpegProbeError
        }
        HardwareCapabilityEvidence::StaticDetector => {
            AdminPlaybackHardwareCapabilityEvidence::StaticDetector
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

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
