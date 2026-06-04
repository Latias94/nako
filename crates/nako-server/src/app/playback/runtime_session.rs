use std::path::Path;

use nako_core::{
    MediaSourceId, NakoError, PlaybackSessionId, PlaybackSessionRecord, Result, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};

use super::{PlaybackAppService, PlaybackRuntimeStore, StartPlaybackSessionRequest, path_exists};

#[derive(Clone, Copy, Debug)]
pub(super) struct TranscodeRuntimeSessionKey<'a> {
    pub(super) source_id: MediaSourceId,
    pub(super) kind: TranscodeSessionKind,
    pub(super) request_key: &'a str,
}

impl<'a> TranscodeRuntimeSessionKey<'a> {
    #[must_use]
    pub(super) const fn new(
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &'a str,
    ) -> Self {
        Self {
            source_id,
            kind,
            request_key,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PlaybackTranscodeRuntimeBinding {
    kind: TranscodeSessionKind,
    session_description: &'static str,
    mode_label: &'static str,
}

impl PlaybackTranscodeRuntimeBinding {
    pub(super) const HLS: Self = Self {
        kind: TranscodeSessionKind::HlsTranscode,
        session_description: "hls transcode session",
        mode_label: "hls",
    };

    pub(super) const REMUX: Self = Self {
        kind: TranscodeSessionKind::Remux,
        session_description: "remux session",
        mode_label: "remux",
    };
}

pub(super) async fn find_active_runtime_session(
    store: &dyn PlaybackRuntimeStore,
    key: TranscodeRuntimeSessionKey<'_>,
) -> Result<Option<TranscodeSessionRecord>> {
    store
        .find_active_transcode_session(key.source_id, key.kind, key.request_key)
        .await
}

pub(super) async fn find_latest_runtime_session(
    store: &dyn PlaybackRuntimeStore,
    key: TranscodeRuntimeSessionKey<'_>,
) -> Result<Option<TranscodeSessionRecord>> {
    store
        .find_latest_transcode_session(key.source_id, key.kind, key.request_key)
        .await
}

pub(super) async fn find_finished_runtime_session_with_output(
    store: &dyn PlaybackRuntimeStore,
    key: TranscodeRuntimeSessionKey<'_>,
    expected_output_path: &Path,
) -> Result<Option<TranscodeSessionRecord>> {
    let Some(latest) = find_latest_runtime_session(store, key).await? else {
        return Ok(None);
    };

    if latest.state == TranscodeSessionState::Finished
        && latest.output_path.as_path() == expected_output_path
        && path_exists(expected_output_path)?
    {
        return Ok(Some(latest));
    }

    Ok(None)
}

pub(super) async fn start_linked_playback_session(
    app: &PlaybackAppService,
    request: StartPlaybackSessionRequest,
    transcode_session_id: TranscodeSessionId,
) -> Result<PlaybackSessionRecord> {
    let playback_session = app.start_playback_session(request).await?;
    link_playback_session_to_transcode(app, playback_session.id, transcode_session_id).await
}

pub(super) async fn link_playback_session_to_transcode(
    app: &PlaybackAppService,
    playback_session_id: PlaybackSessionId,
    transcode_session_id: TranscodeSessionId,
) -> Result<PlaybackSessionRecord> {
    app.link_playback_session_transcode(playback_session_id, transcode_session_id)
        .await
}

pub(super) async fn get_bound_transcode_session(
    app: &PlaybackAppService,
    playback_session_id: PlaybackSessionId,
    source_id: MediaSourceId,
    transcode_session_id: TranscodeSessionId,
    binding: PlaybackTranscodeRuntimeBinding,
) -> Result<TranscodeSessionRecord> {
    let session = app.get_transcode_session(transcode_session_id).await?;
    if session.kind != binding.kind {
        return Err(NakoError::InvalidInput {
            message: format!(
                "session {transcode_session_id} is not a {}",
                binding.session_description
            ),
        });
    }
    if session.source_id != source_id {
        return Err(NakoError::InvalidInput {
            message: format!(
                "{} playback session {playback_session_id} source_id does not match transcode session {}",
                binding.mode_label, session.id
            ),
        });
    }

    Ok(session)
}

pub(super) fn missing_playback_transcode_error(
    playback_session_id: PlaybackSessionId,
    artifact_label: &'static str,
) -> NakoError {
    NakoError::InvalidInput {
        message: format!(
            "playback session {playback_session_id} does not have an {artifact_label}"
        ),
    }
}

pub(super) fn missing_finished_output_error(
    output_path: &Path,
    message: &'static str,
) -> NakoError {
    NakoError::storage_io(output_path.display().to_string(), message)
}

pub(super) fn cancelled_transcode_error(
    provider: &'static str,
    message: &'static str,
) -> NakoError {
    NakoError::Provider {
        provider: provider.to_owned(),
        message: message.to_owned(),
    }
}

pub(super) fn failed_transcode_error(
    session: TranscodeSessionRecord,
    provider: &'static str,
    fallback_message: &'static str,
) -> NakoError {
    NakoError::Provider {
        provider: provider.to_owned(),
        message: session
            .failure_message
            .unwrap_or_else(|| fallback_message.to_owned()),
    }
}
