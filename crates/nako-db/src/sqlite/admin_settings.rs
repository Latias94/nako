use super::{SqliteStore, codec::*};
use nako_core::*;

#[async_trait::async_trait]
impl AdminSettingsRepository for SqliteStore {
    async fn upsert_admin_metadata_raw_cache_settings(
        &self,
        record: AdminMetadataRawCacheSettingsRecord,
    ) -> Result<AdminMetadataRawCacheSettingsRecord> {
        sqlx::query(
            r#"
            INSERT INTO admin_metadata_raw_cache_settings (
                id, retention_ms, cleanup_on_startup, source, effect, updated_at_ms
            )
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                retention_ms = excluded.retention_ms,
                cleanup_on_startup = excluded.cleanup_on_startup,
                source = excluded.source,
                effect = excluded.effect,
                updated_at_ms = excluded.updated_at_ms
            "#,
        )
        .bind(u64_to_i64(record.settings.retention_ms)?)
        .bind(record.settings.cleanup_on_startup)
        .bind(admin_settings_source_to_str(record.source))
        .bind(admin_settings_effect_to_str(record.effect))
        .bind(record.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(record)
    }

    async fn get_admin_metadata_raw_cache_settings(
        &self,
    ) -> Result<Option<AdminMetadataRawCacheSettingsRecord>> {
        let row = sqlx::query(
            r#"
            SELECT retention_ms, cleanup_on_startup, source, effect, updated_at_ms
            FROM admin_metadata_raw_cache_settings
            WHERE id = 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(|row| {
            Ok(AdminMetadataRawCacheSettingsRecord {
                settings: AdminMetadataRawCacheSettings {
                    retention_ms: i64_to_u64(row_get(&row, "retention_ms")?)?,
                    cleanup_on_startup: row_get(&row, "cleanup_on_startup")?,
                },
                source: admin_settings_source_from_str(row_get(&row, "source")?)?,
                effect: admin_settings_effect_from_str(row_get(&row, "effect")?)?,
                updated_at_ms: row_get(&row, "updated_at_ms")?,
            })
        })
        .transpose()
    }
}

fn admin_settings_source_to_str(source: AdminSettingsSource) -> &'static str {
    match source {
        AdminSettingsSource::Configured => "configured",
        AdminSettingsSource::Admin => "admin",
    }
}

fn admin_settings_source_from_str(value: String) -> Result<AdminSettingsSource> {
    match value.as_str() {
        "configured" => Ok(AdminSettingsSource::Configured),
        "admin" => Ok(AdminSettingsSource::Admin),
        _ => Err(NakoError::Database {
            message: format!("unknown admin settings source stored in database: {value}"),
        }),
    }
}

fn admin_settings_effect_to_str(effect: AdminSettingsEffect) -> &'static str {
    match effect {
        AdminSettingsEffect::Active => "active",
        AdminSettingsEffect::RequiresRestart => "requires_restart",
    }
}

fn admin_settings_effect_from_str(value: String) -> Result<AdminSettingsEffect> {
    match value.as_str() {
        "active" => Ok(AdminSettingsEffect::Active),
        "requires_restart" => Ok(AdminSettingsEffect::RequiresRestart),
        _ => Err(NakoError::Database {
            message: format!("unknown admin settings effect stored in database: {value}"),
        }),
    }
}
