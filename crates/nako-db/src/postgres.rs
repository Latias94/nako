use std::{borrow::Cow, fmt::Display, str::FromStr};

use sqlx::{
    Decode, PgPool, Postgres, Row, Type,
    migrate::{Migration, MigrationType, Migrator},
    postgres::{PgPoolOptions, PgRow},
};

use nako_core::*;
#[cfg(test)]
use sqlx::{Executor, postgres::PgConnectOptions};

mod addon_tasks;
mod addons_automation;
mod admin_settings;
mod core_catalog;
mod events;
mod identity;
mod import_state;
mod jobs;
mod managed_artwork;
mod metadata_candidate_review;
mod metadata_catalog;
mod playback_runtime;
mod renderer_runtime;
mod user_playlist;
mod vfs_health;
mod vfs_staging;

const POSTGRES_MAX_CONNECTIONS: u32 = 5;

const MIGRATIONS: &[(i64, &str, &str)] = &[(
    1,
    "baseline",
    include_str!("../migrations/postgres/baseline.sql"),
)];

#[derive(Clone, Debug)]
pub(crate) struct PostgresStore {
    pool: PgPool,
    #[cfg(test)]
    schema_name: Option<String>,
}

impl PostgresStore {
    pub async fn connect(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(POSTGRES_MAX_CONNECTIONS)
            .connect(database_url)
            .await
            .map_err(database_error)?;

        Ok(Self {
            pool,
            #[cfg(test)]
            schema_name: None,
        })
    }

    #[cfg(test)]
    pub async fn connect_with_schema(database_url: &str, schema_name: &str) -> Result<Self> {
        validate_schema_name(schema_name)?;
        let setup_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(database_url)
            .await
            .map_err(database_error)?;
        let create_schema = format!(r#"CREATE SCHEMA IF NOT EXISTS "{schema_name}""#);
        setup_pool
            .execute(create_schema.as_str())
            .await
            .map_err(database_error)?;
        setup_pool.close().await;

        let options = PgConnectOptions::from_str(database_url)
            .map_err(database_error)?
            .options([("search_path", schema_name)]);
        let pool = PgPoolOptions::new()
            .max_connections(POSTGRES_MAX_CONNECTIONS)
            .connect_with(options)
            .await
            .map_err(database_error)?;

        Ok(Self {
            pool,
            schema_name: Some(schema_name.to_owned()),
        })
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    #[cfg(test)]
    pub async fn drop_schema(&self) -> Result<()> {
        let Some(schema_name) = self.schema_name.as_deref() else {
            return Err(NakoError::InvalidInput {
                message: "PostgreSQL default runtime connection does not own an isolated schema"
                    .to_owned(),
            });
        };
        validate_schema_name(schema_name)?;
        let drop_schema = format!(r#"DROP SCHEMA IF EXISTS "{schema_name}" CASCADE"#);
        self.pool
            .execute(drop_schema.as_str())
            .await
            .map_err(database_error)?;
        self.pool.close().await;

        Ok(())
    }
}

#[async_trait::async_trait]
impl DatabaseLifecycle for PostgresStore {
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

fn row_get<'r, T>(row: &'r PgRow, column: &str) -> Result<T>
where
    T: Decode<'r, Postgres> + Type<Postgres>,
{
    row.try_get(column).map_err(database_error)
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

fn provider_subject_kind_to_parts(kind: &ProviderSubjectKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

fn provider_subject_kind_from_parts(kind: String, kind_key: String) -> ProviderSubjectKind {
    ProviderSubjectKind::from_parts(&kind, kind_key)
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
        _ => Err(NakoError::Database {
            message: format!(
                "unknown image owner kind stored in PostgreSQL database: {owner_kind}"
            ),
        }),
    }
}

fn metadata_source_to_parts(source: &MetadataSource) -> (String, String) {
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

fn metadata_source_from_parts(source: String, source_key: String) -> MetadataSource {
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

fn metadata_candidate_source_to_parts(source: &MetadataCandidateSource) -> (String, String) {
    match source {
        MetadataCandidateSource::Local => ("local".to_owned(), String::new()),
        MetadataCandidateSource::Nfo => ("nfo".to_owned(), String::new()),
        MetadataCandidateSource::User => ("user".to_owned(), String::new()),
        MetadataCandidateSource::Addon(addon_id) => ("addon".to_owned(), addon_id.to_string()),
        MetadataCandidateSource::Automation(provider_id) => {
            ("automation".to_owned(), provider_id.to_string())
        }
        MetadataCandidateSource::Provider(provider) => {
            let (provider, provider_key) = provider_to_parts(provider);
            (format!("provider:{provider}"), provider_key)
        }
        MetadataCandidateSource::Other(value) => ("other".to_owned(), value.clone()),
    }
}

fn metadata_candidate_source_from_parts(
    source: String,
    source_key: String,
) -> MetadataCandidateSource {
    match source.as_str() {
        "local" => MetadataCandidateSource::Local,
        "nfo" => MetadataCandidateSource::Nfo,
        "user" => MetadataCandidateSource::User,
        "addon" => parse_id(source_key)
            .map(MetadataCandidateSource::Addon)
            .unwrap_or_else(|_| MetadataCandidateSource::Other(source)),
        "automation" => parse_id(source_key)
            .map(MetadataCandidateSource::Automation)
            .unwrap_or_else(|_| MetadataCandidateSource::Other(source)),
        "other" => MetadataCandidateSource::Other(source_key),
        value if value.starts_with("provider:") => {
            let provider = value.trim_start_matches("provider:").to_owned();
            MetadataCandidateSource::Provider(provider_from_parts(provider, source_key))
        }
        _ => MetadataCandidateSource::Other(source),
    }
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
        _ => Err(NakoError::Database {
            message: format!("unknown metadata field stored in PostgreSQL database: {value}"),
        }),
    }
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

pub(crate) fn u64_to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|err| NakoError::Database {
        message: format!("value does not fit into PostgreSQL bigint: {err}"),
    })
}

fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>> {
    value.map(u64_to_i64).transpose()
}

pub(crate) fn i64_to_u64(value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|err| NakoError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u64: {err}"),
    })
}

pub(crate) fn optional_i64_to_u64(value: Option<i64>) -> Result<Option<u64>> {
    value.map(i64_to_u64).transpose()
}

fn i64_to_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|err| NakoError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u32: {err}"),
    })
}

fn i64_to_u16(value: i64) -> Result<u16> {
    u16::try_from(value).map_err(|err| NakoError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u16: {err}"),
    })
}

fn optional_i64_to_u32(value: Option<i64>) -> Result<Option<u32>> {
    value.map(i64_to_u32).transpose()
}

fn optional_i64_to_u16(value: Option<i64>) -> Result<Option<u16>> {
    value
        .map(|value| {
            u16::try_from(value).map_err(|err| NakoError::Database {
                message: format!("PostgreSQL bigint cannot be converted to u16: {err}"),
            })
        })
        .transpose()
}

fn optional_i64_to_i32(value: Option<i64>) -> Result<Option<i32>> {
    value
        .map(|value| {
            i32::try_from(value).map_err(|err| NakoError::Database {
                message: format!("PostgreSQL bigint cannot be converted to i32: {err}"),
            })
        })
        .transpose()
}

#[cfg(test)]
fn validate_schema_name(schema_name: &str) -> Result<()> {
    let valid = !schema_name.is_empty()
        && schema_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !valid {
        return Err(NakoError::InvalidInput {
            message: "PostgreSQL contract schema name must contain only lowercase ASCII letters, digits, and underscores".to_owned(),
        });
    }

    Ok(())
}

fn database_error<E: Display>(err: E) -> NakoError {
    NakoError::Database {
        message: err.to_string(),
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

    #[test]
    fn postgres_baseline_migration_contains_identity_and_library_access_schema() {
        assert_eq!(MIGRATIONS.len(), 1);
        assert_eq!(MIGRATIONS[0].0, 1);
        assert_eq!(MIGRATIONS[0].1, "baseline");

        let sql = MIGRATIONS[0].2;
        for expected in [
            "CREATE TABLE users",
            "CREATE TABLE user_role_assignments",
            "CREATE TABLE user_library_access_policies",
            "CREATE TABLE role_library_access_policies",
        ] {
            assert!(
                sql.contains(expected),
                "missing PostgreSQL baseline SQL: {expected}"
            );
        }
    }

    #[test]
    fn postgres_baseline_migration_describes_direct_schema_shape() {
        let sql = MIGRATIONS[0].2;
        for fragment in HISTORICAL_REPLAY_FRAGMENTS {
            assert!(
                !sql.contains(fragment),
                "PostgreSQL baseline still contains historical replay fragment: {fragment}"
            );
        }
    }
}
