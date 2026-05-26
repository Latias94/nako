use std::borrow::Cow;

use nako_core::{DatabaseLifecycle, Result};
use sqlx::migrate::{Migration, MigrationType, Migrator};

use super::{SqliteStore, codec::database_error};

const MIGRATIONS: &[(i64, &str, &str)] =
    &[(1, "baseline", include_str!("../../migrations/baseline.sql"))];

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
    async fn baseline_migration_creates_identity_and_library_access_schema() {
        let store = SqliteStore::connect_in_memory().await.unwrap();

        store.migrate().await.unwrap();

        let applied_versions: Vec<i64> =
            sqlx::query_scalar("SELECT version FROM _sqlx_migrations ORDER BY version")
                .fetch_all(store.pool())
                .await
                .unwrap();

        assert_eq!(applied_versions, vec![1]);

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
