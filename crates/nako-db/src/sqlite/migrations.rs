use std::borrow::Cow;

use nako_core::{DatabaseLifecycle, Result};
use sqlx::migrate::{Migration, MigrationType, Migrator};

use super::{SqliteStore, codec::database_error};

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (1, "baseline", include_str!("../../migrations/baseline.sql")),
    (
        2,
        "durable_job_priority",
        include_str!("../../migrations/0002_durable_job_priority.sql"),
    ),
    (
        3,
        "staging_attribution",
        include_str!("../../migrations/0003_staging_attribution.sql"),
    ),
    (
        4,
        "vfs_cache_failure_authority",
        include_str!("../../migrations/0004_vfs_cache_failure_authority.sql"),
    ),
    (
        5,
        "source_duplicate_pair_identity",
        include_str!("../../migrations/0005_source_duplicate_pair_identity.sql"),
    ),
    (
        6,
        "watch_folder_source_key_normalization",
        include_str!("../../migrations/0006_watch_folder_source_key_normalization.sql"),
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

    const HISTORICAL_REPLAY_FRAGMENTS: &[&str] = &[
        "-- From ",
        "ALTER TABLE",
        "ADD COLUMN",
        "DROP INDEX",
        "DROP CONSTRAINT",
        "DELETE FROM",
    ];

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

    #[test]
    fn baseline_migration_describes_direct_schema_shape() {
        assert!(MIGRATIONS.len() >= 2);
        assert_eq!(MIGRATIONS[0].0, 1);
        assert_eq!(MIGRATIONS[0].1, "baseline");

        let sql = MIGRATIONS[0].2;
        for fragment in HISTORICAL_REPLAY_FRAGMENTS {
            assert!(
                !sql.contains(fragment),
                "SQLite baseline still contains historical replay fragment: {fragment}"
            );
        }
    }

    #[test]
    fn latest_incremental_migrations_are_registered() {
        let migration = MIGRATIONS
            .iter()
            .find(|(version, _, _)| *version == 5)
            .expect("SQLite source duplicate pair migration should be registered");

        assert_eq!(migration.1, "source_duplicate_pair_identity");
        assert!(
            migration
                .2
                .contains("source_duplicate_relationships_pair_idx")
        );
        assert!(migration.2.contains("source_id, duplicate_source_id"));

        let migration = MIGRATIONS
            .iter()
            .find(|(version, _, _)| *version == 6)
            .expect("SQLite watch-folder source-key migration should be registered");

        assert_eq!(migration.1, "watch_folder_source_key_normalization");
        assert!(
            migration
                .2
                .contains("watch_folder_source_key_normalization")
        );
        assert!(migration.2.contains("'watch_folder:' || source_uri"));
    }

    #[tokio::test]
    async fn baseline_migration_creates_identity_and_library_access_schema() {
        let store = SqliteStore::connect_in_memory().await.unwrap();

        store.migrate().await.unwrap();

        let applied_versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(store.pool())
                .await
                .unwrap();

        assert_eq!(applied_versions, vec![1, 2, 3, 4, 5, 6]);

        for table in [
            "users",
            "user_role_assignments",
            "user_library_access_policies",
            "role_library_access_policies",
        ] {
            let exists: Option<i64> = sqlx::query_scalar(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            )
            .bind(table)
            .fetch_optional(store.pool())
            .await
            .unwrap();

            assert_eq!(exists, Some(1), "missing baseline table {table}");
        }

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json, domain, preset, options_json)
            VALUES ('018f0000-0000-7000-8000-000000000001', 'Movies', '[]', 'video', 'movies', '{}')
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
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
            VALUES (
                '018f0000-0000-7000-8000-000000000002',
                'local-admin',
                'admin',
                'admin',
                'Local administrator',
                'active',
                1,
                1
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO user_role_assignments (user_id, role, granted_at_ms)
            VALUES ('018f0000-0000-7000-8000-000000000002', 'administrator', 1)
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO user_library_access_policies (
                user_id,
                library_id,
                access,
                created_at_ms,
                updated_at_ms
            )
            VALUES (
                '018f0000-0000-7000-8000-000000000002',
                '018f0000-0000-7000-8000-000000000001',
                'manage',
                1,
                1
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO role_library_access_policies (
                role,
                library_id,
                access,
                created_at_ms,
                updated_at_ms
            )
            VALUES (
                'viewer',
                '018f0000-0000-7000-8000-000000000001',
                'play',
                1,
                1
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
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
    async fn watch_folder_source_key_migration_normalizes_legacy_rows() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json, domain, preset, options_json)
            VALUES ('018f0000-0000-7000-8000-000000000101', 'Movies', '[]', 'video', 'movies', '{}')
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO acquisition_intake_candidates (
                id, target_library_id, source_kind, source_kind_key, source_key,
                source_uri, display_name, intended_locator, size_bytes, fingerprint,
                managed_import_artifact_id, state, diagnostics_json, first_seen_at_ms,
                last_seen_at_ms, created_at_ms, updated_at_ms
            )
            VALUES (
                '018f0000-0000-7000-8000-000000000102',
                '018f0000-0000-7000-8000-000000000101',
                'watch_folder',
                '',
                'local:///watch/Legacy.mkv|size=9|fingerprint=old',
                'local:///watch/Legacy.mkv',
                'Legacy.mkv',
                NULL,
                9,
                'old',
                NULL,
                'inspecting',
                '{"stable":false}',
                100,
                110,
                100,
                110
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();

        let migration = MIGRATIONS
            .iter()
            .find(|(version, _, _)| *version == 6)
            .unwrap();
        sqlx::raw_sql(migration.2)
            .execute(store.pool())
            .await
            .unwrap();

        let rows: Vec<(String, String, String, Option<String>, i64, i64)> = sqlx::query_as(
            r#"
            SELECT id, source_key, state, diagnostics_json, first_seen_at_ms, last_seen_at_ms
            FROM acquisition_intake_candidates
            ORDER BY id
            "#,
        )
        .fetch_all(store.pool())
        .await
        .unwrap();

        assert_eq!(
            rows,
            vec![(
                "018f0000-0000-7000-8000-000000000102".to_owned(),
                "watch_folder:local:///watch/Legacy.mkv".to_owned(),
                "inspecting".to_owned(),
                Some(r#"{"stable":false}"#.to_owned()),
                100,
                110
            )]
        );
    }

    #[tokio::test]
    async fn watch_folder_source_key_migration_collapses_duplicate_rows() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json, domain, preset, options_json)
            VALUES ('018f0000-0000-7000-8000-000000000201', 'Movies', '[]', 'video', 'movies', '{}')
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO managed_import_artifacts (
                id, target_library_id, source_kind, source_kind_key, source_uri,
                artifact_uri, original_file_name, intended_locator, size_bytes,
                fingerprint, state, diagnostics_json, created_at_ms, updated_at_ms
            )
            VALUES (
                '018f0000-0000-7000-8000-000000000202',
                '018f0000-0000-7000-8000-000000000201',
                'watched_candidate',
                '',
                'local:///watch/Duplicate.mkv',
                'local:///library/Duplicate.mkv',
                'Duplicate.mkv',
                'Movies/Duplicate.mkv',
                12,
                'duplicate-fingerprint',
                'staged',
                '{"artifact":true}',
                200,
                200
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();
        sqlx::query(
            r#"
            INSERT INTO acquisition_intake_candidates (
                id, target_library_id, source_kind, source_kind_key, source_key,
                source_uri, display_name, intended_locator, size_bytes, fingerprint,
                managed_import_artifact_id, state, diagnostics_json, first_seen_at_ms,
                last_seen_at_ms, created_at_ms, updated_at_ms
            )
            VALUES
            (
                '018f0000-0000-7000-8000-000000000203',
                '018f0000-0000-7000-8000-000000000201',
                'watch_folder',
                '',
                'watch_folder:local:///watch/Duplicate.mkv',
                'local:///watch/Duplicate.mkv',
                NULL,
                NULL,
                NULL,
                NULL,
                NULL,
                'ready',
                NULL,
                220,
                260,
                220,
                260
            ),
            (
                '018f0000-0000-7000-8000-000000000204',
                '018f0000-0000-7000-8000-000000000201',
                'watch_folder',
                '',
                'local:///watch/Duplicate.mkv|size=12|fingerprint=duplicate-fingerprint',
                'local:///watch/Duplicate.mkv',
                'Duplicate.mkv',
                'Movies/Duplicate.mkv',
                12,
                'duplicate-fingerprint',
                '018f0000-0000-7000-8000-000000000202',
                'accepted',
                '{"accepted":true}',
                210,
                240,
                210,
                240
            )
            "#,
        )
        .execute(store.pool())
        .await
        .unwrap();

        let migration = MIGRATIONS
            .iter()
            .find(|(version, _, _)| *version == 6)
            .unwrap();
        sqlx::raw_sql(migration.2)
            .execute(store.pool())
            .await
            .unwrap();

        let rows: Vec<(
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            i64,
            i64,
        )> = sqlx::query_as(
            r#"
            SELECT
                id,
                source_key,
                state,
                managed_import_artifact_id,
                display_name,
                size_bytes,
                fingerprint,
                diagnostics_json,
                first_seen_at_ms,
                last_seen_at_ms
            FROM acquisition_intake_candidates
            ORDER BY id
            "#,
        )
        .fetch_all(store.pool())
        .await
        .unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0],
            (
                "018f0000-0000-7000-8000-000000000204".to_owned(),
                "watch_folder:local:///watch/Duplicate.mkv".to_owned(),
                "accepted".to_owned(),
                Some("018f0000-0000-7000-8000-000000000202".to_owned()),
                Some("Duplicate.mkv".to_owned()),
                Some(12),
                Some("duplicate-fingerprint".to_owned()),
                Some(r#"{"accepted":true}"#.to_owned()),
                210,
                260
            )
        );
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
