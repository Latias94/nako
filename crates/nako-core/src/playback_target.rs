use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackTargetKind {
    Browser,
    NativeDesktop,
    NativeMobile,
    NakoRemoteClient,
    Chromecast,
    DlnaRenderer,
    Airplay,
}

impl PlaybackTargetKind {
    pub const ALL: [Self; 7] = [
        Self::Browser,
        Self::NativeDesktop,
        Self::NativeMobile,
        Self::NakoRemoteClient,
        Self::Chromecast,
        Self::DlnaRenderer,
        Self::Airplay,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Browser => "browser",
            Self::NativeDesktop => "native_desktop",
            Self::NativeMobile => "native_mobile",
            Self::NakoRemoteClient => "nako_remote_client",
            Self::Chromecast => "chromecast",
            Self::DlnaRenderer => "dlna_renderer",
            Self::Airplay => "airplay",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "browser" => Some(Self::Browser),
            "native_desktop" => Some(Self::NativeDesktop),
            "native_mobile" => Some(Self::NativeMobile),
            "nako_remote_client" => Some(Self::NakoRemoteClient),
            "chromecast" => Some(Self::Chromecast),
            "dlna_renderer" => Some(Self::DlnaRenderer),
            "airplay" => Some(Self::Airplay),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackTargetNetworkScope {
    Local,
    Remote,
    Unknown,
}

impl PlaybackTargetNetworkScope {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackTargetTransportAuth {
    Bearer,
    BrowserTicket,
    CastTicket,
    None,
}

impl PlaybackTargetTransportAuth {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bearer => "bearer",
            Self::BrowserTicket => "browser_ticket",
            Self::CastTicket => "cast_ticket",
            Self::None => "none",
        }
    }

    #[must_use]
    pub const fn uses_ticket(self) -> bool {
        matches!(self, Self::BrowserTicket | Self::CastTicket)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RendererControlCommand {
    ShowItem,
    Play,
    Pause,
    Resume,
    Seek,
    Stop,
    SetVolume,
}

impl RendererControlCommand {
    pub const BASIC_PLAYBACK: [Self; 5] = [
        Self::Play,
        Self::Pause,
        Self::Resume,
        Self::Seek,
        Self::Stop,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ShowItem => "show_item",
            Self::Play => "play",
            Self::Pause => "pause",
            Self::Resume => "resume",
            Self::Seek => "seek",
            Self::Stop => "stop",
            Self::SetVolume => "set_volume",
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct RendererControlCapabilities {
    pub commands: Vec<RendererControlCommand>,
}

impl RendererControlCapabilities {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    #[must_use]
    pub fn basic_playback() -> Self {
        Self {
            commands: RendererControlCommand::BASIC_PLAYBACK.to_vec(),
        }
    }

    #[must_use]
    pub fn full_remote_player() -> Self {
        Self {
            commands: vec![
                RendererControlCommand::ShowItem,
                RendererControlCommand::Play,
                RendererControlCommand::Pause,
                RendererControlCommand::Resume,
                RendererControlCommand::Seek,
                RendererControlCommand::Stop,
                RendererControlCommand::SetVolume,
            ],
        }
    }

    #[must_use]
    pub fn supports(&self, command: RendererControlCommand) -> bool {
        self.commands.contains(&command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_kind_vocabulary_keeps_cast_protocols_as_targets() {
        let values = PlaybackTargetKind::ALL
            .iter()
            .map(|kind| kind.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                "browser",
                "native_desktop",
                "native_mobile",
                "nako_remote_client",
                "chromecast",
                "dlna_renderer",
                "airplay",
            ]
        );
        assert_eq!(
            PlaybackTargetKind::parse("chromecast"),
            Some(PlaybackTargetKind::Chromecast)
        );
    }

    #[test]
    fn renderer_control_capabilities_are_command_based() {
        let basic = RendererControlCapabilities::basic_playback();

        assert!(basic.supports(RendererControlCommand::Play));
        assert!(basic.supports(RendererControlCommand::Seek));
        assert!(!basic.supports(RendererControlCommand::SetVolume));

        let full = RendererControlCapabilities::full_remote_player();
        assert!(full.supports(RendererControlCommand::ShowItem));
        assert!(full.supports(RendererControlCommand::SetVolume));
    }

    #[test]
    fn transport_auth_distinguishes_browser_and_cast_tickets() {
        assert!(PlaybackTargetTransportAuth::BrowserTicket.uses_ticket());
        assert!(PlaybackTargetTransportAuth::CastTicket.uses_ticket());
        assert!(!PlaybackTargetTransportAuth::Bearer.uses_ticket());
        assert_eq!(PlaybackTargetNetworkScope::Remote.as_str(), "remote");
    }
}
