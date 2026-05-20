use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use taru_api::admin::{IgnoreIngestionFailureRequest, JobResponse};
use taru_core::{IngestionFailureStatus, LibraryId};
use tracing::instrument;

use crate::app::TaruApp;

use super::{
    error::ApiResult,
    query::{IngestionFailureQuery, PageQuery},
};

pub(super) fn routes() -> Router<TaruApp> {
    Router::new()
        .route("/libraries", get(list_libraries))
        .route("/libraries/{library_id}", get(get_library))
        .route("/libraries/{library_id}/scan", post(scan_library))
        .route("/libraries/{library_id}/nfo/import", post(import_nfo))
        .route("/libraries/{library_id}/nfo/export", post(export_nfo))
        .route("/libraries/{library_id}/sources", get(list_library_sources))
        .route(
            "/libraries/{library_id}/ingestion/failures",
            get(list_ingestion_failures).post(ignore_ingestion_failure),
        )
}

#[instrument(skip(app))]
pub(super) async fn list_libraries(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.library().list_libraries(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_library(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.library().get_library(library_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn scan_library(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.library_scan().enqueue_library_scan(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn import_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.nfo().enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn export_nfo(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    let job = app.nfo().enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_library_sources(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .list_library_sources(library_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_ingestion_failures(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Query(query): Query<IngestionFailureQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .list_ingestion_failures(
                library_id,
                query.phase,
                query.status.or(Some(IngestionFailureStatus::Open)),
                query.page.try_into()?,
            )
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn ignore_ingestion_failure(
    State(app): State<TaruApp>,
    Path(library_id): Path<LibraryId>,
    Json(request): Json<IgnoreIngestionFailureRequest>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .ignore_ingestion_failure(library_id, request.phase, &request.target_uri)
            .await?,
    ))
}
