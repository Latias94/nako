use std::{fmt::Display, str::FromStr};

use sqlx::{
    Decode, Row, Sqlite, SqlitePool, Type,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
};
use taru_core::{
    CanonicalMetadata, ExternalId, ExternalProvider, Library, LibraryId, LibraryRepository,
    MediaItem, MediaItemId, MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository,
    MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind, Result, TaruError,
    TransactionManager,
};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../migrations/0001_initial.sql"),
    ),
    (
        "0002_media_probe",
        include_str!("../migrations/0002_media_probe.sql"),
    ),
];

#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(database_url)
            .map_err(database_error)?
            .create_if_missing(true)
            .foreign_keys(true);

        Self::connect_with(options).await
    }

    pub async fn connect_in_memory() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .map_err(database_error)?
            .foreign_keys(true);

        Self::connect_with(options).await
    }

    async fn connect_with(options: SqliteConnectOptions) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .map_err(database_error)?;

        Ok(Self { pool })
    }

    #[must_use]
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[async_trait::async_trait]
impl TransactionManager for SqliteStore {
    async fn migrate(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS taru_schema_migrations (
                version TEXT PRIMARY KEY NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        for (version, sql) in MIGRATIONS {
            let already_applied = sqlx::query(
                r#"
                SELECT version
                FROM taru_schema_migrations
                WHERE version = ?1
                "#,
            )
            .bind(*version)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?
            .is_some();

            if already_applied {
                continue;
            }

            let mut transaction = self.pool.begin().await.map_err(database_error)?;

            for statement in split_sql_statements(sql) {
                sqlx::query(&statement)
                    .execute(&mut *transaction)
                    .await
                    .map_err(database_error)?;
            }

            sqlx::query("INSERT INTO taru_schema_migrations (version) VALUES (?1)")
                .bind(*version)
                .execute(&mut *transaction)
                .await
                .map_err(database_error)?;

            transaction.commit().await.map_err(database_error)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl LibraryRepository for SqliteStore {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        let roots_json = serde_json::to_string(&library.roots).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(library.id.to_string())
        .bind(&library.name)
        .bind(roots_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, roots_json
            FROM libraries
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id = parse_id(row_get::<String>(&row, "id")?)?;
        let roots_json = row_get::<String>(&row, "roots_json")?;
        let roots = serde_json::from_str(&roots_json).map_err(database_error)?;

        Ok(Some(Library {
            id,
            name: row_get(&row, "name")?,
            roots,
        }))
    }
}

#[async_trait::async_trait]
impl MediaRepository for SqliteStore {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO media_items (
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                parent_id = excluded.parent_id,
                title = excluded.title,
                original_title = excluded.original_title,
                sort_title = excluded.sort_title,
                overview = excluded.overview,
                release_date = excluded.release_date,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(item.id.to_string())
        .bind(media_kind_to_str(item.kind))
        .bind(item.parent_id.map(|id| id.to_string()))
        .bind(&item.metadata.title)
        .bind(&item.metadata.original_title)
        .bind(&item.metadata.sort_title)
        .bind(&item.metadata.overview)
        .bind(&item.metadata.release_date)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM media_item_external_ids WHERE item_id = ?1")
            .bind(item.id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for external_id in &item.metadata.external_ids {
            let (provider, provider_key) = provider_to_parts(&external_id.provider);
            sqlx::query(
                r#"
                INSERT INTO media_item_external_ids (item_id, provider, provider_key, value)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .bind(item.id.to_string())
            .bind(provider)
            .bind(provider_key)
            .bind(&external_id.value)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT id, kind, parent_id, title, original_title, sort_title, overview, release_date
            FROM media_items
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self.list_external_ids(id).await?;

        Ok(Some(MediaItem {
            id: parse_id(row_get::<String>(&row, "id")?)?,
            kind: parse_media_kind(row_get::<String>(&row, "kind")?)?,
            parent_id: parse_optional_id(row_get::<Option<String>>(&row, "parent_id")?)?,
            metadata: CanonicalMetadata {
                title: row_get(&row, "title")?,
                original_title: row_get(&row, "original_title")?,
                sort_title: row_get(&row, "sort_title")?,
                overview: row_get(&row, "overview")?,
                release_date: row_get(&row, "release_date")?,
                external_ids,
            },
        }))
    }

    async fn upsert_media_source(&self, library_id: LibraryId, source: &MediaSource) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO media_sources (
                id,
                library_id,
                item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                library_id = excluded.library_id,
                item_id = excluded.item_id,
                locator = excluded.locator,
                file_name = excluded.file_name,
                size_bytes = excluded.size_bytes,
                fingerprint = excluded.fingerprint,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(source.id.to_string())
        .bind(library_id.to_string())
        .bind(source.item_id.to_string())
        .bind(&source.locator)
        .bind(&source.file_name)
        .bind(optional_u64_to_i64(source.size_bytes)?)
        .bind(&source.fingerprint)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_media_source_by_locator(&self, locator: &str) -> Result<Option<MediaSource>> {
        let row = sqlx::query(
            r#"
            SELECT id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE locator = ?1
            "#,
        )
        .bind(locator)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_media_source).transpose()
    }

    async fn list_media_sources(&self, library_id: LibraryId) -> Result<Vec<MediaSource>> {
        let rows = sqlx::query(
            r#"
            SELECT id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE library_id = ?1
            ORDER BY locator ASC
            "#,
        )
        .bind(library_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
    }
}

#[async_trait::async_trait]
impl MediaProbeRepository for SqliteStore {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO media_source_probes (source_id, duration_ms, container, bit_rate)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(source_id) DO UPDATE SET
                duration_ms = excluded.duration_ms,
                container = excluded.container,
                bit_rate = excluded.bit_rate,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(source_id.to_string())
        .bind(optional_u64_to_i64(result.duration_ms)?)
        .bind(&result.container)
        .bind(optional_u64_to_i64(result.bit_rate)?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM media_streams WHERE source_id = ?1")
            .bind(source_id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for stream in &result.streams {
            let (kind, kind_key) = stream_kind_to_parts(&stream.kind);

            sqlx::query(
                r#"
                INSERT INTO media_streams (
                    source_id,
                    stream_index,
                    kind,
                    kind_key,
                    codec,
                    language,
                    duration_ms,
                    bit_rate,
                    width,
                    height,
                    channels,
                    sample_rate
                )
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                "#,
            )
            .bind(source_id.to_string())
            .bind(u32_to_i64(stream.index))
            .bind(kind)
            .bind(kind_key)
            .bind(&stream.codec)
            .bind(&stream.language)
            .bind(optional_u64_to_i64(stream.duration_ms)?)
            .bind(optional_u64_to_i64(stream.bit_rate)?)
            .bind(optional_u32_to_i64(stream.width))
            .bind(optional_u32_to_i64(stream.height))
            .bind(optional_u32_to_i64(stream.channels))
            .bind(optional_u32_to_i64(stream.sample_rate))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_probe(&self, source_id: MediaSourceId) -> Result<Option<MediaProbeResult>> {
        let row = sqlx::query(
            r#"
            SELECT duration_ms, container, bit_rate
            FROM media_source_probes
            WHERE source_id = ?1
            "#,
        )
        .bind(source_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let stream_rows = sqlx::query(
            r#"
            SELECT
                stream_index,
                kind,
                kind_key,
                codec,
                language,
                duration_ms,
                bit_rate,
                width,
                height,
                channels,
                sample_rate
            FROM media_streams
            WHERE source_id = ?1
            ORDER BY stream_index ASC
            "#,
        )
        .bind(source_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let streams = stream_rows
            .into_iter()
            .map(row_to_stream_info)
            .collect::<Result<Vec<_>>>()?;

        Ok(Some(MediaProbeResult {
            duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
            container: row_get(&row, "container")?,
            bit_rate: optional_i64_to_u64(row_get(&row, "bit_rate")?)?,
            streams,
        }))
    }
}

impl SqliteStore {
    async fn list_external_ids(&self, item_id: MediaItemId) -> Result<Vec<ExternalId>> {
        let rows = sqlx::query(
            r#"
            SELECT provider, provider_key, value
            FROM media_item_external_ids
            WHERE item_id = ?1
            ORDER BY provider ASC, provider_key ASC, value ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                let provider = provider_from_parts(
                    row_get::<String>(&row, "provider")?,
                    row_get::<String>(&row, "provider_key")?,
                );

                Ok(ExternalId {
                    provider,
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }
}

fn media_kind_to_str(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Movie => "movie",
        MediaKind::Series => "series",
        MediaKind::Season => "season",
        MediaKind::Episode => "episode",
        MediaKind::Collection => "collection",
        MediaKind::Extra => "extra",
        MediaKind::Unknown => "unknown",
    }
}

fn parse_media_kind(value: String) -> Result<MediaKind> {
    match value.as_str() {
        "movie" => Ok(MediaKind::Movie),
        "series" => Ok(MediaKind::Series),
        "season" => Ok(MediaKind::Season),
        "episode" => Ok(MediaKind::Episode),
        "collection" => Ok(MediaKind::Collection),
        "extra" => Ok(MediaKind::Extra),
        "unknown" => Ok(MediaKind::Unknown),
        _ => Err(TaruError::Database {
            message: format!("unknown media kind stored in database: {value}"),
        }),
    }
}

fn provider_to_parts(provider: &ExternalProvider) -> (String, String) {
    match provider {
        ExternalProvider::Tmdb => ("tmdb".to_owned(), String::new()),
        ExternalProvider::Douban => ("douban".to_owned(), String::new()),
        ExternalProvider::Bangumi => ("bangumi".to_owned(), String::new()),
        ExternalProvider::Imdb => ("imdb".to_owned(), String::new()),
        ExternalProvider::Local => ("local".to_owned(), String::new()),
        ExternalProvider::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn provider_from_parts(provider: String, provider_key: String) -> ExternalProvider {
    match provider.as_str() {
        "tmdb" => ExternalProvider::Tmdb,
        "douban" => ExternalProvider::Douban,
        "bangumi" => ExternalProvider::Bangumi,
        "imdb" => ExternalProvider::Imdb,
        "local" => ExternalProvider::Local,
        "other" => ExternalProvider::Other(provider_key),
        _ => ExternalProvider::Other(provider),
    }
}

fn stream_kind_to_parts(kind: &MediaStreamKind) -> (String, String) {
    match kind {
        MediaStreamKind::Video => ("video".to_owned(), String::new()),
        MediaStreamKind::Audio => ("audio".to_owned(), String::new()),
        MediaStreamKind::Subtitle => ("subtitle".to_owned(), String::new()),
        MediaStreamKind::Data => ("data".to_owned(), String::new()),
        MediaStreamKind::Attachment => ("attachment".to_owned(), String::new()),
        MediaStreamKind::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn stream_kind_from_parts(kind: String, kind_key: String) -> MediaStreamKind {
    match kind.as_str() {
        "video" => MediaStreamKind::Video,
        "audio" => MediaStreamKind::Audio,
        "subtitle" => MediaStreamKind::Subtitle,
        "data" => MediaStreamKind::Data,
        "attachment" => MediaStreamKind::Attachment,
        "other" => MediaStreamKind::Other(kind_key),
        _ => MediaStreamKind::Other(kind),
    }
}

fn parse_id<T>(value: String) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    value.parse().map_err(database_error)
}

fn parse_optional_id<T>(value: Option<String>) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Display,
{
    value.map(parse_id).transpose()
}

fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>> {
    value
        .map(|value| {
            i64::try_from(value).map_err(|err| TaruError::Database {
                message: format!("value does not fit into SQLite integer: {err}"),
            })
        })
        .transpose()
}

fn optional_i64_to_u64(value: Option<i64>) -> Result<Option<u64>> {
    value
        .map(|value| {
            u64::try_from(value).map_err(|err| TaruError::Database {
                message: format!("negative SQLite integer cannot be converted to u64: {err}"),
            })
        })
        .transpose()
}

fn optional_u32_to_i64(value: Option<u32>) -> Option<i64> {
    value.map(i64::from)
}

fn optional_i64_to_u32(value: Option<i64>) -> Result<Option<u32>> {
    value
        .map(|value| {
            u32::try_from(value).map_err(|err| TaruError::Database {
                message: format!("SQLite integer cannot be converted to u32: {err}"),
            })
        })
        .transpose()
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

fn i64_to_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|err| TaruError::Database {
        message: format!("SQLite integer cannot be converted to u32: {err}"),
    })
}

fn row_to_media_source(row: SqliteRow) -> Result<MediaSource> {
    Ok(MediaSource {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        locator: row_get(&row, "locator")?,
        file_name: row_get(&row, "file_name")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        fingerprint: row_get(&row, "fingerprint")?,
    })
}

fn row_to_stream_info(row: SqliteRow) -> Result<MediaStreamInfo> {
    Ok(MediaStreamInfo {
        index: i64_to_u32(row_get(&row, "stream_index")?)?,
        kind: stream_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        codec: row_get(&row, "codec")?,
        language: row_get(&row, "language")?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        bit_rate: optional_i64_to_u64(row_get(&row, "bit_rate")?)?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        channels: optional_i64_to_u32(row_get(&row, "channels")?)?,
        sample_rate: optional_i64_to_u32(row_get(&row, "sample_rate")?)?,
    })
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(str::trim)
        .filter(|statement| !statement.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn row_get<T>(row: &SqliteRow, column: &str) -> Result<T>
where
    for<'row> T: Decode<'row, Sqlite> + Type<Sqlite>,
{
    row.try_get(column).map_err(database_error)
}

fn database_error(error: impl Display) -> TaruError {
    TaruError::Database {
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use taru_core::MediaSourceId;

    use super::*;

    #[tokio::test]
    async fn sqlite_store_persists_libraries() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
        };

        store.upsert_library(&library).await.unwrap();
        let loaded = store.get_library(library.id).await.unwrap();

        assert_eq!(loaded, Some(library));
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_media_items_and_sources() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "The Matrix".to_owned(),
                original_title: None,
                sort_title: Some("Matrix, The".to_owned()),
                overview: Some("A hacker discovers the nature of reality.".to_owned()),
                release_date: Some("1999-03-31".to_owned()),
                external_ids: vec![
                    ExternalId {
                        provider: ExternalProvider::Tmdb,
                        value: "603".to_owned(),
                    },
                    ExternalId {
                        provider: ExternalProvider::Other("custom".to_owned()),
                        value: "matrix-local".to_owned(),
                    },
                ],
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///Movies/The Matrix (1999).mkv".to_owned(),
            file_name: "The Matrix (1999).mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: Some("fingerprint".to_owned()),
        };

        let mut expected_item = item.clone();
        expected_item
            .metadata
            .external_ids
            .sort_by(|left, right| external_id_sort_key(left).cmp(&external_id_sort_key(right)));

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library.id, &source)
            .await
            .unwrap();

        assert_eq!(
            store.get_media_item(item.id).await.unwrap(),
            Some(expected_item)
        );
        assert_eq!(
            store.list_media_sources(library.id).await.unwrap(),
            vec![source]
        );
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_media_probe_results() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Probe Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///Movies/Probe Demo.mkv".to_owned(),
            file_name: "Probe Demo.mkv".to_owned(),
            size_bytes: Some(1024),
            fingerprint: None,
        };
        let result = MediaProbeResult {
            duration_ms: Some(120_253),
            container: Some("matroska,webm".to_owned()),
            bit_rate: Some(4_200_000),
            streams: vec![
                MediaStreamInfo {
                    index: 0,
                    kind: MediaStreamKind::Video,
                    codec: Some("h264".to_owned()),
                    language: Some("und".to_owned()),
                    duration_ms: Some(120_250),
                    bit_rate: Some(4_000_000),
                    width: Some(1920),
                    height: Some(1080),
                    channels: None,
                    sample_rate: None,
                },
                MediaStreamInfo {
                    index: 1,
                    kind: MediaStreamKind::Audio,
                    codec: Some("aac".to_owned()),
                    language: Some("eng".to_owned()),
                    duration_ms: Some(120_240),
                    bit_rate: Some(128_000),
                    width: None,
                    height: None,
                    channels: Some(2),
                    sample_rate: Some(48_000),
                },
                MediaStreamInfo {
                    index: 2,
                    kind: MediaStreamKind::Other("timed_id3".to_owned()),
                    codec: None,
                    language: None,
                    duration_ms: None,
                    bit_rate: None,
                    width: None,
                    height: None,
                    channels: None,
                    sample_rate: None,
                },
            ],
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library.id, &source)
            .await
            .unwrap();
        store.upsert_media_probe(source.id, &result).await.unwrap();

        assert_eq!(
            store.get_media_probe(source.id).await.unwrap(),
            Some(result)
        );
    }

    fn external_id_sort_key(external_id: &ExternalId) -> String {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        format!("{provider}\0{provider_key}\0{}", external_id.value)
    }
}
