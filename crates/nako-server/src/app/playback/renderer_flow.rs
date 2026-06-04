use nako_core::{NakoError, PlaybackPermission, PlaybackSessionMode, Result};
use nako_playback::{PlaybackMode, PlaybackPlanningRequest, PlaybackPreferenceContext};
use nako_transcode::HlsPlaybackGeneration;

use super::{
    HlsSourceRequest, PlaybackAppService, PlaybackRuntimeStore, RemuxSourceRequest,
    RendererPlaybackTransportPlan, StartPlaybackSessionRequest, StartRendererPlaybackSessionOutput,
    StartRendererPlaybackSessionRequest, ensure_playback_decision_allowed,
    ensure_playback_permission_allowed, hls_flow, playback_policy_forbidden, remux_flow,
    selection::remux_output_container,
};

pub(super) async fn start_renderer_playback_session(
    app: &PlaybackAppService,
    request: StartRendererPlaybackSessionRequest,
) -> Result<StartRendererPlaybackSessionOutput> {
    let source = app.get_source_or_not_found(request.source_id).await?;
    let probe =
        PlaybackRuntimeStore::get_media_probe(app.runtime_store.as_ref(), source.id).await?;
    let context = app.playback_selection_context_for_source(&source).await?;
    let effective_policy = app
        .effective_playback_policy_for_source(&request.principal, &source)
        .await?;
    ensure_playback_permission_allowed(&effective_policy, PlaybackPermission::RemoteControl)?;

    let decision = app.planner.plan(PlaybackPlanningRequest {
        source: &source,
        probe: probe.as_ref(),
        target: &request.target,
        effective_policy: &effective_policy,
        context,
    });
    ensure_playback_decision_allowed(&decision)?;

    match decision.mode {
        PlaybackMode::DirectPlay => {
            let direct = decision.direct_play_plan().ok_or_else(|| {
                NakoError::Unsupported("direct renderer decision did not include a direct plan")
            })?;
            let session = app
                .start_playback_session(StartPlaybackSessionRequest {
                    principal_id: request.principal.principal_id,
                    source_id: request.source_id,
                    mode: PlaybackSessionMode::Direct,
                    client: Some(request.target.media_capabilities.clone()),
                })
                .await?;

            Ok(StartRendererPlaybackSessionOutput {
                session,
                transport: RendererPlaybackTransportPlan {
                    mode: PlaybackSessionMode::Direct,
                    remux_container: None,
                    content_type: direct.content_type.clone(),
                    supports_range_requests: direct.supports_range_requests,
                },
            })
        }
        PlaybackMode::Remux => {
            let output_container = remux_output_container(&decision)?;
            let remux = remux_flow::start_remux_source_with_policy(
                app,
                RemuxSourceRequest {
                    source_id: request.source_id,
                    client: request.target.media_capabilities.clone(),
                    output_container,
                },
                effective_policy.clone(),
            )
            .await?;
            let session = app
                .start_playback_session(StartPlaybackSessionRequest {
                    principal_id: request.principal.principal_id,
                    source_id: request.source_id,
                    mode: PlaybackSessionMode::Remux,
                    client: Some(request.target.media_capabilities.clone()),
                })
                .await?;
            app.link_playback_session_transcode(session.id, remux.session.id)
                .await?;

            Ok(StartRendererPlaybackSessionOutput {
                session,
                transport: RendererPlaybackTransportPlan {
                    mode: PlaybackSessionMode::Remux,
                    remux_container: Some(output_container),
                    content_type: nako_streaming::content_type_for_file_name(&format!(
                        "stream.{}",
                        output_container.file_extension()
                    ))
                    .to_owned(),
                    supports_range_requests: true,
                },
            })
        }
        PlaybackMode::Transcode => {
            let playlist = hls_flow::hls_playlist_with_policy(
                app,
                HlsSourceRequest {
                    source_id: request.source_id,
                    client: request.target.media_capabilities.clone(),
                    preferences: PlaybackPreferenceContext::default(),
                    playback_generation: HlsPlaybackGeneration::default(),
                },
                effective_policy.clone(),
                None,
            )
            .await?;
            let session = app
                .start_playback_session(StartPlaybackSessionRequest {
                    principal_id: request.principal.principal_id,
                    source_id: request.source_id,
                    mode: PlaybackSessionMode::Hls,
                    client: Some(request.target.media_capabilities.clone()),
                })
                .await?;
            app.link_playback_session_transcode(session.id, playlist.session.id)
                .await?;
            app.cancel_superseded_hls_playback_sessions(
                request.source_id,
                playlist.session.id,
                session.id,
            )
            .await?;

            Ok(StartRendererPlaybackSessionOutput {
                session,
                transport: RendererPlaybackTransportPlan {
                    mode: PlaybackSessionMode::Hls,
                    remux_container: None,
                    content_type: "application/vnd.apple.mpegurl".to_owned(),
                    supports_range_requests: false,
                },
            })
        }
        PlaybackMode::Denied => Err(playback_policy_forbidden(&decision)),
    }
}
