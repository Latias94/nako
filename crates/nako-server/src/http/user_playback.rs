use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
};
use nako_api::public_client::{
    ContinueWatchingItemDto, ContinueWatchingResponse, SetWatchedStateRequest,
    UpdatePlaybackProgressRequest, page_info_from_request, user_playback_state_response_from_state,
    user_playback_state_to_dto,
};
use nako_core::{AuthenticatedPrincipal, MediaItemId, MediaSourceId, NakoError, UserPlaybackState};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use tracing::instrument;

use crate::app::{
    NakoApp,
    user_playback::{
        SetUserWatchedStateRequest as AppSetWatchedStateRequest,
        UpdateUserPlaybackProgressRequest as AppUpdateUserPlaybackProgressRequest,
    },
};

use super::{
    access::{RequiredLibraryAccess, item_has_access, require_item_access, require_source_access},
    error::ApiResult,
    query::PageQuery,
};

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
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Browse).await?;

    Ok(Json(user_playback_state_response_from_state(
        app.user_playback()
            .get_state(&principal.principal_id, item_id)
            .await?,
    )))
}

#[instrument(skip(app, principal))]
async fn list_continue_watching(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Query(page): Query<PageQuery>,
) -> ApiResult<impl IntoResponse> {
    let page = page.try_into()?;
    let states = app
        .user_playback()
        .list_continue_watching(&principal.principal_id, page)
        .await?;
    let mut items = Vec::with_capacity(states.len());

    for state in states {
        if item_has_access(
            &app,
            &principal,
            state.item_id,
            RequiredLibraryAccess::Browse,
        )
        .await?
        {
            items.push(continue_watching_item(&app, state).await?);
        }
    }

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
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Play).await?;
    if let Some(source_id) = source_id {
        require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
    }

    Ok(Json(user_playback_state_response_from_state(
        app.user_playback()
            .update_progress(AppUpdateUserPlaybackProgressRequest {
                principal_id: principal.principal_id,
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
    require_item_access(&app, &principal, item_id, RequiredLibraryAccess::Play).await?;
    if let Some(source_id) = source_id {
        require_source_access(&app, &principal, source_id, RequiredLibraryAccess::Play).await?;
    }

    Ok(Json(user_playback_state_response_from_state(
        app.user_playback()
            .set_watched_state(AppSetWatchedStateRequest {
                principal_id: principal.principal_id,
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

async fn continue_watching_item(
    app: &NakoApp,
    state: UserPlaybackState,
) -> ApiResult<ContinueWatchingItemDto> {
    let detail = app.catalog().get_item(state.item_id).await?;

    Ok(ContinueWatchingItemDto {
        item: detail.item,
        state: user_playback_state_to_dto(state),
        images: detail.images,
    })
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
