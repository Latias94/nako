use std::{fmt::Display, path::PathBuf, str::FromStr};

use sqlx::{
    Decode, Row, Sqlite, SqlitePool, Type,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions, SqliteRow},
};
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonRepository, AddonStatus, ArtworkTask, ArtworkTaskId,
    ArtworkTaskKind, ArtworkTaskRepository, AutomationArtifactId, AutomationArtifactKind,
    AutomationArtifactRecord, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderConfigRecord, AutomationProviderId, AutomationProviderStatus,
    AutomationRepository, CanonicalMetadata, CatalogRepository, Collection, CollectionId,
    CollectionItem, CreditRole, DirectorySnapshot, DomainEventKind, DomainEventSubject, EventId,
    EventOutboxRepository, ExternalId, ExternalProvider, Genre, GenreId, ImageAsset, ImageAssetId,
    ImageKind, ImageOwner, ItemCredit, ItemGenre, ItemStudio, ItemTag, Job, JobId, JobKind,
    JobRepository, JobStatus, Library, LibraryId, LibraryOptions, LibraryRepository, MediaDomain,
    MediaItem, MediaItemId, MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository,
    MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind, MetadataField, MetadataFieldLock,
    MetadataMatchKind, MetadataProviderAttemptRecord, MetadataProviderAttemptStatus,
    MetadataProviderErrorClass, MetadataRepository, MetadataSource, NewAddonRegistration,
    NewAutomationArtifact, NewAutomationProviderConfig, NewJob, NewMetadataProviderAttempt,
    NewOutboxEvent, NewTranscodeSession, NewVfsCacheFailure, NewWebhookDeliveryAttempt,
    NewWebhookEndpoint, OutboxEventRecord, OutboxEventStatus, PageRequest, Person, PersonId,
    ProviderRawResponse, Result, ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus,
    SourceState, Studio, StudioId, Tag, TagId, TaruError, TransactionManager,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionRepository, TranscodeSessionState, VfsCacheFailure, VfsCacheOperation,
    VfsCacheRepository, VfsCachedListing, VfsCachedObject, VfsCachedObjectKind,
    WebhookDeliveryAttemptId, WebhookDeliveryAttemptRecord, WebhookDeliveryStatus,
    WebhookEndpointId, WebhookEndpointRecord, WebhookEndpointStatus, WebhookRepository,
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
    (
        "0008_transcode_sessions",
        include_str!("../migrations/0008_transcode_sessions.sql"),
    ),
    (
        "0009_event_outbox",
        include_str!("../migrations/0009_event_outbox.sql"),
    ),
    (
        "0010_webhooks",
        include_str!("../migrations/0010_webhooks.sql"),
    ),
    (
        "0011_automation",
        include_str!("../migrations/0011_automation.sql"),
    ),
    ("0012_addons", include_str!("../migrations/0012_addons.sql")),
    (
        "0013_vfs_cache",
        include_str!("../migrations/0013_vfs_cache.sql"),
    ),
    (
        "0014_staging_manifest",
        include_str!("../migrations/0014_staging_manifest.sql"),
    ),
    (
        "0015_media_source_library_locator",
        include_str!("../migrations/0015_media_source_library_locator.sql"),
    ),
    (
        "0016_metadata_provider_attempts",
        include_str!("../migrations/0016_metadata_provider_attempts.sql"),
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

mod artwork;
mod catalog;
mod jobs;
mod library;
mod media;
mod metadata;
mod playback;
mod scan;
mod search;
mod staging;
mod vfs_cache;

impl SqliteStore {
    async fn get_job_or_not_found(&self, id: JobId) -> Result<Job> {
        self.get_job(id).await?.ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
    }

    async fn get_transcode_session_or_not_found(
        &self,
        id: TranscodeSessionId,
    ) -> Result<TranscodeSessionRecord> {
        self.get_transcode_session(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "transcode_session",
                id: id.to_string(),
            })
    }

    async fn get_webhook_delivery_attempt_or_not_found(
        &self,
        id: WebhookDeliveryAttemptId,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status,
                http_status,
                error,
                requested_at,
                completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_webhook_delivery_attempt)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
                entity: "webhook_delivery_attempt",
                id: id.to_string(),
            })
    }

    async fn get_automation_artifact_or_not_found(
        &self,
        id: AutomationArtifactId,
    ) -> Result<AutomationArtifactRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status,
                created_at,
                updated_at,
                accepted_at
            FROM automation_artifacts
            WHERE id = ?1
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_automation_artifact)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
                entity: "automation_artifact",
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

async fn upsert_vfs_cache_object_tx(
    transaction: &mut sqlx::Transaction<'_, Sqlite>,
    object: &VfsCachedObject,
) -> Result<()> {
    sqlx::query(vfs_cache_object_upsert_sql())
        .bind(&object.uri)
        .bind(&object.scheme)
        .bind(object.kind.as_str())
        .bind(optional_u64_to_i64(object.len)?)
        .bind(&object.modified_at)
        .bind(&object.etag)
        .bind(&object.fingerprint)
        .bind(u32_to_i64(object.capabilities_bits))
        .bind(object.fetched_at_ms)
        .bind(object.fresh_until_ms)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    Ok(())
}

fn vfs_cache_object_upsert_sql() -> &'static str {
    r#"
    INSERT INTO vfs_cache_objects (
        uri, scheme, kind, len, modified_at, etag, fingerprint,
        capabilities_bits, fetched_at_ms, fresh_until_ms
    )
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
    ON CONFLICT(uri) DO UPDATE SET
        scheme = excluded.scheme,
        kind = excluded.kind,
        len = excluded.len,
        modified_at = excluded.modified_at,
        etag = excluded.etag,
        fingerprint = excluded.fingerprint,
        capabilities_bits = excluded.capabilities_bits,
        fetched_at_ms = excluded.fetched_at_ms,
        fresh_until_ms = excluded.fresh_until_ms,
        updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    "#
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

fn parse_transcode_session_kind(value: String) -> Result<TranscodeSessionKind> {
    TranscodeSessionKind::parse(&value).ok_or_else(|| TaruError::Database {
        message: format!("unknown transcode session kind stored in database: {value}"),
    })
}

fn parse_transcode_session_state(value: String) -> Result<TranscodeSessionState> {
    TranscodeSessionState::parse(&value).ok_or_else(|| TaruError::Database {
        message: format!("unknown transcode session state stored in database: {value}"),
    })
}

fn parse_transcode_failure_category(
    value: Option<String>,
) -> Result<Option<TranscodeFailureCategory>> {
    value
        .map(|value| {
            TranscodeFailureCategory::parse(&value).ok_or_else(|| TaruError::Database {
                message: format!("unknown transcode failure category stored in database: {value}"),
            })
        })
        .transpose()
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

fn i64_to_u64(value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|err| TaruError::Database {
        message: format!("negative SQLite integer cannot be converted to u64: {err}"),
    })
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

fn optional_i64_to_u16(value: Option<i64>) -> Result<Option<u16>> {
    value
        .map(|value| {
            u16::try_from(value).map_err(|err| TaruError::Database {
                message: format!("SQLite integer cannot be converted to u16: {err}"),
            })
        })
        .transpose()
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
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
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

fn row_to_outbox_event(row: SqliteRow) -> Result<OutboxEventRecord> {
    Ok(OutboxEventRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        kind: DomainEventKind::parse(&row_get::<String>(&row, "kind")?)?,
        subject: event_subject_from_parts(
            row_get::<String>(&row, "subject_kind")?,
            row_get::<String>(&row, "subject_id")?,
        )?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        idempotency_key: row_get(&row, "idempotency_key")?,
        payload_json: row_get(&row, "payload_json")?,
        status: OutboxEventStatus::parse(&row_get::<String>(&row, "status")?)?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        last_error: row_get(&row, "last_error")?,
        occurred_at: row_get(&row, "occurred_at")?,
        updated_at: row_get(&row, "updated_at")?,
        next_attempt_at: row_get(&row, "next_attempt_at")?,
    })
}

fn row_to_automation_provider(row: SqliteRow) -> Result<AutomationProviderConfigRecord> {
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

fn row_to_automation_artifact(row: SqliteRow) -> Result<AutomationArtifactRecord> {
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

fn row_to_webhook_endpoint(row: SqliteRow) -> Result<WebhookEndpointRecord> {
    let subscribed_event_kinds =
        serde_json::from_str(&row_get::<String>(&row, "subscribed_event_kinds_json")?)
            .map_err(database_error)?;

    Ok(WebhookEndpointRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        url: row_get(&row, "url")?,
        secret_env: row_get(&row, "secret_env")?,
        subscribed_event_kinds,
        timeout_ms: i64_to_u64(row_get(&row, "timeout_ms")?)?,
        max_attempts: i64_to_u32(row_get(&row, "max_attempts")?)?,
        status: WebhookEndpointStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_webhook_delivery_attempt(row: SqliteRow) -> Result<WebhookDeliveryAttemptRecord> {
    Ok(WebhookDeliveryAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        endpoint_id: parse_id(row_get::<String>(&row, "endpoint_id")?)?,
        event_id: parse_id(row_get::<String>(&row, "event_id")?)?,
        attempt_number: i64_to_u32(row_get(&row, "attempt_number")?)?,
        status: WebhookDeliveryStatus::parse(&row_get::<String>(&row, "status")?)?,
        http_status: optional_i64_to_u16(row_get(&row, "http_status")?)?,
        error: row_get(&row, "error")?,
        requested_at: row_get(&row, "requested_at")?,
        completed_at: row_get(&row, "completed_at")?,
        next_retry_at: row_get(&row, "next_retry_at")?,
    })
}

fn row_to_addon_registration(row: SqliteRow) -> Result<AddonRegistrationRecord> {
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
        granted_scopes,
        status: AddonStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

fn row_to_transcode_session(row: SqliteRow) -> Result<TranscodeSessionRecord> {
    Ok(TranscodeSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        kind: parse_transcode_session_kind(row_get(&row, "kind")?)?,
        request_key: row_get(&row, "request_key")?,
        output_path: PathBuf::from(row_get::<String>(&row, "output_path")?),
        state: parse_transcode_session_state(row_get(&row, "state")?)?,
        failure_category: parse_transcode_failure_category(row_get(&row, "failure_category")?)?,
        failure_message: row_get(&row, "failure_message")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    })
}

fn event_subject_from_parts(kind: String, id: String) -> Result<DomainEventSubject> {
    match kind.as_str() {
        "library" => Ok(DomainEventSubject::Library(parse_id(id)?)),
        "item" => Ok(DomainEventSubject::Item(parse_id(id)?)),
        "source" => Ok(DomainEventSubject::Source(parse_id(id)?)),
        "job" => Ok(DomainEventSubject::Job(parse_id(id)?)),
        "playback_session" => Ok(DomainEventSubject::PlaybackSession(parse_id(id)?)),
        _ => Err(TaruError::Database {
            message: format!("unknown event subject kind stored in database: {kind}"),
        }),
    }
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

fn row_to_metadata_provider_attempt(row: SqliteRow) -> Result<MetadataProviderAttemptRecord> {
    Ok(MetadataProviderAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, String::new()),
        provider_key: row_get(&row, "provider_key")?,
        status: MetadataProviderAttemptStatus::parse(&row_get::<String>(&row, "status")?)?,
        matched_by: row_get::<Option<String>>(&row, "matched_by")?
            .map(|value| MetadataMatchKind::parse(&value))
            .transpose()?,
        started_at: row_get(&row, "started_at")?,
        finished_at: row_get(&row, "finished_at")?,
        error_class: row_get::<Option<String>>(&row, "error_class")?
            .map(|value| MetadataProviderErrorClass::parse(&value))
            .transpose()?,
        message: row_get(&row, "message")?,
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

fn row_to_vfs_cached_object(row: SqliteRow) -> Result<VfsCachedObject> {
    Ok(VfsCachedObject {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        kind: VfsCachedObjectKind::parse(&row_get::<String>(&row, "kind")?)?,
        len: optional_i64_to_u64(row_get(&row, "len")?)?,
        modified_at: row_get(&row, "modified_at")?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        capabilities_bits: i64_to_u32(row_get(&row, "capabilities_bits")?)?,
        fetched_at_ms: row_get(&row, "fetched_at_ms")?,
        fresh_until_ms: row_get(&row, "fresh_until_ms")?,
    })
}

fn row_to_vfs_cache_failure(row: SqliteRow) -> Result<VfsCacheFailure> {
    Ok(VfsCacheFailure {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        operation: VfsCacheOperation::parse(&row_get::<String>(&row, "operation")?)?,
        failed_at_ms: row_get(&row, "failed_at_ms")?,
        failure_count: i64_to_u32(row_get(&row, "failure_count")?)?,
        error: row_get(&row, "error")?,
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
        AutomationJobInput, ContentRating, Credit, CreditRole, ImageKind, ImageOwner, ImageRef,
        LibraryOptions, LibraryPreset, MediaSourceId, MetadataRefreshMode, NewVfsCacheFailure,
        VfsCacheOperation, VfsCachedListing, VfsCachedObject, VfsCachedObjectKind,
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
            library_id: library.id,
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
        store.upsert_media_source(&source).await.unwrap();

        assert_eq!(
            store.get_media_item(item.id).await.unwrap(),
            Some(expected_item)
        );
        assert_eq!(
            store.get_media_source(source.id).await.unwrap(),
            Some(source.clone())
        );
        assert_eq!(
            store
                .get_media_source(source.id)
                .await
                .unwrap()
                .map(|source| source.library_id),
            Some(library.id)
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
    async fn sqlite_store_round_trips_transcode_sessions() {
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
                title: "Session Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "local:///Movies/Session Demo.mkv".to_owned(),
            file_name: "Session Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: None,
        };
        let session_id = TranscodeSessionId::new();
        let request_key = "remux:mp4".to_owned();

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();

        let planned = store
            .create_transcode_session(NewTranscodeSession {
                id: session_id,
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: request_key.clone(),
                output_path: "cache/remux/stream.mp4".into(),
                state: TranscodeSessionState::Planned,
            })
            .await
            .unwrap();

        assert_eq!(planned.id, session_id);
        assert_eq!(planned.state, TranscodeSessionState::Planned);
        assert!(planned.started_at.is_none());
        assert!(planned.completed_at.is_none());
        assert_eq!(
            store
                .find_active_transcode_session(
                    source.id,
                    TranscodeSessionKind::Remux,
                    &request_key,
                )
                .await
                .unwrap()
                .unwrap()
                .id,
            session_id
        );

        let running = store
            .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
            .await
            .unwrap();

        assert_eq!(running.state, TranscodeSessionState::Running);
        assert!(running.started_at.is_some());

        let finished = store
            .set_transcode_session_state(session_id, TranscodeSessionState::Finished, None, None)
            .await
            .unwrap();

        assert_eq!(finished.state, TranscodeSessionState::Finished);
        assert!(finished.completed_at.is_some());
        assert!(
            store
                .find_active_transcode_session(
                    source.id,
                    TranscodeSessionKind::Remux,
                    &request_key,
                )
                .await
                .unwrap()
                .is_none()
        );
        assert_eq!(
            store
                .find_latest_transcode_session(
                    source.id,
                    TranscodeSessionKind::Remux,
                    &request_key,
                )
                .await
                .unwrap()
                .unwrap()
                .id,
            session_id
        );
    }

    #[tokio::test]
    async fn sqlite_store_marks_stale_transcode_sessions_failed() {
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
                title: "Stale Session Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        let source = MediaSource {
            id: MediaSourceId::new(),
            library_id: library.id,
            item_id: item.id,
            locator: "local:///Movies/Stale Session Demo.mkv".to_owned(),
            file_name: "Stale Session Demo.mkv".to_owned(),
            size_bytes: Some(42),
            fingerprint: None,
        };
        let stale_id = TranscodeSessionId::new();
        let finished_id = TranscodeSessionId::new();

        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        store.upsert_media_source(&source).await.unwrap();
        store
            .create_transcode_session(NewTranscodeSession {
                id: stale_id,
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: "remux:mp4".to_owned(),
                output_path: "cache/remux/stale.mp4".into(),
                state: TranscodeSessionState::Running,
            })
            .await
            .unwrap();
        store
            .create_transcode_session(NewTranscodeSession {
                id: finished_id,
                source_id: source.id,
                kind: TranscodeSessionKind::Remux,
                request_key: "remux:mkv".to_owned(),
                output_path: "cache/remux/finished.mkv".into(),
                state: TranscodeSessionState::Planned,
            })
            .await
            .unwrap();
        store
            .set_transcode_session_state(finished_id, TranscodeSessionState::Finished, None, None)
            .await
            .unwrap();

        let recovered = store
            .fail_stale_transcode_sessions(
                TranscodeFailureCategory::Stale,
                "session was active during server startup".to_owned(),
            )
            .await
            .unwrap();

        let stale = store
            .get_transcode_session(stale_id)
            .await
            .unwrap()
            .unwrap();
        let finished = store
            .get_transcode_session(finished_id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(recovered, 1);
        assert_eq!(stale.state, TranscodeSessionState::Failed);
        assert_eq!(
            stale.failure_category,
            Some(TranscodeFailureCategory::Stale)
        );
        assert_eq!(finished.state, TranscodeSessionState::Finished);
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
    async fn sqlite_store_round_trips_metadata_provider_attempts() {
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
                title: "Attempt Demo".to_owned(),
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();
        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::MetadataRefresh,
                resource_class: "metadata.tmdb".to_owned(),
                library_id: Some(library.id),
                source_id: None,
                input_json: None,
            })
            .await
            .unwrap();
        let attempt = NewMetadataProviderAttempt {
            id: taru_core::MetadataProviderAttemptId::new(),
            job_id: job.id,
            item_id: item.id,
            provider: ExternalProvider::Tmdb,
            status: MetadataProviderAttemptStatus::Succeeded,
            provider_key: Some("603".to_owned()),
            matched_by: Some(MetadataMatchKind::Search),
            started_at: "2026-05-14T00:00:00Z".to_owned(),
            finished_at: "2026-05-14T00:00:01Z".to_owned(),
            error_class: None,
            message: None,
        };

        store
            .insert_metadata_provider_attempt(attempt.clone())
            .await
            .unwrap();

        assert_eq!(
            store.list_metadata_provider_attempts(job.id).await.unwrap(),
            vec![MetadataProviderAttemptRecord {
                id: attempt.id,
                job_id: attempt.job_id,
                item_id: attempt.item_id,
                provider: attempt.provider,
                status: attempt.status,
                provider_key: attempt.provider_key,
                matched_by: attempt.matched_by,
                started_at: attempt.started_at,
                finished_at: attempt.finished_at,
                error_class: attempt.error_class,
                message: attempt.message,
            }]
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
            library_id: library.id,
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
        store.upsert_media_source(&source).await.unwrap();
        store.upsert_media_probe(source.id, &result).await.unwrap();

        assert_eq!(
            store.get_media_probe(source.id).await.unwrap(),
            Some(result)
        );
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_vfs_cache_records_and_failures() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let directory = VfsCachedObject {
            uri: "webdav:///Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            kind: VfsCachedObjectKind::Directory,
            len: None,
            modified_at: Some("2026-05-15T00:00:00.000Z".to_owned()),
            etag: Some("movies".to_owned()),
            fingerprint: Some("webdav:etag=movies".to_owned()),
            capabilities_bits: 0b111,
            fetched_at_ms: 100,
            fresh_until_ms: 200,
        };
        let movie = VfsCachedObject {
            uri: "webdav:///Movies/Demo.mkv".to_owned(),
            scheme: "webdav".to_owned(),
            kind: VfsCachedObjectKind::File,
            len: Some(4),
            modified_at: Some("2026-05-15T00:00:01.000Z".to_owned()),
            etag: Some("demo".to_owned()),
            fingerprint: Some("webdav:etag=demo".to_owned()),
            capabilities_bits: 0b101,
            fetched_at_ms: 100,
            fresh_until_ms: 200,
        };
        let listing = VfsCachedListing {
            directory: directory.clone(),
            entries: vec![movie.clone()],
            fetched_at_ms: 100,
            fresh_until_ms: 200,
        };

        store.upsert_vfs_cache_listing(&listing).await.unwrap();
        let loaded_object = store
            .get_vfs_cache_object("webdav:///Movies/Demo.mkv")
            .await
            .unwrap();
        let loaded_listing = store
            .get_vfs_cache_listing("webdav:///Movies/")
            .await
            .unwrap();

        assert_eq!(loaded_object, Some(movie));
        assert_eq!(loaded_listing, Some(listing));

        let first_failure = store
            .record_vfs_cache_failure(NewVfsCacheFailure {
                uri: "webdav:///Movies/".to_owned(),
                scheme: "webdav".to_owned(),
                operation: VfsCacheOperation::List,
                failed_at_ms: 300,
                error: "timeout".to_owned(),
            })
            .await
            .unwrap();
        let second_failure = store
            .record_vfs_cache_failure(NewVfsCacheFailure {
                uri: "webdav:///Movies/".to_owned(),
                scheme: "webdav".to_owned(),
                operation: VfsCacheOperation::List,
                failed_at_ms: 400,
                error: "rate limited".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(first_failure.failure_count, 1);
        assert_eq!(second_failure.failure_count, 2);
        assert_eq!(second_failure.failed_at_ms, 400);
        assert_eq!(second_failure.error, "rate limited");
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
            library_id: library.id,
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
        store.upsert_media_source(&source).await.unwrap();
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

    #[tokio::test]
    async fn sqlite_store_round_trips_outbox_events_idempotently() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();

        let event = NewOutboxEvent {
            id: EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: None,
            idempotency_key: format!("library_scan:{}", library.id),
            payload_json: format!(r#"{{"library_id":"{}","indexed_items":1}}"#, library.id),
        };

        let first = store.enqueue_outbox_event(event.clone()).await.unwrap();
        let duplicate = store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                ..event.clone()
            })
            .await
            .unwrap();

        assert_eq!(first, duplicate);
        assert_eq!(first.kind, DomainEventKind::LibraryScanned);
        assert_eq!(first.subject, DomainEventSubject::Library(library.id));
        assert_eq!(first.status, OutboxEventStatus::Pending);
        assert_eq!(first.attempts, 0);
        assert!(first.occurred_at.ends_with('Z'));
        assert_eq!(store.get_outbox_event(first.id).await.unwrap(), Some(first));

        let events = store
            .list_outbox_events(PageRequest::first_page())
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert!(!events[0].payload_json.contains("TMDB_READ_ACCESS_TOKEN"));
        assert!(!events[0].payload_json.contains("F:/"));
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_webhook_endpoint_and_attempts() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let library = Library {
            id: LibraryId::new(),
            name: "Movies".to_owned(),
            roots: vec!["local:///Movies".to_owned()],
            options: LibraryOptions::from_preset(LibraryPreset::Movies),
        };
        store.upsert_library(&library).await.unwrap();

        let event = store
            .enqueue_outbox_event(NewOutboxEvent {
                id: EventId::new(),
                kind: DomainEventKind::LibraryScanned,
                subject: DomainEventSubject::Library(library.id),
                library_id: Some(library.id),
                source_id: None,
                idempotency_key: format!("library_scan:{}", library.id),
                payload_json: format!(r#"{{"library_id":"{}"}}"#, library.id),
            })
            .await
            .unwrap();
        let endpoint_id = WebhookEndpointId::new();
        let endpoint = store
            .upsert_webhook_endpoint(NewWebhookEndpoint {
                id: endpoint_id,
                name: "Local Receiver".to_owned(),
                url: "https://example.test/taru-webhook".to_owned(),
                secret_env: Some("TARU_TEST_WEBHOOK_SECRET".to_owned()),
                subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
                timeout_ms: 5_000,
                max_attempts: 3,
                status: WebhookEndpointStatus::Enabled,
            })
            .await
            .unwrap();

        assert_eq!(endpoint.id, endpoint_id);
        assert_eq!(
            store.list_enabled_webhook_endpoints().await.unwrap().len(),
            1
        );
        assert_eq!(
            store.get_webhook_endpoint(endpoint_id).await.unwrap(),
            Some(endpoint.clone())
        );

        let attempt = store
            .create_webhook_delivery_attempt(NewWebhookDeliveryAttempt {
                id: WebhookDeliveryAttemptId::new(),
                endpoint_id,
                event_id: event.id,
                attempt_number: 1,
            })
            .await
            .unwrap();
        assert_eq!(attempt.status, WebhookDeliveryStatus::Pending);

        let failed = store
            .set_webhook_delivery_attempt_result(
                attempt.id,
                WebhookDeliveryStatus::Failed,
                Some(503),
                Some("receiver returned 503".to_owned()),
                Some("2026-05-15T00:00:10Z".to_owned()),
            )
            .await
            .unwrap();
        assert_eq!(failed.status, WebhookDeliveryStatus::Failed);
        assert_eq!(failed.http_status, Some(503));
        assert_eq!(
            store
                .list_webhook_delivery_attempts(event.id)
                .await
                .unwrap(),
            vec![failed]
        );
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_automation_provider_and_artifacts() {
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
                ..CanonicalMetadata::default()
            },
        };
        store.upsert_library(&library).await.unwrap();
        store.upsert_media_item(&item).await.unwrap();

        let provider_id = AutomationProviderId::new();
        let provider = store
            .upsert_automation_provider(NewAutomationProviderConfig {
                id: provider_id,
                name: "Gateway".to_owned(),
                base_url: "https://example.test/automation".to_owned(),
                secret_env: Some("TARU_AUTOMATION_SECRET".to_owned()),
                capabilities: vec![
                    AutomationCapability::Summary,
                    AutomationCapability::TitleMatch,
                ],
                timeout_ms: 10_000,
                max_attempts: 2,
                status: AutomationProviderStatus::Enabled,
            })
            .await
            .unwrap();

        assert_eq!(provider.id, provider_id);
        assert_eq!(
            store.list_enabled_automation_providers().await.unwrap(),
            vec![provider.clone()]
        );
        assert_eq!(
            store.get_automation_provider(provider_id).await.unwrap(),
            Some(provider)
        );

        let job = store
            .enqueue_job(NewJob {
                id: JobId::new(),
                kind: JobKind::Automation,
                resource_class: "automation.external_api".to_owned(),
                library_id: Some(library.id),
                source_id: None,
                input_json: Some(
                    serde_json::to_string(&AutomationJobInput {
                        provider_id,
                        capability: AutomationCapability::Summary,
                        library_id: Some(library.id),
                        item_id: Some(item.id),
                        source_id: None,
                        prompt_json: r#"{"title":"The Matrix"}"#.to_owned(),
                        idempotency_key: format!("summary:{}", item.id),
                    })
                    .unwrap(),
                ),
            })
            .await
            .unwrap();
        let artifact = store
            .create_automation_artifact(NewAutomationArtifact {
                id: AutomationArtifactId::new(),
                job_id: job.id,
                provider_id,
                capability: AutomationCapability::Summary,
                kind: AutomationArtifactKind::Summary,
                library_id: Some(library.id),
                item_id: Some(item.id),
                source_id: None,
                artifact_json: r#"{"summary":"A generated summary."}"#.to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(artifact.status, AutomationArtifactStatus::Proposed);
        assert!(artifact.accepted_at.is_none());
        assert_eq!(
            store
                .list_automation_artifacts_for_job(job.id)
                .await
                .unwrap(),
            vec![artifact.clone()]
        );
        assert_eq!(
            store
                .list_automation_artifacts_for_item(item.id, PageRequest::first_page())
                .await
                .unwrap(),
            vec![artifact.clone()]
        );

        let accepted = store
            .set_automation_artifact_status(artifact.id, AutomationArtifactStatus::Accepted)
            .await
            .unwrap();
        assert_eq!(accepted.status, AutomationArtifactStatus::Accepted);
        assert!(accepted.accepted_at.is_some());
    }

    #[tokio::test]
    async fn sqlite_store_round_trips_addon_registration() {
        let store = SqliteStore::connect_in_memory().await.unwrap();
        store.migrate().await.unwrap();

        let addon_id = AddonId::new();
        let manifest_json = r#"{
            "id":"example.metadata",
            "name":"Example Metadata",
            "version":"0.1.0",
            "protocol_version":"2026-05-15",
            "base_url":"https://example.test/addon"
        }"#
        .to_owned();
        let registration = store
            .upsert_addon_registration(NewAddonRegistration {
                id: addon_id,
                manifest_id: "example.metadata".to_owned(),
                name: "Example Metadata".to_owned(),
                version: "0.1.0".to_owned(),
                protocol_version: "2026-05-15".to_owned(),
                base_url: "https://example.test/addon".to_owned(),
                manifest_json,
                granted_scopes: vec!["item_metadata_read".to_owned()],
                status: AddonStatus::Disabled,
            })
            .await
            .unwrap();

        assert_eq!(registration.id, addon_id);
        assert_eq!(registration.status, AddonStatus::Disabled);
        assert_eq!(registration.granted_scopes, vec!["item_metadata_read"]);
        assert_eq!(
            store.get_addon_registration(addon_id).await.unwrap(),
            Some(registration.clone())
        );
        assert_eq!(
            store
                .find_addon_registration_by_manifest_id("example.metadata")
                .await
                .unwrap(),
            Some(registration.clone())
        );
        assert_eq!(
            store
                .list_addon_registrations(Some(AddonStatus::Disabled))
                .await
                .unwrap(),
            vec![registration]
        );
        assert!(
            store
                .list_addon_registrations(Some(AddonStatus::Enabled))
                .await
                .unwrap()
                .is_empty()
        );
    }

    fn external_id_sort_key(external_id: &ExternalId) -> String {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        format!("{provider}\0{provider_key}\0{}", external_id.value)
    }
}
