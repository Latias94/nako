use std::{fmt::Display, path::PathBuf, str::FromStr};

use sqlx::{Decode, Row, Sqlite, SqlitePool, Type, sqlite::SqliteRow};
use taru_core::{
    AddonId, AddonRegistrationRecord, AddonRepository, AddonStatus, ArtworkTask, ArtworkTaskId,
    ArtworkTaskKind, ArtworkTaskRepository, AutomationArtifactId, AutomationArtifactKind,
    AutomationArtifactRecord, AutomationArtifactStatus, AutomationCapability,
    AutomationProviderConfigRecord, AutomationProviderId, AutomationProviderStatus,
    AutomationRepository, CanonicalMetadata, CatalogItemGraphReplacement, CatalogRepository,
    Collection, CollectionId, CollectionItem, CreditRole, DirectorySnapshot, DomainEventKind,
    DomainEventSubject, EventId, EventOutboxRepository, ExternalId, ExternalProvider, Genre,
    GenreId, ImageAsset, ImageAssetId, ImageKind, ImageOwner, IngestionFailureClass,
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRecord,
    IngestionFailureRepository, IngestionFailureStatus, ItemCredit, ItemGenre, ItemStudio, ItemTag,
    Job, JobId, JobKind, JobRepository, JobStatus, Library, LibraryId, LibraryItemRepository,
    LibraryItemState, LibraryOptions, LibraryRepository, LocalInferenceEvidence,
    LocalInferenceEvidenceId, LocalInferenceEvidenceSource, LocalInferenceRepository, MediaDomain,
    MediaItem, MediaItemId, MediaKind, MediaProbeRepository, MediaProbeResult, MediaRepository,
    MediaSource, MediaSourceId, MediaStreamInfo, MediaStreamKind, MetadataAttemptFilter,
    MetadataField, MetadataFieldLock, MetadataMatchKind, MetadataProviderAttemptRecord,
    MetadataProviderAttemptStatus, MetadataProviderErrorClass, MetadataRepository, MetadataSource,
    NewAddonRegistration, NewAutomationArtifact, NewAutomationProviderConfig, NewIngestionFailure,
    NewJob, NewMetadataProviderAttempt, NewOutboxEvent, NewTranscodeSession, NewVfsCacheFailure,
    NewWebhookDeliveryAttempt, NewWebhookEndpoint, OutboxEventRecord, OutboxEventStatus,
    PageRequest, Person, PersonId, ProviderMapping, ProviderMappingRepository,
    ProviderMappingStatus, ProviderRawResponse, ProviderRawResponseCleanup,
    ProviderRawResponseFilter, ProviderSubject, ProviderSubjectId, ProviderSubjectKind, Result,
    ScanRepository, ScanSnapshot, ScanSnapshotId, ScanStatus, SourceDuplicateEvidenceKind,
    SourceDuplicateRelationship, SourceDuplicateRelationshipId, SourceDuplicateRelationshipStatus,
    SourceDuplicateRepository, SourceState, Studio, StudioId, Tag, TagId, TaruError,
    TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind, TranscodeSessionRecord,
    TranscodeSessionRepository, TranscodeSessionState, VfsCacheFailure, VfsCacheOperation,
    VfsCacheRepository, VfsCachedListing, VfsCachedObject, VfsCachedObjectKind,
    WebhookDeliveryAttemptId, WebhookDeliveryAttemptRecord, WebhookDeliveryStatus,
    WebhookEndpointId, WebhookEndpointRecord, WebhookEndpointStatus, WebhookRepository,
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
mod ingestion;
mod jobs;
mod library;
mod library_item;
mod local_inference;
mod media;
mod metadata;
mod migrations;
mod playback;
mod provider_mapping;
mod runtime;
mod scan;
mod search;
mod source_duplicate;
mod staging;
mod vfs_cache;
mod webhooks;

pub(crate) use codec::*;

#[cfg(test)]
mod tests;
