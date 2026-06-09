use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
};
use nako_api::public_client::{
    ContinueWatchingItemDto, ContinueWatchingResponse, SetWatchedStateRequest,
    UpdatePlaybackProgressRequest, page_info_from_request, selected_artwork_to_public_image_ref,
    user_playback_state_response_from_state, user_playback_state_to_dto,
};
use nako_core::{AuthenticatedPrincipal, MediaItemId, MediaSourceId, NakoError};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::instrument;

use crate::app::{
    NakoApp,
    user_playback::{
        SetUserWatchedStateRequest as AppSetWatchedStateRequest,
        UpdateUserPlaybackProgressRequest as AppUpdateUserPlaybackProgressRequest,
    },
};

use super::{error::ApiResult, query::PageQuery};

pub(super) fn routes() -> Router<NakoApp> {
    Router::new()
        .route(
            "/users/me/playback-state/items/{item_id}",
            get(get_user_playback_state),
        )
        .route(
            "/users/me/playback-state/continue-watching",
            get(list_continue_watching),
        )
        .route(
            "/users/me/playback-state/items/{item_id}/progress",
            put(update_user_playback_progress),
        )
        .route(
            "/users/me/playback-state/items/{item_id}/watched",
            put(set_user_watched_state),
        )
}

#[instrument(skip(app, principal))]
async fn get_user_playback_state(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(user_playback_state_response_from_state(
        app.user_playback().get_state(&principal, item_id).await?,
    )))
}

#[instrument(skip(app, principal))]
async fn list_continue_watching(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let entries = app
        .user_playback()
        .list_continue_watching_entries(&principal, page)
        .await?;
    let items = entries
        .into_iter()
        .map(|entry| ContinueWatchingItemDto {
            item: nako_api::public_client::media_item_to_dto(entry.item),
            state: user_playback_state_to_dto(entry.state),
            images: entry
                .images
                .into_iter()
                .map(|image| selected_artwork_to_public_image_ref(image.selected, image.artifact))
                .collect(),
        })
        .collect::<Vec<_>>();

    Ok(Json(ContinueWatchingResponse {
        page: page_info_from_request(page, items.len()),
        items,
    }))
}

#[instrument(skip(app, principal, request))]
async fn update_user_playback_progress(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
    Json(request): Json<UpdatePlaybackProgressRequest>,
) -> ApiResult<impl IntoResponse> {
    let source_id = parse_optional_media_source_id(request.source_id)?;

    Ok(Json(user_playback_state_response_from_state(
        app.user_playback()
            .update_progress(AppUpdateUserPlaybackProgressRequest {
                principal,
                item_id,
                source_id,
                position_ms: request.position_ms,
                duration_ms: request.duration_ms,
                reported_at_ms: parse_optional_rfc3339_ms(request.reported_at.as_deref())?,
            })
            .await?,
    )))
}

#[instrument(skip(app, principal, request))]
async fn set_user_watched_state(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Path(item_id): Path<MediaItemId>,
    Json(request): Json<SetWatchedStateRequest>,
) -> ApiResult<impl IntoResponse> {
    let source_id = parse_optional_media_source_id(request.source_id)?;

    Ok(Json(user_playback_state_response_from_state(
        app.user_playback()
            .set_watched_state(AppSetWatchedStateRequest {
                principal,
                item_id,
                watched: request.watched,
                source_id,
                position_ms: request.position_ms,
                duration_ms: request.duration_ms,
                marked_at_ms: parse_optional_rfc3339_ms(request.marked_at.as_deref())?,
            })
            .await?,
    )))
}

fn parse_optional_media_source_id(
    value: Option<String>,
) -> Result<Option<MediaSourceId>, NakoError> {
    value
        .map(|value| {
            value
                .parse::<MediaSourceId>()
                .map_err(|err| NakoError::InvalidInput {
                    message: format!("invalid source_id: {err}"),
                })
        })
        .transpose()
}

fn parse_optional_rfc3339_ms(value: Option<&str>) -> Result<Option<i64>, NakoError> {
    value.map(parse_rfc3339_ms).transpose()
}

fn parse_rfc3339_ms(value: &str) -> Result<i64, NakoError> {
    let timestamp =
        OffsetDateTime::parse(value, &Rfc3339).map_err(|err| NakoError::InvalidInput {
            message: format!("invalid RFC3339 timestamp: {err}"),
        })?;
    let millis = timestamp
        .unix_timestamp_nanos()
        .checked_div(1_000_000)
        .ok_or_else(|| NakoError::InvalidInput {
            message: "timestamp does not fit millisecond precision".to_owned(),
        })?;

    i64::try_from(millis).map_err(|err| NakoError::InvalidInput {
        message: format!("timestamp does not fit i64 milliseconds: {err}"),
    })
}
