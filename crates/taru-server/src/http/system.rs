use axum::{Json, Router, extract::State, routing::get};
use taru_api::public_client::{API_VERSION, HealthResponse};

use crate::app::TaruApp;

use super::error::ApiResult;

pub(super) fn routes() -> Router<TaruApp> {
    Router::new().route("/storage/backends", get(list_storage_backends))
}

pub(super) fn public_routes() -> Router<TaruApp> {
    Router::new().route("/health", get(health))
}

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}

pub(super) async fn list_storage_backends(
    State(app): State<TaruApp>,
) -> ApiResult<Json<taru_api::admin::StorageBackendDiagnosticsResponse>> {
    Ok(Json(app.storage().list_storage_backend_diagnostics().await))
}
