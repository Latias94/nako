use serde::{Deserialize, Serialize};

use crate::{LibraryId, UserId, UserPrincipalId};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct User {
    pub id: UserId,
    pub principal_id: UserPrincipalId,
    pub username: String,
    pub display_name: String,
    pub status: UserStatus,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Disabled,
}

impl UserStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    #[must_use]
    pub const fn can_authenticate(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Administrator,
    LibraryManager,
    Viewer,
}

impl UserRole {
    pub const ALL: [Self; 3] = [Self::Administrator, Self::LibraryManager, Self::Viewer];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Administrator => "administrator",
            Self::LibraryManager => "library_manager",
            Self::Viewer => "viewer",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "administrator" => Some(Self::Administrator),
            "library_manager" => Some(Self::LibraryManager),
            "viewer" => Some(Self::Viewer),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct RoleAssignment {
    pub user_id: UserId,
    pub role: UserRole,
    pub granted_at_ms: i64,
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
#[serde(rename_all = "snake_case")]
pub enum LibraryAccessLevel {
    #[default]
    None,
    Browse,
    Play,
    Manage,
}

impl LibraryAccessLevel {
    pub const ALL: [Self; 4] = [Self::None, Self::Browse, Self::Play, Self::Manage];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Browse => "browse",
            Self::Play => "play",
            Self::Manage => "manage",
        }
    }

    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "none" => Some(Self::None),
            "browse" => Some(Self::Browse),
            "play" => Some(Self::Play),
            "manage" => Some(Self::Manage),
            _ => None,
        }
    }

    #[must_use]
    pub const fn allows_browse(self) -> bool {
        matches!(self, Self::Browse | Self::Play | Self::Manage)
    }

    #[must_use]
    pub const fn allows_play(self) -> bool {
        matches!(self, Self::Play | Self::Manage)
    }

    #[must_use]
    pub const fn allows_manage(self) -> bool {
        matches!(self, Self::Manage)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "scope", content = "value")]
pub enum LibraryAccessPolicyScope {
    User(UserId),
    Role(UserRole),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct LibraryAccessPolicy {
    pub scope: LibraryAccessPolicyScope,
    pub library_id: LibraryId,
    pub access: LibraryAccessLevel,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct EffectiveLibraryAccess {
    pub library_id: LibraryId,
    pub access: LibraryAccessLevel,
    pub reason: EffectiveLibraryAccessReason,
}

impl EffectiveLibraryAccess {
    #[must_use]
    pub const fn single_admin(library_id: LibraryId) -> Self {
        Self {
            library_id,
            access: LibraryAccessLevel::Manage,
            reason: EffectiveLibraryAccessReason::SingleAdminMode,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectiveLibraryAccessReason {
    SingleAdminMode,
    AdministratorRole,
    UserPolicy,
    RolePolicy,
    NoPolicy,
}

#[must_use]
pub fn effective_library_access(
    user_id: UserId,
    roles: &[UserRole],
    library_id: LibraryId,
    policies: &[LibraryAccessPolicy],
) -> EffectiveLibraryAccess {
    if roles.contains(&UserRole::Administrator) {
        return EffectiveLibraryAccess {
            library_id,
            access: LibraryAccessLevel::Manage,
            reason: EffectiveLibraryAccessReason::AdministratorRole,
        };
    }

    let mut effective = EffectiveLibraryAccess {
        library_id,
        access: LibraryAccessLevel::None,
        reason: EffectiveLibraryAccessReason::NoPolicy,
    };

    for policy in policies
        .iter()
        .filter(|policy| policy.library_id == library_id)
        .filter(|policy| policy_matches_user(*policy, user_id, roles))
    {
        let reason = match policy.scope {
            LibraryAccessPolicyScope::User(_) => EffectiveLibraryAccessReason::UserPolicy,
            LibraryAccessPolicyScope::Role(_) => EffectiveLibraryAccessReason::RolePolicy,
        };

        if policy.access > effective.access
            || (policy.access == effective.access
                && reason == EffectiveLibraryAccessReason::UserPolicy)
        {
            effective.access = policy.access;
            effective.reason = reason;
        }
    }

    effective
}

fn policy_matches_user(policy: &LibraryAccessPolicy, user_id: UserId, roles: &[UserRole]) -> bool {
    match policy.scope {
        LibraryAccessPolicyScope::User(policy_user_id) => policy_user_id == user_id,
        LibraryAccessPolicyScope::Role(role) => roles.contains(&role),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn administrator_role_has_manage_access_without_library_policy() {
        let user_id = UserId::new();
        let library_id = LibraryId::new();

        let access = effective_library_access(user_id, &[UserRole::Administrator], library_id, &[]);

        assert_eq!(access.access, LibraryAccessLevel::Manage);
        assert_eq!(
            access.reason,
            EffectiveLibraryAccessReason::AdministratorRole
        );
        assert!(access.access.allows_browse());
        assert!(access.access.allows_play());
        assert!(access.access.allows_manage());
    }

    #[test]
    fn effective_access_uses_highest_matching_policy() {
        let user_id = UserId::new();
        let other_user_id = UserId::new();
        let library_id = LibraryId::new();
        let other_library_id = LibraryId::new();
        let policies = vec![
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::Role(UserRole::Viewer),
                library_id,
                access: LibraryAccessLevel::Play,
                created_at_ms: 1,
                updated_at_ms: 1,
            },
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::User(user_id),
                library_id,
                access: LibraryAccessLevel::Browse,
                created_at_ms: 2,
                updated_at_ms: 2,
            },
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::User(other_user_id),
                library_id,
                access: LibraryAccessLevel::Manage,
                created_at_ms: 3,
                updated_at_ms: 3,
            },
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::Role(UserRole::Viewer),
                library_id: other_library_id,
                access: LibraryAccessLevel::Manage,
                created_at_ms: 4,
                updated_at_ms: 4,
            },
        ];

        let access = effective_library_access(user_id, &[UserRole::Viewer], library_id, &policies);

        assert_eq!(access.access, LibraryAccessLevel::Play);
        assert_eq!(access.reason, EffectiveLibraryAccessReason::RolePolicy);
    }

    #[test]
    fn direct_user_policy_wins_ties_against_role_policy() {
        let user_id = UserId::new();
        let library_id = LibraryId::new();
        let policies = vec![
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::Role(UserRole::Viewer),
                library_id,
                access: LibraryAccessLevel::Play,
                created_at_ms: 1,
                updated_at_ms: 1,
            },
            LibraryAccessPolicy {
                scope: LibraryAccessPolicyScope::User(user_id),
                library_id,
                access: LibraryAccessLevel::Play,
                created_at_ms: 2,
                updated_at_ms: 2,
            },
        ];

        let access = effective_library_access(user_id, &[UserRole::Viewer], library_id, &policies);

        assert_eq!(access.access, LibraryAccessLevel::Play);
        assert_eq!(access.reason, EffectiveLibraryAccessReason::UserPolicy);
    }

    #[test]
    fn no_matching_policy_means_no_access() {
        let access =
            effective_library_access(UserId::new(), &[UserRole::Viewer], LibraryId::new(), &[]);

        assert_eq!(access.access, LibraryAccessLevel::None);
        assert_eq!(access.reason, EffectiveLibraryAccessReason::NoPolicy);
        assert!(!access.access.allows_browse());
        assert!(!access.access.allows_play());
        assert!(!access.access.allows_manage());
    }
}
