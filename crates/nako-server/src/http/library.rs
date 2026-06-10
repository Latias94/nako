use axum::{
    Extension, Json, Router,
    extract::{Path, Query, RawQuery, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use nako_api::admin::{IgnoreIngestionFailureRequest, JobResponse};
use nako_core::{AuthenticatedPrincipal, IngestionFailureStatus, LibraryId};
use tracing::instrument;

use crate::app::{LibraryScanTraceContext, NakoApp};

use super::{
    access::require_library_manage_access,
    error::ApiResult,
    query::{IngestionFailureQuery, LibraryItemsQuery, PageQuery},
    trace_context::HttpTraceContext,
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/libraries", get(list_libraries))
        .route("/libraries/{library_id}", get(get_library))
        .route("/libraries/{library_id}/scan", post(scan_library))
        .route("/libraries/{library_id}/nfo/import", post(import_nfo))
        .route("/libraries/{library_id}/nfo/export", post(export_nfo))
        .route("/libraries/{library_id}/sources", get(list_library_sources))
        .route("/libraries/{library_id}/items", get(list_library_items))
        .route(
            "/libraries/{library_id}/ingestion/failures",
            get(list_ingestion_failures).post(ignore_ingestion_failure),
        )
}

#[instrument(skip(app))]
pub(super) async fn list_libraries(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .list_libraries_for_browse(&principal, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_library(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .get_library_for_browse(&principal, library_id)
            .await?,
    ))
}

#[instrument(skip(app, principal, http_trace_context))]
pub(super) async fn scan_library(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Extension(http_trace_context): Extension<HttpTraceContext>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    require_library_manage_access(&app, &principal, library_id).await?;

    let trace_context = LibraryScanTraceContext::from_request_id(http_trace_context.request_id())?;
    let job = app
        .library_scan()
        .enqueue_library_scan_with_trace_context(library_id, trace_context)
        .await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn import_nfo(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    require_library_manage_access(&app, &principal, library_id).await?;

    let job = app.nfo().enqueue_nfo_import(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn export_nfo(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
) -> ApiResult<impl IntoResponse> {
    require_library_manage_access(&app, &principal, library_id).await?;

    let job = app.nfo().enqueue_nfo_export(library_id).await?;

    Ok((StatusCode::ACCEPTED, Json(JobResponse::from_job(job))))
}

#[instrument(skip(app))]
pub(super) async fn list_library_sources(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .list_library_sources_for_browse(&principal, library_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_library_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.library()
            .list_library_items_for_browse(
                &principal,
                library_id,
                LibraryItemsQuery::from_raw_query(raw_query.as_deref())?.into_browse_query()?,
            )
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_ingestion_failures(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
    Query(query): Query<IngestionFailureQuery>,
) -> ApiResult<impl IntoResponse> {
    require_library_manage_access(&app, &principal, library_id).await?;

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
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(library_id): Path<LibraryId>,
    Json(request): Json<IgnoreIngestionFailureRequest>,
) -> ApiResult<impl IntoResponse> {
    require_library_manage_access(&app, &principal, library_id).await?;

    Ok(Json(
        app.library()
            .ignore_ingestion_failure(library_id, request.phase, &request.target_uri)
            .await?,
    ))
}
