use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use taru_api::JobResponse;
use taru_core::LibraryId;
use tracing::instrument;

use crate::app::TaruApp;

use super::{error::ApiResult, query::PageQuery};

#[instrument(skip(app))]
pub(super) async fn list_libraries(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_libraries(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn scan_library(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_library_scan(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn import_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn export_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_library_sources(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_library_sources(library_id, page.try_into()?)
            .await?,
    ))
}
