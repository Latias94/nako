use std::{collections::HashSet, path::PathBuf, sync::Arc};

use taru_core::{
    MediaSource, MediaSourceId, NewTranscodeSession, Result, TaruError, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionRepository,
    TranscodeSessionState,
};
use taru_db::TaruDatabase;
use taru_streaming::PlaybackDecision;
use taru_transcode::{
    CancellationToken, FfmpegCommandBuilder, FfmpegHardwareAccelerationDetector, FfmpegHlsRunner,
    FfmpegOverwritePolicy, HardwareAccelerationDetector, HardwareAccelerationReport,
    HardwareAccelerationSelection, HlsRequest, HlsRunOutcome, RemuxRuntimeGuard,
    RemuxRuntimeLimits, TranscodeRequestIdentity, TranscodeSessionManager,
    select_hardware_acceleration,
};
use tokio::sync::Mutex;

use crate::config::TaruServerConfig;

use super::{
    HlsOutputLayout, HlsSourceDisposition, HlsSourceOutput, PlaybackSessionCancellationRegistry,
    map_hls_runner_error, path_exists, persist_session_failure,
    record_playback_session_finished_event,
};

#[derive(Clone, Debug)]
pub(super) struct HlsAppService {
    builder: FfmpegCommandBuilder,
    runner: FfmpegHlsRunner,
    pub(super) hardware_report: HardwareAccelerationReport,
    pub(super) hardware_selection: HardwareAccelerationSelection,
    cancellations: PlaybackSessionCancellationRegistry,
    in_flight: Arc<Mutex<HashSet<HlsRequestKey>>>,
}

impl HlsAppService {
    pub(super) fn new(
        config: &TaruServerConfig,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Result<Self> {
        let detector = FfmpegHardwareAccelerationDetector::new(&config.ffmpeg_path);
        Self::new_with_hardware_report(config, detector.detect(), cancellations)
    }

    #[cfg(test)]
    pub(super) fn new_with_hardware_detector(
        config: &TaruServerConfig,
        detector: &dyn HardwareAccelerationDetector,
    ) -> Result<Self> {
        Self::new_with_hardware_report(
            config,
            detector.detect(),
            PlaybackSessionCancellationRegistry::default(),
        )
    }

    pub(super) fn new_with_hardware_report(
        config: &TaruServerConfig,
        hardware_report: HardwareAccelerationReport,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Result<Self> {
        let hardware_policy = config.transcode.hardware_policy();
        let hardware_selection = select_hardware_acceleration(hardware_policy, &hardware_report)?;
        let transcode_budget = config.transcode.resource_budget();
        let guard = RemuxRuntimeGuard::new(RemuxRuntimeLimits {
            max_concurrent_sessions: transcode_budget.slots_for(hardware_selection.acceleration),
            timeout_ms: config.remux_timeout_ms,
        });

        Ok(Self {
            builder: FfmpegCommandBuilder::new(&config.ffmpeg_path),
            runner: FfmpegHlsRunner::new(guard),
            hardware_report,
            hardware_selection,
            cancellations,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub(super) async fn run(
        &self,
        sessions: &TaruDatabase,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        layout: HlsOutputLayout,
        request_identity: TranscodeRequestIdentity,
    ) -> Result<HlsSourceOutput> {
        let key = HlsRequestKey {
            source_id: source.id,
            request_identity,
        };

        match self.reserve(sessions, &key, &layout).await? {
            HlsRequestAdmission::ReuseExisting { session } => Ok(HlsSourceOutput {
                source,
                decision,
                playlist_path: layout.playlist_path,
                segment_dir: layout.output_dir,
                disposition: HlsSourceDisposition::ReusedExisting,
                session,
            }),
            HlsRequestAdmission::Run { session } => {
                let result = self
                    .run_reserved(sessions, session, source, decision, input_path, layout)
                    .await;
                self.release(&key).await;
                result
            }
        }
    }

    async fn reserve(
        &self,
        sessions: &TaruDatabase,
        key: &HlsRequestKey,
        layout: &HlsOutputLayout,
    ) -> Result<HlsRequestAdmission> {
        let request_key = key.persisted_request_key();
        if let Some(active) = sessions
            .find_active_transcode_session(
                key.source_id,
                TranscodeSessionKind::HlsTranscode,
                &request_key,
            )
            .await?
        {
            return Err(TaruError::Conflict {
                message: format!(
                    "hls request for source {} is already in progress in session {}",
                    key.source_id, active.id
                ),
            });
        }

        let latest = sessions
            .find_latest_transcode_session(
                key.source_id,
                TranscodeSessionKind::HlsTranscode,
                &request_key,
            )
            .await?;
        let playlist_exists = path_exists(&layout.playlist_path)?;

        if let Some(session) = latest.as_ref() {
            if session.state == TranscodeSessionState::Finished
                && session.output_path == layout.playlist_path
                && playlist_exists
            {
                return Ok(HlsRequestAdmission::ReuseExisting {
                    session: session.clone(),
                });
            }
        }

        {
            let mut in_flight = self.in_flight.lock().await;
            if !in_flight.insert(key.clone()) {
                return Err(TaruError::Conflict {
                    message: format!(
                        "hls request for source {} is already in progress",
                        key.source_id
                    ),
                });
            }
        }

        let session = sessions
            .create_transcode_session(NewTranscodeSession {
                id: TranscodeSessionId::new(),
                source_id: key.source_id,
                kind: TranscodeSessionKind::HlsTranscode,
                request_key,
                output_path: layout.playlist_path.clone(),
                state: TranscodeSessionState::Planned,
            })
            .await;

        match session {
            Ok(session) => Ok(HlsRequestAdmission::Run { session }),
            Err(error) => {
                self.release(key).await;
                Err(error)
            }
        }
    }

    async fn run_reserved(
        &self,
        sessions: &TaruDatabase,
        persisted_session: TranscodeSessionRecord,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        layout: HlsOutputLayout,
    ) -> Result<HlsSourceOutput> {
        let session_id = persisted_session.id;
        let cancel = CancellationToken::new();
        let _cancel_handle = self.cancellations.register(session_id, cancel.clone());
        let mut manager = TranscodeSessionManager::new();

        if let Err(error) = manager.plan_hls_with_id(
            session_id,
            HlsRequest {
                source_id: source.id,
                input_path,
                output_dir: layout.output_dir.clone(),
                playlist_path: layout.playlist_path.clone(),
                segment_pattern: layout.segment_pattern.clone(),
                segment_time_seconds: 6,
                hardware_acceleration: self.hardware_selection.acceleration,
                overwrite: FfmpegOverwritePolicy::Allow,
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
            .map_err(map_hls_runner_error);

        drop(_cancel_handle);

        match run_result {
            Ok(HlsRunOutcome::Finished { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Finished,
                        None,
                        None,
                    )
                    .await?;
                record_playback_session_finished_event(sessions, &session).await;

                Ok(HlsSourceOutput {
                    source,
                    decision,
                    playlist_path: layout.playlist_path,
                    segment_dir: layout.output_dir,
                    disposition: HlsSourceDisposition::Finished,
                    session,
                })
            }
            Ok(HlsRunOutcome::Cancelled { .. }) => {
                let session = sessions
                    .set_transcode_session_state(
                        session_id,
                        TranscodeSessionState::Cancelled,
                        Some(TranscodeFailureCategory::Cancelled),
                        Some("hls session was cancelled".to_owned()),
                    )
                    .await?;

                Ok(HlsSourceOutput {
                    source,
                    decision,
                    playlist_path: layout.playlist_path,
                    segment_dir: layout.output_dir,
                    disposition: HlsSourceDisposition::Cancelled,
                    session,
                })
            }
            Err(error) => {
                persist_session_failure(sessions, session_id, &error).await;
                Err(error)
            }
        }
    }

    async fn release(&self, key: &HlsRequestKey) {
        self.in_flight.lock().await.remove(key);
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct HlsRequestKey {
    source_id: MediaSourceId,
    request_identity: TranscodeRequestIdentity,
}

impl HlsRequestKey {
    fn persisted_request_key(&self) -> String {
        self.request_identity.persisted_request_key().to_owned()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum HlsRequestAdmission {
    Run { session: TranscodeSessionRecord },
    ReuseExisting { session: TranscodeSessionRecord },
}
