use axum::Json;
use taru_api::{API_VERSION, HealthResponse};

pub(super) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_owned(),
        version: API_VERSION.to_owned(),
    })
}
