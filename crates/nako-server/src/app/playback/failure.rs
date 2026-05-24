use nako_core::{NakoError, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionState};
use tracing::error;

pub(super) fn map_remux_runner_error(error: NakoError) -> NakoError {
    match error {
        NakoError::Provider { provider, message } if provider == "ffmpeg" => {
            let message = if message.to_ascii_lowercase().contains("timed out") {
                "remux runner timed out".to_owned()
            } else {
                "remux runner failed".to_owned()
            };

            NakoError::Provider {
                provider: "ffmpeg_remux".to_owned(),
                message,
            }
        }
        NakoError::Storage { uri, kind, .. } => {
            NakoError::storage(uri, kind, "remux staging operation failed")
        }
        NakoError::InvalidInput { message } => NakoError::InvalidInput {
            message: format!("invalid remux request: {message}"),
        },
        other => other,
    }
}

pub(super) fn map_hls_runner_error(error: NakoError) -> NakoError {
    match error {
        NakoError::Provider { provider, message } if provider == "ffmpeg" => {
            let message = if message.to_ascii_lowercase().contains("timed out") {
                "hls runner timed out".to_owned()
            } else {
                "hls runner failed".to_owned()
            };

            NakoError::Provider {
                provider: "ffmpeg_hls".to_owned(),
                message,
            }
        }
        NakoError::Storage { uri, kind, .. } => {
            NakoError::storage(uri, kind, "hls staging operation failed")
        }
        NakoError::InvalidInput { message } => NakoError::InvalidInput {
            message: format!("invalid hls request: {message}"),
        },
        other => other,
    }
}

pub(super) async fn persist_session_failure(
    sessions: &dyn super::PlaybackRuntimeStore,
    session_id: TranscodeSessionId,
    error: &NakoError,
) {
    let failure = PlaybackTranscodeFailure::from_error(error);
    if let Err(update_error) = sessions
        .set_transcode_session_state(
            session_id,
            TranscodeSessionState::Failed,
            Some(failure.category),
            Some(failure.operator_message),
        )
        .await
    {
        error!(
            session_id = %session_id,
            error = %update_error,
            "failed to persist transcode session failure"
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PlaybackTranscodeFailure {
    category: TranscodeFailureCategory,
    operator_message: String,
}

impl PlaybackTranscodeFailure {
    fn from_error(error: &NakoError) -> Self {
        let category = TranscodeFailureCategory::from_error(error);
        Self {
            category,
            operator_message: playback_failure_operator_message(category, error),
        }
    }
}

fn playback_failure_operator_message(
    category: TranscodeFailureCategory,
    error: &NakoError,
) -> String {
    match category {
        TranscodeFailureCategory::Probe => "ffmpeg probe failed".to_owned(),
        TranscodeFailureCategory::Plan => "playback transcode planning failed".to_owned(),
        TranscodeFailureCategory::Staging => "playback staging operation failed".to_owned(),
        TranscodeFailureCategory::Budget => "playback resource budget was exhausted".to_owned(),
        TranscodeFailureCategory::HardwareFallback => {
            "playback hardware acceleration was unavailable".to_owned()
        }
        TranscodeFailureCategory::Runner => match error {
            NakoError::Provider { provider, .. } if provider == "ffmpeg_remux" => {
                "remux runner failed".to_owned()
            }
            NakoError::Provider { provider, .. } if provider == "ffmpeg_hls" => {
                "hls runner failed".to_owned()
            }
            _ => "playback transcode runner failed".to_owned(),
        },
        TranscodeFailureCategory::Timeout => match error {
            NakoError::Provider { provider, .. } if provider == "ffmpeg_remux" => {
                "remux runner timed out".to_owned()
            }
            NakoError::Provider { provider, .. } if provider == "ffmpeg_hls" => {
                "hls runner timed out".to_owned()
            }
            _ => "playback transcode operation timed out".to_owned(),
        },
        TranscodeFailureCategory::Storage => "playback storage operation failed".to_owned(),
        TranscodeFailureCategory::Stale => "playback session was stale at startup".to_owned(),
        TranscodeFailureCategory::Cancelled => "playback session was cancelled".to_owned(),
        TranscodeFailureCategory::InvalidRequest => "playback request was invalid".to_owned(),
        TranscodeFailureCategory::Unknown => "playback transcode operation failed".to_owned(),
    }
}
