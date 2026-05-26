use async_trait::async_trait;

use super::PageRequest;
use crate::Result;
use crate::{
    EffectiveLibraryAccess, LibraryAccessPolicy, LibraryAccessPolicyScope, LibraryId,
    RoleAssignment, User, UserId, UserPrincipalId, UserRole,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LibraryAccessPolicyFilter {
    pub user_id: Option<UserId>,
    pub role: Option<UserRole>,
    pub library_id: Option<LibraryId>,
}

#[async_trait]
pub trait IdentityAccessRepository: Send + Sync {
    async fn upsert_user(&self, user: &User) -> Result<()>;

    async fn get_user(&self, id: UserId) -> Result<Option<User>>;

    async fn get_user_by_principal(&self, principal_id: &UserPrincipalId) -> Result<Option<User>>;

    async fn list_users(&self, page: PageRequest) -> Result<Vec<User>>;

    async fn replace_role_assignments(
        &self,
        user_id: UserId,
        assignments: &[RoleAssignment],
    ) -> Result<()>;

    async fn list_role_assignments(&self, user_id: UserId) -> Result<Vec<RoleAssignment>>;

    async fn upsert_library_access_policy(&self, policy: &LibraryAccessPolicy) -> Result<()>;

    async fn delete_library_access_policy(
        &self,
        scope: LibraryAccessPolicyScope,
        library_id: LibraryId,
    ) -> Result<()>;

    async fn list_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<LibraryAccessPolicy>>;

    async fn resolve_effective_library_access(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectiveLibraryAccess>;
}
