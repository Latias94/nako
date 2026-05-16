use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use taru_api::{EnqueueMetadataMaintenanceRequest, JobResponse};
use taru_core::{
    ExternalProvider, MediaItemId, MetadataAttemptFilter, MetadataProviderAttemptStatus,
    ProviderRawResponseFilter,
};
use tracing::instrument;

use crate::app::TaruApp;

use super::{error::ApiResult, query::PageQuery};

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct MetadataAttemptsQuery {
    pub(super) provider: Option<ExternalProvider>,
    pub(super) status: Option<MetadataProviderAttemptStatus>,
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct MetadataRawResponsesQuery {
    pub(super) provider: Option<ExternalProvider>,
    #[serde(flatten)]
    pub(super) page: PageQuery,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(super) struct MetadataRawCleanupQuery {
    pub(super) provider: Option<ExternalProvider>,
    pub(super) fetched_before: Option<String>,
    pub(super) retention_ms: Option<u64>,
}

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route(
            "/items/{item_id}/metadata/refresh",
            post(refresh_item_metadata),
        )
        .route(
            "/items/{item_id}/metadata/attempts",
            get(list_item_metadata_attempts),
        )
        .route(
            "/items/{item_id}/metadata/raw",
            get(list_item_metadata_raw_responses),
        )
        .route("/metadata/providers", get(list_metadata_providers))
        .route(
            "/metadata/maintenance/jobs",
            post(enqueue_metadata_maintenance),
        )
        .route(
            "/metadata/maintenance/plan",
            post(plan_metadata_maintenance),
        )
        .route(
            "/metadata/raw/cleanup",
            post(cleanup_metadata_raw_responses),
        )
}

#[instrument(skip(app))]
pub(super) async fn refresh_item_metadata(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.metadata().enqueue_metadata_refresh(item_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_item_metadata_attempts(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(query): Query<MetadataAttemptsQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .list_metadata_provider_attempts_for_item(
                item_id,
                MetadataAttemptFilter {
                    provider: query.provider,
                    status: query.status,
                },
                query.page.try_into()?,
            )
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_item_metadata_raw_responses(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(query): Query<MetadataRawResponsesQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .list_provider_raw_responses_for_item(
                item_id,
                ProviderRawResponseFilter {
                    provider: query.provider,
                },
                query.page.try_into()?,
            )
            .await?,
    ))
}

#[instrument(skip(app, request))]
pub(super) async fn enqueue_metadata_maintenance(
    State(app): State<TaruApp>,
    Json(request): Json<EnqueueMetadataMaintenanceRequest>,
) -> ApiResult<impl IntoResponse> {
    let job = app.metadata().enqueue_metadata_maintenance(request).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app, request))]
pub(super) async fn plan_metadata_maintenance(
    State(app): State<TaruApp>,
    Json(request): Json<EnqueueMetadataMaintenanceRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata().plan_metadata_maintenance(request).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn cleanup_metadata_raw_responses(
    State(app): State<TaruApp>,
    Query(query): Query<MetadataRawCleanupQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.metadata()
            .cleanup_provider_raw_responses(
                ProviderRawResponseFilter {
                    provider: query.provider,
                },
                query.fetched_before,
                query.retention_ms,
            )
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_metadata_providers(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.metadata().list_metadata_provider_diagnostics()))
}
