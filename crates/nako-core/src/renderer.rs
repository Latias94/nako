use serde::{Deserialize, Serialize};

use crate::{
    MediaItemId, MediaSourceId, PlaybackSessionId, PlaybackTargetKind, PlaybackTargetNetworkScope,
    PlaybackTargetTransportAuth, RendererCommandId, RendererControlCapabilities,
    RendererControlCommand, RendererSessionId, UserPrincipalId,
};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererSessionState {
    Online,
    Offline,
    Revoked,
}

impl RendererSessionState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Online => "online",
            Self::Offline => "offline",
            Self::Revoked => "revoked",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "online" => Some(Self::Online),
            "offline" => Some(Self::Offline),
            "revoked" => Some(Self::Revoked),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_controllable(self) -> bool {
        matches!(self, Self::Online)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererCommandState {
    Queued,
    Delivered,
    Acknowledged,
    Failed,
    Cancelled,
}

impl RendererCommandState {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Delivered => "delivered",
            Self::Acknowledged => "acknowledged",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "delivered" => Some(Self::Delivered),
            "acknowledged" => Some(Self::Acknowledged),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Acknowledged | Self::Failed | Self::Cancelled)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewRendererSession {
    pub id: RendererSessionId,
    pub owner_principal_id: UserPrincipalId,
    pub target_kind: PlaybackTargetKind,
    pub display_name: String,
    pub network_scope: PlaybackTargetNetworkScope,
    pub transport_auth: PlaybackTargetTransportAuth,
    pub media_capabilities_json: Option<String>,
    pub control_capabilities: RendererControlCapabilities,
    pub state: RendererSessionState,
    pub last_seen_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererSessionRecord {
    pub id: RendererSessionId,
    pub owner_principal_id: UserPrincipalId,
    pub target_kind: PlaybackTargetKind,
    pub display_name: String,
    pub network_scope: PlaybackTargetNetworkScope,
    pub transport_auth: PlaybackTargetTransportAuth,
    pub media_capabilities_json: Option<String>,
    pub control_capabilities: RendererControlCapabilities,
    pub state: RendererSessionState,
    pub active_playback_session_id: Option<PlaybackSessionId>,
    pub last_seen_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererSessionHeartbeat {
    pub id: RendererSessionId,
    pub state: RendererSessionState,
    pub last_seen_at_ms: i64,
    pub expires_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NewRendererCommand {
    pub id: RendererCommandId,
    pub renderer_session_id: RendererSessionId,
    pub controlling_principal_id: UserPrincipalId,
    pub command: RendererControlCommand,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub playback_session_id: Option<PlaybackSessionId>,
    pub position_ms: Option<u64>,
    pub volume_percent: Option<u8>,
    pub payload_json: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandRecord {
    pub id: RendererCommandId,
    pub renderer_session_id: RendererSessionId,
    pub controlling_principal_id: UserPrincipalId,
    pub command: RendererControlCommand,
    pub state: RendererCommandState,
    pub item_id: Option<MediaItemId>,
    pub source_id: Option<MediaSourceId>,
    pub playback_session_id: Option<PlaybackSessionId>,
    pub position_ms: Option<u64>,
    pub volume_percent: Option<u8>,
    pub payload_json: Option<String>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub delivered_at_ms: Option<i64>,
    pub completed_at_ms: Option<i64>,
    pub failure_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererCommandCompletion {
    pub id: RendererCommandId,
    pub state: RendererCommandState,
    pub completed_at_ms: i64,
    pub failure_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_session_is_separate_from_playback_attempts() {
        let session = NewRendererSession {
            id: RendererSessionId::new(),
            owner_principal_id: UserPrincipalId::local_admin(),
            target_kind: PlaybackTargetKind::NakoRemoteClient,
            display_name: "Living Room Desktop".to_owned(),
            network_scope: PlaybackTargetNetworkScope::Local,
            transport_auth: PlaybackTargetTransportAuth::Bearer,
            media_capabilities_json: Some(r#"{"containers":["mp4"]}"#.to_owned()),
            control_capabilities: RendererControlCapabilities::full_remote_player(),
            state: RendererSessionState::Online,
            last_seen_at_ms: 1_779_814_400_000,
            expires_at_ms: Some(1_779_814_700_000),
            created_at_ms: 1_779_814_400_000,
            updated_at_ms: 1_779_814_400_000,
        };

        assert_eq!(session.target_kind, PlaybackTargetKind::NakoRemoteClient);
        assert!(session.state.is_controllable());
        assert!(
            session
                .control_capabilities
                .supports(RendererControlCommand::Play)
        );
        assert_eq!(RendererSessionState::parse("online"), Some(session.state));
    }

    #[test]
    fn renderer_commands_have_delivery_lifecycle_not_transcode_state() {
        let command = NewRendererCommand {
            id: RendererCommandId::new(),
            renderer_session_id: RendererSessionId::new(),
            controlling_principal_id: UserPrincipalId::local_admin(),
            command: RendererControlCommand::Seek,
            item_id: Some(MediaItemId::new()),
            source_id: Some(MediaSourceId::new()),
            playback_session_id: Some(PlaybackSessionId::new()),
            position_ms: Some(42_000),
            volume_percent: None,
            payload_json: None,
            created_at_ms: 1,
            updated_at_ms: 1,
        };

        assert_eq!(command.command.as_str(), "seek");
        assert_eq!(
            RendererControlCommand::parse("seek"),
            Some(RendererControlCommand::Seek)
        );
        assert!(!RendererCommandState::Queued.is_terminal());
        assert!(!RendererCommandState::Delivered.is_terminal());
        assert!(RendererCommandState::Acknowledged.is_terminal());
        assert_eq!(
            RendererCommandState::parse("acknowledged"),
            Some(RendererCommandState::Acknowledged)
        );
    }
}
