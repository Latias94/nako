use axum::{
    Extension, Json, Router,
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
    routing::get,
};
use nako_api::public_client::{ItemDetailResponse, SourceProbeResponse};
use nako_core::{
    AuthenticatedPrincipal, GenreId, MediaItemId, MediaSourceId, PageRequest, PersonId,
    SelectedArtworkId, TagId,
};
use tracing::instrument;

use crate::app::NakoApp;

use super::{
    access::{
        RequiredLibraryAccess, item_has_access, page_returned_len, parse_public_item_id,
        parse_public_source_id, require_item_access, require_selected_artwork_access,
        require_source_access,
    },
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
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app.catalog().list_items(page.try_into()?).await?;
    let mut items = Vec::with_capacity(response.items.len());

    for item in response.items {
        let item_id = parse_public_item_id(&item.id)?;
        if item_has_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await? {
            items.push(item);
        }
    }

    response.page.returned = page_returned_len(items.len());
    response.items = items;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn get_item(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    Ok(Json(
        filter_item_detail_sources(&app, &principal, app.catalog().get_item(item_id).await?)
            .await?,
    ))
}

#[instrument(skip(app))]
pub(super) async fn list_item_credits(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    Ok(Json(app.catalog().list_item_credits(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_item_images(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    Ok(Json(app.catalog().list_item_images(item_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn get_image(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(image_id): Path<SelectedArtworkId>,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    require_selected_artwork_access(&app, &principal, image_id, RequiredLibraryAccess::Browse)
        .await?;

    let image = app
        .artwork()
        .read_selected_image(image_id, query.into_variant_request()?)
        .await?;
    Ok(selected_image_response(image, true))
}

#[instrument(skip(app))]
pub(super) async fn head_image(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(image_id): Path<SelectedArtworkId>,
    Query(query): Query<ImageVariantQuery>,
) -> ApiResult<impl IntoResponse> {
    require_selected_artwork_access(&app, &principal, image_id, RequiredLibraryAccess::Browse)
        .await?;

    let image = app
        .artwork()
        .read_selected_image(image_id, query.into_variant_request()?)
        .await?;
    Ok(selected_image_response(image, false))
}

#[instrument(skip(app))]
pub(super) async fn list_people(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app.catalog().list_people(page.try_into()?).await?;
    let mut people = Vec::with_capacity(response.people.len());

    for person in response.people {
        let person_id =
            person
                .id
                .parse::<PersonId>()
                .map_err(|err| nako_core::NakoError::InvalidInput {
                    message: format!("invalid person id in public response: {err}"),
                })?;
        if person_has_accessible_item(&app, &principal, person_id).await? {
            people.push(person);
        }
    }

    response.page.returned = page_returned_len(people.len());
    response.people = people;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn get_person(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(person_id): Path<PersonId>,
) -> ApiResult<impl IntoResponse> {
    if !person_has_accessible_item(&app, &principal, person_id).await? {
        return Err(nako_core::NakoError::Forbidden {
            message: "required Library Access level 'browse' is not available".to_owned(),
        }
        .into());
    }

    Ok(Json(app.catalog().get_person(person_id).await?))
}

#[instrument(skip(app))]
pub(super) async fn list_person_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(person_id): Path<PersonId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app
        .catalog()
        .list_person_items(person_id, page.try_into()?)
        .await?;
    let mut items = Vec::with_capacity(response.items.len());

    for item in response.items {
        let item_id = parse_public_item_id(&item.id)?;
        if item_has_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await? {
            items.push(item);
        }
    }

    response.page.returned = page_returned_len(items.len());
    response.items = items;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn list_tags(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app.catalog().list_tags(page.try_into()?).await?;
    let mut tags = Vec::with_capacity(response.tags.len());

    for tag in response.tags {
        let tag_id = tag
            .id
            .parse::<TagId>()
            .map_err(|err| nako_core::NakoError::InvalidInput {
                message: format!("invalid tag id in public response: {err}"),
            })?;
        if tag_has_accessible_item(&app, &principal, tag_id).await? {
            tags.push(tag);
        }
    }

    response.page.returned = page_returned_len(tags.len());
    response.tags = tags;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn list_tag_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(tag_id): Path<TagId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app
        .catalog()
        .list_tag_items(tag_id, page.try_into()?)
        .await?;
    let mut items = Vec::with_capacity(response.items.len());

    for item in response.items {
        let item_id = parse_public_item_id(&item.id)?;
        if item_has_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await? {
            items.push(item);
        }
    }

    response.page.returned = page_returned_len(items.len());
    response.items = items;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn list_genres(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app.catalog().list_genres(page.try_into()?).await?;
    let mut genres = Vec::with_capacity(response.genres.len());

    for genre in response.genres {
        let genre_id =
            genre
                .id
                .parse::<GenreId>()
                .map_err(|err| nako_core::NakoError::InvalidInput {
                    message: format!("invalid genre id in public response: {err}"),
                })?;
        if genre_has_accessible_item(&app, &principal, genre_id).await? {
            genres.push(genre);
        }
    }

    response.page.returned = page_returned_len(genres.len());
    response.genres = genres;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn list_genre_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(genre_id): Path<GenreId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let mut response = app
        .catalog()
        .list_genre_items(genre_id, page.try_into()?)
        .await?;
    let mut items = Vec::with_capacity(response.items.len());

    for item in response.items {
        let item_id = parse_public_item_id(&item.id)?;
        if item_has_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await? {
            items.push(item);
        }
    }

    response.page.returned = page_returned_len(items.len());
    response.items = items;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn search_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
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

    let mut response = app.catalog().search_items(query.q, facets, page).await?;
    let mut hits = Vec::with_capacity(response.hits.len());

    for hit in response.hits {
        let item_id = parse_public_item_id(&hit.item.id)?;
        if item_has_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await? {
            hits.push(hit);
        }
    }

    response.page.returned = page_returned_len(hits.len());
    response.hits = hits;

    Ok(Json(response))
}

#[instrument(skip(app))]
pub(super) async fn get_source_probe(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(source_id): Path<MediaSourceId>,
) -> ApiResult<Json<SourceProbeResponse>> {
    require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Browse).await?;

    Ok(Json(app.catalog().get_source_probe(source_id).await?))
}

async fn filter_item_detail_sources(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    mut detail: ItemDetailResponse,
) -> ApiResult<ItemDetailResponse> {
    let mut sources = Vec::with_capacity(detail.sources.len());

    for source in detail.sources {
        let source_id = parse_public_source_id(&source.id)?;
        if let Some(record) = app.get_media_source_record(source_id).await? {
            if super::access::has_library_access(
                app,
                principal,
                record.library_id,
                RequiredLibraryAccess::Browse,
            )
            .await?
            {
                sources.push(source);
            }
        }
    }

    detail.sources = sources;

    Ok(detail)
}

async fn person_has_accessible_item(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    person_id: PersonId,
) -> ApiResult<bool> {
    let response = app
        .catalog()
        .list_person_items(person_id, PageRequest::new(PageRequest::MAX_LIMIT, 0))
        .await?;
    any_public_items_accessible(app, principal, response.items).await
}

async fn tag_has_accessible_item(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    tag_id: TagId,
) -> ApiResult<bool> {
    let response = app
        .catalog()
        .list_tag_items(tag_id, PageRequest::new(PageRequest::MAX_LIMIT, 0))
        .await?;
    any_public_items_accessible(app, principal, response.items).await
}

async fn genre_has_accessible_item(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    genre_id: GenreId,
) -> ApiResult<bool> {
    let response = app
        .catalog()
        .list_genre_items(genre_id, PageRequest::new(PageRequest::MAX_LIMIT, 0))
        .await?;
    any_public_items_accessible(app, principal, response.items).await
}

async fn any_public_items_accessible(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    items: Vec<nako_api::public_client::MediaItemDto>,
) -> ApiResult<bool> {
    if items.is_empty() {
        return Ok(principal.is_administrator());
    }

    for item in items {
        let item_id = parse_public_item_id(&item.id)?;
        if item_has_access(app, principal, item_id, RequiredLibraryAccess::Browse).await? {
            return Ok(true);
        }
    }

    Ok(false)
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
    apply_selected_artwork_cache_headers(headers);
    if let Some(etag) = image.etag {
        let quoted = format!("\"{etag}\"");
        if let Ok(value) = HeaderValue::from_str(&quoted) {
            headers.insert(header::ETAG, value);
        }
    }
    response
}

fn apply_selected_artwork_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=86400"),
    );
}
