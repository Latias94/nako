use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, patch, post},
};
use nako_api::extension::{
    AddonAccessCheckRequest, AdminAddonInstallGuidePreviewRequest,
    AdminAddonResourceCallDiagnosticRequest, IssueAddonTokenRequest, RegisterAddonRequest,
    ReplaceAddonGrantsRequest, SubmitAddonAcquisitionCandidateRequest,
    SubmitAddonGeneratedArtifactRequest, SubmitAddonSideEffectRequest, UpdateAddonStatusRequest,
};
use nako_core::{AddonId, AddonTokenId};
use tracing::instrument;

use crate::app::NakoApp;

use super::{auth, error::ApiResult, query::AddonListQuery};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/admin/v1/addons", get(list_addons).post(register_addon))
        .route(
            "/admin/v1/addons/install-guide-preview",
            post(preview_addon_install_guide),
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
            "/admin/v1/addons/{addon_id}/install-guide",
            get(get_addon_install_guide),
        )
        .route(
            "/admin/v1/addons/{addon_id}/diagnostics/resource-call",
            post(diagnose_addon_resource_call),
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
        .route("/addon/v1/access-check", post(check_addon_access))
        .route(
            "/addon/v1/generated-artifacts",
            post(submit_addon_generated_artifact),
        )
        .route(
            "/addon/v1/acquisition/intake/candidates",
            post(submit_addon_acquisition_candidate),
        )
        .route("/addon/v1/side-effects", post(submit_addon_side_effect))
}

#[instrument(skip(app))]
pub(super) async fn preview_addon_install_guide(
    State(app): State<NakoApp>,
    Json(request): Json<AdminAddonInstallGuidePreviewRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().preview_addon_install_guide(request)?))
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
pub(super) async fn get_addon_install_guide(
    State(app): State<NakoApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_install_guide(addon_id).await?))
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
