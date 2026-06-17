use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, Query, RawQuery, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use nako_api::public_client::SourceProbeResponse;
use nako_core::{
    AuthenticatedPrincipal, GenreId, MediaItemId, MediaSourceId, PersonId, SelectedArtworkId, TagId,
};
use tracing::instrument;

use crate::app::NakoApp;

use super::{
    error::ApiResult,
    no_store_json,
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
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_items(&principal, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_item(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().get_item(&principal, item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_credits(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog().list_item_credits(&principal, item_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_item_images(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(
        app.catalog().list_item_images(&principal, item_id).await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_image(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(image_id): Path<SelectedArtworkId>,
    headers: HeaderMap,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    let artwork = app.artwork();
    let image_access = artwork.selected_image_access(&principal, image_id).await?;
    let variant = query.into_variant_request()?;
    if let Some(response) = selected_image_preflight_response(
        &app,
        &image_access,
        variant,
        headers.get(header::IF_NONE_MATCH),
    )
    .await?
    {
        return Ok(response);
    }

    let image = artwork.read_selected_image(&image_access, variant).await?;
    Ok(selected_image_response(
        image,
        true,
        headers.get(header::IF_NONE_MATCH),
    ))
}

#[instrument(skip(app))]
pub(super) async fn head_image(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(image_id): Path<SelectedArtworkId>,
    headers: HeaderMap,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    let artwork = app.artwork();
    let image_access = artwork.selected_image_access(&principal, image_id).await?;
    let variant = query.into_variant_request()?;
    if let Some(response) = selected_image_preflight_response(
        &app,
        &image_access,
        variant,
        headers.get(header::IF_NONE_MATCH),
    )
    .await?
    {
        return Ok(response);
    }

    let image = artwork.read_selected_image(&image_access, variant).await?;
    Ok(selected_image_response(
        image,
        false,
        headers.get(header::IF_NONE_MATCH),
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_people(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_people(&principal, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_person(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(person_id): Path<PersonId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(app.catalog().get_person(&principal, person_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_person_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(person_id): Path<PersonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_person_items(&principal, person_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_tags(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_tags(&principal, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_tag_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(tag_id): Path<TagId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_tag_items(&principal, tag_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_genres(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_genres(&principal, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_genre_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(genre_id): Path<GenreId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    Ok(no_store_json(
        app.catalog()
            .list_accessible_genre_items(&principal, genre_id, page.try_into()?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn search_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    RawQuery(raw_query): RawQuery,
) -> ApiResult<impl IntoResponse> {
    let query = SearchPageQuery::from_raw_query(raw_query.as_deref())?;
    let page = query.page().try_into()?;
    let facets = query.facets();

    Ok(no_store_json(
        app.catalog()
            .search_accessible_items(&principal, query.q.unwrap_or_default(), facets, page)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn get_source_probe(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    Ok(Json(
        app.catalog()
            .get_source_probe(&principal, source_id)
            .await?,
    ))
}

async fn selected_image_preflight_response(
    app: &NakoApp,
    image_access: &crate::app::SelectedArtworkImageAccess,
    variant: crate::app::ImageVariantRequest,
    if_none_match: Option<&HeaderValue>,
) -> ApiResult<Option<axum::response::Response>> {
    let Some(if_none_match) = if_none_match else {
        return Ok(None);
    };
    let Some(preflight) = app
        .artwork()
        .selected_image_preflight(image_access, variant)
        .await?
    else {
        return Ok(None);
    };
    let Some(etag) = selected_image_etag_header_value(&preflight.etag) else {
        return Ok(None);
    };

    Ok(selected_image_etag_matches(Some(if_none_match), &etag)
        .then(|| selected_image_not_modified_response(etag)))
}

fn selected_image_response(
    image: crate::app::ManagedArtworkImageBytes,
    include_body: bool,
    if_none_match: Option<&HeaderValue>,
) -> axum::response::Response {
    let etag = image
        .etag
        .as_deref()
        .and_then(selected_image_etag_header_value);
    if let Some(matched_etag) = etag
        .as_ref()
        .filter(|etag| selected_image_etag_matches(if_none_match, etag))
        .cloned()
    {
        return selected_image_not_modified_response(matched_etag);
    }

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
    apply_selected_artwork_cache_headers(headers);
    if let Some(etag) = etag {
        headers.insert(header::ETAG, etag);
    }
    response
}

fn selected_image_not_modified_response(etag: HeaderValue) -> axum::response::Response {
    let mut response = Body::empty().into_response();
    *response.status_mut() = StatusCode::NOT_MODIFIED;
    let headers = response.headers_mut();
    apply_selected_artwork_cache_headers(headers);
    headers.insert(header::ETAG, etag);
    response
}

fn selected_image_etag_header_value(etag: &str) -> Option<HeaderValue> {
    HeaderValue::from_str(&format!("\"{etag}\"")).ok()
}

fn selected_image_etag_matches(if_none_match: Option<&HeaderValue>, etag: &HeaderValue) -> bool {
    let Some(if_none_match) = if_none_match else {
        return false;
    };
    let Ok(if_none_match) = if_none_match.to_str() else {
        return false;
    };
    let Ok(etag) = etag.to_str() else {
        return false;
    };

    if_none_match
        .split(',')
        .map(str::trim)
        .any(|candidate| selected_image_entity_tag_matches(candidate, etag))
}

fn selected_image_entity_tag_matches(candidate: &str, etag: &str) -> bool {
    if candidate == "*" {
        return true;
    }

    candidate.strip_prefix("W/").unwrap_or(candidate) == etag
}

fn apply_selected_artwork_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=86400"),
    );
}
