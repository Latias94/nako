use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, patch, post},
};
use nako_addon_protocol::{
    ADDON_RUNTIME_ACCESS_CHECK_PATH, ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH,
    ADDON_RUNTIME_GENERATED_ARTIFACTS_PATH, ADDON_RUNTIME_SIDE_EFFECTS_PATH,
    ADDON_RUNTIME_TASK_RUN_CANCEL_PATH, ADDON_RUNTIME_TASK_RUN_CLAIM_PATH,
    ADDON_RUNTIME_TASK_RUN_COMPLETE_PATH, ADDON_RUNTIME_TASK_RUN_FAIL_PATH,
    ADDON_RUNTIME_TASK_RUN_PROGRESS_PATH,
};
use nako_api::extension::{
    AddonAccessCheckRequest, AdminAddonInstallGuidePreviewRequest, AdminAddonManagerPlanRequest,
    AdminAddonResourceCallDiagnosticRequest, AdminAddonResourceLinkCheckRequest,
    AdminAddonResourceSearchDiagnosticRequest, AdminAddonResourceSearchRequest,
    AdminAddonResourceSearchSelectionRequest, CancelAddonTaskRunRequest, ClaimAddonTaskRunRequest,
    CompleteAddonTaskRunRequest, CreateAddonTaskRunRequest, FailAddonTaskRunRequest,
    IssueAddonTokenRequest, RegisterAddonRequest, ReplaceAddonGrantsRequest,
    ReplayAddonEventRequest, ReportAddonTaskRunProgressRequest, RetryAddonTaskRunRequest,
    SubmitAddonAcquisitionCandidateRequest, SubmitAddonGeneratedArtifactRequest,
    SubmitAddonSideEffectRequest, UpdateAddonStatusRequest,
};
use nako_core::{AddonId, AddonTokenId, EventId, JobId};
use tracing::instrument;

use crate::app::NakoApp;

use super::{
    auth,
    error::ApiResult,
    query::{AddonListQuery, PageQuery},
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/admin/v1/addons", get(list_addons).post(register_addon))
        .route(
            "/admin/v1/addons/install-guide-preview",
            post(preview_addon_install_guide),
        )
        .route(
            "/admin/v1/addons/catalog/sources",
            get(list_addon_source_catalog_sources),
        )
        .route(
            "/admin/v1/addons/catalog/entries",
            get(list_addon_source_catalog_entries),
        )
        .route(
            "/admin/v1/addons/catalog/entries/{entry_id}/resolve",
            get(resolve_addon_source_catalog_entry),
        )
        .route("/admin/v1/addons/{addon_id}", get(get_addon))
        .route(
            "/admin/v1/addons/{addon_id}/status",
            patch(update_addon_status),
        )
        .route(
            "/admin/v1/addons/{addon_id}/unregister",
            post(unregister_addon),
        )
        .route(
            "/admin/v1/addons/{addon_id}/health-check",
            post(check_addon_health),
        )
        .route(
            "/admin/v1/addons/{addon_id}/runtime-readiness",
            post(check_addon_runtime_readiness),
        )
        .route(
            "/admin/v1/addons/{addon_id}/surfaces",
            get(get_addon_surfaces),
        )
        .route(
            "/admin/v1/addons/{addon_id}/routing-plans",
            post(sync_addon_routing_plans),
        )
        .route(
            "/admin/v1/events/{event_id}/addon-event-attempts",
            get(list_addon_event_delivery_attempts),
        )
        .route(
            "/admin/v1/events/{event_id}/addon-event-scheduler/work",
            get(list_addon_event_scheduler_work),
        )
        .route(
            "/admin/v1/events/{event_id}/addon-events/deliver",
            post(deliver_addon_events_for_event),
        )
        .route(
            "/admin/v1/events/{event_id}/addon-events/replay",
            post(replay_addon_events_for_event),
        )
        .route(
            "/admin/v1/addons/{addon_id}/task-runs",
            get(list_addon_task_runs).post(create_addon_task_run),
        )
        .route(
            "/admin/v1/addons/{addon_id}/task-runs/{job_id}",
            get(get_addon_task_run),
        )
        .route(
            "/admin/v1/addons/{addon_id}/task-runs/{job_id}/retry",
            post(retry_addon_task_run),
        )
        .route(
            "/admin/v1/addons/{addon_id}/install-guide",
            get(get_addon_install_guide),
        )
        .route(
            "/admin/v1/addons/{addon_id}/manager-plan",
            get(get_addon_manager_plan).post(plan_addon_manager_lifecycle),
        )
        .route(
            "/admin/v1/addons/{addon_id}/diagnostics/resource-call",
            post(diagnose_addon_resource_call),
        )
        .route(
            "/admin/v1/addons/{addon_id}/diagnostics/resource-search",
            post(diagnose_addon_resource_search),
        )
        .route(
            "/admin/v1/addons/{addon_id}/resource-search",
            post(search_addon_resources),
        )
        .route(
            "/admin/v1/addons/{addon_id}/resource-search/{search_id}/selections/{selection_id}/intake-candidate",
            post(select_addon_resource_search_result),
        )
        .route(
            "/admin/v1/addons/{addon_id}/resource-search/{search_id}/selections/{selection_id}/link-check",
            post(check_addon_resource_search_selection_link),
        )
        .route(
            "/admin/v1/addons/{addon_id}/tokens",
            get(list_addon_tokens).post(issue_addon_token),
        )
        .route(
            "/admin/v1/addons/{addon_id}/tokens/{token_id}/rotate",
            post(rotate_addon_token),
        )
        .route(
            "/admin/v1/addons/{addon_id}/tokens/{token_id}/revoke",
            post(revoke_addon_token),
        )
        .route(
            "/admin/v1/addons/{addon_id}/grants",
            get(list_addon_grants).put(replace_addon_grants),
        )
}

pub(super) fn runtime_routes() -> Router<NakoApp> {
    Router::new()
        .route(ADDON_RUNTIME_ACCESS_CHECK_PATH, post(check_addon_access))
        .route(
            ADDON_RUNTIME_GENERATED_ARTIFACTS_PATH,
            post(submit_addon_generated_artifact),
        )
        .route(
            ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH,
            post(submit_addon_acquisition_candidate),
        )
        .route(
            ADDON_RUNTIME_SIDE_EFFECTS_PATH,
            post(submit_addon_side_effect),
        )
        .route(
            ADDON_RUNTIME_TASK_RUN_CLAIM_PATH,
            post(claim_addon_task_run),
        )
        .route(
            ADDON_RUNTIME_TASK_RUN_PROGRESS_PATH,
            post(report_addon_task_run_progress),
        )
        .route(
            ADDON_RUNTIME_TASK_RUN_COMPLETE_PATH,
            post(complete_addon_task_run),
        )
        .route(ADDON_RUNTIME_TASK_RUN_FAIL_PATH, post(fail_addon_task_run))
        .route(
            ADDON_RUNTIME_TASK_RUN_CANCEL_PATH,
            post(cancel_addon_task_run),
        )
}

#[instrument(skip(app))]
pub(super) async fn preview_addon_install_guide(
    State(app): State<NakoApp>,
    Json(request): Json<AdminAddonInstallGuidePreviewRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().preview_addon_install_guide(request)?))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_source_catalog_sources(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_source_catalog_sources()?))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_source_catalog_entries(
    State(app): State<NakoApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_source_catalog_entries()?))
}

#[instrument(skip(app))]
pub(super) async fn resolve_addon_source_catalog_entry(
    State(app): State<NakoApp>,
    Path(entry_id): Path<String>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().resolve_addon_source_catalog_entry(&entry_id)?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn register_addon(
    State(app): State<NakoApp>,
    Json(request): Json<RegisterAddonRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().register_addon(request).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_addons(
    State(app): State<NakoApp>,
    Query(query): Query<AddonListQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().list_addon_registrations(query.status).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_addon(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_registration(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn update_addon_status(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<UpdateAddonStatusRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().update_addon_status(addon_id, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn unregister_addon(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().unregister_addon(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn check_addon_health(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().check_addon_health(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn check_addon_runtime_readiness(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().check_addon_runtime_readiness(addon_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_addon_surfaces(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_surfaces(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn sync_addon_routing_plans(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().sync_addon_routing_plans(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_event_delivery_attempts(
    State(app): State<NakoApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .list_addon_event_delivery_attempts(event_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_event_scheduler_work(
    State(app): State<NakoApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .list_addon_event_scheduler_work(event_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn deliver_addon_events_for_event(
    State(app): State<NakoApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .deliver_addon_events_for_event(event_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn replay_addon_events_for_event(
    State(app): State<NakoApp>,
    Path(event_id): Path<EventId>,
    Json(request): Json<ReplayAddonEventRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .replay_addon_events_for_event(event_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn create_addon_task_run(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<CreateAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .create_addon_task_run(addon_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_task_runs(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .list_addon_task_runs(addon_id, None, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_addon_task_run(
    State(app): State<NakoApp>,
    Path((addon_id, job_id)): Path<(AddonId, JobId)>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().get_addon_task_run(addon_id, job_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn retry_addon_task_run(
    State(app): State<NakoApp>,
    Path((addon_id, job_id)): Path<(AddonId, JobId)>,
    Json(request): Json<RetryAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .retry_addon_task_run(addon_id, job_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_addon_install_guide(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_install_guide(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_addon_manager_plan(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_manager_plan(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn plan_addon_manager_lifecycle(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<AdminAddonManagerPlanRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .plan_addon_manager_lifecycle(addon_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn diagnose_addon_resource_call(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<AdminAddonResourceCallDiagnosticRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .diagnose_addon_resource_call(addon_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn diagnose_addon_resource_search(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<AdminAddonResourceSearchDiagnosticRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .diagnose_addon_resource_search(addon_id, request)
            .await?,
    ))
}

#[instrument(skip(app, request))]
pub(super) async fn search_addon_resources(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<AdminAddonResourceSearchRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .search_addon_resources(addon_id, request)
            .await?,
    ))
}

#[instrument(skip(app, request))]
pub(super) async fn select_addon_resource_search_result(
    State(app): State<NakoApp>,
    Path((addon_id, search_id, selection_id)): Path<(AddonId, String, String)>,
    Json(request): Json<AdminAddonResourceSearchSelectionRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .select_addon_resource_search_result(addon_id, search_id, selection_id, request)
            .await?,
    ))
}

#[instrument(skip(app, request))]
pub(super) async fn check_addon_resource_search_selection_link(
    State(app): State<NakoApp>,
    Path((addon_id, search_id, selection_id)): Path<(AddonId, String, String)>,
    Json(request): Json<AdminAddonResourceLinkCheckRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .check_addon_resource_search_selection_link(addon_id, search_id, selection_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn issue_addon_token(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<IssueAddonTokenRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().issue_addon_token(addon_id, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_tokens(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_tokens(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn rotate_addon_token(
    State(app): State<NakoApp>,
    Path((addon_id, token_id)): Path<(AddonId, AddonTokenId)>,
    Json(request): Json<IssueAddonTokenRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons()
            .rotate_addon_token(addon_id, token_id, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn revoke_addon_token(
    State(app): State<NakoApp>,
    Path((addon_id, token_id)): Path<(AddonId, AddonTokenId)>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().revoke_addon_token(addon_id, token_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn replace_addon_grants(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<ReplaceAddonGrantsRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().replace_addon_grants(addon_id, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_grants(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_grants(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn check_addon_access(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<AddonAccessCheckRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons().check_addon_access(raw_token, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn claim_addon_task_run(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<ClaimAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .claim_addon_task_run(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn report_addon_task_run_progress(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<ReportAddonTaskRunProgressRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .report_addon_task_run_progress(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn complete_addon_task_run(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<CompleteAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .complete_addon_task_run(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn fail_addon_task_run(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<FailAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons().fail_addon_task_run(raw_token, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn cancel_addon_task_run(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<CancelAddonTaskRunRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .cancel_addon_task_run(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn submit_addon_side_effect(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<SubmitAddonSideEffectRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .submit_addon_side_effect(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn submit_addon_generated_artifact(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<SubmitAddonGeneratedArtifactRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .submit_addon_generated_artifact(raw_token, request)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn submit_addon_acquisition_candidate(
    State(app): State<NakoApp>,
    headers: HeaderMap,
    Json(request): Json<SubmitAddonAcquisitionCandidateRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| nako_core::NakoError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .submit_addon_acquisition_candidate(raw_token, request)
            .await?,
    ))
}
