use axum::{
    Json,
    extract::{Path, Query, State},
    response::IntoResponse,
};
use taru_api::SourceProbeResponse;
use taru_core::{GenreId, MediaItemId, MediaSourceId, PersonId, TagId};
use tracing::instrument;

use crate::app::TaruApp;

use super::{
    error::ApiResult,
    query::{PageQuery, SearchPageQuery},
};

#[instrument(skip(app))]
pub(super) async fn list_items(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_items(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_item(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_item(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_credits(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_item_credits(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_images(
    State(app): State<TaruApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_item_images(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_people(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_people(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_person(
    State(app): State<TaruApp>,
    Path(person_id): Path<PersonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.get_person(person_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_person_items(
    State(app): State<TaruApp>,
    Path(person_id): Path<PersonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_person_items(person_id, page.try_into()?).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_tags(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_tags(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_tag_items(
    State(app): State<TaruApp>,
    Path(tag_id): Path<TagId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_tag_items(tag_id, page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_genres(
    State(app): State<TaruApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.list_genres(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_genre_items(
    State(app): State<TaruApp>,
    Path(genre_id): Path<GenreId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.list_genre_items(genre_id, page.try_into()?).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn search_items(
    State(app): State<TaruApp>,
    Query(query): Query<SearchPageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = query.page.try_into()?;
    let facets = query
        .facet
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|facet| !facet.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(Json(app.search_items(query.q, facets, page).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_source_probe(
    State(app): State<TaruApp>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    Ok(Json(app.get_source_probe(source_id).await?))
}
