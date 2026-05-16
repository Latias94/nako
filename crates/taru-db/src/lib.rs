use std::{fmt::Display, path::PathBuf, str::FromStr};

use sqlx::{Decode, Row, Sqlite, SqlitePool, Type, sqlite::SqliteRow};
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
    MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind, MetadataAttemptFilter,
    MetadataField, MetadataFieldLock, MetadataMatchKind, MetadataProviderAttemptRecord,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, MetadataRepository, MetadataSource,
    NewAddonRegistration, NewAutomationArtifact, NewAutomationProviderConfig, NewJob,
    NewMetadataProviderAttempt, NewOutboxEvent, NewTranscodeSession, NewVfsCacheFailure,
    NewWebhookDeliveryAttempt, NewWebhookEndpoint, OutboxEventRecord, OutboxEventStatus,
    PageRequest, Person, PersonId, ProviderRawResponse, ProviderRawResponseCleanup,
    ProviderRawResponseFilter, Result, ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus,
    SourceState, Studio, StudioId, Tag, TagId, TaruError, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord, TranscodeSessionRepository,
    TranscodeSessionState, VfsCacheFailure, VfsCacheOperation, VfsCacheRepository,
    VfsCachedListing, VfsCachedObject, VfsCachedObjectKind, WebhookDeliveryAttemptId,
    WebhookDeliveryAttemptRecord, WebhookDeliveryStatus, WebhookEndpointId, WebhookEndpointRecord,
    WebhookEndpointStatus, WebhookRepository,
};
use taru_search::{SearchDocument, SearchHit, SearchIndex, SearchQuery};

#[derive(Clone, Debug)]
pub struct SqliteStore {
    pool: SqlitePool,
}

mod addons;
mod artwork;
mod automation;
mod catalog;
mod codec;
mod event_outbox;
mod jobs;
mod library;
mod media;
mod metadata;
mod migrations;
mod playback;
mod runtime;
mod scan;
mod search;
mod staging;
mod vfs_cache;
mod webhooks;

pub(crate) use codec::*;

#[cfg(test)]
mod tests;

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
