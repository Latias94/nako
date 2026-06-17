use nako_core::{
    MediaSource, NakoError, PlaybackSessionMode, Result, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState,
};
use nako_playback::{
    EffectivePlaybackPolicy, PlaybackDecision, PlaybackPlanningRequest, PlaybackPreferenceContext,
    PlaybackTargetProfile, PlaybackTranscodeContainer,
};
use nako_transcode::{
    HlsPlaybackGeneration, TranscodeExecutionPolicy, TranscodePipelinePlanner,
    TranscodeRequestIdentity, TranscodeTrackSelection,
};
use tracing::warn;

use crate::app::storage::LibraryStorageBackend;

use super::{
    HlsOutputLayout, HlsPlaylistOutput, HlsPlaylistPlaybackOutput, HlsPlaylistPlaybackRequest,
    HlsPlaylistSessionRequest, HlsSourceDisposition, HlsSourceOutput, HlsSourceRequest,
    PlaybackAppService, PlaybackRuntimeStore, PlaybackTraceContext, StartPlaybackSessionRequest,
    control::{hls_supersede_candidates, request_hls_session_supersede},
    hls_artifact::hls_artifact_manifest_for_session,
    input::FfmpegSourceInputScope,
    playlist::HlsPlaylistSessionBinding,
    playlist::HlsPlaylistUrlDecoration,
    playlist::author_hls_session_playlist,
    resource::PlaybackResourceAdmissionPolicy,
    resource::PlaybackResourceDemand,
    resource::PlaybackResourcePermitSet,
    runtime_session::{
        PlaybackTranscodeRuntimeBinding, TranscodeRuntimeSessionKey, cancelled_transcode_error,
        failed_transcode_error, find_active_runtime_session,
        find_finished_runtime_session_with_output, find_latest_runtime_session,
        get_bound_transcode_session, link_playback_session_to_transcode,
        missing_finished_output_error, missing_playback_transcode_error,
        start_linked_playback_session,
    },
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
    trace_context: Option<PlaybackTraceContext>,
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

    fn source_output(
        self,
        disposition: HlsSourceDisposition,
        session: TranscodeSessionRecord,
    ) -> HlsSourceOutput {
        HlsSourceOutput {
            source: self.source,
            decision: self.decision,
            playlist_path: self.layout.playlist_path,
            segment_dir: self.layout.output_dir,
            disposition,
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

enum HlsSourcePreStagingAdmission {
    ReuseExisting {
        session: TranscodeSessionRecord,
    },
    StartNew {
        permit: Option<PlaybackResourcePermitSet>,
    },
}

pub(super) async fn hls_source_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<HlsSourceOutput> {
    let context = hls_source_context(app, &request, effective_policy.into(), None).await?;
    run_hls_source_context(app, context).await
}

pub(super) async fn hls_playlist_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    trace_context: Option<PlaybackTraceContext>,
) -> Result<HlsPlaylistOutput> {
    let output =
        start_hls_playlist_with_policy(app, request, effective_policy, trace_context).await?;

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

pub(super) async fn hls_playlist_playback(
    app: &PlaybackAppService,
    request: HlsPlaylistPlaybackRequest,
) -> Result<HlsPlaylistPlaybackOutput> {
    let source = app.get_source_or_not_found(request.source_id).await?;
    let effective_policy = app
        .admit_playback_session_start(&request.principal, &source, None)
        .await?;
    let playlist = hls_playlist_with_policy(
        app,
        HlsSourceRequest {
            source_id: request.source_id,
            client: request.client.clone(),
            preferences: request.preferences.clone(),
            playback_generation: request.playback_generation,
        },
        effective_policy,
        request.trace_context.clone(),
    )
    .await?;
    let playback_session = start_linked_playback_session(
        app,
        StartPlaybackSessionRequest {
            principal: request.principal,
            source_id: request.source_id,
            mode: PlaybackSessionMode::Hls,
            client: Some(request.client.clone()),
        },
        playlist.session.id,
    )
    .await?;
    app.cancel_superseded_hls_playback_sessions(
        request.source_id,
        playlist.session.id,
        playback_session.id,
    )
    .await?;
    let body = app
        .hls_artifacts
        .read_playback_playlist(
            &playlist.session,
            playback_session.id,
            request.transport_query.as_deref(),
        )
        .await?;

    Ok(HlsPlaylistPlaybackOutput {
        session: playback_session,
        body,
    })
}

pub(super) async fn hls_playlist_for_playback_session(
    app: &PlaybackAppService,
    request: HlsPlaylistSessionRequest,
) -> Result<HlsPlaylistPlaybackOutput> {
    app.ensure_source_play_access(&request.principal, request.source_id)
        .await?;
    let mut playback_session = app
        .existing_playback_session_for_media_request(
            &request.principal,
            request.playback_session_id,
            request.source_id,
            PlaybackSessionMode::Hls,
        )
        .await?;
    if playback_session.transcode_session_id.is_none() {
        let client =
            PlaybackAppService::client_capabilities_for_playback_session(&playback_session)?;
        let effective_policy = app
            .effective_playback_policy_for_playable_source_id(&request.principal, request.source_id)
            .await?;
        let playlist = hls_playlist_with_policy(
            app,
            HlsSourceRequest {
                source_id: request.source_id,
                client,
                preferences: PlaybackPreferenceContext::default(),
                playback_generation: request.playback_generation,
            },
            effective_policy,
            request.trace_context.clone(),
        )
        .await?;
        playback_session =
            link_playback_session_to_transcode(app, playback_session.id, playlist.session.id)
                .await?;
        app.cancel_superseded_hls_playback_sessions(
            request.source_id,
            playlist.session.id,
            playback_session.id,
        )
        .await?;
        let body = app
            .hls_artifacts
            .read_playback_playlist(
                &playlist.session,
                playback_session.id,
                request.transport_query.as_deref(),
            )
            .await?;

        return Ok(HlsPlaylistPlaybackOutput {
            session: playback_session,
            body,
        });
    }

    let transcode_session_id = playback_session
        .transcode_session_id
        .ok_or_else(|| missing_playback_transcode_error(playback_session.id, "hls artifact"))?;
    let transcode = get_bound_transcode_session(
        app,
        playback_session.id,
        request.source_id,
        transcode_session_id,
        PlaybackTranscodeRuntimeBinding::HLS,
    )
    .await?;
    let body = app
        .hls_artifacts
        .read_playback_playlist(
            &transcode,
            playback_session.id,
            request.transport_query.as_deref(),
        )
        .await?;

    Ok(HlsPlaylistPlaybackOutput {
        session: playback_session,
        body,
    })
}

async fn hls_source_context(
    app: &PlaybackAppService,
    request: &HlsSourceRequest,
    effective_policy: Option<EffectivePlaybackPolicy>,
    trace_context: Option<PlaybackTraceContext>,
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
        trace_context,
        remote_input,
    })
}

async fn run_hls_source_context(
    app: &PlaybackAppService,
    context: HlsSourceContext,
) -> Result<HlsSourceOutput> {
    let resource_demand = context.resource_demand();
    let resource_permit =
        match prepare_hls_source_before_input_staging(app, &context, &resource_demand).await? {
            HlsSourcePreStagingAdmission::ReuseExisting { session } => {
                return Ok(context.source_output(HlsSourceDisposition::ReusedExisting, session));
            }
            HlsSourcePreStagingAdmission::StartNew { permit } => permit,
        };
    let input = app
        .input
        .source_input_scope(&context.source, &context.uri, &context.backend)
        .await?;
    run_hls_source_context_with_input_scope(app, context, input, resource_permit).await
}

async fn run_hls_source_context_with_input_scope(
    app: &PlaybackAppService,
    context: HlsSourceContext,
    input: FfmpegSourceInputScope,
    resource_permit: Option<PlaybackResourcePermitSet>,
) -> Result<HlsSourceOutput> {
    let resource_demand = context.resource_demand();
    let input_service = app.input.clone();
    input_service
        .with_prepared_source_input(input, |input_path| async move {
            app.hls
                .run(
                    app.runtime_store.as_ref(),
                    context.source,
                    context.decision,
                    input_path,
                    context.layout,
                    context.track_selection,
                    context.subtitle_burn_in,
                    context.execution_policy,
                    context.playback_generation,
                    context.request_identity,
                    context.trace_context,
                    &app.resource_admission,
                    resource_demand,
                    resource_permit,
                )
                .await
        })
        .await
}

async fn prepare_hls_source_before_input_staging(
    app: &PlaybackAppService,
    context: &HlsSourceContext,
    resource_demand: &PlaybackResourceDemand,
) -> Result<HlsSourcePreStagingAdmission> {
    let key = TranscodeRuntimeSessionKey::new(
        context.source.id,
        TranscodeSessionKind::HlsTranscode,
        &context.request_key,
    );
    if let Some(active) = find_active_runtime_session(app.runtime_store.as_ref(), key).await? {
        return Err(NakoError::Conflict {
            message: format!(
                "hls request for source {} is already in progress in session {}",
                context.source.id, active.id
            ),
        });
    }

    let supersede_candidates = hls_supersede_candidates(
        app.runtime_store.as_ref(),
        context.source.id,
        context.request_key.clone(),
    )
    .await?;

    if let Some(latest) = find_finished_runtime_session_with_output(
        app.runtime_store.as_ref(),
        key,
        &context.layout.playlist_path,
    )
    .await?
    {
        return Ok(HlsSourcePreStagingAdmission::ReuseExisting { session: latest });
    }

    let permit = if supersede_candidates.is_empty() {
        Some(acquire_hls_start_permit(app, resource_demand).await?)
    } else {
        app.resource_admission.ensure_capacity_for_policy(
            resource_demand,
            PlaybackResourceAdmissionPolicy::HlsSupersede,
        )?;
        None
    };

    Ok(HlsSourcePreStagingAdmission::StartNew { permit })
}

async fn start_hls_playlist_with_policy(
    app: &PlaybackAppService,
    request: HlsSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
    trace_context: Option<PlaybackTraceContext>,
) -> Result<HlsPlaylistReadyOutput> {
    let effective_policy = effective_policy.into();
    let context =
        hls_source_context(app, &request, effective_policy.clone(), trace_context).await?;
    let resource_demand = context.resource_demand();

    let key = TranscodeRuntimeSessionKey::new(
        context.source.id,
        TranscodeSessionKind::HlsTranscode,
        &context.request_key,
    );
    if let Some(active) = find_active_runtime_session(app.runtime_store.as_ref(), key).await? {
        return wait_for_hls_playlist_ready_context(app, context, active.id).await;
    }

    if let Some(latest) = find_finished_runtime_session_with_output(
        app.runtime_store.as_ref(),
        key,
        &context.layout.playlist_path,
    )
    .await?
    {
        return Ok(context.playlist_ready(latest));
    }

    let supersede_candidates = hls_supersede_candidates(
        app.runtime_store.as_ref(),
        context.source.id,
        context.request_key.clone(),
    )
    .await?;
    let resource_permit = if supersede_candidates.is_empty() {
        acquire_hls_start_permit(app, &resource_demand).await?
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
        app.resource_admission
            .acquire_for_policy(
                &resource_demand,
                PlaybackResourceAdmissionPolicy::HlsSupersede,
            )
            .await?
    };
    let input = app
        .input
        .source_input_scope(&context.source, &context.uri, &context.backend)
        .await?;
    let wait_context = context.clone();
    let task_app = app.clone();
    app.runtime
        .spawn("playback_hls_start", "playback.hls", async move {
            if let Err(error) = run_hls_source_context_with_input_scope(
                &task_app,
                context,
                input,
                Some(resource_permit),
            )
            .await
            {
                warn!(error = %error, "background hls start failed");
            }
        });

    wait_for_hls_playlist_ready_context(app, wait_context, None).await
}

async fn acquire_hls_start_permit(
    app: &PlaybackAppService,
    resource_demand: &PlaybackResourceDemand,
) -> Result<PlaybackResourcePermitSet> {
    app.resource_admission
        .ensure_capacity_for_policy(resource_demand, PlaybackResourceAdmissionPolicy::HlsStart)?;
    app.resource_admission
        .acquire_for_policy(resource_demand, PlaybackResourceAdmissionPolicy::HlsStart)
        .await
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
            find_latest_runtime_session(
                app.runtime_store.as_ref(),
                TranscodeRuntimeSessionKey::new(
                    context.source.id,
                    TranscodeSessionKind::HlsTranscode,
                    &context.request_key,
                ),
            )
            .await?
        };

        if let Some(session) = session {
            match session.state {
                TranscodeSessionState::Finished => {
                    if !super::path_exists(&context.layout.playlist_path)? {
                        return Err(missing_finished_output_error(
                            &context.layout.playlist_path,
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
                    return Err(cancelled_transcode_error(
                        "ffmpeg_hls",
                        "hls session was cancelled",
                    ));
                }
                TranscodeSessionState::Failed => {
                    return Err(failed_transcode_error(
                        session,
                        "ffmpeg_hls",
                        "hls runner failed",
                    ));
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
