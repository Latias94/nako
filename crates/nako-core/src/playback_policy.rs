use serde::{Deserialize, Serialize};

use crate::{
    EffectiveLibraryAccess, EffectiveLibraryAccessReason, LibraryAccessLevel, LibraryId, UserId,
    UserRole,
};

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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope", content = "value")]
pub enum PlaybackPolicyScope {
    User(UserId),
    Role(UserRole),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlaybackPolicy {
    pub scope: PlaybackPolicyScope,
    pub library_id: LibraryId,
    pub permissions: PlaybackPermissionPolicy,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl PlaybackPolicy {
    #[must_use]
    pub const fn user(
        user_id: UserId,
        library_id: LibraryId,
        permissions: PlaybackPermissionPolicy,
        now_ms: i64,
    ) -> Self {
        Self {
            scope: PlaybackPolicyScope::User(user_id),
            library_id,
            permissions,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        }
    }

    #[must_use]
    pub const fn role(
        role: UserRole,
        library_id: LibraryId,
        permissions: PlaybackPermissionPolicy,
        now_ms: i64,
    ) -> Self {
        Self {
            scope: PlaybackPolicyScope::Role(role),
            library_id,
            permissions,
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        }
    }
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
    pub fn from_effective_library_access(access: EffectiveLibraryAccess) -> Self {
        match access.reason {
            EffectiveLibraryAccessReason::SingleAdminMode => Self {
                library_id: access.library_id,
                library_access: access.access,
                permissions: PlaybackPermissionPolicy::administrator_defaults(),
                reason: EffectivePlaybackPolicyReason::SingleAdminMode,
            },
            EffectiveLibraryAccessReason::AdministratorRole => {
                Self::administrator(access.library_id, access.access)
            }
            EffectiveLibraryAccessReason::UserPolicy
            | EffectiveLibraryAccessReason::RolePolicy
            | EffectiveLibraryAccessReason::NoPolicy => {
                Self::from_library_access(access.library_id, access.access)
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

#[must_use]
pub fn effective_playback_policy(
    user_id: UserId,
    roles: &[UserRole],
    library_access: EffectiveLibraryAccess,
    policies: &[PlaybackPolicy],
) -> EffectivePlaybackPolicy {
    if !library_access.access.allows_play() {
        return EffectivePlaybackPolicy::from_library_access(
            library_access.library_id,
            library_access.access,
        );
    }

    if roles.contains(&UserRole::Administrator)
        || matches!(
            library_access.reason,
            EffectiveLibraryAccessReason::AdministratorRole
                | EffectiveLibraryAccessReason::SingleAdminMode
        )
    {
        return EffectivePlaybackPolicy::from_effective_library_access(library_access);
    }

    let mut role_policy = None;
    let mut user_policy = None;

    for policy in policies
        .iter()
        .filter(|policy| policy.library_id == library_access.library_id)
    {
        match policy.scope {
            PlaybackPolicyScope::User(policy_user_id) if policy_user_id == user_id => {
                user_policy = Some(policy.permissions);
            }
            PlaybackPolicyScope::Role(role) if roles.contains(&role) => {
                role_policy = Some(match role_policy {
                    Some(current) => restrictive_playback_policy(current, policy.permissions),
                    None => policy.permissions,
                });
            }
            _ => {}
        }
    }

    if let Some(permissions) = user_policy {
        return EffectivePlaybackPolicy {
            library_id: library_access.library_id,
            library_access: library_access.access,
            permissions,
            reason: EffectivePlaybackPolicyReason::UserPolicy,
        };
    }

    if let Some(permissions) = role_policy {
        return EffectivePlaybackPolicy {
            library_id: library_access.library_id,
            library_access: library_access.access,
            permissions,
            reason: EffectivePlaybackPolicyReason::RolePolicy,
        };
    }

    EffectivePlaybackPolicy::from_effective_library_access(library_access)
}

#[must_use]
pub const fn restrictive_playback_policy(
    left: PlaybackPermissionPolicy,
    right: PlaybackPermissionPolicy,
) -> PlaybackPermissionPolicy {
    PlaybackPermissionPolicy {
        allow_media_playback: left.allow_media_playback && right.allow_media_playback,
        allow_direct_play: left.allow_direct_play && right.allow_direct_play,
        allow_remux: left.allow_remux && right.allow_remux,
        allow_audio_transcode: left.allow_audio_transcode && right.allow_audio_transcode,
        allow_video_transcode: left.allow_video_transcode && right.allow_video_transcode,
        allow_remote_playback: left.allow_remote_playback && right.allow_remote_playback,
        allow_remote_control: left.allow_remote_control && right.allow_remote_control,
        allow_cast: left.allow_cast && right.allow_cast,
        max_streaming_bitrate: min_optional_bitrate(
            left.max_streaming_bitrate,
            right.max_streaming_bitrate,
        ),
        max_remote_bitrate: min_optional_bitrate(left.max_remote_bitrate, right.max_remote_bitrate),
    }
}

const fn min_optional_bitrate(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    match (left, right) {
        (Some(left), Some(right)) if left <= right => Some(left),
        (Some(_), Some(right)) => Some(right),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EffectiveLibraryAccessReason;

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

    #[test]
    fn effective_playback_policy_uses_role_policy_after_library_access() {
        let user_id = UserId::new();
        let library_id = LibraryId::new();
        let library_access = EffectiveLibraryAccess {
            library_id,
            access: LibraryAccessLevel::Play,
            reason: EffectiveLibraryAccessReason::RolePolicy,
        };
        let mut permissions = PlaybackPermissionPolicy::current_playback_defaults();
        permissions.allow_remux = false;

        let policy = effective_playback_policy(
            user_id,
            &[UserRole::Viewer],
            library_access,
            &[PlaybackPolicy {
                scope: PlaybackPolicyScope::Role(UserRole::Viewer),
                library_id,
                permissions,
                created_at_ms: 1,
                updated_at_ms: 1,
            }],
        );

        assert_eq!(policy.reason, EffectivePlaybackPolicyReason::RolePolicy);
        assert_eq!(
            policy.check(PlaybackPermission::Remux).reason,
            PlaybackPermissionDecisionReason::RemuxDisabled
        );
        assert!(policy.check(PlaybackPermission::DirectPlay).allowed);
    }

    #[test]
    fn effective_playback_policy_user_policy_overrides_role_policy() {
        let user_id = UserId::new();
        let library_id = LibraryId::new();
        let library_access = EffectiveLibraryAccess {
            library_id,
            access: LibraryAccessLevel::Play,
            reason: EffectiveLibraryAccessReason::UserPolicy,
        };
        let mut role_permissions = PlaybackPermissionPolicy::current_playback_defaults();
        role_permissions.allow_remux = false;
        let mut user_permissions = PlaybackPermissionPolicy::current_playback_defaults();
        user_permissions.allow_cast = true;

        let policy = effective_playback_policy(
            user_id,
            &[UserRole::Viewer],
            library_access,
            &[
                PlaybackPolicy::role(UserRole::Viewer, library_id, role_permissions, 1),
                PlaybackPolicy::user(user_id, library_id, user_permissions, 2),
            ],
        );

        assert_eq!(policy.reason, EffectivePlaybackPolicyReason::UserPolicy);
        assert!(policy.check(PlaybackPermission::Remux).allowed);
        assert!(policy.check(PlaybackPermission::Cast).allowed);
    }

    #[test]
    fn effective_playback_policy_library_access_still_gates_playback_policy() {
        let user_id = UserId::new();
        let library_id = LibraryId::new();
        let library_access = EffectiveLibraryAccess {
            library_id,
            access: LibraryAccessLevel::Browse,
            reason: EffectiveLibraryAccessReason::UserPolicy,
        };

        let policy = effective_playback_policy(
            user_id,
            &[UserRole::Viewer],
            library_access,
            &[PlaybackPolicy::user(
                user_id,
                library_id,
                PlaybackPermissionPolicy::administrator_defaults(),
                1,
            )],
        );

        assert_eq!(policy.reason, EffectivePlaybackPolicyReason::NoPlayAccess);
        assert_eq!(
            policy.check(PlaybackPermission::DirectPlay).reason,
            PlaybackPermissionDecisionReason::LibraryAccessDoesNotAllowPlay
        );
    }
}
