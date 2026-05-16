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
}
