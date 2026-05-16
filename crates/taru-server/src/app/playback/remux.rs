use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use taru_core::{
    MediaSource, MediaSourceId, NewTranscodeSession, Result, TaruError, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionRepository,
    TranscodeSessionState,
};
use taru_db::SqliteStore;
use taru_streaming::PlaybackDecision;
use taru_transcode::{
    CancellationToken, FfmpegCommandBuilder, FfmpegOverwritePolicy, FfmpegRemuxRunner,
    RemuxContainer, RemuxRequest, RemuxRunOutcome, RemuxRuntimeGuard, RemuxRuntimeLimits,
    TranscodeSessionManager,
};
use tokio::sync::Mutex;

use crate::config::TaruServerConfig;

use super::{
    PlaybackSessionCancellationRegistry, RemuxSourceDisposition, RemuxSourceOutput,
    ensure_remux_output_parent, map_remux_runner_error, path_exists, persist_session_failure,
    record_playback_session_finished_event,
};

#[derive(Clone, Debug)]
pub(super) struct RemuxAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegRemuxRunner,
    cancellations: PlaybackSessionCancellationRegistry,
    in_flight: Arc<Mutex<HashSet<RemuxRequestKey>>>,
}

impl RemuxAppService {
    pub(super) fn new(
        config: &TaruServerConfig,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Self {
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
            max_concurrent_sessions: config.remux_concurrency,
            timeout_ms: config.remux_timeout_ms,
        });

        Self {
            builder: FfmpegCommandBuilder::new(&config.ffmpeg_path),
            runner: FfmpegRemuxRunner::new(guard),
            cancellations,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub(super) async fn run(
        &self,
        sessions: &SqliteStore,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
    ) -> Result<RemuxSourceOutput> {
        let key = RemuxRequestKey {
            source_id: source.id,
            output_container,
        };

        match self.reserve(sessions, &key, &output_path).await? {
            RemuxRequestAdmission::ReuseExisting { session } => Ok(RemuxSourceOutput {
                source,
                decision,
                output_path,
                output_container,
                disposition: RemuxSourceDisposition::ReusedExisting,
                session,
            }),
            RemuxRequestAdmission::Run { session } => {
                let result = self
                    .run_reserved(
                        sessions,
                        session,
                        source,
                        decision,
                        input_path,
                        output_path,
                        output_container,
                    )
                    .await;
                self.release(&key).await;
                result
            }
        }
    }

    async fn reserve(
        &self,
        sessions: &SqliteStore,
        key: &RemuxRequestKey,
        output_path: &Path,
    ) -> Result<RemuxRequestAdmission> {
        let request_key = key.persisted_request_key();
        if let Some(active) = sessions
            .find_active_transcode_session(key.source_id, TranscodeSessionKind::Remux, &request_key)
            .await?
        {
            return Err(TaruError::Conflict {
                message: format!(
                    "remux request for source {} as {:?} is already in progress in session {}",
                    key.source_id, key.output_container, active.id
                ),
            });
        }

        let latest = sessions
            .find_latest_transcode_session(key.source_id, TranscodeSessionKind::Remux, &request_key)
            .await?;
        let output_exists = path_exists(output_path)?;

        if let Some(session) = latest.as_ref() {
            if session.state == TranscodeSessionState::Finished
                && session.output_path.as_path() == output_path
                && output_exists
            {
                return Ok(RemuxRequestAdmission::ReuseExisting {
                    session: Some(session.clone()),
                });
            }
        }

        if latest.is_none() && output_exists {
            return Ok(RemuxRequestAdmission::ReuseExisting { session: None });
        }

        {
            let mut in_flight = self.in_flight.lock().await;
            if !in_flight.insert(*key) {
                return Err(TaruError::Conflict {
                    message: format!(
                        "remux request for source {} as {:?} is already in progress",
                        key.source_id, key.output_container
                    ),
                });
            }
        }

        let session = sessions
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: key.source_id,
                kind: TranscodeSessionKind::Remux,
                request_key,
                output_path: output_path.to_path_buf(),
                state: TranscodeSessionState::Planned,
            })
            .await;

        match session {
            Ok(session) => Ok(RemuxRequestAdmission::Run { session }),
            Err(error) => {
                self.release(key).await;
                Err(error)
            }
        }
    }

    async fn run_reserved(
        &self,
        sessions: &SqliteStore,
        persisted_session: TranscodeSessionRecord,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
    ) -> Result<RemuxSourceOutput> {
        let session_id = persisted_session.id;
        let cancel = CancellationToken::new();
        let _cancel_handle = self.cancellations.register(session_id, cancel.clone());

        if let Err(error) = ensure_remux_output_parent(&output_path).await {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        let mut manager = TranscodeSessionManager::new();
        if let Err(error) = manager.plan_remux_with_id(
            session_id,
            RemuxRequest {
                source_id: source.id,
                input_path,
                output_path: output_path.clone(),
                output_container,
                overwrite: FfmpegOverwritePolicy::Never,
            },
            &self.builder,
        ) {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        sessions
            .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
            .await?;

        let run_result = self
            .runner
            .run(&mut manager, session_id, cancel)
            .await
            .map_err(map_remux_runner_error);

        drop(_cancel_handle);

        match run_result {
            Ok(RemuxRunOutcome::Finished { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Finished,
                        None,
                        None,
                    )
                    .await?;
                record_playback_session_finished_event(sessions, &session).await;

                Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Finished,
                    session: Some(session),
                })
            }
            Ok(RemuxRunOutcome::Cancelled { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Cancelled,
                        Some(TranscodeFailureCategory::Cancelled),
                        Some("remux session was cancelled".to_owned()),
                    )
                    .await?;

                Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Cancelled,
                    session: Some(session),
                })
            }
            Err(error) => {
                persist_session_failure(sessions, session_id, &error).await;
                Err(error)
            }
        }
    }

    async fn release(&self, key: &RemuxRequestKey) {
        self.in_flight.lock().await.remove(key);
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RemuxRequestKey {
    pub(crate) source_id: MediaSourceId,
    pub(crate) output_container: RemuxContainer,
}

impl RemuxRequestKey {
    pub(crate) fn persisted_request_key(self) -> String {
        format!("remux:{}", self.output_container.file_extension())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RemuxRequestAdmission {
    Run {
        session: TranscodeSessionRecord,
    },
    ReuseExisting {
        session: Option<TranscodeSessionRecord>,
    },
}
