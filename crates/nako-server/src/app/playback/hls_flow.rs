use nako_core::{
    MediaSource, NakoError, Result, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState,
};
use nako_playback::{
    EffectivePlaybackPolicy, PlaybackDecision, PlaybackPlanningRequest, PlaybackTargetProfile,
    PlaybackTranscodeContainer,
};
use nako_transcode::{
    HlsPlaybackGeneration, TranscodeExecutionPolicy, TranscodePipelinePlanner,
    TranscodeRequestIdentity, TranscodeTrackSelection,
};
use tracing::warn;

use crate::app::storage::LibraryStorageBackend;

use super::{
    HlsOutputLayout, HlsPlaylistOutput, HlsSourceOutput, HlsSourceRequest, PlaybackAppService,
    PlaybackRuntimeStore,
    control::{hls_supersede_candidates, request_hls_session_supersede},
    hls_artifact::hls_artifact_manifest_for_session,
    input::FfmpegSourceInput,
    paths::path_exists,
    playlist::HlsPlaylistSessionBinding,
    playlist::HlsPlaylistUrlDecoration,
    playlist::author_hls_session_playlist,
    resource::PlaybackResourceAdmissionPolicy,
    resource::PlaybackResourceDemand,
    resource::PlaybackResourcePermitSet,
    selection::hls_runtime_plan_request,
    staging_policy::HlsStagingPolicy,
};
use super::{
    default_playback_policy_for_source, ensure_playback_decision_allowed,
    playback_selection_context, playback_target_for_client,
};

#[derive(Clone, Debug)]
struct HlsSourceContext {
    source: MediaSource,
    decision: PlaybackDecision,
    uri: nako_vfs::StorageUri,
    backend: std::sync::Arc<LibraryStorageBackend>,
    layout: HlsOutputLayout,
    track_selection: TranscodeTrackSelection,
    subtitle_burn_in: Option<nako_transcode::HlsSubtitleBurnInPlan>,
    execution_policy: TranscodeExecutionPolicy,
    playback_generation: HlsPlaybackGeneration,
    request_identity: TranscodeRequestIdentity,
    request_key: String,
    remote_input: bool,
}

impl HlsSourceContext {
    fn resource_demand(&self) -> PlaybackResourceDemand {
        PlaybackResourceDemand::hls(self.remote_input, self.execution_policy)
    }

    fn playlist_ready(self, session: TranscodeSessionRecord) -> HlsPlaylistReadyOutput {
        HlsPlaylistReadyOutput {
            source: self.source,
            decision: self.decision,
            playlist_path: self.layout.playlist_path,
            session,
        }
    }
}

#[derive(Clone, Debug)]
struct HlsPlaylistReadyOutput {
    source: MediaSource,
    decision: PlaybackDecision,
    playlist_path: std::path::PathBuf,
    session: TranscodeSessionRecord,
}

pub(super) async fn hls_source_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<HlsSourceOutput> {
    let context = hls_source_context(app, &request, effective_policy.into()).await?;
    run_hls_source_context(app, context).await
}

pub(super) async fn hls_playlist_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<HlsPlaylistOutput> {
    let output = start_hls_playlist_with_policy(app, request, effective_policy).await?;

    let body = tokio::fs::read_to_string(&output.playlist_path)
        .await
        .map_err(|err| {
            NakoError::storage_io(
                output.playlist_path.display().to_string(),
                format!("failed to read hls playlist: {err}"),
            )
        })?;

    let manifest = hls_artifact_manifest_for_session(&output.session)?;
    let body = author_hls_session_playlist(
        &body,
        &manifest,
        HlsPlaylistSessionBinding::Transcode(output.session.id),
        HlsPlaylistUrlDecoration::none(),
    )?;

    Ok(HlsPlaylistOutput {
        source: output.source,
        decision: output.decision,
        body,
        session: output.session,
    })
}

async fn hls_source_context(
    app: &PlaybackAppService,
    request: &HlsSourceRequest,
    effective_policy: Option<EffectivePlaybackPolicy>,
) -> Result<HlsSourceContext> {
    let source = app.get_source_or_not_found(request.source_id).await?;
    let probe =
        PlaybackRuntimeStore::get_media_probe(app.runtime_store.as_ref(), source.id).await?;
    let (uri, backend) = app.storage_backend_for_media_source(&source).await?;
    let mut context = playback_selection_context(&uri, backend.as_ref()).await;
    context.preferences = request.preferences.clone();
    context.preferences.transcode_output_container = Some(PlaybackTranscodeContainer::Hls);
    let target = playback_target_for_client(request.client.clone());
    let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
    let remote_input = target_profile.storage.remote;
    let effective_policy =
        effective_policy.unwrap_or_else(|| default_playback_policy_for_source(&source));
    let decision = app.planner.plan(PlaybackPlanningRequest {
        source: &source,
        probe: probe.as_ref(),
        target: &target,
        effective_policy: &effective_policy,
        context,
    });
    ensure_playback_decision_allowed(&decision)?;
    let hls_runtime_request = hls_runtime_plan_request(
        &source,
        &decision,
        &target_profile,
        app.config.transcode.hardware_policy(),
        request.playback_generation,
        remote_input,
        probe.clone(),
    )?;
    let hls_runtime = TranscodePipelinePlanner::new()
        .plan_hls_runtime(hls_runtime_request, &app.hls.hardware_report)?;
    let execution_policy = hls_runtime.execution_policy;
    let request_identity = hls_runtime.request_identity.clone();
    let staging = HlsStagingPolicy::new(app.config.remux_staging_root.join("hls"))?;
    let layout = staging.layout_for_runtime_plan(source.id, &hls_runtime)?;

    Ok(HlsSourceContext {
        source,
        decision,
        uri,
        backend,
        layout,
        track_selection: hls_runtime.track_selection,
        subtitle_burn_in: hls_runtime.subtitle_burn_in,
        execution_policy,
        playback_generation: request.playback_generation,
        request_identity: request_identity.clone(),
        request_key: request_identity.persisted_request_key().to_owned(),
        remote_input,
    })
}

async fn run_hls_source_context(
    app: &PlaybackAppService,
    context: HlsSourceContext,
) -> Result<HlsSourceOutput> {
    let input = app
        .input
        .source_input_for_ffmpeg(&context.source, &context.uri, &context.backend)
        .await?;
    run_hls_source_context_with_input(app, context, input, None).await
}

async fn run_hls_source_context_with_input(
    app: &PlaybackAppService,
    context: HlsSourceContext,
    input: FfmpegSourceInput,
    resource_permit: Option<PlaybackResourcePermitSet>,
) -> Result<HlsSourceOutput> {
    let resource_demand = context.resource_demand();
    let result = app
        .hls
        .run(
            app.runtime_store.as_ref(),
            context.source,
            context.decision,
            input.path.clone(),
            context.layout,
            context.track_selection,
            context.subtitle_burn_in,
            context.execution_policy,
            context.playback_generation,
            context.request_identity,
            &app.resource_admission,
            resource_demand,
            resource_permit,
        )
        .await;
    match result {
        Ok(output) => {
            app.input.release_source_input(input).await?;
            Ok(output)
        }
        Err(err) => {
            if let Err(release_err) = app.input.release_source_input(input).await {
                warn!(
                    error = %release_err,
                    "failed to release hls staging lease after error"
                );
            }
            Err(err)
        }
    }
}

async fn start_hls_playlist_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<HlsPlaylistReadyOutput> {
    let effective_policy = effective_policy.into();
    let context = hls_source_context(app, &request, effective_policy.clone()).await?;
    let resource_demand = context.resource_demand();

    if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
        app.runtime_store.as_ref(),
        context.source.id,
        TranscodeSessionKind::HlsTranscode,
        &context.request_key,
    )
    .await?
    {
        return wait_for_hls_playlist_ready_context(app, context, active.id).await;
    }

    if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
        app.runtime_store.as_ref(),
        context.source.id,
        TranscodeSessionKind::HlsTranscode,
        &context.request_key,
    )
    .await?
    {
        if latest.state == TranscodeSessionState::Finished
            && latest.output_path == context.layout.playlist_path
            && path_exists(&context.layout.playlist_path)?
        {
            return Ok(context.playlist_ready(latest));
        }
    }

    let supersede_candidates = hls_supersede_candidates(
        app.runtime_store.as_ref(),
        context.source.id,
        context.request_key.clone(),
    )
    .await?;
    let resource_permit = if supersede_candidates.is_empty() {
        None
    } else {
        app.resource_admission.ensure_capacity_for_policy(
            &resource_demand,
            PlaybackResourceAdmissionPolicy::HlsSupersede,
        )?;
        let _ = request_hls_session_supersede(
            app.runtime_store.as_ref(),
            &app.cancellations,
            context.source.id,
            context.request_key.clone(),
            supersede_candidates,
        )
        .await?;
        Some(
            app.resource_admission
                .acquire_for_policy(
                    &resource_demand,
                    PlaybackResourceAdmissionPolicy::HlsSupersede,
                )
                .await?,
        )
    };
    let input = app
        .input
        .source_input_for_ffmpeg(&context.source, &context.uri, &context.backend)
        .await?;
    let resource_permit = match resource_permit {
        Some(permit) => permit,
        None => match app
            .resource_admission
            .acquire_for_policy(&resource_demand, PlaybackResourceAdmissionPolicy::Immediate)
            .await
        {
            Ok(permit) => permit,
            Err(error) => {
                if let Err(release_error) = app.input.release_source_input(input).await {
                    warn!(
                        error = %release_error,
                        "failed to release HLS source input after playback resource admission rejection",
                    );
                }
                return Err(error);
            }
        },
    };
    let wait_context = context.clone();
    let task_app = app.clone();
    app.runtime
        .spawn("playback_hls_start", "playback.hls", async move {
            if let Err(error) =
                run_hls_source_context_with_input(&task_app, context, input, Some(resource_permit))
                    .await
            {
                warn!(error = %error, "background hls start failed");
            }
        });

    wait_for_hls_playlist_ready_context(app, wait_context, None).await
}

async fn wait_for_hls_playlist_ready_context(
    app: &PlaybackAppService,
    context: HlsSourceContext,
    preferred_session_id: impl Into<Option<TranscodeSessionId>>,
) -> Result<HlsPlaylistReadyOutput> {
    let preferred_session_id = preferred_session_id.into();
    let deadline = tokio::time::Instant::now()
        + std::time::Duration::from_millis(app.config.remux_timeout_ms.max(1));

    loop {
        let session = if let Some(session_id) = preferred_session_id {
            Some(app.get_transcode_session(session_id).await?)
        } else {
            PlaybackRuntimeStore::find_latest_transcode_session(
                app.runtime_store.as_ref(),
                context.source.id,
                TranscodeSessionKind::HlsTranscode,
                &context.request_key,
            )
            .await?
        };

        if let Some(session) = session {
            match session.state {
                TranscodeSessionState::Finished => {
                    if !path_exists(&context.layout.playlist_path)? {
                        return Err(NakoError::storage_io(
                            context.layout.playlist_path.display().to_string(),
                            "finished hls session playlist is missing",
                        ));
                    }

                    return Ok(context.playlist_ready(session));
                }
                TranscodeSessionState::Running => {
                    if app.hls_artifacts.playlist_is_ready(&session).await? {
                        return Ok(context.playlist_ready(session));
                    }
                }
                TranscodeSessionState::Cancelled => {
                    return Err(NakoError::Provider {
                        provider: "ffmpeg_hls".to_owned(),
                        message: "hls session was cancelled".to_owned(),
                    });
                }
                TranscodeSessionState::Failed => {
                    return Err(NakoError::Provider {
                        provider: "ffmpeg_hls".to_owned(),
                        message: session
                            .failure_message
                            .unwrap_or_else(|| "hls runner failed".to_owned()),
                    });
                }
                TranscodeSessionState::Planned
                | TranscodeSessionState::Starting
                | TranscodeSessionState::CancelRequested => {}
            }
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(NakoError::Conflict {
                message: format!(
                    "hls playlist for source {} did not become ready before timeout",
                    context.source.id
                ),
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}
