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
    AdminOverviewStorageBackendSummary, AdminOverviewStorageSummary,
    AdminPlaybackFfmpegDiagnostics, AdminPlaybackHardwareCapability,
    AdminPlaybackHardwareCapabilityReason, AdminPlaybackHardwareDiagnostics,
    AdminPlaybackRemoteBudgetDiagnostics, AdminPlaybackRemuxRuntimeDiagnostics,
    AdminPlaybackRuntimeDiagnosticsResponse, AdminPlaybackRuntimeStatus,
    AdminPlaybackSessionListItem, AdminPlaybackSessionListResponse,
    AdminPlaybackStagingDiagnostics, AdminPlaybackTranscodeBudgetDiagnostics,
    MetadataProviderDiagnosticStatus, StorageBackendRuntimeStateScope, StorageBackendStatus,
    page_info_from_request,
};
use taru_transcode::HardwareAccelerationCapability;

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
            "/admin/v1/playback/runtime",
            get(get_admin_playback_runtime),
        )
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

fn hardware_capability_diagnostic(
    capability: &HardwareAccelerationCapability,
) -> AdminPlaybackHardwareCapability {
    AdminPlaybackHardwareCapability {
        accelerator: capability.accelerator,
        available: capability.available,
        reason_code: hardware_capability_reason(capability),
    }
}

fn hardware_capability_reason(
    capability: &HardwareAccelerationCapability,
) -> AdminPlaybackHardwareCapabilityReason {
    if capability.available {
        return AdminPlaybackHardwareCapabilityReason::Available;
    }

    if capability
        .reason
        .as_deref()
        .is_some_and(|reason| reason.contains("not listed"))
    {
        AdminPlaybackHardwareCapabilityReason::EncoderNotListed
    } else {
        AdminPlaybackHardwareCapabilityReason::ProbeError
    }
}

fn remote_budget_summary(
    diagnostics: taru_api::StorageBackendDiagnosticsResponse,
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
