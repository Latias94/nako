use std::{fmt::Display, str::FromStr};

use sqlx::{
    Decode, Row, Sqlite, SqlitePool, Type,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
};
use taru_core::{
    ArtworkTask, ArtworkTaskId, ArtworkTaskKind, ArtworkTaskRepository, CanonicalMetadata,
    CatalogRepository, Collection, CollectionId, CollectionItem, CreditRole, DirectorySnapshot,
    ExternalId, ExternalProvider, Genre, GenreId, ImageAsset, ImageAssetId, ImageKind, ImageOwner,
    ItemCredit, ItemGenre, ItemStudio, ItemTag, Job, JobId, JobKind, JobRepository, JobStatus,
    Library, LibraryId, LibraryOptions, LibraryRepository, MediaDomain, MediaItem, MediaItemId,
    MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository, MediaSource, MediaSourceId,
    MediaStreamInfo, MediaStreamKind, MetadataField, MetadataFieldLock, MetadataRepository,
    MetadataSource, NewJob, PageRequest, Person, PersonId, ProviderRawResponse, Result,
    ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus, SourceState, Studio, StudioId, Tag,
    TagId, TaruError, TransactionManager,
};
use taru_search::{SearchDocument, SearchHit, SearchIndex, SearchQuery};

const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_initial",
        include_str!("../migrations/0001_initial.sql"),
    ),
    (
        "0002_media_probe",
        include_str!("../migrations/0002_media_probe.sql"),
    ),
    ("0003_jobs", include_str!("../migrations/0003_jobs.sql")),
    (
        "0004_job_input_payload",
        include_str!("../migrations/0004_job_input_payload.sql"),
    ),
    (
        "0005_metadata_policy",
        include_str!("../migrations/0005_metadata_policy.sql"),
    ),
    (
        "0006_library_profiles",
        include_str!("../migrations/0006_library_profiles.sql"),
    ),
    (
        "0007_catalog_ingestion",
        include_str!("../migrations/0007_catalog_ingestion.sql"),
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
        let options_json = serde_json::to_string(&library.options).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (id, name, roots_json, domain, preset, options_json)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                domain = excluded.domain,
                preset = excluded.preset,
                options_json = excluded.options_json,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(library.id.to_string())
        .bind(&library.name)
        .bind(roots_json)
        .bind(library.options.domain.as_str())
        .bind(library.options.preset.as_str())
        .bind(options_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, roots_json, domain, preset, options_json
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
        let options = row_to_library_options(&row)?;

        Ok(Some(Library {
            id,
            name: row_get(&row, "name")?,
            roots,
            options,
        }))
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, roots_json, domain, preset, options_json
            FROM libraries
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library).collect()
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
                release_date,
                metadata_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                kind = excluded.kind,
                parent_id = excluded.parent_id,
                title = excluded.title,
                original_title = excluded.original_title,
                sort_title = excluded.sort_title,
                overview = excluded.overview,
                release_date = excluded.release_date,
                metadata_json = excluded.metadata_json,
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
        .bind(serialize_metadata_json(&item.metadata)?)
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
            SELECT
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json
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

        Ok(Some(row_to_media_item(row, external_ids)?))
    }

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json
            FROM media_items
            ORDER BY title ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
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

    async fn list_item_sources(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE item_id = ?1
            ORDER BY locator ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(item_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
    }

    async fn list_media_sources(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaSource>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, item_id, locator, file_name, size_bytes, fingerprint
            FROM media_sources
            WHERE library_id = ?1
            ORDER BY locator ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
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

#[async_trait::async_trait]
impl MetadataRepository for SqliteStore {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        let (source, source_key) = metadata_source_to_parts(&lock.source);

        sqlx::query(
            r#"
            INSERT INTO metadata_field_locks (item_id, field, locked, source, source_key)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id, field) DO UPDATE SET
                locked = excluded.locked,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(lock.item_id.to_string())
        .bind(lock.field.as_str())
        .bind(bool_to_i64(lock.locked))
        .bind(source)
        .bind(source_key)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, field, locked, source, source_key
            FROM metadata_field_locks
            WHERE item_id = ?1
            ORDER BY field ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_metadata_field_lock).collect()
    }

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()> {
        let (provider, provider_key) = provider_to_parts(&response.provider);
        let provider_key = if response.provider_key.is_empty() {
            provider_key
        } else {
            response.provider_key.clone()
        };

        sqlx::query(
            r#"
            INSERT INTO provider_raw_responses (
                item_id,
                provider,
                provider_key,
                body_json,
                fetched_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id, provider, provider_key) DO UPDATE SET
                body_json = excluded.body_json,
                fetched_at = excluded.fetched_at,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(response.item_id.to_string())
        .bind(provider)
        .bind(provider_key)
        .bind(&response.body_json)
        .bind(&response.fetched_at)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_provider_raw_response(
        &self,
        item_id: MediaItemId,
        provider: &ExternalProvider,
        provider_key: &str,
    ) -> Result<Option<ProviderRawResponse>> {
        let (provider, default_provider_key) = provider_to_parts(provider);
        let provider_key = if provider_key.is_empty() {
            default_provider_key
        } else {
            provider_key.to_owned()
        };
        let row = sqlx::query(
            r#"
            SELECT item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = ?1 AND provider = ?2 AND provider_key = ?3
            LIMIT 1
            "#,
        )
        .bind(item_id.to_string())
        .bind(provider)
        .bind(provider_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_raw_response).transpose()
    }
}

#[async_trait::async_trait]
impl JobRepository for SqliteStore {
    async fn enqueue_job(&self, job: NewJob) -> Result<Job> {
        sqlx::query(
            r#"
            INSERT INTO jobs (
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
        )
        .bind(job.id.to_string())
        .bind(job.kind.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(job.resource_class)
        .bind(job.library_id.map(|id| id.to_string()))
        .bind(job.source_id.map(|id| id.to_string()))
        .bind(job.input_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(job.id).await
    }

    async fn start_job(&self, id: JobId) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                started_at = COALESCE(started_at, strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
                completed_at = NULL,
                error = NULL,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn succeed_job(&self, id: JobId, summary_json: Option<String>) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                summary_json = ?3,
                error = NULL,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Succeeded.as_str())
        .bind(summary_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn fail_job(&self, id: JobId, error: String) -> Result<Job> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                kind,
                status,
                resource_class,
                library_id,
                source_id,
                input_json,
                summary_json,
                error,
                queued_at,
                started_at,
                completed_at
            FROM jobs
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }
}

#[async_trait::async_trait]
impl CatalogRepository for SqliteStore {
    async fn upsert_person(&self, person: &Person) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO people (id, name, sort_name, overview)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                sort_name = excluded.sort_name,
                overview = excluded.overview,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(person.id.to_string())
        .bind(&person.name)
        .bind(&person.sort_name)
        .bind(&person.overview)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM person_external_ids WHERE person_id = ?1")
            .bind(person.id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        insert_external_ids(
            &mut transaction,
            "person_external_ids",
            "person_id",
            person.id,
            &person.external_ids,
        )
        .await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>> {
        let row = sqlx::query("SELECT id, name, sort_name, overview FROM people WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT p.id, p.name, p.sort_name, p.overview
            FROM people p
            JOIN person_external_ids e ON e.person_id = p.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY p.name ASC, p.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, sort_name, overview
            FROM people
            WHERE name = ?1
            ORDER BY name ASC, id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, sort_name, overview
            FROM people
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut people = Vec::with_capacity(rows.len());

        for row in rows {
            let id: PersonId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("person_external_ids", "person_id", id)
                .await?;
            people.push(row_to_person(row, external_ids)?);
        }

        Ok(people)
    }

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()> {
        let (role, role_key) = credit_role_to_parts(&credit.role);
        sqlx::query(
            r#"
            INSERT INTO item_credits (
                item_id, person_id, role, role_key, character, sort_order
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(item_id, person_id, role, role_key, character) DO UPDATE SET
                sort_order = excluded.sort_order,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(credit.item_id.to_string())
        .bind(credit.person_id.to_string())
        .bind(role)
        .bind(role_key)
        .bind(credit.character.clone().unwrap_or_default())
        .bind(optional_u32_to_i64(credit.sort_order))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_credits WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, person_id, role, role_key, character, sort_order
            FROM item_credits
            WHERE item_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), role ASC, person_id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, person_id, role, role_key, character, sort_order
            FROM item_credits
            WHERE person_id = ?1
            ORDER BY role ASC, COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(person_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_items(
        &self,
        person_id: PersonId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_credits ic ON ic.item_id = mi.id
            WHERE ic.person_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(person_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_genre(&self, genre: &Genre) -> Result<()> {
        let (source, source_key) = metadata_source_to_parts(&genre.source);
        sqlx::query(
            r#"
            INSERT INTO genres (id, name, source, source_key)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(genre.id.to_string())
        .bind(&genre.name)
        .bind(source)
        .bind(source_key)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM genres WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn find_genre_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Genre>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM genres
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_genre).transpose()
    }

    async fn list_genres(&self, page: PageRequest) -> Result<Vec<Genre>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM genres
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_genre).collect()
    }

    async fn upsert_item_genre(&self, item_genre: &ItemGenre) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO item_genres (item_id, genre_id) VALUES (?1, ?2)")
            .bind(item_genre.item_id.to_string())
            .bind(item_genre.genre_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_genres WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        let rows = sqlx::query(
            "SELECT item_id, genre_id FROM item_genres WHERE item_id = ?1 ORDER BY genre_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_genre).collect()
    }

    async fn list_genre_items(
        &self,
        genre_id: GenreId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_genres ig ON ig.item_id = mi.id
            WHERE ig.genre_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(genre_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_tag(&self, tag: &Tag) -> Result<()> {
        let (source, source_key) = metadata_source_to_parts(&tag.source);
        sqlx::query(
            r#"
            INSERT INTO tags (id, name, source, source_key)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(tag.id.to_string())
        .bind(&tag.name)
        .bind(source)
        .bind(source_key)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM tags WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn find_tag_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Tag>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM tags
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_tag).transpose()
    }

    async fn list_tags(&self, page: PageRequest) -> Result<Vec<Tag>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM tags
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_tag).collect()
    }

    async fn upsert_item_tag(&self, item_tag: &ItemTag) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO item_tags (item_id, tag_id) VALUES (?1, ?2)")
            .bind(item_tag.item_id.to_string())
            .bind(item_tag.tag_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_tags WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        let rows = sqlx::query(
            "SELECT item_id, tag_id FROM item_tags WHERE item_id = ?1 ORDER BY tag_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_tag).collect()
    }

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT
                mi.id,
                mi.kind,
                mi.parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json
            FROM media_items mi
            JOIN item_tags it ON it.item_id = mi.id
            WHERE it.tag_id = ?1
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(tag_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_collection(&self, collection: &Collection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let (source, source_key) = metadata_source_to_parts(&collection.source);

        sqlx::query(
            r#"
            INSERT INTO collections (id, name, overview, source, source_key)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                overview = excluded.overview,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(collection.id.to_string())
        .bind(&collection.name)
        .bind(&collection.overview)
        .bind(source)
        .bind(source_key)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM collection_external_ids WHERE collection_id = ?1")
            .bind(collection.id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        insert_external_ids(
            &mut transaction,
            "collection_external_ids",
            "collection_id",
            collection.id,
            &collection.external_ids,
        )
        .await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        let row = sqlx::query(
            "SELECT id, name, overview, source, source_key FROM collections WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_external_id(
        &self,
        external_id: &ExternalId,
    ) -> Result<Option<Collection>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT c.id, c.name, c.overview, c.source, c.source_key
            FROM collections c
            JOIN collection_external_ids e ON e.collection_id = c.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY c.name ASC, c.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn find_collection_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Collection>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, overview, source, source_key
            FROM collections
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, overview, source, source_key
            FROM collections
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut collections = Vec::with_capacity(rows.len());

        for row in rows {
            let id: CollectionId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("collection_external_ids", "collection_id", id)
                .await?;
            collections.push(row_to_collection(row, external_ids)?);
        }

        Ok(collections)
    }

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO collection_items (collection_id, item_id, sort_order)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(collection_id, item_id) DO UPDATE SET
                sort_order = excluded.sort_order
            "#,
        )
        .bind(item.collection_id.to_string())
        .bind(item.item_id.to_string())
        .bind(optional_u32_to_i64(item.sort_order))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM collection_items WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id, item_id, sort_order
            FROM collection_items
            WHERE item_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), collection_id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn list_collection_items(
        &self,
        collection_id: CollectionId,
    ) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id, item_id, sort_order
            FROM collection_items
            WHERE collection_id = ?1
            ORDER BY COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(collection_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn upsert_studio(&self, studio: &Studio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let (source, source_key) = metadata_source_to_parts(&studio.source);

        sqlx::query(
            r#"
            INSERT INTO studios (id, name, source, source_key)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                source = excluded.source,
                source_key = excluded.source_key,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(studio.id.to_string())
        .bind(&studio.name)
        .bind(source)
        .bind(source_key)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM studio_external_ids WHERE studio_id = ?1")
            .bind(studio.id.to_string())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        insert_external_ids(
            &mut transaction,
            "studio_external_ids",
            "studio_id",
            studio.id,
            &studio.external_ids,
        )
        .await?;

        transaction.commit().await.map_err(database_error)
    }

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>> {
        let row = sqlx::query("SELECT id, name, source, source_key FROM studios WHERE id = ?1")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT s.id, s.name, s.source, s.source_key
            FROM studios s
            JOIN studio_external_ids e ON e.studio_id = s.id
            WHERE e.provider = ?1 AND e.provider_key = ?2 AND e.value = ?3
            ORDER BY s.name ASC, s.id ASC
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_name_source(
        &self,
        name: &str,
        source: &MetadataSource,
    ) -> Result<Option<Studio>> {
        let (source, source_key) = metadata_source_to_parts(source);
        let row = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM studios
            WHERE name = ?1 AND source = ?2 AND source_key = ?3
            ORDER BY id ASC
            LIMIT 1
            "#,
        )
        .bind(name)
        .bind(source)
        .bind(source_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self
            .list_entity_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id, name, source, source_key
            FROM studios
            ORDER BY name ASC, id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut studios = Vec::with_capacity(rows.len());

        for row in rows {
            let id: StudioId = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self
                .list_entity_external_ids("studio_external_ids", "studio_id", id)
                .await?;
            studios.push(row_to_studio(row, external_ids)?);
        }

        Ok(studios)
    }

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()> {
        sqlx::query("INSERT OR IGNORE INTO item_studios (item_id, studio_id) VALUES (?1, ?2)")
            .bind(item_studio.item_id.to_string())
            .bind(item_studio.studio_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_studios WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        let rows = sqlx::query(
            "SELECT item_id, studio_id FROM item_studios WHERE item_id = ?1 ORDER BY studio_id ASC",
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_studio).collect()
    }

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()> {
        let (owner_kind, owner_id) = image_owner_to_parts(&image.owner);
        let (kind, kind_key) = image_kind_to_parts(&image.kind);
        let (provider, provider_key) = provider_to_parts(&image.provider);

        sqlx::query(
            r#"
            INSERT INTO image_assets (
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            ON CONFLICT(id) DO UPDATE SET
                owner_kind = excluded.owner_kind,
                owner_id = excluded.owner_id,
                kind = excluded.kind,
                kind_key = excluded.kind_key,
                source_uri = excluded.source_uri,
                provider = excluded.provider,
                provider_key = excluded.provider_key,
                cache_uri = excluded.cache_uri,
                width = excluded.width,
                height = excluded.height,
                language = excluded.language,
                selected = excluded.selected,
                content_hash = excluded.content_hash,
                etag = excluded.etag,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(image.id.to_string())
        .bind(owner_kind)
        .bind(owner_id)
        .bind(kind)
        .bind(kind_key)
        .bind(&image.source_uri)
        .bind(provider)
        .bind(provider_key)
        .bind(&image.cache_uri)
        .bind(optional_u32_to_i64(image.width))
        .bind(optional_u32_to_i64(image.height))
        .bind(&image.language)
        .bind(bool_to_i64(image.selected))
        .bind(&image.content_hash)
        .bind(&image.etag)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn find_image_asset_by_source(
        &self,
        owner: &ImageOwner,
        kind: &ImageKind,
        source_uri: &str,
    ) -> Result<Option<ImageAsset>> {
        let (owner_kind, owner_id) = image_owner_to_parts(owner);
        let (kind, kind_key) = image_kind_to_parts(kind);
        let row = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE owner_kind = ?1 AND owner_id = ?2 AND kind = ?3
                AND kind_key = ?4 AND source_uri = ?5
            LIMIT 1
            "#,
        )
        .bind(owner_kind)
        .bind(owner_id)
        .bind(kind)
        .bind(kind_key)
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_image_asset).transpose()
    }

    async fn list_item_images(&self, item_id: MediaItemId) -> Result<Vec<ImageAsset>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id, owner_kind, owner_id, kind, kind_key, source_uri, provider,
                provider_key, cache_uri, width, height, language, selected,
                content_hash, etag
            FROM image_assets
            WHERE owner_kind = 'item' AND owner_id = ?1
            ORDER BY selected DESC, kind ASC, id ASC
            "#,
        )
        .bind(item_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_image_asset).collect()
    }
}

#[async_trait::async_trait]
impl ScanRepository for SqliteStore {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            INSERT INTO scan_snapshots (id, library_id, root, status)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(id.to_string())
        .bind(library_id.to_string())
        .bind(root)
        .bind(ScanStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn complete_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        status: ScanStatus,
        error: Option<String>,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            UPDATE scan_snapshots
            SET
                status = ?2,
                error = ?3,
                completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_scan_snapshot(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "scan_snapshot",
                id: id.to_string(),
            })
    }

    async fn get_scan_snapshot(&self, id: ScanSnapshotId) -> Result<Option<ScanSnapshot>> {
        let row = sqlx::query(
            r#"
            SELECT id, library_id, root, started_at, completed_at, status, error
            FROM scan_snapshots
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_scan_snapshot).transpose()
    }

    async fn upsert_directory_snapshot(&self, snapshot: &DirectorySnapshot) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO directory_snapshots (
                scan_id, uri, etag, modified_at, child_count
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(scan_id, uri) DO UPDATE SET
                etag = excluded.etag,
                modified_at = excluded.modified_at,
                child_count = excluded.child_count
            "#,
        )
        .bind(snapshot.scan_id.to_string())
        .bind(&snapshot.uri)
        .bind(&snapshot.etag)
        .bind(&snapshot.modified_at)
        .bind(u64_to_i64(snapshot.child_count)?)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_directory_snapshots(
        &self,
        scan_id: ScanSnapshotId,
    ) -> Result<Vec<DirectorySnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT scan_id, uri, etag, modified_at, child_count
            FROM directory_snapshots
            WHERE scan_id = ?1
            ORDER BY uri ASC
            "#,
        )
        .bind(scan_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_directory_snapshot).collect()
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO source_states (
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(library_id, uri) DO UPDATE SET
                source_id = excluded.source_id,
                size_bytes = excluded.size_bytes,
                modified_at = excluded.modified_at,
                etag = excluded.etag,
                fingerprint = excluded.fingerprint,
                last_seen_scan_id = excluded.last_seen_scan_id,
                tombstoned = excluded.tombstoned,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(state.library_id.to_string())
        .bind(state.source_id.map(|id| id.to_string()))
        .bind(&state.uri)
        .bind(optional_u64_to_i64(state.size_bytes)?)
        .bind(&state.modified_at)
        .bind(&state.etag)
        .bind(&state.fingerprint)
        .bind(state.last_seen_scan_id.to_string())
        .bind(bool_to_i64(state.tombstoned))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1 AND uri = ?2
            "#,
        )
        .bind(library_id.to_string())
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_source_state).transpose()
    }

    async fn list_source_states(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<SourceState>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                library_id, source_id, uri, size_bytes, modified_at, etag,
                fingerprint, last_seen_scan_id, tombstoned
            FROM source_states
            WHERE library_id = ?1
            ORDER BY uri ASC
            LIMIT ?2 OFFSET ?3
            "#,
        )
        .bind(library_id.to_string())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_source_state).collect()
    }
}

#[async_trait::async_trait]
impl ArtworkTaskRepository for SqliteStore {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO artwork_tasks (
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                image_id = excluded.image_id,
                kind = excluded.kind,
                status = excluded.status,
                resource_class = excluded.resource_class,
                attempts = excluded.attempts,
                max_attempts = excluded.max_attempts,
                error = excluded.error,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(task.id.to_string())
        .bind(task.image_id.to_string())
        .bind(task.kind.as_str())
        .bind(task.status.as_str())
        .bind(&task.resource_class)
        .bind(u32_to_i64(task.attempts))
        .bind(u32_to_i64(task.max_attempts))
        .bind(&task.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_artwork_task(&self, id: ArtworkTaskId) -> Result<Option<ArtworkTask>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            FROM artwork_tasks
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_task).transpose()
    }

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            FROM artwork_tasks
            ORDER BY id ASC
            LIMIT ?1 OFFSET ?2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_task).collect()
    }
}

#[async_trait::async_trait]
impl SearchIndex for SqliteStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let facets_json = serde_json::to_string(&document.facets).map_err(database_error)?;
        let facets_text = document.facets.join(" ");

        sqlx::query(
            r#"
            INSERT INTO search_documents (
                item_id, title, body, facets_json, facets_text
            )
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(item_id) DO UPDATE SET
                title = excluded.title,
                body = excluded.body,
                facets_json = excluded.facets_json,
                facets_text = excluded.facets_text,
                updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
            "#,
        )
        .bind(document.item_id.to_string())
        .bind(document.title)
        .bind(document.body)
        .bind(facets_json)
        .bind(facets_text)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn delete(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM search_documents WHERE item_id = ?1")
            .bind(item_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id, title, body, facets_json, facets_text
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let needle = query.query.trim().to_lowercase();
        let required_facets = query
            .facets
            .iter()
            .map(|facet| facet.to_lowercase())
            .collect::<Vec<_>>();
        let offset = query.offset as usize;
        let limit = if query.limit == 0 {
            PageRequest::DEFAULT_LIMIT as usize
        } else {
            query.limit.min(PageRequest::MAX_LIMIT) as usize
        };

        let mut hits = Vec::new();

        for row in rows {
            let title: String = row_get(&row, "title")?;
            let body: String = row_get(&row, "body")?;
            let facets_text: String = row_get(&row, "facets_text")?;
            let haystack = format!("{title} {body} {facets_text}").to_lowercase();

            if !needle.is_empty() && !haystack.contains(&needle) {
                continue;
            }

            let facet_haystack = facets_text.to_lowercase();
            if required_facets
                .iter()
                .any(|facet| !facet_haystack.contains(facet))
            {
                continue;
            }

            let score = if !needle.is_empty() && title.to_lowercase().contains(&needle) {
                1.0
            } else if !needle.is_empty() && body.to_lowercase().contains(&needle) {
                0.7
            } else {
                0.5
            };

            hits.push(SearchHit {
                item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
                score,
            });
        }

        hits.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.item_id.cmp(&right.item_id))
        });

        Ok(hits.into_iter().skip(offset).take(limit).collect())
    }
}

impl SqliteStore {
    async fn get_job_or_not_found(&self, id: JobId) -> Result<Job> {
        self.get_job(id).await?.ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
    }

    async fn rows_to_media_items(&self, rows: Vec<SqliteRow>) -> Result<Vec<MediaItem>> {
        let mut items = Vec::with_capacity(rows.len());

        for row in rows {
            let id = parse_id(row_get::<String>(&row, "id")?)?;
            let external_ids = self.list_external_ids(id).await?;
            items.push(row_to_media_item(row, external_ids)?);
        }

        Ok(items)
    }

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

    async fn list_entity_external_ids<T>(
        &self,
        table: &str,
        owner_column: &str,
        owner_id: T,
    ) -> Result<Vec<ExternalId>>
    where
        T: Display,
    {
        let query = format!(
            "SELECT provider, provider_key, value FROM {table} WHERE {owner_column} = ?1 ORDER BY provider ASC, provider_key ASC, value ASC"
        );
        let rows = sqlx::query(&query)
            .bind(owner_id.to_string())
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter()
            .map(|row| {
                Ok(ExternalId {
                    provider: provider_from_parts(
                        row_get(&row, "provider")?,
                        row_get(&row, "provider_key")?,
                    ),
                    value: row_get(&row, "value")?,
                })
            })
            .collect()
    }
}

async fn insert_external_ids<T>(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    table: &str,
    owner_column: &str,
    owner_id: T,
    external_ids: &[ExternalId],
) -> Result<()>
where
    T: Display + Copy,
{
    let query = format!(
        "INSERT INTO {table} ({owner_column}, provider, provider_key, value) VALUES (?1, ?2, ?3, ?4)"
    );

    for external_id in external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(&query)
            .bind(owner_id.to_string())
            .bind(provider)
            .bind(provider_key)
            .bind(&external_id.value)
            .execute(&mut **transaction)
            .await
            .map_err(database_error)?;
    }

    Ok(())
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

fn metadata_source_to_parts(source: &MetadataSource) -> (String, String) {
    match source {
        MetadataSource::Local => ("local".to_owned(), String::new()),
        MetadataSource::Nfo => ("nfo".to_owned(), String::new()),
        MetadataSource::User => ("user".to_owned(), String::new()),
        MetadataSource::Provider(provider) => {
            let (provider, provider_key) = provider_to_parts(provider);
            (format!("provider:{provider}"), provider_key)
        }
    }
}

fn metadata_source_from_parts(source: String, source_key: String) -> MetadataSource {
    match source.as_str() {
        "local" => MetadataSource::Local,
        "nfo" => MetadataSource::Nfo,
        "user" => MetadataSource::User,
        value if value.starts_with("provider:") => {
            let provider = value.trim_start_matches("provider:").to_owned();
            MetadataSource::Provider(provider_from_parts(provider, source_key))
        }
        _ => MetadataSource::Provider(ExternalProvider::Other(source)),
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

fn credit_role_to_parts(role: &CreditRole) -> (String, String) {
    match role {
        CreditRole::Actor => ("actor".to_owned(), String::new()),
        CreditRole::Director => ("director".to_owned(), String::new()),
        CreditRole::Writer => ("writer".to_owned(), String::new()),
        CreditRole::Producer => ("producer".to_owned(), String::new()),
        CreditRole::Creator => ("creator".to_owned(), String::new()),
        CreditRole::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn credit_role_from_parts(role: String, role_key: String) -> CreditRole {
    match role.as_str() {
        "actor" => CreditRole::Actor,
        "director" => CreditRole::Director,
        "writer" => CreditRole::Writer,
        "producer" => CreditRole::Producer,
        "creator" => CreditRole::Creator,
        "other" => CreditRole::Other(role_key),
        _ => CreditRole::Other(role),
    }
}

fn image_kind_to_parts(kind: &ImageKind) -> (String, String) {
    match kind {
        ImageKind::Poster => ("poster".to_owned(), String::new()),
        ImageKind::Backdrop => ("backdrop".to_owned(), String::new()),
        ImageKind::Logo => ("logo".to_owned(), String::new()),
        ImageKind::Thumbnail => ("thumbnail".to_owned(), String::new()),
        ImageKind::Banner => ("banner".to_owned(), String::new()),
        ImageKind::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn image_kind_from_parts(kind: String, kind_key: String) -> ImageKind {
    match kind.as_str() {
        "poster" => ImageKind::Poster,
        "backdrop" => ImageKind::Backdrop,
        "logo" => ImageKind::Logo,
        "thumbnail" => ImageKind::Thumbnail,
        "banner" => ImageKind::Banner,
        "other" => ImageKind::Other(kind_key),
        _ => ImageKind::Other(kind),
    }
}

fn image_owner_to_parts(owner: &ImageOwner) -> (String, String) {
    match owner {
        ImageOwner::Item(id) => ("item".to_owned(), id.to_string()),
        ImageOwner::Person(id) => ("person".to_owned(), id.to_string()),
        ImageOwner::Collection(id) => ("collection".to_owned(), id.to_string()),
        ImageOwner::Studio(id) => ("studio".to_owned(), id.to_string()),
    }
}

fn image_owner_from_parts(owner_kind: String, owner_id: String) -> Result<ImageOwner> {
    match owner_kind.as_str() {
        "item" => Ok(ImageOwner::Item(parse_id(owner_id)?)),
        "person" => Ok(ImageOwner::Person(parse_id(owner_id)?)),
        "collection" => Ok(ImageOwner::Collection(parse_id(owner_id)?)),
        "studio" => Ok(ImageOwner::Studio(parse_id(owner_id)?)),
        _ => Err(TaruError::Database {
            message: format!("unknown image owner kind stored in database: {owner_kind}"),
        }),
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

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn i64_to_bool(value: i64) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(TaruError::Database {
            message: format!("SQLite integer cannot be converted to bool: {value}"),
        }),
    }
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

fn u64_to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|err| TaruError::Database {
        message: format!("value does not fit into SQLite integer: {err}"),
    })
}

fn i64_to_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|err| TaruError::Database {
        message: format!("SQLite integer cannot be converted to u32: {err}"),
    })
}

fn row_to_library(row: SqliteRow) -> Result<Library> {
    let roots_json = row_get::<String>(&row, "roots_json")?;
    let roots = serde_json::from_str(&roots_json).map_err(database_error)?;
    let options = row_to_library_options(&row)?;

    Ok(Library {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        roots,
        options,
    })
}

fn row_to_library_options(row: &SqliteRow) -> Result<LibraryOptions> {
    if let Some(options_json) = row_get::<Option<String>>(row, "options_json")? {
        return serde_json::from_str(&options_json).map_err(database_error);
    }

    let domain = row_get::<String>(row, "domain")?;
    let preset = row_get::<String>(row, "preset")?;
    let preset = taru_core::LibraryPreset::parse(&preset).ok_or_else(|| TaruError::Database {
        message: format!("unknown library preset stored in database: {preset}"),
    })?;
    let mut options = LibraryOptions::from_preset(preset);
    options.domain = MediaDomain::parse(&domain).ok_or_else(|| TaruError::Database {
        message: format!("unknown media domain stored in database: {domain}"),
    })?;

    Ok(options)
}

fn row_to_media_item(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<MediaItem> {
    let metadata_json = row_get::<Option<String>>(&row, "metadata_json")?;
    let mut metadata = match metadata_json {
        Some(value) => serde_json::from_str::<CanonicalMetadata>(&value).map_err(database_error)?,
        None => CanonicalMetadata {
            title: row_get(&row, "title")?,
            original_title: row_get(&row, "original_title")?,
            sort_title: row_get(&row, "sort_title")?,
            overview: row_get(&row, "overview")?,
            release_date: row_get(&row, "release_date")?,
            ..CanonicalMetadata::default()
        },
    };
    metadata.external_ids = external_ids;

    Ok(MediaItem {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        kind: parse_media_kind(row_get::<String>(&row, "kind")?)?,
        parent_id: parse_optional_id(row_get::<Option<String>>(&row, "parent_id")?)?,
        metadata,
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

fn row_to_job(row: SqliteRow) -> Result<Job> {
    Ok(Job {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        kind: JobKind::parse(&row_get::<String>(&row, "kind")?)?,
        status: JobStatus::parse(&row_get::<String>(&row, "status")?)?,
        resource_class: row_get(&row, "resource_class")?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        input_json: row_get(&row, "input_json")?,
        summary_json: row_get(&row, "summary_json")?,
        error: row_get(&row, "error")?,
        queued_at: row_get(&row, "queued_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    })
}

fn row_to_metadata_field_lock(row: SqliteRow) -> Result<MetadataFieldLock> {
    Ok(MetadataFieldLock {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        field: metadata_field_from_str(&row_get::<String>(&row, "field")?)?,
        locked: i64_to_bool(row_get(&row, "locked")?)?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_provider_raw_response(row: SqliteRow) -> Result<ProviderRawResponse> {
    Ok(ProviderRawResponse {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        provider_key: row_get(&row, "provider_key")?,
        body_json: row_get(&row, "body_json")?,
        fetched_at: row_get(&row, "fetched_at")?,
    })
}

fn row_to_person(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<Person> {
    Ok(Person {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        sort_name: row_get(&row, "sort_name")?,
        overview: row_get(&row, "overview")?,
        external_ids,
    })
}

fn row_to_item_credit(row: SqliteRow) -> Result<ItemCredit> {
    let character = row_get::<String>(&row, "character")?;

    Ok(ItemCredit {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        person_id: parse_id(row_get::<String>(&row, "person_id")?)?,
        role: credit_role_from_parts(row_get(&row, "role")?, row_get(&row, "role_key")?),
        character: (!character.is_empty()).then_some(character),
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_genre(row: SqliteRow) -> Result<Genre> {
    Ok(Genre {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_genre(row: SqliteRow) -> Result<ItemGenre> {
    Ok(ItemGenre {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        genre_id: parse_id(row_get::<String>(&row, "genre_id")?)?,
    })
}

fn row_to_tag(row: SqliteRow) -> Result<Tag> {
    Ok(Tag {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_tag(row: SqliteRow) -> Result<ItemTag> {
    Ok(ItemTag {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        tag_id: parse_id(row_get::<String>(&row, "tag_id")?)?,
    })
}

fn row_to_collection(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<Collection> {
    Ok(Collection {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        overview: row_get(&row, "overview")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_collection_item(row: SqliteRow) -> Result<CollectionItem> {
    Ok(CollectionItem {
        collection_id: parse_id(row_get::<String>(&row, "collection_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_studio(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<Studio> {
    Ok(Studio {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_item_studio(row: SqliteRow) -> Result<ItemStudio> {
    Ok(ItemStudio {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        studio_id: parse_id(row_get::<String>(&row, "studio_id")?)?,
    })
}

fn row_to_image_asset(row: SqliteRow) -> Result<ImageAsset> {
    Ok(ImageAsset {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        owner: image_owner_from_parts(row_get(&row, "owner_kind")?, row_get(&row, "owner_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        source_uri: row_get(&row, "source_uri")?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        cache_uri: row_get(&row, "cache_uri")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        language: row_get(&row, "language")?,
        selected: i64_to_bool(row_get(&row, "selected")?)?,
        content_hash: row_get(&row, "content_hash")?,
        etag: row_get(&row, "etag")?,
    })
}

fn row_to_scan_snapshot(row: SqliteRow) -> Result<ScanSnapshot> {
    Ok(ScanSnapshot {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        root: row_get(&row, "root")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
        status: ScanStatus::parse(&row_get::<String>(&row, "status")?)?,
        error: row_get(&row, "error")?,
    })
}

fn row_to_directory_snapshot(row: SqliteRow) -> Result<DirectorySnapshot> {
    Ok(DirectorySnapshot {
        scan_id: parse_id(row_get::<String>(&row, "scan_id")?)?,
        uri: row_get(&row, "uri")?,
        etag: row_get(&row, "etag")?,
        modified_at: row_get(&row, "modified_at")?,
        child_count: optional_i64_to_u64(Some(row_get(&row, "child_count")?))?.unwrap_or_default(),
    })
}

fn row_to_source_state(row: SqliteRow) -> Result<SourceState> {
    Ok(SourceState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        uri: row_get(&row, "uri")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        modified_at: row_get(&row, "modified_at")?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        last_seen_scan_id: parse_id(row_get::<String>(&row, "last_seen_scan_id")?)?,
        tombstoned: i64_to_bool(row_get(&row, "tombstoned")?)?,
    })
}

fn row_to_artwork_task(row: SqliteRow) -> Result<ArtworkTask> {
    Ok(ArtworkTask {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        image_id: parse_id(row_get::<String>(&row, "image_id")?)?,
        kind: ArtworkTaskKind::parse(&row_get::<String>(&row, "kind")?)?,
        status: JobStatus::parse(&row_get::<String>(&row, "status")?)?,
        resource_class: row_get(&row, "resource_class")?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        max_attempts: i64_to_u32(row_get(&row, "max_attempts")?)?,
        error: row_get(&row, "error")?,
    })
}

fn serialize_metadata_json(metadata: &CanonicalMetadata) -> Result<String> {
    serde_json::to_string(metadata).map_err(database_error)
}

fn metadata_field_from_str(value: &str) -> Result<MetadataField> {
    match value {
        "title" => Ok(MetadataField::Title),
        "original_title" => Ok(MetadataField::OriginalTitle),
        "sort_title" => Ok(MetadataField::SortTitle),
        "overview" => Ok(MetadataField::Overview),
        "release_date" => Ok(MetadataField::ReleaseDate),
        "runtime_minutes" => Ok(MetadataField::RuntimeMinutes),
        "tagline" => Ok(MetadataField::Tagline),
        "genres" => Ok(MetadataField::Genres),
        "tags" => Ok(MetadataField::Tags),
        "ratings" => Ok(MetadataField::Ratings),
        "images" => Ok(MetadataField::Images),
        "credits" => Ok(MetadataField::Credits),
        "collections" => Ok(MetadataField::Collections),
        "studios" => Ok(MetadataField::Studios),
        "external_ids" => Ok(MetadataField::ExternalIds),
        _ => Err(TaruError::Database {
            message: format!("unknown metadata field stored in database: {value}"),
        }),
    }
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
    use taru_core::{
        ContentRating, Credit, CreditRole, ImageKind, ImageOwner, ImageRef, LibraryOptions,
        LibraryPreset, MediaSourceId, MetadataRefreshMode,
    };

    use super::*;

    #[tokio::test]
    async fn sqlite_store_persists_libraries() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };

        store.upsert_library(&library).await.unwrap();
        let loaded = store.get_library(library.id).await.unwrap();

        assert_eq!(loaded, Some(library));
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_library_profiles() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let mut options = LibraryOptions::from_preset(LibraryPreset::Anime);
        options.metadata_profile.refresh_mode = MetadataRefreshMode::MissingOnly;
        options.metadata_profile.metadata_providers = vec![
            ExternalProvider::Bangumi,
            ExternalProvider::Tmdb,
            ExternalProvider::Douban,
        ];
        let library = Library {
            id: LibraryId::new(),
            name: "Anime".to_owned(),
            roots: vec!["local:///Anime".to_owned()],
            options,
        };

        store.upsert_library(&library).await.unwrap();

        assert_eq!(store.get_library(library.id).await.unwrap(), Some(library));
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_media_items_and_sources() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
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
                runtime_minutes: Some(136),
                tagline: Some("Welcome to the Real World".to_owned()),
                genres: vec!["Action".to_owned(), "Science Fiction".to_owned()],
                tags: vec!["cyberpunk".to_owned()],
                ratings: vec![ContentRating {
                    source: "MPAA".to_owned(),
                    value: "R".to_owned(),
                }],
                images: vec![ImageRef {
                    kind: ImageKind::Poster,
                    uri: "https://image.example/poster.jpg".to_owned(),
                    provider: ExternalProvider::Tmdb,
                    width: Some(1000),
                    height: Some(1500),
                    language: Some("en".to_owned()),
                }],
                credits: vec![Credit {
                    name: "Keanu Reeves".to_owned(),
                    role: CreditRole::Actor,
                    character: Some("Neo".to_owned()),
                    order: Some(0),
                    external_ids: Vec::new(),
                }],
                collections: Vec::new(),
                studios: Vec::new(),
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
            store
                .list_media_sources(library.id, PageRequest::first_page())
                .await
                .unwrap(),
            vec![source]
        );
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_metadata_policy_records() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Policy Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let lock = MetadataFieldLock {
            item_id: item.id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::User,
        };
        let raw = ProviderRawResponse {
            item_id: item.id,
            provider: ExternalProvider::Tmdb,
            provider_key: "603".to_owned(),
            fetched_at: "2026-05-14T00:00:00.000Z".to_owned(),
            body_json: r#"{"id":603,"title":"The Matrix"}"#.to_owned(),
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_field_lock(&lock).await.unwrap();
        store.upsert_provider_raw_response(&raw).await.unwrap();

        assert_eq!(store.list_field_locks(item.id).await.unwrap(), vec![lock]);
        assert_eq!(
            store
                .get_provider_raw_response(item.id, &ExternalProvider::Tmdb, "603")
                .await
                .unwrap(),
            Some(raw)
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
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
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

    #[tokio::test]
    async fn sqlite_store_round_trips_catalog_graph_records() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Graph Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let person = Person {
            id: PersonId::new(),
            name: "Keanu Reeves".to_owned(),
            sort_name: Some("Reeves, Keanu".to_owned()),
            overview: Some("Actor".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "6384".to_owned(),
            }],
        };
        let credit = ItemCredit {
            item_id: item.id,
            person_id: person.id,
            role: CreditRole::Actor,
            character: Some("Neo".to_owned()),
            sort_order: Some(0),
        };
        let genre = Genre {
            id: GenreId::new(),
            name: "Science Fiction".to_owned(),
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
        };
        let tag = Tag {
            id: TagId::new(),
            name: "Watchlist".to_owned(),
            source: MetadataSource::User,
        };
        let collection = Collection {
            id: CollectionId::new(),
            name: "Matrix Collection".to_owned(),
            overview: Some("Franchise".to_owned()),
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "2344".to_owned(),
            }],
        };
        let studio = Studio {
            id: StudioId::new(),
            name: "Warner Bros.".to_owned(),
            source: MetadataSource::Provider(ExternalProvider::Tmdb),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "174".to_owned(),
            }],
        };
        let image = ImageAsset {
            id: ImageAssetId::new(),
            owner: ImageOwner::Item(item.id),
            kind: ImageKind::Poster,
            source_uri: "https://image.example/poster.jpg".to_owned(),
            provider: ExternalProvider::Tmdb,
            cache_uri: Some("local:///cache/poster.webp".to_owned()),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
            selected: true,
            content_hash: Some("hash".to_owned()),
            etag: Some("etag".to_owned()),
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_person(&person).await.unwrap();
        store.upsert_item_credit(&credit).await.unwrap();
        store.upsert_genre(&genre).await.unwrap();
        store
            .upsert_item_genre(&ItemGenre {
                item_id: item.id,
                genre_id: genre.id,
            })
            .await
            .unwrap();
        store.upsert_tag(&tag).await.unwrap();
        store
            .upsert_item_tag(&ItemTag {
                item_id: item.id,
                tag_id: tag.id,
            })
            .await
            .unwrap();
        store.upsert_collection(&collection).await.unwrap();
        store
            .upsert_collection_item(&CollectionItem {
                collection_id: collection.id,
                item_id: item.id,
                sort_order: Some(1),
            })
            .await
            .unwrap();
        store.upsert_studio(&studio).await.unwrap();
        store
            .upsert_item_studio(&ItemStudio {
                item_id: item.id,
                studio_id: studio.id,
            })
            .await
            .unwrap();
        store.upsert_image_asset(&image).await.unwrap();

        assert_eq!(store.get_person(person.id).await.unwrap(), Some(person));
        assert_eq!(
            store.list_item_credits(item.id).await.unwrap(),
            vec![credit]
        );
        assert_eq!(store.get_genre(genre.id).await.unwrap(), Some(genre));
        assert_eq!(store.list_item_genres(item.id).await.unwrap().len(), 1);
        assert_eq!(store.get_tag(tag.id).await.unwrap(), Some(tag));
        assert_eq!(store.list_item_tags(item.id).await.unwrap().len(), 1);
        assert_eq!(
            store.get_collection(collection.id).await.unwrap(),
            Some(collection.clone())
        );
        assert_eq!(
            store.list_collection_items(collection.id).await.unwrap(),
            vec![CollectionItem {
                collection_id: collection.id,
                item_id: item.id,
                sort_order: Some(1)
            }]
        );
        assert_eq!(store.get_studio(studio.id).await.unwrap(), Some(studio));
        assert_eq!(store.list_item_studios(item.id).await.unwrap().len(), 1);
        assert_eq!(store.get_image_asset(image.id).await.unwrap(), Some(image));
        assert_eq!(store.list_item_images(item.id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_scan_state_search_and_artwork_tasks() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        let item = MediaItem {
            id: MediaItemId::new(),
            kind: MediaKind::Movie,
            parent_id: None,
            metadata: CanonicalMetadata {
                title: "Searchable Demo".to_owned(),
                overview: Some("A searchable graph fixture.".to_owned()),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            item_id: item.id,
            locator: "local:///Movies/Searchable Demo.mkv".to_owned(),
            file_name: "Searchable Demo.mkv".to_owned(),
            size_bytes: Some(10),
            fingerprint: Some("fingerprint".to_owned()),
        };
        let scan_id = ScanSnapshotId::new();
        let image = ImageAsset {
            id: ImageAssetId::new(),
            owner: ImageOwner::Item(item.id),
            kind: ImageKind::Thumbnail,
            source_uri: "local:///Movies/Searchable Demo.mkv#preview=10".to_owned(),
            provider: ExternalProvider::Local,
            cache_uri: None,
            width: Some(320),
            height: Some(180),
            language: None,
            selected: false,
            content_hash: None,
            etag: None,
        };
        let task = ArtworkTask {
            id: ArtworkTaskId::new(),
            image_id: image.id,
            kind: ArtworkTaskKind::Preview,
            status: JobStatus::Queued,
            resource_class: ArtworkTaskKind::Preview.resource_class().to_owned(),
            attempts: 0,
            max_attempts: 3,
            error: None,
        };

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store
            .upsert_media_source(library.id, &source)
            .await
            .unwrap();
        let running = store
            .begin_scan_snapshot(scan_id, library.id, "local:///Movies")
            .await
            .unwrap();
        store
            .upsert_directory_snapshot(&DirectorySnapshot {
                scan_id,
                uri: "local:///Movies".to_owned(),
                etag: Some("dir-etag".to_owned()),
                modified_at: Some("1".to_owned()),
                child_count: 1,
            })
            .await
            .unwrap();
        store
            .upsert_source_state(&SourceState {
                library_id: library.id,
                source_id: Some(source.id),
                uri: source.locator.clone(),
                size_bytes: source.size_bytes,
                modified_at: Some("1".to_owned()),
                etag: None,
                fingerprint: source.fingerprint.clone(),
                last_seen_scan_id: scan_id,
                tombstoned: false,
            })
            .await
            .unwrap();
        let completed = store
            .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
            .await
            .unwrap();
        let failed_scan_id = ScanSnapshotId::new();
        store
            .begin_scan_snapshot(failed_scan_id, library.id, "local:///Broken")
            .await
            .unwrap();
        let failed = store
            .complete_scan_snapshot(
                failed_scan_id,
                ScanStatus::Failed,
                Some("scan failed".to_owned()),
            )
            .await
            .unwrap();
        store
            .upsert(SearchDocument {
                item_id: item.id,
                title: item.metadata.title.clone(),
                body: item.metadata.overview.clone().unwrap(),
                facets: vec!["genre:sci-fi".to_owned()],
            })
            .await
            .unwrap();
        store.upsert_image_asset(&image).await.unwrap();
        store.enqueue_artwork_task(&task).await.unwrap();

        assert_eq!(running.status, ScanStatus::Running);
        assert_eq!(completed.status, ScanStatus::Succeeded);
        assert!(completed.completed_at.is_some());
        assert_eq!(failed.status, ScanStatus::Failed);
        assert_eq!(failed.error, Some("scan failed".to_owned()));
        assert_eq!(
            store.list_directory_snapshots(scan_id).await.unwrap().len(),
            1
        );
        assert_eq!(
            store
                .get_source_state(library.id, &source.locator)
                .await
                .unwrap()
                .unwrap()
                .fingerprint,
            Some("fingerprint".to_owned())
        );
        assert_eq!(
            store
                .search(SearchQuery {
                    query: "searchable".to_owned(),
                    facets: Vec::new(),
                    limit: 10,
                    offset: 0,
                })
                .await
                .unwrap()[0]
                .item_id,
            item.id
        );
        assert_eq!(store.get_artwork_task(task.id).await.unwrap(), Some(task));
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_job_lifecycle() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();

        let id = JobId::new();
        let queued = store
            .enqueue_job(NewJob {
                id,
                kind: JobKind::LibraryScan,
                resource_class: "disk.scan".to_owned(),
                library_id: Some(library.id),
                source_id: None,
                input_json: Some(r#"{"library_id":"demo"}"#.to_owned()),
            })
            .await
            .unwrap();
        let running = store.start_job(id).await.unwrap();
        let succeeded = store
            .succeed_job(id, Some(r#"{"discovered_files":1}"#.to_owned()))
            .await
            .unwrap();

        assert_eq!(queued.status, JobStatus::Queued);
        assert_eq!(
            queued.input_json,
            Some(r#"{"library_id":"demo"}"#.to_owned())
        );
        assert_eq!(running.status, JobStatus::Running);
        assert!(running.started_at.is_some());
        assert_eq!(succeeded.status, JobStatus::Succeeded);
        assert_eq!(
            succeeded.summary_json,
            Some(r#"{"discovered_files":1}"#.to_owned())
        );
        assert!(succeeded.completed_at.is_some());
        assert_eq!(store.get_job(id).await.unwrap(), Some(succeeded));

        let failed_id = JobId::new();
        store
            .enqueue_job(NewJob {
                id: failed_id,
                kind: JobKind::LibraryProbe,
                resource_class: "media.probe".to_owned(),
                library_id: Some(library.id),
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
        store.start_job(failed_id).await.unwrap();
        let failed = store
            .fail_job(failed_id, "probe failed".to_owned())
            .await
            .unwrap();

        assert_eq!(failed.status, JobStatus::Failed);
        assert_eq!(failed.error, Some("probe failed".to_owned()));
        assert!(failed.completed_at.is_some());
    }

    fn external_id_sort_key(external_id: &ExternalId) -> String {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        format!("{provider}\0{provider_key}\0{}", external_id.value)
    }
}
