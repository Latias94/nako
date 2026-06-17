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
    use std::sync::Arc;

    use nako_core::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use tokio::sync::Notify;

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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn on_disk_runtime_keeps_playback_writes_pending_while_a_write_lock_is_held() {
        let temp_dir = tempfile::tempdir().unwrap();
        let database_path = temp_dir.path().join("pressure.db");
        let options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);

        let store = SqliteStore::connect_with_options(
            options,
            SqliteRuntimeOptions {
                max_connections: 4,
                ..SqliteRuntimeOptions::on_disk()
            },
        )
        .await
        .unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Pressure".to_owned(),
            roots: vec!["local:///Pressure".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Pressure Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "local:///Pressure/Pressure Demo.mkv".to_owned(),
            file_name: "Pressure Demo.mkv".to_owned(),
            size_bytes: Some(128),
            fingerprint: Some("pressure-demo".to_owned()),
        };
        let playback_session_id = PlaybackSessionId::new();
        let transcode_session_id = TranscodeSessionId::new();
        let metrics = TranscodeSessionRuntimeMetrics {
            frame_count: Some(12),
            output_time_ms: Some(1_500),
            speed_millis: Some(1_000),
            progress: Some(TranscodeSessionRuntimeProgress::Continue),
            ..TranscodeSessionRuntimeMetrics::default()
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();
        store
            .create_playback_session(NewPlaybackSession {
                id: playback_session_id,
                principal_id: UserPrincipalId::local_admin(),
                source_id: source.id,
                item_id: source.item_id,
                mode: PlaybackSessionMode::Direct,
                state: PlaybackSessionState::Active,
                client_capabilities_json: None,
                started_at_ms: 1_779_814_400_000,
                updated_at_ms: 1_779_814_400_000,
            })
            .await
            .unwrap();
        store
            .create_transcode_session(NewTranscodeSession {
                id: transcode_session_id,
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: "pressure:remux".to_owned(),
                output_path: "cache/remux/pressure.mp4".into(),
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();

        let mut lock_conn = store.pool().acquire().await.unwrap();
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *lock_conn)
            .await
            .unwrap();

        let heartbeat_started = Arc::new(Notify::new());
        let metrics_started = Arc::new(Notify::new());

        let heartbeat_handle = {
            let store = store.clone();
            let started = heartbeat_started.clone();
            tokio::spawn(async move {
                started.notify_one();
                store
                    .record_playback_session_heartbeat(PlaybackSessionHeartbeat {
                        id: playback_session_id,
                        state: PlaybackSessionState::Paused,
                        position_ms: Some(42_000),
                        duration_ms: Some(600_000),
                        heartbeat_at_ms: 1_779_814_401_000,
                    })
                    .await
            })
        };

        let metrics_handle = {
            let store = store.clone();
            let started = metrics_started.clone();
            let metrics = metrics.clone();
            tokio::spawn(async move {
                started.notify_one();
                store
                    .update_transcode_session_runtime_metrics(transcode_session_id, metrics)
                    .await
            })
        };

        heartbeat_started.notified().await;
        metrics_started.notified().await;
        assert!(!heartbeat_handle.is_finished());
        assert!(!metrics_handle.is_finished());

        sqlx::query("ROLLBACK")
            .execute(&mut *lock_conn)
            .await
            .unwrap();
        drop(lock_conn);

        let heartbeat = tokio::time::timeout(Duration::from_secs(5), heartbeat_handle)
            .await
            .expect("playback heartbeat should finish after the write lock is released")
            .expect("playback heartbeat task should not panic")
            .expect("playback heartbeat write should succeed")
            .expect("playback heartbeat should update an active session");
        let updated_metrics = tokio::time::timeout(Duration::from_secs(5), metrics_handle)
            .await
            .expect("transcode metrics should finish after the write lock is released")
            .expect("transcode metrics task should not panic")
            .expect("transcode metrics write should succeed")
            .expect("transcode metrics should update the running session");

        assert_eq!(heartbeat.state, PlaybackSessionState::Paused);
        assert_eq!(heartbeat.last_heartbeat_at_ms, Some(1_779_814_401_000));
        assert_eq!(updated_metrics.runtime_metrics, metrics);
    }
}
