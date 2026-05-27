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

const LOCAL_CREDENTIAL_SELECT: &str = r#"
            SELECT user_id, password_hash, updated_at_ms
            FROM local_user_credentials
            "#;

const USER_SESSION_SELECT: &str = r#"
            SELECT
                id,
                user_id,
                token_hash,
                created_at_ms,
                last_seen_at_ms,
                expires_at_ms,
                revoked_at_ms
            FROM user_sessions
            "#;

const USER_INVITATION_SELECT: &str = r#"
            SELECT
                id,
                created_by_user_id,
                email_or_username,
                token_hash,
                roles_json,
                status,
                expires_at_ms,
                redeemed_at_ms,
                redeemed_by_user_id,
                revoked_at_ms,
                created_at_ms,
                updated_at_ms
            FROM user_invitations
            "#;

#[async_trait::async_trait]
impl IdentityAccessRepository for SqliteStore {
    async fn create_user_invitation(&self, invitation: &UserInvitationRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_invitations (
                id,
                created_by_user_id,
                email_or_username,
                token_hash,
                roles_json,
                status,
                expires_at_ms,
                redeemed_at_ms,
                redeemed_by_user_id,
                revoked_at_ms,
                created_at_ms,
                updated_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
        )
        .bind(invitation.id.to_string())
        .bind(invitation.created_by_user_id.to_string())
        .bind(&invitation.email_or_username)
        .bind(&invitation.token_hash)
        .bind(invitation_roles_json(&invitation.roles)?)
        .bind(invitation.status.as_str())
        .bind(invitation.expires_at_ms)
        .bind(invitation.redeemed_at_ms)
        .bind(invitation.redeemed_by_user_id.map(|id| id.to_string()))
        .bind(invitation.revoked_at_ms)
        .bind(invitation.created_at_ms)
        .bind(invitation.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_user_invitation(
        &self,
        invitation_id: UserInvitationId,
    ) -> Result<Option<UserInvitationRecord>> {
        let query = format!("{USER_INVITATION_SELECT} WHERE id = ?1");
        let row = sqlx::query(&query)
            .bind(invitation_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_invitation).transpose()
    }

    async fn get_user_invitation_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserInvitationRecord>> {
        let query = format!("{USER_INVITATION_SELECT} WHERE token_hash = ?1");
        let row = sqlx::query(&query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_invitation).transpose()
    }

    async fn list_user_invitations(&self, page: PageRequest) -> Result<Vec<UserInvitationRecord>> {
        let page = page.clamped();
        let query = format!(
            r#"
            {USER_INVITATION_SELECT}
            ORDER BY created_at_ms DESC, id ASC
            LIMIT ?1 OFFSET ?2
            "#
        );
        let rows = sqlx::query(&query)
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_user_invitation).collect()
    }

    async fn mark_user_invitation_redeemed(
        &self,
        invitation_id: UserInvitationId,
        redeemed_by_user_id: UserId,
        redeemed_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = 'redeemed',
                redeemed_at_ms = ?2,
                redeemed_by_user_id = ?3,
                updated_at_ms = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
              AND status = 'pending'
              AND redeemed_at_ms IS NULL
              AND revoked_at_ms IS NULL
            "#,
        )
        .bind(invitation_id.to_string())
        .bind(redeemed_at_ms)
        .bind(redeemed_by_user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_invitation(invitation_id).await
    }

    async fn redeem_user_invitation(
        &self,
        invitation_id: UserInvitationId,
        user: &User,
        credential: &LocalCredentialRecord,
        assignments: &[RoleAssignment],
        redeemed_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
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
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        for assignment in assignments {
            if assignment.user_id != user.id {
                return Err(NakoError::InvalidInput {
                    message: "role assignment user_id must match redeemed user".to_owned(),
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

        if credential.user_id != user.id {
            return Err(NakoError::InvalidInput {
                message: "credential user_id must match redeemed user".to_owned(),
            });
        }
        sqlx::query(
            r#"
            INSERT INTO local_user_credentials (user_id, password_hash, updated_at_ms)
            VALUES (?1, ?2, ?3)
            "#,
        )
        .bind(credential.user_id.to_string())
        .bind(&credential.password_hash)
        .bind(credential.updated_at_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = 'redeemed',
                redeemed_at_ms = ?2,
                redeemed_by_user_id = ?3,
                updated_at_ms = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
              AND status = 'pending'
              AND redeemed_at_ms IS NULL
              AND revoked_at_ms IS NULL
            "#,
        )
        .bind(invitation_id.to_string())
        .bind(redeemed_at_ms)
        .bind(user.id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        self.get_user_invitation(invitation_id).await
    }

    async fn revoke_user_invitation(
        &self,
        invitation_id: UserInvitationId,
        revoked_at_ms: i64,
    ) -> Result<Option<UserInvitationRecord>> {
        sqlx::query(
            r#"
            UPDATE user_invitations
            SET status = 'revoked',
                revoked_at_ms = ?2,
                updated_at_ms = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
              AND status = 'pending'
              AND redeemed_at_ms IS NULL
              AND revoked_at_ms IS NULL
            "#,
        )
        .bind(invitation_id.to_string())
        .bind(revoked_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_invitation(invitation_id).await
    }

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

    async fn upsert_local_credential(&self, credential: &LocalCredentialRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO local_user_credentials (user_id, password_hash, updated_at_ms)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(user_id) DO UPDATE SET
                password_hash = excluded.password_hash,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(credential.user_id.to_string())
        .bind(&credential.password_hash)
        .bind(credential.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_local_credential_by_user(
        &self,
        user_id: UserId,
    ) -> Result<Option<LocalCredentialRecord>> {
        let query = format!("{LOCAL_CREDENTIAL_SELECT} WHERE user_id = ?1");
        let row = sqlx::query(&query)
            .bind(user_id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_local_credential).transpose()
    }

    async fn get_local_credential_by_username(
        &self,
        username: &str,
    ) -> Result<Option<LocalCredentialRecord>> {
        let query = format!(
            r#"
            {LOCAL_CREDENTIAL_SELECT}
            WHERE user_id = (
                SELECT id FROM users WHERE normalized_username = ?1
            )
            "#
        );
        let row = sqlx::query(&query)
            .bind(normalized_username(username)?)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_local_credential).transpose()
    }

    async fn delete_local_credential(&self, user_id: UserId) -> Result<()> {
        sqlx::query("DELETE FROM local_user_credentials WHERE user_id = ?1")
            .bind(user_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn create_user_session(&self, session: &UserSessionRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO user_sessions (
                id,
                user_id,
                token_hash,
                created_at_ms,
                last_seen_at_ms,
                expires_at_ms,
                revoked_at_ms
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(session.id.to_string())
        .bind(session.user_id.to_string())
        .bind(&session.token_hash)
        .bind(session.created_at_ms)
        .bind(session.last_seen_at_ms)
        .bind(session.expires_at_ms)
        .bind(session.revoked_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_user_session(&self, id: UserSessionId) -> Result<Option<UserSessionRecord>> {
        let query = format!("{USER_SESSION_SELECT} WHERE id = ?1");
        let row = sqlx::query(&query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_session).transpose()
    }

    async fn get_user_session_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<UserSessionRecord>> {
        let query = format!("{USER_SESSION_SELECT} WHERE token_hash = ?1");
        let row = sqlx::query(&query)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_session).transpose()
    }

    async fn touch_user_session(
        &self,
        id: UserSessionId,
        last_seen_at_ms: i64,
    ) -> Result<Option<UserSessionRecord>> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_seen_at_ms = ?2,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(last_seen_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_session(id).await
    }

    async fn revoke_user_session(
        &self,
        id: UserSessionId,
        revoked_at_ms: i64,
    ) -> Result<Option<UserSessionRecord>> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET revoked_at_ms = COALESCE(revoked_at_ms, ?2),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(revoked_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_session(id).await
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

#[async_trait::async_trait]
impl PlaybackPolicyRepository for SqliteStore {
    async fn upsert_playback_policy(&self, policy: &PlaybackPolicy) -> Result<()> {
        match policy.scope {
            PlaybackPolicyScope::User(user_id) => {
                sqlx::query(
                    r#"
                    INSERT INTO user_playback_permission_policies (
                        user_id, library_id,
                        allow_media_playback, allow_direct_play, allow_remux,
                        allow_audio_transcode, allow_video_transcode,
                        allow_remote_playback, allow_remote_control, allow_cast,
                        max_streaming_bitrate, max_remote_bitrate,
                        created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    ON CONFLICT(user_id, library_id) DO UPDATE SET
                        allow_media_playback = excluded.allow_media_playback,
                        allow_direct_play = excluded.allow_direct_play,
                        allow_remux = excluded.allow_remux,
                        allow_audio_transcode = excluded.allow_audio_transcode,
                        allow_video_transcode = excluded.allow_video_transcode,
                        allow_remote_playback = excluded.allow_remote_playback,
                        allow_remote_control = excluded.allow_remote_control,
                        allow_cast = excluded.allow_cast,
                        max_streaming_bitrate = excluded.max_streaming_bitrate,
                        max_remote_bitrate = excluded.max_remote_bitrate,
                        updated_at_ms = excluded.updated_at_ms
                    "#,
                )
                .bind(user_id.to_string())
                .bind(policy.library_id.to_string())
                .bind(bool_to_i64(policy.permissions.allow_media_playback))
                .bind(bool_to_i64(policy.permissions.allow_direct_play))
                .bind(bool_to_i64(policy.permissions.allow_remux))
                .bind(bool_to_i64(policy.permissions.allow_audio_transcode))
                .bind(bool_to_i64(policy.permissions.allow_video_transcode))
                .bind(bool_to_i64(policy.permissions.allow_remote_playback))
                .bind(bool_to_i64(policy.permissions.allow_remote_control))
                .bind(bool_to_i64(policy.permissions.allow_cast))
                .bind(optional_u64_to_i64(
                    policy.permissions.max_streaming_bitrate,
                )?)
                .bind(optional_u64_to_i64(policy.permissions.max_remote_bitrate)?)
                .bind(policy.created_at_ms)
                .bind(policy.updated_at_ms)
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
            PlaybackPolicyScope::Role(role) => {
                sqlx::query(
                    r#"
                    INSERT INTO role_playback_permission_policies (
                        role, library_id,
                        allow_media_playback, allow_direct_play, allow_remux,
                        allow_audio_transcode, allow_video_transcode,
                        allow_remote_playback, allow_remote_control, allow_cast,
                        max_streaming_bitrate, max_remote_bitrate,
                        created_at_ms, updated_at_ms
                    )
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                    ON CONFLICT(role, library_id) DO UPDATE SET
                        allow_media_playback = excluded.allow_media_playback,
                        allow_direct_play = excluded.allow_direct_play,
                        allow_remux = excluded.allow_remux,
                        allow_audio_transcode = excluded.allow_audio_transcode,
                        allow_video_transcode = excluded.allow_video_transcode,
                        allow_remote_playback = excluded.allow_remote_playback,
                        allow_remote_control = excluded.allow_remote_control,
                        allow_cast = excluded.allow_cast,
                        max_streaming_bitrate = excluded.max_streaming_bitrate,
                        max_remote_bitrate = excluded.max_remote_bitrate,
                        updated_at_ms = excluded.updated_at_ms
                    "#,
                )
                .bind(role.as_str())
                .bind(policy.library_id.to_string())
                .bind(bool_to_i64(policy.permissions.allow_media_playback))
                .bind(bool_to_i64(policy.permissions.allow_direct_play))
                .bind(bool_to_i64(policy.permissions.allow_remux))
                .bind(bool_to_i64(policy.permissions.allow_audio_transcode))
                .bind(bool_to_i64(policy.permissions.allow_video_transcode))
                .bind(bool_to_i64(policy.permissions.allow_remote_playback))
                .bind(bool_to_i64(policy.permissions.allow_remote_control))
                .bind(bool_to_i64(policy.permissions.allow_cast))
                .bind(optional_u64_to_i64(
                    policy.permissions.max_streaming_bitrate,
                )?)
                .bind(optional_u64_to_i64(policy.permissions.max_remote_bitrate)?)
                .bind(policy.created_at_ms)
                .bind(policy.updated_at_ms)
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
        }

        Ok(())
    }

    async fn delete_playback_policy(
        &self,
        scope: PlaybackPolicyScope,
        library_id: LibraryId,
    ) -> Result<()> {
        match scope {
            PlaybackPolicyScope::User(user_id) => {
                sqlx::query(
                    "DELETE FROM user_playback_permission_policies WHERE user_id = ?1 AND library_id = ?2",
                )
                .bind(user_id.to_string())
                .bind(library_id.to_string())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;
            }
            PlaybackPolicyScope::Role(role) => {
                sqlx::query(
                    "DELETE FROM role_playback_permission_policies WHERE role = ?1 AND library_id = ?2",
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

    async fn list_playback_policies(
        &self,
        filter: PlaybackPolicyFilter,
        page: PageRequest,
    ) -> Result<Vec<PlaybackPolicy>> {
        let mut policies = Vec::new();

        if filter.role.is_none() {
            policies.extend(self.list_user_playback_policies(filter).await?);
        }
        if filter.user_id.is_none() {
            policies.extend(self.list_role_playback_policies(filter).await?);
        }

        policies.sort_by_key(playback_policy_sort_key);
        Ok(page_vec(policies, page))
    }

    async fn resolve_effective_playback_policy(
        &self,
        user_id: UserId,
        library_id: LibraryId,
    ) -> Result<EffectivePlaybackPolicy> {
        let roles = self
            .list_role_assignments(user_id)
            .await?
            .into_iter()
            .map(|assignment| assignment.role)
            .collect::<Vec<_>>();
        let library_access = self
            .resolve_effective_library_access(user_id, library_id)
            .await?;
        let filter = PlaybackPolicyFilter {
            user_id: None,
            role: None,
            library_id: Some(library_id),
        };
        let mut policies = self.list_user_playback_policies(filter).await?;
        policies.extend(self.list_role_playback_policies(filter).await?);

        Ok(effective_playback_policy(
            user_id,
            &roles,
            library_access,
            &policies,
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

    async fn list_user_playback_policies(
        &self,
        filter: PlaybackPolicyFilter,
    ) -> Result<Vec<PlaybackPolicy>> {
        let rows = sqlx::query(
            r#"
            SELECT
                user_id, library_id,
                allow_media_playback, allow_direct_play, allow_remux,
                allow_audio_transcode, allow_video_transcode,
                allow_remote_playback, allow_remote_control, allow_cast,
                max_streaming_bitrate, max_remote_bitrate,
                created_at_ms, updated_at_ms
            FROM user_playback_permission_policies
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

        rows.into_iter().map(row_to_user_playback_policy).collect()
    }

    async fn list_role_playback_policies(
        &self,
        filter: PlaybackPolicyFilter,
    ) -> Result<Vec<PlaybackPolicy>> {
        let rows = sqlx::query(
            r#"
            SELECT
                role, library_id,
                allow_media_playback, allow_direct_play, allow_remux,
                allow_audio_transcode, allow_video_transcode,
                allow_remote_playback, allow_remote_control, allow_cast,
                max_streaming_bitrate, max_remote_bitrate,
                created_at_ms, updated_at_ms
            FROM role_playback_permission_policies
            WHERE (?1 IS NULL OR role = ?1)
              AND (?2 IS NULL OR library_id = ?2)
            ORDER BY library_id ASC, role ASC
            "#,
        )
        .bind(filter.role.map(|role| role.as_str().to_owned()))
        .bind(filter.library_id.map(|id| id.to_string()))
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_role_playback_policy).collect()
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

fn row_to_local_credential(row: SqliteRow) -> Result<LocalCredentialRecord> {
    Ok(LocalCredentialRecord {
        user_id: parse_id(row_get::<String>(&row, "user_id")?)?,
        password_hash: row_get(&row, "password_hash")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_user_session(row: SqliteRow) -> Result<UserSessionRecord> {
    Ok(UserSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        user_id: parse_id(row_get::<String>(&row, "user_id")?)?,
        token_hash: row_get(&row, "token_hash")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        last_seen_at_ms: row_get(&row, "last_seen_at_ms")?,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        revoked_at_ms: row_get(&row, "revoked_at_ms")?,
    })
}

fn row_to_user_invitation(row: SqliteRow) -> Result<UserInvitationRecord> {
    Ok(UserInvitationRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        created_by_user_id: parse_id(row_get::<String>(&row, "created_by_user_id")?)?,
        email_or_username: row_get(&row, "email_or_username")?,
        token_hash: row_get(&row, "token_hash")?,
        roles: invitation_roles_from_json(row_get(&row, "roles_json")?)?,
        status: parse_user_invitation_status(row_get(&row, "status")?)?,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        redeemed_at_ms: row_get(&row, "redeemed_at_ms")?,
        redeemed_by_user_id: row_get::<Option<String>>(&row, "redeemed_by_user_id")?
            .map(parse_id)
            .transpose()?,
        revoked_at_ms: row_get(&row, "revoked_at_ms")?,
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

fn row_to_user_playback_policy(row: SqliteRow) -> Result<PlaybackPolicy> {
    Ok(PlaybackPolicy {
        scope: PlaybackPolicyScope::User(parse_id(row_get::<String>(&row, "user_id")?)?),
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        permissions: row_to_playback_permission_policy(&row)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_role_playback_policy(row: SqliteRow) -> Result<PlaybackPolicy> {
    Ok(PlaybackPolicy {
        scope: PlaybackPolicyScope::Role(parse_user_role(row_get(&row, "role")?)?),
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        permissions: row_to_playback_permission_policy(&row)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_playback_permission_policy(row: &SqliteRow) -> Result<PlaybackPermissionPolicy> {
    Ok(PlaybackPermissionPolicy {
        allow_media_playback: i64_to_bool(row_get(row, "allow_media_playback")?)?,
        allow_direct_play: i64_to_bool(row_get(row, "allow_direct_play")?)?,
        allow_remux: i64_to_bool(row_get(row, "allow_remux")?)?,
        allow_audio_transcode: i64_to_bool(row_get(row, "allow_audio_transcode")?)?,
        allow_video_transcode: i64_to_bool(row_get(row, "allow_video_transcode")?)?,
        allow_remote_playback: i64_to_bool(row_get(row, "allow_remote_playback")?)?,
        allow_remote_control: i64_to_bool(row_get(row, "allow_remote_control")?)?,
        allow_cast: i64_to_bool(row_get(row, "allow_cast")?)?,
        max_streaming_bitrate: optional_i64_to_u64(row_get(row, "max_streaming_bitrate")?)?,
        max_remote_bitrate: optional_i64_to_u64(row_get(row, "max_remote_bitrate")?)?,
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

fn parse_user_invitation_status(value: String) -> Result<UserInvitationStatus> {
    UserInvitationStatus::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown user invitation status stored in SQLite database: {value}"),
    })
}

fn invitation_roles_json(roles: &[UserRole]) -> Result<String> {
    let roles = roles.iter().map(|role| role.as_str()).collect::<Vec<_>>();
    serde_json::to_string(&roles).map_err(database_error)
}

fn invitation_roles_from_json(value: String) -> Result<Vec<UserRole>> {
    let values = serde_json::from_str::<Vec<String>>(&value).map_err(database_error)?;
    values
        .into_iter()
        .map(|value| {
            UserRole::parse(&value).ok_or_else(|| NakoError::Database {
                message: format!("unknown user role stored in SQLite invitation roles: {value}"),
            })
        })
        .collect()
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

fn playback_policy_sort_key(policy: &PlaybackPolicy) -> (LibraryId, &'static str, String) {
    match policy.scope {
        PlaybackPolicyScope::User(user_id) => (policy.library_id, "user", user_id.to_string()),
        PlaybackPolicyScope::Role(role) => (policy.library_id, "role", role.as_str().to_owned()),
    }
}

fn page_vec<T>(values: Vec<T>, page: PageRequest) -> Vec<T> {
    let page = page.clamped();
    let start = usize::try_from(page.offset).unwrap_or(usize::MAX);
    let limit = usize::try_from(page.limit).unwrap_or(usize::MAX);

    values.into_iter().skip(start).take(limit).collect()
}
