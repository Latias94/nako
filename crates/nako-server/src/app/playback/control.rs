use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use nako_core::{
    MediaSourceId, PageRequest, Result, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionListFilter, TranscodeSessionRecord, TranscodeSessionState,
};
use nako_transcode::CancellationToken;
use tracing::debug;

#[derive(Clone, Debug, Default)]
pub(super) struct PlaybackSessionCancellationRegistry {
    tokens: Arc<Mutex<HashMap<TranscodeSessionId, CancellationToken>>>,
}

#[derive(Clone, Debug)]
pub(super) struct PlaybackSessionCancellationHandle {
    registry: PlaybackSessionCancellationRegistry,
    session_id: TranscodeSessionId,
}

impl PlaybackSessionCancellationRegistry {
    #[must_use]
    pub(super) fn register(
        &self,
        session_id: TranscodeSessionId,
        token: CancellationToken,
    ) -> PlaybackSessionCancellationHandle {
        self.tokens
            .lock()
            .expect("playback cancellation registry poisoned")
            .insert(session_id, token);

        PlaybackSessionCancellationHandle {
            registry: self.clone(),
            session_id,
        }
    }

    pub(super) fn cancel(&self, session_id: TranscodeSessionId) -> bool {
        let Some(token) = self
            .tokens
            .lock()
            .expect("playback cancellation registry poisoned")
            .get(&session_id)
            .cloned()
        else {
            return false;
        };

        token.cancel();
        true
    }

    pub(super) fn remove(&self, session_id: TranscodeSessionId) {
        self.tokens
            .lock()
            .expect("playback cancellation registry poisoned")
            .remove(&session_id);
    }
}

impl Drop for PlaybackSessionCancellationHandle {
    fn drop(&mut self) {
        self.registry.remove(self.session_id);
    }
}

pub(super) async fn hls_supersede_candidates(
    sessions: &dyn super::PlaybackRuntimeStore,
    source_id: MediaSourceId,
    replacement_request_key: String,
) -> Result<Vec<TranscodeSessionRecord>> {
    let mut candidates = Vec::new();
    let mut seen = HashSet::new();

    for state in SUPERSEDABLE_HLS_STATES {
        let active = sessions
            .list_transcode_sessions(
                TranscodeSessionListFilter {
                    source_id: Some(source_id),
                    kind: Some(TranscodeSessionKind::HlsTranscode),
                    state: Some(state),
                },
                PageRequest::new(PageRequest::MAX_LIMIT, 0),
            )
            .await?;

        for session in active {
            if session.request_key == replacement_request_key || !seen.insert(session.id) {
                continue;
            }
            candidates.push(session);
        }
    }

    Ok(candidates)
}

pub(super) async fn request_hls_session_supersede(
    sessions: &dyn super::PlaybackRuntimeStore,
    cancellations: &PlaybackSessionCancellationRegistry,
    source_id: MediaSourceId,
    replacement_request_key: String,
    candidates: Vec<TranscodeSessionRecord>,
) -> Result<Vec<TranscodeSessionRecord>> {
    let mut superseded = Vec::new();

    for session in candidates {
        let local_cancelled = cancellations.cancel(session.id);
        let updated = sessions
            .request_transcode_session_cancellation(
                session.id,
                format!(
                    "hls session {} superseded by hls request {}",
                    session.id, replacement_request_key
                ),
            )
            .await?;

        if local_cancelled {
            debug!(
                transcode_session_id = %session.id,
                source_id = %source_id,
                "signalled local hls runner cancellation for superseded session"
            );
        }

        superseded.push(updated.unwrap_or(session));
    }

    Ok(superseded)
}

const SUPERSEDABLE_HLS_STATES: [TranscodeSessionState; 4] = [
    TranscodeSessionState::Planned,
    TranscodeSessionState::Starting,
    TranscodeSessionState::Running,
    TranscodeSessionState::CancelRequested,
];
