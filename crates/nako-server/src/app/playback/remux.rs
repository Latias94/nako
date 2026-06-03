use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

use nako_core::{
    MediaSource, MediaSourceId, NakoError, NewTranscodeSession, Result, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};
use nako_playback::PlaybackDecision;
use nako_transcode::{
    CancellationToken, FfmpegExecutionPlanner, FfmpegRemuxRunner, RemuxContainer,
    RemuxExecutionPlanRequest, TranscodeEngineAdapter, TranscodeEngineStartCommand,
    TranscodeEngineStartOutcome, TranscodeRequestIdentity, TranscodeRuntimeGuard,
};
use tokio::sync::Mutex;

use crate::config::NakoServerConfig;

use super::{
    PlaybackSessionCancellationRegistry, RemuxSourceDisposition, RemuxSourceOutput,
    ensure_remux_output_parent, map_remux_runner_error, path_exists, persist_session_failure,
    record_playback_session_finished_event,
    resource::{PlaybackResourceDemand, PlaybackResourcePermitSet, PlaybackRuntimeAdmission},
};

#[derive(Clone, Debug)]
pub(super) struct RemuxAppService {
    execution_planner: FfmpegExecutionPlanner,
    engine: FfmpegRemuxRunner,
    cancellations: PlaybackSessionCancellationRegistry,
    in_flight: Arc<Mutex<HashSet<RemuxRequestKey>>>,
}

impl RemuxAppService {
    pub(super) fn new(
        config: &NakoServerConfig,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Self {
        let guard = TranscodeRuntimeGuard::timeout_only(config.remux_timeout_ms);

        Self {
            execution_planner: FfmpegExecutionPlanner::new(&config.ffmpeg_path),
            engine: FfmpegRemuxRunner::new(guard),
            cancellations,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub(super) async fn run(
        &self,
        sessions: &dyn super::PlaybackRuntimeStore,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
        request_identity: TranscodeRequestIdentity,
        resource_admission: &PlaybackRuntimeAdmission,
        resource_demand: PlaybackResourceDemand,
        resource_permit: Option<PlaybackResourcePermitSet>,
    ) -> Result<RemuxSourceOutput> {
        let key = RemuxRequestKey {
            source_id: source.id,
            request_identity,
        };

        match self
            .reserve(
                sessions,
                &key,
                &output_path,
                resource_admission,
                &resource_demand,
                resource_permit,
            )
            .await?
        {
            RemuxRequestAdmission::ReuseExisting { session } => Ok(RemuxSourceOutput {
                source,
                decision,
                output_path,
                output_container,
                disposition: RemuxSourceDisposition::ReusedExisting,
                session,
            }),
            RemuxRequestAdmission::Run { session, permit } => {
                let result = self
                    .run_reserved(
                        sessions,
                        session,
                        source,
                        decision,
                        input_path,
                        output_path,
                        output_container,
                        permit,
                    )
                    .await;
                self.release(&key).await;
                result
            }
        }
    }

    async fn reserve(
        &self,
        sessions: &dyn super::PlaybackRuntimeStore,
        key: &RemuxRequestKey,
        output_path: &Path,
        resource_admission: &PlaybackRuntimeAdmission,
        resource_demand: &PlaybackResourceDemand,
        resource_permit: Option<PlaybackResourcePermitSet>,
    ) -> Result<RemuxRequestAdmission> {
        let request_key = key.persisted_request_key();
        if let Some(active) = sessions
            .find_active_transcode_session(key.source_id, TranscodeSessionKind::Remux, &request_key)
            .await?
        {
            return Err(NakoError::Conflict {
                message: format!(
                    "remux request for source {} is already in progress in session {}",
                    key.source_id, active.id
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
            if !in_flight.insert(key.clone()) {
                return Err(NakoError::Conflict {
                    message: format!(
                        "remux request for source {} is already in progress",
                        key.source_id
                    ),
                });
            }
        }

        let permit = match resource_permit {
            Some(permit) => permit,
            None => match resource_admission.try_acquire(resource_demand) {
                Ok(permit) => permit,
                Err(error) => {
                    self.release(key).await;
                    return Err(error);
                }
            },
        };

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
            Ok(session) => Ok(RemuxRequestAdmission::Run { session, permit }),
            Err(error) => {
                self.release(key).await;
                Err(error)
            }
        }
    }

    async fn run_reserved(
        &self,
        sessions: &dyn super::PlaybackRuntimeStore,
        persisted_session: TranscodeSessionRecord,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        output_path: PathBuf,
        output_container: RemuxContainer,
        _permit: PlaybackResourcePermitSet,
    ) -> Result<RemuxSourceOutput> {
        let session_id = persisted_session.id;
        let cancel = CancellationToken::new();
        let _cancel_handle = self.cancellations.register(session_id, cancel.clone());

        if let Err(error) = ensure_remux_output_parent(&output_path).await {
            persist_session_failure(sessions, session_id, &error).await;
            return Err(error);
        }

        let execution = match self.execution_planner.plan_remux_with_id(
            session_id,
            RemuxExecutionPlanRequest {
                source_id: source.id,
                input_path,
                output_path: output_path.clone(),
                output_container,
            },
        ) {
            Ok(execution) => execution,
            Err(error) => {
                persist_session_failure(sessions, session_id, &error).await;
                return Err(error);
            }
        };

        sessions
            .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
            .await?;

        let run_result = self
            .engine
            .start(TranscodeEngineStartCommand { execution, cancel })
            .await
            .map_err(map_remux_runner_error);

        drop(_cancel_handle);

        match run_result {
            Ok(TranscodeEngineStartOutcome::Finished { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Finished,
                        None,
                        None,
                    )
                    .await?;
                record_playback_session_finished_event(sessions, &session, None).await;

                Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Finished,
                    session: Some(session),
                })
            }
            Ok(TranscodeEngineStartOutcome::Cancelled { .. }) => {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct RemuxRequestKey {
    pub(crate) source_id: MediaSourceId,
    pub(crate) request_identity: TranscodeRequestIdentity,
}

impl RemuxRequestKey {
    pub(crate) fn persisted_request_key(&self) -> String {
        self.request_identity.persisted_request_key().to_owned()
    }
}

#[derive(Debug)]
enum RemuxRequestAdmission {
    Run {
        session: TranscodeSessionRecord,
        permit: PlaybackResourcePermitSet,
    },
    ReuseExisting {
        session: Option<TranscodeSessionRecord>,
    },
}
