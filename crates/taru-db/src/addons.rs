use super::*;

#[async_trait::async_trait]
impl AddonRepository for SqliteStore {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord> {
        let granted_scopes_json =
            serde_json::to_string(&addon.granted_scopes).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO addon_registrations (
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                manifest_id = excluded.manifest_id,
                name = excluded.name,
                version = excluded.version,
                protocol_version = excluded.protocol_version,
                base_url = excluded.base_url,
                manifest_json = excluded.manifest_json,
                granted_scopes_json = excluded.granted_scopes_json,
                status = excluded.status,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(addon.id.to_string())
        .bind(&addon.manifest_id)
        .bind(&addon.name)
        .bind(&addon.version)
        .bind(&addon.protocol_version)
        .bind(&addon.base_url)
        .bind(&addon.manifest_json)
        .bind(granted_scopes_json)
        .bind(addon.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_registration(addon.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon registration {} was not found after upsert", addon.id),
            })
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status,
                created_at,
                updated_at
            FROM addon_registrations
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status,
                created_at,
                updated_at
            FROM addon_registrations
            WHERE manifest_id = ?1
            "#,
        )
        .bind(manifest_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                r#"
                SELECT
                    id,
                    manifest_id,
                    name,
                    version,
                    protocol_version,
                    base_url,
                    manifest_json,
                    granted_scopes_json,
                    status,
                    created_at,
                    updated_at
                FROM addon_registrations
                WHERE status = ?1
                ORDER BY created_at ASC, id ASC
                "#,
            )
            .bind(status.as_str())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?
        } else {
            sqlx::query(
                r#"
                SELECT
                    id,
                    manifest_id,
                    name,
                    version,
                    protocol_version,
                    base_url,
                    manifest_json,
                    granted_scopes_json,
                    status,
                    created_at,
                    updated_at
                FROM addon_registrations
                ORDER BY created_at ASC, id ASC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?
        };

        rows.into_iter().map(row_to_addon_registration).collect()
    }

    async fn create_addon_token(&self, token: NewAddonToken) -> Result<AddonTokenRecord> {
        sqlx::query(
            r#"
            INSERT INTO addon_tokens (
                id,
                addon_id,
                label,
                token_prefix,
                token_hash,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(token.id.to_string())
        .bind(token.addon_id.to_string())
        .bind(&token.label)
        .bind(&token.token_prefix)
        .bind(&token.token_hash)
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_token(token.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon token {} was not found after create", token.id),
            })
    }

    async fn get_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let sql = addon_token_select_sql("WHERE id = ?1");
        let row = sqlx::query(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn find_addon_token_by_hash(&self, token_hash: &str) -> Result<Option<AddonTokenRecord>> {
        let sql = addon_token_select_sql("WHERE token_hash = ?1");
        let row = sqlx::query(&sql)
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<Vec<AddonTokenRecord>> {
        let sql = addon_token_select_sql("WHERE addon_id = ?1 ORDER BY created_at ASC, id ASC");
        let rows = sqlx::query(&sql)
            .bind(addon_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_token).collect()
    }

    async fn mark_addon_token_used(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        sqlx::query(
            r#"
            UPDATE addon_tokens
            SET last_used_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?2
            "#,
        )
        .bind(id.to_string())
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_token(id).await
    }

    async fn rotate_addon_token(
        &self,
        rotated_token_id: AddonTokenId,
        new_token: NewAddonToken,
    ) -> Result<(AddonTokenRecord, AddonTokenRecord)> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let rotate_result = sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = ?2,
                rotated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?3 AND addon_id = ?4
            "#,
        )
        .bind(rotated_token_id.to_string())
        .bind(AddonTokenStatus::Rotated.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .bind(new_token.addon_id.to_string())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if rotate_result.rows_affected() == 0 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: format!("addon token {rotated_token_id} is not active"),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO addon_tokens (
                id,
                addon_id,
                label,
                token_prefix,
                token_hash,
                status
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(new_token.id.to_string())
        .bind(new_token.addon_id.to_string())
        .bind(&new_token.label)
        .bind(&new_token.token_prefix)
        .bind(&new_token.token_hash)
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        let rotated = self
            .get_addon_token(rotated_token_id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon token {rotated_token_id} was not found after rotate"),
            })?;
        let created =
            self.get_addon_token(new_token.id)
                .await?
                .ok_or_else(|| TaruError::Database {
                    message: format!("addon token {} was not found after rotate", new_token.id),
                })?;

        Ok((rotated, created))
    }

    async fn revoke_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = ?2,
                revoked_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1 AND status = ?3
            "#,
        )
        .bind(id.to_string())
        .bind(AddonTokenStatus::Revoked.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return self.get_addon_token(id).await;
        }

        self.get_addon_token(id).await
    }

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query("DELETE FROM addon_grants WHERE addon_id = ?1")
            .bind(addon_id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for grant in grants {
            sqlx::query(
                r#"
                INSERT INTO addon_grants (
                    id,
                    addon_id,
                    permission,
                    library_id
                )
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .bind(grant.id.to_string())
            .bind(grant.addon_id.to_string())
            .bind(grant.permission.as_str())
            .bind(grant.library_id.map(|id| id.to_string()))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;
        self.list_addon_grants(addon_id).await
    }

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                addon_id,
                permission,
                library_id,
                created_at
            FROM addon_grants
            WHERE addon_id = ?1
            ORDER BY permission ASC, library_id ASC, created_at ASC, id ASC
            "#,
        )
        .bind(addon_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_grant).collect()
    }

    async fn create_addon_side_effect(
        &self,
        side_effect: NewAddonSideEffect,
    ) -> Result<AddonSideEffectRecord> {
        sqlx::query(
            r#"
            INSERT INTO addon_side_effects (
                id,
                addon_id,
                token_id,
                permission,
                library_id,
                target_kind,
                target_id,
                idempotency_key,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            ON CONFLICT(addon_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(side_effect.id.to_string())
        .bind(side_effect.addon_id.to_string())
        .bind(side_effect.token_id.to_string())
        .bind(side_effect.permission.as_str())
        .bind(side_effect.library_id.to_string())
        .bind(side_effect.target.kind.as_str())
        .bind(&side_effect.target.id)
        .bind(&side_effect.idempotency_key)
        .bind(&side_effect.provenance_json)
        .bind(&side_effect.payload_json)
        .bind(side_effect.validation_status.as_str())
        .bind(&side_effect.safe_error_code)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_addon_side_effect_by_idempotency_key(
            side_effect.addon_id,
            &side_effect.idempotency_key,
        )
        .await?
        .ok_or_else(|| TaruError::Database {
            message: format!(
                "addon side effect {} was not found after create",
                side_effect.id
            ),
        })
    }

    async fn find_addon_side_effect_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonSideEffectRecord>> {
        let sql = addon_side_effect_select_sql(
            "WHERE addon_id = ?1 AND idempotency_key = ?2 ORDER BY created_at ASC, id ASC LIMIT 1",
        );
        let row = sqlx::query(&sql)
            .bind(addon_id.to_string())
            .bind(idempotency_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_side_effect).transpose()
    }

    async fn set_addon_side_effect_apply_outcome(
        &self,
        id: AddonSideEffectId,
        outcome: AddonSideEffectApplyOutcome,
    ) -> Result<AddonSideEffectRecord> {
        sqlx::query(
            r#"
            UPDATE addon_side_effects
            SET
                apply_status = ?2,
                apply_error_code = ?3,
                applied_item_id = ?4,
                applied_source = ?5,
                apply_report_json = ?6,
                applied_at = CASE
                    WHEN ?2 = 'applied' THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
                    ELSE applied_at
                END
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(outcome.status.as_str())
        .bind(&outcome.error_code)
        .bind(outcome.item_id.map(|id| id.to_string()))
        .bind(&outcome.source)
        .bind(&outcome.report_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        let sql = addon_side_effect_select_sql("WHERE id = ?1 LIMIT 1");
        let row = sqlx::query(&sql)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_side_effect)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
                entity: "addon_side_effect",
                id: id.to_string(),
            })
    }
}

fn addon_token_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            id,
            addon_id,
            label,
            token_prefix,
            token_hash,
            status,
            created_at,
            rotated_at,
            revoked_at,
            last_used_at
        FROM addon_tokens
        {where_clause}
        "#
    )
}

fn addon_side_effect_select_sql(where_clause: &str) -> String {
    format!(
        r#"
        SELECT
            id,
            addon_id,
            token_id,
            permission,
            library_id,
            target_kind,
            target_id,
            idempotency_key,
            provenance_json,
            payload_json,
            validation_status,
            safe_error_code,
            apply_status,
            apply_error_code,
            applied_item_id,
            applied_source,
            apply_report_json,
            applied_at,
            created_at
        FROM addon_side_effects
        {where_clause}
        "#
    )
}
