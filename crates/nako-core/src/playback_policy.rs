use serde::{Deserialize, Serialize};

use crate::{LibraryAccessLevel, LibraryId};

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackPermission {
    MediaPlayback,
    DirectPlay,
    Remux,
    AudioTranscode,
    VideoTranscode,
    RemotePlayback,
    RemoteControl,
    Cast,
}

impl PlaybackPermission {
    pub const ALL: [Self; 8] = [
        Self::MediaPlayback,
        Self::DirectPlay,
        Self::Remux,
        Self::AudioTranscode,
        Self::VideoTranscode,
        Self::RemotePlayback,
        Self::RemoteControl,
        Self::Cast,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MediaPlayback => "media_playback",
            Self::DirectPlay => "direct_play",
            Self::Remux => "remux",
            Self::AudioTranscode => "audio_transcode",
            Self::VideoTranscode => "video_transcode",
            Self::RemotePlayback => "remote_playback",
            Self::RemoteControl => "remote_control",
            Self::Cast => "cast",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "media_playback" => Some(Self::MediaPlayback),
            "direct_play" => Some(Self::DirectPlay),
            "remux" => Some(Self::Remux),
            "audio_transcode" => Some(Self::AudioTranscode),
            "video_transcode" => Some(Self::VideoTranscode),
            "remote_playback" => Some(Self::RemotePlayback),
            "remote_control" => Some(Self::RemoteControl),
            "cast" => Some(Self::Cast),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackPermissionDecisionReason {
    Allowed,
    LibraryAccessDoesNotAllowPlay,
    MediaPlaybackDisabled,
    DirectPlayDisabled,
    RemuxDisabled,
    AudioTranscodeDisabled,
    VideoTranscodeDisabled,
    RemotePlaybackDisabled,
    RemoteControlDisabled,
    CastDisabled,
}

impl PlaybackPermissionDecisionReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Allowed => "allowed",
            Self::LibraryAccessDoesNotAllowPlay => "library_access_does_not_allow_play",
            Self::MediaPlaybackDisabled => "media_playback_disabled",
            Self::DirectPlayDisabled => "direct_play_disabled",
            Self::RemuxDisabled => "remux_disabled",
            Self::AudioTranscodeDisabled => "audio_transcode_disabled",
            Self::VideoTranscodeDisabled => "video_transcode_disabled",
            Self::RemotePlaybackDisabled => "remote_playback_disabled",
            Self::RemoteControlDisabled => "remote_control_disabled",
            Self::CastDisabled => "cast_disabled",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PlaybackPermissionDecision {
    pub permission: PlaybackPermission,
    pub allowed: bool,
    pub reason: PlaybackPermissionDecisionReason,
}

impl PlaybackPermissionDecision {
    #[must_use]
    pub const fn allowed(permission: PlaybackPermission) -> Self {
        Self {
            permission,
            allowed: true,
            reason: PlaybackPermissionDecisionReason::Allowed,
        }
    }

    #[must_use]
    pub const fn denied(
        permission: PlaybackPermission,
        reason: PlaybackPermissionDecisionReason,
    ) -> Self {
        Self {
            permission,
            allowed: false,
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackPermissionPolicy {
    pub allow_media_playback: bool,
    pub allow_direct_play: bool,
    pub allow_remux: bool,
    pub allow_audio_transcode: bool,
    pub allow_video_transcode: bool,
    pub allow_remote_playback: bool,
    pub allow_remote_control: bool,
    pub allow_cast: bool,
    pub max_streaming_bitrate: Option<u64>,
    pub max_remote_bitrate: Option<u64>,
}

impl PlaybackPermissionPolicy {
    #[must_use]
    pub const fn deny_all() -> Self {
        Self {
            allow_media_playback: false,
            allow_direct_play: false,
            allow_remux: false,
            allow_audio_transcode: false,
            allow_video_transcode: false,
            allow_remote_playback: false,
            allow_remote_control: false,
            allow_cast: false,
            max_streaming_bitrate: None,
            max_remote_bitrate: None,
        }
    }

    #[must_use]
    pub const fn current_playback_defaults() -> Self {
        Self {
            allow_media_playback: true,
            allow_direct_play: true,
            allow_remux: true,
            allow_audio_transcode: true,
            allow_video_transcode: true,
            allow_remote_playback: true,
            allow_remote_control: false,
            allow_cast: false,
            max_streaming_bitrate: None,
            max_remote_bitrate: None,
        }
    }

    #[must_use]
    pub const fn administrator_defaults() -> Self {
        Self {
            allow_media_playback: true,
            allow_direct_play: true,
            allow_remux: true,
            allow_audio_transcode: true,
            allow_video_transcode: true,
            allow_remote_playback: true,
            allow_remote_control: true,
            allow_cast: true,
            max_streaming_bitrate: None,
            max_remote_bitrate: None,
        }
    }

    #[must_use]
    pub const fn check(self, permission: PlaybackPermission) -> PlaybackPermissionDecision {
        if matches!(
            permission,
            PlaybackPermission::MediaPlayback
                | PlaybackPermission::DirectPlay
                | PlaybackPermission::Remux
                | PlaybackPermission::AudioTranscode
                | PlaybackPermission::VideoTranscode
                | PlaybackPermission::RemotePlayback
                | PlaybackPermission::Cast
        ) && !self.allow_media_playback
        {
            return PlaybackPermissionDecision::denied(
                permission,
                PlaybackPermissionDecisionReason::MediaPlaybackDisabled,
            );
        }

        let reason = match permission {
            PlaybackPermission::MediaPlayback => None,
            PlaybackPermission::DirectPlay if !self.allow_direct_play => {
                Some(PlaybackPermissionDecisionReason::DirectPlayDisabled)
            }
            PlaybackPermission::Remux if !self.allow_remux => {
                Some(PlaybackPermissionDecisionReason::RemuxDisabled)
            }
            PlaybackPermission::AudioTranscode if !self.allow_audio_transcode => {
                Some(PlaybackPermissionDecisionReason::AudioTranscodeDisabled)
            }
            PlaybackPermission::VideoTranscode if !self.allow_video_transcode => {
                Some(PlaybackPermissionDecisionReason::VideoTranscodeDisabled)
            }
            PlaybackPermission::RemotePlayback if !self.allow_remote_playback => {
                Some(PlaybackPermissionDecisionReason::RemotePlaybackDisabled)
            }
            PlaybackPermission::RemoteControl if !self.allow_remote_control => {
                Some(PlaybackPermissionDecisionReason::RemoteControlDisabled)
            }
            PlaybackPermission::Cast if !self.allow_cast => {
                Some(PlaybackPermissionDecisionReason::CastDisabled)
            }
            _ => None,
        };

        match reason {
            Some(reason) => PlaybackPermissionDecision::denied(permission, reason),
            None => PlaybackPermissionDecision::allowed(permission),
        }
    }
}

impl Default for PlaybackPermissionPolicy {
    fn default() -> Self {
        Self::current_playback_defaults()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectivePlaybackPolicyReason {
    SingleAdminMode,
    AdministratorRole,
    LibraryAccessDefault,
    UserPolicy,
    RolePolicy,
    NoPlayAccess,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EffectivePlaybackPolicy {
    pub library_id: LibraryId,
    pub library_access: LibraryAccessLevel,
    pub permissions: PlaybackPermissionPolicy,
    pub reason: EffectivePlaybackPolicyReason,
}

impl EffectivePlaybackPolicy {
    #[must_use]
    pub fn from_library_access(library_id: LibraryId, access: LibraryAccessLevel) -> Self {
        if access.allows_play() {
            Self {
                library_id,
                library_access: access,
                permissions: PlaybackPermissionPolicy::current_playback_defaults(),
                reason: EffectivePlaybackPolicyReason::LibraryAccessDefault,
            }
        } else {
            Self {
                library_id,
                library_access: access,
                permissions: PlaybackPermissionPolicy::deny_all(),
                reason: EffectivePlaybackPolicyReason::NoPlayAccess,
            }
        }
    }

    #[must_use]
    pub fn administrator(library_id: LibraryId, access: LibraryAccessLevel) -> Self {
        Self {
            library_id,
            library_access: access,
            permissions: PlaybackPermissionPolicy::administrator_defaults(),
            reason: EffectivePlaybackPolicyReason::AdministratorRole,
        }
    }

    #[must_use]
    pub fn check(&self, permission: PlaybackPermission) -> PlaybackPermissionDecision {
        if !self.library_access.allows_play() {
            return PlaybackPermissionDecision::denied(
                permission,
                PlaybackPermissionDecisionReason::LibraryAccessDoesNotAllowPlay,
            );
        }

        self.permissions.check(permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn playback_policy_from_play_access_matches_current_playback_defaults() {
        let policy = EffectivePlaybackPolicy::from_library_access(
            LibraryId::new(),
            LibraryAccessLevel::Play,
        );

        assert!(policy.check(PlaybackPermission::DirectPlay).allowed);
        assert!(policy.check(PlaybackPermission::Remux).allowed);
        assert!(policy.check(PlaybackPermission::AudioTranscode).allowed);
        assert!(policy.check(PlaybackPermission::VideoTranscode).allowed);
        assert!(policy.check(PlaybackPermission::RemotePlayback).allowed);
        assert_eq!(
            policy.check(PlaybackPermission::Cast).reason,
            PlaybackPermissionDecisionReason::CastDisabled
        );
        assert_eq!(
            policy.check(PlaybackPermission::RemoteControl).reason,
            PlaybackPermissionDecisionReason::RemoteControlDisabled
        );
    }

    #[test]
    fn playback_policy_denies_everything_without_play_access() {
        let policy = EffectivePlaybackPolicy::from_library_access(
            LibraryId::new(),
            LibraryAccessLevel::Browse,
        );

        for permission in PlaybackPermission::ALL {
            let decision = policy.check(permission);
            assert!(!decision.allowed);
            assert_eq!(
                decision.reason,
                PlaybackPermissionDecisionReason::LibraryAccessDoesNotAllowPlay
            );
        }
    }

    #[test]
    fn playback_policy_reports_mode_specific_denial_reasons() {
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_remux = false;
        permissions.allow_video_transcode = false;
        permissions.max_streaming_bitrate = Some(8_000_000);

        assert_eq!(
            permissions.check(PlaybackPermission::Remux),
            PlaybackPermissionDecision::denied(
                PlaybackPermission::Remux,
                PlaybackPermissionDecisionReason::RemuxDisabled,
            )
        );
        assert_eq!(
            permissions.check(PlaybackPermission::VideoTranscode),
            PlaybackPermissionDecision::denied(
                PlaybackPermission::VideoTranscode,
                PlaybackPermissionDecisionReason::VideoTranscodeDisabled,
            )
        );
        assert_eq!(permissions.max_streaming_bitrate, Some(8_000_000));
        assert!(permissions.check(PlaybackPermission::DirectPlay).allowed);
    }

    #[test]
    fn administrator_playback_policy_allows_control_and_cast() {
        let policy =
            EffectivePlaybackPolicy::administrator(LibraryId::new(), LibraryAccessLevel::Manage);

        assert_eq!(
            policy.reason,
            EffectivePlaybackPolicyReason::AdministratorRole
        );
        assert!(policy.check(PlaybackPermission::RemoteControl).allowed);
        assert!(policy.check(PlaybackPermission::Cast).allowed);
    }
}
