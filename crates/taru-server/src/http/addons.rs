use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use taru_api::RegisterAddonRequest;
use taru_core::AddonId;
use tracing::instrument;

use crate::app::TaruApp;

use super::{error::ApiResult, query::AddonListQuery};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route("/addons", get(list_addons).post(register_addon))
        .route("/addons/{addon_id}", get(get_addon))
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
