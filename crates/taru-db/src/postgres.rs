use std::{borrow::Cow, fmt::Display, path::PathBuf, str::FromStr};

use sqlx::{
    Decode, PgPool, Postgres, Row, Type,
    migrate::{Migration, MigrationType, Migrator},
    postgres::{PgPoolOptions, PgRow},
};

#[cfg(test)]
use sqlx::{Executor, postgres::PgConnectOptions};
use taru_core::*;
use taru_search::{
    SearchDocument, SearchEvaluationDocument, SearchHit, SearchIndex, SearchQuery,
    evaluate_search_documents,
};

const POSTGRES_MAX_CONNECTIONS: u32 = 5;

const MIGRATIONS: &[(i64, &str, &str)] = &[
    (
        1,
        "contract jobs",
        include_str!("../migrations/postgres/0001_contract_jobs.sql"),
    ),
    (
        2,
        "managed artwork",
        include_str!("../migrations/postgres/0002_managed_artwork.sql"),
    ),
    (
        3,
        "managed import artifacts",
        include_str!("../migrations/postgres/0003_managed_import_artifacts.sql"),
    ),
];

const JOB_SELECT: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM jobs
            "#;

const JOB_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM jobs
            WHERE id = $1
            "#;

const JOB_LEASE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                worker_id::text AS worker_id,
                run_token::text AS run_token,
                to_char(heartbeat_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS heartbeat_at,
                to_char(lease_expires_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS lease_expires_at,
                to_char(cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at,
                cancel_reason
            FROM jobs
            WHERE id = $1
            "#;

const MEDIA_ITEM_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                parent_id::text AS parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json::text AS metadata_json
            FROM media_items
            WHERE id = $1
            "#;

const MEDIA_SOURCE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE id = $1
            "#;

const LOCAL_INFERENCE_EVIDENCE_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                inferred_kind,
                inferred_title,
                inferred_year,
                inferred_season,
                inferred_episode,
                confidence_milli,
                evidence_source,
                evidence_source_key,
                evidence_value,
                inference_version
            FROM local_inference_evidence
            WHERE id = $1
            "#;

const PROVIDER_SUBJECT_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE id = $1
            "#;

const SOURCE_DUPLICATE_RELATIONSHIP_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                duplicate_source_id::text AS duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE id = $1
            "#;

const USER_PLAYBACK_STATE_SELECT: &str = r#"
            SELECT
                principal_id,
                item_id::text AS item_id,
                source_id::text AS source_id,
                resume_position_ms,
                duration_ms,
                watched,
                watched_at_ms,
                last_played_at_ms,
                updated_at_ms,
                version
            FROM user_playback_states
            "#;

const TRANSCODE_SESSION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                kind,
                request_key,
                output_path,
                state,
                failure_category,
                failure_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM transcode_sessions
            "#;

const TRANSCODE_SESSION_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                kind,
                request_key,
                output_path,
                state,
                failure_category,
                failure_message,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at
            FROM transcode_sessions
            WHERE id = $1
            "#;

const OUTBOX_EVENT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                kind,
                subject_kind,
                subject_id::text AS subject_id,
                library_id::text AS library_id,
                source_id::text AS source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS occurred_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                next_attempt_at
            FROM event_outbox
            "#;

const OUTBOX_EVENT_SELECT_BY_ID: &str = r#"
            SELECT
                id::text AS id,
                kind,
                subject_kind,
                subject_id::text AS subject_id,
                library_id::text AS library_id,
                source_id::text AS source_id,
                idempotency_key,
                payload_json,
                status,
                attempts,
                last_error,
                to_char(occurred_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS occurred_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                next_attempt_at
            FROM event_outbox
            WHERE id = $1
            "#;

const WEBHOOK_ENDPOINT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json::text AS subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM webhook_endpoints
            "#;

const WEBHOOK_DELIVERY_ATTEMPT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                endpoint_id::text AS endpoint_id,
                event_id::text AS event_id,
                attempt_number,
                status,
                http_status,
                error,
                to_char(requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS requested_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                next_retry_at
            FROM webhook_delivery_attempts
            "#;

const AUTOMATION_PROVIDER_SELECT: &str = r#"
            SELECT
                id::text AS id,
                name,
                base_url,
                secret_env,
                capabilities_json::text AS capabilities_json,
                timeout_ms,
                max_attempts,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM automation_providers
            "#;

const AUTOMATION_ARTIFACT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                job_id::text AS job_id,
                provider_id::text AS provider_id,
                capability,
                kind,
                library_id::text AS library_id,
                item_id::text AS item_id,
                source_id::text AS source_id,
                artifact_json,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                to_char(accepted_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS accepted_at
            FROM automation_artifacts
            "#;

const ADDON_REGISTRATION_SELECT: &str = r#"
            SELECT
                id::text AS id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json::text AS granted_scopes_json,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_registrations
            "#;

const ADDON_TOKEN_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                label,
                token_prefix,
                token_hash,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(rotated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS rotated_at,
                to_char(revoked_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS revoked_at,
                to_char(last_used_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS last_used_at
            FROM addon_tokens
            "#;

const ADDON_SIDE_EFFECT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                token_id::text AS token_id,
                permission,
                library_id::text AS library_id,
                target_kind,
                target_id,
                idempotency_key,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code,
                apply_status,
                apply_error_code,
                applied_item_id::text AS applied_item_id,
                applied_source,
                apply_report_json,
                to_char(applied_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS applied_at,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
            FROM addon_side_effects
            "#;

const ARTWORK_CANDIDATE_SELECT: &str = r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                side_effect_id::text AS side_effect_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                source_kind,
                source_uri,
                width,
                height,
                language,
                status,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM addon_artwork_candidates
            "#;

const ARTWORK_TASK_SELECT: &str = r#"
            SELECT
                id::text AS id,
                image_id::text AS image_id,
                kind,
                status,
                resource_class,
                attempts,
                max_attempts,
                error
            FROM artwork_tasks
            "#;

const MANAGED_ARTWORK_INGEST_SELECT: &str = r#"
            SELECT
                id::text AS id,
                candidate_id::text AS candidate_id,
                job_id::text AS job_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                status,
                artifact_id::text AS artifact_id,
                failure_code,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM managed_artwork_ingests
            "#;

const MANAGED_ARTWORK_ARTIFACT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                ingest_id::text AS ingest_id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                storage_uri,
                content_hash,
                width,
                height,
                byte_len,
                media_type,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM managed_artwork_artifacts
            WHERE deleted_at IS NULL
            "#;

const SELECTED_ARTWORK_SELECT: &str = r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                kind,
                kind_key,
                artifact_id::text AS artifact_id,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
            FROM selected_artworks
            "#;

const VFS_CACHE_OBJECT_SELECT: &str = r#"
            SELECT
                uri,
                scheme,
                kind,
                len,
                modified_at,
                etag,
                fingerprint,
                capabilities_bits,
                fetched_at_ms,
                fresh_until_ms
            FROM vfs_cache_objects
            "#;

const STAGING_MANIFEST_RECORD_SELECT: &str = r#"
            SELECT
                id::text AS id,
                source_uri,
                source_scheme,
                purpose,
                local_path,
                size_bytes,
                etag,
                fingerprint,
                state,
                created_at_ms,
                updated_at_ms,
                last_accessed_at_ms,
                expires_at_ms,
                active_leases,
                validation_error
            FROM staging_manifest_records
            "#;

const MANAGED_IMPORT_ARTIFACT_SELECT: &str = r#"
            SELECT
                id::text AS id,
                target_library_id::text AS target_library_id,
                source_kind,
                source_kind_key,
                source_uri,
                staging_manifest_id::text AS staging_manifest_id,
                artifact_uri,
                original_file_name,
                intended_locator,
                size_bytes,
                fingerprint,
                state,
                diagnostics_json,
                created_at_ms,
                updated_at_ms
            FROM managed_import_artifacts
            "#;

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
            return Err(TaruError::InvalidInput {
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

    async fn get_job_or_not_found(&self, id: JobId) -> Result<Job> {
        self.get_job(id).await?.ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
    }

    async fn get_leased_job(&self, id: JobId) -> Result<LeasedJob> {
        let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_leased_job)
            .transpose()?
            .ok_or_else(|| TaruError::NotFound {
                entity: "job",
                id: id.to_string(),
            })
    }

    async fn get_cancellation_request_record(
        &self,
        id: JobId,
        terminal: bool,
    ) -> Result<JobCancellationRequestRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                kind,
                status,
                resource_class,
                library_id::text AS library_id,
                source_id::text AS source_id,
                input_json,
                summary_json,
                error,
                to_char(queued_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS queued_at,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                to_char(cancel_requested_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS cancel_requested_at
            FROM jobs
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Err(TaruError::NotFound {
                entity: "job",
                id: id.to_string(),
            });
        };
        let cancel_requested_at = row_get(&row, "cancel_requested_at")?;
        let job = row_to_job(row)?;

        Ok(JobCancellationRequestRecord {
            job,
            requested: true,
            terminal,
            cancel_requested_at,
        })
    }
}

#[async_trait::async_trait]
impl DatabaseLifecycle for PostgresStore {
    async fn migrate(&self) -> Result<()> {
        migrator().run(self.pool()).await.map_err(database_error)
    }
}

#[async_trait::async_trait]
impl LibraryRepository for PostgresStore {
    async fn upsert_library(&self, library: &Library) -> Result<()> {
        let roots_json = serde_json::to_string(&library.roots).map_err(database_error)?;
        let options_json = serde_json::to_string(&library.options).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO libraries (
                id, name, roots_json, options_json, domain, preset
            )
            VALUES ($1, $2, $3::jsonb, $4::jsonb, $5, $6)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                roots_json = excluded.roots_json,
                options_json = excluded.options_json,
                domain = excluded.domain,
                preset = excluded.preset,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(library.id.as_uuid())
        .bind(&library.name)
        .bind(roots_json)
        .bind(options_json)
        .bind(library.options.domain.as_str())
        .bind(library.options.preset.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_library(&self, id: LibraryId) -> Result<Option<Library>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_library).transpose()
    }

    async fn list_libraries(&self, page: PageRequest) -> Result<Vec<Library>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                name,
                roots_json::text AS roots_json,
                options_json::text AS options_json
            FROM libraries
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
impl JobRepository for PostgresStore {
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
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(job.id.as_uuid())
        .bind(job.kind.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(job.resource_class)
        .bind(job.library_id.map(|id| id.as_uuid()))
        .bind(job.source_id.map(|id| id.as_uuid()))
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
                status = $2,
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                status = $2,
                summary_json = $3,
                error = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                status = $2,
                error = $3,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_job_or_not_found(id).await
    }

    async fn fail_unfinished_jobs(&self, error: String) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $1,
                error = $2,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE status = $3
                AND kind <> $4
            "#,
        )
        .bind(JobStatus::Failed.as_str())
        .bind(error)
        .bind(JobStatus::Running.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }

    async fn get_job(&self, id: JobId) -> Result<Option<Job>> {
        let row = sqlx::query(JOB_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_job).transpose()
    }

    async fn list_jobs(&self, filter: JobListFilter, page: PageRequest) -> Result<Vec<Job>> {
        let page = page.clamped();
        let status = filter.status.map(|status| status.as_str().to_owned());
        let kind = filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = filter.resource_class;
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let source_id = filter.source_id.map(|id| id.as_uuid());
        let rows = sqlx::query(&format!(
            r#"
            {JOB_SELECT}
            WHERE ($1::text IS NULL OR status = $1)
                AND ($2::text IS NULL OR kind = $2)
                AND ($3::text IS NULL OR resource_class = $3)
                AND ($4::uuid IS NULL OR library_id = $4)
                AND ($5::uuid IS NULL OR source_id = $5)
            ORDER BY queued_at DESC, id DESC
            LIMIT $6 OFFSET $7
            "#
        ))
        .bind(status.as_deref())
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(library_id)
        .bind(source_id)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_job).collect()
    }
}

#[async_trait::async_trait]
impl EventOutboxRepository for PostgresStore {
    async fn enqueue_outbox_event(&self, event: NewOutboxEvent) -> Result<OutboxEventRecord> {
        let subject_kind = event.subject.kind();
        let subject_id = event.subject.id();

        sqlx::query(
            r#"
            INSERT INTO event_outbox (
                id,
                kind,
                subject_kind,
                subject_id,
                library_id,
                source_id,
                idempotency_key,
                payload_json,
                status
            )
            VALUES ($1, $2, $3, $4::uuid, $5, $6, $7, $8, $9)
            ON CONFLICT(kind, idempotency_key) DO NOTHING
            "#,
        )
        .bind(event.id.as_uuid())
        .bind(event.kind.as_str())
        .bind(subject_kind)
        .bind(subject_id)
        .bind(event.library_id.map(|id| id.as_uuid()))
        .bind(event.source_id.map(|id| id.as_uuid()))
        .bind(&event.idempotency_key)
        .bind(&event.payload_json)
        .bind(OutboxEventStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_outbox_event_by_idempotency_key(event.kind, &event.idempotency_key)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "outbox event was not found after enqueue for key {}",
                    event.idempotency_key
                ),
            })
    }

    async fn get_outbox_event(&self, id: taru_core::EventId) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(OUTBOX_EVENT_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn find_outbox_event_by_idempotency_key(
        &self,
        kind: DomainEventKind,
        idempotency_key: &str,
    ) -> Result<Option<OutboxEventRecord>> {
        let row = sqlx::query(&format!(
            "{OUTBOX_EVENT_SELECT} WHERE kind = $1 AND idempotency_key = $2"
        ))
        .bind(kind.as_str())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_outbox_event).transpose()
    }

    async fn list_outbox_events(
        &self,
        filter: OutboxEventListFilter,
        page: PageRequest,
    ) -> Result<Vec<OutboxEventRecord>> {
        let page = page.clamped();
        let kind = filter.kind.map(|kind| kind.as_str().to_owned());
        let status = filter.status.map(|status| status.as_str().to_owned());
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let source_id = filter.source_id.map(|id| id.as_uuid());
        let rows = sqlx::query(&format!(
            r#"
            {OUTBOX_EVENT_SELECT}
            WHERE ($1::text IS NULL OR kind = $1)
                AND ($2::text IS NULL OR status = $2)
                AND ($3::uuid IS NULL OR library_id = $3)
                AND ($4::uuid IS NULL OR source_id = $4)
            ORDER BY occurred_at DESC, id DESC
            LIMIT $5 OFFSET $6
            "#
        ))
        .bind(kind.as_deref())
        .bind(status.as_deref())
        .bind(library_id)
        .bind(source_id)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_outbox_event).collect()
    }
}

#[async_trait::async_trait]
impl AddonRepository for PostgresStore {
    async fn upsert_addon_registration(
        &self,
        addon: NewAddonRegistration,
    ) -> Result<AddonRegistrationRecord> {
        let granted_scopes_json =
            serde_json::to_string(&addon.granted_scopes).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO addon_registrations (
                id,
                manifest_id,
                name,
                version,
                protocol_version,
                base_url,
                manifest_json,
                granted_scopes_json,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8::jsonb, $9)
            ON CONFLICT(id) DO UPDATE SET
                manifest_id = excluded.manifest_id,
                name = excluded.name,
                version = excluded.version,
                protocol_version = excluded.protocol_version,
                base_url = excluded.base_url,
                manifest_json = excluded.manifest_json,
                granted_scopes_json = excluded.granted_scopes_json,
                status = excluded.status,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(addon.id.as_uuid())
        .bind(&addon.manifest_id)
        .bind(&addon.name)
        .bind(&addon.version)
        .bind(&addon.protocol_version)
        .bind(&addon.base_url)
        .bind(&addon.manifest_json)
        .bind(granted_scopes_json)
        .bind(addon.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_registration(addon.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon registration {} was not found after upsert", addon.id),
            })
    }

    async fn get_addon_registration(&self, id: AddonId) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(&format!("{ADDON_REGISTRATION_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn find_addon_registration_by_manifest_id(
        &self,
        manifest_id: &str,
    ) -> Result<Option<AddonRegistrationRecord>> {
        let row = sqlx::query(&format!(
            "{ADDON_REGISTRATION_SELECT} WHERE manifest_id = $1"
        ))
        .bind(manifest_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_registration).transpose()
    }

    async fn list_addon_registrations(
        &self,
        status: Option<AddonStatus>,
    ) -> Result<Vec<AddonRegistrationRecord>> {
        let status = status.map(|status| status.as_str().to_owned());
        let rows = sqlx::query(&format!(
            r#"
            {ADDON_REGISTRATION_SELECT}
            WHERE ($1::text IS NULL OR status = $1)
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(status.as_deref())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_registration).collect()
    }

    async fn create_addon_token(&self, token: NewAddonToken) -> Result<AddonTokenRecord> {
        sqlx::query(
            r#"
            INSERT INTO addon_tokens (
                id,
                addon_id,
                label,
                token_prefix,
                token_hash,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(token.id.as_uuid())
        .bind(token.addon_id.as_uuid())
        .bind(&token.label)
        .bind(&token.token_prefix)
        .bind(&token.token_hash)
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_token(token.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon token {} was not found after create", token.id),
            })
    }

    async fn get_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let row = sqlx::query(&format!("{ADDON_TOKEN_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn find_addon_token_by_hash(&self, token_hash: &str) -> Result<Option<AddonTokenRecord>> {
        let row = sqlx::query(&format!("{ADDON_TOKEN_SELECT} WHERE token_hash = $1"))
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_addon_token).transpose()
    }

    async fn list_addon_tokens(&self, addon_id: AddonId) -> Result<Vec<AddonTokenRecord>> {
        let rows = sqlx::query(&format!(
            "{ADDON_TOKEN_SELECT} WHERE addon_id = $1 ORDER BY created_at ASC, id ASC"
        ))
        .bind(addon_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_token).collect()
    }

    async fn mark_addon_token_used(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        sqlx::query(
            r#"
            UPDATE addon_tokens
            SET last_used_at = statement_timestamp()
            WHERE id = $1 AND status = $2
            "#,
        )
        .bind(id.as_uuid())
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_addon_token(id).await
    }

    async fn rotate_addon_token(
        &self,
        rotated_token_id: AddonTokenId,
        new_token: NewAddonToken,
    ) -> Result<(AddonTokenRecord, AddonTokenRecord)> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let rotate_result = sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = $2,
                rotated_at = statement_timestamp()
            WHERE id = $1 AND status = $3 AND addon_id = $4
            "#,
        )
        .bind(rotated_token_id.as_uuid())
        .bind(AddonTokenStatus::Rotated.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .bind(new_token.addon_id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if rotate_result.rows_affected() == 0 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: format!("addon token {rotated_token_id} is not active"),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO addon_tokens (
                id,
                addon_id,
                label,
                token_prefix,
                token_hash,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(new_token.id.as_uuid())
        .bind(new_token.addon_id.as_uuid())
        .bind(&new_token.label)
        .bind(&new_token.token_prefix)
        .bind(&new_token.token_hash)
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        transaction.commit().await.map_err(database_error)?;

        let rotated = self
            .get_addon_token(rotated_token_id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!("addon token {rotated_token_id} was not found after rotate"),
            })?;
        let created =
            self.get_addon_token(new_token.id)
                .await?
                .ok_or_else(|| TaruError::Database {
                    message: format!("addon token {} was not found after rotate", new_token.id),
                })?;

        Ok((rotated, created))
    }

    async fn revoke_addon_token(&self, id: AddonTokenId) -> Result<Option<AddonTokenRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE addon_tokens
            SET
                status = $2,
                revoked_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(id.as_uuid())
        .bind(AddonTokenStatus::Revoked.as_str())
        .bind(AddonTokenStatus::Active.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return self.get_addon_token(id).await;
        }

        self.get_addon_token(id).await
    }

    async fn replace_addon_grants(
        &self,
        addon_id: AddonId,
        grants: Vec<NewAddonGrant>,
    ) -> Result<Vec<AddonGrantRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query("DELETE FROM addon_grants WHERE addon_id = $1")
            .bind(addon_id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for grant in grants {
            sqlx::query(
                r#"
                INSERT INTO addon_grants (
                    id,
                    addon_id,
                    permission,
                    library_id
                )
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(grant.id.as_uuid())
            .bind(grant.addon_id.as_uuid())
            .bind(grant.permission.as_str())
            .bind(grant.library_id.map(|id| id.as_uuid()))
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;
        self.list_addon_grants(addon_id).await
    }

    async fn list_addon_grants(&self, addon_id: AddonId) -> Result<Vec<AddonGrantRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                addon_id::text AS addon_id,
                permission,
                library_id::text AS library_id,
                to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at
            FROM addon_grants
            WHERE addon_id = $1
            ORDER BY permission ASC, library_id ASC, created_at ASC, id ASC
            "#,
        )
        .bind(addon_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_addon_grant).collect()
    }

    async fn create_addon_side_effect(
        &self,
        side_effect: NewAddonSideEffect,
    ) -> Result<AddonSideEffectRecord> {
        sqlx::query(
            r#"
            INSERT INTO addon_side_effects (
                id,
                addon_id,
                token_id,
                permission,
                library_id,
                target_kind,
                target_id,
                idempotency_key,
                provenance_json,
                payload_json,
                validation_status,
                safe_error_code
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT(addon_id, idempotency_key) DO NOTHING
            "#,
        )
        .bind(side_effect.id.as_uuid())
        .bind(side_effect.addon_id.as_uuid())
        .bind(side_effect.token_id.as_uuid())
        .bind(side_effect.permission.as_str())
        .bind(side_effect.library_id.as_uuid())
        .bind(side_effect.target.kind.as_str())
        .bind(&side_effect.target.id)
        .bind(&side_effect.idempotency_key)
        .bind(&side_effect.provenance_json)
        .bind(&side_effect.payload_json)
        .bind(side_effect.validation_status.as_str())
        .bind(&side_effect.safe_error_code)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_addon_side_effect_by_idempotency_key(
            side_effect.addon_id,
            &side_effect.idempotency_key,
        )
        .await?
        .ok_or_else(|| TaruError::Database {
            message: format!(
                "addon side effect {} was not found after create",
                side_effect.id
            ),
        })
    }

    async fn find_addon_side_effect_by_idempotency_key(
        &self,
        addon_id: AddonId,
        idempotency_key: &str,
    ) -> Result<Option<AddonSideEffectRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {ADDON_SIDE_EFFECT_SELECT}
            WHERE addon_id = $1 AND idempotency_key = $2
            ORDER BY created_at ASC, id ASC
            LIMIT 1
            "#
        ))
        .bind(addon_id.as_uuid())
        .bind(idempotency_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_addon_side_effect).transpose()
    }

    async fn set_addon_side_effect_apply_outcome(
        &self,
        id: AddonSideEffectId,
        outcome: AddonSideEffectApplyOutcome,
    ) -> Result<AddonSideEffectRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let record = set_addon_side_effect_apply_outcome_tx(&mut transaction, id, &outcome).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(record)
    }
}

async fn set_addon_side_effect_apply_outcome_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: AddonSideEffectId,
    outcome: &AddonSideEffectApplyOutcome,
) -> Result<AddonSideEffectRecord> {
    sqlx::query(
        r#"
        UPDATE addon_side_effects
        SET
            apply_status = $2,
            apply_error_code = $3,
            applied_item_id = $4,
            applied_source = $5,
            apply_report_json = $6,
            applied_at = CASE
                WHEN $2 = 'applied' THEN statement_timestamp()
                ELSE applied_at
            END
        WHERE id = $1
        "#,
    )
    .bind(id.as_uuid())
    .bind(outcome.status.as_str())
    .bind(&outcome.error_code)
    .bind(outcome.item_id.map(|id| id.as_uuid()))
    .bind(&outcome.source)
    .bind(&outcome.report_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    let row = sqlx::query(&format!("{ADDON_SIDE_EFFECT_SELECT} WHERE id = $1 LIMIT 1"))
        .bind(id.as_uuid())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_addon_side_effect)
        .transpose()?
        .ok_or_else(|| TaruError::NotFound {
            entity: "addon_side_effect",
            id: id.to_string(),
        })
}

#[async_trait::async_trait]
impl AutomationRepository for PostgresStore {
    async fn upsert_automation_provider(
        &self,
        provider: NewAutomationProviderConfig,
    ) -> Result<AutomationProviderConfigRecord> {
        let capabilities_json =
            serde_json::to_string(&provider.capabilities).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO automation_providers (
                id,
                name,
                base_url,
                secret_env,
                capabilities_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                base_url = excluded.base_url,
                secret_env = excluded.secret_env,
                capabilities_json = excluded.capabilities_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(provider.id.as_uuid())
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(&provider.secret_env)
        .bind(capabilities_json)
        .bind(u64_to_i64(provider.timeout_ms)?)
        .bind(u32_to_i64(provider.max_attempts))
        .bind(provider.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_provider(provider.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "automation provider {} was not found after upsert",
                    provider.id
                ),
            })
    }

    async fn get_automation_provider(
        &self,
        id: AutomationProviderId,
    ) -> Result<Option<AutomationProviderConfigRecord>> {
        let row = sqlx::query(&format!("{AUTOMATION_PROVIDER_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_automation_provider).transpose()
    }

    async fn list_enabled_automation_providers(
        &self,
    ) -> Result<Vec<AutomationProviderConfigRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_PROVIDER_SELECT}
            WHERE status = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(AutomationProviderStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_provider).collect()
    }

    async fn create_automation_artifact(
        &self,
        artifact: NewAutomationArtifact,
    ) -> Result<AutomationArtifactRecord> {
        sqlx::query(
            r#"
            INSERT INTO automation_artifacts (
                id,
                job_id,
                provider_id,
                capability,
                kind,
                library_id,
                item_id,
                source_id,
                artifact_json,
                status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(artifact.id.as_uuid())
        .bind(artifact.job_id.as_uuid())
        .bind(artifact.provider_id.as_uuid())
        .bind(artifact.capability.as_str())
        .bind(artifact.kind.as_str())
        .bind(artifact.library_id.map(|id| id.as_uuid()))
        .bind(artifact.item_id.map(|id| id.as_uuid()))
        .bind(artifact.source_id.map(|id| id.as_uuid()))
        .bind(artifact.artifact_json)
        .bind(AutomationArtifactStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(artifact.id).await
    }

    async fn set_automation_artifact_status(
        &self,
        id: AutomationArtifactId,
        status: AutomationArtifactStatus,
    ) -> Result<AutomationArtifactRecord> {
        let query = if status == AutomationArtifactStatus::Accepted {
            r#"
            UPDATE automation_artifacts
            SET
                status = $2,
                accepted_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#
        } else {
            r#"
            UPDATE automation_artifacts
            SET
                status = $2,
                accepted_at = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#
        };

        sqlx::query(query)
            .bind(id.as_uuid())
            .bind(status.as_str())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        self.get_automation_artifact_or_not_found(id).await
    }

    async fn list_automation_artifacts_for_job(
        &self,
        job_id: JobId,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_ARTIFACT_SELECT}
            WHERE job_id = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(job_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }

    async fn list_automation_artifacts_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<AutomationArtifactRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {AUTOMATION_ARTIFACT_SELECT}
            WHERE item_id = $1
            ORDER BY created_at DESC, id DESC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_automation_artifact).collect()
    }
}

#[async_trait::async_trait]
impl WebhookRepository for PostgresStore {
    async fn upsert_webhook_endpoint(
        &self,
        endpoint: NewWebhookEndpoint,
    ) -> Result<WebhookEndpointRecord> {
        let event_kinds_json =
            serde_json::to_string(&endpoint.subscribed_event_kinds).map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO webhook_endpoints (
                id,
                name,
                url,
                secret_env,
                subscribed_event_kinds_json,
                timeout_ms,
                max_attempts,
                status
            )
            VALUES ($1, $2, $3, $4, $5::jsonb, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                url = excluded.url,
                secret_env = excluded.secret_env,
                subscribed_event_kinds_json = excluded.subscribed_event_kinds_json,
                timeout_ms = excluded.timeout_ms,
                max_attempts = excluded.max_attempts,
                status = excluded.status,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(endpoint.id.as_uuid())
        .bind(&endpoint.name)
        .bind(&endpoint.url)
        .bind(&endpoint.secret_env)
        .bind(event_kinds_json)
        .bind(u64_to_i64(endpoint.timeout_ms)?)
        .bind(u32_to_i64(endpoint.max_attempts))
        .bind(endpoint.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_endpoint(endpoint.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "webhook endpoint {} was not found after upsert",
                    endpoint.id
                ),
            })
    }

    async fn get_webhook_endpoint(
        &self,
        id: WebhookEndpointId,
    ) -> Result<Option<WebhookEndpointRecord>> {
        let row = sqlx::query(&format!("{WEBHOOK_ENDPOINT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_webhook_endpoint).transpose()
    }

    async fn list_enabled_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {WEBHOOK_ENDPOINT_SELECT}
            WHERE status = $1
            ORDER BY created_at ASC, id ASC
            "#
        ))
        .bind(WebhookEndpointStatus::Enabled.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_webhook_endpoint).collect()
    }

    async fn create_webhook_delivery_attempt(
        &self,
        attempt: NewWebhookDeliveryAttempt,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            INSERT INTO webhook_delivery_attempts (
                id,
                endpoint_id,
                event_id,
                attempt_number,
                status
            )
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(attempt.id.as_uuid())
        .bind(attempt.endpoint_id.as_uuid())
        .bind(attempt.event_id.as_uuid())
        .bind(u32_to_i64(attempt.attempt_number))
        .bind(WebhookDeliveryStatus::Pending.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(attempt.id)
            .await
    }

    async fn set_webhook_delivery_attempt_result(
        &self,
        id: WebhookDeliveryAttemptId,
        status: WebhookDeliveryStatus,
        http_status: Option<u16>,
        error: Option<String>,
        next_retry_at: Option<String>,
    ) -> Result<WebhookDeliveryAttemptRecord> {
        sqlx::query(
            r#"
            UPDATE webhook_delivery_attempts
            SET
                status = $2,
                http_status = $3,
                error = $4,
                completed_at = statement_timestamp(),
                next_retry_at = $5
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(status.as_str())
        .bind(http_status.map(i64::from))
        .bind(error)
        .bind(next_retry_at)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_webhook_delivery_attempt_or_not_found(id).await
    }

    async fn list_webhook_delivery_attempts(
        &self,
        event_id: taru_core::EventId,
    ) -> Result<Vec<WebhookDeliveryAttemptRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {WEBHOOK_DELIVERY_ATTEMPT_SELECT}
            WHERE event_id = $1
            ORDER BY attempt_number ASC, requested_at ASC, id ASC
            "#
        ))
        .bind(event_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_webhook_delivery_attempt)
            .collect()
    }
}

#[async_trait::async_trait]
impl LibraryItemRepository for PostgresStore {
    async fn upsert_library_item_state(&self, state: &LibraryItemState) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_library_item_state_tx(&mut transaction, state).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_library_item_state(
        &self,
        library_id: LibraryId,
        item_id: MediaItemId,
    ) -> Result<Option<LibraryItemState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                item_id::text AS item_id,
                provisional
            FROM library_item_states
            WHERE library_id = $1 AND item_id = $2
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(item_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_library_item_state).transpose()
    }

    async fn list_library_item_states_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<LibraryItemState>> {
        let rows = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                item_id::text AS item_id,
                provisional
            FROM library_item_states
            WHERE item_id = $1
            ORDER BY library_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_library_item_state).collect()
    }

    async fn find_library_item_by_kind_parent_title(
        &self,
        library_id: LibraryId,
        kind: MediaKind,
        parent_id: Option<MediaItemId>,
        title: &str,
    ) -> Result<Option<MediaItem>> {
        let row = sqlx::query(
            r#"
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json
            FROM media_items
            INNER JOIN library_item_states
                ON library_item_states.item_id = media_items.id
            WHERE library_item_states.library_id = $1
              AND media_items.kind = $2
              AND (
                  ($3::uuid IS NULL AND media_items.parent_id IS NULL)
                  OR media_items.parent_id = $3
              )
              AND media_items.title = $4
            ORDER BY media_items.id ASC
            LIMIT 1
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(kind.as_str())
        .bind(parent_id.map(|id| id.as_uuid()))
        .bind(title)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let id = parse_id(row_get::<String>(&row, "id")?)?;
        let external_ids = self.list_external_ids(id).await?;

        row_to_media_item(row, external_ids).map(Some)
    }
}

#[async_trait::async_trait]
impl MediaRepository for PostgresStore {
    async fn upsert_media_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_media_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_item(&self, id: MediaItemId) -> Result<Option<MediaItem>> {
        let row = sqlx::query(MEDIA_ITEM_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self.list_external_ids(id).await?;

        row_to_media_item(row, external_ids).map(Some)
    }

    async fn list_media_items(&self, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                kind,
                parent_id::text AS parent_id,
                title,
                original_title,
                sort_title,
                overview,
                release_date,
                metadata_json::text AS metadata_json
            FROM media_items
            ORDER BY title ASC, id ASC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn list_media_items_for_library(
        &self,
        library_id: LibraryId,
        page: PageRequest,
    ) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json
            FROM media_items
            WHERE media_items.id IN (
                SELECT item_id FROM media_sources WHERE library_id = $1
                UNION
                SELECT item_id FROM library_item_states WHERE library_id = $1
            )
            ORDER BY media_items.title ASC, media_items.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_media_source(&self, source: &MediaSource) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_media_source_tx(&mut transaction, source).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_media_source(&self, id: MediaSourceId) -> Result<Option<MediaSource>> {
        let row = sqlx::query(MEDIA_SOURCE_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_media_source).transpose()
    }

    async fn get_media_source_by_locator(
        &self,
        library_id: LibraryId,
        locator: &str,
    ) -> Result<Option<MediaSource>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE library_id = $1 AND locator = $2
            "#,
        )
        .bind(library_id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE item_id = $1
            ORDER BY locator ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                item_id::text AS item_id,
                locator,
                file_name,
                size_bytes,
                fingerprint
            FROM media_sources
            WHERE library_id = $1
            ORDER BY locator ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_media_source).collect()
    }
}

#[async_trait::async_trait]
impl MediaProbeRepository for PostgresStore {
    async fn upsert_media_probe(
        &self,
        source_id: MediaSourceId,
        result: &MediaProbeResult,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        sqlx::query(
            r#"
            INSERT INTO media_source_probes (source_id, duration_ms, container, bit_rate)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(source_id) DO UPDATE SET
                duration_ms = excluded.duration_ms,
                container = excluded.container,
                bit_rate = excluded.bit_rate,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(result.duration_ms.map(u64_to_i64).transpose()?)
        .bind(&result.container)
        .bind(result.bit_rate.map(u64_to_i64).transpose()?)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM media_streams WHERE source_id = $1")
            .bind(source_id.as_uuid())
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
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
            )
            .bind(source_id.as_uuid())
            .bind(u32_to_i64(stream.index))
            .bind(kind)
            .bind(kind_key)
            .bind(&stream.codec)
            .bind(&stream.language)
            .bind(stream.duration_ms.map(u64_to_i64).transpose()?)
            .bind(stream.bit_rate.map(u64_to_i64).transpose()?)
            .bind(stream.width.map(u32_to_i64))
            .bind(stream.height.map(u32_to_i64))
            .bind(stream.channels.map(u32_to_i64))
            .bind(stream.sample_rate.map(u32_to_i64))
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
            WHERE source_id = $1
            "#,
        )
        .bind(source_id.as_uuid())
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
            WHERE source_id = $1
            ORDER BY stream_index ASC
            "#,
        )
        .bind(source_id.as_uuid())
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
impl LocalInferenceRepository for PostgresStore {
    async fn upsert_local_inference_evidence(
        &self,
        evidence: &LocalInferenceEvidence,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_local_inference_evidence_tx(&mut transaction, evidence).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_local_inference_evidence(
        &self,
        id: LocalInferenceEvidenceId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        let row = sqlx::query(LOCAL_INFERENCE_EVIDENCE_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_local_inference_evidence).transpose()
    }

    async fn list_local_inference_evidence_for_source(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<LocalInferenceEvidence>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                inferred_kind,
                inferred_title,
                inferred_year,
                inferred_season,
                inferred_episode,
                confidence_milli,
                evidence_source,
                evidence_source_key,
                evidence_value,
                inference_version
            FROM local_inference_evidence
            WHERE source_id = $1
            ORDER BY inference_version ASC, created_at ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_local_inference_evidence)
            .collect()
    }
}

#[async_trait::async_trait]
impl IngestionFailureRepository for PostgresStore {
    async fn record_ingestion_failure(
        &self,
        failure: NewIngestionFailure,
    ) -> Result<IngestionFailureRecord> {
        sqlx::query(
            r#"
            INSERT INTO ingestion_failures (
                library_id, phase, target_uri, target_kind, job_id, scan_id,
                source_id, failure_class, status, message, retryable, attempts,
                first_failed_at_ms, last_failed_at_ms, resolved_at_ms, ignored_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1, $12, $12, NULL, NULL)
            ON CONFLICT(library_id, phase, target_uri) DO UPDATE SET
                target_kind = excluded.target_kind,
                job_id = excluded.job_id,
                scan_id = excluded.scan_id,
                source_id = excluded.source_id,
                failure_class = excluded.failure_class,
                status = excluded.status,
                message = excluded.message,
                retryable = excluded.retryable,
                attempts = ingestion_failures.attempts + 1,
                last_failed_at_ms = excluded.last_failed_at_ms,
                resolved_at_ms = NULL,
                ignored_at_ms = NULL,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(failure.library_id.as_uuid())
        .bind(failure.phase.as_str())
        .bind(&failure.target_uri)
        .bind(&failure.target_kind)
        .bind(failure.job_id.map(|id| id.as_uuid()))
        .bind(failure.scan_id.map(|id| id.as_uuid()))
        .bind(failure.source_id.map(|id| id.as_uuid()))
        .bind(failure.failure_class.as_str())
        .bind(IngestionFailureStatus::Open.as_str())
        .bind(&failure.message)
        .bind(failure.retryable)
        .bind(failure.failed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(failure.library_id, failure.phase, &failure.target_uri)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "ingestion_failure",
                id: format!(
                    "{}:{}:{}",
                    failure.library_id,
                    failure.phase.as_str(),
                    failure.target_uri
                ),
            })
    }

    async fn resolve_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        resolved_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Resolved,
            Some(resolved_at_ms),
            None,
        )
        .await
    }

    async fn ignore_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        ignored_at_ms: i64,
    ) -> Result<Option<IngestionFailureRecord>> {
        self.update_ingestion_failure_status(
            library_id,
            phase,
            target_uri,
            IngestionFailureStatus::Ignored,
            None,
            Some(ignored_at_ms),
        )
        .await
    }

    async fn list_ingestion_failures(
        &self,
        filter: IngestionFailureFilter,
        page: PageRequest,
    ) -> Result<Vec<IngestionFailureRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let phase = filter.phase.map(|phase| phase.as_str().to_owned());
        let status = filter.status.map(|status| status.as_str().to_owned());
        let rows = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                phase,
                target_uri,
                target_kind,
                job_id::text AS job_id,
                scan_id::text AS scan_id,
                source_id::text AS source_id,
                failure_class,
                status,
                message,
                retryable,
                attempts,
                first_failed_at_ms,
                last_failed_at_ms,
                resolved_at_ms,
                ignored_at_ms
            FROM ingestion_failures
            WHERE ($1::uuid IS NULL OR library_id = $1)
              AND ($2::text IS NULL OR phase = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY last_failed_at_ms DESC, target_uri ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(library_id)
        .bind(phase.as_deref())
        .bind(status.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_ingestion_failure).collect()
    }

    async fn count_ingestion_failures(
        &self,
        library_id: LibraryId,
        phase: Option<IngestionFailurePhase>,
        status: IngestionFailureStatus,
    ) -> Result<u64> {
        let phase = phase.map(|phase| phase.as_str().to_owned());
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM ingestion_failures
            WHERE library_id = $1
              AND ($2::text IS NULL OR phase = $2)
              AND status = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_deref())
        .bind(status.as_str())
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        i64_to_u64(count)
    }
}

#[async_trait::async_trait]
impl ScanRepository for PostgresStore {
    async fn begin_scan_snapshot(
        &self,
        id: ScanSnapshotId,
        library_id: LibraryId,
        root: &str,
    ) -> Result<ScanSnapshot> {
        sqlx::query(
            r#"
            INSERT INTO scan_snapshots (id, library_id, root, status)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(id.as_uuid())
        .bind(library_id.as_uuid())
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
                status = $2,
                error = $3,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
            SELECT
                id::text AS id,
                library_id::text AS library_id,
                root,
                to_char(started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
                to_char(completed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS completed_at,
                status,
                error
            FROM scan_snapshots
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT(scan_id, uri) DO UPDATE SET
                etag = excluded.etag,
                modified_at = excluded.modified_at,
                child_count = excluded.child_count
            "#,
        )
        .bind(snapshot.scan_id.as_uuid())
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
            SELECT scan_id::text AS scan_id, uri, etag, modified_at, child_count
            FROM directory_snapshots
            WHERE scan_id = $1
            ORDER BY uri ASC
            "#,
        )
        .bind(scan_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_directory_snapshot).collect()
    }

    async fn upsert_source_state(&self, state: &SourceState) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_source_state_tx(&mut transaction, state).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_library_scan_source(
        &self,
        commit: &LibraryScanSourcePersistenceCommit,
    ) -> Result<LibraryScanSourcePersistenceSummary> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for item in &commit.items {
            upsert_media_item_tx(&mut transaction, item).await?;
        }
        upsert_media_source_tx(&mut transaction, &commit.source).await?;
        upsert_source_state_tx(&mut transaction, &commit.source_state).await?;
        for state in &commit.library_item_states {
            upsert_library_item_state_tx(&mut transaction, state).await?;
        }
        for evidence in &commit.local_inference_evidence {
            upsert_local_inference_evidence_tx(&mut transaction, evidence).await?;
        }
        for projection in &commit.search_projections {
            upsert_search_projection_tx(&mut transaction, projection).await?;
        }
        let mut resolved_ingestion_failures = 0;
        for resolution in &commit.resolved_ingestion_failures {
            resolved_ingestion_failures += resolve_ingestion_failure_tx(
                &mut transaction,
                resolution.library_id,
                resolution.phase,
                &resolution.target_uri,
                resolution.resolved_at_ms,
            )
            .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(LibraryScanSourcePersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            source_id: commit.source.id,
            library_item_states: commit.library_item_states.len() as u64,
            local_inference_evidence: commit.local_inference_evidence.len() as u64,
            search_projections: commit.search_projections.len() as u64,
            resolved_ingestion_failures,
        })
    }

    async fn get_source_state(
        &self,
        library_id: LibraryId,
        uri: &str,
    ) -> Result<Option<SourceState>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                source_id::text AS source_id,
                uri,
                size_bytes,
                modified_at,
                etag,
                fingerprint,
                last_seen_scan_id::text AS last_seen_scan_id,
                tombstoned
            FROM source_states
            WHERE library_id = $1 AND uri = $2
            "#,
        )
        .bind(library_id.as_uuid())
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
                library_id::text AS library_id,
                source_id::text AS source_id,
                uri,
                size_bytes,
                modified_at,
                etag,
                fingerprint,
                last_seen_scan_id::text AS last_seen_scan_id,
                tombstoned
            FROM source_states
            WHERE library_id = $1
            ORDER BY uri ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_source_state).collect()
    }
}

#[async_trait::async_trait]
impl SearchIndex for PostgresStore {
    async fn upsert(&self, document: SearchDocument) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let mut projection =
            CatalogSearchProjection::new(document.item_id, document.title, document.body);
        projection.projection_version = document.projection_version;
        projection.aliases = document.aliases;
        projection.browse_facets = document.browse_facets;

        upsert_search_projection_tx(&mut transaction, &projection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn delete(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM search_documents WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchHit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                projection_version,
                title,
                body,
                aliases_json::text AS aliases_json,
                facets_json::text AS facets_json
            FROM search_documents
            ORDER BY title ASC, item_id ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let documents = rows
            .into_iter()
            .map(|row| {
                let aliases_json: String = row_get(&row, "aliases_json")?;
                let facets_json: String = row_get(&row, "facets_json")?;
                Ok(SearchEvaluationDocument::from_facet_labels(
                    parse_id(row_get::<String>(&row, "item_id")?)?,
                    i64_to_u16(row_get::<i64>(&row, "projection_version")?)?,
                    row_get::<String>(&row, "title")?,
                    row_get::<String>(&row, "body")?,
                    serde_json::from_str(&aliases_json).map_err(database_error)?,
                    serde_json::from_str(&facets_json).map_err(database_error)?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(evaluate_search_documents(&query, documents))
    }
}

#[async_trait::async_trait]
impl ProviderMappingRepository for PostgresStore {
    async fn upsert_provider_subject(&self, subject: &ProviderSubject) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_subject_tx(&mut transaction, subject).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_provider_subject(&self, id: ProviderSubjectId) -> Result<Option<ProviderSubject>> {
        let row = sqlx::query(PROVIDER_SUBJECT_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_provider_subject).transpose()
    }

    async fn find_provider_subject(
        &self,
        provider: &ExternalProvider,
        subject_kind: &ProviderSubjectKind,
        subject_key: &str,
    ) -> Result<Option<ProviderSubject>> {
        let (provider, provider_key) = provider_to_parts(provider);
        let (subject_kind, subject_kind_key) = provider_subject_kind_to_parts(subject_kind);
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                provider,
                provider_key,
                subject_kind,
                subject_kind_key,
                subject_key,
                title,
                release_year,
                locale
            FROM provider_subjects
            WHERE provider = $1
              AND provider_key = $2
              AND subject_kind = $3
              AND subject_kind_key = $4
              AND subject_key = $5
            LIMIT 1
            "#,
        )
        .bind(provider)
        .bind(provider_key)
        .bind(subject_kind)
        .bind(subject_kind_key)
        .bind(subject_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_subject).transpose()
    }

    async fn list_provider_subjects_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderSubject>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                provider_subjects.id::text AS id,
                provider_subjects.provider,
                provider_subjects.provider_key,
                provider_subjects.subject_kind,
                provider_subjects.subject_kind_key,
                provider_subjects.subject_key,
                provider_subjects.title,
                provider_subjects.release_year,
                provider_subjects.locale
            FROM provider_subjects
            INNER JOIN provider_mappings
                ON provider_mappings.subject_id = provider_subjects.id
            WHERE provider_mappings.item_id = $1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_subject).collect()
    }

    async fn upsert_provider_mapping(&self, mapping: &ProviderMapping) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_mapping_tx(&mut transaction, mapping).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn list_provider_mappings_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ProviderMapping>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                provider_mappings.id::text AS id,
                provider_mappings.item_id::text AS item_id,
                provider_mappings.subject_id::text AS subject_id,
                provider_mappings.status,
                provider_mappings.confidence_milli,
                provider_mappings.source,
                provider_mappings.source_key
            FROM provider_mappings
            INNER JOIN provider_subjects
                ON provider_subjects.id = provider_mappings.subject_id
            WHERE provider_mappings.item_id = $1
            ORDER BY
                provider_subjects.provider ASC,
                provider_subjects.provider_key ASC,
                provider_subjects.subject_kind ASC,
                provider_subjects.subject_kind_key ASC,
                provider_subjects.subject_key ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_mapping).collect()
    }
}

#[async_trait::async_trait]
impl MetadataRepository for PostgresStore {
    async fn upsert_field_lock(&self, lock: &MetadataFieldLock) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_field_lock_tx(&mut transaction, lock).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn list_field_locks(&self, item_id: MediaItemId) -> Result<Vec<MetadataFieldLock>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, field, locked, source, source_key
            FROM metadata_field_locks
            WHERE item_id = $1
            ORDER BY field ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_metadata_field_lock).collect()
    }

    async fn upsert_provider_raw_response(&self, response: &ProviderRawResponse) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_provider_raw_response_tx(&mut transaction, response).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_metadata_refresh(
        &self,
        commit: &MetadataRefreshPersistenceCommit,
    ) -> Result<MetadataRefreshPersistenceSummary> {
        if commit.raw_response.item_id != commit.item.id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "metadata refresh raw response item_id {} does not match item {}",
                    commit.raw_response.item_id, commit.item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_item_tx(&mut transaction, &commit.item).await?;
        upsert_provider_raw_response_tx(&mut transaction, &commit.raw_response).await?;
        upsert_provider_subject_tx(&mut transaction, &commit.provider_mapping.subject).await?;

        let mapping_id = commit
            .provider_mapping
            .id
            .unwrap_or_else(ProviderMappingId::new);
        let mapping = ProviderMapping {
            id: mapping_id,
            item_id: commit.item.id,
            subject_id: commit.provider_mapping.subject.id,
            status: ProviderMappingStatus::Accepted,
            confidence_milli: commit.provider_mapping.confidence_milli,
            source: commit.provider_mapping.source.clone(),
        };
        upsert_provider_mapping_tx(&mut transaction, &mapping).await?;

        let confirmed_libraries = library_ids_for_item_tx(&mut transaction, commit.item.id).await?;
        for library_id in &confirmed_libraries {
            upsert_library_item_state_tx(
                &mut transaction,
                &LibraryItemState {
                    library_id: *library_id,
                    item_id: commit.item.id,
                    provisional: false,
                },
            )
            .await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(MetadataRefreshPersistenceSummary {
            item_id: commit.item.id,
            provider_subject_id: commit.provider_mapping.subject.id,
            provider_mapping_id: mapping_id,
            confirmed_libraries,
        })
    }

    async fn commit_nfo_import(
        &self,
        commit: &NfoImportPersistenceCommit,
    ) -> Result<NfoImportPersistenceSummary> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        for item in &commit.items {
            upsert_media_item_tx(&mut transaction, item).await?;
        }
        for lock in &commit.field_locks {
            upsert_field_lock_tx(&mut transaction, lock).await?;
        }
        for state in &commit.library_item_states {
            upsert_library_item_state_tx(&mut transaction, state).await?;
        }
        for projection in &commit.catalog_projections {
            replace_item_catalog_graph_tx(
                &mut transaction,
                projection.search.item_id,
                &projection.graph,
            )
            .await?;
            upsert_search_projection_tx(&mut transaction, &projection.search).await?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(NfoImportPersistenceSummary {
            item_ids: commit.items.iter().map(|item| item.id).collect(),
            locked_fields: commit.field_locks.len() as u64,
            confirmed_items: commit.library_item_states.len() as u64,
            projected_items: commit.catalog_projections.len() as u64,
        })
    }

    async fn commit_addon_metadata_write(
        &self,
        commit: &AddonMetadataWritePersistenceCommit,
    ) -> Result<AddonMetadataWritePersistenceSummary> {
        if commit.catalog.search.item_id != commit.item.id {
            return Err(TaruError::InvalidInput {
                message: format!(
                    "addon metadata write search projection item_id {} does not match item {}",
                    commit.catalog.search.item_id, commit.item.id
                ),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_media_item_tx(&mut transaction, &commit.item).await?;
        if let Some(graph) = &commit.catalog.graph {
            replace_item_catalog_graph_tx(&mut transaction, commit.item.id, graph).await?;
        }
        upsert_search_projection_tx(&mut transaction, &commit.catalog.search).await?;
        let side_effect = set_addon_side_effect_apply_outcome_tx(
            &mut transaction,
            commit.side_effect_id,
            &AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Applied,
                error_code: None,
                item_id: Some(commit.item.id),
                source: Some(commit.applied_source.clone()),
                report_json: commit.apply_report_json.clone(),
            },
        )
        .await?;

        transaction.commit().await.map_err(database_error)?;

        Ok(AddonMetadataWritePersistenceSummary {
            item_id: commit.item.id,
            projected_items: 1,
            side_effect,
        })
    }

    async fn commit_metadata_item(&self, item: &MediaItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_media_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
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
            SELECT item_id::text AS item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = $1 AND provider = $2 AND provider_key = $3
            LIMIT 1
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_provider_raw_response).transpose()
    }

    async fn list_provider_raw_responses(
        &self,
        item_id: MediaItemId,
        filter: ProviderRawResponseFilter,
        page: PageRequest,
    ) -> Result<Vec<ProviderRawResponse>> {
        let page = page.clamped();
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, provider, provider_key, body_json, fetched_at
            FROM provider_raw_responses
            WHERE item_id = $1
              AND ($2::text IS NULL OR provider = $2)
            ORDER BY fetched_at DESC, provider ASC, provider_key ASC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(provider.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_provider_raw_response).collect()
    }

    async fn cleanup_provider_raw_responses(
        &self,
        filter: ProviderRawResponseFilter,
        fetched_before: &str,
    ) -> Result<ProviderRawResponseCleanup> {
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let deleted = sqlx::query(
            r#"
            DELETE FROM provider_raw_responses
            WHERE fetched_at < $1
              AND ($2::text IS NULL OR provider = $2)
            "#,
        )
        .bind(fetched_before)
        .bind(provider.as_deref())
        .execute(&self.pool)
        .await
        .map_err(database_error)?
        .rows_affected();

        Ok(ProviderRawResponseCleanup {
            provider: provider.map(|provider| provider_from_parts(provider, String::new())),
            fetched_before: fetched_before.to_owned(),
            deleted,
        })
    }

    async fn insert_metadata_provider_attempt(
        &self,
        attempt: NewMetadataProviderAttempt,
    ) -> Result<()> {
        let (provider, _) = provider_to_parts(&attempt.provider);

        sqlx::query(
            r#"
            INSERT INTO metadata_provider_attempts (
                id,
                job_id,
                item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(attempt.id.as_uuid())
        .bind(attempt.job_id.as_uuid())
        .bind(attempt.item_id.as_uuid())
        .bind(provider)
        .bind(attempt.provider_key)
        .bind(attempt.status.as_str())
        .bind(attempt.matched_by.map(MetadataMatchKind::as_str))
        .bind(attempt.started_at)
        .bind(attempt.finished_at)
        .bind(attempt.error_class.map(MetadataProviderErrorClass::as_str))
        .bind(attempt.message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn list_metadata_provider_attempts(
        &self,
        job_id: JobId,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                job_id::text AS job_id,
                item_id::text AS item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE job_id = $1
            ORDER BY started_at ASC, created_at ASC
            "#,
        )
        .bind(job_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_provider_attempt)
            .collect()
    }

    async fn list_metadata_provider_attempts_for_item(
        &self,
        item_id: MediaItemId,
        filter: MetadataAttemptFilter,
        page: PageRequest,
    ) -> Result<Vec<MetadataProviderAttemptRecord>> {
        let page = page.clamped();
        let provider = filter
            .provider
            .map(|provider| provider_to_parts(&provider).0);
        let status = filter.status.map(MetadataProviderAttemptStatus::as_str);
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                job_id::text AS job_id,
                item_id::text AS item_id,
                provider,
                provider_key,
                status,
                matched_by,
                started_at,
                finished_at,
                error_class,
                message
            FROM metadata_provider_attempts
            WHERE item_id = $1
              AND ($2::text IS NULL OR provider = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY started_at DESC, created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(provider.as_deref())
        .bind(status)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_metadata_provider_attempt)
            .collect()
    }
}

#[async_trait::async_trait]
impl CatalogRepository for PostgresStore {
    async fn replace_item_catalog_graph(
        &self,
        item_id: MediaItemId,
        replacement: &CatalogItemGraphReplacement,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        replace_item_catalog_graph_tx(&mut transaction, item_id, replacement).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn commit_item_projection(&self, commit: &CatalogItemProjectionCommit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        replace_item_catalog_graph_tx(&mut transaction, commit.search.item_id, &commit.graph)
            .await?;
        upsert_search_projection_tx(&mut transaction, &commit.search).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_search_projection(&self, projection: &CatalogSearchProjection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_search_projection_tx(&mut transaction, projection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn upsert_person(&self, person: &Person) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_person_tx(&mut transaction, person).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_person(&self, id: PersonId) -> Result<Option<Person>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, sort_name, overview FROM people WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };

        let external_ids = self
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Person>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT p.id::text AS id, p.name, p.sort_name, p.overview
            FROM people p
            JOIN person_external_ids e ON e.person_id = p.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
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
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn find_person_by_name(&self, name: &str) -> Result<Option<Person>> {
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, sort_name, overview
            FROM people
            WHERE name = $1
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
            .list_catalog_external_ids("person_external_ids", "person_id", id)
            .await?;
        row_to_person(row, external_ids).map(Some)
    }

    async fn list_people(&self, page: PageRequest) -> Result<Vec<Person>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, sort_name, overview
            FROM people
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
                .list_catalog_external_ids("person_external_ids", "person_id", id)
                .await?;
            people.push(row_to_person(row, external_ids)?);
        }

        Ok(people)
    }

    async fn upsert_item_credit(&self, credit: &ItemCredit) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_credit_tx(&mut transaction, credit).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_credits(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_credits WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_credits(&self, item_id: MediaItemId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                person_id::text AS person_id,
                role,
                role_key,
                character,
                sort_order
            FROM item_credits
            WHERE item_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), role ASC, person_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_credit).collect()
    }

    async fn list_person_credits(&self, person_id: PersonId) -> Result<Vec<ItemCredit>> {
        let rows = sqlx::query(
            r#"
            SELECT
                item_id::text AS item_id,
                person_id::text AS person_id,
                role,
                role_key,
                character,
                sort_order
            FROM item_credits
            WHERE person_id = $1
            ORDER BY role ASC, COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(person_id.as_uuid())
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
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_credits WHERE person_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(person_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_genre(&self, genre: &Genre) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_genre_tx(&mut transaction, genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_genre(&self, id: GenreId) -> Result<Option<Genre>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, source, source_key FROM genres WHERE id = $1",
        )
        .bind(id.as_uuid())
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
            SELECT id::text AS id, name, source, source_key
            FROM genres
            WHERE name = $1 AND source = $2 AND source_key = $3
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
            SELECT id::text AS id, name, source, source_key
            FROM genres
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_genre_tx(&mut transaction, item_genre).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_genres(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_genres WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_genres(&self, item_id: MediaItemId) -> Result<Vec<ItemGenre>> {
        let rows = sqlx::query(
            "SELECT item_id::text AS item_id, genre_id::text AS genre_id FROM item_genres WHERE item_id = $1 ORDER BY genre_id ASC",
        )
        .bind(item_id.as_uuid())
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
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_genres WHERE genre_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(genre_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_tag(&self, tag: &Tag) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_tag_tx(&mut transaction, tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_tag(&self, id: TagId) -> Result<Option<Tag>> {
        let row =
            sqlx::query("SELECT id::text AS id, name, source, source_key FROM tags WHERE id = $1")
                .bind(id.as_uuid())
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
            SELECT id::text AS id, name, source, source_key
            FROM tags
            WHERE name = $1 AND source = $2 AND source_key = $3
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
            SELECT id::text AS id, name, source, source_key
            FROM tags
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_tag_tx(&mut transaction, item_tag).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_tags(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_tags WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_tags(&self, item_id: MediaItemId) -> Result<Vec<ItemTag>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, tag_id::text AS tag_id
            FROM item_tags
            WHERE item_id = $1
            ORDER BY tag_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_tag).collect()
    }

    async fn list_tag_items(&self, tag_id: TagId, page: PageRequest) -> Result<Vec<MediaItem>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                mi.id::text AS id,
                mi.kind,
                mi.parent_id::text AS parent_id,
                mi.title,
                mi.original_title,
                mi.sort_title,
                mi.overview,
                mi.release_date,
                mi.metadata_json::text AS metadata_json
            FROM media_items mi
            WHERE mi.id IN (
                SELECT item_id FROM item_tags WHERE tag_id = $1
            )
            ORDER BY mi.title ASC, mi.id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(tag_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        self.rows_to_media_items(rows).await
    }

    async fn upsert_collection(&self, collection: &Collection) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_collection_tx(&mut transaction, collection).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>> {
        let row = sqlx::query(
            r#"
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
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
            SELECT c.id::text AS id, c.name, c.overview, c.source, c.source_key
            FROM collections c
            JOIN collection_external_ids e ON e.collection_id = c.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
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
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
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
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            WHERE name = $1 AND source = $2 AND source_key = $3
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
            .list_catalog_external_ids("collection_external_ids", "collection_id", id)
            .await?;
        row_to_collection(row, external_ids).map(Some)
    }

    async fn list_collections(&self, page: PageRequest) -> Result<Vec<Collection>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, overview, source, source_key
            FROM collections
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
                .list_catalog_external_ids("collection_external_ids", "collection_id", id)
                .await?;
            collections.push(row_to_collection(row, external_ids)?);
        }

        Ok(collections)
    }

    async fn upsert_collection_item(&self, item: &CollectionItem) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_collection_item_tx(&mut transaction, item).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_collections(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM collection_items WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_collections(&self, item_id: MediaItemId) -> Result<Vec<CollectionItem>> {
        let rows = sqlx::query(
            r#"
            SELECT collection_id::text AS collection_id, item_id::text AS item_id, sort_order
            FROM collection_items
            WHERE item_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), collection_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
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
            SELECT collection_id::text AS collection_id, item_id::text AS item_id, sort_order
            FROM collection_items
            WHERE collection_id = $1
            ORDER BY COALESCE(sort_order, 2147483647), item_id ASC
            "#,
        )
        .bind(collection_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_collection_item).collect()
    }

    async fn upsert_studio(&self, studio: &Studio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_studio_tx(&mut transaction, studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_studio(&self, id: StudioId) -> Result<Option<Studio>> {
        let row = sqlx::query(
            "SELECT id::text AS id, name, source, source_key FROM studios WHERE id = $1",
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            return Ok(None);
        };
        let external_ids = self
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn find_studio_by_external_id(&self, external_id: &ExternalId) -> Result<Option<Studio>> {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        let row = sqlx::query(
            r#"
            SELECT s.id::text AS id, s.name, s.source, s.source_key
            FROM studios s
            JOIN studio_external_ids e ON e.studio_id = s.id
            WHERE e.provider = $1 AND e.provider_key = $2 AND e.value = $3
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
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
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
            SELECT id::text AS id, name, source, source_key
            FROM studios
            WHERE name = $1 AND source = $2 AND source_key = $3
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
            .list_catalog_external_ids("studio_external_ids", "studio_id", id)
            .await?;
        row_to_studio(row, external_ids).map(Some)
    }

    async fn list_studios(&self, page: PageRequest) -> Result<Vec<Studio>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT id::text AS id, name, source, source_key
            FROM studios
            ORDER BY name ASC, id ASC
            LIMIT $1 OFFSET $2
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
                .list_catalog_external_ids("studio_external_ids", "studio_id", id)
                .await?;
            studios.push(row_to_studio(row, external_ids)?);
        }

        Ok(studios)
    }

    async fn upsert_item_studio(&self, item_studio: &ItemStudio) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_item_studio_tx(&mut transaction, item_studio).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn clear_item_studios(&self, item_id: MediaItemId) -> Result<()> {
        sqlx::query("DELETE FROM item_studios WHERE item_id = $1")
            .bind(item_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn list_item_studios(&self, item_id: MediaItemId) -> Result<Vec<ItemStudio>> {
        let rows = sqlx::query(
            r#"
            SELECT item_id::text AS item_id, studio_id::text AS studio_id
            FROM item_studios
            WHERE item_id = $1
            ORDER BY studio_id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_item_studio).collect()
    }

    async fn upsert_image_asset(&self, image: &ImageAsset) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        upsert_image_asset_tx(&mut transaction, image).await?;
        transaction.commit().await.map_err(database_error)
    }

    async fn get_image_asset(&self, id: ImageAssetId) -> Result<Option<ImageAsset>> {
        let row = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
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
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE owner_kind = $1 AND owner_id = $2::uuid AND kind = $3
                AND kind_key = $4 AND source_uri = $5
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
                id::text AS id,
                owner_kind,
                owner_id::text AS owner_id,
                kind,
                kind_key,
                source_uri,
                provider,
                provider_key,
                cache_uri,
                width,
                height,
                language,
                selected,
                content_hash,
                etag
            FROM image_assets
            WHERE owner_kind = 'item' AND owner_id = $1
            ORDER BY selected DESC, kind ASC, id ASC
            "#,
        )
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_image_asset).collect()
    }
}

#[async_trait::async_trait]
impl SourceDuplicateRepository for PostgresStore {
    async fn upsert_source_duplicate_relationship(
        &self,
        relationship: &SourceDuplicateRelationship,
    ) -> Result<()> {
        let relationship = relationship.canonicalized();
        let (evidence_kind, evidence_kind_key) =
            source_duplicate_evidence_kind_to_parts(&relationship.evidence_kind);

        sqlx::query(
            r#"
            INSERT INTO source_duplicate_relationships (
                id,
                source_id,
                duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                source_id = excluded.source_id,
                duplicate_source_id = excluded.duplicate_source_id,
                evidence_kind = excluded.evidence_kind,
                evidence_kind_key = excluded.evidence_kind_key,
                evidence_value = excluded.evidence_value,
                status = excluded.status,
                confidence_milli = excluded.confidence_milli,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(relationship.id.as_uuid())
        .bind(relationship.source_id.as_uuid())
        .bind(relationship.duplicate_source_id.as_uuid())
        .bind(evidence_kind)
        .bind(evidence_kind_key)
        .bind(&relationship.evidence_value)
        .bind(relationship.status.as_str())
        .bind(relationship.confidence_milli.map(i64::from))
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(())
    }

    async fn get_source_duplicate_relationship(
        &self,
        id: SourceDuplicateRelationshipId,
    ) -> Result<Option<SourceDuplicateRelationship>> {
        let row = sqlx::query(SOURCE_DUPLICATE_RELATIONSHIP_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_source_duplicate_relationship).transpose()
    }

    async fn list_source_duplicate_relationships(
        &self,
        source_id: MediaSourceId,
        page: PageRequest,
    ) -> Result<Vec<SourceDuplicateRelationship>> {
        let page = page.clamped();
        let rows = sqlx::query(
            r#"
            SELECT
                id::text AS id,
                source_id::text AS source_id,
                duplicate_source_id::text AS duplicate_source_id,
                evidence_kind,
                evidence_kind_key,
                evidence_value,
                status,
                confidence_milli
            FROM source_duplicate_relationships
            WHERE source_id = $1 OR duplicate_source_id = $1
            ORDER BY source_id ASC, duplicate_source_id ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(source_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_source_duplicate_relationship)
            .collect()
    }
}

#[async_trait::async_trait]
impl ManagedImportRepository for PostgresStore {
    async fn upsert_managed_import_artifact(
        &self,
        artifact: NewManagedImportArtifact,
    ) -> Result<ManagedImportArtifactRecord> {
        let (source_kind, source_kind_key) =
            managed_import_source_kind_to_parts(&artifact.source_kind);
        sqlx::query(
            r#"
            INSERT INTO managed_import_artifacts (
                id, target_library_id, source_kind, source_kind_key, source_uri,
                staging_manifest_id, artifact_uri, original_file_name, intended_locator,
                size_bytes, fingerprint, state, diagnostics_json, created_at_ms, updated_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            ON CONFLICT(id) DO UPDATE SET
                target_library_id = excluded.target_library_id,
                source_kind = excluded.source_kind,
                source_kind_key = excluded.source_kind_key,
                source_uri = excluded.source_uri,
                staging_manifest_id = excluded.staging_manifest_id,
                artifact_uri = excluded.artifact_uri,
                original_file_name = excluded.original_file_name,
                intended_locator = excluded.intended_locator,
                size_bytes = excluded.size_bytes,
                fingerprint = excluded.fingerprint,
                state = excluded.state,
                diagnostics_json = excluded.diagnostics_json,
                updated_at_ms = excluded.updated_at_ms,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(artifact.id.as_uuid())
        .bind(artifact.target_library_id.as_uuid())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(&artifact.source_uri)
        .bind(artifact.staging_manifest_id.map(|id| id.as_uuid()))
        .bind(&artifact.artifact_uri)
        .bind(&artifact.original_file_name)
        .bind(&artifact.intended_locator)
        .bind(optional_u64_to_i64(artifact.size_bytes)?)
        .bind(&artifact.fingerprint)
        .bind(artifact.state.as_str())
        .bind(&artifact.diagnostics_json)
        .bind(artifact.created_at_ms)
        .bind(artifact.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_managed_import_artifact(artifact.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_import_artifact",
                id: artifact.id.to_string(),
            })
    }

    async fn get_managed_import_artifact(
        &self,
        id: ManagedImportArtifactId,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {MANAGED_IMPORT_ARTIFACT_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_import_artifact).transpose()
    }

    async fn find_managed_import_artifact_by_source(
        &self,
        target_library_id: LibraryId,
        source_kind: &ManagedImportSourceKind,
        source_uri: &str,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        let (source_kind, source_kind_key) = managed_import_source_kind_to_parts(source_kind);
        let row = sqlx::query(&format!(
            r#"
            {MANAGED_IMPORT_ARTIFACT_SELECT}
            WHERE target_library_id = $1
              AND source_kind = $2
              AND source_kind_key = $3
              AND source_uri = $4
            "#
        ))
        .bind(target_library_id.as_uuid())
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_import_artifact).transpose()
    }

    async fn list_managed_import_artifacts(
        &self,
        filter: ManagedImportArtifactListFilter,
        page: PageRequest,
    ) -> Result<Vec<ManagedImportArtifactRecord>> {
        let page = page.clamped();
        let target_library_id = filter.target_library_id.map(|id| id.as_uuid());
        let state = filter.state.map(ManagedImportArtifactState::as_str);
        let (source_kind, source_kind_key) = filter
            .source_kind
            .as_ref()
            .map(managed_import_source_kind_to_parts)
            .map_or((None, None), |(kind, kind_key)| {
                (Some(kind), Some(kind_key))
            });
        let rows = sqlx::query(&format!(
            r#"
            {MANAGED_IMPORT_ARTIFACT_SELECT}
            WHERE ($1::uuid IS NULL OR target_library_id = $1)
              AND ($2::text IS NULL OR state = $2)
              AND ($3::text IS NULL OR (source_kind = $3 AND source_kind_key = $4))
            ORDER BY updated_at_ms DESC, id ASC
            LIMIT $5 OFFSET $6
            "#
        ))
        .bind(target_library_id)
        .bind(state)
        .bind(source_kind)
        .bind(source_kind_key)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_managed_import_artifact)
            .collect()
    }

    async fn set_managed_import_artifact_state(
        &self,
        id: ManagedImportArtifactId,
        state: ManagedImportArtifactState,
        updated_at_ms: i64,
        diagnostics_json: Option<String>,
    ) -> Result<Option<ManagedImportArtifactRecord>> {
        sqlx::query(
            r#"
            UPDATE managed_import_artifacts
            SET state = $2,
                updated_at_ms = $3,
                diagnostics_json = $4,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(state.as_str())
        .bind(updated_at_ms)
        .bind(diagnostics_json)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_managed_import_artifact(id).await
    }
}

#[async_trait::async_trait]
impl CatalogGovernanceRepository for PostgresStore {
    async fn list_catalog_governance_items(
        &self,
        filter: CatalogGovernanceItemListFilter,
        page: PageRequest,
    ) -> Result<Vec<CatalogGovernanceItemRecord>> {
        let page = page.clamped();
        let library_id = filter.library_id.map(|id| id.as_uuid());
        let rows = sqlx::query(
            r#"
            SELECT
                media_items.id::text AS id,
                media_items.kind,
                media_items.parent_id::text AS parent_id,
                media_items.title,
                media_items.original_title,
                media_items.sort_title,
                media_items.overview,
                media_items.release_date,
                media_items.metadata_json::text AS metadata_json,
                media_sources.library_id::text AS governance_library_id,
                COUNT(DISTINCT media_sources.id) AS source_count,
                (
                    SELECT representative.id::text
                    FROM media_sources AS representative
                    WHERE representative.item_id = media_items.id
                      AND representative.library_id = media_sources.library_id
                    ORDER BY representative.file_name ASC, representative.id ASC
                    LIMIT 1
                ) AS representative_source_id,
                (
                    SELECT representative.file_name
                    FROM media_sources AS representative
                    WHERE representative.item_id = media_items.id
                      AND representative.library_id = media_sources.library_id
                    ORDER BY representative.file_name ASC, representative.id ASC
                    LIMIT 1
                ) AS representative_file_name,
                (
                    SELECT COUNT(*)
                    FROM provider_mappings
                    WHERE provider_mappings.item_id = media_items.id
                ) AS provider_mapping_count,
                (
                    SELECT COUNT(*)
                    FROM provider_mappings
                    WHERE provider_mappings.item_id = media_items.id
                      AND provider_mappings.status = 'accepted'
                ) AS accepted_provider_mapping_count,
                (
                    SELECT COUNT(DISTINCT duplicate.id)
                    FROM source_duplicate_relationships AS duplicate
                    INNER JOIN media_sources AS duplicate_source
                        ON duplicate_source.id = duplicate.source_id
                        OR duplicate_source.id = duplicate.duplicate_source_id
                    WHERE duplicate_source.item_id = media_items.id
                      AND duplicate_source.library_id = media_sources.library_id
                ) AS duplicate_relationship_count,
                (
                    SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                    FROM local_inference_evidence AS evidence
                    INNER JOIN media_sources AS evidence_source
                        ON evidence_source.id = evidence.source_id
                    WHERE evidence_source.item_id = media_items.id
                      AND evidence_source.library_id = media_sources.library_id
                ) AS best_confidence_milli
            FROM media_items
            INNER JOIN media_sources
                ON media_sources.item_id = media_items.id
            WHERE ($1::uuid IS NULL OR media_sources.library_id = $1)
            GROUP BY media_items.id, media_sources.library_id
            HAVING media_items.kind = $2
                OR (
                    (
                        SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                        FROM local_inference_evidence AS evidence
                        INNER JOIN media_sources AS evidence_source
                            ON evidence_source.id = evidence.source_id
                        WHERE evidence_source.item_id = media_items.id
                          AND evidence_source.library_id = media_sources.library_id
                    ) IS NOT NULL
                    AND (
                        SELECT MAX(COALESCE(evidence.confidence_milli, 0))
                        FROM local_inference_evidence AS evidence
                        INNER JOIN media_sources AS evidence_source
                            ON evidence_source.id = evidence.source_id
                        WHERE evidence_source.item_id = media_items.id
                          AND evidence_source.library_id = media_sources.library_id
                    ) <= $3
                )
            ORDER BY
                CASE WHEN media_items.kind = $2 THEN 0 ELSE 1 END ASC,
                best_confidence_milli ASC,
                media_items.title ASC,
                media_items.id ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(library_id)
        .bind(MediaKind::Unknown.as_str())
        .bind(i64::from(filter.max_confidence_milli))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        let mut records = Vec::with_capacity(rows.len());
        for row in rows {
            records.push(self.governance_row_to_record(row).await?);
        }

        Ok(records)
    }
}

#[async_trait::async_trait]
impl UserPlaybackStateRepository for PostgresStore {
    async fn upsert_user_playback_state(
        &self,
        state: UserPlaybackStateWrite,
    ) -> Result<UserPlaybackState> {
        sqlx::query(
            r#"
            INSERT INTO user_playback_states (
                principal_id,
                item_id,
                source_id,
                resume_position_ms,
                duration_ms,
                watched,
                watched_at_ms,
                last_played_at_ms,
                updated_at_ms
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(principal_id, item_id) DO UPDATE SET
                source_id = excluded.source_id,
                resume_position_ms = excluded.resume_position_ms,
                duration_ms = excluded.duration_ms,
                watched = excluded.watched,
                watched_at_ms = excluded.watched_at_ms,
                last_played_at_ms = excluded.last_played_at_ms,
                updated_at_ms = excluded.updated_at_ms,
                version = user_playback_states.version + 1,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(state.principal_id.as_str())
        .bind(state.item_id.as_uuid())
        .bind(state.source_id.map(|id| id.as_uuid()))
        .bind(state.resume_position_ms.map(u64_to_i64).transpose()?)
        .bind(state.duration_ms.map(u64_to_i64).transpose()?)
        .bind(state.watched)
        .bind(state.watched_at_ms)
        .bind(state.last_played_at_ms)
        .bind(state.updated_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_user_playback_state(&state.principal_id, state.item_id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: format!(
                    "user playback state for principal {} and item {} was not found after PostgreSQL upsert",
                    state.principal_id, state.item_id
                ),
            })
    }

    async fn get_user_playback_state(
        &self,
        principal_id: &UserPrincipalId,
        item_id: MediaItemId,
    ) -> Result<Option<UserPlaybackState>> {
        let query =
            format!("{USER_PLAYBACK_STATE_SELECT} WHERE principal_id = $1 AND item_id = $2");
        let row = sqlx::query(&query)
            .bind(principal_id.as_str())
            .bind(item_id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_user_playback_state).transpose()
    }

    async fn list_continue_watching_states(
        &self,
        principal_id: &UserPrincipalId,
        page: PageRequest,
    ) -> Result<Vec<UserPlaybackState>> {
        let page = page.clamped();
        let query = format!(
            r#"
            {USER_PLAYBACK_STATE_SELECT}
            WHERE principal_id = $1
              AND watched = false
              AND resume_position_ms IS NOT NULL
              AND resume_position_ms > 0
            ORDER BY last_played_at_ms DESC, item_id ASC
            LIMIT $2 OFFSET $3
            "#
        );
        let rows = sqlx::query(&query)
            .bind(principal_id.as_str())
            .bind(u32_to_i64(page.limit))
            .bind(u64_to_i64(page.offset)?)
            .fetch_all(&self.pool)
            .await
            .map_err(database_error)?;

        rows.into_iter().map(row_to_user_playback_state).collect()
    }
}

#[async_trait::async_trait]
impl TranscodeSessionRepository for PostgresStore {
    async fn create_transcode_session(
        &self,
        session: NewTranscodeSession,
    ) -> Result<TranscodeSessionRecord> {
        sqlx::query(
            r#"
            INSERT INTO transcode_sessions (
                id, source_id, kind, request_key, output_path, state
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(session.id.as_uuid())
        .bind(session.source_id.as_uuid())
        .bind(session.kind.as_str())
        .bind(&session.request_key)
        .bind(session.output_path.display().to_string())
        .bind(session.state.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_transcode_session_or_not_found(session.id).await
    }

    async fn get_transcode_session(
        &self,
        id: TranscodeSessionId,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(TRANSCODE_SESSION_SELECT_BY_ID)
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn list_transcode_sessions(
        &self,
        filter: TranscodeSessionListFilter,
        page: PageRequest,
    ) -> Result<Vec<TranscodeSessionRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE ($1::uuid IS NULL OR source_id = $1)
              AND ($2::text IS NULL OR kind = $2)
              AND ($3::text IS NULL OR state = $3)
            ORDER BY updated_at DESC, id DESC
            LIMIT $4 OFFSET $5
            "#
        ))
        .bind(filter.source_id.map(|id| id.as_uuid()))
        .bind(filter.kind.map(TranscodeSessionKind::as_str))
        .bind(filter.state.map(TranscodeSessionState::as_str))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_transcode_session).collect()
    }

    async fn find_latest_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE source_id = $1 AND kind = $2 AND request_key = $3
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(source_id.as_uuid())
        .bind(kind.as_str())
        .bind(request_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn find_active_transcode_session(
        &self,
        source_id: MediaSourceId,
        kind: TranscodeSessionKind,
        request_key: &str,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {TRANSCODE_SESSION_SELECT}
            WHERE source_id = $1
                AND kind = $2
                AND request_key = $3
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            ORDER BY updated_at DESC, id DESC
            LIMIT 1
            "#
        ))
        .bind(source_id.as_uuid())
        .bind(kind.as_str())
        .bind(request_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_transcode_session).transpose()
    }

    async fn set_transcode_session_state(
        &self,
        id: TranscodeSessionId,
        state: TranscodeSessionState,
        failure_category: Option<TranscodeFailureCategory>,
        failure_message: Option<String>,
    ) -> Result<TranscodeSessionRecord> {
        sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = $2,
                failure_category = $3,
                failure_message = $4,
                started_at = CASE
                    WHEN started_at IS NULL AND $2 IN ('starting', 'running')
                    THEN statement_timestamp()
                    ELSE started_at
                END,
                completed_at = CASE
                    WHEN $2 IN ('cancelled', 'failed', 'finished')
                    THEN statement_timestamp()
                    ELSE completed_at
                END,
                updated_at = statement_timestamp()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(state.as_str())
        .bind(failure_category.map(TranscodeFailureCategory::as_str))
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_transcode_session_or_not_found(id).await
    }

    async fn request_transcode_session_cancellation(
        &self,
        id: TranscodeSessionId,
        failure_message: String,
    ) -> Result<Option<TranscodeSessionRecord>> {
        let result = sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = 'cancel_requested',
                failure_category = 'cancelled',
                failure_message = $2,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND state IN ('planned', 'starting', 'running', 'cancel_requested')
            "#,
        )
        .bind(id.as_uuid())
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Ok(None);
        }

        Ok(Some(self.get_transcode_session_or_not_found(id).await?))
    }

    async fn fail_stale_transcode_sessions(
        &self,
        failure_category: TranscodeFailureCategory,
        failure_message: String,
    ) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE transcode_sessions
            SET
                state = 'failed',
                failure_category = $1,
                failure_message = $2,
                completed_at = COALESCE(completed_at, statement_timestamp()),
                updated_at = statement_timestamp()
            WHERE state IN ('planned', 'starting', 'running', 'cancel_requested')
            "#,
        )
        .bind(failure_category.as_str())
        .bind(failure_message)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }
}

#[async_trait::async_trait]
impl JobLeaseRepository for PostgresStore {
    async fn claim_next_job_lease(
        &self,
        request: JobLeaseClaimRequest,
    ) -> Result<Option<LeasedJob>> {
        validate_lease_duration(request.lease_duration_ms)?;

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let kind = request.filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = request.filter.resource_class;
        let requested_job_id = request.filter.job_id.map(|id| id.as_uuid());
        let library_id = request.filter.library_id.map(|id| id.as_uuid());
        let source_id = request.filter.source_id.map(|id| id.as_uuid());
        let job_id = sqlx::query_scalar::<_, String>(
            r#"
            SELECT id::text
            FROM jobs
            WHERE status = $1
                AND ($2::text IS NULL OR kind = $2)
                AND ($3::text IS NULL OR resource_class = $3)
                AND ($4::uuid IS NULL OR id = $4)
                AND ($5::uuid IS NULL OR library_id = $5)
                AND ($6::uuid IS NULL OR source_id = $6)
            ORDER BY queued_at ASC, id ASC
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#,
        )
        .bind(JobStatus::Queued.as_str())
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(requested_job_id)
        .bind(library_id)
        .bind(source_id)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some(job_id) = job_id else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        let job_id: JobId = parse_id(job_id)?;
        let run_token = JobRunToken::new();

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $2,
                worker_id = $3,
                run_token = $4,
                heartbeat_at = statement_timestamp(),
                lease_expires_at = statement_timestamp() + ($5::double precision * INTERVAL '1 millisecond'),
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND status = $6
            "#,
        )
        .bind(job_id.as_uuid())
        .bind(JobStatus::Running.as_str())
        .bind(request.worker_id.as_uuid())
        .bind(run_token.as_uuid())
        .bind(u64_to_i64(request.lease_duration_ms)?)
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let leased = get_leased_job_tx(&mut transaction, job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(Some(leased))
    }

    async fn heartbeat_job_lease(&self, heartbeat: JobLeaseHeartbeat) -> Result<LeasedJob> {
        validate_lease_duration(heartbeat.lease_duration_ms)?;

        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                heartbeat_at = statement_timestamp(),
                lease_expires_at = statement_timestamp() + ($3::double precision * INTERVAL '1 millisecond'),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $4
            "#,
        )
        .bind(heartbeat.guard.job_id.as_uuid())
        .bind(heartbeat.guard.run_token.as_uuid())
        .bind(u64_to_i64(heartbeat.lease_duration_ms)?)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_leased_job(heartbeat.guard.job_id).await
    }

    async fn succeed_leased_job(&self, completion: CompleteLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                summary_json = $4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $5
            "#,
        )
        .bind(completion.guard.job_id.as_uuid())
        .bind(completion.guard.run_token.as_uuid())
        .bind(JobStatus::Succeeded.as_str())
        .bind(completion.summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(completion.guard.job_id).await
    }

    async fn fail_leased_job(&self, failure: FailLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                error = $4,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                cancel_requested_at = NULL,
                cancel_reason = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $5
            "#,
        )
        .bind(failure.guard.job_id.as_uuid())
        .bind(failure.guard.run_token.as_uuid())
        .bind(JobStatus::Failed.as_str())
        .bind(failure.error)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(failure.guard.job_id).await
    }

    async fn request_job_cancellation(
        &self,
        request: RequestJobCancellation,
    ) -> Result<JobCancellationRequestRecord> {
        let current = self.get_job_or_not_found(request.job_id).await?;
        match current.status {
            JobStatus::Queued => {
                let updated = sqlx::query(
                    r#"
                    UPDATE jobs
                    SET
                        status = $2,
                        cancel_requested_at = COALESCE(cancel_requested_at, statement_timestamp()),
                        cancel_reason = COALESCE(cancel_reason, $3),
                        summary_json = NULL,
                        error = NULL,
                        worker_id = NULL,
                        run_token = NULL,
                        heartbeat_at = NULL,
                        lease_expires_at = NULL,
                        completed_at = statement_timestamp(),
                        updated_at = statement_timestamp()
                    WHERE id = $1
                        AND status = $4
                    "#,
                )
                .bind(request.job_id.as_uuid())
                .bind(JobStatus::Cancelled.as_str())
                .bind(request.reason)
                .bind(JobStatus::Queued.as_str())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;

                if updated.rows_affected() != 1 {
                    return Err(stale_job_lease_error());
                }

                self.get_cancellation_request_record(request.job_id, true)
                    .await
            }
            JobStatus::Running => {
                let updated = sqlx::query(
                    r#"
                    UPDATE jobs
                    SET
                        cancel_requested_at = COALESCE(cancel_requested_at, statement_timestamp()),
                        cancel_reason = COALESCE(cancel_reason, $2),
                        updated_at = statement_timestamp()
                    WHERE id = $1
                        AND status = $3
                    "#,
                )
                .bind(request.job_id.as_uuid())
                .bind(request.reason)
                .bind(JobStatus::Running.as_str())
                .execute(&self.pool)
                .await
                .map_err(database_error)?;

                if updated.rows_affected() != 1 {
                    return Err(stale_job_lease_error());
                }

                self.get_cancellation_request_record(request.job_id, false)
                    .await
            }
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled => {
                Err(TaruError::Conflict {
                    message: "terminal jobs cannot be cancelled".to_owned(),
                })
            }
        }
    }

    async fn cancel_leased_job(&self, cancellation: CancelLeasedJob) -> Result<Job> {
        let updated = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $3,
                summary_json = $4,
                error = NULL,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1
                AND run_token = $2
                AND status = $5
                AND cancel_requested_at IS NOT NULL
            "#,
        )
        .bind(cancellation.guard.job_id.as_uuid())
        .bind(cancellation.guard.run_token.as_uuid())
        .bind(JobStatus::Cancelled.as_str())
        .bind(cancellation.summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if updated.rows_affected() != 1 {
            return Err(stale_job_lease_error());
        }

        self.get_job_or_not_found(cancellation.guard.job_id).await
    }

    async fn recover_expired_job_leases(&self, recovery: RecoverExpiredJobLeases) -> Result<u64> {
        let kind = recovery.filter.kind.map(|kind| kind.as_str().to_owned());
        let resource_class = recovery.filter.resource_class;
        let job_id = recovery.filter.job_id.map(|id| id.as_uuid());
        let library_id = recovery.filter.library_id.map(|id| id.as_uuid());
        let source_id = recovery.filter.source_id.map(|id| id.as_uuid());
        let result = sqlx::query(
            r#"
            UPDATE jobs
            SET
                status = $8,
                error = $9,
                worker_id = NULL,
                run_token = NULL,
                heartbeat_at = NULL,
                lease_expires_at = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE status = $1
                AND lease_expires_at IS NOT NULL
                AND lease_expires_at < $2::timestamptz
                AND ($3::text IS NULL OR kind = $3)
                AND ($4::text IS NULL OR resource_class = $4)
                AND ($5::uuid IS NULL OR id = $5)
                AND ($6::uuid IS NULL OR library_id = $6)
                AND ($7::uuid IS NULL OR source_id = $7)
            "#,
        )
        .bind(JobStatus::Running.as_str())
        .bind(recovery.expired_before)
        .bind(kind.as_deref())
        .bind(resource_class.as_deref())
        .bind(job_id)
        .bind(library_id)
        .bind(source_id)
        .bind(JobStatus::Failed.as_str())
        .bind(recovery.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(result.rows_affected())
    }
}

#[async_trait::async_trait]
impl ArtworkTaskRepository for PostgresStore {
    async fn enqueue_artwork_task(&self, task: &ArtworkTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO artwork_tasks (
                id, image_id, kind, status, resource_class, attempts,
                max_attempts, error
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT(id) DO UPDATE SET
                image_id = excluded.image_id,
                kind = excluded.kind,
                status = excluded.status,
                resource_class = excluded.resource_class,
                attempts = excluded.attempts,
                max_attempts = excluded.max_attempts,
                error = excluded.error,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(task.id.as_uuid())
        .bind(task.image_id.as_uuid())
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
        let row = sqlx::query(&format!(
            r#"
            {ARTWORK_TASK_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_task).transpose()
    }

    async fn list_artwork_tasks(&self, page: PageRequest) -> Result<Vec<ArtworkTask>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {ARTWORK_TASK_SELECT}
            ORDER BY id ASC
            LIMIT $1 OFFSET $2
            "#
        ))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_task).collect()
    }
}

#[async_trait::async_trait]
impl ArtworkCandidateRepository for PostgresStore {
    async fn create_artwork_candidate(
        &self,
        candidate: NewArtworkCandidate,
    ) -> Result<ArtworkCandidateRecord> {
        let (kind, kind_key) = image_kind_to_parts(&candidate.kind);
        sqlx::query(
            r#"
            INSERT INTO addon_artwork_candidates (
                id, addon_id, side_effect_id, library_id, item_id, kind, kind_key,
                source_kind, source_uri, width, height, language, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT DO NOTHING
            "#,
        )
        .bind(candidate.id.as_uuid())
        .bind(candidate.addon_id.as_uuid())
        .bind(candidate.side_effect_id.as_uuid())
        .bind(candidate.library_id.as_uuid())
        .bind(candidate.item_id.as_uuid())
        .bind(&kind)
        .bind(&kind_key)
        .bind(candidate.source_kind.as_str())
        .bind(&candidate.source_uri)
        .bind(candidate.width.map(u32_to_i64))
        .bind(candidate.height.map(u32_to_i64))
        .bind(&candidate.language)
        .bind(ArtworkCandidateStatus::Proposed.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.find_artwork_candidate_by_source(
            candidate.addon_id,
            candidate.library_id,
            candidate.item_id,
            &candidate.kind,
            candidate.source_kind,
            &candidate.source_uri,
        )
        .await?
        .ok_or_else(|| TaruError::Database {
            message: "failed to load created addon artwork candidate".to_owned(),
        })
    }

    async fn get_artwork_candidate(
        &self,
        id: ArtworkCandidateId,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        get_artwork_candidate(&self.pool, id).await
    }

    async fn set_artwork_candidate_status(
        &self,
        id: ArtworkCandidateId,
        status: ArtworkCandidateStatus,
    ) -> Result<ArtworkCandidateRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let candidate = update_artwork_candidate_status_tx(&mut transaction, id, status).await?;
        transaction.commit().await.map_err(database_error)?;
        Ok(candidate)
    }

    async fn find_artwork_candidate_by_source(
        &self,
        addon_id: AddonId,
        library_id: LibraryId,
        item_id: MediaItemId,
        kind: &ImageKind,
        source_kind: ArtworkCandidateSourceKind,
        source_uri: &str,
    ) -> Result<Option<ArtworkCandidateRecord>> {
        let (kind, kind_key) = image_kind_to_parts(kind);
        let row = sqlx::query(&format!(
            r#"
            {ARTWORK_CANDIDATE_SELECT}
            WHERE addon_id = $1 AND library_id = $2 AND item_id = $3
                AND kind = $4 AND kind_key = $5 AND source_kind = $6
                AND source_uri = $7
            LIMIT 1
            "#
        ))
        .bind(addon_id.as_uuid())
        .bind(library_id.as_uuid())
        .bind(item_id.as_uuid())
        .bind(kind)
        .bind(kind_key)
        .bind(source_kind.as_str())
        .bind(source_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_artwork_candidate).transpose()
    }

    async fn list_artwork_candidates_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<Vec<ArtworkCandidateRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {ARTWORK_CANDIDATE_SELECT}
            WHERE item_id = $1
            ORDER BY created_at DESC, id ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(item_id.as_uuid())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_artwork_candidate).collect()
    }
}

#[async_trait::async_trait]
impl ManagedArtworkRepository for PostgresStore {
    async fn accept_managed_artwork_candidate_ingest(
        &self,
        candidate_id: ArtworkCandidateId,
        ingest: NewManagedArtworkIngest,
        job: NewJob,
    ) -> Result<ManagedArtworkAcceptanceRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        if let Some(existing) =
            get_managed_artwork_ingest_by_candidate_tx(&mut transaction, candidate_id).await?
        {
            let candidate = update_artwork_candidate_status_tx(
                &mut transaction,
                candidate_id,
                ArtworkCandidateStatus::Accepted,
            )
            .await?;
            let job = get_job_tx(&mut transaction, existing.job_id).await?;
            transaction.commit().await.map_err(database_error)?;
            return Ok(ManagedArtworkAcceptanceRecord {
                candidate,
                ingest: existing,
                job,
            });
        }

        insert_job_tx(&mut transaction, job).await?;
        let saved_ingest = insert_managed_artwork_ingest_tx(&mut transaction, ingest).await?;
        let candidate = update_artwork_candidate_status_tx(
            &mut transaction,
            candidate_id,
            ArtworkCandidateStatus::Accepted,
        )
        .await?;
        let job = get_job_tx(&mut transaction, saved_ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkAcceptanceRecord {
            candidate,
            ingest: saved_ingest,
            job,
        })
    }

    async fn get_managed_artwork_ingest(
        &self,
        id: ManagedArtworkIngestId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        get_managed_artwork_ingest(&self.pool, id).await
    }

    async fn find_managed_artwork_ingest_by_candidate(
        &self,
        candidate_id: ArtworkCandidateId,
    ) -> Result<Option<ManagedArtworkIngestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {MANAGED_ARTWORK_INGEST_SELECT}
            WHERE candidate_id = $1
            "#
        ))
        .bind(candidate_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_managed_artwork_ingest).transpose()
    }

    async fn claim_next_queued_managed_artwork_ingest(
        &self,
    ) -> Result<Option<ManagedArtworkIngestClaimRecord>> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let row = sqlx::query(
            r#"
            SELECT i.id::text AS id
            FROM managed_artwork_ingests i
            JOIN jobs j ON j.id = i.job_id
            JOIN addon_artwork_candidates c ON c.id = i.candidate_id
            WHERE i.status = $1
                AND j.status = $2
                AND j.kind = $3
                AND j.resource_class = $4
                AND c.status = $5
            ORDER BY i.created_at ASC, i.id ASC
            LIMIT 1
            FOR UPDATE OF i SKIP LOCKED
            "#,
        )
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .bind(JobStatus::Queued.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .bind(ArtworkCandidateStatus::Accepted.as_str())
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;

        let Some(row) = row else {
            transaction.commit().await.map_err(database_error)?;
            return Ok(None);
        };
        let ingest_id: ManagedArtworkIngestId = parse_id(row_get::<String>(&row, "id")?)?;

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                started_at = COALESCE(started_at, statement_timestamp()),
                completed_at = NULL,
                summary_json = NULL,
                error = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Running.as_str())
        .bind(JobStatus::Queued.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Ok(None);
        }

        let candidate = get_artwork_candidate_tx(&mut transaction, ingest.candidate_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "artwork_candidate",
                id: ingest.candidate_id.to_string(),
            })?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(Some(ManagedArtworkIngestClaimRecord {
            candidate,
            ingest,
            job,
        }))
    }

    async fn commit_managed_artwork_artifact(
        &self,
        ingest_id: ManagedArtworkIngestId,
        artifact: NewManagedArtworkArtifact,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        if artifact.ingest_id != ingest_id {
            return Err(TaruError::InvalidInput {
                message: "managed artwork artifact ingest_id must match committed ingest"
                    .to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let artifact_id = artifact.id;
        insert_managed_artwork_artifact_tx(&mut transaction, artifact).await?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                artifact_id = $3,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status IN ($4, $5)
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Stored.as_str())
        .bind(artifact_id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not claimable for artifact commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                summary_json = $3,
                error = NULL,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $4
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Succeeded.as_str())
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not running for artifact commit".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact = get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::Database {
                message: "stored managed artwork ingest is missing artifact metadata".to_owned(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestProcessingRecord {
            ingest,
            artifact: Some(artifact),
            job,
        })
    }

    async fn fail_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<ManagedArtworkIngestProcessingRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = $3,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status IN ($4, $5)
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .bind(&failure_code)
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not claimable for failure commit".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                error = $3,
                summary_json = $4,
                completed_at = statement_timestamp(),
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $5
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Failed.as_str())
        .bind(job_error)
        .bind(job_summary_json)
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not running for failure commit".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let artifact =
            get_managed_artwork_artifact_by_ingest_tx(&mut transaction, ingest.id).await?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestProcessingRecord {
            ingest,
            artifact,
            job,
        })
    }

    async fn fail_unfinished_managed_artwork_ingests(
        &self,
        failure_code: String,
        job_error: String,
        job_summary_json: Option<String>,
    ) -> Result<u64> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let result = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $1,
                failure_code = $2,
                updated_at = statement_timestamp()
            WHERE status IN ($3, $4)
                AND artifact_id IS NULL
                AND EXISTS (
                    SELECT 1
                    FROM jobs j
                    WHERE j.id = managed_artwork_ingests.job_id
                        AND j.kind = $5
                        AND j.resource_class = $6
                        AND j.status = $7
                )
            "#,
        )
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .bind(&failure_code)
        .bind(ManagedArtworkIngestStatus::Fetching.as_str())
        .bind(ManagedArtworkIngestStatus::Validating.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .bind(JobStatus::Running.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        let recovered = result.rows_affected();

        if recovered > 0 {
            sqlx::query(
                r#"
                UPDATE jobs
                SET status = $1,
                    error = $2,
                    summary_json = $3,
                    completed_at = statement_timestamp(),
                    updated_at = statement_timestamp()
                WHERE kind = $4
                    AND resource_class = $5
                    AND status = $6
                    AND EXISTS (
                        SELECT 1
                        FROM managed_artwork_ingests i
                        WHERE i.job_id = jobs.id
                            AND i.status = $7
                            AND i.failure_code = $8
                    )
                "#,
            )
            .bind(JobStatus::Failed.as_str())
            .bind(job_error)
            .bind(job_summary_json)
            .bind(JobKind::ManagedArtworkIngest.as_str())
            .bind("artwork.ingest")
            .bind(JobStatus::Running.as_str())
            .bind(ManagedArtworkIngestStatus::Failed.as_str())
            .bind(&failure_code)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(recovered)
    }

    async fn requeue_managed_artwork_ingest(
        &self,
        ingest_id: ManagedArtworkIngestId,
    ) -> Result<ManagedArtworkIngestRequeueRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest_id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest_id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;

        if job.kind != JobKind::ManagedArtworkIngest || job.resource_class != "artwork.ingest" {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not an artwork ingest job".to_owned(),
            });
        }

        if ingest.status == ManagedArtworkIngestStatus::Queued {
            if job.status != JobStatus::Queued {
                transaction.rollback().await.map_err(database_error)?;
                return Err(TaruError::Conflict {
                    message: "queued managed artwork ingest job is not queued".to_owned(),
                });
            }
            transaction.commit().await.map_err(database_error)?;
            return Ok(ManagedArtworkIngestRequeueRecord {
                ingest,
                job,
                requeued: false,
                had_failure: false,
            });
        }

        if ingest.status != ManagedArtworkIngestStatus::Failed || ingest.artifact_id.is_some() {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        if job.status != JobStatus::Failed {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest_updated = sqlx::query(
            r#"
            UPDATE managed_artwork_ingests
            SET status = $2,
                failure_code = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1 AND status = $3 AND artifact_id IS NULL
            "#,
        )
        .bind(ingest.id.as_uuid())
        .bind(ManagedArtworkIngestStatus::Queued.as_str())
        .bind(ManagedArtworkIngestStatus::Failed.as_str())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if ingest_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest is not failed or queued for requeue".to_owned(),
            });
        }

        let job_updated = sqlx::query(
            r#"
            UPDATE jobs
            SET status = $2,
                summary_json = NULL,
                error = NULL,
                started_at = NULL,
                completed_at = NULL,
                updated_at = statement_timestamp()
            WHERE id = $1
                AND status = $3
                AND kind = $4
                AND resource_class = $5
            "#,
        )
        .bind(ingest.job_id.as_uuid())
        .bind(JobStatus::Queued.as_str())
        .bind(JobStatus::Failed.as_str())
        .bind(JobKind::ManagedArtworkIngest.as_str())
        .bind("artwork.ingest")
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
        if job_updated.rows_affected() != 1 {
            transaction.rollback().await.map_err(database_error)?;
            return Err(TaruError::Conflict {
                message: "managed artwork ingest job is not failed for requeue".to_owned(),
            });
        }

        let ingest = get_managed_artwork_ingest_tx(&mut transaction, ingest.id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "managed_artwork_ingest",
                id: ingest.id.to_string(),
            })?;
        let job = get_job_tx(&mut transaction, ingest.job_id).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkIngestRequeueRecord {
            ingest,
            job,
            requeued: true,
            had_failure: true,
        })
    }

    async fn get_managed_artwork_artifact(
        &self,
        id: ManagedArtworkArtifactId,
    ) -> Result<Option<ManagedArtworkArtifactRecord>> {
        get_managed_artwork_artifact(&self.pool, id).await
    }

    async fn publish_selected_artwork(
        &self,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        publish_selected_artwork_tx(&self.pool, artifact_id, None).await
    }

    async fn publish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
        artifact_id: ManagedArtworkArtifactId,
    ) -> Result<SelectedArtworkPublicationRecord> {
        publish_selected_artwork_tx(&self.pool, artifact_id, Some((item_id, kind))).await
    }

    async fn unpublish_selected_artwork_for_item_kind(
        &self,
        item_id: MediaItemId,
        kind: ImageKind,
    ) -> Result<SelectedArtworkUnpublicationRecord> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let (kind_part, kind_key) = image_kind_to_parts(&kind);
        let unpublished =
            get_selected_artwork_by_slot_tx(&mut transaction, item_id, &kind_part, &kind_key)
                .await?;
        let artifact = if let Some(selected) = unpublished.as_ref() {
            Some(
                get_managed_artwork_artifact_tx(&mut transaction, selected.artifact_id)
                    .await?
                    .ok_or_else(|| TaruError::Database {
                        message: "selected artwork is linked to a missing managed artwork artifact"
                            .to_owned(),
                    })?,
            )
        } else {
            None
        };

        if let Some(selected) = unpublished.as_ref() {
            sqlx::query(
                r#"
                DELETE FROM selected_artworks
                WHERE id = $1
                "#,
            )
            .bind(selected.id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(SelectedArtworkUnpublicationRecord {
            item_id,
            kind,
            changed: unpublished.is_some(),
            unpublished,
            artifact,
        })
    }

    async fn get_selected_artwork(
        &self,
        id: SelectedArtworkId,
    ) -> Result<Option<SelectedArtworkRecord>> {
        get_selected_artwork(&self.pool, id).await
    }

    async fn list_selected_artwork_for_item(
        &self,
        item_id: MediaItemId,
    ) -> Result<Vec<SelectedArtworkRecord>> {
        let rows = sqlx::query(&format!(
            r#"
            {SELECTED_ARTWORK_SELECT}
            WHERE item_id = $1
            ORDER BY kind ASC, id ASC
            "#
        ))
        .bind(item_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter().map(row_to_selected_artwork).collect()
    }

    async fn get_managed_artwork_gallery_for_item(
        &self,
        item_id: MediaItemId,
        page: PageRequest,
    ) -> Result<ManagedArtworkGallerySnapshot> {
        let page = page.clamped();
        let candidates = managed_artwork_gallery_candidates(&self.pool, item_id, page).await?;
        let artifacts = managed_artwork_gallery_artifacts(&self.pool, item_id, page).await?;
        let selected = managed_artwork_gallery_selected(&self.pool, item_id).await?;
        let summary = ManagedArtworkGallerySummary {
            candidates: u32::try_from(candidates.len()).unwrap_or(u32::MAX),
            artifacts: u32::try_from(artifacts.len()).unwrap_or(u32::MAX),
            selected: u32::try_from(selected.len()).unwrap_or(u32::MAX),
        };

        Ok(ManagedArtworkGallerySnapshot {
            item_id,
            summary,
            candidates,
            artifacts,
            selected,
        })
    }

    async fn list_managed_artwork_artifact_lifecycle(
        &self,
        filter: ManagedArtworkArtifactLifecycleFilter,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactLifecycleSnapshot> {
        let summary = managed_artwork_artifact_lifecycle_summary(&self.pool).await?;
        let artifacts = managed_artwork_artifact_lifecycle_rows(&self.pool, filter, page).await?;

        Ok(ManagedArtworkArtifactLifecycleSnapshot { summary, artifacts })
    }

    async fn cleanup_unselected_managed_artwork_artifacts(
        &self,
        page: PageRequest,
    ) -> Result<ManagedArtworkArtifactCleanupReport> {
        let page = page.clamped();
        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let candidates = managed_artwork_artifact_lifecycle_rows_tx(
            &mut transaction,
            ManagedArtworkArtifactLifecycleFilter::CleanupCandidates,
            page,
        )
        .await?;
        let examined_artifacts = u32::try_from(candidates.len()).unwrap_or(u32::MAX);
        let mut cleaned_artifacts = Vec::new();

        for candidate in candidates {
            let artifact = candidate.artifact;
            let result = sqlx::query(
                r#"
                UPDATE managed_artwork_artifacts
                SET deleted_at = statement_timestamp(),
                    updated_at = statement_timestamp()
                WHERE id = $1
                    AND deleted_at IS NULL
                    AND NOT EXISTS (
                        SELECT 1
                        FROM selected_artworks s
                        WHERE s.artifact_id = managed_artwork_artifacts.id
                    )
                "#,
            )
            .bind(artifact.id.as_uuid())
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

            if result.rows_affected() == 1 {
                cleaned_artifacts.push(artifact);
            }
        }

        transaction.commit().await.map_err(database_error)?;

        Ok(ManagedArtworkArtifactCleanupReport {
            examined_artifacts,
            cleanup_candidate_artifacts: examined_artifacts,
            cleaned_artifacts,
        })
    }
}

#[async_trait::async_trait]
impl VfsCacheRepository for PostgresStore {
    async fn upsert_vfs_cache_object(&self, object: &VfsCachedObject) -> Result<()> {
        upsert_vfs_cache_object(&self.pool, object).await
    }

    async fn upsert_vfs_cache_listing(&self, listing: &VfsCachedListing) -> Result<()> {
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        upsert_vfs_cache_object_tx(&mut transaction, &listing.directory).await?;
        for entry in &listing.entries {
            upsert_vfs_cache_object_tx(&mut transaction, entry).await?;
        }

        sqlx::query(
            r#"
            INSERT INTO vfs_cache_listings (
                uri, scheme, fetched_at_ms, fresh_until_ms
            )
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(uri) DO UPDATE SET
                scheme = excluded.scheme,
                fetched_at_ms = excluded.fetched_at_ms,
                fresh_until_ms = excluded.fresh_until_ms,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(&listing.directory.uri)
        .bind(&listing.directory.scheme)
        .bind(listing.fetched_at_ms)
        .bind(listing.fresh_until_ms)
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;

        sqlx::query("DELETE FROM vfs_cache_listing_entries WHERE listing_uri = $1")
            .bind(&listing.directory.uri)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;

        for (index, entry) in listing.entries.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO vfs_cache_listing_entries (
                    listing_uri, entry_uri, sort_order
                )
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(&listing.directory.uri)
            .bind(&entry.uri)
            .bind(u64_to_i64(index as u64)?)
            .execute(&mut *transaction)
            .await
            .map_err(database_error)?;
        }

        transaction.commit().await.map_err(database_error)?;
        Ok(())
    }

    async fn get_vfs_cache_object(&self, uri: &str) -> Result<Option<VfsCachedObject>> {
        let row = sqlx::query(&format!(
            r#"
            {VFS_CACHE_OBJECT_SELECT}
            WHERE uri = $1
            "#
        ))
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cached_object).transpose()
    }

    async fn get_vfs_cache_listing(&self, uri: &str) -> Result<Option<VfsCachedListing>> {
        let listing_row = sqlx::query(
            r#"
            SELECT uri, fetched_at_ms, fresh_until_ms
            FROM vfs_cache_listings
            WHERE uri = $1
            "#,
        )
        .bind(uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        let Some(listing_row) = listing_row else {
            return Ok(None);
        };

        let directory =
            self.get_vfs_cache_object(uri)
                .await?
                .ok_or_else(|| TaruError::Database {
                    message: format!("VFS cache listing missing directory object: {uri}"),
                })?;

        let entry_rows = sqlx::query(&format!(
            r#"
            {VFS_CACHE_OBJECT_SELECT}
            JOIN vfs_cache_listing_entries entry ON entry.entry_uri = vfs_cache_objects.uri
            WHERE entry.listing_uri = $1
            ORDER BY entry.sort_order ASC, entry.entry_uri ASC
            "#
        ))
        .bind(uri)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(Some(VfsCachedListing {
            directory,
            entries: entry_rows
                .into_iter()
                .map(row_to_vfs_cached_object)
                .collect::<Result<Vec<_>>>()?,
            fetched_at_ms: row_get(&listing_row, "fetched_at_ms")?,
            fresh_until_ms: row_get(&listing_row, "fresh_until_ms")?,
        }))
    }

    async fn record_vfs_cache_failure(
        &self,
        failure: NewVfsCacheFailure,
    ) -> Result<VfsCacheFailure> {
        sqlx::query(
            r#"
            INSERT INTO vfs_cache_failures (
                uri, scheme, operation, failed_at_ms, failure_count, error
            )
            VALUES ($1, $2, $3, $4, 1, $5)
            ON CONFLICT(uri, operation) DO UPDATE SET
                scheme = excluded.scheme,
                failed_at_ms = excluded.failed_at_ms,
                failure_count = vfs_cache_failures.failure_count + 1,
                error = excluded.error,
                updated_at = statement_timestamp()
            "#,
        )
        .bind(&failure.uri)
        .bind(&failure.scheme)
        .bind(failure.operation.as_str())
        .bind(failure.failed_at_ms)
        .bind(&failure.error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_vfs_cache_failure(&failure.uri, failure.operation)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "vfs_cache_failure",
                id: format!("{}:{}", failure.uri, failure.operation.as_str()),
            })
    }

    async fn get_vfs_cache_failure(
        &self,
        uri: &str,
        operation: VfsCacheOperation,
    ) -> Result<Option<VfsCacheFailure>> {
        let row = sqlx::query(
            r#"
            SELECT uri, scheme, operation, failed_at_ms, failure_count, error
            FROM vfs_cache_failures
            WHERE uri = $1 AND operation = $2
            "#,
        )
        .bind(uri)
        .bind(operation.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_vfs_cache_failure).transpose()
    }

    async fn summarize_vfs_cache(&self, now_ms: i64) -> Result<VfsCacheSummary> {
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM vfs_cache_objects)::bigint AS object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings)::bigint AS listing_count,
                (SELECT COUNT(*) FROM vfs_cache_failures)::bigint AS failure_count,
                (SELECT COUNT(*) FROM vfs_cache_objects WHERE fresh_until_ms < $1)::bigint AS stale_object_count,
                (SELECT COUNT(*) FROM vfs_cache_listings WHERE fresh_until_ms < $1)::bigint AS stale_listing_count,
                (SELECT MAX(failed_at_ms) FROM vfs_cache_failures) AS last_failure_at_ms
            "#,
        )
        .bind(now_ms)
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        Ok(VfsCacheSummary {
            object_count: i64_to_u64(row_get::<i64>(&row, "object_count")?)?,
            listing_count: i64_to_u64(row_get::<i64>(&row, "listing_count")?)?,
            failure_count: i64_to_u64(row_get::<i64>(&row, "failure_count")?)?,
            stale_object_count: i64_to_u64(row_get::<i64>(&row, "stale_object_count")?)?,
            stale_listing_count: i64_to_u64(row_get::<i64>(&row, "stale_listing_count")?)?,
            last_failure_at_ms: row_get(&row, "last_failure_at_ms")?,
        })
    }
}

#[async_trait::async_trait]
impl StagingManifestRepository for PostgresStore {
    async fn upsert_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        upsert_staging_manifest_record(&self.pool, record).await
    }

    async fn reserve_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
        max_total_bytes: u64,
        now_ms: i64,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Reserved {
            return Err(TaruError::InvalidInput {
                message: "staging reservation must use reserved state".to_owned(),
            });
        }

        let mut transaction = self.pool.begin().await.map_err(database_error)?;
        let existing_row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE local_path = $1
            FOR UPDATE
            "#
        ))
        .bind(&record.local_path)
        .fetch_optional(&mut *transaction)
        .await
        .map_err(database_error)?;
        let existing = existing_row
            .map(row_to_staging_manifest_record)
            .transpose()?;

        if existing.as_ref().is_some_and(|existing| {
            matches!(
                existing.state,
                StagingState::Reserved | StagingState::Staging
            ) && !record_expired(existing, now_ms)
        }) {
            return Err(TaruError::storage_resource_budget_closed(
                record.source_uri,
                format!("staging input is already reserved: {}", record.local_path),
            ));
        }
        if existing.as_ref().is_some_and(|existing| {
            existing.state == StagingState::Leased || existing.active_leases > 0
        }) {
            return Err(TaruError::Conflict {
                message: format!("staging input is actively leased: {}", record.local_path),
            });
        }

        let incoming_bytes = record.size_bytes.unwrap_or(0);
        let existing_bytes = existing
            .as_ref()
            .filter(|existing| staging_state_counts_toward_budget(existing.state))
            .and_then(|existing| existing.size_bytes)
            .unwrap_or(0);
        let additional_bytes = incoming_bytes.saturating_sub(existing_bytes);
        let used_bytes = sum_staging_manifest_bytes_tx(&mut transaction).await?;
        let projected_bytes = used_bytes.saturating_add(additional_bytes);

        if additional_bytes > 0 && projected_bytes > max_total_bytes {
            return Err(TaruError::storage_staging_budget_exhausted(
                record.source_uri,
                format!(
                    "staging disk budget exhausted: used={used_bytes}, additional={additional_bytes}, max={max_total_bytes}",
                ),
            ));
        }

        let record = match existing {
            Some(existing) => NewStagingManifestRecord {
                id: existing.id,
                created_at_ms: existing.created_at_ms,
                ..record
            },
            None => record,
        };
        let saved = upsert_staging_manifest_record_tx(&mut transaction, record).await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(saved)
    }

    async fn start_staging_manifest_record(
        &self,
        id: StagingManifestId,
        started_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3
            WHERE id = $1
              AND state = $4
              AND active_leases = 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Staging.as_str())
        .bind(started_at_ms)
        .bind(StagingState::Reserved.as_str())
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} is not reserved"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn complete_staging_manifest_record(
        &self,
        record: NewStagingManifestRecord,
    ) -> Result<StagingManifestRecord> {
        if record.state != StagingState::Ready {
            return Err(TaruError::InvalidInput {
                message: "completed staging manifest must use ready state".to_owned(),
            });
        }

        self.upsert_staging_manifest_record(record).await
    }

    async fn fail_staging_manifest_record(
        &self,
        id: StagingManifestId,
        failed_at_ms: i64,
        validation_error: String,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0,
                validation_error = $4
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Failed.as_str())
        .bind(failed_at_ms)
        .bind(validation_error)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn expire_staging_manifest_record(
        &self,
        id: StagingManifestId,
        expired_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0
            WHERE id = $1
              AND active_leases = 0
              AND state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Expired.as_str())
        .bind(expired_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn mark_deleted_staging_manifest_record(
        &self,
        id: StagingManifestId,
        deleted_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3,
                active_leases = 0,
                expires_at_ms = NULL
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Deleted.as_str())
        .bind(deleted_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn acquire_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        leased_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET active_leases = active_leases + 1,
                state = $2,
                updated_at_ms = $3,
                last_accessed_at_ms = $3
            WHERE id = $1
              AND state IN ('ready', 'leased')
              AND active_leases >= 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Leased.as_str())
        .bind(leased_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} is not ready to lease"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn release_staging_manifest_lease(
        &self,
        id: StagingManifestId,
        released_at_ms: i64,
    ) -> Result<StagingManifestRecord> {
        let result = sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET active_leases = active_leases - 1,
                state = CASE
                    WHEN active_leases - 1 = 0 THEN $2
                    ELSE $3
                END,
                updated_at_ms = $4,
                last_accessed_at_ms = $4
            WHERE id = $1
              AND state = $3
              AND active_leases > 0
            "#,
        )
        .bind(id.as_uuid())
        .bind(StagingState::Ready.as_str())
        .bind(StagingState::Leased.as_str())
        .bind(released_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        if result.rows_affected() == 0 {
            return Err(TaruError::Conflict {
                message: format!("staging manifest {id} has no active lease to release"),
            });
        }

        self.get_staging_manifest_record(id)
            .await?
            .ok_or_else(|| TaruError::NotFound {
                entity: "staging_manifest_record",
                id: id.to_string(),
            })
    }

    async fn get_staging_manifest_record(
        &self,
        id: StagingManifestId,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE id = $1
            "#
        ))
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_staging_manifest_record).transpose()
    }

    async fn find_staging_manifest_record_by_path(
        &self,
        local_path: &str,
    ) -> Result<Option<StagingManifestRecord>> {
        let row = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE local_path = $1
            "#
        ))
        .bind(local_path)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_staging_manifest_record).transpose()
    }

    async fn list_staging_manifest_records(
        &self,
        purpose: Option<StagingPurpose>,
        state: Option<StagingState>,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        let page = page.clamped();
        let purpose = purpose.map(|purpose| purpose.as_str().to_owned());
        let state = state.map(|state| state.as_str().to_owned());
        let rows = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE ($1::text IS NULL OR purpose = $1)
              AND ($2::text IS NULL OR state = $2)
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT $3 OFFSET $4
            "#
        ))
        .bind(purpose.as_deref())
        .bind(state.as_deref())
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_staging_manifest_record)
            .collect()
    }

    async fn list_staging_cleanup_candidates(
        &self,
        now_ms: i64,
        page: PageRequest,
    ) -> Result<Vec<StagingManifestRecord>> {
        let page = page.clamped();
        let rows = sqlx::query(&format!(
            r#"
            {STAGING_MANIFEST_RECORD_SELECT}
            WHERE state IN ('reserved', 'staging', 'ready', 'failed', 'expired')
              AND active_leases = 0
              AND expires_at_ms IS NOT NULL
              AND expires_at_ms <= $1
            ORDER BY last_accessed_at_ms ASC, id ASC
            LIMIT $2 OFFSET $3
            "#
        ))
        .bind(now_ms)
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&self.pool)
        .await
        .map_err(database_error)?;

        rows.into_iter()
            .map(row_to_staging_manifest_record)
            .collect()
    }

    async fn touch_staging_manifest_record(
        &self,
        id: StagingManifestId,
        accessed_at_ms: i64,
    ) -> Result<Option<StagingManifestRecord>> {
        sqlx::query(
            r#"
            UPDATE staging_manifest_records
            SET last_accessed_at_ms = $2,
                updated_at_ms = $2
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(accessed_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_staging_manifest_record(id).await
    }

    async fn delete_staging_manifest_record(&self, id: StagingManifestId) -> Result<()> {
        sqlx::query("DELETE FROM staging_manifest_records WHERE id = $1")
            .bind(id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database_error)?;

        Ok(())
    }

    async fn sum_staging_manifest_bytes(&self) -> Result<u64> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(size_bytes), 0)::bigint AS total_bytes
            FROM staging_manifest_records
            WHERE size_bytes IS NOT NULL
              AND state IN ('reserved', 'staging', 'ready', 'leased')
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(database_error)?;

        i64_to_u64(row_get::<i64>(&row, "total_bytes")?)
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

async fn insert_job_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    job: NewJob,
) -> Result<()> {
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
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(job.id.as_uuid())
    .bind(job.kind.as_str())
    .bind(JobStatus::Queued.as_str())
    .bind(job.resource_class)
    .bind(job.library_id.map(|id| id.as_uuid()))
    .bind(job.source_id.map(|id| id.as_uuid()))
    .bind(job.input_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_job_tx(transaction: &mut sqlx::Transaction<'_, Postgres>, id: JobId) -> Result<Job> {
    let row = sqlx::query(JOB_SELECT_BY_ID)
        .bind(id.as_uuid())
        .fetch_optional(&mut **transaction)
        .await
        .map_err(database_error)?;

    row.map(row_to_job)
        .transpose()?
        .ok_or_else(|| TaruError::NotFound {
            entity: "job",
            id: id.to_string(),
        })
}

async fn get_artwork_candidate(
    pool: &PgPool,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {ARTWORK_CANDIDATE_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn get_artwork_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ArtworkCandidateId,
) -> Result<Option<ArtworkCandidateRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {ARTWORK_CANDIDATE_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_artwork_candidate).transpose()
}

async fn update_artwork_candidate_status_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ArtworkCandidateId,
    status: ArtworkCandidateStatus,
) -> Result<ArtworkCandidateRecord> {
    sqlx::query(
        r#"
        UPDATE addon_artwork_candidates
        SET status = $2,
            updated_at = statement_timestamp()
        WHERE id = $1
        "#,
    )
    .bind(id.as_uuid())
    .bind(status.as_str())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_artwork_candidate_tx(transaction, id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "artwork_candidate",
            id: id.to_string(),
        })
}

async fn insert_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    ingest: NewManagedArtworkIngest,
) -> Result<ManagedArtworkIngestRecord> {
    let (kind, kind_key) = image_kind_to_parts(&ingest.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_ingests (
            id, candidate_id, job_id, library_id, item_id, kind, kind_key,
            status, artifact_id, failure_code
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
    .bind(ingest.id.as_uuid())
    .bind(ingest.candidate_id.as_uuid())
    .bind(ingest.job_id.as_uuid())
    .bind(ingest.library_id.as_uuid())
    .bind(ingest.item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .bind(ingest.status.as_str())
    .bind(ingest.artifact_id.map(|id| id.as_uuid()))
    .bind(ingest.failure_code)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    get_managed_artwork_ingest_tx(transaction, ingest.id)
        .await?
        .ok_or_else(|| TaruError::Database {
            message: "failed to load created managed artwork ingest".to_owned(),
        })
}

async fn get_managed_artwork_ingest(
    pool: &PgPool,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn get_managed_artwork_ingest_by_candidate_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    candidate_id: ArtworkCandidateId,
) -> Result<Option<ManagedArtworkIngestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_INGEST_SELECT}
        WHERE candidate_id = $1
        "#
    ))
    .bind(candidate_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_ingest).transpose()
}

async fn insert_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    artifact: NewManagedArtworkArtifact,
) -> Result<()> {
    let (kind, kind_key) = image_kind_to_parts(&artifact.kind);
    sqlx::query(
        r#"
        INSERT INTO managed_artwork_artifacts (
            id, ingest_id, library_id, item_id, kind, kind_key, storage_uri,
            content_hash, width, height, byte_len, media_type
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        "#,
    )
    .bind(artifact.id.as_uuid())
    .bind(artifact.ingest_id.as_uuid())
    .bind(artifact.library_id.as_uuid())
    .bind(artifact.item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .bind(artifact.storage_uri)
    .bind(artifact.content_hash)
    .bind(artifact.width.map(u32_to_i64))
    .bind(artifact.height.map(u32_to_i64))
    .bind(optional_u64_to_i64(artifact.byte_len)?)
    .bind(artifact.media_type)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn get_managed_artwork_artifact(
    pool: &PgPool,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: ManagedArtworkArtifactId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn get_managed_artwork_artifact_by_ingest_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    ingest_id: ManagedArtworkIngestId,
) -> Result<Option<ManagedArtworkArtifactRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {MANAGED_ARTWORK_ARTIFACT_SELECT}
            AND ingest_id = $1
        "#
    ))
    .bind(ingest_id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_managed_artwork_artifact).transpose()
}

async fn publish_selected_artwork_tx(
    pool: &PgPool,
    artifact_id: ManagedArtworkArtifactId,
    expected_slot: Option<(MediaItemId, ImageKind)>,
) -> Result<SelectedArtworkPublicationRecord> {
    let mut transaction = pool.begin().await.map_err(database_error)?;
    let artifact = get_managed_artwork_artifact_tx(&mut transaction, artifact_id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "managed_artwork_artifact",
            id: artifact_id.to_string(),
        })?;

    if let Some((expected_item_id, expected_kind)) = expected_slot.as_ref() {
        if artifact.item_id != *expected_item_id || artifact.kind != *expected_kind {
            return Err(TaruError::Conflict {
                message: "managed artwork artifact does not match the requested item artwork slot"
                    .to_owned(),
            });
        }
    }

    get_managed_artwork_ingest_tx(&mut transaction, artifact.ingest_id)
        .await?
        .filter(|ingest| ingest.artifact_id == Some(artifact.id))
        .filter(|ingest| ingest.status == ManagedArtworkIngestStatus::Stored)
        .ok_or_else(|| TaruError::Conflict {
            message: "managed artwork artifact is not linked to a stored ingest".to_owned(),
        })?;

    let (kind, kind_key) = image_kind_to_parts(&artifact.kind);
    let existing =
        get_selected_artwork_by_slot_tx(&mut transaction, artifact.item_id, &kind, &kind_key)
            .await?;
    let selected_id = existing
        .as_ref()
        .map_or_else(SelectedArtworkId::new, |selected| selected.id);
    let changed = existing
        .as_ref()
        .is_none_or(|selected| selected.artifact_id != artifact.id);

    if let Some(existing) = existing {
        sqlx::query(
            r#"
            UPDATE selected_artworks
            SET library_id = $2,
                artifact_id = $3,
                updated_at = CASE
                    WHEN artifact_id = $3 THEN updated_at
                    ELSE statement_timestamp()
                END
            WHERE id = $1
            "#,
        )
        .bind(existing.id.as_uuid())
        .bind(artifact.library_id.as_uuid())
        .bind(artifact.id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO selected_artworks (
                id, library_id, item_id, kind, kind_key, artifact_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(selected_id.as_uuid())
        .bind(artifact.library_id.as_uuid())
        .bind(artifact.item_id.as_uuid())
        .bind(kind)
        .bind(kind_key)
        .bind(artifact.id.as_uuid())
        .execute(&mut *transaction)
        .await
        .map_err(database_error)?;
    }

    let selected_artwork = get_selected_artwork_tx(&mut transaction, selected_id)
        .await?
        .ok_or_else(|| TaruError::Database {
            message: "failed to load selected artwork publication".to_owned(),
        })?;
    transaction.commit().await.map_err(database_error)?;

    Ok(SelectedArtworkPublicationRecord {
        selected_artwork,
        artifact,
        changed,
    })
}

async fn get_selected_artwork(
    pool: &PgPool,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: SelectedArtworkId,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn get_selected_artwork_by_slot_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
    kind: &str,
    kind_key: &str,
) -> Result<Option<SelectedArtworkRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {SELECTED_ARTWORK_SELECT}
        WHERE item_id = $1 AND kind = $2 AND kind_key = $3
        "#
    ))
    .bind(item_id.as_uuid())
    .bind(kind)
    .bind(kind_key)
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_selected_artwork).transpose()
}

async fn managed_artwork_gallery_candidates(
    pool: &PgPool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryCandidateRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(
        r#"
        SELECT
            c.id::text AS id,
            c.addon_id::text AS addon_id,
            c.side_effect_id::text AS side_effect_id,
            c.library_id::text AS library_id,
            c.item_id::text AS item_id,
            c.kind,
            c.kind_key,
            c.source_kind,
            c.source_uri,
            c.width,
            c.height,
            c.language,
            c.status,
            to_char(c.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(c.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
            i.id::text AS ingest_id,
            i.job_id::text AS ingest_job_id,
            i.status AS ingest_status,
            i.artifact_id::text AS ingest_artifact_id,
            i.failure_code AS ingest_failure_code,
            to_char(i.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS ingest_created_at,
            to_char(i.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS ingest_updated_at,
            COUNT(s.id)::bigint AS selected_artwork_count
        FROM addon_artwork_candidates c
        LEFT JOIN managed_artwork_ingests i ON i.candidate_id = c.id
        LEFT JOIN managed_artwork_artifacts a ON a.ingest_id = i.id
            AND a.deleted_at IS NULL
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE c.item_id = $1
        GROUP BY
            c.id,
            c.addon_id,
            c.side_effect_id,
            c.library_id,
            c.item_id,
            c.kind,
            c.kind_key,
            c.source_kind,
            c.source_uri,
            c.width,
            c.height,
            c.language,
            c.status,
            c.created_at,
            c.updated_at,
            i.id,
            i.job_id,
            i.status,
            i.artifact_id,
            i.failure_code,
            i.created_at,
            i.updated_at
        ORDER BY c.created_at DESC, c.id ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(item_id.as_uuid())
    .bind(u32_to_i64(page.limit))
    .bind(u64_to_i64(page.offset)?)
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_candidate)
        .collect()
}

async fn managed_artwork_gallery_artifacts(
    pool: &PgPool,
    item_id: MediaItemId,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkGalleryArtifactRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(
        r#"
        SELECT
            a.id::text AS id,
            a.ingest_id::text AS ingest_id,
            i.candidate_id::text AS candidate_id,
            a.library_id::text AS library_id,
            a.item_id::text AS item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            (a.content_hash IS NOT NULL) AS has_content_hash,
            COUNT(s.id)::bigint AS selected_artwork_count,
            to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
            to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at
        FROM managed_artwork_artifacts a
        INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE a.item_id = $1 AND a.deleted_at IS NULL
        GROUP BY
            a.id,
            a.ingest_id,
            i.candidate_id,
            a.library_id,
            a.item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            a.content_hash,
            a.created_at,
            a.updated_at
        ORDER BY a.created_at DESC, a.id ASC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(item_id.as_uuid())
    .bind(u32_to_i64(page.limit))
    .bind(u64_to_i64(page.offset)?)
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_artifact)
        .collect()
}

async fn managed_artwork_gallery_selected(
    pool: &PgPool,
    item_id: MediaItemId,
) -> Result<Vec<ManagedArtworkGallerySelectedRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT
            s.id::text AS selected_id,
            s.library_id::text AS selected_library_id,
            s.item_id::text AS selected_item_id,
            s.kind AS selected_kind,
            s.kind_key AS selected_kind_key,
            s.artifact_id::text AS selected_artifact_id,
            to_char(s.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_created_at,
            to_char(s.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS selected_updated_at,
            a.id::text AS artifact_id,
            a.ingest_id::text AS artifact_ingest_id,
            i.candidate_id::text AS artifact_candidate_id,
            a.library_id::text AS artifact_library_id,
            a.item_id::text AS artifact_item_id,
            a.kind AS artifact_kind,
            a.kind_key AS artifact_kind_key,
            a.width AS artifact_width,
            a.height AS artifact_height,
            a.byte_len AS artifact_byte_len,
            a.media_type AS artifact_media_type,
            (a.content_hash IS NOT NULL) AS artifact_has_content_hash,
            COUNT(linked_s.id)::bigint AS artifact_selected_artwork_count,
            to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS artifact_created_at,
            to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS artifact_updated_at
        FROM selected_artworks s
        INNER JOIN managed_artwork_artifacts a ON a.id = s.artifact_id
            AND a.deleted_at IS NULL
        INNER JOIN managed_artwork_ingests i ON i.id = a.ingest_id
        LEFT JOIN selected_artworks linked_s ON linked_s.artifact_id = a.id
        WHERE s.item_id = $1
        GROUP BY
            s.id,
            s.library_id,
            s.item_id,
            s.kind,
            s.kind_key,
            s.artifact_id,
            s.created_at,
            s.updated_at,
            a.id,
            a.ingest_id,
            i.candidate_id,
            a.library_id,
            a.item_id,
            a.kind,
            a.kind_key,
            a.width,
            a.height,
            a.byte_len,
            a.media_type,
            a.content_hash,
            a.created_at,
            a.updated_at
        ORDER BY s.kind ASC, s.id ASC
        "#,
    )
    .bind(item_id.as_uuid())
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_gallery_selected)
        .collect()
}

async fn managed_artwork_artifact_lifecycle_summary(
    pool: &PgPool,
) -> Result<ManagedArtworkArtifactLifecycleSummary> {
    let rows = sqlx::query(
        r#"
        SELECT
            a.byte_len,
            COUNT(s.id)::bigint AS selected_artwork_count
        FROM managed_artwork_artifacts a
        LEFT JOIN selected_artworks s ON s.artifact_id = a.id
        WHERE a.deleted_at IS NULL
        GROUP BY a.id, a.byte_len
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(database_error)?;

    managed_artwork_lifecycle_summary_from_rows(rows)
}

async fn managed_artwork_artifact_lifecycle_rows(
    pool: &PgPool,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(lifecycle_select_sql(filter))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(pool)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_artifact_lifecycle)
        .collect()
}

async fn managed_artwork_artifact_lifecycle_rows_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    filter: ManagedArtworkArtifactLifecycleFilter,
    page: PageRequest,
) -> Result<Vec<ManagedArtworkArtifactLifecycleRecord>> {
    let page = page.clamped();
    let rows = sqlx::query(lifecycle_select_sql(filter))
        .bind(u32_to_i64(page.limit))
        .bind(u64_to_i64(page.offset)?)
        .fetch_all(&mut **transaction)
        .await
        .map_err(database_error)?;

    rows.into_iter()
        .map(row_to_managed_artwork_artifact_lifecycle)
        .collect()
}

fn managed_artwork_lifecycle_summary_from_rows(
    rows: Vec<PgRow>,
) -> Result<ManagedArtworkArtifactLifecycleSummary> {
    let mut summary = ManagedArtworkArtifactLifecycleSummary::default();
    for row in rows {
        let selected_artwork_count = i64_to_u32(row_get(&row, "selected_artwork_count")?)?;
        let byte_len = optional_i64_to_u64(row_get(&row, "byte_len")?)?;

        summary.total_artifacts = summary.total_artifacts.saturating_add(1);
        if selected_artwork_count == 0 {
            summary.cleanup_candidate_artifacts =
                summary.cleanup_candidate_artifacts.saturating_add(1);
        } else {
            summary.protected_artifacts = summary.protected_artifacts.saturating_add(1);
        }

        match byte_len {
            Some(byte_len) => {
                summary.known_total_bytes = summary.known_total_bytes.saturating_add(byte_len);
                if selected_artwork_count == 0 {
                    summary.known_cleanup_candidate_bytes = summary
                        .known_cleanup_candidate_bytes
                        .saturating_add(byte_len);
                } else {
                    summary.known_protected_bytes =
                        summary.known_protected_bytes.saturating_add(byte_len);
                }
            }
            None => {
                summary.unknown_byte_len_artifacts =
                    summary.unknown_byte_len_artifacts.saturating_add(1);
            }
        }
    }

    Ok(summary)
}

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT: &str = r#"
            SELECT
                a.id::text AS id,
                a.ingest_id::text AS ingest_id,
                a.library_id::text AS library_id,
                a.item_id::text AS item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                COUNT(s.id)::bigint AS selected_artwork_count
            FROM managed_artwork_artifacts a
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE a.deleted_at IS NULL
            GROUP BY
                a.id,
                a.ingest_id,
                a.library_id,
                a.item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                a.created_at,
                a.updated_at
            ORDER BY a.created_at ASC, a.id ASC
            LIMIT $1 OFFSET $2
            "#;

const MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT: &str = r#"
            SELECT
                a.id::text AS id,
                a.ingest_id::text AS ingest_id,
                a.library_id::text AS library_id,
                a.item_id::text AS item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                to_char(a.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
                to_char(a.updated_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS updated_at,
                COUNT(s.id)::bigint AS selected_artwork_count
            FROM managed_artwork_artifacts a
            LEFT JOIN selected_artworks s ON s.artifact_id = a.id
            WHERE a.deleted_at IS NULL
            GROUP BY
                a.id,
                a.ingest_id,
                a.library_id,
                a.item_id,
                a.kind,
                a.kind_key,
                a.storage_uri,
                a.content_hash,
                a.width,
                a.height,
                a.byte_len,
                a.media_type,
                a.created_at,
                a.updated_at
            HAVING COUNT(s.id) = 0
            ORDER BY a.created_at ASC, a.id ASC
            LIMIT $1 OFFSET $2
            "#;

const fn lifecycle_select_sql(filter: ManagedArtworkArtifactLifecycleFilter) -> &'static str {
    match filter {
        ManagedArtworkArtifactLifecycleFilter::All => MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_SELECT,
        ManagedArtworkArtifactLifecycleFilter::CleanupCandidates => {
            MANAGED_ARTWORK_ARTIFACT_LIFECYCLE_CLEANUP_SELECT
        }
    }
}

async fn upsert_vfs_cache_object(pool: &PgPool, object: &VfsCachedObject) -> Result<()> {
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
        .execute(pool)
        .await
        .map_err(database_error)?;

    Ok(())
}

async fn upsert_vfs_cache_object_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
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
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
        updated_at = statement_timestamp()
    "#
}

async fn upsert_staging_manifest_record(
    pool: &PgPool,
    record: NewStagingManifestRecord,
) -> Result<StagingManifestRecord> {
    sqlx::query(staging_manifest_record_upsert_sql())
        .bind(record.id.as_uuid())
        .bind(&record.source_uri)
        .bind(&record.source_scheme)
        .bind(record.purpose.as_str())
        .bind(&record.local_path)
        .bind(optional_u64_to_i64(record.size_bytes)?)
        .bind(&record.etag)
        .bind(&record.fingerprint)
        .bind(record.state.as_str())
        .bind(record.created_at_ms)
        .bind(record.updated_at_ms)
        .bind(record.last_accessed_at_ms)
        .bind(record.expires_at_ms)
        .bind(u32_to_i64(record.active_leases))
        .bind(&record.validation_error)
        .execute(pool)
        .await
        .map_err(database_error)?;

    get_staging_manifest_record(pool, record.id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "staging_manifest_record",
            id: record.id.to_string(),
        })
}

async fn upsert_staging_manifest_record_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    record: NewStagingManifestRecord,
) -> Result<StagingManifestRecord> {
    sqlx::query(staging_manifest_record_upsert_sql())
        .bind(record.id.as_uuid())
        .bind(&record.source_uri)
        .bind(&record.source_scheme)
        .bind(record.purpose.as_str())
        .bind(&record.local_path)
        .bind(optional_u64_to_i64(record.size_bytes)?)
        .bind(&record.etag)
        .bind(&record.fingerprint)
        .bind(record.state.as_str())
        .bind(record.created_at_ms)
        .bind(record.updated_at_ms)
        .bind(record.last_accessed_at_ms)
        .bind(record.expires_at_ms)
        .bind(u32_to_i64(record.active_leases))
        .bind(&record.validation_error)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    get_staging_manifest_record_tx(transaction, record.id)
        .await?
        .ok_or_else(|| TaruError::NotFound {
            entity: "staging_manifest_record",
            id: record.id.to_string(),
        })
}

fn staging_manifest_record_upsert_sql() -> &'static str {
    r#"
    INSERT INTO staging_manifest_records (
        id, source_uri, source_scheme, purpose, local_path, size_bytes,
        etag, fingerprint, state, created_at_ms, updated_at_ms,
        last_accessed_at_ms, expires_at_ms, active_leases, validation_error
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
    ON CONFLICT(id) DO UPDATE SET
        source_uri = excluded.source_uri,
        source_scheme = excluded.source_scheme,
        purpose = excluded.purpose,
        local_path = excluded.local_path,
        size_bytes = excluded.size_bytes,
        etag = excluded.etag,
        fingerprint = excluded.fingerprint,
        state = excluded.state,
        updated_at_ms = excluded.updated_at_ms,
        last_accessed_at_ms = excluded.last_accessed_at_ms,
        expires_at_ms = excluded.expires_at_ms,
        active_leases = excluded.active_leases,
        validation_error = excluded.validation_error
    "#
}

async fn get_staging_manifest_record(
    pool: &PgPool,
    id: StagingManifestId,
) -> Result<Option<StagingManifestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {STAGING_MANIFEST_RECORD_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(pool)
    .await
    .map_err(database_error)?;

    row.map(row_to_staging_manifest_record).transpose()
}

async fn get_staging_manifest_record_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: StagingManifestId,
) -> Result<Option<StagingManifestRecord>> {
    let row = sqlx::query(&format!(
        r#"
        {STAGING_MANIFEST_RECORD_SELECT}
        WHERE id = $1
        "#
    ))
    .bind(id.as_uuid())
    .fetch_optional(&mut **transaction)
    .await
    .map_err(database_error)?;

    row.map(row_to_staging_manifest_record).transpose()
}

async fn sum_staging_manifest_bytes_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
) -> Result<u64> {
    let row = sqlx::query(
        r#"
        SELECT COALESCE(SUM(size_bytes), 0)::bigint AS total_bytes
        FROM staging_manifest_records
        WHERE size_bytes IS NOT NULL
          AND state IN ('reserved', 'staging', 'ready', 'leased')
        "#,
    )
    .fetch_one(&mut **transaction)
    .await
    .map_err(database_error)?;

    i64_to_u64(row_get::<i64>(&row, "total_bytes")?)
}

fn record_expired(record: &StagingManifestRecord, now_ms: i64) -> bool {
    record
        .expires_at_ms
        .is_some_and(|expires_at_ms| expires_at_ms <= now_ms)
}

fn staging_state_counts_toward_budget(state: StagingState) -> bool {
    matches!(
        state,
        StagingState::Reserved | StagingState::Staging | StagingState::Ready | StagingState::Leased
    )
}

async fn get_leased_job_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    id: JobId,
) -> Result<LeasedJob> {
    let row = sqlx::query(JOB_LEASE_SELECT_BY_ID)
        .bind(id.as_uuid())
        .fetch_one(&mut **transaction)
        .await
        .map_err(database_error)?;

    row_to_leased_job(row)
}

async fn upsert_media_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item: &MediaItem,
) -> Result<()> {
    let metadata_json = serde_json::to_string(&item.metadata).map_err(database_error)?;

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
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb)
        ON CONFLICT(id) DO UPDATE SET
            kind = excluded.kind,
            parent_id = excluded.parent_id,
            title = excluded.title,
            original_title = excluded.original_title,
            sort_title = excluded.sort_title,
            overview = excluded.overview,
            release_date = excluded.release_date,
            metadata_json = excluded.metadata_json,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(item.id.as_uuid())
    .bind(item.kind.as_str())
    .bind(item.parent_id.map(|id| id.as_uuid()))
    .bind(&item.metadata.title)
    .bind(&item.metadata.original_title)
    .bind(&item.metadata.sort_title)
    .bind(&item.metadata.overview)
    .bind(&item.metadata.release_date)
    .bind(metadata_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM media_item_external_ids WHERE item_id = $1")
        .bind(item.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &item.metadata.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO media_item_external_ids (item_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(item.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_media_source_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    source: &MediaSource,
) -> Result<()> {
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
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT(id) DO UPDATE SET
            library_id = excluded.library_id,
            item_id = excluded.item_id,
            locator = excluded.locator,
            file_name = excluded.file_name,
            size_bytes = excluded.size_bytes,
            fingerprint = excluded.fingerprint,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(source.id.as_uuid())
    .bind(source.library_id.as_uuid())
    .bind(source.item_id.as_uuid())
    .bind(&source.locator)
    .bind(&source.file_name)
    .bind(source.size_bytes.map(u64_to_i64).transpose()?)
    .bind(&source.fingerprint)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_library_item_state_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    state: &LibraryItemState,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO library_item_states (library_id, item_id, provisional)
        VALUES ($1, $2, $3)
        ON CONFLICT(library_id, item_id) DO UPDATE SET
            provisional = excluded.provisional,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(state.library_id.as_uuid())
    .bind(state.item_id.as_uuid())
    .bind(state.provisional)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_source_state_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    state: &SourceState,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO source_states (
            library_id, source_id, uri, size_bytes, modified_at, etag,
            fingerprint, last_seen_scan_id, tombstoned
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT(library_id, uri) DO UPDATE SET
            source_id = excluded.source_id,
            size_bytes = excluded.size_bytes,
            modified_at = excluded.modified_at,
            etag = excluded.etag,
            fingerprint = excluded.fingerprint,
            last_seen_scan_id = excluded.last_seen_scan_id,
            tombstoned = excluded.tombstoned,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(state.library_id.as_uuid())
    .bind(state.source_id.map(|id| id.as_uuid()))
    .bind(&state.uri)
    .bind(state.size_bytes.map(u64_to_i64).transpose()?)
    .bind(&state.modified_at)
    .bind(&state.etag)
    .bind(&state.fingerprint)
    .bind(state.last_seen_scan_id.as_uuid())
    .bind(state.tombstoned)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_local_inference_evidence_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    evidence: &LocalInferenceEvidence,
) -> Result<()> {
    let (evidence_source, evidence_source_key) =
        local_inference_evidence_source_to_parts(&evidence.evidence_source);

    sqlx::query(
        r#"
        INSERT INTO local_inference_evidence (
            id,
            source_id,
            inferred_kind,
            inferred_title,
            inferred_year,
            inferred_season,
            inferred_episode,
            confidence_milli,
            evidence_source,
            evidence_source_key,
            evidence_value,
            inference_version
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT(
            source_id,
            evidence_source,
            evidence_source_key,
            inference_version
        ) DO UPDATE SET
            inferred_kind = excluded.inferred_kind,
            inferred_title = excluded.inferred_title,
            inferred_year = excluded.inferred_year,
            inferred_season = excluded.inferred_season,
            inferred_episode = excluded.inferred_episode,
            confidence_milli = excluded.confidence_milli,
            evidence_value = excluded.evidence_value,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(evidence.id.as_uuid())
    .bind(evidence.source_id.as_uuid())
    .bind(evidence.inferred_kind.as_str())
    .bind(&evidence.inferred_title)
    .bind(evidence.inferred_year.map(i64::from))
    .bind(evidence.inferred_season.map(u32_to_i64))
    .bind(evidence.inferred_episode.map(u32_to_i64))
    .bind(evidence.confidence_milli.map(i64::from))
    .bind(evidence_source)
    .bind(evidence_source_key)
    .bind(&evidence.evidence_value)
    .bind(&evidence.inference_version)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_search_projection_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    projection: &CatalogSearchProjection,
) -> Result<()> {
    let aliases_json = serde_json::to_string(&projection.aliases).map_err(database_error)?;
    let facets = projection.facet_labels();
    let facets_json = serde_json::to_string(&facets).map_err(database_error)?;
    let facets_text = facets.join(" ");
    let sort_keys_json = serde_json::to_string(&projection.sort_keys).map_err(database_error)?;
    let provider_identifiers_json =
        serde_json::to_string(&projection.provider_identifiers).map_err(database_error)?;

    sqlx::query(
        r#"
        INSERT INTO search_documents (
            item_id, projection_version, title, body, aliases_json, facets_json,
            facets_text, sort_keys_json, provider_identifiers_json
        )
        VALUES ($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7, $8::jsonb, $9::jsonb)
        ON CONFLICT(item_id) DO UPDATE SET
            projection_version = excluded.projection_version,
            title = excluded.title,
            body = excluded.body,
            aliases_json = excluded.aliases_json,
            facets_json = excluded.facets_json,
            facets_text = excluded.facets_text,
            sort_keys_json = excluded.sort_keys_json,
            provider_identifiers_json = excluded.provider_identifiers_json,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(projection.item_id.as_uuid())
    .bind(i64::from(projection.projection_version))
    .bind(&projection.title)
    .bind(projection.searchable_text())
    .bind(aliases_json)
    .bind(facets_json)
    .bind(facets_text)
    .bind(sort_keys_json)
    .bind(provider_identifiers_json)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn resolve_ingestion_failure_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    library_id: LibraryId,
    phase: IngestionFailurePhase,
    target_uri: &str,
    resolved_at_ms: i64,
) -> Result<u64> {
    let result = sqlx::query(
        r#"
        UPDATE ingestion_failures
        SET
            status = $4,
            resolved_at_ms = $5,
            ignored_at_ms = NULL,
            updated_at = statement_timestamp()
        WHERE library_id = $1 AND phase = $2 AND target_uri = $3
        "#,
    )
    .bind(library_id.as_uuid())
    .bind(phase.as_str())
    .bind(target_uri)
    .bind(IngestionFailureStatus::Resolved.as_str())
    .bind(resolved_at_ms)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(result.rows_affected())
}

async fn upsert_field_lock_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    lock: &MetadataFieldLock,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&lock.source);

    sqlx::query(
        r#"
        INSERT INTO metadata_field_locks (item_id, field, locked, source, source_key)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(item_id, field) DO UPDATE SET
            locked = excluded.locked,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(lock.item_id.as_uuid())
    .bind(lock.field.as_str())
    .bind(lock.locked)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_raw_response_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    response: &ProviderRawResponse,
) -> Result<()> {
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
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(item_id, provider, provider_key) DO UPDATE SET
            body_json = excluded.body_json,
            fetched_at = excluded.fetched_at,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(response.item_id.as_uuid())
    .bind(provider)
    .bind(provider_key)
    .bind(&response.body_json)
    .bind(&response.fetched_at)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_subject_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    subject: &ProviderSubject,
) -> Result<()> {
    let (provider, provider_key) = provider_to_parts(&subject.provider);
    let (subject_kind, subject_kind_key) = provider_subject_kind_to_parts(&subject.subject_kind);

    sqlx::query(
        r#"
        INSERT INTO provider_subjects (
            id,
            provider,
            provider_key,
            subject_kind,
            subject_kind_key,
            subject_key,
            title,
            release_year,
            locale
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT(id) DO UPDATE SET
            provider = excluded.provider,
            provider_key = excluded.provider_key,
            subject_kind = excluded.subject_kind,
            subject_kind_key = excluded.subject_kind_key,
            subject_key = excluded.subject_key,
            title = excluded.title,
            release_year = excluded.release_year,
            locale = excluded.locale,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(subject.id.as_uuid())
    .bind(provider)
    .bind(provider_key)
    .bind(subject_kind)
    .bind(subject_kind_key)
    .bind(&subject.subject_key)
    .bind(&subject.title)
    .bind(subject.release_year.map(i64::from))
    .bind(&subject.locale)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_provider_mapping_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    mapping: &ProviderMapping,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&mapping.source);

    sqlx::query(
        r#"
        INSERT INTO provider_mappings (
            id,
            item_id,
            subject_id,
            status,
            confidence_milli,
            source,
            source_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT(id) DO UPDATE SET
            item_id = excluded.item_id,
            subject_id = excluded.subject_id,
            status = excluded.status,
            confidence_milli = excluded.confidence_milli,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(mapping.id.as_uuid())
    .bind(mapping.item_id.as_uuid())
    .bind(mapping.subject_id.as_uuid())
    .bind(mapping.status.as_str())
    .bind(mapping.confidence_milli.map(i64::from))
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn library_ids_for_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
) -> Result<Vec<LibraryId>> {
    let rows = sqlx::query(
        r#"
        SELECT library_id::text AS library_id
        FROM library_item_states
        WHERE item_id = $1
        ORDER BY library_id ASC
        "#,
    )
    .bind(item_id.as_uuid())
    .fetch_all(&mut **transaction)
    .await
    .map_err(database_error)?;

    rows.into_iter()
        .map(|row| parse_id(row_get::<String>(&row, "library_id")?))
        .collect()
}

async fn replace_item_catalog_graph_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_id: MediaItemId,
    replacement: &CatalogItemGraphReplacement,
) -> Result<()> {
    sqlx::query("DELETE FROM item_credits WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_genres WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_tags WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM collection_items WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    sqlx::query("DELETE FROM item_studios WHERE item_id = $1")
        .bind(item_id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for person in &replacement.people {
        upsert_person_tx(transaction, person).await?;
    }
    for credit in &replacement.credits {
        upsert_item_credit_tx(transaction, credit).await?;
    }
    for genre in &replacement.genres {
        upsert_genre_tx(transaction, genre).await?;
    }
    for item_genre in &replacement.item_genres {
        upsert_item_genre_tx(transaction, item_genre).await?;
    }
    for tag in &replacement.tags {
        upsert_tag_tx(transaction, tag).await?;
    }
    for item_tag in &replacement.item_tags {
        upsert_item_tag_tx(transaction, item_tag).await?;
    }
    for collection in &replacement.collections {
        upsert_collection_tx(transaction, collection).await?;
    }
    for collection_item in &replacement.collection_items {
        upsert_collection_item_tx(transaction, collection_item).await?;
    }
    for studio in &replacement.studios {
        upsert_studio_tx(transaction, studio).await?;
    }
    for item_studio in &replacement.item_studios {
        upsert_item_studio_tx(transaction, item_studio).await?;
    }
    for image in &replacement.images {
        upsert_image_asset_tx(transaction, image).await?;
    }

    Ok(())
}

async fn upsert_person_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    person: &Person,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO people (id, name, sort_name, overview)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            sort_name = excluded.sort_name,
            overview = excluded.overview,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(person.id.as_uuid())
    .bind(&person.name)
    .bind(&person.sort_name)
    .bind(&person.overview)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM person_external_ids WHERE person_id = $1")
        .bind(person.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &person.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO person_external_ids (person_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(person_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(person.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_item_credit_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    credit: &ItemCredit,
) -> Result<()> {
    let (role, role_key) = credit_role_to_parts(&credit.role);
    sqlx::query(
        r#"
        INSERT INTO item_credits (
            item_id, person_id, role, role_key, character, sort_order
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT(item_id, person_id, role, role_key, character) DO UPDATE SET
            sort_order = excluded.sort_order,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(credit.item_id.as_uuid())
    .bind(credit.person_id.as_uuid())
    .bind(role)
    .bind(role_key)
    .bind(credit.character.clone().unwrap_or_default())
    .bind(credit.sort_order.map(u32_to_i64))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    genre: &Genre,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&genre.source);
    sqlx::query(
        r#"
        INSERT INTO genres (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(genre.id.as_uuid())
    .bind(&genre.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_genre_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_genre: &ItemGenre,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_genres (item_id, genre_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, genre_id) DO NOTHING
        "#,
    )
    .bind(item_genre.item_id.as_uuid())
    .bind(item_genre.genre_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_tag_tx(transaction: &mut sqlx::Transaction<'_, Postgres>, tag: &Tag) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&tag.source);
    sqlx::query(
        r#"
        INSERT INTO tags (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(tag.id.as_uuid())
    .bind(&tag.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_item_tag_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_tag: &ItemTag,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_tags (item_id, tag_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, tag_id) DO NOTHING
        "#,
    )
    .bind(item_tag.item_id.as_uuid())
    .bind(item_tag.tag_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_collection_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    collection: &Collection,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&collection.source);

    sqlx::query(
        r#"
        INSERT INTO collections (id, name, overview, source, source_key)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            overview = excluded.overview,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(collection.id.as_uuid())
    .bind(&collection.name)
    .bind(&collection.overview)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM collection_external_ids WHERE collection_id = $1")
        .bind(collection.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &collection.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO collection_external_ids (collection_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(collection_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(collection.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_collection_item_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item: &CollectionItem,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO collection_items (collection_id, item_id, sort_order)
        VALUES ($1, $2, $3)
        ON CONFLICT(collection_id, item_id) DO UPDATE SET
            sort_order = excluded.sort_order
        "#,
    )
    .bind(item.collection_id.as_uuid())
    .bind(item.item_id.as_uuid())
    .bind(item.sort_order.map(u32_to_i64))
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    studio: &Studio,
) -> Result<()> {
    let (source, source_key) = metadata_source_to_parts(&studio.source);

    sqlx::query(
        r#"
        INSERT INTO studios (id, name, source, source_key)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            source = excluded.source,
            source_key = excluded.source_key,
            updated_at = statement_timestamp()
        "#,
    )
    .bind(studio.id.as_uuid())
    .bind(&studio.name)
    .bind(source)
    .bind(source_key)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    sqlx::query("DELETE FROM studio_external_ids WHERE studio_id = $1")
        .bind(studio.id.as_uuid())
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;

    for external_id in &studio.external_ids {
        let (provider, provider_key) = provider_to_parts(&external_id.provider);
        sqlx::query(
            r#"
            INSERT INTO studio_external_ids (studio_id, provider, provider_key, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT(studio_id, provider, provider_key, value) DO NOTHING
            "#,
        )
        .bind(studio.id.as_uuid())
        .bind(provider)
        .bind(provider_key)
        .bind(&external_id.value)
        .execute(&mut **transaction)
        .await
        .map_err(database_error)?;
    }

    Ok(())
}

async fn upsert_item_studio_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    item_studio: &ItemStudio,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO item_studios (item_id, studio_id)
        VALUES ($1, $2)
        ON CONFLICT(item_id, studio_id) DO NOTHING
        "#,
    )
    .bind(item_studio.item_id.as_uuid())
    .bind(item_studio.studio_id.as_uuid())
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

async fn upsert_image_asset_tx(
    transaction: &mut sqlx::Transaction<'_, Postgres>,
    image: &ImageAsset,
) -> Result<()> {
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
        VALUES ($1, $2, $3::uuid, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
            updated_at = statement_timestamp()
        "#,
    )
    .bind(image.id.as_uuid())
    .bind(owner_kind)
    .bind(owner_id)
    .bind(kind)
    .bind(kind_key)
    .bind(&image.source_uri)
    .bind(provider)
    .bind(provider_key)
    .bind(&image.cache_uri)
    .bind(image.width.map(u32_to_i64))
    .bind(image.height.map(u32_to_i64))
    .bind(&image.language)
    .bind(image.selected)
    .bind(&image.content_hash)
    .bind(&image.etag)
    .execute(&mut **transaction)
    .await
    .map_err(database_error)?;

    Ok(())
}

impl PostgresStore {
    async fn rows_to_media_items(&self, rows: Vec<PgRow>) -> Result<Vec<MediaItem>> {
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
            WHERE item_id = $1
            ORDER BY provider ASC, provider_key ASC, value ASC
            "#,
        )
        .bind(item_id.as_uuid())
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

    async fn list_catalog_external_ids<T>(
        &self,
        table: &str,
        owner_column: &str,
        owner_id: T,
    ) -> Result<Vec<ExternalId>>
    where
        T: Display,
    {
        validate_catalog_external_id_lookup(table, owner_column)?;
        let query = format!(
            "SELECT provider, provider_key, value FROM {table} WHERE {owner_column} = $1::uuid ORDER BY provider ASC, provider_key ASC, value ASC"
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

    async fn governance_row_to_record(&self, row: PgRow) -> Result<CatalogGovernanceItemRecord> {
        let item_id = parse_id(row_get::<String>(&row, "id")?)?;
        let library_id = parse_id(row_get::<String>(&row, "governance_library_id")?)?;
        let source_count = i64_to_u32(row_get(&row, "source_count")?)?;
        let representative_source_id =
            parse_optional_id(row_get::<Option<String>>(&row, "representative_source_id")?)?;
        let representative_file_name = row_get(&row, "representative_file_name")?;
        let provider_mapping_count = i64_to_u32(row_get(&row, "provider_mapping_count")?)?;
        let accepted_provider_mapping_count =
            i64_to_u32(row_get(&row, "accepted_provider_mapping_count")?)?;
        let duplicate_relationship_count =
            i64_to_u32(row_get(&row, "duplicate_relationship_count")?)?;
        let external_ids = self.list_external_ids(item_id).await?;
        let item = row_to_media_item(row, external_ids)?;
        let best_local_inference = self
            .best_local_inference_evidence_for_item_library(item.id, library_id)
            .await?;

        Ok(CatalogGovernanceItemRecord {
            item,
            library_id,
            source_count,
            representative_source_id,
            representative_file_name,
            best_local_inference,
            provider_mapping_count,
            accepted_provider_mapping_count,
            duplicate_relationship_count,
        })
    }

    async fn best_local_inference_evidence_for_item_library(
        &self,
        item_id: MediaItemId,
        library_id: LibraryId,
    ) -> Result<Option<LocalInferenceEvidence>> {
        let row = sqlx::query(
            r#"
            SELECT
                evidence.id::text AS id,
                evidence.source_id::text AS source_id,
                evidence.inferred_kind,
                evidence.inferred_title,
                evidence.inferred_year,
                evidence.inferred_season,
                evidence.inferred_episode,
                evidence.confidence_milli,
                evidence.evidence_source,
                evidence.evidence_source_key,
                evidence.evidence_value,
                evidence.inference_version
            FROM local_inference_evidence AS evidence
            INNER JOIN media_sources AS source
                ON source.id = evidence.source_id
            WHERE source.item_id = $1
              AND source.library_id = $2
            ORDER BY
                evidence.confidence_milli IS NULL ASC,
                evidence.confidence_milli DESC,
                evidence.updated_at DESC,
                evidence.inference_version DESC,
                evidence.id ASC
            LIMIT 1
            "#,
        )
        .bind(item_id.as_uuid())
        .bind(library_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_local_inference_evidence).transpose()
    }

    async fn get_ingestion_failure(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
    ) -> Result<Option<IngestionFailureRecord>> {
        let row = sqlx::query(
            r#"
            SELECT
                library_id::text AS library_id,
                phase,
                target_uri,
                target_kind,
                job_id::text AS job_id,
                scan_id::text AS scan_id,
                source_id::text AS source_id,
                failure_class,
                status,
                message,
                retryable,
                attempts,
                first_failed_at_ms,
                last_failed_at_ms,
                resolved_at_ms,
                ignored_at_ms
            FROM ingestion_failures
            WHERE library_id = $1 AND phase = $2 AND target_uri = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_str())
        .bind(target_uri)
        .fetch_optional(&self.pool)
        .await
        .map_err(database_error)?;

        row.map(row_to_ingestion_failure).transpose()
    }

    async fn update_ingestion_failure_status(
        &self,
        library_id: LibraryId,
        phase: IngestionFailurePhase,
        target_uri: &str,
        status: IngestionFailureStatus,
        resolved_at_ms: Option<i64>,
        ignored_at_ms: Option<i64>,
    ) -> Result<Option<IngestionFailureRecord>> {
        sqlx::query(
            r#"
            UPDATE ingestion_failures
            SET
                status = $4,
                resolved_at_ms = $5,
                ignored_at_ms = $6,
                updated_at = statement_timestamp()
            WHERE library_id = $1 AND phase = $2 AND target_uri = $3
            "#,
        )
        .bind(library_id.as_uuid())
        .bind(phase.as_str())
        .bind(target_uri)
        .bind(status.as_str())
        .bind(resolved_at_ms)
        .bind(ignored_at_ms)
        .execute(&self.pool)
        .await
        .map_err(database_error)?;

        self.get_ingestion_failure(library_id, phase, target_uri)
            .await
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
        let row = sqlx::query(&format!("{WEBHOOK_DELIVERY_ATTEMPT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
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
        let row = sqlx::query(&format!("{AUTOMATION_ARTIFACT_SELECT} WHERE id = $1"))
            .bind(id.as_uuid())
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
}

fn row_to_library(row: PgRow) -> Result<Library> {
    let roots_json: String = row_get(&row, "roots_json")?;
    let options_json: String = row_get(&row, "options_json")?;

    Ok(Library {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        roots: serde_json::from_str(&roots_json).map_err(database_error)?,
        options: serde_json::from_str(&options_json).map_err(database_error)?,
    })
}

fn row_to_library_item_state(row: PgRow) -> Result<LibraryItemState> {
    Ok(LibraryItemState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provisional: row_get(&row, "provisional")?,
    })
}

fn row_to_media_item(row: PgRow, external_ids: Vec<ExternalId>) -> Result<MediaItem> {
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
        kind: parse_media_kind(row_get(&row, "kind")?)?,
        parent_id: parse_optional_id(row_get::<Option<String>>(&row, "parent_id")?)?,
        metadata,
    })
}

fn row_to_media_source(row: PgRow) -> Result<MediaSource> {
    Ok(MediaSource {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        locator: row_get(&row, "locator")?,
        file_name: row_get(&row, "file_name")?,
        size_bytes: row_get::<Option<i64>>(&row, "size_bytes")?
            .map(i64_to_u64)
            .transpose()?,
        fingerprint: row_get(&row, "fingerprint")?,
    })
}

fn row_to_stream_info(row: PgRow) -> Result<MediaStreamInfo> {
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

fn row_to_local_inference_evidence(row: PgRow) -> Result<LocalInferenceEvidence> {
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

fn row_to_scan_snapshot(row: PgRow) -> Result<ScanSnapshot> {
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

fn row_to_directory_snapshot(row: PgRow) -> Result<DirectorySnapshot> {
    Ok(DirectorySnapshot {
        scan_id: parse_id(row_get::<String>(&row, "scan_id")?)?,
        uri: row_get(&row, "uri")?,
        etag: row_get(&row, "etag")?,
        modified_at: row_get(&row, "modified_at")?,
        child_count: i64_to_u64(row_get(&row, "child_count")?)?,
    })
}

fn row_to_source_state(row: PgRow) -> Result<SourceState> {
    Ok(SourceState {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        uri: row_get(&row, "uri")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        modified_at: row_get(&row, "modified_at")?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        last_seen_scan_id: parse_id(row_get::<String>(&row, "last_seen_scan_id")?)?,
        tombstoned: row_get(&row, "tombstoned")?,
    })
}

fn row_to_ingestion_failure(row: PgRow) -> Result<IngestionFailureRecord> {
    Ok(IngestionFailureRecord {
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        job_id: parse_optional_id(row_get::<Option<String>>(&row, "job_id")?)?,
        scan_id: parse_optional_id(row_get::<Option<String>>(&row, "scan_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        phase: IngestionFailurePhase::parse(&row_get::<String>(&row, "phase")?)?,
        target_uri: row_get(&row, "target_uri")?,
        target_kind: row_get(&row, "target_kind")?,
        failure_class: IngestionFailureClass::parse(&row_get::<String>(&row, "failure_class")?)?,
        status: IngestionFailureStatus::parse(&row_get::<String>(&row, "status")?)?,
        message: row_get(&row, "message")?,
        retryable: row_get(&row, "retryable")?,
        attempts: i64_to_u32(row_get(&row, "attempts")?)?,
        first_failed_at_ms: row_get(&row, "first_failed_at_ms")?,
        last_failed_at_ms: row_get(&row, "last_failed_at_ms")?,
        resolved_at_ms: row_get(&row, "resolved_at_ms")?,
        ignored_at_ms: row_get(&row, "ignored_at_ms")?,
    })
}

fn row_to_metadata_field_lock(row: PgRow) -> Result<MetadataFieldLock> {
    Ok(MetadataFieldLock {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        field: metadata_field_from_str(&row_get::<String>(&row, "field")?)?,
        locked: row_get(&row, "locked")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_provider_raw_response(row: PgRow) -> Result<ProviderRawResponse> {
    Ok(ProviderRawResponse {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        provider: provider_from_parts(row_get(&row, "provider")?, row_get(&row, "provider_key")?),
        provider_key: row_get(&row, "provider_key")?,
        body_json: row_get(&row, "body_json")?,
        fetched_at: row_get(&row, "fetched_at")?,
    })
}

fn row_to_metadata_provider_attempt(row: PgRow) -> Result<MetadataProviderAttemptRecord> {
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

fn row_to_provider_subject(row: PgRow) -> Result<ProviderSubject> {
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

fn row_to_provider_mapping(row: PgRow) -> Result<ProviderMapping> {
    Ok(ProviderMapping {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        subject_id: parse_id(row_get::<String>(&row, "subject_id")?)?,
        status: ProviderMappingStatus::parse(&row_get::<String>(&row, "status")?)?,
        confidence_milli: optional_i64_to_u16(row_get(&row, "confidence_milli")?)?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_person(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Person> {
    Ok(Person {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        sort_name: row_get(&row, "sort_name")?,
        overview: row_get(&row, "overview")?,
        external_ids,
    })
}

fn row_to_item_credit(row: PgRow) -> Result<ItemCredit> {
    let character = row_get::<String>(&row, "character")?;
    Ok(ItemCredit {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        person_id: parse_id(row_get::<String>(&row, "person_id")?)?,
        role: credit_role_from_parts(row_get(&row, "role")?, row_get(&row, "role_key")?),
        character: (!character.is_empty()).then_some(character),
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_genre(row: PgRow) -> Result<Genre> {
    Ok(Genre {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_genre(row: PgRow) -> Result<ItemGenre> {
    Ok(ItemGenre {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        genre_id: parse_id(row_get::<String>(&row, "genre_id")?)?,
    })
}

fn row_to_tag(row: PgRow) -> Result<Tag> {
    Ok(Tag {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
    })
}

fn row_to_item_tag(row: PgRow) -> Result<ItemTag> {
    Ok(ItemTag {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        tag_id: parse_id(row_get::<String>(&row, "tag_id")?)?,
    })
}

fn row_to_collection(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Collection> {
    Ok(Collection {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        overview: row_get(&row, "overview")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_collection_item(row: PgRow) -> Result<CollectionItem> {
    Ok(CollectionItem {
        collection_id: parse_id(row_get::<String>(&row, "collection_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        sort_order: optional_i64_to_u32(row_get(&row, "sort_order")?)?,
    })
}

fn row_to_studio(row: PgRow, external_ids: Vec<ExternalId>) -> Result<Studio> {
    Ok(Studio {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        name: row_get(&row, "name")?,
        source: metadata_source_from_parts(row_get(&row, "source")?, row_get(&row, "source_key")?),
        external_ids,
    })
}

fn row_to_item_studio(row: PgRow) -> Result<ItemStudio> {
    Ok(ItemStudio {
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        studio_id: parse_id(row_get::<String>(&row, "studio_id")?)?,
    })
}

fn row_to_image_asset(row: PgRow) -> Result<ImageAsset> {
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
        selected: row_get(&row, "selected")?,
        content_hash: row_get(&row, "content_hash")?,
        etag: row_get(&row, "etag")?,
    })
}

fn row_to_source_duplicate_relationship(row: PgRow) -> Result<SourceDuplicateRelationship> {
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

fn row_to_managed_import_artifact(row: PgRow) -> Result<ManagedImportArtifactRecord> {
    Ok(ManagedImportArtifactRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        target_library_id: parse_id(row_get::<String>(&row, "target_library_id")?)?,
        source_kind: managed_import_source_kind_from_parts(
            row_get(&row, "source_kind")?,
            row_get(&row, "source_kind_key")?,
        ),
        source_uri: row_get(&row, "source_uri")?,
        staging_manifest_id: parse_optional_id(row_get::<Option<String>>(
            &row,
            "staging_manifest_id",
        )?)?,
        artifact_uri: row_get(&row, "artifact_uri")?,
        original_file_name: row_get(&row, "original_file_name")?,
        intended_locator: row_get(&row, "intended_locator")?,
        size_bytes: optional_i64_to_u64(row_get(&row, "size_bytes")?)?,
        fingerprint: row_get(&row, "fingerprint")?,
        state: ManagedImportArtifactState::parse(&row_get::<String>(&row, "state")?)?,
        diagnostics_json: row_get(&row, "diagnostics_json")?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
    })
}

fn row_to_user_playback_state(row: PgRow) -> Result<UserPlaybackState> {
    Ok(UserPlaybackState {
        principal_id: UserPrincipalId::new(row_get::<String>(&row, "principal_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "item_id")?)?,
        source_id: parse_optional_id(row_get::<Option<String>>(&row, "source_id")?)?,
        resume_position_ms: optional_i64_to_u64(row_get(&row, "resume_position_ms")?)?,
        duration_ms: optional_i64_to_u64(row_get(&row, "duration_ms")?)?,
        watched: row_get(&row, "watched")?,
        watched_at_ms: row_get(&row, "watched_at_ms")?,
        last_played_at_ms: row_get(&row, "last_played_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        version: i64_to_u64(row_get(&row, "version")?)?,
    })
}

fn row_to_transcode_session(row: PgRow) -> Result<TranscodeSessionRecord> {
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

fn row_to_outbox_event(row: PgRow) -> Result<OutboxEventRecord> {
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

fn row_to_webhook_endpoint(row: PgRow) -> Result<WebhookEndpointRecord> {
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

fn row_to_webhook_delivery_attempt(row: PgRow) -> Result<WebhookDeliveryAttemptRecord> {
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

fn row_to_automation_provider(row: PgRow) -> Result<AutomationProviderConfigRecord> {
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

fn row_to_automation_artifact(row: PgRow) -> Result<AutomationArtifactRecord> {
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

fn row_to_addon_registration(row: PgRow) -> Result<AddonRegistrationRecord> {
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

fn row_to_addon_token(row: PgRow) -> Result<AddonTokenRecord> {
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

fn row_to_addon_grant(row: PgRow) -> Result<AddonGrantRecord> {
    Ok(AddonGrantRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        permission: AddonPermission::parse(&row_get::<String>(&row, "permission")?)?,
        library_id: parse_optional_id(row_get::<Option<String>>(&row, "library_id")?)?,
        created_at: row_get(&row, "created_at")?,
    })
}

fn row_to_addon_side_effect(row: PgRow) -> Result<AddonSideEffectRecord> {
    let target = AddonSideEffectTarget {
        kind: AddonSideEffectTargetKind::parse(&row_get::<String>(&row, "target_kind")?)?,
        id: row_get(&row, "target_id")?,
    };

    Ok(AddonSideEffectRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        addon_id: parse_id(row_get::<String>(&row, "addon_id")?)?,
        token_id: parse_id(row_get::<String>(&row, "token_id")?)?,
        permission: AddonPermission::parse(&row_get::<String>(&row, "permission")?)?,
        library_id: parse_id(row_get::<String>(&row, "library_id")?)?,
        target,
        idempotency_key: row_get(&row, "idempotency_key")?,
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

fn row_to_artwork_candidate(row: PgRow) -> Result<ArtworkCandidateRecord> {
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

fn row_to_artwork_task(row: PgRow) -> Result<ArtworkTask> {
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

fn row_to_managed_artwork_ingest(row: PgRow) -> Result<ManagedArtworkIngestRecord> {
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

fn row_to_managed_artwork_artifact(row: PgRow) -> Result<ManagedArtworkArtifactRecord> {
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

fn row_to_selected_artwork(row: PgRow) -> Result<SelectedArtworkRecord> {
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

fn row_to_managed_artwork_artifact_lifecycle(
    row: PgRow,
) -> Result<ManagedArtworkArtifactLifecycleRecord> {
    let selected_artwork_count = i64_to_u32(row_get(&row, "selected_artwork_count")?)?;

    Ok(ManagedArtworkArtifactLifecycleRecord {
        artifact: row_to_managed_artwork_artifact(row)?,
        selected_artwork_count,
    })
}

fn row_to_managed_artwork_gallery_candidate(
    row: PgRow,
) -> Result<ManagedArtworkGalleryCandidateRecord> {
    let id = parse_id(row_get::<String>(&row, "id")?)?;
    let addon_id = parse_id(row_get::<String>(&row, "addon_id")?)?;
    let side_effect_id = parse_id(row_get::<String>(&row, "side_effect_id")?)?;
    let library_id = parse_id(row_get::<String>(&row, "library_id")?)?;
    let item_id = parse_id(row_get::<String>(&row, "item_id")?)?;
    let kind = image_kind_from_parts(row_get(&row, "kind")?, row_get(&row, "kind_key")?);
    let source_kind = ArtworkCandidateSourceKind::parse(&row_get::<String>(&row, "source_kind")?)?;
    let width = optional_i64_to_u32(row_get(&row, "width")?)?;
    let height = optional_i64_to_u32(row_get(&row, "height")?)?;
    let language = row_get(&row, "language")?;
    let status = ArtworkCandidateStatus::parse(&row_get::<String>(&row, "status")?)?;
    let created_at = row_get(&row, "created_at")?;
    let updated_at = row_get(&row, "updated_at")?;
    let ingest_id: Option<ManagedArtworkIngestId> =
        parse_optional_id(row_get::<Option<String>>(&row, "ingest_id")?)?;
    let ingest = if let Some(ingest_id) = ingest_id {
        Some(ManagedArtworkIngestRecord {
            id: ingest_id,
            candidate_id: id,
            job_id: parse_id(row_get::<String>(&row, "ingest_job_id")?)?,
            library_id,
            item_id,
            kind: kind.clone(),
            status: ManagedArtworkIngestStatus::parse(&row_get::<String>(&row, "ingest_status")?)?,
            artifact_id: parse_optional_id(row_get::<Option<String>>(&row, "ingest_artifact_id")?)?,
            failure_code: row_get(&row, "ingest_failure_code")?,
            created_at: row_get(&row, "ingest_created_at")?,
            updated_at: row_get(&row, "ingest_updated_at")?,
        })
    } else {
        None
    };
    let artifact_id = ingest.as_ref().and_then(|ingest| ingest.artifact_id);

    Ok(ManagedArtworkGalleryCandidateRecord {
        id,
        addon_id,
        side_effect_id,
        library_id,
        item_id,
        kind,
        source_kind,
        width,
        height,
        language,
        status,
        ingest,
        artifact_id,
        selected_artwork_count: i64_to_u32(row_get(&row, "selected_artwork_count")?)?,
        created_at,
        updated_at,
    })
}

fn row_to_managed_artwork_gallery_artifact(
    row: PgRow,
) -> Result<ManagedArtworkGalleryArtifactRecord> {
    managed_artwork_gallery_artifact_from_row(&row, "")
}

fn row_to_managed_artwork_gallery_selected(
    row: PgRow,
) -> Result<ManagedArtworkGallerySelectedRecord> {
    let selected_artwork = SelectedArtworkRecord {
        id: parse_id(row_get::<String>(&row, "selected_id")?)?,
        library_id: parse_id(row_get::<String>(&row, "selected_library_id")?)?,
        item_id: parse_id(row_get::<String>(&row, "selected_item_id")?)?,
        kind: image_kind_from_parts(
            row_get(&row, "selected_kind")?,
            row_get(&row, "selected_kind_key")?,
        ),
        artifact_id: parse_id(row_get::<String>(&row, "selected_artifact_id")?)?,
        created_at: row_get(&row, "selected_created_at")?,
        updated_at: row_get(&row, "selected_updated_at")?,
    };
    let artifact = managed_artwork_gallery_artifact_from_row(&row, "artifact_")?;

    Ok(ManagedArtworkGallerySelectedRecord {
        selected_artwork,
        artifact,
    })
}

fn managed_artwork_gallery_artifact_from_row(
    row: &PgRow,
    prefix: &str,
) -> Result<ManagedArtworkGalleryArtifactRecord> {
    Ok(ManagedArtworkGalleryArtifactRecord {
        id: parse_id(row_get::<String>(row, &format!("{prefix}id"))?)?,
        ingest_id: parse_id(row_get::<String>(row, &format!("{prefix}ingest_id"))?)?,
        candidate_id: parse_id(row_get::<String>(row, &format!("{prefix}candidate_id"))?)?,
        library_id: parse_id(row_get::<String>(row, &format!("{prefix}library_id"))?)?,
        item_id: parse_id(row_get::<String>(row, &format!("{prefix}item_id"))?)?,
        kind: image_kind_from_parts(
            row_get(row, &format!("{prefix}kind"))?,
            row_get(row, &format!("{prefix}kind_key"))?,
        ),
        width: optional_i64_to_u32(row_get(row, &format!("{prefix}width"))?)?,
        height: optional_i64_to_u32(row_get(row, &format!("{prefix}height"))?)?,
        byte_len: optional_i64_to_u64(row_get(row, &format!("{prefix}byte_len"))?)?,
        media_type: row_get(row, &format!("{prefix}media_type"))?,
        has_content_hash: row_get(row, &format!("{prefix}has_content_hash"))?,
        selected_artwork_count: i64_to_u32(row_get(
            row,
            &format!("{prefix}selected_artwork_count"),
        )?)?,
        created_at: row_get(row, &format!("{prefix}created_at"))?,
        updated_at: row_get(row, &format!("{prefix}updated_at"))?,
    })
}

fn row_to_vfs_cached_object(row: PgRow) -> Result<VfsCachedObject> {
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

fn row_to_vfs_cache_failure(row: PgRow) -> Result<VfsCacheFailure> {
    Ok(VfsCacheFailure {
        uri: row_get(&row, "uri")?,
        scheme: row_get(&row, "scheme")?,
        operation: VfsCacheOperation::parse(&row_get::<String>(&row, "operation")?)?,
        failed_at_ms: row_get(&row, "failed_at_ms")?,
        failure_count: i64_to_u32(row_get(&row, "failure_count")?)?,
        error: row_get(&row, "error")?,
    })
}

fn row_to_staging_manifest_record(row: PgRow) -> Result<StagingManifestRecord> {
    Ok(StagingManifestRecord {
        id: parse_id(row_get::<String>(&row, "id")?)?,
        source_uri: row_get(&row, "source_uri")?,
        source_scheme: row_get(&row, "source_scheme")?,
        purpose: StagingPurpose::parse(&row_get::<String>(&row, "purpose")?)?,
        local_path: row_get(&row, "local_path")?,
        size_bytes: optional_i64_to_u64(row_get::<Option<i64>>(&row, "size_bytes")?)?,
        etag: row_get(&row, "etag")?,
        fingerprint: row_get(&row, "fingerprint")?,
        state: StagingState::parse(&row_get::<String>(&row, "state")?)?,
        created_at_ms: row_get(&row, "created_at_ms")?,
        updated_at_ms: row_get(&row, "updated_at_ms")?,
        last_accessed_at_ms: row_get(&row, "last_accessed_at_ms")?,
        expires_at_ms: row_get(&row, "expires_at_ms")?,
        active_leases: i64_to_u32(row_get(&row, "active_leases")?)?,
        validation_error: row_get(&row, "validation_error")?,
    })
}

fn row_to_job(row: PgRow) -> Result<Job> {
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

fn row_to_leased_job(row: PgRow) -> Result<LeasedJob> {
    let job_id: JobId = parse_id(row_get::<String>(&row, "id")?)?;
    let job = Job {
        id: job_id,
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
    };
    let lease = JobLeaseRecord {
        job_id,
        worker_id: parse_id(row_get::<String>(&row, "worker_id")?)?,
        run_token: parse_id(row_get::<String>(&row, "run_token")?)?,
        heartbeat_at: row_get(&row, "heartbeat_at")?,
        lease_expires_at: row_get(&row, "lease_expires_at")?,
        cancel_requested_at: row_get(&row, "cancel_requested_at")?,
        cancel_reason: row_get(&row, "cancel_reason")?,
    };

    Ok(LeasedJob { job, lease })
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
            message: format!("unknown media kind stored in PostgreSQL database: {value}"),
        }),
    }
}

fn event_subject_from_parts(kind: String, id: String) -> Result<DomainEventSubject> {
    match kind.as_str() {
        "library" => Ok(DomainEventSubject::Library(parse_id(id)?)),
        "item" => Ok(DomainEventSubject::Item(parse_id(id)?)),
        "source" => Ok(DomainEventSubject::Source(parse_id(id)?)),
        "job" => Ok(DomainEventSubject::Job(parse_id(id)?)),
        "playback_session" => Ok(DomainEventSubject::PlaybackSession(parse_id(id)?)),
        _ => Err(TaruError::Database {
            message: format!("unknown event subject kind stored in PostgreSQL database: {kind}"),
        }),
    }
}

fn parse_transcode_session_kind(value: String) -> Result<TranscodeSessionKind> {
    TranscodeSessionKind::parse(&value).ok_or_else(|| TaruError::Database {
        message: format!("unknown transcode session kind stored in PostgreSQL database: {value}"),
    })
}

fn parse_transcode_session_state(value: String) -> Result<TranscodeSessionState> {
    TranscodeSessionState::parse(&value).ok_or_else(|| TaruError::Database {
        message: format!("unknown transcode session state stored in PostgreSQL database: {value}"),
    })
}

fn parse_transcode_failure_category(
    value: Option<String>,
) -> Result<Option<TranscodeFailureCategory>> {
    value
        .map(|value| {
            TranscodeFailureCategory::parse(&value).ok_or_else(|| TaruError::Database {
                message: format!(
                    "unknown transcode failure category stored in PostgreSQL database: {value}"
                ),
            })
        })
        .transpose()
}

fn validate_catalog_external_id_lookup(table: &str, owner_column: &str) -> Result<()> {
    let valid = matches!(
        (table, owner_column),
        ("person_external_ids", "person_id")
            | ("collection_external_ids", "collection_id")
            | ("studio_external_ids", "studio_id")
    );
    if valid {
        Ok(())
    } else {
        Err(TaruError::Database {
            message: format!(
                "invalid PostgreSQL catalog external-id lookup: {table}.{owner_column}"
            ),
        })
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

fn provider_subject_kind_to_parts(kind: &ProviderSubjectKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

fn provider_subject_kind_from_parts(kind: String, kind_key: String) -> ProviderSubjectKind {
    ProviderSubjectKind::from_parts(&kind, kind_key)
}

fn source_duplicate_evidence_kind_to_parts(kind: &SourceDuplicateEvidenceKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

fn source_duplicate_evidence_kind_from_parts(
    kind: String,
    kind_key: String,
) -> SourceDuplicateEvidenceKind {
    SourceDuplicateEvidenceKind::from_parts(&kind, kind_key)
}

fn managed_import_source_kind_to_parts(kind: &ManagedImportSourceKind) -> (String, String) {
    let (kind, kind_key) = kind.as_parts();
    (kind.to_owned(), kind_key.to_owned())
}

fn managed_import_source_kind_from_parts(
    kind: String,
    kind_key: String,
) -> ManagedImportSourceKind {
    ManagedImportSourceKind::from_parts(&kind, kind_key)
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
            message: format!("unknown metadata field stored in PostgreSQL database: {value}"),
        }),
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

fn local_inference_evidence_source_to_parts(
    source: &LocalInferenceEvidenceSource,
) -> (String, String) {
    let (source, source_key) = source.as_parts();
    (source.to_owned(), source_key.to_owned())
}

fn local_inference_evidence_source_from_parts(
    source: String,
    source_key: String,
) -> LocalInferenceEvidenceSource {
    LocalInferenceEvidenceSource::from_parts(&source, source_key)
}

fn u32_to_i64(value: u32) -> i64 {
    i64::from(value)
}

fn u64_to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|err| TaruError::Database {
        message: format!("value does not fit into PostgreSQL bigint: {err}"),
    })
}

fn optional_u64_to_i64(value: Option<u64>) -> Result<Option<i64>> {
    value.map(u64_to_i64).transpose()
}

fn i64_to_u64(value: i64) -> Result<u64> {
    u64::try_from(value).map_err(|err| TaruError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u64: {err}"),
    })
}

fn optional_i64_to_u64(value: Option<i64>) -> Result<Option<u64>> {
    value.map(i64_to_u64).transpose()
}

fn i64_to_u32(value: i64) -> Result<u32> {
    u32::try_from(value).map_err(|err| TaruError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u32: {err}"),
    })
}

fn i64_to_u16(value: i64) -> Result<u16> {
    u16::try_from(value).map_err(|err| TaruError::Database {
        message: format!("PostgreSQL bigint cannot be converted to u16: {err}"),
    })
}

fn optional_i64_to_u32(value: Option<i64>) -> Result<Option<u32>> {
    value.map(i64_to_u32).transpose()
}

fn optional_i64_to_u16(value: Option<i64>) -> Result<Option<u16>> {
    value
        .map(|value| {
            u16::try_from(value).map_err(|err| TaruError::Database {
                message: format!("PostgreSQL bigint cannot be converted to u16: {err}"),
            })
        })
        .transpose()
}

fn optional_i64_to_i32(value: Option<i64>) -> Result<Option<i32>> {
    value
        .map(|value| {
            i32::try_from(value).map_err(|err| TaruError::Database {
                message: format!("PostgreSQL bigint cannot be converted to i32: {err}"),
            })
        })
        .transpose()
}

fn validate_lease_duration(lease_duration_ms: u64) -> Result<()> {
    if lease_duration_ms == 0 {
        return Err(TaruError::InvalidInput {
            message: "lease_duration_ms must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn stale_job_lease_error() -> TaruError {
    TaruError::Conflict {
        message: "job lease is no longer owned by this run token".to_owned(),
    }
}

#[cfg(test)]
fn validate_schema_name(schema_name: &str) -> Result<()> {
    let valid = !schema_name.is_empty()
        && schema_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    if !valid {
        return Err(TaruError::InvalidInput {
            message: "PostgreSQL contract schema name must contain only lowercase ASCII letters, digits, and underscores".to_owned(),
        });
    }

    Ok(())
}

fn database_error<E: Display>(err: E) -> TaruError {
    TaruError::Database {
        message: err.to_string(),
    }
}
