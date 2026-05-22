use axum::{Json, Router, extract::State, routing::get};
use nako_api::public_client::{API_VERSION, HealthResponse};

use crate::app::NakoApp;

use super::error::ApiResult;

pub(super) fn routes() -> Router<NakoApp> {
    Router::new().route("/storage/backends", get(list_storage_backends))
}

pub(super) fn public_routes() -> Router<NakoApp> {
    Router::new().route("/health", get(health))
}

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}

pub(super) async fn list_storage_backends(
    State(app): State<NakoApp>,
) -> ApiResult<Json<nako_api::admin::StorageBackendDiagnosticsResponse>> {
    Ok(Json(app.storage().list_storage_backend_diagnostics().await))
}
