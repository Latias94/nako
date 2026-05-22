use axum::{
    Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderValue, header},
    response::IntoResponse,
    routing::get,
};
use nako_api::public_client::SourceProbeResponse;
use nako_core::{GenreId, MediaItemId, MediaSourceId, PersonId, SelectedArtworkId, TagId};
use tracing::instrument;

use crate::app::NakoApp;

use super::{
    error::ApiResult,
    query::{ImageVariantQuery, PageQuery, SearchPageQuery},
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route("/items", get(list_items))
        .route("/items/{item_id}", get(get_item))
        .route("/items/{item_id}/credits", get(list_item_credits))
        .route("/items/{item_id}/images", get(list_item_images))
        .route("/images/{image_id}", get(get_image).head(head_image))
        .route("/people", get(list_people))
        .route("/people/{person_id}", get(get_person))
        .route("/people/{person_id}/items", get(list_person_items))
        .route("/tags", get(list_tags))
        .route("/tags/{tag_id}/items", get(list_tag_items))
        .route("/genres", get(list_genres))
        .route("/genres/{genre_id}/items", get(list_genre_items))
        .route("/search", get(search_items))
        .route("/sources/{source_id}/probe", get(get_source_probe))
}

#[instrument(skip(app))]
pub(super) async fn list_items(
    State(app): State<NakoApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_items(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_item(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().get_item(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_credits(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_item_credits(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_images(
    State(app): State<NakoApp>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_item_images(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_image(
    State(app): State<NakoApp>,
    Path(image_id): Path<SelectedArtworkId>,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    let image = app
        .artwork()
        .read_selected_image(image_id, query.into_variant_request()?)
        .await?;
    Ok(selected_image_response(image, true))
}

#[instrument(skip(app))]
pub(super) async fn head_image(
    State(app): State<NakoApp>,
    Path(image_id): Path<SelectedArtworkId>,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    let image = app
        .artwork()
        .read_selected_image(image_id, query.into_variant_request()?)
        .await?;
    Ok(selected_image_response(image, false))
}

#[instrument(skip(app))]
pub(super) async fn list_people(
    State(app): State<NakoApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_people(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_person(
    State(app): State<NakoApp>,
    Path(person_id): Path<PersonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().get_person(person_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_person_items(
    State(app): State<NakoApp>,
    Path(person_id): Path<PersonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .list_person_items(person_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_tags(
    State(app): State<NakoApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_tags(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_tag_items(
    State(app): State<NakoApp>,
    Path(tag_id): Path<TagId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .list_tag_items(tag_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_genres(
    State(app): State<NakoApp>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().list_genres(page.try_into()?).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_genre_items(
    State(app): State<NakoApp>,
    Path(genre_id): Path<GenreId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog()
            .list_genre_items(genre_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn search_items(
    State(app): State<NakoApp>,
    Query(query): Query<SearchPageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = query.page().try_into()?;
    let facets = query
        .facet
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|facet| !facet.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(Json(
        app.catalog().search_items(query.q, facets, page).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_source_probe(
    State(app): State<NakoApp>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    Ok(Json(app.catalog().get_source_probe(source_id).await?))
}

fn selected_image_response(
    image: crate::app::ManagedArtworkImageBytes,
    include_body: bool,
) -> axum::response::Response {
    let mut response = if include_body {
        Body::from(image.bytes).into_response()
    } else {
        Body::empty().into_response()
    };
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&image.media_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&image.content_length.to_string())
            .expect("content length is a valid header"),
    );
    if let Some(etag) = image.etag {
        let quoted = format!("\"{etag}\"");
        if let Ok(value) = HeaderValue::from_str(&quoted) {
            headers.insert(header::ETAG, value);
        }
    }
    response
}
