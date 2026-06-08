use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
};
use nako_api::public_client::{
    AddUserPlaylistItemRequest, CreateUserPlaylistRequest, ReorderUserPlaylistItemsRequest,
    UpdateUserPlaylistRequest, UserPlaylistDeleteResponse, UserPlaylistDto,
    UserPlaylistItemsResponse, UserPlaylistResponse, UserPlaylistsResponse, media_item_to_dto,
    page_info_from_request, selected_artwork_to_public_image_ref, user_playlist_item_to_dto,
    user_playlist_to_dto,
};
use nako_core::{
    AuthenticatedPrincipal, MediaItemId, NakoError, UserPlaylistId, UserPlaylistSummaryProjection,
};
use tracing::instrument;

use crate::app::{
    NakoApp,
    user_playlist::{
        AddUserPlaylistItemRequest as AppAddUserPlaylistItemRequest,
        CreateUserPlaylistRequest as AppCreateUserPlaylistRequest,
        RemoveUserPlaylistItemRequest as AppRemoveUserPlaylistItemRequest,
        RenameUserPlaylistRequest as AppRenameUserPlaylistRequest,
        ReorderUserPlaylistItemsRequest as AppReorderUserPlaylistItemsRequest,
    },
};

use super::{
    access::{RequiredLibraryAccess, require_item_access},
    error::ApiResult,
    query::PageQuery,
};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route(
            "/users/me/playlists",
            get(list_user_playlists).post(create_user_playlist),
        )
        .route(
            "/users/me/playlists/{playlist_id}",
            get(get_user_playlist)
                .patch(update_user_playlist)
                .delete(delete_user_playlist),
        )
        .route(
            "/users/me/playlists/{playlist_id}/items",
            get(list_user_playlist_items),
        )
        .route(
            "/users/me/playlists/{playlist_id}/items/{item_id}",
            put(add_user_playlist_item).delete(remove_user_playlist_item),
        )
        .route(
            "/users/me/playlists/{playlist_id}/items/reorder",
            put(reorder_user_playlist_items),
        )
}

#[instrument(skip(app, principal))]
async fn list_user_playlists(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let playlists = app
        .user_playlist()
        .list_playlist_summaries(&principal, page)
        .await?;
    let public_playlists: Vec<UserPlaylistDto> =
        playlists.into_iter().map(public_playlist_dto).collect();

    Ok(Json(UserPlaylistsResponse {
        page: page_info_from_request(page, public_playlists.len()),
        playlists: public_playlists,
    }))
}

#[instrument(skip(app, principal, request))]
async fn create_user_playlist(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<CreateUserPlaylistRequest>,
) -> ApiResult<impl IntoResponse> {
    let playlist = app
        .user_playlist()
        .create_playlist(AppCreateUserPlaylistRequest {
            principal_id: principal.principal_id.clone(),
            name: request.name,
            created_at_ms: None,
        })
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_summary_dto(&app, &principal, playlist.id).await?,
    }))
}

#[instrument(skip(app, principal))]
async fn get_user_playlist(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(playlist_id): Path<UserPlaylistId>,
) -> ApiResult<impl IntoResponse> {
    let summary = app
        .user_playlist()
        .get_playlist_summary(&principal, playlist_id)
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_dto(summary),
    }))
}

#[instrument(skip(app, principal, request))]
async fn update_user_playlist(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(playlist_id): Path<UserPlaylistId>,
    Json(request): Json<UpdateUserPlaylistRequest>,
) -> ApiResult<impl IntoResponse> {
    let playlist = app
        .user_playlist()
        .rename_playlist(AppRenameUserPlaylistRequest {
            principal_id: principal.principal_id.clone(),
            playlist_id,
            name: request.name,
            expected_version: request.expected_version,
            updated_at_ms: None,
        })
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_summary_dto(&app, &principal, playlist.id).await?,
    }))
}

#[instrument(skip(app, principal))]
async fn delete_user_playlist(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(playlist_id): Path<UserPlaylistId>,
) -> ApiResult<impl IntoResponse> {
    app.user_playlist()
        .delete_playlist(&principal.principal_id, playlist_id)
        .await?;

    Ok(Json(UserPlaylistDeleteResponse {
        playlist_id: playlist_id.to_string(),
        deleted: true,
    }))
}

#[instrument(skip(app, principal))]
async fn list_user_playlist_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(playlist_id): Path<UserPlaylistId>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let projection = app
        .user_playlist()
        .get_items_projection(&principal, playlist_id, page)
        .await?;
    let items = projection
        .items
        .into_iter()
        .enumerate()
        .map(|(index, entry)| {
            let images = entry
                .images
                .into_iter()
                .map(|image| selected_artwork_to_public_image_ref(image.selected, image.artifact))
                .collect();

            user_playlist_item_to_dto(
                entry.playlist_item,
                public_position(page.offset, index),
                media_item_to_dto(entry.item),
                images,
            )
        })
        .collect::<Vec<_>>();

    Ok(Json(UserPlaylistItemsResponse {
        playlist: user_playlist_to_dto(projection.playlist, projection.accessible_item_count),
        page: page_info_from_request(page, items.len()),
        items,
    }))
}

#[instrument(skip(app, principal, request))]
async fn add_user_playlist_item(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path((playlist_id, item_id)): Path<(UserPlaylistId, MediaItemId)>,
    Json(request): Json<AddUserPlaylistItemRequest>,
) -> ApiResult<impl IntoResponse> {
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    let playlist = app
        .user_playlist()
        .add_item(AppAddUserPlaylistItemRequest {
            principal_id: principal.principal_id.clone(),
            playlist_id,
            item_id,
            position: request.position,
            expected_version: request.expected_version,
            added_at_ms: None,
        })
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_summary_dto(&app, &principal, playlist.id).await?,
    }))
}

#[instrument(skip(app, principal))]
async fn remove_user_playlist_item(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path((playlist_id, item_id)): Path<(UserPlaylistId, MediaItemId)>,
) -> ApiResult<impl IntoResponse> {
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    let playlist = app
        .user_playlist()
        .remove_item(AppRemoveUserPlaylistItemRequest {
            principal_id: principal.principal_id.clone(),
            playlist_id,
            item_id,
            expected_version: None,
            updated_at_ms: None,
        })
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_summary_dto(&app, &principal, playlist.id).await?,
    }))
}

#[instrument(skip(app, principal, request))]
async fn reorder_user_playlist_items(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(playlist_id): Path<UserPlaylistId>,
    Json(request): Json<ReorderUserPlaylistItemsRequest>,
) -> ApiResult<impl IntoResponse> {
    let item_ids = parse_playlist_item_ids(request.item_ids)?;
    for item_id in &item_ids {
        require_item_access(&app, &principal, *item_id, RequiredLibraryAccess::Browse).await?;
    }

    let playlist = app
        .user_playlist()
        .reorder_items(AppReorderUserPlaylistItemsRequest {
            principal_id: principal.principal_id.clone(),
            playlist_id,
            item_ids,
            expected_version: request.expected_version,
            updated_at_ms: None,
        })
        .await?;

    Ok(Json(UserPlaylistResponse {
        playlist: public_playlist_summary_dto(&app, &principal, playlist.id).await?,
    }))
}

async fn public_playlist_summary_dto(
    app: &NakoApp,
    principal: &AuthenticatedPrincipal,
    playlist_id: UserPlaylistId,
) -> ApiResult<UserPlaylistDto> {
    Ok(public_playlist_dto(
        app.user_playlist()
            .get_playlist_summary(principal, playlist_id)
            .await?,
    ))
}

fn public_playlist_dto(projection: UserPlaylistSummaryProjection) -> UserPlaylistDto {
    user_playlist_to_dto(projection.playlist, projection.accessible_item_count)
}

fn parse_playlist_item_ids(values: Vec<String>) -> Result<Vec<MediaItemId>, NakoError> {
    values
        .into_iter()
        .map(|value| {
            value
                .parse::<MediaItemId>()
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid playlist item id: {err}"),
                })
        })
        .collect()
}

fn public_position(page_offset: u64, page_index: usize) -> u32 {
    page_offset
        .saturating_add(u64::try_from(page_index).unwrap_or(u64::MAX))
        .try_into()
        .unwrap_or(u32::MAX)
}
