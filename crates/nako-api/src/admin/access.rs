use nako_client_protocol::PageInfo;
use nako_core::{
    LibraryAccessLevel, LibraryAccessPolicy, LibraryAccessPolicyScope, LibraryId, User, UserId,
    UserRole, UserStatus,
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
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

impl AdminAccessUserRecord {
    #[must_use]
    pub fn from_user(user: User, roles: Vec<UserRole>, bootstrap: bool) -> Self {
        Self {
            user_id: user.id,
            principal_id: user.principal_id.to_string(),
            username: user.username,
            display_name: user.display_name,
            status: user.status,
            roles,
            bootstrap,
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
