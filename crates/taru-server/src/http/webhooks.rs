use axum::{
    Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, post},
};
use taru_api::UpsertWebhookEndpointRequest;
use taru_core::{EventId, WebhookEndpointId};
use tracing::instrument;

use crate::app::TaruApp;

use super::error::ApiResult;

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route(
            "/webhooks/endpoints",
            get(list_webhook_endpoints).post(upsert_webhook_endpoint),
        )
        .route(
            "/webhooks/endpoints/{endpoint_id}",
            get(get_webhook_endpoint),
        )
        .route(
            "/events/{event_id}/webhook-attempts",
            get(list_webhook_delivery_attempts),
        )
        .route(
            "/events/{event_id}/webhooks/deliver",
            post(deliver_webhooks_for_event),
        )
}

#[instrument(skip(app))]
pub(super) async fn upsert_webhook_endpoint(
    State(app): State<TaruApp>,
    Json(request): Json<UpsertWebhookEndpointRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.webhooks().upsert_webhook_endpoint(request).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_webhook_endpoints(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.webhooks().list_enabled_webhook_endpoints().await?))
}

#[instrument(skip(app))]
pub(super) async fn get_webhook_endpoint(
    State(app): State<TaruApp>,
    Path(endpoint_id): Path<WebhookEndpointId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.webhooks().get_webhook_endpoint(endpoint_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_webhook_delivery_attempts(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.webhooks()
            .list_webhook_delivery_attempts(event_id)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn deliver_webhooks_for_event(
    State(app): State<TaruApp>,
    Path(event_id): Path<EventId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.webhooks().deliver_webhooks_for_event(event_id).await?,
    ))
}
