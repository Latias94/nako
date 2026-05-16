use axum::{Json, extract::State};
use taru_api::{API_VERSION, HealthResponse};

use crate::app::TaruApp;

use super::error::ApiResult;

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}

pub(super) async fn list_storage_backends(
    State(app): State<TaruApp>,
) -> ApiResult<Json<taru_api::StorageBackendDiagnosticsResponse>> {
    Ok(Json(app.list_storage_backend_diagnostics().await))
}
