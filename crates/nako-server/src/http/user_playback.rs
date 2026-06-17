use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, put},
};
use nako_api::public_client::{
    ClientHlsSegmentContainer, ClientHlsVariantPolicy, ContinueWatchingItemDto,
    ContinueWatchingResponse, DeleteUserPlaybackProfilePreferenceResponse,
    SetUserPlaybackProfilePreferenceRequest, SetWatchedStateRequest, UpdatePlaybackProgressRequest,
    page_info_from_request, selected_artwork_to_public_image_ref,
    user_playback_profile_preference_response_from_record, user_playback_state_response_from_state,
    user_playback_state_to_dto,
};
use nako_core::{AuthenticatedPrincipal, MediaItemId, MediaSourceId, NakoError};
use nako_playback::{
    ClientPlaybackCapabilities, ClientPlaybackCapabilityRequest, PlaybackHlsSegmentContainer,
    PlaybackHlsVariantPolicy,
};
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
            "/users/me/playback-profile",
            get(get_user_playback_profile_preference)
                .put(set_user_playback_profile_preference)
                .delete(delete_user_playback_profile_preference),
        )
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
async fn get_user_playback_profile_preference(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(user_playback_profile_preference_response_from_record(
        app.user_playback()
            .get_profile_preference(&principal)
            .await?,
    )?))
}

#[instrument(skip(app, principal, request))]
async fn set_user_playback_profile_preference(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
    Json(request): Json<SetUserPlaybackProfilePreferenceRequest>,
) -> ApiResult<impl IntoResponse> {
    let capabilities = playback_profile_preference_request_to_client(request)?;
    let capabilities_json =
        serde_json::to_string(&capabilities).map_err(|err| NakoError::InvalidInput {
            message: format!("failed to serialize resolved playback capabilities: {err}"),
        })?;

    Ok(Json(user_playback_profile_preference_response_from_record(
        Some(
            app.user_playback()
                .set_profile_preference(&principal, capabilities_json)
                .await?,
        ),
    )?))
}

#[instrument(skip(app, principal))]
async fn delete_user_playback_profile_preference(
    State(app): State<NakoApp>,
    Extension(principal): Extension<AuthenticatedPrincipal>,
) -> ApiResult<impl IntoResponse> {
    Ok(Json(DeleteUserPlaybackProfilePreferenceResponse {
        deleted: app
            .user_playback()
            .delete_profile_preference(&principal)
            .await?,
    }))
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

fn playback_profile_preference_request_to_client(
    request: SetUserPlaybackProfilePreferenceRequest,
) -> Result<ClientPlaybackCapabilities, NakoError> {
    let hls_variant_policy = match request.hls_variant_policy {
        Some(ClientHlsVariantPolicy::SingleVariant) => {
            Some(PlaybackHlsVariantPolicy::SingleVariant)
        }
        Some(ClientHlsVariantPolicy::Adaptive) => Some(PlaybackHlsVariantPolicy::Adaptive),
        None => None,
        Some(ClientHlsVariantPolicy::Other(value)) => {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "unsupported playback profile preference HLS variant policy: {value}"
                ),
            });
        }
    };
    let hls_segment_container = match request.hls_segment_container {
        Some(ClientHlsSegmentContainer::MpegTs) => Some(PlaybackHlsSegmentContainer::MpegTs),
        Some(ClientHlsSegmentContainer::Fmp4) => Some(PlaybackHlsSegmentContainer::Fmp4),
        None => None,
        Some(ClientHlsSegmentContainer::Other(value)) => {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "unsupported playback profile preference HLS segment container: {value}"
                ),
            });
        }
    };

    Ok(ClientPlaybackCapabilityRequest {
        direct_play: request.direct_play,
        device_family: request.device_family,
        profile_version: request.profile_version,
        containers: request.containers,
        video_codecs: request.video_codecs,
        audio_codecs: request.audio_codecs,
        max_video_bitrate: request.max_video_bitrate,
        max_width: request.max_width,
        max_height: request.max_height,
        max_audio_channels: request.max_audio_channels,
        supports_hdr: request.supports_hdr,
        supports_subtitles: request.supports_subtitles,
        hls_variant_policy,
        hls_segment_container,
    }
    .resolve())
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
