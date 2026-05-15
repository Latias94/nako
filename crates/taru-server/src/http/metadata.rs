use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use taru_api::JobResponse;
use taru_core::MediaItemId;
use tracing::instrument;

use crate::app::TaruApp;

use super::{error::ApiResult, query::PageQuery};

#[instrument(skip(app))]
pub(super) async fn refresh_item_metadata(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_metadata_refresh(item_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_item_metadata_attempts(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_metadata_provider_attempts_for_item(item_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_item_metadata_raw_responses(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_provider_raw_responses_for_item(item_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_metadata_providers(
    State(app): State<TaruApp>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_metadata_provider_diagnostics()))
}
