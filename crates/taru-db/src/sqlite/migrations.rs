use std::borrow::Cow;

use sqlx::migrate::{Migration, MigrationType, Migrator};
use taru_core::{DatabaseLifecycle, Result};

use super::{SqliteStore, codec::database_error};

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "initial",
        include_str!("../../migrations/0001_initial.sql"),
    ),
    (
        2,
        "media probe",
        include_str!("../../migrations/0002_media_probe.sql"),
    ),
    (3, "jobs", include_str!("../../migrations/0003_jobs.sql")),
    (
        4,
        "job input payload",
        include_str!("../../migrations/0004_job_input_payload.sql"),
    ),
    (
        5,
        "metadata policy",
        include_str!("../../migrations/0005_metadata_policy.sql"),
    ),
    (
        6,
        "library profiles",
        include_str!("../../migrations/0006_library_profiles.sql"),
    ),
    (
        7,
        "catalog ingestion",
        include_str!("../../migrations/0007_catalog_ingestion.sql"),
    ),
    (
        8,
        "transcode sessions",
        include_str!("../../migrations/0008_transcode_sessions.sql"),
    ),
    (
        9,
        "event outbox",
        include_str!("../../migrations/0009_event_outbox.sql"),
    ),
    (
        10,
        "webhooks",
        include_str!("../../migrations/0010_webhooks.sql"),
    ),
    (
        11,
        "automation",
        include_str!("../../migrations/0011_automation.sql"),
    ),
    (
        12,
        "addons",
        include_str!("../../migrations/0012_addons.sql"),
    ),
    (
        13,
        "vfs cache",
        include_str!("../../migrations/0013_vfs_cache.sql"),
    ),
    (
        14,
        "staging manifest",
        include_str!("../../migrations/0014_staging_manifest.sql"),
    ),
    (
        15,
        "media source library locator",
        include_str!("../../migrations/0015_media_source_library_locator.sql"),
    ),
    (
        16,
        "metadata provider attempts",
        include_str!("../../migrations/0016_metadata_provider_attempts.sql"),
    ),
    (
        17,
        "ingestion failures",
        include_str!("../../migrations/0017_ingestion_failures.sql"),
    ),
    (
        18,
        "metadata catalog domain",
        include_str!("../../migrations/0018_metadata_catalog_domain.sql"),
    ),
    (
        19,
        "library item states",
        include_str!("../../migrations/0019_library_item_states.sql"),
    ),
    (
        20,
        "local inference evidence snapshot key",
        include_str!("../../migrations/0020_local_inference_evidence_snapshot_key.sql"),
    ),
    (
        21,
        "addon tokens and grants",
        include_str!("../../migrations/0021_addon_tokens_and_grants.sql"),
    ),
    (
        22,
        "addon side effects",
        include_str!("../../migrations/0022_addon_side_effects.sql"),
    ),
    (
        23,
        "addon side effect apply outcome",
        include_str!("../../migrations/0023_addon_side_effect_apply_outcome.sql"),
    ),
    (
        24,
        "addon side effect apply report",
        include_str!("../../migrations/0024_addon_side_effect_apply_report.sql"),
    ),
    (
        25,
        "addon artwork candidates",
        include_str!("../../migrations/0025_addon_artwork_candidates.sql"),
    ),
    (
        26,
        "managed artwork ingest",
        include_str!("../../migrations/0026_managed_artwork_ingest.sql"),
    ),
    (
        27,
        "selected artwork publication",
        include_str!("../../migrations/0027_selected_artwork_publication.sql"),
    ),
    (
        28,
        "managed artwork artifact cleanup",
        include_str!("../../migrations/0028_managed_artwork_artifact_cleanup.sql"),
    ),
    (
        29,
        "job ownership leases",
        include_str!("../../migrations/0029_job_ownership_leases.sql"),
    ),
    (
        30,
        "user playback states",
        include_str!("../../migrations/0030_user_playback_states.sql"),
    ),
    (
        31,
        "managed import artifacts",
        include_str!("../../migrations/0031_managed_import_artifacts.sql"),
    ),
    (
        32,
        "managed import promotion applies",
        include_str!("../../migrations/0032_managed_import_promotion_applies.sql"),
    ),
];

#[async_trait::async_trait]
impl DatabaseLifecycle for SqliteStore {
    async fn migrate(&self) -> Result<()> {
        migrator().run(self.pool()).await.map_err(database_error)
    }
}

fn migrator() -> Migrator {
    Migrator {
        migrations: Cow::Owned(
            MIGRATIONS
                .iter()
                .map(|(version, description, sql)| {
                    Migration::new(
                        *version,
                        Cow::Borrowed(*description),
                        MigrationType::Simple,
                        Cow::Borrowed(*sql),
                        false,
                    )
                })
                .collect(),
        ),
        ..Migrator::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn single_migration(sql: &'static str) -> Migrator {
        Migrator {
            migrations: Cow::Owned(vec![Migration::new(
                1,
                Cow::Borrowed("test migration"),
                MigrationType::Simple,
                Cow::Borrowed(sql),
                false,
            )]),
            ..Migrator::DEFAULT
        }
    }

    #[tokio::test]
    async fn migration_sql_allows_semicolons_inside_string_literals() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let migrator = single_migration(
            r#"
            CREATE TABLE semicolon_fixture (
                id INTEGER PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT INTO semicolon_fixture (value) VALUES ('literal;inside');
            "#,
        );

        migrator.run(store.pool()).await.unwrap();

        let value: String = sqlx::query_scalar("SELECT value FROM semicolon_fixture WHERE id = 1")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let applied: i64 =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE version = 1")
                .fetch_one(store.pool())
                .await
                .unwrap();

        assert_eq!(value, "literal;inside");
        assert_eq!(applied, 1);
    }

    #[tokio::test]
    async fn failed_migration_rolls_back_schema_and_version_record() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        let migrator = single_migration(
            r#"
            CREATE TABLE rollback_fixture (
                id INTEGER PRIMARY KEY,
                value TEXT NOT NULL
            );
            INSERT INTO rollback_fixture (value) VALUES ('created before failure');
            INSERT INTO missing_table (value) VALUES ('boom');
            "#,
        );

        let err = migrator.run(store.pool()).await.unwrap_err();
        assert!(!err.to_string().is_empty());

        let table_exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'rollback_fixture'",
        )
        .fetch_optional(store.pool())
        .await
        .unwrap();
        let version_exists: Option<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations WHERE version = 1")
                .fetch_optional(store.pool())
                .await
                .unwrap();

        assert_eq!(table_exists, None);
        assert_eq!(version_exists, None);
    }
}
