use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryAccessLevel, LibraryAccessPolicy, LibraryAccessPolicyScope, LibraryId,
    PlaybackPermissionPolicy, PlaybackPolicy, PlaybackPolicyScope, User, UserId, UserInvitationId,
    UserInvitationRecord, UserInvitationStatus, UserRole, UserStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessUserListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub users: Vec<AdminAccessUserRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessUserResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub user: AdminAccessUserRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminAccessUserRecord {
    pub user_id: UserId,
    pub principal_id: String,
    pub username: String,
    pub display_name: String,
    pub status: UserStatus,
    pub roles: Vec<UserRole>,
    pub bootstrap: bool,
    pub local_password_configured: bool,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl AdminAccessUserRecord {
    #[must_use]
    pub fn from_user(
        user: User,
        roles: Vec<UserRole>,
        bootstrap: bool,
        local_password_configured: bool,
    ) -> Self {
        Self {
            user_id: user.id,
            principal_id: user.principal_id.to_string(),
            username: user.username,
            display_name: user.display_name,
            status: user.status,
            roles,
            bootstrap,
            local_password_configured,
            created_at_ms: user.created_at_ms,
            updated_at_ms: user.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCreateUserRequest {
    pub username: String,
    pub display_name: String,
    #[serde(default)]
    pub roles: Vec<UserRole>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpdateUserStatusRequest {
    pub status: UserStatus,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminReplaceUserRolesRequest {
    pub roles: Vec<UserRole>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminSetLocalPasswordRequest {
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLocalPasswordResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub user_id: UserId,
    pub local_password_configured: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminInvitationListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub invitations: Vec<AdminInvitationRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminInvitationResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub invitation: AdminInvitationRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCreateInvitationResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub invitation: AdminInvitationRecord,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminInvitationRecord {
    pub invitation_id: UserInvitationId,
    pub created_by_user_id: UserId,
    pub email_or_username: Option<String>,
    pub status: UserInvitationStatus,
    pub roles: Vec<UserRole>,
    pub expires_at_ms: i64,
    pub redeemed_at_ms: Option<i64>,
    pub redeemed_by_user_id: Option<UserId>,
    pub revoked_at_ms: Option<i64>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl From<UserInvitationRecord> for AdminInvitationRecord {
    fn from(value: UserInvitationRecord) -> Self {
        Self {
            invitation_id: value.id,
            created_by_user_id: value.created_by_user_id,
            email_or_username: value.email_or_username,
            status: value.status,
            roles: value.roles,
            expires_at_ms: value.expires_at_ms,
            redeemed_at_ms: value.redeemed_at_ms,
            redeemed_by_user_id: value.redeemed_by_user_id,
            revoked_at_ms: value.revoked_at_ms,
            created_at_ms: value.created_at_ms,
            updated_at_ms: value.updated_at_ms,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminCreateInvitationRequest {
    pub email_or_username: Option<String>,
    #[serde(default)]
    pub roles: Vec<UserRole>,
    pub expires_in_ms: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessPolicyListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub policies: Vec<AdminLibraryAccessPolicyRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessPolicyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub policy: AdminLibraryAccessPolicyRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessPolicyDeleteResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope")]
pub enum AdminLibraryAccessPolicyScope {
    User { user_id: UserId },
    Role { role: UserRole },
}

impl From<LibraryAccessPolicyScope> for AdminLibraryAccessPolicyScope {
    fn from(value: LibraryAccessPolicyScope) -> Self {
        match value {
            LibraryAccessPolicyScope::User(user_id) => Self::User { user_id },
            LibraryAccessPolicyScope::Role(role) => Self::Role { role },
        }
    }
}

impl From<AdminLibraryAccessPolicyScope> for LibraryAccessPolicyScope {
    fn from(value: AdminLibraryAccessPolicyScope) -> Self {
        match value {
            AdminLibraryAccessPolicyScope::User { user_id } => Self::User(user_id),
            AdminLibraryAccessPolicyScope::Role { role } => Self::Role(role),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminLibraryAccessPolicyRecord {
    pub scope: AdminLibraryAccessPolicyScope,
    pub library_id: LibraryId,
    pub access: LibraryAccessLevel,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl From<LibraryAccessPolicy> for AdminLibraryAccessPolicyRecord {
    fn from(value: LibraryAccessPolicy) -> Self {
        Self {
            scope: value.scope.into(),
            library_id: value.library_id,
            access: value.access,
            created_at_ms: value.created_at_ms,
            updated_at_ms: value.updated_at_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpsertLibraryAccessPolicyRequest {
    pub scope: AdminLibraryAccessPolicyScope,
    pub library_id: LibraryId,
    pub access: LibraryAccessLevel,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPolicyListResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub policies: Vec<AdminPlaybackPolicyRecord>,
    pub page: PageInfo,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPolicyResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub policy: AdminPlaybackPolicyRecord,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPolicyDeleteResponse {
    pub admin_api_version: String,
    pub public_api_version: String,
    pub deleted: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope")]
pub enum AdminPlaybackPolicyScope {
    User { user_id: UserId },
    Role { role: UserRole },
}

impl From<PlaybackPolicyScope> for AdminPlaybackPolicyScope {
    fn from(value: PlaybackPolicyScope) -> Self {
        match value {
            PlaybackPolicyScope::User(user_id) => Self::User { user_id },
            PlaybackPolicyScope::Role(role) => Self::Role { role },
        }
    }
}

impl From<AdminPlaybackPolicyScope> for PlaybackPolicyScope {
    fn from(value: AdminPlaybackPolicyScope) -> Self {
        match value {
            AdminPlaybackPolicyScope::User { user_id } => Self::User(user_id),
            AdminPlaybackPolicyScope::Role { role } => Self::Role(role),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPermissionPolicy {
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

impl From<PlaybackPermissionPolicy> for AdminPlaybackPermissionPolicy {
    fn from(value: PlaybackPermissionPolicy) -> Self {
        Self {
            allow_media_playback: value.allow_media_playback,
            allow_direct_play: value.allow_direct_play,
            allow_remux: value.allow_remux,
            allow_audio_transcode: value.allow_audio_transcode,
            allow_video_transcode: value.allow_video_transcode,
            allow_remote_playback: value.allow_remote_playback,
            allow_remote_control: value.allow_remote_control,
            allow_cast: value.allow_cast,
            max_streaming_bitrate: value.max_streaming_bitrate,
            max_remote_bitrate: value.max_remote_bitrate,
        }
    }
}

impl From<AdminPlaybackPermissionPolicy> for PlaybackPermissionPolicy {
    fn from(value: AdminPlaybackPermissionPolicy) -> Self {
        Self {
            allow_media_playback: value.allow_media_playback,
            allow_direct_play: value.allow_direct_play,
            allow_remux: value.allow_remux,
            allow_audio_transcode: value.allow_audio_transcode,
            allow_video_transcode: value.allow_video_transcode,
            allow_remote_playback: value.allow_remote_playback,
            allow_remote_control: value.allow_remote_control,
            allow_cast: value.allow_cast,
            max_streaming_bitrate: value.max_streaming_bitrate,
            max_remote_bitrate: value.max_remote_bitrate,
        }
    }
}

impl Default for AdminPlaybackPermissionPolicy {
    fn default() -> Self {
        PlaybackPermissionPolicy::current_playback_defaults().into()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminPlaybackPolicyRecord {
    pub scope: AdminPlaybackPolicyScope,
    pub library_id: LibraryId,
    pub permissions: AdminPlaybackPermissionPolicy,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl From<PlaybackPolicy> for AdminPlaybackPolicyRecord {
    fn from(value: PlaybackPolicy) -> Self {
        Self {
            scope: value.scope.into(),
            library_id: value.library_id,
            permissions: value.permissions.into(),
            created_at_ms: value.created_at_ms,
            updated_at_ms: value.updated_at_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AdminUpsertPlaybackPolicyRequest {
    pub scope: AdminPlaybackPolicyScope,
    pub library_id: LibraryId,
    pub permissions: AdminPlaybackPermissionPolicy,
}
