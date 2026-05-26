use super::{SqliteStore, codec::*};
use nako_core::*;
use sqlx::sqlite::SqliteRow;

const USER_SELECT: &str = r#"
            SELECT
                id,
                principal_id,
                username,
                display_name,
                status,
                created_at_ms,
                updated_at_ms
            FROM users
            "#;

#[async_trait::async_trait]
impl IdentityAccessRepository for SqliteStore {
    async fn upsert_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (
                id,
                principal_id,
                username,
                normalized_username,
                display_name,
                status,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                principal_id = excluded.principal_id,
                username = excluded.username,
                normalized_username = excluded.normalized_username,
                display_name = excluded.display_name,
                status = excluded.status,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(user.id.to_string())
        .bind(user.principal_id.as_str())
        .bind(&user.username)
        .bind(normalized_username(&user.username)?)
        .bind(&user.display_name)
        .bind(user.status.as_str())
        .bind(user.created_at_ms)
        .bind(user.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_user(&self, id: UserId) -> Result<Option<User>> {
        let query = format!("{USER_SELECT} WHERE id = ?1");
        let row = sqlx::query(&query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user).transpose()
    }

    async fn get_user_by_principal(&self, principal_id: &UserPrincipalId) -> Result<Option<User>> {
        let query = format!("{USER_SELECT} WHERE principal_id = ?1");
        let row = sqlx::query(&query)
            .bind(principal_id.as_str())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user).transpose()
    }

    async fn list_users(&self, page: PageRequest) -> Result<Vec<User>> {
        let page = page.clamped();
        let query = format!(
            r#"
            {USER_SELECT}
            ORDER BY normalized_username ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#
        );
        let rows = sqlx::query(&query)
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_user).collect()
    }

    async fn replace_role_assignments(
        &self,
        user_id: UserId,
        assignments: &[RoleAssignment],
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        sqlx::query("DELETE FROM user_role_assignments WHERE user_id = ?1")
            .bind(user_id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for assignment in assignments {
            if assignment.user_id != user_id {
                return Err(NakoError::InvalidInput {
                    message: "role assignment user_id must match replacement target".to_owned(),
                });
            }
            sqlx::query(
                r#"
                INSERT INTO user_role_assignments (user_id, role, granted_at_ms)
                VALUES (?1, ?2, ?3)
                "#,
            )
            .bind(assignment.user_id.to_string())
            .bind(assignment.role.as_str())
            .bind(assignment.granted_at_ms)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)
    }

    async fn list_role_assignments(&self, user_id: UserId) -> Result<Vec<RoleAssignment>> {
        let rows = sqlx::query(
            r#"
            SELECT user_id, role, granted_at_ms
            FROM user_role_assignments
            WHERE user_id = ?1
            ORDER BY role ASC, user_id ASC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_role_assignment).collect()
    }

    async fn upsert_library_access_policy(&self, policy: &LibraryAccessPolicy) -> Result<()> {
        match policy.scope {
            LibraryAccessPolicyScope::User(user_id) => {
                sqlx::query(
                    r#"
                    INSERT INTO user_library_access_policies (
                        user_id, library_id, access, created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    ON CONFLICT(user_id, library_id) DO UPDATE SET
                        access = excluded.access,
                        updated_at_ms = excluded.updated_at_ms
                    "#,
                )
                .bind(user_id.to_string())
                .bind(policy.library_id.to_string())
                .bind(policy.access.as_str())
                .bind(policy.created_at_ms)
                .bind(policy.updated_at_ms)
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
            LibraryAccessPolicyScope::Role(role) => {
                sqlx::query(
                    r#"
                    INSERT INTO role_library_access_policies (
                        role, library_id, access, created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5)
                    ON CONFLICT(role, library_id) DO UPDATE SET
                        access = excluded.access,
                        updated_at_ms = excluded.updated_at_ms
                    "#,
                )
                .bind(role.as_str())
                .bind(policy.library_id.to_string())
                .bind(policy.access.as_str())
                .bind(policy.created_at_ms)
                .bind(policy.updated_at_ms)
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
        }

        Ok(())
    }

    async fn delete_library_access_policy(
        &self,
        scope: LibraryAccessPolicyScope,
        library_id: LibraryId,
    ) -> Result<()> {
        match scope {
            LibraryAccessPolicyScope::User(user_id) => {
                sqlx::query(
                    "DELETE FROM user_library_access_policies WHERE user_id = ?1 AND library_id = ?2",
                )
                .bind(user_id.to_string())
                .bind(library_id.to_string())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
            LibraryAccessPolicyScope::Role(role) => {
                sqlx::query(
                    "DELETE FROM role_library_access_policies WHERE role = ?1 AND library_id = ?2",
                )
                .bind(role.as_str())
                .bind(library_id.to_string())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
        }

        Ok(())
    }

    async fn list_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<LibraryAccessPolicy>> {
        let mut policies = Vec::new();

        if filter.role.is_none() {
            policies.extend(self.list_user_library_access_policies(filter).await?);
        }
        if filter.user_id.is_none() {
            policies.extend(self.list_role_library_access_policies(filter).await?);
        }

        policies.sort_by_key(policy_sort_key);
        Ok(page_vec(policies, page))
    }

    async fn resolve_effective_library_access(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectiveLibraryAccess> {
        let roles = self
            .list_role_assignments(user_id)
            .await?
            .into_iter()
            .map(|assignment| assignment.role)
            .collect::<Vec<_>>();
        let filter = LibraryAccessPolicyFilter {
            user_id: None,
            role: None,
            library_id: Some(library_id),
        };
        let mut policies = self.list_user_library_access_policies(filter).await?;
        policies.extend(self.list_role_library_access_policies(filter).await?);

        Ok(effective_library_access(
            user_id, &roles, library_id, &policies,
        ))
    }
}

impl SqliteStore {
    async fn list_user_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
    ) -> Result<Vec<LibraryAccessPolicy>> {
        let rows = sqlx::query(
            r#"
            SELECT user_id, library_id, access, created_at_ms, updated_at_ms
            FROM user_library_access_policies
            WHERE (?1 IS NULL OR user_id = ?1)
              AND (?2 IS NULL OR library_id = ?2)
            ORDER BY library_id ASC, user_id ASC
            "#,
        )
        .bind(filter.user_id.map(|id| id.to_string()))
        .bind(filter.library_id.map(|id| id.to_string()))
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_user_policy).collect()
    }

    async fn list_role_library_access_policies(
        &self,
        filter: LibraryAccessPolicyFilter,
    ) -> Result<Vec<LibraryAccessPolicy>> {
        let rows = sqlx::query(
            r#"
            SELECT role, library_id, access, created_at_ms, updated_at_ms
            FROM role_library_access_policies
            WHERE (?1 IS NULL OR role = ?1)
              AND (?2 IS NULL OR library_id = ?2)
            ORDER BY library_id ASC, role ASC
            "#,
        )
        .bind(filter.role.map(|role| role.as_str()))
        .bind(filter.library_id.map(|id| id.to_string()))
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_role_policy).collect()
    }
}

fn row_to_user(row: SqliteRow) -> Result<User> {
    Ok(User {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        username: row_get(&row, "username")?,
        display_name: row_get(&row, "display_name")?,
        status: parse_user_status(row_get(&row, "status")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_role_assignment(row: SqliteRow) -> Result<RoleAssignment> {
    Ok(RoleAssignment {
        user_id: parse_id(row_get::<String>(&row, "user_id")?)?,
        role: parse_user_role(row_get(&row, "role")?)?,
        granted_at_ms: row_get(&row, "granted_at_ms")?,
    })
}

fn row_to_user_policy(row: SqliteRow) -> Result<LibraryAccessPolicy> {
    Ok(LibraryAccessPolicy {
        scope: LibraryAccessPolicyScope::User(parse_id(row_get::<String>(&row, "user_id")?)?),
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        access: parse_library_access_level(row_get(&row, "access")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_role_policy(row: SqliteRow) -> Result<LibraryAccessPolicy> {
    Ok(LibraryAccessPolicy {
        scope: LibraryAccessPolicyScope::Role(parse_user_role(row_get(&row, "role")?)?),
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        access: parse_library_access_level(row_get(&row, "access")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn normalized_username(username: &str) -> Result<String> {
    let normalized = username.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "username cannot be empty".to_owned(),
        });
    }
    if normalized.chars().any(char::is_control) {
        return Err(NakoError::InvalidInput {
            message: "username cannot contain control characters".to_owned(),
        });
    }

    Ok(normalized)
}

fn parse_user_status(value: String) -> Result<UserStatus> {
    UserStatus::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown user status stored in SQLite database: {value}"),
    })
}

fn parse_user_role(value: String) -> Result<UserRole> {
    UserRole::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown user role stored in SQLite database: {value}"),
    })
}

fn parse_library_access_level(value: String) -> Result<LibraryAccessLevel> {
    LibraryAccessLevel::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown Library Access level stored in SQLite database: {value}"),
    })
}

fn policy_sort_key(policy: &LibraryAccessPolicy) -> (LibraryId, &'static str, String) {
    match policy.scope {
        LibraryAccessPolicyScope::User(user_id) => (policy.library_id, "user", user_id.to_string()),
        LibraryAccessPolicyScope::Role(role) => {
            (policy.library_id, "role", role.as_str().to_owned())
        }
    }
}

fn page_vec<T>(values: Vec<T>, page: PageRequest) -> Vec<T> {
    let page = page.clamped();
    let start = usize::try_from(page.offset).unwrap_or(usize::MAX);
    let limit = usize::try_from(page.limit).unwrap_or(usize::MAX);

    values.into_iter().skip(start).take(limit).collect()
}
