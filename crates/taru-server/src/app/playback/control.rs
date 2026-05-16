use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use taru_core::TranscodeSessionId;
use taru_transcode::CancellationToken;

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
