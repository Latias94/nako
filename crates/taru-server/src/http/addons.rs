use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use taru_api::{
    AddonAccessCheckRequest, IssueAddonTokenRequest, RegisterAddonRequest,
    ReplaceAddonGrantsRequest, SubmitAddonSideEffectRequest,
};
use taru_core::{AddonId, AddonTokenId};
use tracing::instrument;

use crate::app::TaruApp;

use super::{auth, error::ApiResult, query::AddonListQuery};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route("/addons", get(list_addons).post(register_addon))
        .route("/addons/{addon_id}", get(get_addon))
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

pub(super) fn runtime_routes() -> Router<TaruApp> {
    Router::new()
        .route("/addon/v1/access-check", post(check_addon_access))
        .route("/addon/v1/side-effects", post(submit_addon_side_effect))
}

#[instrument(skip(app))]
pub(super) async fn register_addon(
    State(app): State<TaruApp>,
    Json(request): Json<RegisterAddonRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().register_addon(request).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_addons(
    State(app): State<TaruApp>,
    Query(query): Query<AddonListQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().list_addon_registrations(query.status).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_addon(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().get_addon_registration(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn issue_addon_token(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<IssueAddonTokenRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().issue_addon_token(addon_id, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_tokens(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_tokens(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn rotate_addon_token(
    State(app): State<TaruApp>,
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
    State(app): State<TaruApp>,
    Path((addon_id, token_id)): Path<(AddonId, AddonTokenId)>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().revoke_addon_token(addon_id, token_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn replace_addon_grants(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
    Json(request): Json<ReplaceAddonGrantsRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.addons().replace_addon_grants(addon_id, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_addon_grants(
    State(app): State<TaruApp>,
    Path(addon_id): Path<AddonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.addons().list_addon_grants(addon_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn check_addon_access(
    State(app): State<TaruApp>,
    headers: HeaderMap,
    Json(request): Json<AddonAccessCheckRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| taru_core::TaruError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons().check_addon_access(raw_token, request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn submit_addon_side_effect(
    State(app): State<TaruApp>,
    headers: HeaderMap,
    Json(request): Json<SubmitAddonSideEffectRequest>,
) -> ApiResult<impl IntoResponse> {
    let raw_token =
        auth::request_bearer_token(&headers).ok_or_else(|| taru_core::TaruError::Unauthorized {
            message: "addon token is required".to_owned(),
        })?;

    Ok(Json(
        app.addons()
            .submit_addon_side_effect(raw_token, request)
            .await?,
    ))
}
