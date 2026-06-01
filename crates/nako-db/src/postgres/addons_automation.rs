use std::collections::HashSet;

use sqlx::{Postgres, postgres::PgRow};

use super::core_catalog::{upsert_media_item_tx, upsert_search_projection_tx};
use super::jobs::insert_job_tx;
use super::metadata_catalog::{
    replace_item_catalog_graph_tx, upsert_provider_mapping_tx, upsert_provider_subject_tx,
};
use super::{
    PostgresStore, database_error, i64_to_u32, i64_to_u64, parse_id, parse_optional_id, row_get,
    u32_to_i64, u64_to_i64,
};
use nako_core::*;

const AUTOMATION_PROVIDER_SELECT: &str = r#"
            SELECT
                id::text AS id,
                name,
                base_url,
                secret_env,
                capabilities_json::text AS capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM automation_providers
            "#;

const AUTOMATION_ARTIFACT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                job_id::text AS job_id,
                provider_id::text AS provider_id,
                capability,
                kind,
                library_id::text AS library_id,
                item_id::text AS item_id,
                source_id::text AS source_id,
                artifact_json,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS accepted_at
            FROM automation_artifacts
            "#;

const ADDON_REGISTRATION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                outbound_task_dispatch_secret_env,
                granted_scopes_json::text AS granted_scopes_json,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_registrations
            "#;

const ADDON_TOKEN_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                label,
                token_prefix,
                token_hash,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(rotated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS rotated_at,
                to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS revoked_at,
                to_char(last_used_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS last_used_at
            FROM addon_tokens
            "#;

const ADDON_ROUTING_PLAN_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                manifest_id,
                manifest_version,
                manifest_fingerprint,
                declaration_kind,
                declaration_id,
                status,
                target,
                safe_reason_code,
                job_kind,
                event_kind,
                plan_json,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_routing_plans
            "#;

const ADDON_SIDE_EFFECT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                token_id::text AS token_id,
                permission,
                library_id::text AS library_id,
                target_kind,
                target_id,
                idempotency_key,
                request_fingerprint,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code,
                apply_status,
                apply_error_code,
                applied_item_id::text AS applied_item_id,
                applied_source,
                apply_report_json,
                to_char(applied_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS applied_at,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
            FROM addon_side_effects
            "#;

#[async_trait::async_trait]
impl AddonRepository for PostgresStore {
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
                outbound_task_dispatch_secret_env,
                granted_scopes_json,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10)
            ON CONFLICT(id) DO UPDATE SET
                manifest_id = excluded.manifest_id,
                name = excluded.name,
                version = excluded.version,
                protocol_version = excluded.protocol_version,
                base_url = excluded.base_url,
                manifest_json = excluded.manifest_json,
                outbound_task_dispatch_secret_env = excluded.outbound_task_dispatch_secret_env,
                granted_scopes_json = excluded.granted_scopes_json,
                status = excluded.status,
                updated_at = statement_timestamp()
            WHERE addon_registrations.status <> 'unregistered'
            "#,
        )
        .bind(addon.id.as_uuid())
        .bind(&addon.manifest_id)
        .bind(&addon.name)
        .bind(&addon.version)
        .bind(&addon.protocol_version)
        .bind(&addon.base_url)
        .bind(&addon.manifest_json)
        .bind(&addon.outbound_task_dispatch_secret_env)
        .bind(granted_scopes_json)
        .bind(addon.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_registration(addon.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!("addon registration {} was not found after upsert", addon.id),
            })
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(&format!("{ADDON_REGISTRATION_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(&format!(
            "{ADDON_REGISTRATION_SELECT} WHERE manifest_id = $1"
        ))
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
        let status = status.map(|status| status.as_str().to_owned());
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_REGISTRATION_SELECT}
            WHERE ($1::text IS NULL OR status = $1)
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(status.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_registration).collect()
    }

    async fn update_addon_registration_status(
        &self,
        id: AddonId,
        status: AddonStatus,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE addon_registrations
            SET
                status = $2,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        self.get_addon_registration(id).await
    }

    async fn unregister_addon_registration(
        &self,
        id: AddonId,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let result = sqlx::query(
            r#"
            UPDATE addon_registrations
            SET
                status = $2,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(AddonStatus::Unregistered.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        }

        sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = $2,
                revoked_at = COALESCE(revoked_at, statement_timestamp())
            WHERE addon_id = $1 AND status = $3
            "#,
        )
        .bind(id.as_uuid())
        .bind(AddonTokenStatus::Revoked.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM addon_grants WHERE addon_id = $1")
            .bind(id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;
        self.get_addon_registration(id).await
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
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(token.id.as_uuid())
        .bind(token.addon_id.as_uuid())
        .bind(&token.label)
        .bind(&token.token_prefix)
        .bind(&token.token_hash)
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_token(token.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!("addon token {} was not found after create", token.id),
            })
    }

    async fn get_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let row = sqlx::query(&format!("{ADDON_TOKEN_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn find_addon_token_by_hash(&self, token_hash: &str) -> Result<Option<AddonTokenRecord>> {
        let row = sqlx::query(&format!("{ADDON_TOKEN_SELECT} WHERE token_hash = $1"))
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<Vec<AddonTokenRecord>> {
        let rows = sqlx::query(&format!(
            "{ADDON_TOKEN_SELECT} WHERE addon_id = $1 ORDER BY created_at ASC, id ASC"
        ))
        .bind(addon_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_token).collect()
    }

    async fn mark_addon_token_used(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        sqlx::query(
            r#"
            UPDATE addon_tokens
            SET last_used_at = statement_timestamp()
            WHERE id = $1 AND status = $2
            "#,
        )
        .bind(id.as_uuid())
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
                status = $2,
                rotated_at = statement_timestamp()
            WHERE id = $1 AND status = $3 AND addon_id = $4
            "#,
        )
        .bind(rotated_token_id.as_uuid())
        .bind(AddonTokenStatus::Rotated.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .bind(new_token.addon_id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if rotate_result.rows_affected() == 0 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(NakoError::Conflict {
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
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(new_token.id.as_uuid())
        .bind(new_token.addon_id.as_uuid())
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
            .ok_or_else(|| NakoError::Database {
                message: format!("addon token {rotated_token_id} was not found after rotate"),
            })?;
        let created =
            self.get_addon_token(new_token.id)
                .await?
                .ok_or_else(|| NakoError::Database {
                    message: format!("addon token {} was not found after rotate", new_token.id),
                })?;

        Ok((rotated, created))
    }

    async fn revoke_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = $2,
                revoked_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(id.as_uuid())
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

        sqlx::query("DELETE FROM addon_grants WHERE addon_id = $1")
            .bind(addon_id.as_uuid())
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
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(grant.id.as_uuid())
            .bind(grant.addon_id.as_uuid())
            .bind(grant.permission.as_str())
            .bind(grant.library_id.map(|id| id.as_uuid()))
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
                id::text AS id,
                addon_id::text AS addon_id,
                permission,
                library_id::text AS library_id,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
            FROM addon_grants
            WHERE addon_id = $1
            ORDER BY permission ASC, library_id ASC, created_at ASC, id ASC
            "#,
        )
        .bind(addon_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_grant).collect()
    }

    async fn replace_addon_routing_plans(
        &self,
        addon_id: AddonId,
        plans: Vec<NewAddonRoutingPlan>,
    ) -> Result<Vec<AddonRoutingPlanRecord>> {
        for plan in &plans {
            if plan.addon_id != addon_id {
                return Err(NakoError::InvalidInput {
                    message: "addon routing plan addon_id does not match replacement target"
                        .to_owned(),
                });
            }
        }

        let desired_keys = plans
            .iter()
            .map(|plan| {
                (
                    plan.declaration_kind.as_str().to_owned(),
                    plan.declaration_id.clone(),
                )
            })
            .collect::<HashSet<_>>();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for plan in plans {
            sqlx::query(
                r#"
                INSERT INTO addon_routing_plans (
                    id,
                    addon_id,
                    manifest_id,
                    manifest_version,
                    manifest_fingerprint,
                    declaration_kind,
                    declaration_id,
                    status,
                    target,
                    safe_reason_code,
                    job_kind,
                    event_kind,
                    plan_json
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT(addon_id, declaration_kind, declaration_id) DO UPDATE SET
                    manifest_id = excluded.manifest_id,
                    manifest_version = excluded.manifest_version,
                    manifest_fingerprint = excluded.manifest_fingerprint,
                    status = excluded.status,
                    target = excluded.target,
                    safe_reason_code = excluded.safe_reason_code,
                    job_kind = excluded.job_kind,
                    event_kind = excluded.event_kind,
                    plan_json = excluded.plan_json,
                    updated_at = statement_timestamp()
                "#,
            )
            .bind(plan.id.as_uuid())
            .bind(plan.addon_id.as_uuid())
            .bind(&plan.manifest_id)
            .bind(&plan.manifest_version)
            .bind(plan.manifest_fingerprint.as_str())
            .bind(plan.declaration_kind.as_str())
            .bind(&plan.declaration_id)
            .bind(plan.status.as_str())
            .bind(plan.target.as_str())
            .bind(&plan.safe_reason_code)
            .bind(plan.job_kind.map(|kind| kind.as_str().to_owned()))
            .bind(&plan.event_kind)
            .bind(&plan.plan_json)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        let existing_rows = sqlx::query(
            r#"
            SELECT declaration_kind, declaration_id
            FROM addon_routing_plans
            WHERE addon_id = $1
            "#,
        )
        .bind(addon_id.as_uuid())
        .fetch_all(&mut *transaction)
        .await
        .map_err(database_error)?;
        for row in existing_rows {
            let declaration_kind = row_get::<String>(&row, "declaration_kind")?;
            let declaration_id = row_get::<String>(&row, "declaration_id")?;
            if desired_keys.contains(&(declaration_kind.clone(), declaration_id.clone())) {
                continue;
            }
            sqlx::query(
                r#"
                DELETE FROM addon_routing_plans
                WHERE addon_id = $1 AND declaration_kind = $2 AND declaration_id = $3
                "#,
            )
            .bind(addon_id.as_uuid())
            .bind(declaration_kind)
            .bind(declaration_id)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;
        self.list_addon_routing_plans(addon_id).await
    }

    async fn list_addon_routing_plans(
        &self,
        addon_id: AddonId,
    ) -> Result<Vec<AddonRoutingPlanRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_ROUTING_PLAN_SELECT}
            WHERE addon_id = $1
            ORDER BY declaration_kind ASC, declaration_id ASC, created_at ASC, id ASC
            "#
        ))
        .bind(addon_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_routing_plan).collect()
    }

    async fn create_addon_side_effect(
        &self,
        side_effect: NewAddonSideEffect,
    ) -> Result<AddonSideEffectRecord> {
        let request_fingerprint = AddonSideEffectRequestFingerprint::new(
            side_effect.permission,
            side_effect.library_id,
            &side_effect.target,
            &side_effect.provenance_json,
            &side_effect.payload_json,
        );

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
                request_fingerprint,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT(addon_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(side_effect.id.as_uuid())
        .bind(side_effect.addon_id.as_uuid())
        .bind(side_effect.token_id.as_uuid())
        .bind(side_effect.permission.as_str())
        .bind(side_effect.library_id.as_uuid())
        .bind(side_effect.target.kind.as_str())
        .bind(&side_effect.target.id)
        .bind(&side_effect.idempotency_key)
        .bind(request_fingerprint.as_str())
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
        .ok_or_else(|| NakoError::Database {
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
        let row = sqlx::query(&format!(
            r#"
            {ADDON_SIDE_EFFECT_SELECT}
            WHERE addon_id = $1 AND idempotency_key = $2
            ORDER BY created_at ASC, id ASC
            LIMIT 1
            "#
        ))
        .bind(addon_id.as_uuid())
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
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let record = set_addon_side_effect_apply_outcome_tx(&mut transaction, id, &outcome).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(record)
    }
}

pub(super) async fn set_addon_side_effect_apply_outcome_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: AddonSideEffectId,
    outcome: &AddonSideEffectApplyOutcome,
) -> Result<AddonSideEffectRecord> {
    sqlx::query(
        r#"
        UPDATE addon_side_effects
        SET
            apply_status = $2,
            apply_error_code = $3,
            applied_item_id = $4,
            applied_source = $5,
            apply_report_json = $6,
            applied_at = CASE
                WHEN $2 = 'applied' THEN statement_timestamp()
                ELSE applied_at
            END
        WHERE id = $1
        "#,
    )
    .bind(id.as_uuid())
    .bind(outcome.status.as_str())
    .bind(&outcome.error_code)
    .bind(outcome.item_id.map(|id| id.as_uuid()))
    .bind(&outcome.source)
    .bind(&outcome.report_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    let row = sqlx::query(&format!("{ADDON_SIDE_EFFECT_SELECT} WHERE id = $1 LIMIT 1"))
        .bind(id.as_uuid())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_addon_side_effect)
        .transpose()?
        .ok_or_else(|| NakoError::NotFound {
            entity: "addon_side_effect",
            id: id.to_string(),
        })
}

#[async_trait::async_trait]
impl AutomationRepository for PostgresStore {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord> {
        let capabilities_json =
            serde_json::to_string(&provider.capabilities).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO automation_providers (
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                secret_env = excluded.secret_env,
                capabilities_json = excluded.capabilities_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(provider.id.as_uuid())
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.secret_env)
        .bind(capabilities_json)
        .bind(u64_to_i64(provider.timeout_ms)?)
        .bind(u32_to_i64(provider.max_attempts))
        .bind(provider.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_provider(provider.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "automation provider {} was not found after upsert",
                    provider.id
                ),
            })
    }

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>> {
        let row = sqlx::query(&format!("{AUTOMATION_PROVIDER_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_automation_provider).transpose()
    }

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_PROVIDER_SELECT}
            WHERE status = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(AutomationProviderStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_provider).collect()
    }

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord> {
        sqlx::query(
            r#"
            INSERT INTO automation_artifacts (
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(artifact.id.as_uuid())
        .bind(artifact.job_id.as_uuid())
        .bind(artifact.provider_id.as_uuid())
        .bind(artifact.capability.as_str())
        .bind(artifact.kind.as_str())
        .bind(artifact.library_id.map(|id| id.as_uuid()))
        .bind(artifact.item_id.map(|id| id.as_uuid()))
        .bind(artifact.source_id.map(|id| id.as_uuid()))
        .bind(artifact.artifact_json)
        .bind(AutomationArtifactStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(artifact.id).await
    }

    async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>> {
        self.get_automation_artifact(id).await
    }

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord> {
        let query = if status == AutomationArtifactStatus::Accepted {
            r#"
            UPDATE automation_artifacts
            SET
                status = $2,
                accepted_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#
        } else {
            r#"
            UPDATE automation_artifacts
            SET
                status = $2,
                accepted_at = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#
        };

        sqlx::query(query)
            .bind(id.as_uuid())
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(id).await
    }

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_ARTIFACT_SELECT}
            WHERE job_id = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(job_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_ARTIFACT_SELECT}
            WHERE item_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_generated_artifact_proposals(
        &self,
        page: PageRequest,
    ) -> Result<Vec<GeneratedArtifactProposal>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                automation_artifacts.id::text AS id,
                automation_artifacts.job_id::text AS job_id,
                automation_artifacts.provider_id::text AS provider_id,
                automation_artifacts.capability,
                automation_artifacts.kind,
                automation_artifacts.library_id::text AS library_id,
                automation_artifacts.item_id::text AS item_id,
                automation_artifacts.source_id::text AS source_id,
                automation_artifacts.artifact_json,
                automation_artifacts.status,
                to_char(automation_artifacts.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(automation_artifacts.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(automation_artifacts.accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS accepted_at,
                automation_providers.id::text AS provider_exists_id,
                automation_providers.name AS provider_name,
                jobs.id::text AS job_exists_id,
                jobs.input_json AS job_input_json,
                jobs.summary_json AS job_summary_json,
                libraries.id::text AS library_exists_id,
                media_items.id::text AS item_exists_id,
                media_sources.id::text AS source_exists_id,
                media_sources.library_id::text AS source_library_id,
                media_sources.item_id::text AS source_item_id
            FROM automation_artifacts
            LEFT JOIN automation_providers
                ON automation_providers.id = automation_artifacts.provider_id
            LEFT JOIN jobs
                ON jobs.id = automation_artifacts.job_id
            LEFT JOIN libraries
                ON libraries.id = automation_artifacts.library_id
            LEFT JOIN media_items
                ON media_items.id = automation_artifacts.item_id
            LEFT JOIN media_sources
                ON media_sources.id = automation_artifacts.source_id
            ORDER BY automation_artifacts.created_at DESC, automation_artifacts.id DESC
            LIMIT $1 OFFSET $2
            "#
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_generated_artifact_proposal)
            .collect()
    }

    async fn find_generated_artifact_metadata_apply_outcome(
        &self,
        artifact_id: AutomationArtifactId,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataApplyOutcomeRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                artifact_id::text AS artifact_id,
                idempotency_key,
                status,
                applied,
                changed,
                applied_source,
                item_id::text AS item_id,
                plan_json::text AS plan_json,
                error_code,
                error_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM generated_artifact_metadata_apply_outcomes
            WHERE artifact_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(artifact_id.as_uuid())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_generated_artifact_metadata_apply_outcome)
            .transpose()
    }

    async fn commit_generated_artifact_metadata_apply_outcome(
        &self,
        commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord> {
        let plan_json = serde_json::to_string(&commit.plan).map_err(database_error)?;
        validate_generated_artifact_provider_mapping_apply_commits(commit)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(application) = &commit.metadata_application {
            if application.catalog_projection.search.item_id != application.item.id {
                return Err(NakoError::InvalidInput {
                    message: format!(
                        "generated artifact metadata apply search projection item_id {} does not match item {}",
                        application.catalog_projection.search.item_id, application.item.id
                    ),
                });
            }
            upsert_media_item_tx(&mut transaction, &application.item).await?;
            replace_item_catalog_graph_tx(
                &mut transaction,
                application.item.id,
                &application.catalog_projection.graph,
            )
            .await?;
            upsert_search_projection_tx(&mut transaction, &application.catalog_projection.search)
                .await?;
        }
        for provider_mapping in &commit.provider_mappings {
            upsert_provider_subject_tx(&mut transaction, &provider_mapping.subject).await?;
            upsert_provider_mapping_tx(&mut transaction, &provider_mapping.mapping).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO generated_artifact_metadata_apply_outcomes (
                id,
                artifact_id,
                idempotency_key,
                status,
                applied,
                changed,
                applied_source,
                item_id,
                plan_json,
                error_code,
                error_message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11)
            "#,
        )
        .bind(commit.id.as_uuid())
        .bind(commit.artifact_id.as_uuid())
        .bind(&commit.idempotency_key)
        .bind(commit.status.as_str())
        .bind(commit.applied)
        .bind(commit.changed)
        .bind(&commit.applied_source)
        .bind(commit.item_id.map(|id| id.as_uuid()))
        .bind(plan_json)
        .bind(&commit.error_code)
        .bind(&commit.error_message)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        self.find_generated_artifact_metadata_apply_outcome(
            commit.artifact_id,
            &commit.idempotency_key,
        )
        .await?
        .ok_or_else(|| NakoError::Database {
            message: format!(
                "generated artifact metadata apply outcome {} was not found after commit",
                commit.id
            ),
        })
    }

    async fn get_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        self.load_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await
    }

    async fn find_generated_artifact_metadata_bulk_apply_batch(
        &self,
        idempotency_key: &str,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT id::text AS id
            FROM generated_artifact_metadata_bulk_apply_batches
            WHERE idempotency_key = $1
            "#,
        )
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        self.load_generated_artifact_metadata_bulk_apply_batch(parse_id(row_get::<String>(
            &row, "id",
        )?)?)
        .await
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        validate_generated_artifact_metadata_bulk_apply_batch_commit(commit)?;
        if let Some(existing) = self
            .find_generated_artifact_metadata_bulk_apply_batch(&commit.idempotency_key)
            .await?
        {
            return Ok(existing);
        }

        let selection_json = serde_json::to_string(&commit.selection).map_err(database_error)?;
        let summary_json = serde_json::to_string(&commit.summary).map_err(database_error)?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        insert_job_tx(&mut transaction, commit.job.clone()).await?;

        sqlx::query(
            r#"
            INSERT INTO generated_artifact_metadata_bulk_apply_batches (
                id,
                job_id,
                idempotency_key,
                status,
                selection_json,
                summary_json
            )
            VALUES ($1, $2, $3, $4, $5::jsonb, $6::jsonb)
            "#,
        )
        .bind(commit.id.as_uuid())
        .bind(commit.job.id.as_uuid())
        .bind(&commit.idempotency_key)
        .bind(commit.status.as_str())
        .bind(selection_json)
        .bind(summary_json)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        for item in &commit.items {
            let plan_item_json = serde_json::to_string(&item.plan_item).map_err(database_error)?;
            sqlx::query(
                r#"
                INSERT INTO generated_artifact_metadata_bulk_apply_batch_items (
                    batch_id,
                    position,
                    artifact_id,
                    status,
                    idempotency_key,
                    outcome_id,
                    error_code,
                    error_message,
                    plan_item_json
                )
                VALUES ($1, $2, $3, $4, $5, NULL, NULL, NULL, $6::jsonb)
                "#,
            )
            .bind(commit.id.as_uuid())
            .bind(u32_to_i64(item.position))
            .bind(item.artifact_id.as_uuid())
            .bind(item.status.as_str())
            .bind(&item.idempotency_key)
            .bind(plan_item_json)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        self.load_generated_artifact_metadata_bulk_apply_batch(commit.id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {} was not found after commit",
                    commit.id
                ),
            })
    }

    async fn commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(
        &self,
        commit: &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        validate_generated_artifact_metadata_bulk_apply_batch_item_outcome_commit(commit)?;
        let result = sqlx::query(
            r#"
            UPDATE generated_artifact_metadata_bulk_apply_batch_items
            SET status = $1,
                outcome_id = $2,
                error_code = $3,
                error_message = $4,
                updated_at = statement_timestamp()
            WHERE batch_id = $5 AND artifact_id = $6
            "#,
        )
        .bind(commit.status.as_str())
        .bind(commit.outcome_id.map(|id| id.as_uuid()))
        .bind(&commit.error_code)
        .bind(&commit.error_message)
        .bind(commit.batch_id.as_uuid())
        .bind(commit.artifact_id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(NakoError::NotFound {
                entity: "generated_artifact_metadata_bulk_apply_batch_item",
                id: format!("{}:{}", commit.batch_id, commit.artifact_id),
            });
        }

        self.load_generated_artifact_metadata_bulk_apply_batch(commit.batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {} was not found after item outcome commit",
                    commit.batch_id
                ),
            })
    }

    async fn update_generated_artifact_metadata_bulk_apply_batch_status(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
        expected: GeneratedArtifactMetadataBulkApplyBatchStatus,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus,
    ) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
        let result = sqlx::query(
            r#"
            UPDATE generated_artifact_metadata_bulk_apply_batches
            SET status = $1,
                updated_at = statement_timestamp()
            WHERE id = $2 AND status = $3
            "#,
        )
        .bind(status.as_str())
        .bind(batch_id.as_uuid())
        .bind(expected.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            if self
                .load_generated_artifact_metadata_bulk_apply_batch(batch_id)
                .await?
                .is_none()
            {
                return Err(NakoError::NotFound {
                    entity: "generated_artifact_metadata_bulk_apply_batch",
                    id: batch_id.to_string(),
                });
            }
            return Err(NakoError::InvalidInput {
                message: format!(
                    "cannot transition generated artifact metadata bulk apply batch {batch_id} from {:?} to {:?}",
                    expected, status
                ),
            });
        }

        self.load_generated_artifact_metadata_bulk_apply_batch(batch_id)
            .await?
            .ok_or_else(|| NakoError::Database {
                message: format!(
                    "generated artifact metadata bulk apply batch {batch_id} was not found after status update"
                ),
            })
    }
}

impl PostgresStore {
    async fn load_generated_artifact_metadata_bulk_apply_batch(
        &self,
        batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    ) -> Result<Option<GeneratedArtifactMetadataBulkApplyBatchRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                job_id::text AS job_id,
                idempotency_key,
                status,
                selection_json::text AS selection_json,
                summary_json::text AS summary_json,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM generated_artifact_metadata_bulk_apply_batches
            WHERE id = $1
            "#,
        )
        .bind(batch_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let item_rows = sqlx::query(
            r#"
            SELECT
                batch_id::text AS batch_id,
                position,
                artifact_id::text AS artifact_id,
                status,
                idempotency_key,
                outcome_id::text AS outcome_id,
                error_code,
                error_message,
                plan_item_json::text AS plan_item_json,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM generated_artifact_metadata_bulk_apply_batch_items
            WHERE batch_id = $1
            ORDER BY position ASC
            "#,
        )
        .bind(batch_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;
        let items = item_rows
            .into_iter()
            .map(row_to_generated_artifact_metadata_bulk_apply_batch_item)
            .collect::<Result<Vec<_>>>()?;

        row_to_generated_artifact_metadata_bulk_apply_batch(row, items).map(Some)
    }

    async fn get_automation_artifact(
        &self,
        id: AutomationArtifactId,
    ) -> Result<Option<AutomationArtifactRecord>> {
        let row = sqlx::query(&format!("{AUTOMATION_ARTIFACT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_automation_artifact).transpose()
    }

    async fn get_automation_artifact_or_not_found(
        &self,
        id: AutomationArtifactId,
    ) -> Result<AutomationArtifactRecord> {
        self.get_automation_artifact(id)
            .await?
            .ok_or_else(|| NakoError::NotFound {
                entity: "automation_artifact",
                id: id.to_string(),
            })
    }
}

fn row_to_automation_provider(row: PgRow) -> Result<AutomationProviderConfigRecord> {
    let capability_names =
        serde_json::from_str::<Vec<String>>(&row_get::<String>(&row, "capabilities_json")?)
            .map_err(database_error)?;
    let capabilities = capability_names
        .into_iter()
        .map(|name| AutomationCapability::parse(&name))
        .collect::<Result<Vec<_>>>()?;

    Ok(AutomationProviderConfigRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        base_url: row_get(&row, "base_url")?,
        secret_env: row_get(&row, "secret_env")?,
        capabilities,
        timeout_ms: i64_to_u64(row_get(&row, "timeout_ms")?)?,
        max_attempts: i64_to_u32(row_get(&row, "max_attempts")?)?,
        status: AutomationProviderStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_automation_artifact(row: PgRow) -> Result<AutomationArtifactRecord> {
    Ok(AutomationArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        provider_id: parse_id(row_get::<String>(&row, "provider_id")?)?,
        capability: AutomationCapability::parse(&row_get::<String>(&row, "capability")?)?,
        kind: AutomationArtifactKind::parse(&row_get::<String>(&row, "kind")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        artifact_json: row_get(&row, "artifact_json")?,
        status: AutomationArtifactStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        accepted_at: row_get(&row, "accepted_at")?,
    })
}

fn row_to_generated_artifact_proposal(row: PgRow) -> Result<GeneratedArtifactProposal> {
    let artifact = AutomationArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        provider_id: parse_id(row_get::<String>(&row, "provider_id")?)?,
        capability: AutomationCapability::parse(&row_get::<String>(&row, "capability")?)?,
        kind: AutomationArtifactKind::parse(&row_get::<String>(&row, "kind")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        artifact_json: row_get(&row, "artifact_json")?,
        status: AutomationArtifactStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        accepted_at: row_get(&row, "accepted_at")?,
    };
    Ok(crate::automation_proposals::generated_artifact_proposal(
        crate::automation_proposals::GeneratedArtifactProposalFacts {
            artifact,
            provider_exists: row_get::<Option<String>>(&row, "provider_exists_id")?.is_some(),
            provider_name: row_get::<Option<String>>(&row, "provider_name")?,
            job_exists: row_get::<Option<String>>(&row, "job_exists_id")?.is_some(),
            job_input_json: row_get::<Option<String>>(&row, "job_input_json")?,
            job_summary_json: row_get::<Option<String>>(&row, "job_summary_json")?,
            library_exists: row_get::<Option<String>>(&row, "library_exists_id")?.is_some(),
            item_exists: row_get::<Option<String>>(&row, "item_exists_id")?.is_some(),
            source_exists: row_get::<Option<String>>(&row, "source_exists_id")?.is_some(),
            source_library_id: parse_optional_id(row_get::<Option<String>>(
                &row,
                "source_library_id",
            )?)?,
            source_item_id: parse_optional_id(row_get::<Option<String>>(&row, "source_item_id")?)?,
        },
    ))
}

fn row_to_generated_artifact_metadata_apply_outcome(
    row: PgRow,
) -> Result<GeneratedArtifactMetadataApplyOutcomeRecord> {
    let plan_json: String = row_get(&row, "plan_json")?;
    let plan = serde_json::from_str(&plan_json).map_err(database_error)?;

    Ok(GeneratedArtifactMetadataApplyOutcomeRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        status: GeneratedArtifactMetadataApplyOutcomeStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        applied: row_get(&row, "applied")?,
        changed: row_get(&row, "changed")?,
        applied_source: row_get(&row, "applied_source")?,
        item_id: parse_optional_id(row_get::<Option<String>>(&row, "item_id")?)?,
        plan,
        error_code: row_get(&row, "error_code")?,
        error_message: row_get(&row, "error_message")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn validate_generated_artifact_provider_mapping_apply_commits(
    commit: &GeneratedArtifactMetadataApplyOutcomeCommit,
) -> Result<()> {
    for provider_mapping in &commit.provider_mappings {
        if provider_mapping.mapping.subject_id != provider_mapping.subject.id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact provider mapping subject_id {} does not match subject {}",
                    provider_mapping.mapping.subject_id, provider_mapping.subject.id
                ),
            });
        }
        if provider_mapping.mapping.status != ProviderMappingStatus::Accepted {
            return Err(NakoError::InvalidInput {
                message:
                    "generated artifact provider mapping apply commits must use accepted status"
                        .to_owned(),
            });
        }
        if Some(provider_mapping.mapping.item_id) != commit.item_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact provider mapping item_id {} does not match outcome item {:?}",
                    provider_mapping.mapping.item_id, commit.item_id
                ),
            });
        }
    }

    Ok(())
}

fn validate_generated_artifact_metadata_bulk_apply_batch_commit(
    commit: &GeneratedArtifactMetadataBulkApplyBatchCommit,
) -> Result<()> {
    if commit.job.kind != JobKind::GeneratedArtifactMetadataBulkApply {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch job must use JobKind::GeneratedArtifactMetadataBulkApply"
                .to_owned(),
        });
    }
    if commit.job.resource_class != GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch job resource_class is invalid"
                .to_owned(),
        });
    }
    if commit.idempotency_key.trim().is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch idempotency_key cannot be empty"
                .to_owned(),
        });
    }
    if commit.items.is_empty() {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply batch requires at least one item"
                .to_owned(),
        });
    }
    for item in &commit.items {
        if item.idempotency_key.trim().is_empty() {
            return Err(NakoError::InvalidInput {
                message:
                    "generated artifact metadata bulk apply item idempotency_key cannot be empty"
                        .to_owned(),
            });
        }
        if item.artifact_id != item.plan_item.artifact_id {
            return Err(NakoError::InvalidInput {
                message: format!(
                    "generated artifact metadata bulk apply item artifact_id {} does not match plan item {}",
                    item.artifact_id, item.plan_item.artifact_id
                ),
            });
        }
    }

    Ok(())
}

fn validate_generated_artifact_metadata_bulk_apply_batch_item_outcome_commit(
    commit: &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
) -> Result<()> {
    if matches!(
        commit.status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
            | GeneratedArtifactMetadataBulkApplyBatchItemStatus::Skipped
    ) {
        return Err(NakoError::InvalidInput {
            message: "generated artifact metadata bulk apply item outcome must be terminal"
                .to_owned(),
        });
    }
    if commit.error_code.as_deref().is_some_and(str::is_empty)
        || commit.error_message.as_deref().is_some_and(str::is_empty)
    {
        return Err(NakoError::InvalidInput {
            message:
                "generated artifact metadata bulk apply item outcome error fields cannot be empty"
                    .to_owned(),
        });
    }

    Ok(())
}

fn row_to_generated_artifact_metadata_bulk_apply_batch(
    row: PgRow,
    items: Vec<GeneratedArtifactMetadataBulkApplyBatchItemRecord>,
) -> Result<GeneratedArtifactMetadataBulkApplyBatchRecord> {
    let selection_json: String = row_get(&row, "selection_json")?;
    let summary_json: String = row_get(&row, "summary_json")?;

    Ok(GeneratedArtifactMetadataBulkApplyBatchRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        status: GeneratedArtifactMetadataBulkApplyBatchStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        selection: serde_json::from_str(&selection_json).map_err(database_error)?,
        summary: serde_json::from_str(&summary_json).map_err(database_error)?,
        execution_summary: GeneratedArtifactMetadataBulkApplyBatchExecutionSummary::from_items(
            &items,
        ),
        items,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_generated_artifact_metadata_bulk_apply_batch_item(
    row: PgRow,
) -> Result<GeneratedArtifactMetadataBulkApplyBatchItemRecord> {
    let plan_item_json: String = row_get(&row, "plan_item_json")?;

    Ok(GeneratedArtifactMetadataBulkApplyBatchItemRecord {
        batch_id: parse_id(row_get::<String>(&row, "batch_id")?)?,
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        position: i64_to_u32(row_get::<i64>(&row, "position")?)?,
        status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::parse(&row_get::<String>(
            &row, "status",
        )?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        outcome_id: parse_optional_id(row_get::<Option<String>>(&row, "outcome_id")?)?,
        error_code: row_get(&row, "error_code")?,
        error_message: row_get(&row, "error_message")?,
        plan_item: serde_json::from_str(&plan_item_json).map_err(database_error)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_addon_registration(row: PgRow) -> Result<AddonRegistrationRecord> {
    let granted_scopes = serde_json::from_str(&row_get::<String>(&row, "granted_scopes_json")?)
        .map_err(database_error)?;

    Ok(AddonRegistrationRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        manifest_id: row_get(&row, "manifest_id")?,
        name: row_get(&row, "name")?,
        version: row_get(&row, "version")?,
        protocol_version: row_get(&row, "protocol_version")?,
        base_url: row_get(&row, "base_url")?,
        manifest_json: row_get(&row, "manifest_json")?,
        outbound_task_dispatch_secret_env: row_get(&row, "outbound_task_dispatch_secret_env")?,
        granted_scopes,
        status: AddonStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_addon_token(row: PgRow) -> Result<AddonTokenRecord> {
    Ok(AddonTokenRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        label: row_get(&row, "label")?,
        token_prefix: row_get(&row, "token_prefix")?,
        token_hash: row_get(&row, "token_hash")?,
        status: AddonTokenStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        rotated_at: row_get(&row, "rotated_at")?,
        revoked_at: row_get(&row, "revoked_at")?,
        last_used_at: row_get(&row, "last_used_at")?,
    })
}

fn row_to_addon_grant(row: PgRow) -> Result<AddonGrantRecord> {
    Ok(AddonGrantRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        permission: AddonPermission::parse(&row_get::<String>(&row, "permission")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        created_at: row_get(&row, "created_at")?,
    })
}

fn row_to_addon_routing_plan(row: PgRow) -> Result<AddonRoutingPlanRecord> {
    let job_kind = row_get::<Option<String>>(&row, "job_kind")?
        .map(|kind| JobKind::parse(&kind))
        .transpose()?;

    Ok(AddonRoutingPlanRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        manifest_id: row_get(&row, "manifest_id")?,
        manifest_version: row_get(&row, "manifest_version")?,
        manifest_fingerprint: AddonManifestFingerprint::parse(row_get::<String>(
            &row,
            "manifest_fingerprint",
        )?)?,
        declaration_kind: AddonRoutingDeclarationKind::parse(&row_get::<String>(
            &row,
            "declaration_kind",
        )?)?,
        declaration_id: row_get(&row, "declaration_id")?,
        status: AddonRoutingPlanStatus::parse(&row_get::<String>(&row, "status")?)?,
        target: AddonRoutingPlanTarget::parse(&row_get::<String>(&row, "target")?)?,
        safe_reason_code: row_get(&row, "safe_reason_code")?,
        job_kind,
        event_kind: row_get(&row, "event_kind")?,
        plan_json: row_get(&row, "plan_json")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_addon_side_effect(row: PgRow) -> Result<AddonSideEffectRecord> {
    let target = AddonSideEffectTarget {
        kind: AddonSideEffectTargetKind::parse(&row_get::<String>(&row, "target_kind")?)?,
        id: row_get(&row, "target_id")?,
    };

    let permission = AddonPermission::parse(&row_get::<String>(&row, "permission")?)?;
    let library_id = parse_id(row_get::<String>(&row, "library_id")?)?;
    let request_fingerprint =
        AddonSideEffectRequestFingerprint::parse(row_get::<String>(&row, "request_fingerprint")?)?;

    Ok(AddonSideEffectRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        token_id: parse_id(row_get::<String>(&row, "token_id")?)?,
        permission,
        library_id,
        target,
        idempotency_key: row_get(&row, "idempotency_key")?,
        request_fingerprint,
        provenance_json: row_get(&row, "provenance_json")?,
        payload_json: row_get(&row, "payload_json")?,
        validation_status: AddonSideEffectValidationStatus::parse(&row_get::<String>(
            &row,
            "validation_status",
        )?)?,
        safe_error_code: row_get(&row, "safe_error_code")?,
        apply_status: AddonSideEffectApplyStatus::parse(&row_get::<String>(&row, "apply_status")?)?,
        apply_error_code: row_get(&row, "apply_error_code")?,
        applied_item_id: parse_optional_id(row_get::<Option<String>>(&row, "applied_item_id")?)?,
        applied_source: row_get(&row, "applied_source")?,
        apply_report_json: row_get(&row, "apply_report_json")?,
        applied_at: row_get(&row, "applied_at")?,
        created_at: row_get(&row, "created_at")?,
    })
}
