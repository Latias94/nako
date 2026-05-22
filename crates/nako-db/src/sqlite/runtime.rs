use std::{str::FromStr, time::Duration};

use nako_core::Result;
use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
};

use super::{SqliteStore, codec::database_error};

const ON_DISK_MAX_CONNECTIONS: u32 = 8;
const IN_MEMORY_MAX_CONNECTIONS: u32 = 1;
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SqliteRuntimeOptions {
    pub max_connections: u32,
    pub busy_timeout: Duration,
    pub journal_mode: Option<SqliteJournalMode>,
    pub synchronous: Option<SqliteSynchronous>,
}

impl SqliteRuntimeOptions {
    #[must_use]
    pub const fn on_disk() -> Self {
        Self {
            max_connections: ON_DISK_MAX_CONNECTIONS,
            busy_timeout: SQLITE_BUSY_TIMEOUT,
            journal_mode: Some(SqliteJournalMode::Wal),
            synchronous: Some(SqliteSynchronous::Normal),
        }
    }

    #[must_use]
    pub const fn in_memory() -> Self {
        Self {
            max_connections: IN_MEMORY_MAX_CONNECTIONS,
            busy_timeout: SQLITE_BUSY_TIMEOUT,
            journal_mode: None,
            synchronous: None,
        }
    }

    fn apply_to(self, mut options: SqliteConnectOptions) -> SqliteConnectOptions {
        options = options.foreign_keys(true).busy_timeout(self.busy_timeout);
        if let Some(journal_mode) = self.journal_mode {
            options = options.journal_mode(journal_mode);
        }
        if let Some(synchronous) = self.synchronous {
            options = options.synchronous(synchronous);
        }
        options
    }
}

impl SqliteStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let runtime = if is_in_memory_database_url(database_url) {
            SqliteRuntimeOptions::in_memory()
        } else {
            SqliteRuntimeOptions::on_disk()
        };
        Self::connect_with_runtime(database_url, runtime).await
    }

    pub async fn connect_with_runtime(
        database_url: &str,
        runtime: SqliteRuntimeOptions,
    ) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(database_error)?
            .create_if_missing(true);

        Self::connect_with_options(options, runtime).await
    }

    pub async fn connect_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:").map_err(database_error)?;
        Self::connect_with_options(options, SqliteRuntimeOptions::in_memory()).await
    }

    async fn connect_with_options(
        options: SqliteConnectOptions,
        runtime: SqliteRuntimeOptions,
    ) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(runtime.max_connections)
            .connect_with(runtime.apply_to(options))
            .await
            .map_err(database_error)?;

        Ok(Self { pool })
    }

    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

fn is_in_memory_database_url(database_url: &str) -> bool {
    database_url.contains(":memory:") || database_url.contains("mode=memory")
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqliteConnectOptions;

    use super::*;

    #[tokio::test]
    async fn on_disk_runtime_uses_wal_and_busy_timeout() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database_path = temp_dir.path().join("runtime.db");
        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);

        let store = SqliteStore::connect_with_options(
            options,
            SqliteRuntimeOptions {
                max_connections: 2,
                ..SqliteRuntimeOptions::on_disk()
            },
        )
        .await
        .unwrap();

        let journal_mode: String = sqlx::query_scalar("PRAGMA journal_mode")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let busy_timeout_ms: i64 = sqlx::query_scalar("PRAGMA busy_timeout")
            .fetch_one(store.pool())
            .await
            .unwrap();
        let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
            .fetch_one(store.pool())
            .await
            .unwrap();

        assert_eq!(journal_mode, "wal");
        assert_eq!(busy_timeout_ms, SQLITE_BUSY_TIMEOUT.as_millis() as i64);
        assert_eq!(foreign_keys, 1);
    }

    #[tokio::test]
    async fn memory_runtime_keeps_a_single_connection() {
        assert!(is_in_memory_database_url("sqlite::memory:"));
        assert!(is_in_memory_database_url("sqlite://?mode=memory"));
        assert_eq!(
            SqliteRuntimeOptions::in_memory().max_connections,
            IN_MEMORY_MAX_CONNECTIONS
        );
    }
}
