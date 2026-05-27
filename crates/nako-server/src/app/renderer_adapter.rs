use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use nako_core::{
    MediaSourceId, NakoError, PlaybackSessionId, PlaybackTargetId, PlaybackTargetKind,
    PlaybackTargetNetworkScope, PlaybackTargetTransportAuth, RendererControlCapabilities,
    RendererControlCommand, RendererSessionId, Result,
};
use nako_playback::{ClientPlaybackCapabilities, PlaybackTarget};

use super::playback::RendererPlaybackTransportPlan;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RendererAdapterTargetRecord {
    pub(crate) adapter_id: String,
    pub(crate) stable_device_id: String,
    pub(crate) target_kind: PlaybackTargetKind,
    pub(crate) display_name: String,
    pub(crate) network_scope: PlaybackTargetNetworkScope,
    pub(crate) media_capabilities: ClientPlaybackCapabilities,
    pub(crate) control_capabilities: RendererControlCapabilities,
    pub(crate) updated_at_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PublishRendererAdapterTargetRequest {
    pub(crate) adapter_id: String,
    pub(crate) stable_device_id: String,
    pub(crate) target_kind: PlaybackTargetKind,
    pub(crate) display_name: String,
    pub(crate) network_scope: PlaybackTargetNetworkScope,
    pub(crate) media_capabilities: ClientPlaybackCapabilities,
    pub(crate) control_capabilities: RendererControlCapabilities,
    pub(crate) now_ms: i64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BuildRendererAdapterCommandEnvelopeRequest {
    pub(crate) adapter_id: String,
    pub(crate) stable_device_id: String,
    pub(crate) renderer_session_id: RendererSessionId,
    pub(crate) playback_session_id: PlaybackSessionId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) command: RendererControlCommand,
    pub(crate) position_ms: Option<u64>,
    pub(crate) transport: RendererPlaybackTransportPlan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RendererAdapterCommandEnvelope {
    pub(crate) adapter_id: String,
    pub(crate) stable_device_id: String,
    pub(crate) target_kind: PlaybackTargetKind,
    pub(crate) renderer_session_id: RendererSessionId,
    pub(crate) playback_session_id: PlaybackSessionId,
    pub(crate) source_id: MediaSourceId,
    pub(crate) command: RendererControlCommand,
    pub(crate) position_ms: Option<u64>,
    pub(crate) transport: RendererPlaybackTransportPlan,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RendererAdapterBridgeService {
    store: Arc<Mutex<RendererAdapterBridgeStore>>,
}

impl RendererAdapterBridgeService {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn publish_target(
        &self,
        request: PublishRendererAdapterTargetRequest,
    ) -> Result<RendererAdapterTargetRecord> {
        ensure_external_target_kind(request.target_kind)?;
        ensure_local_network_scope(request.network_scope)?;
        ensure_play_command_supported(&request.control_capabilities)?;

        let adapter_id = normalize_identifier("adapter_id", &request.adapter_id)?;
        let stable_device_id = normalize_identifier("stable_device_id", &request.stable_device_id)?;
        let display_name = normalize_display_name(&request.display_name)?;
        let record = RendererAdapterTargetRecord {
            adapter_id,
            stable_device_id,
            target_kind: request.target_kind,
            display_name,
            network_scope: request.network_scope,
            media_capabilities: request.media_capabilities,
            control_capabilities: request.control_capabilities,
            updated_at_ms: request.now_ms,
        };
        let mut store = self
            .store
            .lock()
            .expect("renderer adapter bridge store mutex poisoned");
        store.targets.insert(
            adapter_target_key(&record.adapter_id, &record.stable_device_id),
            record.clone(),
        );

        Ok(record)
    }

    pub(crate) fn list_targets(&self) -> Vec<RendererAdapterTargetRecord> {
        self.store
            .lock()
            .expect("renderer adapter bridge store mutex poisoned")
            .targets
            .values()
            .cloned()
            .collect()
    }

    pub(crate) fn playback_target_for(
        &self,
        target: &RendererAdapterTargetRecord,
    ) -> PlaybackTarget {
        PlaybackTarget {
            id: PlaybackTargetId::new(),
            kind: target.target_kind,
            display_name: target.display_name.clone(),
            network_scope: target.network_scope,
            transport_auth: PlaybackTargetTransportAuth::CastTicket,
            media_capabilities: target.media_capabilities.clone(),
            control_capabilities: target.control_capabilities.clone(),
        }
    }

    pub(crate) fn build_command_envelope(
        &self,
        request: BuildRendererAdapterCommandEnvelopeRequest,
    ) -> Result<RendererAdapterCommandEnvelope> {
        let adapter_id = normalize_identifier("adapter_id", &request.adapter_id)?;
        let stable_device_id = normalize_identifier("stable_device_id", &request.stable_device_id)?;
        let target = self
            .store
            .lock()
            .expect("renderer adapter bridge store mutex poisoned")
            .targets
            .get(&adapter_target_key(&adapter_id, &stable_device_id))
            .cloned()
            .ok_or_else(|| NakoError::NotFound {
                entity: "renderer_adapter_target",
                id: format!("{adapter_id}/{stable_device_id}"),
            })?;
        if !target.control_capabilities.supports(request.command) {
            return Err(NakoError::Forbidden {
                message: format!(
                    "renderer adapter target {}/{} does not support {}",
                    adapter_id,
                    stable_device_id,
                    request.command.as_str()
                ),
            });
        }

        Ok(RendererAdapterCommandEnvelope {
            adapter_id,
            stable_device_id,
            target_kind: target.target_kind,
            renderer_session_id: request.renderer_session_id,
            playback_session_id: request.playback_session_id,
            source_id: request.source_id,
            command: request.command,
            position_ms: request.position_ms,
            transport: request.transport,
        })
    }
}

#[derive(Debug, Default)]
struct RendererAdapterBridgeStore {
    targets: HashMap<String, RendererAdapterTargetRecord>,
}

fn adapter_target_key(adapter_id: &str, stable_device_id: &str) -> String {
    format!("{adapter_id}\u{1f}{stable_device_id}")
}

fn ensure_external_target_kind(target_kind: PlaybackTargetKind) -> Result<()> {
    if matches!(
        target_kind,
        PlaybackTargetKind::Chromecast
            | PlaybackTargetKind::DlnaRenderer
            | PlaybackTargetKind::Airplay
    ) {
        return Ok(());
    }

    Err(NakoError::InvalidInput {
        message: format!(
            "renderer adapter target_kind must be an external casting protocol, got {}",
            target_kind.as_str()
        ),
    })
}

fn ensure_local_network_scope(network_scope: PlaybackTargetNetworkScope) -> Result<()> {
    if network_scope == PlaybackTargetNetworkScope::Local {
        return Ok(());
    }

    Err(NakoError::InvalidInput {
        message: format!(
            "renderer adapter targets must be local network scoped until remote casting policy is accepted, got {}",
            network_scope.as_str()
        ),
    })
}

fn ensure_play_command_supported(capabilities: &RendererControlCapabilities) -> Result<()> {
    if capabilities.supports(RendererControlCommand::Play) {
        return Ok(());
    }

    Err(NakoError::InvalidInput {
        message: "renderer adapter target must support play commands".to_owned(),
    })
}

fn normalize_identifier(field: &'static str, value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(NakoError::InvalidInput {
            message: format!("{field} is required"),
        });
    }
    if trimmed.len() > 128 {
        return Err(NakoError::InvalidInput {
            message: format!("{field} is too long"),
        });
    }

    Ok(trimmed.to_owned())
}

fn normalize_display_name(value: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "display_name is required".to_owned(),
        });
    }
    if trimmed.len() > 160 {
        return Err(NakoError::InvalidInput {
            message: "display_name is too long".to_owned(),
        });
    }

    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use nako_core::PlaybackSessionMode;
    use nako_transcode::RemuxContainer;

    use super::*;

    #[test]
    fn renderer_adapter_bridge_publishes_external_target_as_cast_ticket_playback_target() {
        let service = RendererAdapterBridgeService::new();

        let record = service
            .publish_target(PublishRendererAdapterTargetRequest {
                adapter_id: "official.chromecast".to_owned(),
                stable_device_id: "living-room-tv".to_owned(),
                target_kind: PlaybackTargetKind::Chromecast,
                display_name: " Living Room TV ".to_owned(),
                network_scope: PlaybackTargetNetworkScope::Local,
                media_capabilities: ClientPlaybackCapabilities {
                    direct_play: true,
                    containers: vec!["mp4".to_owned()],
                    video_codecs: vec!["h264".to_owned()],
                    audio_codecs: vec!["aac".to_owned()],
                },
                control_capabilities: RendererControlCapabilities::basic_playback(),
                now_ms: 1_779_814_400_000,
            })
            .unwrap();

        assert_eq!(record.adapter_id, "official.chromecast");
        assert_eq!(record.stable_device_id, "living-room-tv");
        assert_eq!(record.display_name, "Living Room TV");
        assert_eq!(record.target_kind, PlaybackTargetKind::Chromecast);
        assert_eq!(service.list_targets(), vec![record.clone()]);

        let target = service.playback_target_for(&record);
        assert_eq!(target.kind, PlaybackTargetKind::Chromecast);
        assert_eq!(target.network_scope, PlaybackTargetNetworkScope::Local);
        assert_eq!(
            target.transport_auth,
            PlaybackTargetTransportAuth::CastTicket
        );
        assert!(target.requires_cast_permission());
        assert!(target.requires_ticket_transport());
    }

    #[test]
    fn renderer_adapter_bridge_rejects_non_external_or_non_local_targets() {
        let service = RendererAdapterBridgeService::new();
        let base = PublishRendererAdapterTargetRequest {
            adapter_id: "official.cast".to_owned(),
            stable_device_id: "device".to_owned(),
            target_kind: PlaybackTargetKind::NakoRemoteClient,
            display_name: "Device".to_owned(),
            network_scope: PlaybackTargetNetworkScope::Local,
            media_capabilities: ClientPlaybackCapabilities::default(),
            control_capabilities: RendererControlCapabilities::basic_playback(),
            now_ms: 1,
        };

        let err = service.publish_target(base.clone()).unwrap_err();
        assert!(err.to_string().contains("external casting protocol"));

        let err = service
            .publish_target(PublishRendererAdapterTargetRequest {
                target_kind: PlaybackTargetKind::DlnaRenderer,
                network_scope: PlaybackTargetNetworkScope::Remote,
                ..base.clone()
            })
            .unwrap_err();
        assert!(err.to_string().contains("local network scoped"));

        let err = service
            .publish_target(PublishRendererAdapterTargetRequest {
                target_kind: PlaybackTargetKind::Airplay,
                control_capabilities: RendererControlCapabilities::none(),
                ..base
            })
            .unwrap_err();
        assert!(err.to_string().contains("support play commands"));
    }

    #[test]
    fn renderer_adapter_command_envelope_is_bounded_and_redacted() {
        let service = RendererAdapterBridgeService::new();
        service
            .publish_target(PublishRendererAdapterTargetRequest {
                adapter_id: "official.dlna".to_owned(),
                stable_device_id: "den-renderer".to_owned(),
                target_kind: PlaybackTargetKind::DlnaRenderer,
                display_name: "Den Renderer".to_owned(),
                network_scope: PlaybackTargetNetworkScope::Local,
                media_capabilities: ClientPlaybackCapabilities::default(),
                control_capabilities: RendererControlCapabilities::basic_playback(),
                now_ms: 1,
            })
            .unwrap();

        let envelope = service
            .build_command_envelope(BuildRendererAdapterCommandEnvelopeRequest {
                adapter_id: "official.dlna".to_owned(),
                stable_device_id: "den-renderer".to_owned(),
                renderer_session_id: RendererSessionId::new(),
                playback_session_id: PlaybackSessionId::new(),
                source_id: MediaSourceId::new(),
                command: RendererControlCommand::Play,
                position_ms: Some(42),
                transport: RendererPlaybackTransportPlan {
                    mode: PlaybackSessionMode::Remux,
                    remux_container: Some(RemuxContainer::Mp4),
                    content_type: "video/mp4".to_owned(),
                    supports_range_requests: true,
                },
            })
            .unwrap();

        assert_eq!(envelope.adapter_id, "official.dlna");
        assert_eq!(envelope.stable_device_id, "den-renderer");
        assert_eq!(envelope.target_kind, PlaybackTargetKind::DlnaRenderer);
        assert_eq!(envelope.command, RendererControlCommand::Play);
        assert_eq!(envelope.position_ms, Some(42));

        let debug = format!("{envelope:?}").to_ascii_lowercase();
        for forbidden in [
            "bearer",
            "payload_json",
            "source_locator",
            "local_path",
            "renderer_ticket",
            "nako_rtt_",
        ] {
            assert!(
                !debug.contains(forbidden),
                "renderer adapter command envelope leaked forbidden term: {forbidden}"
            );
        }

        let err = service
            .build_command_envelope(BuildRendererAdapterCommandEnvelopeRequest {
                adapter_id: "official.dlna".to_owned(),
                stable_device_id: "den-renderer".to_owned(),
                renderer_session_id: RendererSessionId::new(),
                playback_session_id: PlaybackSessionId::new(),
                source_id: MediaSourceId::new(),
                command: RendererControlCommand::SetVolume,
                position_ms: None,
                transport: RendererPlaybackTransportPlan {
                    mode: PlaybackSessionMode::Hls,
                    remux_container: None,
                    content_type: "application/vnd.apple.mpegurl".to_owned(),
                    supports_range_requests: false,
                },
            })
            .unwrap_err();
        assert!(err.to_string().contains("does not support set_volume"));
    }
}
