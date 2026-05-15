use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use taru_api::UpsertWebhookEndpointRequest;
use taru_core::{EventId, WebhookEndpointId};
use tracing::instrument;

use crate::app::TaruApp;

use super::error::ApiResult;

#[instrument(skip(app))]
pub(super) async fn upsert_webhook_endpoint(
    State(app): State<TaruApp>,
    Json(request): Json<UpsertWebhookEndpointRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.upsert_webhook_endpoint(request).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_webhook_endpoints(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_enabled_webhook_endpoints().await?))
}

#[instrument(skip(app))]
pub(super) async fn get_webhook_endpoint(
    State(app): State<TaruApp>,
    Path(endpoint_id): Path<WebhookEndpointId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_webhook_endpoint(endpoint_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_webhook_delivery_attempts(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_webhook_delivery_attempts(event_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn deliver_webhooks_for_event(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.deliver_webhooks_for_event(event_id).await?))
}
