use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use nako_core::{
    MediaSource, NakoError, Result, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionRecord, TranscodeSessionState,
};
use nako_playback::{
    EffectivePlaybackPolicy, PlaybackDecision, PlaybackPlanningRequest, PlaybackTargetProfile,
};
use nako_streaming::{DirectPlayRangeRequest, DirectPlayResponsePlan};
use nako_transcode::{
    PlaybackRemuxProfileRequest, RemuxContainer, TranscodeRequestIdentity, TranscodeSourceIdentity,
    build_playback_remux_profile,
};
use nako_vfs::StorageUri;
use tracing::warn;

use crate::{
    app::storage::LibraryStorageBackend,
    playback_mapping::{
        playback_track_selection_to_transcode, transcode_remux_container_to_playback,
    },
};

use super::{
    PlaybackAppService, PlaybackRuntimeStore, RemuxPlaybackPreflightOutput,
    RemuxPlaybackPreflightRequest, RemuxPlaybackSessionStreamRequest, RemuxPlaybackStreamOutput,
    RemuxPlaybackStreamRequest, RemuxRequestKey, RemuxSessionStart, RemuxSourceDisposition,
    RemuxSourceOutput, RemuxSourceRequest, StartPlaybackSessionRequest,
    default_playback_policy_for_source, ensure_playback_decision_allowed, path_exists,
    playback_target_for_client,
    resource::PlaybackResourcePermitSet,
    selection::{playback_selection_context, remux_output_container},
    staging_policy::RemuxStagingPolicy,
};
use nako_core::PlaybackSessionMode;

pub(super) async fn remux_playback_stream(
    app: &PlaybackAppService,
    request: RemuxPlaybackStreamRequest,
) -> Result<RemuxPlaybackStreamOutput> {
    let effective_policy = app
        .effective_playback_policy_for_source_id(&request.principal, request.source_id)
        .await?;
    let remux_request = RemuxSourceRequest {
        source_id: request.source_id,
        client: request.client.clone(),
        output_container: request.output_container,
    };
    let remux_start = start_remux_source_with_policy(app, remux_request, effective_policy).await?;
    let playback_session = app
        .start_playback_session(StartPlaybackSessionRequest {
            principal_id: request.principal.principal_id,
            source_id: request.source_id,
            mode: PlaybackSessionMode::Remux,
            client: Some(request.client.clone()),
        })
        .await?;
    app.link_playback_session_transcode(playback_session.id, remux_start.session.id)
        .await?;
    let remux = wait_for_remux_start(app, remux_start).await?;

    if remux.disposition == RemuxSourceDisposition::Cancelled {
        return Err(NakoError::Provider {
            provider: "ffmpeg_remux".to_owned(),
            message: "remux session was cancelled".to_owned(),
        });
    }

    let response = remux_direct_play_response(
        &remux.output_path,
        request.output_container,
        request.range_request,
    )
    .await?;

    Ok(RemuxPlaybackStreamOutput {
        session: playback_session,
        output_path: remux.output_path,
        response,
    })
}

pub(super) async fn remux_playback_session_stream(
    app: &PlaybackAppService,
    request: RemuxPlaybackSessionStreamRequest,
) -> Result<RemuxPlaybackStreamOutput> {
    let mut playback_session = app
        .existing_playback_session_for_media_request(
            &request.principal,
            request.playback_session_id,
            request.source_id,
            PlaybackSessionMode::Remux,
        )
        .await?;
    let transcode_session_id = match playback_session.transcode_session_id {
        Some(transcode_session_id) => transcode_session_id,
        None => {
            let client =
                PlaybackAppService::client_capabilities_for_playback_session(&playback_session)?;
            let effective_policy = app
                .effective_playback_policy_for_source_id(&request.principal, request.source_id)
                .await?;
            let remux_start = start_remux_source_with_policy(
                app,
                RemuxSourceRequest {
                    source_id: request.source_id,
                    client,
                    output_container: request.output_container,
                },
                effective_policy,
            )
            .await?;
            playback_session = app
                .link_playback_session_transcode(playback_session.id, remux_start.session.id)
                .await?;
            remux_start.session.id
        }
    };
    let transcode = wait_for_remux_transcode_output(app, transcode_session_id).await?;
    if transcode.source_id != request.source_id {
        return Err(NakoError::InvalidInput {
            message: format!(
                "remux playback session {} source_id does not match transcode session {}",
                playback_session.id, transcode.id
            ),
        });
    }

    let response = remux_direct_play_response(
        &transcode.output_path,
        request.output_container,
        request.range_request,
    )
    .await?;

    Ok(RemuxPlaybackStreamOutput {
        session: playback_session,
        output_path: transcode.output_path,
        response,
    })
}

pub(super) async fn remux_playback_preflight(
    app: &PlaybackAppService,
    request: RemuxPlaybackPreflightRequest,
) -> Result<RemuxPlaybackPreflightOutput> {
    let effective_policy = app
        .effective_playback_policy_for_source_id(&request.principal, request.source_id)
        .await?;
    let remux = start_remux_source_with_policy(
        app,
        RemuxSourceRequest {
            source_id: request.source_id,
            client: request.client.clone(),
            output_container: request.output_container,
        },
        effective_policy,
    )
    .await?;
    let playback_session = app
        .start_playback_session(StartPlaybackSessionRequest {
            principal_id: request.principal.principal_id,
            source_id: request.source_id,
            mode: PlaybackSessionMode::Remux,
            client: Some(request.client.clone()),
        })
        .await?;
    app.link_playback_session_transcode(playback_session.id, remux.session.id)
        .await?;

    let response = nako_streaming::plan_direct_play_response(
        0,
        nako_streaming::content_type_for_file_name(&format!(
            "stream.{}",
            request.output_container.file_extension()
        )),
        DirectPlayRangeRequest::None,
    );

    Ok(RemuxPlaybackPreflightOutput {
        session: playback_session,
        response,
    })
}

pub(super) async fn remux_source(
    app: &PlaybackAppService,
    request: RemuxSourceRequest,
) -> Result<RemuxSourceOutput> {
    let context = remux_source_context(app, &request, None).await?;
    run_remux_source_context(app, context, None).await
}

pub(super) async fn start_remux_source_with_policy(
    app: &PlaybackAppService,
    request: RemuxSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<RemuxSessionStart> {
    let effective_policy = effective_policy.into();
    let context = remux_source_context(app, &request, effective_policy.clone()).await?;
    if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
        app.runtime_store.as_ref(),
        context.source.id,
        TranscodeSessionKind::Remux,
        &context.request_key,
    )
    .await?
    {
        return Ok(context.session_start(active));
    }

    if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
        app.runtime_store.as_ref(),
        context.source.id,
        TranscodeSessionKind::Remux,
        &context.request_key,
    )
    .await?
    {
        if latest.state == TranscodeSessionState::Finished
            && latest.output_path == context.output_path
            && path_exists(&context.output_path)?
        {
            return Ok(context.session_start(latest));
        }
    }

    let resource_permit = app
        .resource_admission
        .try_acquire(&context.resource_demand())?;
    let task_app = app.clone();
    let task_request = request.clone();
    let task_effective_policy = effective_policy;
    app.runtime
        .spawn("playback_remux_start", "playback.remux", async move {
            if let Err(error) = remux_source_with_policy(
                &task_app,
                task_request,
                task_effective_policy,
                Some(resource_permit),
            )
            .await
            {
                warn!(error = %error, "background remux start failed");
            }
        });

    wait_for_started_remux_source_context(app, context).await
}

async fn remux_source_with_policy(
    app: &PlaybackAppService,
    request: RemuxSourceRequest,
    effective_policy: Option<EffectivePlaybackPolicy>,
    resource_permit: Option<PlaybackResourcePermitSet>,
) -> Result<RemuxSourceOutput> {
    let context = remux_source_context(app, &request, effective_policy).await?;
    run_remux_source_context(app, context, resource_permit).await
}

async fn wait_for_remux_start(
    app: &PlaybackAppService,
    start: RemuxSessionStart,
) -> Result<RemuxSourceOutput> {
    wait_for_remux_session_output(
        app,
        start.source,
        start.decision,
        start.output_path,
        start.output_container,
        start.session.id,
    )
    .await
}

async fn run_remux_source_context(
    app: &PlaybackAppService,
    context: RemuxSourceContext,
    resource_permit: Option<PlaybackResourcePermitSet>,
) -> Result<RemuxSourceOutput> {
    let resource_demand = context.resource_demand();
    let input = app
        .input
        .source_input_scope(&context.source, &context.uri, &context.backend)
        .await?;
    let input_service = app.input.clone();
    input_service
        .with_prepared_source_input(input, |input_path| async move {
            app.remux
                .run(
                    app.runtime_store.as_ref(),
                    context.source,
                    context.decision,
                    input_path,
                    context.output_path,
                    context.output_container,
                    context.request_identity,
                    &app.resource_admission,
                    resource_demand,
                    resource_permit,
                )
                .await
        })
        .await
}

async fn wait_for_started_remux_source_context(
    app: &PlaybackAppService,
    context: RemuxSourceContext,
) -> Result<RemuxSessionStart> {
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(active) = PlaybackRuntimeStore::find_active_transcode_session(
            app.runtime_store.as_ref(),
            context.source.id,
            TranscodeSessionKind::Remux,
            &context.request_key,
        )
        .await?
        {
            return Ok(context.session_start(active));
        }

        if let Some(latest) = PlaybackRuntimeStore::find_latest_transcode_session(
            app.runtime_store.as_ref(),
            context.source.id,
            TranscodeSessionKind::Remux,
            &context.request_key,
        )
        .await?
        {
            if latest.state.is_terminal() {
                return Ok(context.session_start(latest));
            }
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(NakoError::Conflict {
                message: format!(
                    "remux request for source {} did not expose a playback session before timeout",
                    context.source.id
                ),
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
}

async fn wait_for_remux_session_output(
    app: &PlaybackAppService,
    source: MediaSource,
    decision: PlaybackDecision,
    output_path: PathBuf,
    output_container: RemuxContainer,
    session_id: TranscodeSessionId,
) -> Result<RemuxSourceOutput> {
    let deadline = tokio::time::Instant::now()
        + std::time::Duration::from_millis(app.config.remux_timeout_ms.max(1));
    loop {
        let session = app.get_transcode_session(session_id).await?;
        match session.state {
            TranscodeSessionState::Finished => {
                if !path_exists(&output_path)? {
                    return Err(NakoError::storage_io(
                        output_path.display().to_string(),
                        "finished remux session output is missing",
                    ));
                }

                return Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::ReusedExisting,
                    session: Some(session),
                });
            }
            TranscodeSessionState::Cancelled => {
                return Ok(RemuxSourceOutput {
                    source,
                    decision,
                    output_path,
                    output_container,
                    disposition: RemuxSourceDisposition::Cancelled,
                    session: Some(session),
                });
            }
            TranscodeSessionState::Failed => {
                return Err(NakoError::Provider {
                    provider: "ffmpeg_remux".to_owned(),
                    message: session
                        .failure_message
                        .unwrap_or_else(|| "remux runner failed".to_owned()),
                });
            }
            TranscodeSessionState::Planned
            | TranscodeSessionState::Starting
            | TranscodeSessionState::Running
            | TranscodeSessionState::CancelRequested => {}
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(NakoError::Provider {
                provider: "ffmpeg_remux".to_owned(),
                message: format!("remux session {session_id} timed out while waiting"),
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

async fn wait_for_remux_transcode_output(
    app: &PlaybackAppService,
    session_id: TranscodeSessionId,
) -> Result<TranscodeSessionRecord> {
    let deadline = tokio::time::Instant::now()
        + std::time::Duration::from_millis(app.config.remux_timeout_ms.max(1));
    loop {
        let session = app.get_transcode_session(session_id).await?;
        if session.kind != TranscodeSessionKind::Remux {
            return Err(NakoError::InvalidInput {
                message: format!("session {session_id} is not a remux session"),
            });
        }
        match session.state {
            TranscodeSessionState::Finished => {
                if !path_exists(&session.output_path)? {
                    return Err(NakoError::storage_io(
                        session.output_path.display().to_string(),
                        "finished remux session output is missing",
                    ));
                }

                return Ok(session);
            }
            TranscodeSessionState::Cancelled => {
                return Err(NakoError::Provider {
                    provider: "ffmpeg_remux".to_owned(),
                    message: "remux session was cancelled".to_owned(),
                });
            }
            TranscodeSessionState::Failed => {
                return Err(NakoError::Provider {
                    provider: "ffmpeg_remux".to_owned(),
                    message: session
                        .failure_message
                        .unwrap_or_else(|| "remux runner failed".to_owned()),
                });
            }
            TranscodeSessionState::Planned
            | TranscodeSessionState::Starting
            | TranscodeSessionState::Running
            | TranscodeSessionState::CancelRequested => {}
        }

        if tokio::time::Instant::now() >= deadline {
            return Err(NakoError::Provider {
                provider: "ffmpeg_remux".to_owned(),
                message: format!("remux session {session_id} timed out while waiting"),
            });
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}

async fn remux_source_context(
    app: &PlaybackAppService,
    request: &RemuxSourceRequest,
    effective_policy: impl Into<Option<EffectivePlaybackPolicy>>,
) -> Result<RemuxSourceContext> {
    let source = app.get_source_or_not_found(request.source_id).await?;
    let probe =
        PlaybackRuntimeStore::get_media_probe(app.runtime_store.as_ref(), source.id).await?;
    let (uri, backend) = app.storage_backend_for_media_source(&source).await?;
    let mut context = playback_selection_context(&uri, backend.as_ref()).await;
    context.preferences.remux_output_container = Some(transcode_remux_container_to_playback(
        request.output_container,
    ));
    let target = playback_target_for_client(request.client.clone());
    let target_profile = PlaybackTargetProfile::from_target(&target, context.clone());
    let remote_input = target_profile.storage.remote;
    let effective_policy = effective_policy
        .into()
        .unwrap_or_else(|| default_playback_policy_for_source(&source));
    let decision = app.planner.plan(PlaybackPlanningRequest {
        source: &source,
        probe: probe.as_ref(),
        target: &target,
        effective_policy: &effective_policy,
        context,
    });
    ensure_playback_decision_allowed(&decision)?;
    let output_container = remux_output_container(&decision)?;
    let profile_identity = build_playback_remux_profile(PlaybackRemuxProfileRequest {
        output_container,
        track_selection: playback_track_selection_to_transcode(target_profile.track_selection()),
        remote_input: target_profile.storage.remote,
        playback_profile_key: target_profile.identity_key(),
    })?
    .identity();
    let request_identity =
        profile_identity.bind_source(&TranscodeSourceIdentity::from_media_source(&source));
    let staging = RemuxStagingPolicy::new(&app.config.remux_staging_root)?;
    let output_path = staging.output_path(source.id, &request_identity, output_container)?;
    let request_key = RemuxRequestKey {
        source_id: source.id,
        request_identity: request_identity.clone(),
    }
    .persisted_request_key();

    Ok(RemuxSourceContext {
        source,
        decision,
        uri,
        backend,
        output_path,
        output_container,
        request_identity,
        request_key,
        remote_input,
    })
}

async fn remux_direct_play_response(
    output_path: &Path,
    output_container: RemuxContainer,
    range_request: DirectPlayRangeRequest,
) -> Result<DirectPlayResponsePlan> {
    let total_len = tokio::fs::metadata(output_path)
        .await
        .map_err(|err| {
            NakoError::storage_io(
                output_path.display().to_string(),
                format!("failed to read remux output length: {err}"),
            )
        })?
        .len();

    Ok(nako_streaming::plan_direct_play_response(
        total_len,
        nako_streaming::content_type_for_file_name(&format!(
            "stream.{}",
            output_container.file_extension()
        )),
        range_request,
    ))
}

#[derive(Clone, Debug)]
struct RemuxSourceContext {
    source: MediaSource,
    decision: PlaybackDecision,
    uri: StorageUri,
    backend: Arc<LibraryStorageBackend>,
    output_path: PathBuf,
    output_container: RemuxContainer,
    request_identity: TranscodeRequestIdentity,
    request_key: String,
    remote_input: bool,
}

impl RemuxSourceContext {
    fn resource_demand(&self) -> super::PlaybackResourceDemand {
        super::PlaybackResourceDemand::remux(self.remote_input)
    }

    fn session_start(self, session: TranscodeSessionRecord) -> RemuxSessionStart {
        RemuxSessionStart {
            source: self.source,
            decision: self.decision,
            output_path: self.output_path,
            output_container: self.output_container,
            session,
        }
    }
}
