use std::{collections::HashSet, path::PathBuf, sync::Arc};

use nako_core::{
    MediaSource, MediaSourceId, NakoError, NewTranscodeSession, Result, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionState,
};
use nako_playback::PlaybackDecision;
use nako_transcode::{
    CancellationToken, FfmpegExecutionPlanner, FfmpegHardwareAccelerationDetector, FfmpegHlsRunner,
    HardwareAccelerationDetector, HardwareAccelerationReport, HlsExecutionPlanRequest,
    HlsOutputPublicationPolicy, HlsPlaybackGeneration, HlsSubtitleBurnInPlan, TranscodeArtifactSet,
    TranscodeAudioOutputRequirement, TranscodeColorPipelineRequirement, TranscodeEngineAdapter,
    TranscodeEngineStartCommand, TranscodeEngineStartOutcome, TranscodeExecutionPolicy,
    TranscodeOutputConstraints, TranscodePipelinePlan, TranscodePipelinePlanner,
    TranscodePipelineReadiness, TranscodePipelineRequest, TranscodePipelineSourceFacts,
    TranscodeRequestIdentity, TranscodeResourceBudget, TranscodeRuntimeGuard,
    TranscodeTrackSelection, transcode_pipeline_readiness_without_selection,
};
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::config::NakoServerConfig;

use super::{
    HlsOutputLayout, HlsSourceDisposition, HlsSourceOutput, PlaybackSessionCancellationRegistry,
    PlaybackTraceContext,
    control::{hls_supersede_candidates, request_hls_session_supersede},
    map_hls_runner_error, path_exists, persist_session_failure,
    record_playback_session_finished_event,
    resource::{
        PlaybackResourceAdmissionPolicy, PlaybackResourceDemand, PlaybackResourcePermitSet,
        PlaybackRuntimeAdmission,
    },
};

#[derive(Clone, Debug)]
pub(super) struct HlsAppService {
    execution_planner: FfmpegExecutionPlanner,
    engine: FfmpegHlsRunner,
    hardware_policy: nako_transcode::HardwareAccelerationPolicy,
    pub(super) hardware_report: HardwareAccelerationReport,
    pipeline_plan: Option<TranscodePipelinePlan>,
    pipeline_readiness: TranscodePipelineReadiness,
    pipeline_planner: TranscodePipelinePlanner,
    cancellations: PlaybackSessionCancellationRegistry,
    in_flight: Arc<Mutex<HashSet<HlsRequestKey>>>,
}

impl HlsAppService {
    pub(super) fn new(
        config: &NakoServerConfig,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Result<Self> {
        let detector = FfmpegHardwareAccelerationDetector::new(&config.ffmpeg_path);
        Self::new_with_hardware_report(config, detector.detect(), cancellations)
    }

    #[cfg(test)]
    pub(super) fn new_with_hardware_detector(
        config: &NakoServerConfig,
        detector: &dyn HardwareAccelerationDetector,
    ) -> Result<Self> {
        Self::new_with_hardware_report(
            config,
            detector.detect(),
            PlaybackSessionCancellationRegistry::default(),
        )
    }

    pub(super) fn new_with_hardware_report(
        config: &NakoServerConfig,
        hardware_report: HardwareAccelerationReport,
        cancellations: PlaybackSessionCancellationRegistry,
    ) -> Result<Self> {
        let hardware_policy = config.transcode.hardware_policy();
        let pipeline_planner = TranscodePipelinePlanner::new();
        let pipeline_plan = pipeline_planner.plan_hls_single_variant(
            TranscodePipelineRequest::hls_single_variant(
                hardware_policy,
                TranscodeTrackSelection::default(),
                TranscodeOutputConstraints::default(),
            ),
            &hardware_report,
        );
        let pipeline_readiness = pipeline_plan.as_ref().map_or_else(
            |_| transcode_pipeline_readiness_without_selection(hardware_policy, &hardware_report),
            |plan| plan.readiness,
        );
        let guard = TranscodeRuntimeGuard::timeout_only(config.remux_timeout_ms);

        Ok(Self {
            execution_planner: FfmpegExecutionPlanner::new(&config.ffmpeg_path),
            engine: FfmpegHlsRunner::new_with_output_publication_policy(
                guard,
                HlsOutputPublicationPolicy::ServeWhileRunning,
            ),
            hardware_policy,
            hardware_report,
            pipeline_plan: pipeline_plan.ok(),
            pipeline_readiness,
            pipeline_planner,
            cancellations,
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        })
    }

    pub(super) fn execution_policy_for_hls(
        &self,
        track_selection: TranscodeTrackSelection,
        output_constraints: TranscodeOutputConstraints,
        audio_output: TranscodeAudioOutputRequirement,
        color_pipeline: TranscodeColorPipelineRequirement,
        source: Option<TranscodePipelineSourceFacts>,
    ) -> Result<TranscodeExecutionPolicy> {
        let mut request = TranscodePipelineRequest::hls_single_variant_with_audio_output(
            self.hardware_policy,
            track_selection,
            output_constraints,
            audio_output,
        )
        .with_color_pipeline(color_pipeline);
        if let Some(source) = source {
            request = request.with_source(source);
        }

        Ok(self
            .pipeline_planner
            .plan_hls_single_variant(request, &self.hardware_report)?
            .execution_policy())
    }

    #[must_use]
    pub(super) const fn pipeline_readiness(&self) -> TranscodePipelineReadiness {
        self.pipeline_readiness
    }

    #[must_use]
    pub(super) fn selected_hls_slots(&self, budget: TranscodeResourceBudget) -> usize {
        self.pipeline_plan
            .map_or(0, |plan| budget.slots_for(plan.selected_acceleration()))
    }

    pub(super) async fn run(
        &self,
        sessions: &dyn super::PlaybackRuntimeStore,
        source: MediaSource,
        decision: PlaybackDecision,
        input_path: PathBuf,
        layout: HlsOutputLayout,
        track_selection: TranscodeTrackSelection,
        subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
        execution_policy: TranscodeExecutionPolicy,
        playback_generation: HlsPlaybackGeneration,
        request_identity: TranscodeRequestIdentity,
        trace_context: Option<PlaybackTraceContext>,
        resource_admission: &PlaybackRuntimeAdmission,
        resource_demand: PlaybackResourceDemand,
        resource_permit: Option<PlaybackResourcePermitSet>,
    ) -> Result<HlsSourceOutput> {
        let key = HlsRequestKey {
            source_id: source.id,
            request_identity,
        };

        match self
            .reserve(
                sessions,
                &key,
                &layout,
                resource_admission,
                &resource_demand,
                resource_permit,
            )
            .await?
        {
            HlsRequestAdmission::ReuseExisting { session } => Ok(HlsSourceOutput {
                source,
                decision,
                playlist_path: layout.playlist_path,
                segment_dir: layout.output_dir,
                disposition: HlsSourceDisposition::ReusedExisting,
                session,
            }),
            HlsRequestAdmission::StartNew { session, permit } => {
                let result = self
                    .run_reserved(
                        sessions,
                        session,
                        source,
                        decision,
                        input_path,
                        layout,
                        track_selection,
                        subtitle_burn_in,
                        execution_policy,
                        playback_generation,
                        trace_context,
                        permit,
                    )
                    .await;
                self.release(&key).await;
                result
            }
            HlsRequestAdmission::SupersedeAndStart {
                session,
                superseded,
                permit,
            } => {
                debug!(
                    source_id = %source.id,
                    superseded_count = superseded.len(),
                    "starting hls request after superseding active sessions"
                );
                let result = self
                    .run_reserved(
                        sessions,
                        session,
                        source,
                        decision,
                        input_path,
                        layout,
                        track_selection,
                        subtitle_burn_in,
                        execution_policy,
                        playback_generation,
                        trace_context,
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
        key: &HlsRequestKey,
        layout: &HlsOutputLayout,
        resource_admission: &PlaybackRuntimeAdmission,
        resource_demand: &PlaybackResourceDemand,
        resource_permit: Option<PlaybackResourcePermitSet>,
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
            return Err(NakoError::Conflict {
                message: format!(
                    "hls request for source {} is already in progress in session {}",
                    key.source_id, active.id
                ),
            });
        }

        let supersede_candidates =
            hls_supersede_candidates(sessions, key.source_id, request_key.clone()).await?;

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
                return Err(NakoError::Conflict {
                    message: format!(
                        "hls request for source {} is already in progress",
                        key.source_id
                    ),
                });
            }
        }

        if resource_permit.is_none() && !supersede_candidates.is_empty() {
            if let Err(error) = resource_admission.ensure_capacity_for_policy(
                resource_demand,
                PlaybackResourceAdmissionPolicy::HlsSupersede,
            ) {
                self.release(key).await;
                return Err(error);
            }
        }

        let superseded = if supersede_candidates.is_empty() {
            Vec::new()
        } else {
            match request_hls_session_supersede(
                sessions,
                &self.cancellations,
                key.source_id,
                request_key.clone(),
                supersede_candidates,
            )
            .await
            {
                Ok(superseded) => superseded,
                Err(error) => {
                    self.release(key).await;
                    return Err(error);
                }
            }
        };

        let permit = match resource_permit {
            Some(permit) => permit,
            None => match resource_admission
                .acquire_for_policy(
                    resource_demand,
                    if superseded.is_empty() {
                        PlaybackResourceAdmissionPolicy::HlsStart
                    } else {
                        PlaybackResourceAdmissionPolicy::HlsSupersede
                    },
                )
                .await
            {
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
                kind: TranscodeSessionKind::HlsTranscode,
                request_key,
                output_path: TranscodeArtifactSet::hls(layout.artifacts.clone())
                    .primary_output_path()
                    .to_path_buf(),
                state: TranscodeSessionState::Planned,
            })
            .await;

        match session {
            Ok(session) if superseded.is_empty() => {
                Ok(HlsRequestAdmission::StartNew { session, permit })
            }
            Ok(session) => Ok(HlsRequestAdmission::SupersedeAndStart {
                session,
                superseded,
                permit,
            }),
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
        layout: HlsOutputLayout,
        track_selection: TranscodeTrackSelection,
        subtitle_burn_in: Option<HlsSubtitleBurnInPlan>,
        execution_policy: TranscodeExecutionPolicy,
        playback_generation: HlsPlaybackGeneration,
        trace_context: Option<PlaybackTraceContext>,
        _permit: PlaybackResourcePermitSet,
    ) -> Result<HlsSourceOutput> {
        let session_id = persisted_session.id;
        let cancel = CancellationToken::new();
        let _cancel_handle = self.cancellations.register(session_id, cancel.clone());

        let execution = match self.execution_planner.plan_hls_with_id(
            session_id,
            HlsExecutionPlanRequest {
                source_id: source.id,
                input_path,
                playback_generation,
                artifacts: layout.artifacts.clone(),
                segment_time_seconds: 6,
                track_selection,
                subtitle_burn_in,
                execution_policy,
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
            .map_err(map_hls_runner_error);

        drop(_cancel_handle);

        if let Ok(outcome) = &run_result {
            let metrics = outcome.runtime_metrics();
            if !metrics.is_empty() {
                if let Err(error) = sessions
                    .update_transcode_session_runtime_metrics(session_id, metrics.clone())
                    .await
                {
                    warn!(
                        error = %error,
                        transcode_session_id = %session_id,
                        "failed to persist hls runtime metrics"
                    );
                }
            }
        }

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
                record_playback_session_finished_event(sessions, &session, trace_context.as_ref())
                    .await;

                Ok(HlsSourceOutput {
                    source,
                    decision,
                    playlist_path: layout.playlist_path,
                    segment_dir: layout.output_dir,
                    disposition: HlsSourceDisposition::Finished,
                    session,
                })
            }
            Ok(TranscodeEngineStartOutcome::Cancelled { .. }) => {
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

#[derive(Debug)]
enum HlsRequestAdmission {
    StartNew {
        session: TranscodeSessionRecord,
        permit: PlaybackResourcePermitSet,
    },
    SupersedeAndStart {
        session: TranscodeSessionRecord,
        superseded: Vec<TranscodeSessionRecord>,
        permit: PlaybackResourcePermitSet,
    },
    ReuseExisting {
        session: TranscodeSessionRecord,
    },
}
