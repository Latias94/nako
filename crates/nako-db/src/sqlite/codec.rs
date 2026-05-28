use std::{fmt::Display, path::PathBuf, str::FromStr};

use nako_core::*;
use sqlx::{Decode, Row, Sqlite, Type, sqlite::SqliteRow};

pub(crate) async fn insert_external_ids<T>(
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

pub(crate) async fn upsert_vfs_cache_object_tx(
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

pub(crate) fn vfs_cache_object_upsert_sql() -> &'static str {
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

pub(crate) fn media_kind_to_str(kind: MediaKind) -> &'static str {
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

pub(crate) fn parse_media_kind(value: String) -> Result<MediaKind> {
    match value.as_str() {
        "movie" => Ok(MediaKind::Movie),
        "series" => Ok(MediaKind::Series),
        "season" => Ok(MediaKind::Season),
        "episode" => Ok(MediaKind::Episode),
        "collection" => Ok(MediaKind::Collection),
        "extra" => Ok(MediaKind::Extra),
        "unknown" => Ok(MediaKind::Unknown),
        _ => Err(NakoError::Database {
            message: format!("unknown media kind stored in database: {value}"),
        }),
    }
}

pub(crate) fn provider_to_parts(provider: &ExternalProvider) -> (String, String) {
    match provider {
        ExternalProvider::Tmdb => ("tmdb".to_owned(), String::new()),
        ExternalProvider::Douban => ("douban".to_owned(), String::new()),
        ExternalProvider::Bangumi => ("bangumi".to_owned(), String::new()),
        ExternalProvider::Imdb => ("imdb".to_owned(), String::new()),
        ExternalProvider::Local => ("local".to_owned(), String::new()),
        ExternalProvider::Other(value) => ("other".to_owned(), value.clone()),
    }
}

pub(crate) fn provider_from_parts(provider: String, provider_key: String) -> ExternalProvider {
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

pub(crate) fn provider_subject_kind_to_parts(kind: &ProviderSubjectKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

pub(crate) fn provider_subject_kind_from_parts(
    kind: String,
    kind_key: String,
) -> ProviderSubjectKind {
    ProviderSubjectKind::from_parts(&kind, kind_key)
}

pub(crate) fn source_duplicate_evidence_kind_to_parts(
    kind: &SourceDuplicateEvidenceKind,
) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

pub(crate) fn source_duplicate_evidence_kind_from_parts(
    kind: String,
    kind_key: String,
) -> SourceDuplicateEvidenceKind {
    SourceDuplicateEvidenceKind::from_parts(&kind, kind_key)
}

pub(crate) fn managed_import_source_kind_to_parts(
    kind: &ManagedImportSourceKind,
) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

pub(crate) fn managed_import_source_kind_from_parts(
    kind: String,
    kind_key: String,
) -> ManagedImportSourceKind {
    ManagedImportSourceKind::from_parts(&kind, kind_key)
}

pub(crate) fn acquisition_intake_source_kind_to_parts(
    kind: &AcquisitionIntakeSourceKind,
) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

pub(crate) fn acquisition_intake_source_kind_from_parts(
    kind: String,
    kind_key: String,
) -> AcquisitionIntakeSourceKind {
    AcquisitionIntakeSourceKind::from_parts(&kind, kind_key)
}

pub(crate) fn local_inference_evidence_source_to_parts(
    source: &LocalInferenceEvidenceSource,
) -> (String, String) {
    let (source, source_key) = source.as_parts();
    (source.to_owned(), source_key.to_owned())
}

pub(crate) fn local_inference_evidence_source_from_parts(
    source: String,
    source_key: String,
) -> LocalInferenceEvidenceSource {
    LocalInferenceEvidenceSource::from_parts(&source, source_key)
}

pub(crate) fn metadata_source_to_parts(source: &MetadataSource) -> (String, String) {
    match source {
        MetadataSource::Local => ("local".to_owned(), String::new()),
        MetadataSource::Nfo => ("nfo".to_owned(), String::new()),
        MetadataSource::User => ("user".to_owned(), String::new()),
        MetadataSource::Addon(addon_id) => ("addon".to_owned(), addon_id.to_string()),
        MetadataSource::Provider(provider) => {
            let (provider, provider_key) = provider_to_parts(provider);
            (format!("provider:{provider}"), provider_key)
        }
    }
}

pub(crate) fn metadata_source_from_parts(source: String, source_key: String) -> MetadataSource {
    match source.as_str() {
        "local" => MetadataSource::Local,
        "nfo" => MetadataSource::Nfo,
        "user" => MetadataSource::User,
        "addon" => parse_id(source_key)
            .map(MetadataSource::Addon)
            .unwrap_or_else(|_| MetadataSource::Provider(ExternalProvider::Other(source))),
        value if value.starts_with("provider:") => {
            let provider = value.trim_start_matches("provider:").to_owned();
            MetadataSource::Provider(provider_from_parts(provider, source_key))
        }
        _ => MetadataSource::Provider(ExternalProvider::Other(source)),
    }
}

pub(crate) fn stream_kind_to_parts(kind: &MediaStreamKind) -> (String, String) {
    match kind {
        MediaStreamKind::Video => ("video".to_owned(), String::new()),
        MediaStreamKind::Audio => ("audio".to_owned(), String::new()),
        MediaStreamKind::Subtitle => ("subtitle".to_owned(), String::new()),
        MediaStreamKind::Data => ("data".to_owned(), String::new()),
        MediaStreamKind::Attachment => ("attachment".to_owned(), String::new()),
        MediaStreamKind::Other(value) => ("other".to_owned(), value.clone()),
    }
}

pub(crate) fn stream_kind_from_parts(kind: String, kind_key: String) -> MediaStreamKind {
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

pub(crate) fn parse_transcode_session_kind(value: String) -> Result<TranscodeSessionKind> {
    TranscodeSessionKind::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown transcode session kind stored in database: {value}"),
    })
}

pub(crate) fn parse_transcode_session_state(value: String) -> Result<TranscodeSessionState> {
    TranscodeSessionState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown transcode session state stored in database: {value}"),
    })
}

pub(crate) fn parse_transcode_failure_category(
    value: Option<String>,
) -> Result<Option<TranscodeFailureCategory>> {
    value
        .map(|value| {
            TranscodeFailureCategory::parse(&value).ok_or_else(|| NakoError::Database {
                message: format!("unknown transcode failure category stored in database: {value}"),
            })
        })
        .transpose()
}

pub(crate) fn parse_playback_session_mode(value: String) -> Result<PlaybackSessionMode> {
    PlaybackSessionMode::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown playback session mode stored in database: {value}"),
    })
}

pub(crate) fn parse_playback_session_state(value: String) -> Result<PlaybackSessionState> {
    PlaybackSessionState::parse(&value).ok_or_else(|| NakoError::Database {
        message: format!("unknown playback session state stored in database: {value}"),
    })
}

pub(crate) fn ingestion_failure_phase_to_str(phase: IngestionFailurePhase) -> &'static str {
    phase.as_str()
}

pub(crate) fn parse_ingestion_failure_phase(value: String) -> Result<IngestionFailurePhase> {
    IngestionFailurePhase::parse(&value)
}

pub(crate) fn ingestion_failure_class_to_str(class: IngestionFailureClass) -> &'static str {
    class.as_str()
}

pub(crate) fn parse_ingestion_failure_class(value: String) -> Result<IngestionFailureClass> {
    IngestionFailureClass::parse(&value)
}

pub(crate) fn ingestion_failure_status_to_str(status: IngestionFailureStatus) -> &'static str {
    status.as_str()
}

pub(crate) fn parse_ingestion_failure_status(value: String) -> Result<IngestionFailureStatus> {
    IngestionFailureStatus::parse(&value)
}

pub(crate) fn credit_role_to_parts(role: &CreditRole) -> (String, String) {
    match role {
        CreditRole::Actor => ("actor".to_owned(), String::new()),
        CreditRole::Director => ("director".to_owned(), String::new()),
        CreditRole::Writer => ("writer".to_owned(), String::new()),
        CreditRole::Producer => ("producer".to_owned(), String::new()),
        CreditRole::Creator => ("creator".to_owned(), String::new()),
        CreditRole::Other(value) => ("other".to_owned(), value.clone()),
    }
}

pub(crate) fn credit_role_from_parts(role: String, role_key: String) -> CreditRole {
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

pub(crate) fn image_kind_to_parts(kind: &ImageKind) -> (String, String) {
    match kind {
        ImageKind::Poster => ("poster".to_owned(), String::new()),
        ImageKind::Backdrop => ("backdrop".to_owned(), String::new()),
        ImageKind::Logo => ("logo".to_owned(), String::new()),
        ImageKind::Thumbnail => ("thumbnail".to_owned(), String::new()),
        ImageKind::Banner => ("banner".to_owned(), String::new()),
        ImageKind::Other(value) => ("other".to_owned(), value.clone()),
    }
}

pub(crate) fn image_kind_from_parts(kind: String, kind_key: String) -> ImageKind {
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

pub(crate) fn image_owner_to_parts(owner: &ImageOwner) -> (String, String) {
    match owner {
        ImageOwner::Item(id) => ("item".to_owned(), id.to_string()),
        ImageOwner::Person(id) => ("person".to_owned(), id.to_string()),
        ImageOwner::Collection(id) => ("collection".to_owned(), id.to_string()),
        ImageOwner::Studio(id) => ("studio".to_owned(), id.to_string()),
    }
}

pub(crate) fn image_owner_from_parts(owner_kind: String, owner_id: String) -> Result<ImageOwner> {
    match owner_kind.as_str() {
        "item" => Ok(ImageOwner::Item(parse_id(owner_id)?)),
        "person" => Ok(ImageOwner::Person(parse_id(owner_id)?)),
        "collection" => Ok(ImageOwner::Collection(parse_id(owner_id)?)),
        "studio" => Ok(ImageOwner::Studio(parse_id(owner_id)?)),
        _ => Err(NakoError::Database {
            message: format!("unknown image owner kind stored in database: {owner_kind}"),
        }),
    }
}

pub(crate) fn parse_id<T>(value: String) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    value.parse().map_err(database_error)
}

pub(crate) fn parse_optional_id<T>(value: Option<String>) -> Result<Option<T>>
where
    T: FromStr,
    T::Err: Display,
{
    value.map(parse_id).transpose()
}

pub(crate) fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>> {
    value
        .map(|value| {
            i64::try_from(value).map_err(|err| NakoError::Database {
                message: format!("value does not fit into SQLite integer: {err}"),
            })
        })
        .transpose()
}

pub(crate) fn optional_i64_to_u64(value: Option<i64>) -> Result<Option<u64>> {
    value
        .map(|value| {
            u64::try_from(value).map_err(|err| NakoError::Database {
                message: format!("negative SQLite integer cannot be converted to u64: {err}"),
            })
        })
        .transpose()
}

pub(crate) fn i64_to_u64(value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|err| NakoError::Database {
        message: format!("negative SQLite integer cannot be converted to u64: {err}"),
    })
}

pub(crate) fn optional_u32_to_i64(value: Option<u32>) -> Option<i64> {
    value.map(i64::from)
}

pub(crate) fn optional_i64_to_u32(value: Option<i64>) -> Result<Option<u32>> {
    value
        .map(|value| {
            u32::try_from(value).map_err(|err| NakoError::Database {
                message: format!("SQLite integer cannot be converted to u32: {err}"),
            })
        })
        .transpose()
}

pub(crate) fn optional_i64_to_u16(value: Option<i64>) -> Result<Option<u16>> {
    value
        .map(|value| {
            u16::try_from(value).map_err(|err| NakoError::Database {
                message: format!("SQLite integer cannot be converted to u16: {err}"),
            })
        })
        .transpose()
}

pub(crate) fn optional_u16_to_i64(value: Option<u16>) -> Option<i64> {
    value.map(i64::from)
}

pub(crate) fn optional_i64_to_i32(value: Option<i64>) -> Result<Option<i32>> {
    value
        .map(|value| {
            i32::try_from(value).map_err(|err| NakoError::Database {
                message: format!("SQLite integer cannot be converted to i32: {err}"),
            })
        })
        .transpose()
}

pub(crate) fn optional_i32_to_i64(value: Option<i32>) -> Option<i64> {
    value.map(i64::from)
}

pub(crate) fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

pub(crate) fn i64_to_bool(value: i64) -> Result<bool> {
    match value {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(NakoError::Database {
            message: format!("SQLite integer cannot be converted to bool: {value}"),
        }),
    }
}

pub(crate) fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

pub(crate) fn u64_to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|err| NakoError::Database {
        message: format!("value does not fit into SQLite integer: {err}"),
    })
}

pub(crate) fn i64_to_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|err| NakoError::Database {
        message: format!("SQLite integer cannot be converted to u32: {err}"),
    })
}

pub(crate) fn row_to_library(row: SqliteRow) -> Result<Library> {
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

pub(crate) fn row_to_library_item_state(row: SqliteRow) -> Result<LibraryItemState> {
    Ok(LibraryItemState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provisional: i64_to_bool(row_get(&row, "provisional")?)?,
    })
}

pub(crate) fn row_to_library_options(row: &SqliteRow) -> Result<LibraryOptions> {
    if let Some(options_json) = row_get::<Option<String>>(row, "options_json")? {
        return serde_json::from_str(&options_json).map_err(database_error);
    }

    let domain = row_get::<String>(row, "domain")?;
    let preset = row_get::<String>(row, "preset")?;
    let preset = nako_core::LibraryPreset::parse(&preset).ok_or_else(|| NakoError::Database {
        message: format!("unknown library preset stored in database: {preset}"),
    })?;
    let mut options = LibraryOptions::from_preset(preset);
    options.domain = MediaDomain::parse(&domain).ok_or_else(|| NakoError::Database {
        message: format!("unknown media domain stored in database: {domain}"),
    })?;

    Ok(options)
}

pub(crate) fn row_to_media_item(
    row: SqliteRow,
    external_ids: Vec<ExternalId>,
) -> Result<MediaItem> {
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

pub(crate) fn row_to_media_source(row: SqliteRow) -> Result<MediaSource> {
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

pub(crate) fn row_to_stream_info(row: SqliteRow) -> Result<MediaStreamInfo> {
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
        technical: deserialize_stream_technical_json(row_get(&row, "technical_json")?)?,
    })
}

fn deserialize_stream_technical_json(value: Option<String>) -> Result<MediaStreamTechnicalFacts> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str(&value).map_err(database_error)
        }
        _ => Ok(MediaStreamTechnicalFacts::default()),
    }
}

pub(crate) fn row_to_job(row: SqliteRow) -> Result<Job> {
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

pub(crate) fn row_to_outbox_event(row: SqliteRow) -> Result<OutboxEventRecord> {
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

pub(crate) fn row_to_automation_provider(row: SqliteRow) -> Result<AutomationProviderConfigRecord> {
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

pub(crate) fn row_to_automation_artifact(row: SqliteRow) -> Result<AutomationArtifactRecord> {
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

pub(crate) fn row_to_webhook_endpoint(row: SqliteRow) -> Result<WebhookEndpointRecord> {
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

pub(crate) fn row_to_webhook_delivery_attempt(
    row: SqliteRow,
) -> Result<WebhookDeliveryAttemptRecord> {
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

pub(crate) fn row_to_addon_event_delivery_attempt(
    row: SqliteRow,
) -> Result<AddonEventDeliveryAttemptRecord> {
    Ok(AddonEventDeliveryAttemptRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        event_id: parse_id(row_get::<String>(&row, "event_id")?)?,
        declaration_id: row_get(&row, "declaration_id")?,
        attempt_number: i64_to_u32(row_get(&row, "attempt_number")?)?,
        status: AddonEventDeliveryStatus::parse(&row_get::<String>(&row, "status")?)?,
        http_status: optional_i64_to_u16(row_get(&row, "http_status")?)?,
        error: row_get(&row, "error")?,
        requested_at: row_get(&row, "requested_at")?,
        completed_at: row_get(&row, "completed_at")?,
        next_retry_at: row_get(&row, "next_retry_at")?,
        lease_expires_at: row_get(&row, "lease_expires_at")?,
        forced_replay: row_get::<i64>(&row, "forced_replay")? != 0,
        replay_reason_code: row_get(&row, "replay_reason_code")?,
    })
}

pub(crate) fn row_to_addon_registration(row: SqliteRow) -> Result<AddonRegistrationRecord> {
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
        outbound_task_dispatch_secret_env: row_get(&row, "outbound_task_dispatch_secret_env")?,
        granted_scopes,
        status: AddonStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn row_to_addon_token(row: SqliteRow) -> Result<AddonTokenRecord> {
    Ok(AddonTokenRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        label: row_get(&row, "label")?,
        token_prefix: row_get(&row, "token_prefix")?,
        token_hash: row_get(&row, "token_hash")?,
        status: AddonTokenStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        rotated_at: row_get(&row, "rotated_at")?,
        revoked_at: row_get(&row, "revoked_at")?,
        last_used_at: row_get(&row, "last_used_at")?,
    })
}

pub(crate) fn row_to_addon_grant(row: SqliteRow) -> Result<AddonGrantRecord> {
    Ok(AddonGrantRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        permission: AddonPermission::parse(&row_get::<String>(&row, "permission")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        created_at: row_get(&row, "created_at")?,
    })
}

pub(crate) fn row_to_addon_routing_plan(row: SqliteRow) -> Result<AddonRoutingPlanRecord> {
    let job_kind = row_get::<Option<String>>(&row, "job_kind")?
        .map(|kind| JobKind::parse(&kind))
        .transpose()?;

    Ok(AddonRoutingPlanRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        manifest_id: row_get(&row, "manifest_id")?,
        manifest_version: row_get(&row, "manifest_version")?,
        manifest_fingerprint: AddonManifestFingerprint::parse(row_get::<String>(
            &row,
            "manifest_fingerprint",
        )?)?,
        declaration_kind: AddonRoutingDeclarationKind::parse(&row_get::<String>(
            &row,
            "declaration_kind",
        )?)?,
        declaration_id: row_get(&row, "declaration_id")?,
        status: AddonRoutingPlanStatus::parse(&row_get::<String>(&row, "status")?)?,
        target: AddonRoutingPlanTarget::parse(&row_get::<String>(&row, "target")?)?,
        safe_reason_code: row_get(&row, "safe_reason_code")?,
        job_kind,
        event_kind: row_get(&row, "event_kind")?,
        plan_json: row_get(&row, "plan_json")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn row_to_addon_side_effect(row: SqliteRow) -> Result<AddonSideEffectRecord> {
    let target = AddonSideEffectTarget {
        kind: AddonSideEffectTargetKind::parse(&row_get::<String>(&row, "target_kind")?)?,
        id: row_get(&row, "target_id")?,
    };

    let permission = AddonPermission::parse(&row_get::<String>(&row, "permission")?)?;
    let library_id = parse_id(row_get::<String>(&row, "library_id")?)?;
    let request_fingerprint =
        AddonSideEffectRequestFingerprint::parse(row_get::<String>(&row, "request_fingerprint")?)?;

    Ok(AddonSideEffectRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        token_id: parse_id(row_get::<String>(&row, "token_id")?)?,
        permission,
        library_id,
        target,
        idempotency_key: row_get(&row, "idempotency_key")?,
        request_fingerprint,
        provenance_json: row_get(&row, "provenance_json")?,
        payload_json: row_get(&row, "payload_json")?,
        validation_status: AddonSideEffectValidationStatus::parse(&row_get::<String>(
            &row,
            "validation_status",
        )?)?,
        safe_error_code: row_get(&row, "safe_error_code")?,
        apply_status: AddonSideEffectApplyStatus::parse(&row_get::<String>(&row, "apply_status")?)?,
        apply_error_code: row_get(&row, "apply_error_code")?,
        applied_item_id: parse_optional_id(row_get::<Option<String>>(&row, "applied_item_id")?)?,
        applied_source: row_get(&row, "applied_source")?,
        apply_report_json: row_get(&row, "apply_report_json")?,
        applied_at: row_get(&row, "applied_at")?,
        created_at: row_get(&row, "created_at")?,
    })
}

pub(crate) fn row_to_transcode_session(row: SqliteRow) -> Result<TranscodeSessionRecord> {
    Ok(TranscodeSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        kind: parse_transcode_session_kind(row_get(&row, "kind")?)?,
        request_key: row_get(&row, "request_key")?,
        output_path: PathBuf::from(row_get::<String>(&row, "output_path")?),
        state: parse_transcode_session_state(row_get(&row, "state")?)?,
        failure_category: parse_transcode_failure_category(row_get(&row, "failure_category")?)?,
        failure_message: row_get(&row, "failure_message")?,
        runtime_metrics: deserialize_transcode_runtime_metrics_json(row_get(
            &row,
            "runtime_metrics_json",
        )?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
        started_at: row_get(&row, "started_at")?,
        completed_at: row_get(&row, "completed_at")?,
    })
}

fn deserialize_transcode_runtime_metrics_json(
    value: Option<String>,
) -> Result<TranscodeSessionRuntimeMetrics> {
    match value {
        Some(value) if !value.trim().is_empty() => {
            serde_json::from_str(&value).map_err(database_error)
        }
        _ => Ok(TranscodeSessionRuntimeMetrics::default()),
    }
}

pub(crate) fn row_to_playback_session(row: SqliteRow) -> Result<PlaybackSessionRecord> {
    Ok(PlaybackSessionRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        mode: parse_playback_session_mode(row_get(&row, "mode")?)?,
        state: parse_playback_session_state(row_get(&row, "state")?)?,
        client_capabilities_json: row_get(&row, "client_capabilities_json")?,
        transcode_session_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "transcode_session_id",
        )?)?,
        position_ms: optional_i64_to_u64(row_get(&row, "position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        last_heartbeat_at_ms: row_get(&row, "last_heartbeat_at_ms")?,
        started_at_ms: row_get(&row, "started_at_ms")?,
        ended_at_ms: row_get(&row, "ended_at_ms")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn event_subject_from_parts(kind: String, id: String) -> Result<DomainEventSubject> {
    match kind.as_str() {
        "library" => Ok(DomainEventSubject::Library(parse_id(id)?)),
        "item" => Ok(DomainEventSubject::Item(parse_id(id)?)),
        "source" => Ok(DomainEventSubject::Source(parse_id(id)?)),
        "job" => Ok(DomainEventSubject::Job(parse_id(id)?)),
        "playback_session" => Ok(DomainEventSubject::PlaybackSession(parse_id(id)?)),
        _ => Err(NakoError::Database {
            message: format!("unknown event subject kind stored in database: {kind}"),
        }),
    }
}

pub(crate) fn row_to_metadata_field_lock(row: SqliteRow) -> Result<MetadataFieldLock> {
    Ok(MetadataFieldLock {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        field: metadata_field_from_str(&row_get::<String>(&row, "field")?)?,
        locked: i64_to_bool(row_get(&row, "locked")?)?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

pub(crate) fn row_to_provider_raw_response(row: SqliteRow) -> Result<ProviderRawResponse> {
    Ok(ProviderRawResponse {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        provider_key: row_get(&row, "provider_key")?,
        body_json: row_get(&row, "body_json")?,
        fetched_at: row_get(&row, "fetched_at")?,
    })
}

pub(crate) fn row_to_metadata_provider_attempt(
    row: SqliteRow,
) -> Result<MetadataProviderAttemptRecord> {
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

pub(crate) fn row_to_provider_subject(row: SqliteRow) -> Result<ProviderSubject> {
    Ok(ProviderSubject {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        subject_kind: provider_subject_kind_from_parts(
            row_get(&row, "subject_kind")?,
            row_get(&row, "subject_kind_key")?,
        ),
        subject_key: row_get(&row, "subject_key")?,
        title: row_get(&row, "title")?,
        release_year: optional_i64_to_i32(row_get(&row, "release_year")?)?,
        locale: row_get(&row, "locale")?,
    })
}

pub(crate) fn row_to_provider_mapping(row: SqliteRow) -> Result<ProviderMapping> {
    Ok(ProviderMapping {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        subject_id: parse_id(row_get::<String>(&row, "subject_id")?)?,
        status: ProviderMappingStatus::parse(&row_get::<String>(&row, "status")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

pub(crate) fn row_to_source_duplicate_relationship(
    row: SqliteRow,
) -> Result<SourceDuplicateRelationship> {
    Ok(SourceDuplicateRelationship {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        duplicate_source_id: parse_id(row_get::<String>(&row, "duplicate_source_id")?)?,
        evidence_kind: source_duplicate_evidence_kind_from_parts(
            row_get(&row, "evidence_kind")?,
            row_get(&row, "evidence_kind_key")?,
        ),
        evidence_value: row_get(&row, "evidence_value")?,
        status: SourceDuplicateRelationshipStatus::parse(&row_get::<String>(&row, "status")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
    })
}

pub(crate) fn row_to_local_inference_evidence(row: SqliteRow) -> Result<LocalInferenceEvidence> {
    Ok(LocalInferenceEvidence {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_id: parse_id(row_get::<String>(&row, "source_id")?)?,
        inferred_kind: parse_media_kind(row_get(&row, "inferred_kind")?)?,
        inferred_title: row_get(&row, "inferred_title")?,
        inferred_year: optional_i64_to_i32(row_get(&row, "inferred_year")?)?,
        inferred_season: optional_i64_to_u32(row_get(&row, "inferred_season")?)?,
        inferred_episode: optional_i64_to_u32(row_get(&row, "inferred_episode")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
        evidence_source: local_inference_evidence_source_from_parts(
            row_get(&row, "evidence_source")?,
            row_get(&row, "evidence_source_key")?,
        ),
        evidence_value: row_get(&row, "evidence_value")?,
        inference_version: row_get(&row, "inference_version")?,
    })
}

pub(crate) fn row_to_person(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<Person> {
    Ok(Person {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        sort_name: row_get(&row, "sort_name")?,
        overview: row_get(&row, "overview")?,
        external_ids,
    })
}

pub(crate) fn row_to_item_credit(row: SqliteRow) -> Result<ItemCredit> {
    let character = row_get::<String>(&row, "character")?;

    Ok(ItemCredit {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        person_id: parse_id(row_get::<String>(&row, "person_id")?)?,
        role: credit_role_from_parts(row_get(&row, "role")?, row_get(&row, "role_key")?),
        character: (!character.is_empty()).then_some(character),
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

pub(crate) fn row_to_genre(row: SqliteRow) -> Result<Genre> {
    Ok(Genre {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

pub(crate) fn row_to_item_genre(row: SqliteRow) -> Result<ItemGenre> {
    Ok(ItemGenre {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        genre_id: parse_id(row_get::<String>(&row, "genre_id")?)?,
    })
}

pub(crate) fn row_to_tag(row: SqliteRow) -> Result<Tag> {
    Ok(Tag {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

pub(crate) fn row_to_item_tag(row: SqliteRow) -> Result<ItemTag> {
    Ok(ItemTag {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        tag_id: parse_id(row_get::<String>(&row, "tag_id")?)?,
    })
}

pub(crate) fn row_to_collection(
    row: SqliteRow,
    external_ids: Vec<ExternalId>,
) -> Result<Collection> {
    Ok(Collection {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        overview: row_get(&row, "overview")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

pub(crate) fn row_to_collection_item(row: SqliteRow) -> Result<CollectionItem> {
    Ok(CollectionItem {
        collection_id: parse_id(row_get::<String>(&row, "collection_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

pub(crate) fn row_to_studio(row: SqliteRow, external_ids: Vec<ExternalId>) -> Result<Studio> {
    Ok(Studio {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

pub(crate) fn row_to_item_studio(row: SqliteRow) -> Result<ItemStudio> {
    Ok(ItemStudio {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        studio_id: parse_id(row_get::<String>(&row, "studio_id")?)?,
    })
}

pub(crate) fn row_to_image_asset(row: SqliteRow) -> Result<ImageAsset> {
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

pub(crate) fn row_to_scan_snapshot(row: SqliteRow) -> Result<ScanSnapshot> {
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

pub(crate) fn row_to_directory_snapshot(row: SqliteRow) -> Result<DirectorySnapshot> {
    Ok(DirectorySnapshot {
        scan_id: parse_id(row_get::<String>(&row, "scan_id")?)?,
        uri: row_get(&row, "uri")?,
        etag: row_get(&row, "etag")?,
        modified_at: row_get(&row, "modified_at")?,
        child_count: optional_i64_to_u64(Some(row_get(&row, "child_count")?))?.unwrap_or_default(),
    })
}

pub(crate) fn row_to_source_state(row: SqliteRow) -> Result<SourceState> {
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

pub(crate) fn row_to_ingestion_failure(row: SqliteRow) -> Result<IngestionFailureRecord> {
    Ok(IngestionFailureRecord {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        job_id: parse_optional_id(row_get::<Option<String>>(&row, "job_id")?)?,
        scan_id: parse_optional_id(row_get::<Option<String>>(&row, "scan_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        phase: parse_ingestion_failure_phase(row_get(&row, "phase")?)?,
        target_uri: row_get(&row, "target_uri")?,
        target_kind: row_get(&row, "target_kind")?,
        failure_class: parse_ingestion_failure_class(row_get(&row, "failure_class")?)?,
        status: parse_ingestion_failure_status(row_get(&row, "status")?)?,
        message: row_get(&row, "message")?,
        retryable: i64_to_bool(row_get(&row, "retryable")?)?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        first_failed_at_ms: row_get(&row, "first_failed_at_ms")?,
        last_failed_at_ms: row_get(&row, "last_failed_at_ms")?,
        resolved_at_ms: row_get(&row, "resolved_at_ms")?,
        ignored_at_ms: row_get(&row, "ignored_at_ms")?,
    })
}

pub(crate) fn row_to_vfs_cached_object(row: SqliteRow) -> Result<VfsCachedObject> {
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

pub(crate) fn row_to_vfs_cache_failure(row: SqliteRow) -> Result<VfsCacheFailure> {
    Ok(VfsCacheFailure {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        operation: VfsCacheOperation::parse(&row_get::<String>(&row, "operation")?)?,
        failed_at_ms: row_get(&row, "failed_at_ms")?,
        failure_count: i64_to_u32(row_get(&row, "failure_count")?)?,
        error: row_get(&row, "error")?,
    })
}

pub(crate) fn row_to_artwork_task(row: SqliteRow) -> Result<ArtworkTask> {
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

pub(crate) fn row_to_artwork_candidate(row: SqliteRow) -> Result<ArtworkCandidateRecord> {
    Ok(ArtworkCandidateRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        side_effect_id: parse_id(row_get::<String>(&row, "side_effect_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        source_kind: ArtworkCandidateSourceKind::parse(&row_get::<String>(&row, "source_kind")?)?,
        source_uri: row_get(&row, "source_uri")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        language: row_get(&row, "language")?,
        status: ArtworkCandidateStatus::parse(&row_get::<String>(&row, "status")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn row_to_managed_artwork_ingest(row: SqliteRow) -> Result<ManagedArtworkIngestRecord> {
    Ok(ManagedArtworkIngestRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        candidate_id: parse_id(row_get::<String>(&row, "candidate_id")?)?,
        job_id: parse_id(row_get::<String>(&row, "job_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        status: ManagedArtworkIngestStatus::parse(&row_get::<String>(&row, "status")?)?,
        artifact_id: parse_optional_id(row_get::<Option<String>>(&row, "artifact_id")?)?,
        failure_code: row_get(&row, "failure_code")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn row_to_managed_artwork_artifact(
    row: SqliteRow,
) -> Result<ManagedArtworkArtifactRecord> {
    Ok(ManagedArtworkArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        ingest_id: parse_id(row_get::<String>(&row, "ingest_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        storage_uri: row_get(&row, "storage_uri")?,
        content_hash: row_get(&row, "content_hash")?,
        width: optional_i64_to_u32(row_get(&row, "width")?)?,
        height: optional_i64_to_u32(row_get(&row, "height")?)?,
        byte_len: optional_i64_to_u64(row_get(&row, "byte_len")?)?,
        media_type: row_get(&row, "media_type")?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn row_to_managed_artwork_artifact_lifecycle(
    row: SqliteRow,
) -> Result<ManagedArtworkArtifactLifecycleRecord> {
    let selected_artwork_count = i64_to_u32(row_get(&row, "selected_artwork_count")?)?;

    Ok(ManagedArtworkArtifactLifecycleRecord {
        artifact: row_to_managed_artwork_artifact(row)?,
        selected_artwork_count,
    })
}

pub(crate) fn row_to_selected_artwork(row: SqliteRow) -> Result<SelectedArtworkRecord> {
    Ok(SelectedArtworkRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        kind: image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?),
        artifact_id: parse_id(row_get::<String>(&row, "artifact_id")?)?,
        created_at: row_get(&row, "created_at")?,
        updated_at: row_get(&row, "updated_at")?,
    })
}

pub(crate) fn serialize_metadata_json(metadata: &CanonicalMetadata) -> Result<String> {
    serde_json::to_string(metadata).map_err(database_error)
}

pub(crate) fn metadata_field_from_str(value: &str) -> Result<MetadataField> {
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
        _ => Err(NakoError::Database {
            message: format!("unknown metadata field stored in database: {value}"),
        }),
    }
}

pub(crate) fn row_get<T>(row: &SqliteRow, column: &str) -> Result<T>
where
    for<'row> T: Decode<'row, Sqlite> + Type<Sqlite>,
{
    row.try_get(column).map_err(database_error)
}

pub(crate) fn database_error(error: impl Display) -> NakoError {
    NakoError::Database {
        message: error.to_string(),
    }
}
