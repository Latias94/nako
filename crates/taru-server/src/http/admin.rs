use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
};
use taru_api::{
    ADMIN_API_VERSION, API_VERSION, AdminJobListItem, AdminJobListResponse,
    AdminOverviewMetadataProviderSummary, AdminOverviewMetadataSummary, AdminOverviewResponse,
    AdminOverviewRuntimeSummary, AdminOverviewStartupSummary, AdminOverviewStatus,
    AdminOverviewStorageBackendSummary, AdminOverviewStorageSummary, AdminPlaybackSessionListItem,
    AdminPlaybackSessionListResponse, MetadataProviderDiagnosticStatus, StorageBackendStatus,
    page_info_from_request,
};

use crate::app::{RuntimeSupervisorDiagnostics, TaruApp};

use super::{
    error::ApiResult,
    query::{JobListQuery, PlaybackSessionListQuery},
};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route("/admin/v1/overview", get(get_admin_overview))
        .route("/admin/v1/jobs", get(list_admin_jobs))
        .route(
            "/admin/v1/playback/sessions",
            get(list_admin_playback_sessions),
        )
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

fn storage_summary(
    diagnostics: taru_api::StorageBackendDiagnosticsResponse,
) -> AdminOverviewStorageSummary {
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
    diagnostics: taru_api::MetadataProviderDiagnosticsResponse,
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

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}
