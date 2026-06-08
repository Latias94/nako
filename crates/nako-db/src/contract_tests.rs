use std::{future::Future, sync::OnceLock};

use nako_core::{
    AcquisitionIntakeCandidateId, AcquisitionIntakeCandidateListFilter,
    AcquisitionIntakeCandidateState, AcquisitionIntakeRepository, AcquisitionIntakeSourceKind,
    AddonEventDeliveryAttemptId, AddonEventDeliveryRepository, AddonEventDeliveryStatus, AddonId,
    AddonManifestFingerprint, AddonMetadataWriteCatalogCommit, AddonMetadataWritePersistenceCommit,
    AddonPermission, AddonRepository, AddonRoutingDeclarationKind, AddonRoutingPlanId,
    AddonRoutingPlanStatus, AddonRoutingPlanTarget, AddonSideEffectApplyOutcome,
    AddonSideEffectApplyStatus, AddonSideEffectId, AddonSideEffectRequestFingerprint,
    AddonSideEffectTarget, AddonSideEffectValidationStatus, AddonStatus, AddonTaskRunListFilter,
    AddonTaskRunRepository, AddonTaskRunRequestFingerprint, AddonTokenId,
    AdminMetadataRawCacheSettings, AdminMetadataRawCacheSettingsRecord, AdminSettingsDocumentKey,
    AdminSettingsDocumentRecord, AdminSettingsEffect, AdminSettingsRepository, AdminSettingsSource,
    ArtworkCandidateId, ArtworkCandidateRecord, ArtworkCandidateRepository,
    ArtworkCandidateSourceKind, ArtworkCandidateStatus, ArtworkTask, ArtworkTaskId,
    ArtworkTaskKind, ArtworkTaskRepository, AuthenticatedPrincipal, AutomationArtifactKind,
    AutomationArtifactStatus, AutomationCapability, AutomationProviderStatus, AutomationRepository,
    CancelLeasedJob, CanonicalMetadata, CatalogGovernanceItemListFilter,
    CatalogGovernanceRepository, CatalogItemGraphReplacement, CatalogItemProjectionCommit,
    CatalogRepository, CatalogSearchProjection, ClaimAddonEventDeliveryAttempt, Collection,
    CollectionId, CollectionItem, CompleteLeasedJob, CreditRole, DatabaseLifecycle,
    DirectorySnapshot, DomainEventKind, DomainEventSubject, EnqueueJobRetry, EventOutboxRepository,
    ExternalId, ExternalProvider, FailLeasedJob,
    GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS,
    GeneratedArtifactMetadataApplyOutcomeCommit, GeneratedArtifactMetadataApplyOutcomeId,
    GeneratedArtifactMetadataApplyOutcomeStatus, GeneratedArtifactMetadataApplyPlan,
    GeneratedArtifactMetadataApplyPlanReason, GeneratedArtifactMetadataApplyPlanStatus,
    GeneratedArtifactMetadataApplyRecoveryAttention, GeneratedArtifactMetadataApplyRecoveryFilter,
    GeneratedArtifactMetadataApplyRecoveryReason, GeneratedArtifactMetadataBulkApplyBatchCommit,
    GeneratedArtifactMetadataBulkApplyBatchId, GeneratedArtifactMetadataBulkApplyBatchItemCommit,
    GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit,
    GeneratedArtifactMetadataBulkApplyBatchItemStatus,
    GeneratedArtifactMetadataBulkApplyBatchStatus, GeneratedArtifactMetadataBulkApplyPlanItem,
    GeneratedArtifactMetadataBulkApplyPlanItemReason,
    GeneratedArtifactMetadataBulkApplyPlanItemStatus,
    GeneratedArtifactMetadataBulkApplyPlanSelection, GeneratedArtifactMetadataBulkApplyPlanSummary,
    GeneratedArtifactPayloadShape, GeneratedArtifactPayloadSummary,
    GeneratedArtifactProviderMappingAction, GeneratedArtifactProviderMappingApplyCommit,
    GeneratedArtifactProviderMappingPlan, GeneratedArtifactProviderMappingReason,
    GeneratedArtifactProviderSubjectPlan, GeneratedArtifactTarget, Genre, GenreId,
    IdentityAccessRepository, ImageAsset, ImageAssetId, ImageKind, ImageOwner,
    IngestionFailureClass, IngestionFailureFilter, IngestionFailurePhase,
    IngestionFailureRepository, IngestionFailureResolution, IngestionFailureStatus, ItemCredit,
    ItemGenre, ItemStudio, ItemTag, Job, JobId, JobKind, JobLeaseClaimFilter, JobLeaseClaimRequest,
    JobLeaseGuard, JobLeaseHeartbeat, JobLeaseRepository, JobListFilter, JobPriority,
    JobRepository, JobRunToken, JobStatus, JobWorkerId, Library, LibraryAccessLevel,
    LibraryAccessPolicy, LibraryAccessPolicyFilter, LibraryAccessPolicyScope, LibraryId,
    LibraryItemBrowseFacet, LibraryItemBrowseQuery, LibraryItemBrowseSortKey,
    LibraryItemBrowseSortOrder, LibraryItemRepository, LibraryItemState,
    LibraryItemWatchStateFilter, LibraryOptions, LibraryPreset, LibraryRepository,
    LibraryScanSourcePersistenceCommit, LocalCredentialRecord, LocalInferenceEvidence,
    LocalInferenceEvidenceId, LocalInferenceEvidenceSource, LocalInferenceRepository,
    METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS, ManagedArtworkAcceptanceRecord,
    ManagedArtworkArtifactId, ManagedArtworkArtifactLifecycleFilter, ManagedArtworkIngestId,
    ManagedArtworkIngestStatus, ManagedArtworkRepository, ManagedImportArtifactId,
    ManagedImportArtifactListFilter, ManagedImportArtifactState, ManagedImportPromotionApplyId,
    ManagedImportPromotionApplyState, ManagedImportPromotionOperationKind, ManagedImportRepository,
    ManagedImportSourceKind, MediaColorInfo, MediaHdrMetadata, MediaItem, MediaItemId, MediaKind,
    MediaProbeRepository, MediaProbeResult, MediaRational, MediaRepository, MediaSource,
    MediaSourceId, MediaStreamDisposition, MediaStreamInfo, MediaStreamKind,
    MediaStreamTechnicalFacts, MetadataApplicationPersistenceCommit, MetadataAttemptFilter,
    MetadataCandidateReviewApplicationAction, MetadataCandidateReviewApplicationPlan,
    MetadataCandidateReviewApplicationReason, MetadataCandidateReviewBatchCommit,
    MetadataCandidateReviewBatchId, MetadataCandidateReviewBatchItemCommit,
    MetadataCandidateReviewBatchItemOutcomeCommit, MetadataCandidateReviewBatchItemStatus,
    MetadataCandidateReviewBatchPlanSelection, MetadataCandidateReviewBatchPlanSummary,
    MetadataCandidateReviewBatchStatus, MetadataCandidateReviewId, MetadataCandidateReviewNode,
    MetadataCandidateReviewPlan, MetadataCandidateReviewRecord, MetadataCandidateReviewRepository,
    MetadataCandidateReviewStatus, MetadataCandidateSource, MetadataCandidateSubject,
    MetadataField, MetadataFieldLock, MetadataMatchKind, MetadataProviderAttemptId,
    MetadataProviderAttemptStatus, MetadataRefreshPersistenceCommit,
    MetadataRefreshProviderMappingCommit, MetadataRepository, MetadataSource, NakoError,
    NewAcquisitionIntakeCandidate, NewAddonEventDeliveryAttempt, NewAddonGrant,
    NewAddonRegistration, NewAddonRoutingPlan, NewAddonSideEffect, NewAddonTaskRun, NewAddonToken,
    NewArtworkCandidate, NewAutomationArtifact, NewAutomationProviderConfig, NewIngestionFailure,
    NewJob, NewManagedArtworkArtifact, NewManagedArtworkIngest, NewManagedImportArtifact,
    NewManagedImportPromotionApply, NewMetadataCandidateReview, NewMetadataProviderAttempt,
    NewNfoSidecarApply, NewOutboxEvent, NewPlaybackSession, NewRendererCommand, NewRendererSession,
    NewStagingManifestRecord, NewTranscodeSession, NewUserPlaylist, NewVfsCacheFailure,
    NewWebhookDeliveryAttempt, NewWebhookEndpoint, NfoImportPersistenceCommit, NfoSidecarApplyId,
    NfoSidecarApplyOperationKind, NfoSidecarApplyRepository, NfoSidecarApplyState,
    OutboxEventListFilter, OutboxEventStatus, PageRequest, Person, PersonId,
    PlaybackPermissionPolicy, PlaybackPolicy, PlaybackPolicyFilter, PlaybackPolicyRepository,
    PlaybackPolicyScope, PlaybackSessionHeartbeat, PlaybackSessionId, PlaybackSessionListFilter,
    PlaybackSessionMode, PlaybackSessionRepository, PlaybackSessionState, PlaybackTargetKind,
    PlaybackTargetNetworkScope, PlaybackTargetTransportAuth, ProviderMapping, ProviderMappingId,
    ProviderMappingRepository, ProviderMappingStatus, ProviderRawResponse, ProviderSubject,
    ProviderSubjectId, ProviderSubjectKind, RecoverExpiredJobLeases, RendererCommandCompletion,
    RendererCommandId, RendererCommandListFilter, RendererCommandState,
    RendererControlCapabilities, RendererControlCommand, RendererSessionHeartbeat,
    RendererSessionId, RendererSessionListFilter, RendererSessionRepository, RendererSessionState,
    RequestJobCancellation, RoleAssignment, ScanRepository, ScanSnapshotId, ScanStatus,
    SelectedArtworkRecord, SourceDuplicateEvidenceKind, SourceDuplicateRelationship,
    SourceDuplicateRelationshipId, SourceDuplicateRelationshipStatus, SourceDuplicateRepository,
    SourceState, StagingAttribution, StagingManifestId, StagingManifestRepository, StagingPurpose,
    StagingState, StorageBackendHealthListFilter, StorageBackendHealthRecord,
    StorageBackendHealthRepository, StorageBackendHealthStatus, StorageCircuitBreakerState,
    StorageFailureClass, Studio, StudioId, Tag, TagId, TranscodeFailureCategory,
    TranscodeSessionId, TranscodeSessionKind, TranscodeSessionListFilter,
    TranscodeSessionRepository, TranscodeSessionState, User, UserId, UserInvitationId,
    UserInvitationRecord, UserInvitationStatus, UserPlaybackStateRepository,
    UserPlaybackStateWrite, UserPlaylistId, UserPlaylistItemRemoval, UserPlaylistItemWrite,
    UserPlaylistReorder, UserPlaylistRepository, UserPrincipalId, UserRole, UserSessionId,
    UserSessionRecord, UserStatus, VfsCacheFailureAuthority, VfsCacheOperation, VfsCacheRepository,
    VfsCachedListing, VfsCachedObject, VfsCachedObjectKind, WebhookDeliveryStatus,
    WebhookEndpointStatus, WebhookRepository,
};
use nako_search::{SearchIndex, SearchQuery};

use crate::{NakoDatabase, postgres::PostgresStore};

const NAKO_TEST_POSTGRES_URL: &str = "NAKO_TEST_POSTGRES_URL";

static POSTGRES_CONTRACT_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContractFamily {
    Lifecycle,
    JobLease,
    JobRetry,
    LibraryMedia,
    ScanCommit,
    MetadataCatalog,
    MetadataCandidateReviewBatch,
    ManagedArtwork,
    AcquisitionIntake,
    ManagedImport,
    NfoSidecarApply,
    PlaybackRuntime,
    RendererRuntime,
    EventAddonAutomation,
    SourceDuplicate,
    RuntimePromotion,
    VfsStaging,
    StorageBackendHealth,
    AdminSettings,
    IdentityAccess,
    CredentialSession,
}

impl ContractFamily {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::JobLease => "job_lease",
            Self::JobRetry => "job_retry",
            Self::LibraryMedia => "library_media",
            Self::ScanCommit => "scan_commit",
            Self::MetadataCatalog => "metadata_catalog",
            Self::MetadataCandidateReviewBatch => "metadata_candidate_review_batch",
            Self::ManagedArtwork => "managed_artwork",
            Self::AcquisitionIntake => "acquisition_intake",
            Self::ManagedImport => "managed_import",
            Self::NfoSidecarApply => "nfo_sidecar_apply",
            Self::PlaybackRuntime => "playback_runtime",
            Self::RendererRuntime => "renderer_runtime",
            Self::EventAddonAutomation => "event_addon_automation",
            Self::SourceDuplicate => "source_duplicate",
            Self::RuntimePromotion => "runtime_promotion",
            Self::VfsStaging => "vfs_staging",
            Self::StorageBackendHealth => "storage_backend_health",
            Self::AdminSettings => "admin_settings",
            Self::IdentityAccess => "identity_access",
            Self::CredentialSession => "credential_session",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContractSetup {
    Fresh,
    Migrated,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ContractCase {
    family: ContractFamily,
    name: &'static str,
    setup: ContractSetup,
}

impl ContractCase {
    const fn fresh(family: ContractFamily, name: &'static str) -> Self {
        Self {
            family,
            name,
            setup: ContractSetup::Fresh,
        }
    }

    const fn migrated(family: ContractFamily, name: &'static str) -> Self {
        Self {
            family,
            name,
            setup: ContractSetup::Migrated,
        }
    }

    fn label(self) -> String {
        format!("{}.{}", self.family.as_str(), self.name)
    }
}

trait LifecycleContractBackend: DatabaseLifecycle + Send + Sync {}

impl<T> LifecycleContractBackend for T where T: DatabaseLifecycle + Send + Sync {}

trait AdminSettingsContractBackend: LifecycleContractBackend + AdminSettingsRepository {}

impl<T> AdminSettingsContractBackend for T where
    T: LifecycleContractBackend + AdminSettingsRepository
{
}

trait IdentityAccessContractBackend:
    LifecycleContractBackend + IdentityAccessRepository + LibraryRepository + PlaybackPolicyRepository
{
}

impl<T> IdentityAccessContractBackend for T where
    T: LifecycleContractBackend
        + IdentityAccessRepository
        + LibraryRepository
        + PlaybackPolicyRepository
{
}

trait CredentialSessionContractBackend: LifecycleContractBackend + IdentityAccessRepository {}

impl<T> CredentialSessionContractBackend for T where
    T: LifecycleContractBackend + IdentityAccessRepository
{
}

trait JobLeaseContractBackend:
    LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

impl<T> JobLeaseContractBackend for T where
    T: LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

trait JobRetryContractBackend:
    LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

impl<T> JobRetryContractBackend for T where
    T: LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

trait LibraryMediaContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + LibraryItemRepository
    + MediaRepository
    + MediaProbeRepository
{
}

impl<T> LibraryMediaContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + LibraryItemRepository
        + MediaRepository
        + MediaProbeRepository
{
}

trait LibraryBrowseContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + LibraryItemRepository
    + MediaRepository
    + UserPlaybackStateRepository
{
}

impl<T> LibraryBrowseContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + LibraryItemRepository
        + MediaRepository
        + UserPlaybackStateRepository
{
}

trait ScanCommitContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + LibraryItemRepository
    + MediaRepository
    + MediaProbeRepository
    + ScanRepository
    + LocalInferenceRepository
    + IngestionFailureRepository
    + SearchIndex
{
}

impl<T> ScanCommitContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + LibraryItemRepository
        + MediaRepository
        + MediaProbeRepository
        + ScanRepository
        + LocalInferenceRepository
        + IngestionFailureRepository
        + SearchIndex
{
}

trait MetadataCatalogContractBackend:
    LifecycleContractBackend
    + AddonRepository
    + CatalogRepository
    + JobRepository
    + LibraryRepository
    + LibraryItemRepository
    + MediaRepository
    + MetadataRepository
    + ProviderMappingRepository
    + SearchIndex
{
}

impl<T> MetadataCatalogContractBackend for T where
    T: LifecycleContractBackend
        + AddonRepository
        + CatalogRepository
        + JobRepository
        + LibraryRepository
        + LibraryItemRepository
        + MediaRepository
        + MetadataRepository
        + ProviderMappingRepository
        + SearchIndex
{
}

trait GeneratedArtifactMetadataApplyOutcomeContractBackend:
    MetadataCatalogContractBackend + AutomationRepository
{
}

impl<T> GeneratedArtifactMetadataApplyOutcomeContractBackend for T where
    T: MetadataCatalogContractBackend + AutomationRepository
{
}

trait MetadataCandidateReviewBatchContractBackend:
    MetadataCatalogContractBackend + MetadataCandidateReviewRepository
{
}

impl<T> MetadataCandidateReviewBatchContractBackend for T where
    T: MetadataCatalogContractBackend + MetadataCandidateReviewRepository
{
}

trait ManagedArtworkContractBackend:
    LifecycleContractBackend
    + AddonRepository
    + ArtworkTaskRepository
    + ArtworkCandidateRepository
    + CatalogRepository
    + JobRepository
    + LibraryRepository
    + LibraryItemRepository
    + ManagedArtworkRepository
    + MediaRepository
{
}

impl<T> ManagedArtworkContractBackend for T where
    T: LifecycleContractBackend
        + AddonRepository
        + ArtworkTaskRepository
        + ArtworkCandidateRepository
        + CatalogRepository
        + JobRepository
        + LibraryRepository
        + LibraryItemRepository
        + ManagedArtworkRepository
        + MediaRepository
{
}

trait PlaybackRuntimeContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + MediaRepository
    + PlaybackSessionRepository
    + TranscodeSessionRepository
    + UserPlaybackStateRepository
    + UserPlaylistRepository
{
}

impl<T> PlaybackRuntimeContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + MediaRepository
        + PlaybackSessionRepository
        + TranscodeSessionRepository
        + UserPlaybackStateRepository
        + UserPlaylistRepository
{
}

trait RendererRuntimeContractBackend:
    PlaybackRuntimeContractBackend + RendererSessionRepository
{
}

impl<T> RendererRuntimeContractBackend for T where
    T: PlaybackRuntimeContractBackend + RendererSessionRepository
{
}

trait EventAddonAutomationContractBackend:
    LifecycleContractBackend
    + AddonEventDeliveryRepository
    + AddonRepository
    + AddonTaskRunRepository
    + AutomationRepository
    + EventOutboxRepository
    + JobRepository
    + LibraryRepository
    + MediaRepository
    + WebhookRepository
{
}

impl<T> EventAddonAutomationContractBackend for T where
    T: LifecycleContractBackend
        + AddonEventDeliveryRepository
        + AddonRepository
        + AddonTaskRunRepository
        + AutomationRepository
        + EventOutboxRepository
        + JobRepository
        + LibraryRepository
        + MediaRepository
        + WebhookRepository
{
}

trait RuntimePromotionContractBackend:
    EventAddonAutomationContractBackend
    + CatalogGovernanceRepository
    + LocalInferenceRepository
    + ProviderMappingRepository
    + SourceDuplicateRepository
    + UserPlaybackStateRepository
    + TranscodeSessionRepository
{
}

impl<T> RuntimePromotionContractBackend for T where
    T: EventAddonAutomationContractBackend
        + CatalogGovernanceRepository
        + LocalInferenceRepository
        + ProviderMappingRepository
        + SourceDuplicateRepository
        + UserPlaybackStateRepository
        + TranscodeSessionRepository
{
}

trait SourceDuplicateContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + MediaRepository
    + ScanRepository
    + SourceDuplicateRepository
{
}

impl<T> SourceDuplicateContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + MediaRepository
        + ScanRepository
        + SourceDuplicateRepository
{
}

trait VfsStagingContractBackend:
    LifecycleContractBackend + VfsCacheRepository + StagingManifestRepository
{
}

impl<T> VfsStagingContractBackend for T where
    T: LifecycleContractBackend + VfsCacheRepository + StagingManifestRepository
{
}

trait StorageBackendHealthContractBackend:
    LifecycleContractBackend + StorageBackendHealthRepository
{
}

impl<T> StorageBackendHealthContractBackend for T where
    T: LifecycleContractBackend + StorageBackendHealthRepository
{
}

trait ManagedImportContractBackend:
    LifecycleContractBackend
    + LibraryRepository
    + MediaRepository
    + StagingManifestRepository
    + ManagedImportRepository
{
}

impl<T> ManagedImportContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + MediaRepository
        + StagingManifestRepository
        + ManagedImportRepository
{
}

trait AcquisitionIntakeContractBackend:
    ManagedImportContractBackend + AcquisitionIntakeRepository
{
}

impl<T> AcquisitionIntakeContractBackend for T where
    T: ManagedImportContractBackend + AcquisitionIntakeRepository
{
}

trait NfoSidecarApplyContractBackend:
    LifecycleContractBackend + LibraryRepository + MediaRepository + NfoSidecarApplyRepository
{
}

impl<T> NfoSidecarApplyContractBackend for T where
    T: LifecycleContractBackend + LibraryRepository + MediaRepository + NfoSidecarApplyRepository
{
}

macro_rules! database_contract_pair {
    (
        sqlite = $sqlite_test:ident,
        postgres = $postgres_test:ident,
        case = $case:expr,
        contract = $contract:path $(,)?
    ) => {
        #[tokio::test]
        async fn $sqlite_test() {
            run_sqlite_contract($case, $contract).await;
        }

        #[tokio::test]
        #[ignore = "requires NAKO_TEST_POSTGRES_URL"]
        async fn $postgres_test() {
            run_postgres_contract($case, $contract).await;
        }
    };
}

async fn sqlite_contract_database(setup: ContractSetup) -> NakoDatabase {
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    apply_contract_setup(&store, setup).await;
    store
}

async fn postgres_contract_database(database_url: &str, setup: ContractSetup) -> PostgresStore {
    let schema_name = format!(
        "nako_contract_{}",
        JobRunToken::new().to_string().replace('-', "_")
    );
    let store = PostgresStore::connect_with_schema(database_url, &schema_name)
        .await
        .unwrap();
    apply_contract_setup(&store, setup).await;
    store
}

async fn apply_contract_setup<S>(store: &S, setup: ContractSetup)
where
    S: DatabaseLifecycle + ?Sized,
{
    if setup == ContractSetup::Migrated {
        store.migrate().await.unwrap();
    }
}

async fn run_sqlite_contract<F, Fut>(case: ContractCase, contract: F)
where
    F: FnOnce(NakoDatabase) -> Fut,
    Fut: Future<Output = ()>,
{
    contract(sqlite_contract_database(case.setup).await).await;
}

async fn run_postgres_contract<F, Fut>(case: ContractCase, contract: F)
where
    F: FnOnce(PostgresStore) -> Fut,
    Fut: Future<Output = ()>,
{
    let database_url = std::env::var(NAKO_TEST_POSTGRES_URL).unwrap_or_else(|_| {
        panic!(
            "PostgreSQL {} contract requires {NAKO_TEST_POSTGRES_URL}; do not run ignored PostgreSQL contract gates without a test database URL",
            case.label()
        )
    });

    let lock = POSTGRES_CONTRACT_LOCK.get_or_init(|| tokio::sync::Mutex::new(()));
    let _guard = lock.lock().await;
    let store = postgres_contract_database(&database_url, case.setup).await;

    contract(store.clone()).await;
    store.drop_schema().await.unwrap();
}

async fn seed_contract_library<S>(store: &S) -> Library
where
    S: LibraryRepository + ?Sized,
{
    seed_named_browse_library(store, "Contract Movies").await
}

async fn seed_named_browse_library<S>(store: &S, name: &str) -> Library
where
    S: LibraryRepository + ?Sized,
{
    let library = Library {
        id: LibraryId::new(),
        name: name.to_owned(),
        roots: vec![format!("local:///{name}")],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&library).await.unwrap();
    library
}

async fn enqueue_contract_job<S>(
    store: &S,
    kind: JobKind,
    resource_class: &str,
    library_id: Option<LibraryId>,
    input_json: Option<&str>,
) -> Job
where
    S: JobRepository + ?Sized,
{
    enqueue_contract_job_with_priority(
        store,
        kind,
        resource_class,
        JobPriority::Normal,
        library_id,
        input_json,
    )
    .await
}

async fn enqueue_contract_job_with_priority<S>(
    store: &S,
    kind: JobKind,
    resource_class: &str,
    priority: JobPriority,
    library_id: Option<LibraryId>,
    input_json: Option<&str>,
) -> Job
where
    S: JobRepository + ?Sized,
{
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind,
            resource_class: resource_class.to_owned(),
            priority,
            library_id,
            source_id: None,
            input_json: input_json.map(str::to_owned),
        })
        .await
        .unwrap()
}

async fn claim_next<S>(
    store: &S,
    worker_id: JobWorkerId,
    filter: JobLeaseClaimFilter,
) -> Option<nako_core::LeasedJob>
where
    S: JobLeaseRepository + ?Sized,
{
    store
        .claim_next_job_lease(JobLeaseClaimRequest {
            worker_id,
            lease_duration_ms: 30_000,
            filter,
        })
        .await
        .unwrap()
}

async fn claim_next_job_lease_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let library = seed_contract_library(&store).await;
    let skipped = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        Some(library.id),
        None,
    )
    .await;
    let target = enqueue_contract_job(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        Some(library.id),
        Some(r#"{"library_id":"movies"}"#),
    )
    .await;
    let decoy = enqueue_contract_job(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        Some(library.id),
        Some(r#"{"library_id":"movies","slot":"decoy"}"#),
    )
    .await;

    let worker_id = JobWorkerId::new();
    let exact_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            job_id: Some(target.id),
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            library_id: Some(library.id),
            source_id: None,
        },
    )
    .await
    .expect("target library scan job should be claimable by id");

    assert_eq!(exact_claim.job.id, target.id);
    assert_eq!(exact_claim.job.input_json, target.input_json);
    assert_eq!(exact_claim.lease.job_id, target.id);
    assert_eq!(exact_claim.lease.worker_id, worker_id);
    assert_eq!(exact_claim.lease.cancel_requested_at, None);
    assert_eq!(exact_claim.lease.cancel_reason, None);

    let claimed = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            job_id: None,
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            library_id: Some(library.id),
            source_id: None,
        },
    )
    .await
    .expect("remaining library scan job should be claimable");

    assert_eq!(claimed.job.id, decoy.id);
    assert_eq!(claimed.job.status, JobStatus::Running);
    assert_eq!(claimed.job.input_json, decoy.input_json);
    assert!(claimed.job.started_at.is_some());
    assert_eq!(claimed.job.completed_at, None);
    assert_eq!(claimed.job.error, None);
    assert_eq!(claimed.lease.job_id, decoy.id);
    assert_eq!(claimed.lease.worker_id, worker_id);
    assert_eq!(claimed.lease.cancel_requested_at, None);
    assert_eq!(claimed.lease.cancel_reason, None);
    assert!(!claimed.lease.heartbeat_at.is_empty());
    assert!(!claimed.lease.lease_expires_at.is_empty());

    let loaded = store.get_job(target.id).await.unwrap().unwrap();
    let decoy_loaded = store.get_job(decoy.id).await.unwrap().unwrap();
    assert_eq!(loaded.status, JobStatus::Running);
    assert_eq!(decoy_loaded.status, JobStatus::Running);
    assert_eq!(
        store.get_job(skipped.id).await.unwrap().unwrap().status,
        JobStatus::Queued
    );
    assert!(
        claim_next(
            &store,
            worker_id,
            JobLeaseClaimFilter {
                job_id: None,
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                library_id: Some(library.id),
                source_id: None,
            },
        )
        .await
        .is_none()
    );
}

async fn job_priority_policy_contract<S>(store: S)
where
    S: JobRetryContractBackend,
{
    let low = enqueue_contract_job_with_priority(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        JobPriority::Low,
        None,
        Some(r#"{"slot":"low"}"#),
    )
    .await;
    let high = enqueue_contract_job_with_priority(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        JobPriority::High,
        None,
        Some(r#"{"slot":"high"}"#),
    )
    .await;
    let worker_id = JobWorkerId::new();
    let priority_candidates = store
        .list_claimable_jobs_for_lease(
            JobLeaseClaimFilter {
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                ..JobLeaseClaimFilter::default()
            },
            PageRequest::new(10, 0),
        )
        .await
        .unwrap();
    assert_eq!(priority_candidates[0].id, high.id);
    assert_eq!(priority_candidates[1].id, low.id);

    let high_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("high priority job should be claimable first");
    assert_eq!(high_claim.job.id, high.id);
    assert_eq!(high_claim.job.priority, JobPriority::High);

    let low_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("low priority job should remain claimable");
    assert_eq!(low_claim.job.id, low.id);
    assert_eq!(low_claim.job.priority, JobPriority::Low);

    let aged_low = enqueue_contract_job_with_priority(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        JobPriority::Low,
        None,
        Some(r#"{"slot":"aged-low"}"#),
    )
    .await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    let fresh_high = enqueue_contract_job_with_priority(
        &store,
        JobKind::LibraryScan,
        "disk.scan",
        JobPriority::High,
        None,
        Some(r#"{"slot":"fresh-high"}"#),
    )
    .await;
    let fairness_candidates = store
        .list_claimable_jobs_for_lease(
            JobLeaseClaimFilter {
                kind: Some(JobKind::LibraryScan),
                resource_class: Some("disk.scan".to_owned()),
                ..JobLeaseClaimFilter::default()
            },
            PageRequest::new(10, 0),
        )
        .await
        .unwrap();
    assert_eq!(fairness_candidates[0].id, aged_low.id);
    assert_eq!(fairness_candidates[1].id, fresh_high.id);

    let fairness_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            resource_class: Some("disk.scan".to_owned()),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("aged low priority job should be claimable before fresh high priority work");
    assert_eq!(fairness_claim.job.id, aged_low.id);
    assert_eq!(fairness_claim.job.priority, JobPriority::Low);
    assert_eq!(
        store.get_job(fresh_high.id).await.unwrap().unwrap().status,
        JobStatus::Queued
    );

    let failed_high = enqueue_contract_job_with_priority(
        &store,
        JobKind::MetadataRefresh,
        "metadata.tmdb",
        JobPriority::High,
        None,
        Some(r#"{"provider":"tmdb"}"#),
    )
    .await;
    let failed_high = store
        .fail_job(failed_high.id, "transient metadata failure".to_owned())
        .await
        .unwrap();
    let retry = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: failed_high.id,
            retry_job_id: JobId::new(),
            max_attempts: 2,
            next_attempt_at: None,
        })
        .await
        .unwrap();
    assert_eq!(retry.priority, JobPriority::High);

    let retry_claim = claim_next(
        &store,
        worker_id,
        JobLeaseClaimFilter {
            job_id: Some(retry.id),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("retry should be claimable by id");
    assert_eq!(retry_claim.job.priority, JobPriority::High);
    let recovered = store
        .recover_expired_job_leases(RecoverExpiredJobLeases {
            filter: JobLeaseClaimFilter {
                job_id: Some(retry.id),
                ..JobLeaseClaimFilter::default()
            },
            expired_before: "9999-01-01T00:00:00.000Z".to_owned(),
            error: "lease expired during startup recovery".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(recovered, 1);
    let recovered_retry = store.get_job(retry.id).await.unwrap().unwrap();
    assert_eq!(recovered_retry.status, JobStatus::Failed);
    assert_eq!(recovered_retry.priority, JobPriority::High);
}

async fn job_lease_run_token_fence_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let job = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let claimed = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("queued library scan job should be claimable");
    assert_eq!(claimed.job.id, job.id);

    let stale_guard = JobLeaseGuard {
        job_id: claimed.job.id,
        run_token: JobRunToken::new(),
    };
    let stale_heartbeat = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: stale_guard,
            lease_duration_ms: 10_000,
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_heartbeat, NakoError::Conflict { .. }));

    let heartbeat = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: claimed.lease.guard(),
            lease_duration_ms: 20_000,
        })
        .await
        .unwrap();
    assert_eq!(heartbeat.job.id, claimed.job.id);
    assert_eq!(heartbeat.job.status, JobStatus::Running);
    assert_eq!(heartbeat.lease.run_token, claimed.lease.run_token);
    assert_ne!(
        heartbeat.lease.lease_expires_at,
        claimed.lease.lease_expires_at
    );

    let stale_success = store
        .succeed_leased_job(CompleteLeasedJob {
            guard: stale_guard,
            summary_json: Some(r#"{"ignored":true}"#.to_owned()),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_success, NakoError::Conflict { .. }));
    assert_eq!(
        store.get_job(job.id).await.unwrap().unwrap().status,
        JobStatus::Running
    );

    let succeeded = store
        .succeed_leased_job(CompleteLeasedJob {
            guard: claimed.lease.guard(),
            summary_json: Some(r#"{"done":true}"#.to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(succeeded.status, JobStatus::Succeeded);
    assert_eq!(succeeded.summary_json, Some(r#"{"done":true}"#.to_owned()));
    assert_eq!(succeeded.error, None);
    assert!(succeeded.completed_at.is_some());

    let stale_failure = store
        .fail_leased_job(FailLeasedJob {
            guard: claimed.lease.guard(),
            error: "too late".to_owned(),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_failure, NakoError::Conflict { .. }));
}

async fn job_cancellation_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let queued = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        None,
        None,
    )
    .await;
    let queued_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: queued.id,
            reason: Some("operator request".to_owned()),
        })
        .await
        .unwrap();
    assert!(queued_cancel.requested);
    assert!(queued_cancel.terminal);
    assert_eq!(queued_cancel.job.status, JobStatus::Cancelled);
    assert_eq!(queued_cancel.job.error, None);
    assert!(queued_cancel.job.completed_at.is_some());
    assert!(queued_cancel.cancel_requested_at.is_some());

    let running = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let claimed = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("queued running job should be claimable");
    assert_eq!(claimed.job.id, running.id);

    let running_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: running.id,
            reason: Some("operator stop".to_owned()),
        })
        .await
        .unwrap();
    assert!(running_cancel.requested);
    assert!(!running_cancel.terminal);
    assert_eq!(running_cancel.job.status, JobStatus::Running);
    assert!(running_cancel.cancel_requested_at.is_some());

    let refreshed = store
        .heartbeat_job_lease(JobLeaseHeartbeat {
            guard: claimed.lease.guard(),
            lease_duration_ms: 10_000,
        })
        .await
        .unwrap();
    assert!(refreshed.lease.cancel_requested_at.is_some());
    assert_eq!(
        refreshed.lease.cancel_reason.as_deref(),
        Some("operator stop")
    );

    let stale_cancel = store
        .cancel_leased_job(CancelLeasedJob {
            guard: JobLeaseGuard {
                job_id: running.id,
                run_token: JobRunToken::new(),
            },
            summary_json: Some(r#"{"ignored":true}"#.to_owned()),
        })
        .await
        .unwrap_err();
    assert!(matches!(stale_cancel, NakoError::Conflict { .. }));

    let cancelled = store
        .cancel_leased_job(CancelLeasedJob {
            guard: claimed.lease.guard(),
            summary_json: Some(r#"{"cancelled":true}"#.to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(cancelled.status, JobStatus::Cancelled);
    assert_eq!(
        cancelled.summary_json,
        Some(r#"{"cancelled":true}"#.to_owned())
    );
    assert_eq!(cancelled.error, None);
    assert!(cancelled.completed_at.is_some());

    let terminal_cancel = store
        .request_job_cancellation(RequestJobCancellation {
            job_id: cancelled.id,
            reason: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(terminal_cancel, NakoError::Conflict { .. }));
}

async fn recover_expired_job_leases_contract<S>(store: S)
where
    S: JobLeaseContractBackend,
{
    let queued = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        None,
        None,
    )
    .await;
    let running = enqueue_contract_job(&store, JobKind::LibraryScan, "disk.scan", None, None).await;
    let active =
        enqueue_contract_job(&store, JobKind::LibraryProbe, "media.probe", None, None).await;

    let expired_claim = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryScan),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("library scan job should be claimable");
    let active_claim = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::LibraryProbe),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("library probe job should be claimable");
    assert_eq!(expired_claim.job.id, running.id);
    assert_eq!(active_claim.job.id, active.id);

    let exact_recovery = store
        .recover_expired_job_leases(RecoverExpiredJobLeases {
            filter: JobLeaseClaimFilter {
                job_id: Some(running.id),
                ..JobLeaseClaimFilter::default()
            },
            expired_before: "9999-01-01T00:00:00.000Z".to_owned(),
            error: "lease expired during startup recovery".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(exact_recovery, 1);

    let running = store.get_job(running.id).await.unwrap().unwrap();
    let active = store.get_job(active.id).await.unwrap().unwrap();
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(active.status, JobStatus::Running);

    let recovered = store
        .recover_expired_job_leases(RecoverExpiredJobLeases {
            filter: JobLeaseClaimFilter::default(),
            expired_before: "9999-01-01T00:00:00.000Z".to_owned(),
            error: "lease expired during startup recovery".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(recovered, 1);

    let queued = store.get_job(queued.id).await.unwrap().unwrap();
    let running = store.get_job(running.id).await.unwrap().unwrap();
    let active = store.get_job(active.id).await.unwrap().unwrap();

    assert_eq!(queued.status, JobStatus::Queued);
    assert_eq!(running.status, JobStatus::Failed);
    assert_eq!(
        running.error.as_deref(),
        Some("lease expired during startup recovery")
    );
    assert_eq!(active.status, JobStatus::Failed);
    assert!(running.completed_at.is_some());
    assert!(active.completed_at.is_some());
}

async fn job_retry_backoff_contract<S>(store: S)
where
    S: JobRetryContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.tmdb",
        Some(library.id),
        Some(r#"{"provider":"tmdb","secret":"must-not-leak"}"#),
    )
    .await;
    assert_eq!(source.attempt, 1);
    assert_eq!(source.max_attempts, 1);
    assert_eq!(source.retry_of_job_id, None);
    assert_eq!(source.next_attempt_at, None);

    let failed = store
        .fail_job(source.id, "provider token must-not-leak failed".to_owned())
        .await
        .unwrap();
    assert_eq!(failed.status, JobStatus::Failed);

    let retry = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: failed.id,
            retry_job_id: JobId::new(),
            max_attempts: 3,
            next_attempt_at: Some("9999-01-01T00:00:00.000Z".to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(retry.kind, JobKind::MetadataRefresh);
    assert_eq!(retry.status, JobStatus::Queued);
    assert_eq!(retry.resource_class, "metadata.tmdb");
    assert_eq!(retry.library_id, Some(library.id));
    assert_eq!(retry.input_json, failed.input_json);
    assert_eq!(retry.attempt, 2);
    assert_eq!(retry.max_attempts, 3);
    assert_eq!(retry.retry_of_job_id, Some(failed.id));
    assert_eq!(
        retry.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00.000Z")
    );

    let not_due = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::MetadataRefresh),
            resource_class: Some("metadata.tmdb".to_owned()),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await;
    assert!(not_due.is_none(), "future retry must not be claimable");

    let summaries = store.summarize_job_queue_pressure().await.unwrap();
    let retry_pressure = summaries
        .iter()
        .find(|summary| {
            summary.kind == JobKind::MetadataRefresh
                && summary.status == JobStatus::Queued
                && summary.resource_class == "metadata.tmdb"
        })
        .expect("queued metadata retry pressure should be summarized");
    assert_eq!(retry_pressure.count, 1);
    assert_eq!(retry_pressure.claimable_count, 0);
    assert_eq!(retry_pressure.delayed_retry_count, 1);
    assert_eq!(
        retry_pressure.next_attempt_at.as_deref(),
        Some("9999-01-01T00:00:00.000Z")
    );
    let diagnostics = serde_json::to_string(&summaries).unwrap();
    assert!(!diagnostics.contains("must-not-leak"));
    assert!(!diagnostics.contains("provider token"));
    assert!(!diagnostics.contains("input_json"));

    let retry_not_failed = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: retry.id,
            retry_job_id: JobId::new(),
            max_attempts: 3,
            next_attempt_at: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(retry_not_failed, NakoError::Conflict { .. }));

    let failed_retry = store
        .fail_job(retry.id, "retry still failed".to_owned())
        .await
        .unwrap();
    let immediate_retry = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: failed_retry.id,
            retry_job_id: JobId::new(),
            max_attempts: 3,
            next_attempt_at: Some("0001-01-01T00:00:00.000Z".to_owned()),
        })
        .await
        .unwrap();
    assert_eq!(immediate_retry.attempt, 3);
    assert_eq!(immediate_retry.max_attempts, 3);

    let claimed = claim_next(
        &store,
        JobWorkerId::new(),
        JobLeaseClaimFilter {
            kind: Some(JobKind::MetadataRefresh),
            resource_class: Some("metadata.tmdb".to_owned()),
            ..JobLeaseClaimFilter::default()
        },
    )
    .await
    .expect("past retry should be claimable");
    assert_eq!(claimed.job.id, immediate_retry.id);
    assert_eq!(claimed.job.status, JobStatus::Running);
    assert_eq!(claimed.job.next_attempt_at, None);

    let exhausted = store
        .fail_job(claimed.job.id, "last attempt failed".to_owned())
        .await
        .unwrap();
    let retry_exhausted = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: exhausted.id,
            retry_job_id: JobId::new(),
            max_attempts: 3,
            next_attempt_at: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(retry_exhausted, NakoError::Conflict { .. }));

    let cancelled = enqueue_contract_job(
        &store,
        JobKind::WebhookDelivery,
        "network.webhook",
        None,
        Some(r#"{"webhook_secret":"must-not-leak"}"#),
    )
    .await;
    store
        .request_job_cancellation(RequestJobCancellation {
            job_id: cancelled.id,
            reason: Some("operator request".to_owned()),
        })
        .await
        .unwrap();
    let retry_cancelled = store
        .enqueue_job_retry(EnqueueJobRetry {
            source_job_id: cancelled.id,
            retry_job_id: JobId::new(),
            max_attempts: 2,
            next_attempt_at: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(retry_cancelled, NakoError::Conflict { .. }));
}

async fn library_media_identity_contract<S>(store: S)
where
    S: LibraryMediaContractBackend,
{
    let movies = Library {
        id: LibraryId::new(),
        name: "Movies".to_owned(),
        roots: vec!["local:///Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    let anime = Library {
        id: LibraryId::new(),
        name: "Anime".to_owned(),
        roots: vec!["webdav:///Anime".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Anime),
    };
    store.upsert_library(&movies).await.unwrap();
    store.upsert_library(&anime).await.unwrap();

    assert_eq!(
        store.get_library(movies.id).await.unwrap(),
        Some(movies.clone())
    );
    assert_eq!(
        store
            .list_libraries(PageRequest {
                limit: 10,
                offset: 0
            })
            .await
            .unwrap(),
        vec![anime.clone(), movies.clone()]
    );

    let parent = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Series,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Contract Series".to_owned(),
            original_title: Some("Original Contract Series".to_owned()),
            sort_title: Some("Contract Series".to_owned()),
            overview: Some("A backend-neutral contract parent.".to_owned()),
            release_date: Some("2026-05-20".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "series-1".to_owned(),
            }],
            ..CanonicalMetadata::default()
        },
    };
    let episode = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Episode,
        parent_id: Some(parent.id),
        metadata: CanonicalMetadata {
            title: "Episode 1".to_owned(),
            overview: Some("A backend-neutral contract child.".to_owned()),
            external_ids: vec![
                ExternalId {
                    provider: ExternalProvider::Bangumi,
                    value: "bgm-1".to_owned(),
                },
                ExternalId {
                    provider: ExternalProvider::Other("contract".to_owned()),
                    value: "episode-1".to_owned(),
                },
            ],
            ..CanonicalMetadata::default()
        },
    };
    store.upsert_media_item(&parent).await.unwrap();
    store.upsert_media_item(&episode).await.unwrap();

    assert_eq!(
        store.get_media_item(parent.id).await.unwrap(),
        Some(parent.clone())
    );
    assert_eq!(
        store.get_media_item(episode.id).await.unwrap(),
        Some(episode.clone())
    );
    assert_eq!(
        store
            .list_media_items(PageRequest {
                limit: 10,
                offset: 0
            })
            .await
            .unwrap(),
        vec![parent.clone(), episode.clone()]
    );

    let movie_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: movies.id,
        item_id: episode.id,
        locator: "local:///Movies/Contract Series/Episode 1.mkv".to_owned(),
        file_name: "Episode 1.mkv".to_owned(),
        size_bytes: Some(42),
        fingerprint: Some("source:v1:content_hash:sha256:contract-content-hash".to_owned()),
    };
    let anime_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: anime.id,
        item_id: episode.id,
        locator: "local:///Movies/Contract Series/Episode 1.mkv".to_owned(),
        file_name: "Episode 1-alt.mkv".to_owned(),
        size_bytes: Some(43),
        fingerprint: Some("fingerprint-remote".to_owned()),
    };
    store.upsert_media_source(&movie_source).await.unwrap();
    store.upsert_media_source(&anime_source).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: movies.id,
            item_id: episode.id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: anime.id,
            item_id: episode.id,
            provisional: true,
        })
        .await
        .unwrap();

    assert_eq!(
        store.get_media_source(movie_source.id).await.unwrap(),
        Some(movie_source.clone())
    );
    assert_eq!(
        store
            .get_media_source_by_locator(movies.id, &movie_source.locator)
            .await
            .unwrap(),
        Some(movie_source.clone())
    );
    assert_eq!(
        store
            .get_media_source_by_locator(anime.id, &anime_source.locator)
            .await
            .unwrap(),
        Some(anime_source.clone())
    );
    assert_eq!(
        store
            .list_item_sources(
                episode.id,
                PageRequest {
                    limit: 10,
                    offset: 0
                }
            )
            .await
            .unwrap(),
        vec![movie_source.clone(), anime_source.clone()]
    );
    assert_eq!(
        store
            .list_media_sources(
                movies.id,
                PageRequest {
                    limit: 10,
                    offset: 0
                }
            )
            .await
            .unwrap(),
        vec![movie_source.clone()]
    );
    assert_eq!(
        store.summarize_media_source_fingerprints().await.unwrap(),
        nako_core::MediaSourceFingerprintSummary {
            total_sources: 2,
            fingerprinted_sources: 2,
            content_hash_sources: 1,
        }
    );
    assert_eq!(
        store
            .get_library_item_state(movies.id, episode.id)
            .await
            .unwrap(),
        Some(LibraryItemState {
            library_id: movies.id,
            item_id: episode.id,
            provisional: false,
        })
    );
    assert_eq!(
        store
            .list_library_item_states_for_item(episode.id)
            .await
            .unwrap(),
        vec![
            LibraryItemState {
                library_id: movies.id,
                item_id: episode.id,
                provisional: false,
            },
            LibraryItemState {
                library_id: anime.id,
                item_id: episode.id,
                provisional: true,
            },
        ]
    );
    assert_eq!(
        store
            .find_library_item_by_kind_parent_title(
                movies.id,
                MediaKind::Episode,
                Some(parent.id),
                "Episode 1",
            )
            .await
            .unwrap(),
        Some(episode.clone())
    );
    assert_eq!(
        store
            .list_media_items_for_library(
                anime.id,
                PageRequest {
                    limit: 10,
                    offset: 0
                }
            )
            .await
            .unwrap(),
        vec![episode.clone()]
    );
    let anime_added_at = store.list_library_item_added_at(anime.id).await.unwrap();
    assert_eq!(anime_added_at.len(), 1);
    assert_eq!(anime_added_at[0].item_id, episode.id);
    assert!(!anime_added_at[0].added_at.is_empty());
}

async fn library_source_inventory_projection_contract<S>(store: S)
where
    S: LibraryMediaContractBackend,
{
    let library = seed_named_browse_library(&store, "Inventory Movies").await;
    let other_library = seed_named_browse_library(&store, "Other Inventory Movies").await;

    let hydrated_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Hydrated Source Item".to_owned(),
            external_ids: vec![
                ExternalId {
                    provider: ExternalProvider::Bangumi,
                    value: "inventory-hydrated-bangumi".to_owned(),
                },
                ExternalId {
                    provider: ExternalProvider::Tmdb,
                    value: "inventory-hydrated".to_owned(),
                },
            ],
            ..CanonicalMetadata::default()
        },
    };
    let missing_probe_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Missing Probe Item".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let tail_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Tail Source Item".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let other_item = MediaItem {
        id: MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Other Library Source Item".to_owned(),
            ..CanonicalMetadata::default()
        },
    };

    for item in [&hydrated_item, &missing_probe_item, &tail_item, &other_item] {
        store.upsert_media_item(item).await.unwrap();
    }

    let hydrated_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: hydrated_item.id,
        locator: "local:///Inventory Movies/01-hydrated.mkv".to_owned(),
        file_name: "01-hydrated.mkv".to_owned(),
        size_bytes: Some(101),
        fingerprint: Some("source:v1:content_hash:sha256:inventory-hydrated".to_owned()),
    };
    let missing_probe_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: missing_probe_item.id,
        locator: "local:///Inventory Movies/02-missing-probe.mkv".to_owned(),
        file_name: "02-missing-probe.mkv".to_owned(),
        size_bytes: Some(202),
        fingerprint: None,
    };
    let tail_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: library.id,
        item_id: tail_item.id,
        locator: "local:///Inventory Movies/03-tail.mkv".to_owned(),
        file_name: "03-tail.mkv".to_owned(),
        size_bytes: Some(303),
        fingerprint: None,
    };
    let other_source = MediaSource {
        id: MediaSourceId::new(),
        library_id: other_library.id,
        item_id: other_item.id,
        locator: "local:///Other Inventory Movies/01-other.mkv".to_owned(),
        file_name: "01-other.mkv".to_owned(),
        size_bytes: Some(404),
        fingerprint: None,
    };

    for source in [
        &hydrated_source,
        &missing_probe_source,
        &tail_source,
        &other_source,
    ] {
        store.upsert_media_source(source).await.unwrap();
    }

    let hydrated_probe = MediaProbeResult {
        duration_ms: Some(120_000),
        container: Some("matroska".to_owned()),
        bit_rate: Some(8_000),
        streams: vec![
            MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some("h264".to_owned()),
                language: None,
                duration_ms: Some(120_000),
                bit_rate: Some(7_500),
                width: Some(1920),
                height: Some(1080),
                channels: None,
                sample_rate: None,
                technical: MediaStreamTechnicalFacts::default(),
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("jpn".to_owned()),
                duration_ms: Some(120_000),
                bit_rate: Some(384_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: MediaStreamTechnicalFacts::default(),
            },
        ],
    };
    store
        .upsert_media_probe(hydrated_source.id, &hydrated_probe)
        .await
        .unwrap();

    let first_page = store
        .list_library_source_inventory(
            library.id,
            PageRequest {
                limit: 2,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(first_page.len(), 2);
    assert_eq!(first_page[0].source, hydrated_source);
    assert_eq!(first_page[0].item, Some(hydrated_item));
    assert_eq!(first_page[0].probe, Some(hydrated_probe));
    assert_eq!(first_page[1].source, missing_probe_source);
    assert_eq!(first_page[1].item, Some(missing_probe_item));
    assert_eq!(first_page[1].probe, None);

    let second_page = store
        .list_library_source_inventory(
            library.id,
            PageRequest {
                limit: 2,
                offset: 2,
            },
        )
        .await
        .unwrap();
    assert_eq!(second_page.len(), 1);
    assert_eq!(second_page[0].source, tail_source);
    assert_eq!(second_page[0].item, Some(tail_item));
    assert_eq!(second_page[0].probe, None);

    let other_page = store
        .list_library_source_inventory(
            other_library.id,
            PageRequest {
                limit: 10,
                offset: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(other_page.len(), 1);
    assert_eq!(other_page[0].source, other_source);
    assert_eq!(other_page[0].item, Some(other_item));
    assert_eq!(other_page[0].probe, None);
}

async fn library_item_browse_query_contract<S>(store: S)
where
    S: LibraryBrowseContractBackend,
{
    let library = seed_contract_library(&store).await;
    let other_library = Library {
        id: LibraryId::new(),
        name: "Other Contract Movies".to_owned(),
        roots: vec!["local:///Other Contract Movies".to_owned()],
        options: LibraryOptions::from_preset(LibraryPreset::Movies),
    };
    store.upsert_library(&other_library).await.unwrap();

    let source_only = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Zed Source Only",
        Some("A Source Only"),
        Some("2025-01-01"),
    );
    let state_only = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Beta State Only",
        None,
        Some("2025-03-01"),
    );
    let source_and_state = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Gamma Watched",
        None,
        Some("2025-02-01"),
    );
    let duplicate_source = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Delta Duplicate Source",
        None,
        None,
    );
    let in_progress_episode = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Episode,
        "Episode In Progress",
        None,
        None,
    );
    let outside_item = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Outside Library",
        None,
        Some("2025-04-01"),
    );

    for item in [
        &source_only,
        &state_only,
        &source_and_state,
        &duplicate_source,
        &in_progress_episode,
        &outside_item,
    ] {
        store.upsert_media_item(item).await.unwrap();
    }

    insert_browse_source(&store, library.id, source_only.id, "source-only-1").await;
    sleep_for_distinct_timestamp().await;
    insert_browse_state(&store, library.id, state_only.id).await;
    sleep_for_distinct_timestamp().await;
    insert_browse_source(
        &store,
        library.id,
        source_and_state.id,
        "source-and-state-1",
    )
    .await;
    sleep_for_distinct_timestamp().await;
    insert_browse_state(&store, library.id, source_and_state.id).await;
    sleep_for_distinct_timestamp().await;
    insert_browse_source(
        &store,
        library.id,
        duplicate_source.id,
        "duplicate-source-1",
    )
    .await;
    sleep_for_distinct_timestamp().await;
    insert_browse_source(
        &store,
        library.id,
        duplicate_source.id,
        "duplicate-source-2",
    )
    .await;
    sleep_for_distinct_timestamp().await;
    insert_browse_source(
        &store,
        library.id,
        in_progress_episode.id,
        "in-progress-episode-1",
    )
    .await;
    insert_browse_source(
        &store,
        other_library.id,
        outside_item.id,
        "outside-library-1",
    )
    .await;

    let principal_id = UserPrincipalId::local_admin();
    let other_principal_id = UserPrincipalId::new("contract-other-viewer").unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal_id.clone(),
            item_id: state_only.id,
            source_id: None,
            resume_position_ms: Some(0),
            duration_ms: Some(100_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(100),
            updated_at_ms: 100,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal_id.clone(),
            item_id: source_and_state.id,
            source_id: None,
            resume_position_ms: Some(90_000),
            duration_ms: Some(100_000),
            watched: true,
            watched_at_ms: Some(300),
            last_played_at_ms: Some(300),
            updated_at_ms: 300,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal_id.clone(),
            item_id: in_progress_episode.id,
            source_id: None,
            resume_position_ms: Some(10_000),
            duration_ms: Some(100_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(500),
            updated_at_ms: 500,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: other_principal_id,
            item_id: duplicate_source.id,
            source_id: None,
            resume_position_ms: Some(25_000),
            duration_ms: Some(100_000),
            watched: true,
            watched_at_ms: Some(9_000),
            last_played_at_ms: Some(9_000),
            updated_at_ms: 9_000,
        })
        .await
        .unwrap();

    let date_added_asc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::DateAdded,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        date_added_asc,
        vec![
            source_only.id,
            state_only.id,
            source_and_state.id,
            duplicate_source.id,
            in_progress_episode.id,
        ]
    );
    assert_eq!(
        date_added_asc
            .iter()
            .filter(|id| **id == duplicate_source.id)
            .count(),
        1
    );
    assert!(!date_added_asc.contains(&outside_item.id));

    let date_added_page = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(2, 2),
                    LibraryItemBrowseSortKey::DateAdded,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        date_added_page,
        vec![source_and_state.id, duplicate_source.id]
    );

    let title_asc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::Title,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        title_asc,
        vec![
            source_only.id,
            state_only.id,
            duplicate_source.id,
            in_progress_episode.id,
            source_and_state.id,
        ]
    );

    let movie_ids = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &LibraryItemBrowseQuery {
                    facets: vec![LibraryItemBrowseFacet::Kind(MediaKind::Movie)],
                    ..browse_contract_query(
                        PageRequest::new(10, 0),
                        LibraryItemBrowseSortKey::Title,
                        LibraryItemBrowseSortOrder::Asc,
                    )
                },
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        movie_ids,
        vec![
            source_only.id,
            state_only.id,
            duplicate_source.id,
            source_and_state.id,
        ]
    );

    let impossible_kind_ids = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &LibraryItemBrowseQuery {
                    facets: vec![
                        LibraryItemBrowseFacet::Kind(MediaKind::Movie),
                        LibraryItemBrowseFacet::Kind(MediaKind::Episode),
                    ],
                    ..browse_contract_query(
                        PageRequest::new(10, 0),
                        LibraryItemBrowseSortKey::Title,
                        LibraryItemBrowseSortOrder::Asc,
                    )
                },
            )
            .await
            .unwrap(),
    );
    assert!(impossible_kind_ids.is_empty());

    let watched_ids = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &LibraryItemBrowseQuery {
                    watch_state: LibraryItemWatchStateFilter::Watched,
                    ..browse_contract_query(
                        PageRequest::new(10, 0),
                        LibraryItemBrowseSortKey::Title,
                        LibraryItemBrowseSortOrder::Asc,
                    )
                },
            )
            .await
            .unwrap(),
    );
    assert_eq!(watched_ids, vec![source_and_state.id]);

    let unwatched_ids = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &LibraryItemBrowseQuery {
                    watch_state: LibraryItemWatchStateFilter::Unwatched,
                    ..browse_contract_query(
                        PageRequest::new(10, 0),
                        LibraryItemBrowseSortKey::Title,
                        LibraryItemBrowseSortOrder::Asc,
                    )
                },
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        unwatched_ids,
        vec![
            source_only.id,
            state_only.id,
            duplicate_source.id,
            in_progress_episode.id,
        ]
    );

    let in_progress_ids = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &LibraryItemBrowseQuery {
                    watch_state: LibraryItemWatchStateFilter::InProgress,
                    ..browse_contract_query(
                        PageRequest::new(10, 0),
                        LibraryItemBrowseSortKey::LastPlayed,
                        LibraryItemBrowseSortOrder::Desc,
                    )
                },
            )
            .await
            .unwrap(),
    );
    assert_eq!(in_progress_ids, vec![in_progress_episode.id]);

    let release_none_tail = sorted_ids([duplicate_source.id, in_progress_episode.id]);
    let release_asc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::ReleaseDate,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        release_asc,
        [
            vec![source_only.id, source_and_state.id, state_only.id],
            release_none_tail.clone(),
        ]
        .concat()
    );

    let release_desc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::ReleaseDate,
                    LibraryItemBrowseSortOrder::Desc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        release_desc,
        [
            vec![state_only.id, source_and_state.id, source_only.id],
            release_none_tail,
        ]
        .concat()
    );

    let last_played_none_tail = sorted_ids([source_only.id, duplicate_source.id]);
    let last_played_asc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::LastPlayed,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        last_played_asc,
        [
            vec![state_only.id, source_and_state.id, in_progress_episode.id],
            last_played_none_tail.clone(),
        ]
        .concat()
    );

    let last_played_desc = browse_ids(
        store
            .list_library_items_for_browse(
                library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::LastPlayed,
                    LibraryItemBrowseSortOrder::Desc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        last_played_desc,
        [
            vec![in_progress_episode.id, source_and_state.id, state_only.id],
            last_played_none_tail,
        ]
        .concat()
    );
}

async fn library_item_browse_sort_edges_contract<S>(store: S)
where
    S: LibraryBrowseContractBackend,
{
    let title_library = seed_named_browse_library(&store, "Browse Title Sort").await;
    let title_upper = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Alpha Title",
        None,
        None,
    );
    let title_lower = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "alpha Title",
        None,
        None,
    );
    let title_non_ascii = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Éclair Title",
        None,
        None,
    );
    let title_tie_left = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Tie Title Left",
        Some("Tie Title"),
        None,
    );
    let title_tie_right = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Tie Title Right",
        Some("Tie Title"),
        None,
    );

    for (item, key) in [
        (&title_upper, "title-upper"),
        (&title_lower, "title-lower"),
        (&title_non_ascii, "title-non-ascii"),
        (&title_tie_left, "title-tie-left"),
        (&title_tie_right, "title-tie-right"),
    ] {
        insert_browse_item_with_source(&store, title_library.id, item, key).await;
    }

    let principal_id = UserPrincipalId::local_admin();
    let title_ties = sorted_ids([title_tie_left.id, title_tie_right.id]);
    let title_asc = browse_ids(
        store
            .list_library_items_for_browse(
                title_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::Title,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        title_asc,
        [
            vec![title_upper.id],
            title_ties.clone(),
            vec![title_lower.id, title_non_ascii.id],
        ]
        .concat()
    );

    let title_desc = browse_ids(
        store
            .list_library_items_for_browse(
                title_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::Title,
                    LibraryItemBrowseSortOrder::Desc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        title_desc,
        [
            vec![title_non_ascii.id, title_lower.id],
            title_ties,
            vec![title_upper.id],
        ]
        .concat()
    );

    let date_library = seed_named_browse_library(&store, "Browse Date Added Sort").await;
    let min_source_then_state = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Min Source Then State",
        None,
        None,
    );
    let middle_source = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Middle Source",
        None,
        None,
    );
    let late_source = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Late Source",
        None,
        None,
    );
    insert_browse_item_with_source(
        &store,
        date_library.id,
        &min_source_then_state,
        "date-min-source",
    )
    .await;
    sleep_for_distinct_timestamp().await;
    insert_browse_item_with_source(&store, date_library.id, &middle_source, "date-middle").await;
    sleep_for_distinct_timestamp().await;
    insert_browse_item_with_source(&store, date_library.id, &late_source, "date-late").await;
    sleep_for_distinct_timestamp().await;
    insert_browse_state(&store, date_library.id, min_source_then_state.id).await;

    let date_added_asc = browse_ids(
        store
            .list_library_items_for_browse(
                date_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::DateAdded,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        date_added_asc,
        vec![min_source_then_state.id, middle_source.id, late_source.id]
    );

    let date_added_desc = browse_ids(
        store
            .list_library_items_for_browse(
                date_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::DateAdded,
                    LibraryItemBrowseSortOrder::Desc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        date_added_desc,
        vec![late_source.id, middle_source.id, min_source_then_state.id]
    );

    let release_library = seed_named_browse_library(&store, "Browse Release Sort").await;
    let release_tie_left = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Release Tie Left",
        None,
        Some("2026-01-01"),
    );
    let release_tie_right = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Release Tie Right",
        None,
        Some("2026-01-01"),
    );
    let release_none = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Release None",
        None,
        None,
    );
    for (item, key) in [
        (&release_tie_left, "release-tie-left"),
        (&release_tie_right, "release-tie-right"),
        (&release_none, "release-none"),
    ] {
        insert_browse_item_with_source(&store, release_library.id, item, key).await;
    }

    let release_ties = sorted_ids([release_tie_left.id, release_tie_right.id]);
    let release_tie_asc = browse_ids(
        store
            .list_library_items_for_browse(
                release_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::ReleaseDate,
                    LibraryItemBrowseSortOrder::Asc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        release_tie_asc,
        [release_ties, vec![release_none.id]].concat()
    );

    let last_played_library = seed_named_browse_library(&store, "Browse Last Played Sort").await;
    let last_played_tie_left = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Last Played Tie Left",
        None,
        None,
    );
    let last_played_tie_right = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Last Played Tie Right",
        None,
        None,
    );
    let last_played_none = browse_contract_item(
        MediaItemId::new(),
        MediaKind::Movie,
        "Last Played None",
        None,
        None,
    );
    for (item, key) in [
        (&last_played_tie_left, "last-played-tie-left"),
        (&last_played_tie_right, "last-played-tie-right"),
        (&last_played_none, "last-played-none"),
    ] {
        insert_browse_item_with_source(&store, last_played_library.id, item, key).await;
    }
    for item_id in [last_played_tie_left.id, last_played_tie_right.id] {
        store
            .upsert_user_playback_state(UserPlaybackStateWrite {
                principal_id: principal_id.clone(),
                item_id,
                source_id: None,
                resume_position_ms: Some(0),
                duration_ms: Some(100_000),
                watched: false,
                watched_at_ms: None,
                last_played_at_ms: Some(700),
                updated_at_ms: 700,
            })
            .await
            .unwrap();
    }

    let last_played_ties = sorted_ids([last_played_tie_left.id, last_played_tie_right.id]);
    let last_played_tie_desc = browse_ids(
        store
            .list_library_items_for_browse(
                last_played_library.id,
                &principal_id,
                &browse_contract_query(
                    PageRequest::new(10, 0),
                    LibraryItemBrowseSortKey::LastPlayed,
                    LibraryItemBrowseSortOrder::Desc,
                ),
            )
            .await
            .unwrap(),
    );
    assert_eq!(
        last_played_tie_desc,
        [last_played_ties, vec![last_played_none.id]].concat()
    );
}

async fn scan_commit_writes_full_source_unit_and_resolves_failure_contract<S>(store: S)
where
    S: ScanCommitContractBackend,
{
    let library = seed_contract_library(&store).await;
    let scan_id = ScanSnapshotId::new();
    let item_id = MediaItemId::new();
    let source_id = MediaSourceId::new();
    let locator = "local:///Contract Movies/M19.mkv";
    let item = contract_media_item(item_id, "M19");
    let source = contract_media_source(library.id, item_id, source_id, locator);
    let source_state = contract_source_state(library.id, source_id, scan_id, locator);
    let evidence = contract_local_inference_evidence(source_id);

    let scan = store
        .begin_scan_snapshot(scan_id, library.id, "local:///Contract Movies")
        .await
        .unwrap();
    assert_eq!(scan.id, scan_id);
    assert_eq!(scan.status, ScanStatus::Running);

    let directory = DirectorySnapshot {
        scan_id,
        uri: "local:///Contract Movies".to_owned(),
        etag: Some("contract-root-etag".to_owned()),
        modified_at: Some("2026-05-20T00:00:00Z".to_owned()),
        child_count: 1,
    };
    store.upsert_directory_snapshot(&directory).await.unwrap();

    store
        .record_ingestion_failure(NewIngestionFailure {
            library_id: library.id,
            job_id: None,
            scan_id: Some(scan_id),
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: locator.to_owned(),
            target_kind: "source".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            message: "source was previously unreadable".to_owned(),
            retryable: true,
            failed_at_ms: 10,
        })
        .await
        .unwrap();

    let summary = store
        .commit_library_scan_source(&LibraryScanSourcePersistenceCommit {
            items: vec![item.clone()],
            source: source.clone(),
            source_state: source_state.clone(),
            library_item_states: vec![LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: true,
            }],
            local_inference_evidence: vec![evidence.clone()],
            search_projections: vec![
                CatalogSearchProjection::try_from_facet_labels(
                    item_id,
                    "M19",
                    "M19 Contract Movies M19.mkv",
                    vec!["kind:movie".to_owned(), "source:M19.mkv".to_owned()],
                )
                .unwrap(),
            ],
            source_duplicate_relationships: Vec::new(),
            resolved_ingestion_failures: vec![IngestionFailureResolution {
                library_id: library.id,
                phase: IngestionFailurePhase::Scan,
                target_uri: locator.to_owned(),
                resolved_at_ms: 20,
            }],
        })
        .await
        .unwrap();

    assert_eq!(summary.item_ids, vec![item_id]);
    assert_eq!(summary.source_id, source_id);
    assert_eq!(summary.library_item_states, 1);
    assert_eq!(summary.local_inference_evidence, 1);
    assert_eq!(summary.search_projections, 1);
    assert_eq!(summary.source_duplicate_relationships, 0);
    assert_eq!(summary.resolved_ingestion_failures, 1);

    assert_eq!(store.get_media_item(item_id).await.unwrap(), Some(item));
    assert_eq!(
        store.get_media_source(source_id).await.unwrap(),
        Some(source.clone())
    );
    assert_eq!(
        store
            .get_media_source_by_locator(library.id, locator)
            .await
            .unwrap(),
        Some(source)
    );
    assert_eq!(
        store.get_source_state(library.id, locator).await.unwrap(),
        Some(source_state.clone())
    );
    assert_eq!(
        store
            .list_source_states(library.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![source_state]
    );
    assert_eq!(
        store.list_directory_snapshots(scan_id).await.unwrap(),
        vec![directory]
    );
    assert_eq!(
        store
            .get_library_item_state(library.id, item_id)
            .await
            .unwrap(),
        Some(LibraryItemState {
            library_id: library.id,
            item_id,
            provisional: true,
        })
    );
    assert_eq!(
        store
            .get_local_inference_evidence(evidence.id)
            .await
            .unwrap(),
        Some(evidence.clone())
    );
    assert_eq!(
        store
            .list_local_inference_evidence_for_source(source_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![evidence]
    );
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels("m19", vec!["source:M19.mkv".to_owned()], 10, 0,)
                    .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        item_id
    );
    assert_eq!(
        store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library.id),
                    phase: Some(IngestionFailurePhase::Scan),
                    status: Some(IngestionFailureStatus::Resolved),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .len(),
        1
    );

    let probe = MediaProbeResult {
        duration_ms: Some(7_200_000),
        container: Some("matroska".to_owned()),
        bit_rate: Some(8_000_000),
        streams: vec![
            MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some("h264".to_owned()),
                language: None,
                duration_ms: Some(7_200_000),
                bit_rate: Some(7_000_000),
                width: Some(1920),
                height: Some(1080),
                channels: None,
                sample_rate: None,
                technical: MediaStreamTechnicalFacts {
                    codec_profile: Some("High".to_owned()),
                    codec_level: Some(41),
                    pixel_format: Some("yuv420p10le".to_owned()),
                    bits_per_raw_sample: Some(10),
                    average_frame_rate: Some(MediaRational {
                        numerator: 24_000,
                        denominator: 1_001,
                    }),
                    color: MediaColorInfo {
                        transfer: Some("smpte2084".to_owned()),
                        primaries: Some("bt2020".to_owned()),
                        ..MediaColorInfo::default()
                    },
                    hdr: MediaHdrMetadata {
                        dynamic_range: Some("hdr10".to_owned()),
                        mastering_display: true,
                        content_light_level: true,
                        ..MediaHdrMetadata::default()
                    },
                    disposition: MediaStreamDisposition {
                        default: true,
                        ..MediaStreamDisposition::default()
                    },
                    ..MediaStreamTechnicalFacts::default()
                },
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: Some("jpn".to_owned()),
                duration_ms: Some(7_200_000),
                bit_rate: Some(384_000),
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
                technical: MediaStreamTechnicalFacts {
                    channel_layout: Some("stereo".to_owned()),
                    bits_per_sample: Some(16),
                    ..MediaStreamTechnicalFacts::default()
                },
            },
        ],
    };
    store.upsert_media_probe(source_id, &probe).await.unwrap();
    assert_eq!(store.get_media_probe(source_id).await.unwrap(), Some(probe));

    let completed = store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();
    assert_eq!(completed.status, ScanStatus::Succeeded);
    assert!(completed.completed_at.is_some());
    assert_eq!(
        store.get_scan_snapshot(scan_id).await.unwrap(),
        Some(completed)
    );
}

async fn scan_commit_rolls_back_when_search_projection_fails_contract<S>(store: S)
where
    S: ScanCommitContractBackend,
{
    let library = seed_contract_library(&store).await;
    let scan_id = ScanSnapshotId::new();
    let item_id = MediaItemId::new();
    let missing_item_id = MediaItemId::new();
    let source_id = MediaSourceId::new();
    let locator = "local:///Contract Movies/BrokenSearch.mkv";
    let item = contract_media_item(item_id, "Broken Search");
    let source = contract_media_source(library.id, item_id, source_id, locator);
    let source_state = contract_source_state(library.id, source_id, scan_id, locator);

    store
        .begin_scan_snapshot(scan_id, library.id, "local:///Contract Movies")
        .await
        .unwrap();
    store
        .record_ingestion_failure(NewIngestionFailure {
            library_id: library.id,
            job_id: None,
            scan_id: Some(scan_id),
            source_id: None,
            phase: IngestionFailurePhase::Scan,
            target_uri: locator.to_owned(),
            target_kind: "source".to_owned(),
            failure_class: IngestionFailureClass::Storage,
            message: "source was previously unreadable".to_owned(),
            retryable: true,
            failed_at_ms: 10,
        })
        .await
        .unwrap();

    let err = store
        .commit_library_scan_source(&LibraryScanSourcePersistenceCommit {
            items: vec![item],
            source,
            source_state,
            library_item_states: vec![LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: true,
            }],
            local_inference_evidence: vec![contract_local_inference_evidence(source_id)],
            search_projections: vec![CatalogSearchProjection::new(
                missing_item_id,
                "Broken Search",
                "projection points at a missing item",
            )],
            source_duplicate_relationships: Vec::new(),
            resolved_ingestion_failures: vec![IngestionFailureResolution {
                library_id: library.id,
                phase: IngestionFailurePhase::Scan,
                target_uri: locator.to_owned(),
                resolved_at_ms: 20,
            }],
        })
        .await
        .unwrap_err();

    assert!(!err.to_string().is_empty());
    assert_eq!(store.get_media_item(item_id).await.unwrap(), None);
    assert_eq!(store.get_media_source(source_id).await.unwrap(), None);
    assert_eq!(
        store.get_source_state(library.id, locator).await.unwrap(),
        None
    );
    assert_eq!(
        store
            .get_library_item_state(library.id, item_id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .list_local_inference_evidence_for_source(source_id, PageRequest::first_page())
            .await
            .unwrap(),
        Vec::new()
    );
    assert_eq!(
        store
            .list_ingestion_failures(
                IngestionFailureFilter {
                    library_id: Some(library.id),
                    phase: Some(IngestionFailurePhase::Scan),
                    status: Some(IngestionFailureStatus::Open),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .len(),
        1
    );
}

fn contract_media_item(id: MediaItemId, title: &str) -> MediaItem {
    MediaItem {
        id,
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            ..CanonicalMetadata::default()
        },
    }
}

fn browse_contract_item(
    id: MediaItemId,
    kind: MediaKind,
    title: &str,
    sort_title: Option<&str>,
    release_date: Option<&str>,
) -> MediaItem {
    MediaItem {
        id,
        kind,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: title.to_owned(),
            sort_title: sort_title.map(str::to_owned),
            release_date: release_date.map(str::to_owned),
            ..CanonicalMetadata::default()
        },
    }
}

fn contract_media_source(
    library_id: LibraryId,
    item_id: MediaItemId,
    source_id: MediaSourceId,
    locator: &str,
) -> MediaSource {
    MediaSource {
        id: source_id,
        library_id,
        item_id,
        locator: locator.to_owned(),
        file_name: locator
            .rsplit('/')
            .next()
            .unwrap_or("contract.mkv")
            .to_owned(),
        size_bytes: Some(19),
        fingerprint: Some("contract-fingerprint".to_owned()),
    }
}

async fn insert_browse_source<S>(store: &S, library_id: LibraryId, item_id: MediaItemId, key: &str)
where
    S: MediaRepository + ?Sized,
{
    store
        .upsert_media_source(&MediaSource {
            id: MediaSourceId::new(),
            library_id,
            item_id,
            locator: format!("local:///Contract Browse/{key}.mkv"),
            file_name: format!("{key}.mkv"),
            size_bytes: Some(19),
            fingerprint: Some(format!("contract-browse:{key}")),
        })
        .await
        .unwrap();
}

async fn insert_browse_item_with_source<S>(
    store: &S,
    library_id: LibraryId,
    item: &MediaItem,
    key: &str,
) where
    S: MediaRepository + ?Sized,
{
    store.upsert_media_item(item).await.unwrap();
    insert_browse_source(store, library_id, item.id, key).await;
}

async fn insert_browse_state<S>(store: &S, library_id: LibraryId, item_id: MediaItemId)
where
    S: LibraryItemRepository + ?Sized,
{
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id,
            provisional: false,
        })
        .await
        .unwrap();
}

async fn sleep_for_distinct_timestamp() {
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
}

fn browse_contract_query(
    page: PageRequest,
    sort: LibraryItemBrowseSortKey,
    order: LibraryItemBrowseSortOrder,
) -> LibraryItemBrowseQuery {
    LibraryItemBrowseQuery {
        page,
        sort,
        order,
        facets: Vec::new(),
        watch_state: LibraryItemWatchStateFilter::Any,
    }
}

fn browse_ids(items: Vec<MediaItem>) -> Vec<MediaItemId> {
    items.into_iter().map(|item| item.id).collect()
}

fn sorted_ids<const N: usize>(ids: [MediaItemId; N]) -> Vec<MediaItemId> {
    let mut ids = ids.to_vec();
    ids.sort();
    ids
}

async fn seed_contract_media_item_with_source<S>(
    store: &S,
    library_id: LibraryId,
    title: &str,
    locator: &str,
) -> MediaSource
where
    S: MediaRepository + ?Sized,
{
    let item_id = MediaItemId::new();
    let source_id = MediaSourceId::new();
    let item = contract_media_item(item_id, title);
    let source = contract_media_source(library_id, item_id, source_id, locator);

    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();

    source
}

fn contract_source_state(
    library_id: LibraryId,
    source_id: MediaSourceId,
    scan_id: ScanSnapshotId,
    locator: &str,
) -> SourceState {
    SourceState {
        library_id,
        source_id: Some(source_id),
        uri: locator.to_owned(),
        size_bytes: Some(19),
        modified_at: Some("2026-05-20T00:00:00Z".to_owned()),
        etag: Some("contract-etag".to_owned()),
        fingerprint: Some("contract-fingerprint".to_owned()),
        last_seen_scan_id: scan_id,
        tombstoned: false,
    }
}

fn contract_local_inference_evidence(source_id: MediaSourceId) -> LocalInferenceEvidence {
    LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id,
        inferred_kind: MediaKind::Movie,
        inferred_title: Some("M19".to_owned()),
        inferred_year: Some(2026),
        inferred_season: None,
        inferred_episode: None,
        confidence_milli: Some(900),
        evidence_source: LocalInferenceEvidenceSource::FileName,
        evidence_value: "M19.mkv".to_owned(),
        inference_version: "contract".to_owned(),
    }
}

async fn metadata_refresh_commit_confirms_provider_mapping_and_library_state_contract<S>(store: S)
where
    S: MetadataCatalogContractBackend,
{
    let library = seed_contract_library(&store).await;
    let job = enqueue_contract_job(
        &store,
        JobKind::MetadataRefresh,
        "metadata.refresh",
        Some(library.id),
        Some(r#"{"scope":"contract"}"#),
    )
    .await;
    let item_id = MediaItemId::new();
    let original = contract_media_item(item_id, "Local Title");
    let updated = MediaItem {
        metadata: CanonicalMetadata {
            title: "Provider Title".to_owned(),
            original_title: Some("Original Provider Title".to_owned()),
            overview: Some("Provider supplied overview.".to_owned()),
            external_ids: vec![ExternalId {
                provider: ExternalProvider::Tmdb,
                value: "603".to_owned(),
            }],
            ..CanonicalMetadata::default()
        },
        ..original.clone()
    };
    let subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("Provider Title".to_owned()),
        release_year: Some(1999),
        locale: Some("en-US".to_owned()),
    };
    let raw_response = ProviderRawResponse {
        item_id,
        provider: ExternalProvider::Tmdb,
        provider_key: "movie".to_owned(),
        fetched_at: "2026-05-20T00:00:00Z".to_owned(),
        body_json: r#"{"id":603,"title":"Provider Title"}"#.to_owned(),
    };

    store.upsert_media_item(&original).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id,
            provisional: true,
        })
        .await
        .unwrap();

    let summary = store
        .commit_metadata_refresh(&MetadataRefreshPersistenceCommit {
            item: updated.clone(),
            raw_response: raw_response.clone(),
            provider_mapping: MetadataRefreshProviderMappingCommit {
                id: None,
                subject: subject.clone(),
                confidence_milli: Some(950),
                source: MetadataSource::Provider(ExternalProvider::Tmdb),
            },
        })
        .await
        .unwrap();

    let attempt = NewMetadataProviderAttempt {
        id: MetadataProviderAttemptId::new(),
        job_id: job.id,
        item_id,
        provider: ExternalProvider::Tmdb,
        status: MetadataProviderAttemptStatus::Succeeded,
        provider_key: Some("movie".to_owned()),
        matched_by: Some(MetadataMatchKind::ExternalId),
        started_at: "2026-05-20T00:00:00Z".to_owned(),
        finished_at: "2026-05-20T00:00:01Z".to_owned(),
        error_class: None,
        message: None,
    };
    store
        .insert_metadata_provider_attempt(attempt.clone())
        .await
        .unwrap();

    assert_eq!(summary.item_id, item_id);
    assert_eq!(summary.provider_subject_id, subject.id);
    assert_eq!(summary.confirmed_libraries, vec![library.id]);
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(updated.clone())
    );
    assert_eq!(
        store
            .get_provider_raw_response(item_id, &ExternalProvider::Tmdb, "movie")
            .await
            .unwrap(),
        Some(raw_response.clone())
    );
    assert_eq!(
        store
            .list_provider_raw_responses(item_id, Default::default(), PageRequest::first_page())
            .await
            .unwrap(),
        vec![raw_response]
    );
    assert_eq!(
        store
            .find_provider_subject(&ExternalProvider::Tmdb, &ProviderSubjectKind::Movie, "603")
            .await
            .unwrap(),
        Some(subject.clone())
    );
    assert_eq!(
        store
            .list_provider_subjects_for_item(item_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![subject]
    );

    let mappings = store
        .list_provider_mappings_for_item(item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].id, summary.provider_mapping_id);
    assert_eq!(mappings[0].item_id, item_id);
    assert_eq!(mappings[0].subject_id, summary.provider_subject_id);
    assert_eq!(mappings[0].confidence_milli, Some(950));
    assert_eq!(
        mappings[0].source,
        MetadataSource::Provider(ExternalProvider::Tmdb)
    );
    assert_eq!(
        store
            .get_library_item_state(library.id, item_id)
            .await
            .unwrap(),
        Some(LibraryItemState {
            library_id: library.id,
            item_id,
            provisional: false,
        })
    );
    let expected_attempt = nako_core::MetadataProviderAttemptRecord {
        id: attempt.id,
        job_id: attempt.job_id,
        item_id: attempt.item_id,
        provider: attempt.provider.clone(),
        status: attempt.status,
        provider_key: attempt.provider_key.clone(),
        matched_by: attempt.matched_by,
        started_at: attempt.started_at.clone(),
        finished_at: attempt.finished_at.clone(),
        error_class: attempt.error_class,
        message: attempt.message.clone(),
    };
    assert_eq!(
        store.list_metadata_provider_attempts(job.id).await.unwrap(),
        vec![expected_attempt.clone()]
    );
    assert_eq!(
        store
            .list_metadata_provider_attempts_for_item(
                item_id,
                MetadataAttemptFilter {
                    provider: Some(ExternalProvider::Tmdb),
                    status: Some(MetadataProviderAttemptStatus::Succeeded),
                },
                PageRequest::first_page()
            )
            .await
            .unwrap(),
        vec![expected_attempt]
    );
}

async fn nfo_import_commit_writes_catalog_graph_search_and_rolls_back_contract<S>(store: S)
where
    S: MetadataCatalogContractBackend,
{
    let library = seed_contract_library(&store).await;
    let item_id = MediaItemId::new();
    let missing_item_id = MediaItemId::new();
    let original = contract_media_item(item_id, "Original");
    let updated = contract_media_item(item_id, "NFO Title");
    let person_external_id = ExternalId {
        provider: ExternalProvider::Tmdb,
        value: "31".to_owned(),
    };
    let person = Person {
        id: PersonId::new(),
        name: "Keanu Reeves".to_owned(),
        sort_name: Some("Reeves, Keanu".to_owned()),
        overview: Some("Actor imported from NFO credits.".to_owned()),
        external_ids: vec![person_external_id.clone()],
    };
    let genre = Genre {
        id: GenreId::new(),
        name: "Action".to_owned(),
        source: MetadataSource::Nfo,
    };
    let tag = Tag {
        id: TagId::new(),
        name: "cyberpunk".to_owned(),
        source: MetadataSource::Nfo,
    };
    let collection_external_id = ExternalId {
        provider: ExternalProvider::Tmdb,
        value: "2344".to_owned(),
    };
    let collection = Collection {
        id: CollectionId::new(),
        name: "Matrix Collection".to_owned(),
        overview: Some("Franchise Collection imported from NFO.".to_owned()),
        source: MetadataSource::Nfo,
        external_ids: vec![collection_external_id.clone()],
    };
    let studio_external_id = ExternalId {
        provider: ExternalProvider::Other("wikidata".to_owned()),
        value: "Q126399".to_owned(),
    };
    let studio = Studio {
        id: StudioId::new(),
        name: "Warner Bros.".to_owned(),
        source: MetadataSource::Nfo,
        external_ids: vec![studio_external_id.clone()],
    };
    let poster = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item_id),
        kind: ImageKind::Poster,
        source_uri: "file:///library/The.Matrix/poster.jpg".to_owned(),
        provider: ExternalProvider::Local,
        cache_uri: Some("nako://artwork/poster-contract".to_owned()),
        width: Some(1000),
        height: Some(1500),
        language: Some("en".to_owned()),
        selected: true,
        content_hash: Some("sha256:contract-poster".to_owned()),
        etag: Some("poster-etag".to_owned()),
    };

    store.upsert_media_item(&original).await.unwrap();

    let summary = store
        .commit_nfo_import(&NfoImportPersistenceCommit {
            items: vec![updated.clone()],
            field_locks: vec![MetadataFieldLock {
                item_id,
                field: MetadataField::Title,
                locked: true,
                source: MetadataSource::Nfo,
            }],
            library_item_states: vec![LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: false,
            }],
            catalog_projections: vec![CatalogItemProjectionCommit {
                graph: CatalogItemGraphReplacement {
                    people: vec![person.clone()],
                    credits: vec![ItemCredit {
                        item_id,
                        person_id: person.id,
                        role: CreditRole::Actor,
                        character: Some("Neo".to_owned()),
                        sort_order: Some(1),
                    }],
                    genres: vec![genre.clone()],
                    item_genres: vec![ItemGenre {
                        item_id,
                        genre_id: genre.id,
                    }],
                    tags: vec![tag.clone()],
                    item_tags: vec![ItemTag {
                        item_id,
                        tag_id: tag.id,
                    }],
                    collections: vec![collection.clone()],
                    collection_items: vec![CollectionItem {
                        collection_id: collection.id,
                        item_id,
                        sort_order: Some(1),
                    }],
                    studios: vec![studio.clone()],
                    item_studios: vec![ItemStudio {
                        item_id,
                        studio_id: studio.id,
                    }],
                    images: vec![poster.clone()],
                    ..CatalogItemGraphReplacement::default()
                },
                search: CatalogSearchProjection::try_from_facet_labels(
                    item_id,
                    "NFO Title",
                    "NFO Title Action",
                    vec!["genre:Action".to_owned(), "kind:movie".to_owned()],
                )
                .unwrap(),
            }],
        })
        .await
        .unwrap();

    assert_eq!(summary.item_ids, vec![item_id]);
    assert_eq!(summary.locked_fields, 1);
    assert_eq!(summary.confirmed_items, 1);
    assert_eq!(summary.projected_items, 1);
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(updated.clone())
    );
    assert_eq!(
        store.list_field_locks(item_id).await.unwrap(),
        vec![MetadataFieldLock {
            item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::Nfo,
        }]
    );
    assert_eq!(
        store.list_genres(PageRequest::first_page()).await.unwrap(),
        vec![genre.clone()]
    );
    assert_eq!(
        store.list_item_genres(item_id).await.unwrap(),
        vec![ItemGenre {
            item_id,
            genre_id: genre.id,
        }]
    );
    assert_eq!(
        store.find_person_by_name("Keanu Reeves").await.unwrap(),
        Some(person.clone())
    );
    assert_eq!(
        store
            .find_person_by_external_id(&person_external_id)
            .await
            .unwrap(),
        Some(person.clone())
    );
    assert_eq!(
        store.list_people(PageRequest::first_page()).await.unwrap(),
        vec![person.clone()]
    );
    assert_eq!(
        store.list_item_credits(item_id).await.unwrap(),
        vec![ItemCredit {
            item_id,
            person_id: person.id,
            role: CreditRole::Actor,
            character: Some("Neo".to_owned()),
            sort_order: Some(1),
        }]
    );
    assert_eq!(
        store
            .list_person_items(person.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![updated.clone()]
    );
    assert_eq!(
        store.list_tags(PageRequest::first_page()).await.unwrap(),
        vec![tag.clone()]
    );
    assert_eq!(
        store.list_item_tags(item_id).await.unwrap(),
        vec![ItemTag {
            item_id,
            tag_id: tag.id,
        }]
    );
    assert_eq!(
        store
            .find_collection_by_external_id(&collection_external_id)
            .await
            .unwrap(),
        Some(collection.clone())
    );
    assert_eq!(
        store.list_item_collections(item_id).await.unwrap(),
        vec![CollectionItem {
            collection_id: collection.id,
            item_id,
            sort_order: Some(1),
        }]
    );
    assert_eq!(
        store
            .find_studio_by_external_id(&studio_external_id)
            .await
            .unwrap(),
        Some(studio.clone())
    );
    assert_eq!(
        store.list_item_studios(item_id).await.unwrap(),
        vec![ItemStudio {
            item_id,
            studio_id: studio.id,
        }]
    );
    assert_eq!(
        store
            .find_image_asset_by_source(
                &ImageOwner::Item(item_id),
                &ImageKind::Poster,
                &poster.source_uri
            )
            .await
            .unwrap(),
        Some(poster.clone())
    );
    assert_eq!(store.list_item_images(item_id).await.unwrap(), vec![poster]);
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels("nfo", vec!["genre:Action".to_owned()], 10, 0,)
                    .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        item_id
    );

    let broken = store
        .commit_nfo_import(&NfoImportPersistenceCommit {
            items: vec![MediaItem {
                metadata: CanonicalMetadata {
                    title: "Broken NFO".to_owned(),
                    ..CanonicalMetadata::default()
                },
                ..original
            }],
            field_locks: vec![MetadataFieldLock {
                item_id,
                field: MetadataField::Overview,
                locked: true,
                source: MetadataSource::Nfo,
            }],
            library_item_states: vec![LibraryItemState {
                library_id: library.id,
                item_id,
                provisional: false,
            }],
            catalog_projections: vec![CatalogItemProjectionCommit {
                graph: CatalogItemGraphReplacement {
                    genres: vec![Genre {
                        id: GenreId::new(),
                        name: "Broken".to_owned(),
                        source: MetadataSource::Nfo,
                    }],
                    item_genres: vec![ItemGenre {
                        item_id: missing_item_id,
                        genre_id: GenreId::new(),
                    }],
                    ..CatalogItemGraphReplacement::default()
                },
                search: CatalogSearchProjection::new(missing_item_id, "Broken", String::new()),
            }],
        })
        .await
        .unwrap_err();

    assert!(!broken.to_string().is_empty());
    assert_eq!(
        store
            .get_library_item_state(library.id, item_id)
            .await
            .unwrap(),
        Some(LibraryItemState {
            library_id: library.id,
            item_id,
            provisional: false,
        })
    );
    assert_eq!(
        store.list_field_locks(item_id).await.unwrap(),
        vec![MetadataFieldLock {
            item_id,
            field: MetadataField::Title,
            locked: true,
            source: MetadataSource::Nfo,
        }]
    );
    assert_eq!(
        store.list_item_tags(item_id).await.unwrap(),
        vec![ItemTag {
            item_id,
            tag_id: tag.id,
        }]
    );
    assert_eq!(
        store.list_item_credits(item_id).await.unwrap(),
        vec![ItemCredit {
            item_id,
            person_id: person.id,
            role: CreditRole::Actor,
            character: Some("Neo".to_owned()),
            sort_order: Some(1),
        }]
    );
    assert_eq!(
        store
            .search(SearchQuery::from_facet_labels("broken", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap(),
        Vec::new()
    );
}

async fn addon_metadata_write_commit_updates_projection_apply_outcome_and_rolls_back_contract<S>(
    store: S,
) where
    S: MetadataCatalogContractBackend,
{
    let library = seed_contract_library(&store).await;
    let item_id = MediaItemId::new();
    let original = contract_media_item(item_id, "Original Addon Title");
    let addon_id = nako_core::AddonId::new();
    let existing_genre = Genre {
        id: GenreId::new(),
        name: "Existing Genre".to_owned(),
        source: MetadataSource::Local,
    };
    let addon_genre = Genre {
        id: GenreId::new(),
        name: "Addon Genre".to_owned(),
        source: MetadataSource::Addon(addon_id),
    };

    store.upsert_media_item(&original).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id,
            provisional: false,
        })
        .await
        .unwrap();
    store
        .replace_item_catalog_graph(
            item_id,
            &CatalogItemGraphReplacement {
                genres: vec![existing_genre.clone()],
                item_genres: vec![ItemGenre {
                    item_id,
                    genre_id: existing_genre.id,
                }],
                ..CatalogItemGraphReplacement::default()
            },
        )
        .await
        .unwrap();
    store
        .upsert_search_projection(
            &CatalogSearchProjection::try_from_facet_labels(
                item_id,
                "Original Addon Title",
                "Original Addon Title Existing Genre",
                vec!["genre:Existing Genre".to_owned(), "kind:movie".to_owned()],
            )
            .unwrap(),
        )
        .await
        .unwrap();

    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.addon-metadata-write".to_owned(),
            name: "Contract Metadata Addon".to_owned(),
            version: "1.0.0".to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "http://127.0.0.1:43124".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.addon-metadata-write"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["metadata.write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let token = store
        .create_addon_token(NewAddonToken {
            id: AddonTokenId::new(),
            addon_id,
            label: "contract".to_owned(),
            token_prefix: "nako_at_metadata".to_owned(),
            token_hash: "hash-addon-metadata".to_owned(),
        })
        .await
        .unwrap();

    let search_only_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: nako_core::AddonSideEffectId::new(),
            addon_id,
            token_id: token.id,
            permission: AddonPermission::MetadataWrite,
            library_id: library.id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: "addon:metadata-write:search-only".to_owned(),
            provenance_json: r#"{"addon":"contract"}"#.to_owned(),
            payload_json: r#"{"title":"Search Only Addon Title"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let search_only_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Search Only Addon Title".to_owned(),
            ..CanonicalMetadata::default()
        },
        ..original.clone()
    };
    let search_only_summary = store
        .commit_addon_metadata_write(&AddonMetadataWritePersistenceCommit {
            side_effect_id: search_only_effect.id,
            item: search_only_item.clone(),
            catalog: AddonMetadataWriteCatalogCommit {
                graph: None,
                search: CatalogSearchProjection::try_from_facet_labels(
                    item_id,
                    "Search Only Addon Title",
                    "Search Only Addon Title Existing Genre",
                    vec!["genre:Existing Genre".to_owned(), "kind:movie".to_owned()],
                )
                .unwrap(),
            },
            applied_source: format!("addon:{addon_id}"),
            apply_report_json: None,
        })
        .await
        .unwrap();

    assert_eq!(search_only_summary.item_id, item_id);
    assert_eq!(search_only_summary.projected_items, 1);
    assert_eq!(
        search_only_summary.side_effect.apply_status,
        AddonSideEffectApplyStatus::Applied
    );
    assert_eq!(
        search_only_summary.side_effect.applied_item_id,
        Some(item_id)
    );
    assert_eq!(
        search_only_summary.side_effect.applied_source.as_deref(),
        Some(format!("addon:{addon_id}").as_str())
    );
    assert!(search_only_summary.side_effect.applied_at.is_some());
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(search_only_item.clone())
    );
    assert_eq!(
        store.list_item_genres(item_id).await.unwrap(),
        vec![ItemGenre {
            item_id,
            genre_id: existing_genre.id
        }]
    );
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels(
                    "search only",
                    vec!["genre:Existing Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        item_id
    );

    let graph_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: nako_core::AddonSideEffectId::new(),
            addon_id,
            token_id: token.id,
            permission: AddonPermission::MetadataWrite,
            library_id: library.id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: "addon:metadata-write:graph".to_owned(),
            provenance_json: r#"{"addon":"contract"}"#.to_owned(),
            payload_json: r#"{"title":"Graph Addon Title","genres":["Addon Genre"]}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let graph_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Graph Addon Title".to_owned(),
            genres: vec!["Addon Genre".to_owned()],
            ..CanonicalMetadata::default()
        },
        ..original.clone()
    };
    store
        .commit_addon_metadata_write(&AddonMetadataWritePersistenceCommit {
            side_effect_id: graph_effect.id,
            item: graph_item.clone(),
            catalog: AddonMetadataWriteCatalogCommit {
                graph: Some(CatalogItemGraphReplacement {
                    genres: vec![addon_genre.clone()],
                    item_genres: vec![ItemGenre {
                        item_id,
                        genre_id: addon_genre.id,
                    }],
                    ..CatalogItemGraphReplacement::default()
                }),
                search: CatalogSearchProjection::try_from_facet_labels(
                    item_id,
                    "Graph Addon Title",
                    "Graph Addon Title Addon Genre",
                    vec!["genre:Addon Genre".to_owned(), "kind:movie".to_owned()],
                )
                .unwrap(),
            },
            applied_source: format!("addon:{addon_id}"),
            apply_report_json: Some(r#"{"changed":["genres"]}"#.to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(graph_item.clone())
    );
    assert_eq!(
        store.list_item_genres(item_id).await.unwrap(),
        vec![ItemGenre {
            item_id,
            genre_id: addon_genre.id
        }]
    );
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels(
                    "graph",
                    vec!["genre:Addon Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        item_id
    );
    assert_eq!(
        store
            .find_addon_side_effect_by_idempotency_key(addon_id, "addon:metadata-write:graph")
            .await
            .unwrap()
            .unwrap()
            .apply_report_json
            .as_deref(),
        Some(r#"{"changed":["genres"]}"#)
    );

    let broken_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: nako_core::AddonSideEffectId::new(),
            addon_id,
            token_id: token.id,
            permission: AddonPermission::MetadataWrite,
            library_id: library.id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: "addon:metadata-write:broken".to_owned(),
            provenance_json: r#"{"addon":"contract"}"#.to_owned(),
            payload_json: r#"{"title":"Broken Addon Title"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();
    let broken_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Broken Addon Title".to_owned(),
            ..CanonicalMetadata::default()
        },
        ..original
    };
    let missing_item_id = MediaItemId::new();
    let broken_error = store
        .commit_addon_metadata_write(&AddonMetadataWritePersistenceCommit {
            side_effect_id: broken_effect.id,
            item: broken_item,
            catalog: AddonMetadataWriteCatalogCommit {
                graph: Some(CatalogItemGraphReplacement {
                    genres: vec![Genre {
                        id: GenreId::new(),
                        name: "Broken Genre".to_owned(),
                        source: MetadataSource::Addon(addon_id),
                    }],
                    item_genres: vec![ItemGenre {
                        item_id: missing_item_id,
                        genre_id: GenreId::new(),
                    }],
                    ..CatalogItemGraphReplacement::default()
                }),
                search: CatalogSearchProjection::new(
                    item_id,
                    "Broken Addon Title",
                    "graph replacement references a missing item",
                ),
            },
            applied_source: format!("addon:{addon_id}"),
            apply_report_json: None,
        })
        .await
        .unwrap_err();

    assert!(!broken_error.to_string().is_empty());
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(graph_item)
    );
    assert_eq!(
        store
            .find_addon_side_effect_by_idempotency_key(addon_id, "addon:metadata-write:broken")
            .await
            .unwrap()
            .unwrap()
            .apply_status,
        AddonSideEffectApplyStatus::Pending
    );
    assert_eq!(
        store
            .search(SearchQuery::from_facet_labels("broken addon", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap(),
        Vec::new()
    );
}

async fn metadata_application_commit_updates_item_projection_and_rolls_back_contract<S>(store: S)
where
    S: MetadataCatalogContractBackend,
{
    let _library = seed_contract_library(&store).await;
    let item_id = MediaItemId::new();
    let original = contract_media_item(item_id, "Original Metadata Application Title");
    store.upsert_media_item(&original).await.unwrap();

    let application_genre = Genre {
        id: GenreId::new(),
        name: "Application Genre".to_owned(),
        source: MetadataSource::User,
    };
    let applied_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Applied Metadata Application Title".to_owned(),
            genres: vec!["Application Genre".to_owned()],
            ..CanonicalMetadata::default()
        },
        ..original.clone()
    };
    let summary = store
        .commit_metadata_application(&MetadataApplicationPersistenceCommit {
            item: applied_item.clone(),
            catalog_projection: CatalogItemProjectionCommit {
                graph: CatalogItemGraphReplacement {
                    genres: vec![application_genre.clone()],
                    item_genres: vec![ItemGenre {
                        item_id,
                        genre_id: application_genre.id,
                    }],
                    ..CatalogItemGraphReplacement::default()
                },
                search: CatalogSearchProjection::try_from_facet_labels(
                    item_id,
                    "Applied Metadata Application Title",
                    "Applied Metadata Application Title Application Genre",
                    vec![
                        "genre:Application Genre".to_owned(),
                        "kind:movie".to_owned(),
                    ],
                )
                .unwrap(),
            },
        })
        .await
        .unwrap();

    assert_eq!(summary.item_id, item_id);
    assert_eq!(summary.projected_items, 1);
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(applied_item.clone())
    );
    assert_eq!(
        store.list_item_genres(item_id).await.unwrap(),
        vec![ItemGenre {
            item_id,
            genre_id: application_genre.id
        }]
    );
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels(
                    "applied",
                    vec!["genre:Application Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        item_id
    );

    let broken_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Broken Metadata Application Title".to_owned(),
            ..CanonicalMetadata::default()
        },
        ..original
    };
    let missing_item_id = MediaItemId::new();
    let broken_error = store
        .commit_metadata_application(&MetadataApplicationPersistenceCommit {
            item: broken_item,
            catalog_projection: CatalogItemProjectionCommit {
                graph: CatalogItemGraphReplacement {
                    genres: vec![Genre {
                        id: GenreId::new(),
                        name: "Broken Application Genre".to_owned(),
                        source: MetadataSource::User,
                    }],
                    item_genres: vec![ItemGenre {
                        item_id: missing_item_id,
                        genre_id: GenreId::new(),
                    }],
                    ..CatalogItemGraphReplacement::default()
                },
                search: CatalogSearchProjection::new(
                    item_id,
                    "Broken Metadata Application Title",
                    "graph replacement references a missing item",
                ),
            },
        })
        .await
        .unwrap_err();

    assert!(!broken_error.to_string().is_empty());
    assert_eq!(
        store.get_media_item(item_id).await.unwrap(),
        Some(applied_item)
    );
    assert_eq!(
        store.list_item_genres(item_id).await.unwrap(),
        vec![ItemGenre {
            item_id,
            genre_id: application_genre.id
        }]
    );
    assert_eq!(
        store
            .search(SearchQuery::from_facet_labels("broken metadata", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap(),
        Vec::new()
    );
}

async fn generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic_contract<S>(store: S)
where
    S: GeneratedArtifactMetadataApplyOutcomeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Generated Artifact Apply Outcome",
        "local:///Contract Movies/generated-artifact-apply.mkv",
    )
    .await;
    let job = enqueue_contract_job(
        &store,
        JobKind::Automation,
        "automation.external_api",
        Some(library.id),
        Some(r#"{"capability":"metadata_cleanup"}"#),
    )
    .await;
    let provider_id = nako_core::AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Contract AI".to_owned(),
            base_url: "https://automation.example.test".to_owned(),
            secret_env: None,
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 2_500,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: nako_core::AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library.id),
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"contract generated overview"}"#.to_owned(),
        })
        .await
        .unwrap();
    store
        .set_automation_artifact_status(artifact.id, AutomationArtifactStatus::Accepted)
        .await
        .unwrap();

    let applied_genre = Genre {
        id: GenreId::new(),
        name: "Generated Apply Genre".to_owned(),
        source: MetadataSource::User,
    };
    let applied_item = MediaItem {
        id: source.item_id,
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Generated Artifact Apply Outcome".to_owned(),
            overview: Some("contract generated overview".to_owned()),
            genres: vec![applied_genre.name.clone()],
            ..CanonicalMetadata::default()
        },
    };
    let mut plan = contract_generated_artifact_metadata_apply_plan(
        artifact.id,
        library.id,
        source.item_id,
        Some(source.id),
        GeneratedArtifactMetadataApplyPlanStatus::Ready,
        vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
    );
    let provider_subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "603".to_owned(),
        title: Some("Generated Artifact Provider Subject".to_owned()),
        release_year: Some(1999),
        locale: Some("en-US".to_owned()),
    };
    let provider_mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: source.item_id,
        subject_id: provider_subject.id,
        status: ProviderMappingStatus::Accepted,
        confidence_milli: Some(930),
        source: MetadataSource::User,
    };
    plan.provider_mappings = vec![GeneratedArtifactProviderMappingPlan {
        subject: GeneratedArtifactProviderSubjectPlan {
            provider: Some(ExternalProvider::Tmdb),
            provider_name: Some("tmdb".to_owned()),
            subject_kind: Some(ProviderSubjectKind::Movie),
            subject_kind_name: Some("movie".to_owned()),
            subject_key: Some("603".to_owned()),
            title: provider_subject.title.clone(),
            release_year: provider_subject.release_year,
            locale: provider_subject.locale.clone(),
        },
        action: GeneratedArtifactProviderMappingAction::Apply,
        reasons: vec![GeneratedArtifactProviderMappingReason::Ready],
        confidence_milli: Some(930),
        existing_mapping_status: None,
    }];
    plan.apply_provider_mapping_count = 1;
    let idempotency_key = "generated-artifact-apply:contract";
    let outcome = store
        .commit_generated_artifact_metadata_apply_outcome(
            &GeneratedArtifactMetadataApplyOutcomeCommit {
                id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                artifact_id: artifact.id,
                idempotency_key: idempotency_key.to_owned(),
                status: GeneratedArtifactMetadataApplyOutcomeStatus::Applied,
                applied: true,
                changed: true,
                applied_source: Some("user".to_owned()),
                item_id: Some(source.item_id),
                plan: plan.clone(),
                error_code: None,
                error_message: None,
                metadata_application: Some(MetadataApplicationPersistenceCommit {
                    item: applied_item.clone(),
                    catalog_projection: CatalogItemProjectionCommit {
                        graph: CatalogItemGraphReplacement {
                            genres: vec![applied_genre.clone()],
                            item_genres: vec![ItemGenre {
                                item_id: source.item_id,
                                genre_id: applied_genre.id,
                            }],
                            ..CatalogItemGraphReplacement::default()
                        },
                        search: CatalogSearchProjection::try_from_facet_labels(
                            source.item_id,
                            "Generated Artifact Apply Outcome",
                            "contract generated overview Generated Apply Genre",
                            vec![
                                "genre:Generated Apply Genre".to_owned(),
                                "kind:movie".to_owned(),
                            ],
                        )
                        .unwrap(),
                    },
                }),
                provider_mappings: vec![GeneratedArtifactProviderMappingApplyCommit {
                    subject: provider_subject.clone(),
                    mapping: provider_mapping.clone(),
                }],
            },
        )
        .await
        .unwrap();

    assert_eq!(outcome.artifact_id, artifact.id);
    assert_eq!(
        outcome.status,
        GeneratedArtifactMetadataApplyOutcomeStatus::Applied
    );
    assert_eq!(outcome.plan, plan);
    assert_eq!(
        store
            .find_generated_artifact_metadata_apply_outcome(artifact.id, idempotency_key)
            .await
            .unwrap()
            .unwrap()
            .id,
        outcome.id
    );
    assert_eq!(
        store
            .get_generated_artifact_metadata_apply_outcome(outcome.id)
            .await
            .unwrap()
            .unwrap()
            .id,
        outcome.id
    );
    let failed_recovery_outcome = store
        .commit_generated_artifact_metadata_apply_outcome(
            &GeneratedArtifactMetadataApplyOutcomeCommit {
                id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                artifact_id: artifact.id,
                idempotency_key: "generated-artifact-apply:failed-recovery".to_owned(),
                status: GeneratedArtifactMetadataApplyOutcomeStatus::Failed,
                applied: false,
                changed: false,
                applied_source: None,
                item_id: Some(source.item_id),
                plan: plan.clone(),
                error_code: Some("plan_not_executable".to_owned()),
                error_message: Some("target became stale before apply".to_owned()),
                metadata_application: None,
                provider_mappings: Vec::new(),
            },
        )
        .await
        .unwrap();
    let repair_entries = store
        .list_generated_artifact_metadata_apply_recovery_entries(
            GeneratedArtifactMetadataApplyRecoveryFilter {
                attention: Some(GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(repair_entries.len(), 1);
    assert_eq!(
        repair_entries[0].outcome_id,
        Some(failed_recovery_outcome.id)
    );
    assert_eq!(
        repair_entries[0].attention,
        GeneratedArtifactMetadataApplyRecoveryAttention::NeedsRepair
    );
    assert_eq!(
        repair_entries[0].reason,
        GeneratedArtifactMetadataApplyRecoveryReason::ApplyOutcomeFailed
    );
    let resolved_entries = store
        .list_generated_artifact_metadata_apply_recovery_entries(
            GeneratedArtifactMetadataApplyRecoveryFilter {
                attention: Some(GeneratedArtifactMetadataApplyRecoveryAttention::Resolved),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert!(
        resolved_entries
            .iter()
            .any(|entry| entry.outcome_id == Some(outcome.id))
    );
    assert_eq!(
        store.get_media_item(source.item_id).await.unwrap(),
        Some(applied_item.clone())
    );
    assert_eq!(
        store.list_item_genres(source.item_id).await.unwrap(),
        vec![ItemGenre {
            item_id: source.item_id,
            genre_id: applied_genre.id,
        }]
    );
    assert_eq!(
        store
            .find_provider_subject(&ExternalProvider::Tmdb, &ProviderSubjectKind::Movie, "603")
            .await
            .unwrap(),
        Some(provider_subject.clone())
    );
    assert_eq!(
        store
            .list_provider_mappings_for_item(source.item_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![provider_mapping]
    );
    assert_eq!(
        store
            .search(
                SearchQuery::from_facet_labels(
                    "generated overview",
                    vec!["genre:Generated Apply Genre".to_owned()],
                    10,
                    0,
                )
                .unwrap()
            )
            .await
            .unwrap()[0]
            .item_id,
        source.item_id
    );

    let duplicate_error = store
        .commit_generated_artifact_metadata_apply_outcome(
            &GeneratedArtifactMetadataApplyOutcomeCommit {
                id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                artifact_id: artifact.id,
                idempotency_key: idempotency_key.to_owned(),
                status: GeneratedArtifactMetadataApplyOutcomeStatus::Noop,
                applied: false,
                changed: false,
                applied_source: None,
                item_id: Some(source.item_id),
                plan: plan.clone(),
                error_code: None,
                error_message: None,
                metadata_application: None,
                provider_mappings: Vec::new(),
            },
        )
        .await
        .unwrap_err();
    assert!(!duplicate_error.to_string().is_empty());

    let broken_item = MediaItem {
        metadata: CanonicalMetadata {
            title: "Broken Generated Artifact Apply".to_owned(),
            ..CanonicalMetadata::default()
        },
        ..applied_item
    };
    let broken_key = "generated-artifact-apply:broken";
    let missing_item_id = MediaItemId::new();
    let broken_error = store
        .commit_generated_artifact_metadata_apply_outcome(
            &GeneratedArtifactMetadataApplyOutcomeCommit {
                id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                artifact_id: artifact.id,
                idempotency_key: broken_key.to_owned(),
                status: GeneratedArtifactMetadataApplyOutcomeStatus::Applied,
                applied: true,
                changed: true,
                applied_source: Some("user".to_owned()),
                item_id: Some(source.item_id),
                plan,
                error_code: None,
                error_message: None,
                metadata_application: Some(MetadataApplicationPersistenceCommit {
                    item: broken_item,
                    catalog_projection: CatalogItemProjectionCommit {
                        graph: CatalogItemGraphReplacement {
                            genres: vec![Genre {
                                id: GenreId::new(),
                                name: "Broken Generated Apply Genre".to_owned(),
                                source: MetadataSource::User,
                            }],
                            item_genres: vec![ItemGenre {
                                item_id: missing_item_id,
                                genre_id: GenreId::new(),
                            }],
                            ..CatalogItemGraphReplacement::default()
                        },
                        search: CatalogSearchProjection::new(
                            source.item_id,
                            "Broken Generated Artifact Apply",
                            "broken generated artifact apply",
                        ),
                    },
                }),
                provider_mappings: Vec::new(),
            },
        )
        .await
        .unwrap_err();

    assert!(!broken_error.to_string().is_empty());
    assert!(
        store
            .find_generated_artifact_metadata_apply_outcome(artifact.id, broken_key)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .search(SearchQuery::from_facet_labels("broken generated", Vec::new(), 10, 0).unwrap())
            .await
            .unwrap(),
        Vec::new()
    );
}

async fn generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic_contract<S>(store: S)
where
    S: GeneratedArtifactMetadataApplyOutcomeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Generated Artifact Bulk Apply Batch",
        "local:///Contract Movies/generated-artifact-bulk-apply.mkv",
    )
    .await;
    let job = enqueue_contract_job(
        &store,
        JobKind::Automation,
        "automation.external_api",
        Some(library.id),
        Some(r#"{"capability":"metadata_cleanup"}"#),
    )
    .await;
    let provider_id = nako_core::AutomationProviderId::new();
    store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Contract AI".to_owned(),
            base_url: "https://automation.example.test".to_owned(),
            secret_env: None,
            capabilities: vec![AutomationCapability::MetadataCleanup],
            timeout_ms: 2_500,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    let first_artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: nako_core::AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library.id),
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            artifact_json: r#"{"overview":"contract generated overview"}"#.to_owned(),
        })
        .await
        .unwrap();
    let second_artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: nako_core::AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::MetadataCleanup,
            kind: AutomationArtifactKind::MetadataSuggestion,
            library_id: Some(library.id),
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            artifact_json: r#"{"tagline":"contract generated tagline"}"#.to_owned(),
        })
        .await
        .unwrap();
    for artifact_id in [first_artifact.id, second_artifact.id] {
        store
            .set_automation_artifact_status(artifact_id, AutomationArtifactStatus::Accepted)
            .await
            .unwrap();
    }

    let first_plan_item = contract_generated_artifact_metadata_bulk_apply_plan_item(
        contract_generated_artifact_metadata_apply_plan(
            first_artifact.id,
            library.id,
            source.item_id,
            Some(source.id),
            GeneratedArtifactMetadataApplyPlanStatus::Ready,
            vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
        ),
    );
    let second_plan_item = contract_generated_artifact_metadata_bulk_apply_plan_item(
        contract_generated_artifact_metadata_apply_plan(
            second_artifact.id,
            library.id,
            source.item_id,
            Some(source.id),
            GeneratedArtifactMetadataApplyPlanStatus::Ready,
            vec![GeneratedArtifactMetadataApplyPlanReason::Ready],
        ),
    );
    let batch_id = GeneratedArtifactMetadataBulkApplyBatchId::new();
    let commit = contract_generated_artifact_metadata_bulk_apply_batch_commit(
        batch_id,
        "generated-artifact-bulk-apply:contract",
        vec![first_plan_item.clone(), second_plan_item.clone()],
    );

    let created = store
        .commit_generated_artifact_metadata_bulk_apply_batch(&commit)
        .await
        .unwrap();

    assert_eq!(created.id, batch_id);
    assert_eq!(created.job_id, commit.job.id);
    let batch_job = store.get_job(created.job_id).await.unwrap().unwrap();
    assert_eq!(batch_job.kind, JobKind::GeneratedArtifactMetadataBulkApply);
    assert_eq!(batch_job.status, JobStatus::Queued);
    assert_eq!(
        created.status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Queued
    );
    assert_eq!(created.selection.requested_artifact_count, 2);
    assert_eq!(created.summary.executable_artifact_count, 2);
    assert_eq!(created.items.len(), 2);
    assert_eq!(created.items[0].position, 0);
    assert_eq!(created.items[0].artifact_id, first_artifact.id);
    assert_eq!(
        created.items[0].status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending
    );
    assert_eq!(created.execution_summary.pending_item_count, 2);
    assert_eq!(created.items[0].plan_item, first_plan_item);
    assert_eq!(created.plan().items.len(), 2);
    assert_eq!(
        store
            .find_generated_artifact_metadata_bulk_apply_batch(
                "generated-artifact-bulk-apply:contract"
            )
            .await
            .unwrap()
            .unwrap()
            .id,
        batch_id
    );

    let replay = store
        .commit_generated_artifact_metadata_bulk_apply_batch(
            &contract_generated_artifact_metadata_bulk_apply_batch_commit(
                GeneratedArtifactMetadataBulkApplyBatchId::new(),
                "generated-artifact-bulk-apply:contract",
                vec![second_plan_item.clone()],
            ),
        )
        .await
        .unwrap();
    assert_eq!(replay.id, created.id);
    assert_eq!(replay.items, created.items);

    let running = store
        .update_generated_artifact_metadata_bulk_apply_batch_status(
            batch_id,
            GeneratedArtifactMetadataBulkApplyBatchStatus::Queued,
            GeneratedArtifactMetadataBulkApplyBatchStatus::Running,
        )
        .await
        .unwrap();
    assert_eq!(
        running.status,
        GeneratedArtifactMetadataBulkApplyBatchStatus::Running
    );
    let applied_outcome = store
        .commit_generated_artifact_metadata_apply_outcome(
            &GeneratedArtifactMetadataApplyOutcomeCommit {
                id: GeneratedArtifactMetadataApplyOutcomeId::new(),
                artifact_id: created.items[0].artifact_id,
                idempotency_key: created.items[0].idempotency_key.clone(),
                status: GeneratedArtifactMetadataApplyOutcomeStatus::Applied,
                applied: true,
                changed: true,
                applied_source: Some("user".to_owned()),
                item_id: Some(source.item_id),
                plan: created.items[0].plan_item.plan.clone().unwrap(),
                error_code: None,
                error_message: None,
                metadata_application: None,
                provider_mappings: Vec::new(),
            },
        )
        .await
        .unwrap();
    let with_item_outcome = store
        .commit_generated_artifact_metadata_bulk_apply_batch_item_outcome(
            &GeneratedArtifactMetadataBulkApplyBatchItemOutcomeCommit {
                batch_id,
                artifact_id: created.items[0].artifact_id,
                status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied,
                outcome_id: Some(applied_outcome.id),
                error_code: None,
                error_message: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        with_item_outcome.items[0].status,
        GeneratedArtifactMetadataBulkApplyBatchItemStatus::Applied
    );
    assert_eq!(
        with_item_outcome.items[0].outcome_id,
        Some(applied_outcome.id)
    );
    assert_eq!(with_item_outcome.execution_summary.applied_item_count, 1);
    assert_eq!(with_item_outcome.execution_summary.pending_item_count, 1);
    let invalid_transition = store
        .update_generated_artifact_metadata_bulk_apply_batch_status(
            batch_id,
            GeneratedArtifactMetadataBulkApplyBatchStatus::Queued,
            GeneratedArtifactMetadataBulkApplyBatchStatus::Completed,
        )
        .await
        .unwrap_err();
    assert!(invalid_transition.to_string().contains("cannot transition"));

    let conflict_batch_id = GeneratedArtifactMetadataBulkApplyBatchId::new();
    let mut conflict = contract_generated_artifact_metadata_bulk_apply_batch_commit(
        conflict_batch_id,
        "generated-artifact-bulk-apply:conflict",
        vec![first_plan_item],
    );
    conflict.items[0].idempotency_key = created.items[0].idempotency_key.clone();
    let conflict_error = store
        .commit_generated_artifact_metadata_bulk_apply_batch(&conflict)
        .await
        .unwrap_err();
    assert!(!conflict_error.to_string().is_empty());
    assert!(
        store
            .get_generated_artifact_metadata_bulk_apply_batch(conflict_batch_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.get_job(conflict.job.id).await.unwrap().is_none());
}

async fn metadata_candidate_review_batch_is_idempotent_and_atomic_contract<S>(store: S)
where
    S: MetadataCandidateReviewBatchContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Metadata Candidate Review Batch",
        "local:///Contract Movies/metadata-candidate-review-batch.mkv",
    )
    .await;
    let first_review = store
        .upsert_metadata_candidate_review(contract_metadata_candidate_review(
            source.item_id,
            "bangumi:batch:first",
            "first-subject",
            "First Candidate Review Batch",
            1_000,
        ))
        .await
        .unwrap();
    let first_review = store
        .set_metadata_candidate_review_status(
            first_review.id,
            MetadataCandidateReviewStatus::Accepted,
            1_100,
        )
        .await
        .unwrap()
        .unwrap();
    let second_review = store
        .upsert_metadata_candidate_review(contract_metadata_candidate_review(
            source.item_id,
            "bangumi:batch:second",
            "second-subject",
            "Second Candidate Review Batch",
            1_200,
        ))
        .await
        .unwrap();
    let second_review = store
        .set_metadata_candidate_review_status(
            second_review.id,
            MetadataCandidateReviewStatus::Accepted,
            1_300,
        )
        .await
        .unwrap()
        .unwrap();
    let first_plan = contract_metadata_candidate_review_application_plan(
        &first_review,
        MetadataCandidateReviewApplicationAction::Apply,
    );
    let second_plan = contract_metadata_candidate_review_application_plan(
        &second_review,
        MetadataCandidateReviewApplicationAction::Noop,
    );
    let batch_id = MetadataCandidateReviewBatchId::new();
    let commit = contract_metadata_candidate_review_batch_commit(
        batch_id,
        "metadata-candidate-review-batch:contract",
        vec![
            (first_review.clone(), first_plan.clone()),
            (second_review.clone(), second_plan.clone()),
        ],
    );

    let created = store
        .commit_metadata_candidate_review_batch(&commit)
        .await
        .unwrap();

    assert_eq!(created.id, batch_id);
    assert_eq!(created.job_id, commit.job.id);
    let batch_job = store.get_job(created.job_id).await.unwrap().unwrap();
    assert_eq!(batch_job.kind, JobKind::MetadataCandidateReviewBatchApply);
    assert_eq!(
        batch_job.resource_class,
        METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS
    );
    assert_eq!(batch_job.status, JobStatus::Queued);
    assert_eq!(created.status, MetadataCandidateReviewBatchStatus::Queued);
    assert_eq!(created.selection.requested_review_count, 2);
    assert_eq!(created.summary.apply_count, 1);
    assert_eq!(created.summary.noop_count, 1);
    assert_eq!(created.items.len(), 2);
    assert_eq!(created.items[0].position, 0);
    assert_eq!(created.items[0].review_id, first_review.id);
    assert_eq!(
        created.items[0].status,
        MetadataCandidateReviewBatchItemStatus::Pending
    );
    assert_eq!(created.items[0].plan, first_plan);
    assert_eq!(created.execution_summary.pending_item_count, 2);
    assert_eq!(
        store
            .find_metadata_candidate_review_batch("metadata-candidate-review-batch:contract")
            .await
            .unwrap()
            .unwrap()
            .id,
        batch_id
    );

    let replay = store
        .commit_metadata_candidate_review_batch(&contract_metadata_candidate_review_batch_commit(
            MetadataCandidateReviewBatchId::new(),
            "metadata-candidate-review-batch:contract",
            vec![(second_review.clone(), second_plan.clone())],
        ))
        .await
        .unwrap();
    assert_eq!(replay.id, created.id);
    assert_eq!(replay.items, created.items);

    let running = store
        .update_metadata_candidate_review_batch_status(
            batch_id,
            MetadataCandidateReviewBatchStatus::Queued,
            MetadataCandidateReviewBatchStatus::Running,
        )
        .await
        .unwrap();
    assert_eq!(running.status, MetadataCandidateReviewBatchStatus::Running);

    let with_item_outcome = store
        .commit_metadata_candidate_review_batch_item_outcome(
            &MetadataCandidateReviewBatchItemOutcomeCommit {
                batch_id,
                review_id: created.items[0].review_id,
                status: MetadataCandidateReviewBatchItemStatus::Applied,
                provider_subject_id: None,
                provider_mapping_id: None,
                error_code: None,
                error_message: None,
            },
        )
        .await
        .unwrap();
    assert_eq!(
        with_item_outcome.items[0].status,
        MetadataCandidateReviewBatchItemStatus::Applied
    );
    assert_eq!(with_item_outcome.execution_summary.applied_item_count, 1);
    assert_eq!(with_item_outcome.execution_summary.pending_item_count, 1);

    let invalid_transition = store
        .update_metadata_candidate_review_batch_status(
            batch_id,
            MetadataCandidateReviewBatchStatus::Queued,
            MetadataCandidateReviewBatchStatus::Completed,
        )
        .await
        .unwrap_err();
    assert!(invalid_transition.to_string().contains("cannot transition"));

    let conflict_batch_id = MetadataCandidateReviewBatchId::new();
    let mut conflict = contract_metadata_candidate_review_batch_commit(
        conflict_batch_id,
        "metadata-candidate-review-batch:conflict",
        vec![(first_review, first_plan)],
    );
    conflict.items[0].idempotency_key = created.items[0].idempotency_key.clone();
    let conflict_error = store
        .commit_metadata_candidate_review_batch(&conflict)
        .await
        .unwrap_err();
    assert!(!conflict_error.to_string().is_empty());
    assert!(
        store
            .get_metadata_candidate_review_batch(conflict_batch_id)
            .await
            .unwrap()
            .is_none()
    );
    assert!(store.get_job(conflict.job.id).await.unwrap().is_none());
}

fn contract_metadata_candidate_review(
    item_id: MediaItemId,
    source_key: &str,
    subject_key: &str,
    title: &str,
    updated_at_ms: i64,
) -> NewMetadataCandidateReview {
    let subject = MetadataCandidateSubject {
        provider: ExternalProvider::Bangumi,
        subject_kind: ProviderSubjectKind::Series,
        subject_key: subject_key.to_owned(),
        title: Some(title.to_owned()),
        release_year: Some(2026),
        locale: Some("ja-JP".to_owned()),
    };
    NewMetadataCandidateReview {
        id: MetadataCandidateReviewId::new(),
        item_id,
        source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
        source_key: source_key.to_owned(),
        plan: MetadataCandidateReviewPlan {
            root: MetadataCandidateReviewNode {
                source: MetadataCandidateSource::Provider(ExternalProvider::Bangumi),
                kind: MediaKind::Series,
                subject: Some(subject),
                metadata: nako_core::MetadataCandidateRecord {
                    title: Some(title.to_owned()),
                    release_date: Some("2026-01-01".to_owned()),
                    ..nako_core::MetadataCandidateRecord::default()
                },
            },
            related: Vec::new(),
            relationships: Vec::new(),
        },
        expires_at_ms: None,
        created_at_ms: updated_at_ms,
        updated_at_ms,
    }
}

fn contract_metadata_candidate_review_application_plan(
    review: &MetadataCandidateReviewRecord,
    action: MetadataCandidateReviewApplicationAction,
) -> MetadataCandidateReviewApplicationPlan {
    MetadataCandidateReviewApplicationPlan {
        review_id: review.id,
        item_id: review.item_id,
        action,
        reasons: vec![match action {
            MetadataCandidateReviewApplicationAction::Apply
            | MetadataCandidateReviewApplicationAction::Noop => {
                MetadataCandidateReviewApplicationReason::Ready
            }
            MetadataCandidateReviewApplicationAction::Skip => {
                MetadataCandidateReviewApplicationReason::ReviewNotAccepted
            }
        }],
        source: Some(MetadataSource::Provider(ExternalProvider::Bangumi)),
        root_subject: review.plan.root.subject.clone(),
        existing_mapping_id: None,
        existing_mapping_status: None,
    }
}

fn contract_metadata_candidate_review_batch_commit(
    batch_id: MetadataCandidateReviewBatchId,
    idempotency_key: &str,
    rows: Vec<(
        MetadataCandidateReviewRecord,
        MetadataCandidateReviewApplicationPlan,
    )>,
) -> MetadataCandidateReviewBatchCommit {
    let item_count = rows.len() as u32;
    let apply_count = rows
        .iter()
        .filter(|(_, plan)| plan.action == MetadataCandidateReviewApplicationAction::Apply)
        .count() as u32;
    let noop_count = rows
        .iter()
        .filter(|(_, plan)| plan.action == MetadataCandidateReviewApplicationAction::Noop)
        .count() as u32;
    let skip_count = rows
        .iter()
        .filter(|(_, plan)| plan.action == MetadataCandidateReviewApplicationAction::Skip)
        .count() as u32;

    MetadataCandidateReviewBatchCommit {
        id: batch_id,
        job: NewJob {
            id: JobId::new(),
            kind: JobKind::MetadataCandidateReviewBatchApply,
            resource_class: METADATA_CANDIDATE_REVIEW_BATCH_APPLY_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: None,
            source_id: None,
            input_json: Some(format!(r#"{{"batch_id":"{batch_id}"}}"#)),
        },
        idempotency_key: idempotency_key.to_owned(),
        status: MetadataCandidateReviewBatchStatus::Queued,
        selection: MetadataCandidateReviewBatchPlanSelection {
            requested_review_count: item_count,
            selected_review_count: item_count,
            duplicate_review_count: 0,
            max_review_count: 100,
        },
        summary: MetadataCandidateReviewBatchPlanSummary {
            planned_review_count: item_count,
            apply_count,
            noop_count,
            skip_count,
        },
        items: rows
            .into_iter()
            .enumerate()
            .map(
                |(index, (review, plan))| MetadataCandidateReviewBatchItemCommit {
                    review_id: review.id,
                    item_id: review.item_id,
                    position: index as u32,
                    status: MetadataCandidateReviewBatchItemStatus::Pending,
                    idempotency_key: format!(
                        "metadata-candidate-review-batch-item:{batch_id}:{}",
                        review.id
                    ),
                    expected_updated_at_ms: Some(review.updated_at_ms),
                    plan,
                },
            )
            .collect(),
    }
}

fn contract_generated_artifact_metadata_apply_plan(
    artifact_id: nako_core::AutomationArtifactId,
    library_id: LibraryId,
    item_id: MediaItemId,
    source_id: Option<MediaSourceId>,
    status: GeneratedArtifactMetadataApplyPlanStatus,
    reasons: Vec<GeneratedArtifactMetadataApplyPlanReason>,
) -> GeneratedArtifactMetadataApplyPlan {
    GeneratedArtifactMetadataApplyPlan {
        artifact_id,
        status,
        executable: status.executable(),
        reasons,
        target: GeneratedArtifactTarget::from_scope(Some(library_id), Some(item_id), source_id),
        payload: GeneratedArtifactPayloadSummary {
            valid_json: true,
            shape: GeneratedArtifactPayloadShape::Object,
            payload_fingerprint: "sha256:contract".to_owned(),
            payload_bytes: 42,
            object_field_count: Some(1),
            array_item_count: None,
            has_textual_values: true,
            has_explanation: false,
            confidence_milli: Some(810),
        },
        fields: Vec::new(),
        provider_mappings: Vec::new(),
        apply_field_count: u32::from(status.executable()),
        skipped_field_count: 0,
        noop_field_count: 0,
        apply_provider_mapping_count: 0,
        skipped_provider_mapping_count: 0,
        noop_provider_mapping_count: 0,
    }
}

fn contract_generated_artifact_metadata_bulk_apply_plan_item(
    plan: GeneratedArtifactMetadataApplyPlan,
) -> GeneratedArtifactMetadataBulkApplyPlanItem {
    GeneratedArtifactMetadataBulkApplyPlanItem {
        artifact_id: plan.artifact_id,
        status: GeneratedArtifactMetadataBulkApplyPlanItemStatus::Planned,
        executable: plan.executable,
        reasons: vec![GeneratedArtifactMetadataBulkApplyPlanItemReason::Planned],
        plan: Some(plan),
    }
}

fn contract_generated_artifact_metadata_bulk_apply_batch_commit(
    batch_id: GeneratedArtifactMetadataBulkApplyBatchId,
    idempotency_key: &str,
    plan_items: Vec<GeneratedArtifactMetadataBulkApplyPlanItem>,
) -> GeneratedArtifactMetadataBulkApplyBatchCommit {
    let item_count = plan_items.len() as u32;
    let job_id = JobId::new();
    GeneratedArtifactMetadataBulkApplyBatchCommit {
        id: batch_id,
        job: NewJob {
            id: job_id,
            kind: JobKind::GeneratedArtifactMetadataBulkApply,
            resource_class: GENERATED_ARTIFACT_METADATA_BULK_APPLY_JOB_RESOURCE_CLASS.to_owned(),
            priority: JobPriority::Normal,
            library_id: None,
            source_id: None,
            input_json: Some(format!(r#"{{"batch_id":"{batch_id}"}}"#)),
        },
        idempotency_key: idempotency_key.to_owned(),
        status: GeneratedArtifactMetadataBulkApplyBatchStatus::Queued,
        selection: GeneratedArtifactMetadataBulkApplyPlanSelection {
            requested_artifact_count: item_count,
            selected_artifact_count: item_count,
            duplicate_artifact_count: 0,
            max_artifact_count: 100,
        },
        summary: GeneratedArtifactMetadataBulkApplyPlanSummary {
            planned_artifact_count: item_count,
            missing_artifact_count: 0,
            ready_artifact_count: item_count,
            blocked_artifact_count: 0,
            stale_artifact_count: 0,
            executable_artifact_count: item_count,
            apply_field_count: item_count,
            skipped_field_count: 0,
            noop_field_count: 0,
            apply_provider_mapping_count: 0,
            skipped_provider_mapping_count: 0,
            noop_provider_mapping_count: 0,
        },
        items: plan_items
            .into_iter()
            .enumerate()
            .map(
                |(index, plan_item)| GeneratedArtifactMetadataBulkApplyBatchItemCommit {
                    artifact_id: plan_item.artifact_id,
                    position: index as u32,
                    status: GeneratedArtifactMetadataBulkApplyBatchItemStatus::Pending,
                    idempotency_key: format!(
                        "generated-artifact-bulk-apply-item:{batch_id}:{}",
                        plan_item.artifact_id
                    ),
                    plan_item,
                },
            )
            .collect(),
    }
}

async fn user_playback_state_is_principal_scoped_and_continue_watching_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let first_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "First Continue",
        "local:///Contract Movies/First Continue.mkv",
    )
    .await;
    let second_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Second Continue",
        "local:///Contract Movies/Second Continue.mkv",
    )
    .await;
    let watched_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Already Watched",
        "local:///Contract Movies/Already Watched.mkv",
    )
    .await;
    let other_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Other Principal",
        "local:///Contract Movies/Other Principal.mkv",
    )
    .await;
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("contract-second-profile").unwrap();

    let first_state = store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.clone(),
            item_id: first_source.item_id,
            source_id: Some(first_source.id),
            resume_position_ms: Some(45_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(1_000),
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();
    assert_eq!(first_state.version, 1);

    let updated_first = store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.clone(),
            item_id: first_source.item_id,
            source_id: Some(first_source.id),
            resume_position_ms: Some(90_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(3_000),
            updated_at_ms: 3_000,
        })
        .await
        .unwrap();
    assert_eq!(updated_first.version, 2);
    assert_eq!(updated_first.resume_position_ms, Some(90_000));

    let second_state = store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.clone(),
            item_id: second_source.item_id,
            source_id: Some(second_source.id),
            resume_position_ms: Some(120_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(4_000),
            updated_at_ms: 4_000,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal.clone(),
            item_id: watched_source.item_id,
            source_id: Some(watched_source.id),
            resume_position_ms: None,
            duration_ms: Some(600_000),
            watched: true,
            watched_at_ms: Some(5_000),
            last_played_at_ms: Some(5_000),
            updated_at_ms: 5_000,
        })
        .await
        .unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: other_principal.clone(),
            item_id: other_source.item_id,
            source_id: Some(other_source.id),
            resume_position_ms: Some(240_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(6_000),
            updated_at_ms: 6_000,
        })
        .await
        .unwrap();

    assert_eq!(
        store
            .get_user_playback_state(&principal, first_source.item_id)
            .await
            .unwrap(),
        Some(updated_first.clone())
    );
    assert_eq!(
        store
            .get_user_playback_state(&other_principal, first_source.item_id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .list_continue_watching_states(&principal, PageRequest::first_page())
            .await
            .unwrap(),
        vec![second_state, updated_first]
    );
}

async fn continue_watching_projection_filters_access_before_pagination_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend + ManagedArtworkContractBackend + IdentityAccessRepository,
{
    let accessible_library = seed_contract_library(&store).await;
    let inaccessible_library = seed_contract_library(&store).await;

    let user_id = UserId::new();
    let principal_id = UserPrincipalId::new(format!("continue-watching:{user_id}")).unwrap();
    let user = User {
        id: user_id,
        principal_id: principal_id.clone(),
        username: format!("continue-watching-{user_id}"),
        display_name: "Continue Watching Viewer".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
    };
    store.upsert_user(&user).await.unwrap();
    store
        .replace_role_assignments(
            user_id,
            &[RoleAssignment {
                user_id,
                role: UserRole::Viewer,
                granted_at_ms: 1_000,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::User(user_id),
            library_id: accessible_library.id,
            access: LibraryAccessLevel::Browse,
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();

    let inaccessible_source = seed_contract_media_item_with_source(
        &store,
        inaccessible_library.id,
        "Blocked Continue",
        "local:///Contract Movies/Blocked Continue.mkv",
    )
    .await;
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal_id.clone(),
            item_id: inaccessible_source.item_id,
            source_id: Some(inaccessible_source.id),
            resume_position_ms: Some(45_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(2_000),
            updated_at_ms: 2_000,
        })
        .await
        .unwrap();

    let visible_source = seed_contract_media_item_with_source(
        &store,
        accessible_library.id,
        "Visible Continue",
        "local:///Contract Movies/Visible Continue.mkv",
    )
    .await;
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: principal_id.clone(),
            item_id: visible_source.item_id,
            source_id: Some(visible_source.id),
            resume_position_ms: Some(60_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(1_000),
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();
    let visible_artwork = seed_published_selected_artwork_for_item(
        &store,
        accessible_library.id,
        visible_source.item_id,
        "continue-watching-visible-artwork",
        ImageKind::Poster,
        "https://cdn.example.test/continue-visible.jpg",
    )
    .await;

    let principal = AuthenticatedPrincipal {
        user_id,
        principal_id: principal_id.clone(),
        roles: vec![UserRole::Viewer],
        bootstrap: false,
    };
    let entries = store
        .list_continue_watching_entries(
            &principal,
            PageRequest {
                limit: 1,
                offset: 0,
            },
        )
        .await
        .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].state.principal_id, principal_id);
    assert_eq!(entries[0].state.item_id, visible_source.item_id);
    assert_eq!(entries[0].item.id, visible_source.item_id);
    assert_eq!(entries[0].images.len(), 1);
    assert_eq!(
        entries[0].images[0].selected.item_id,
        visible_source.item_id
    );
    assert_eq!(entries[0].images[0].selected.id, visible_artwork.id);
    assert_eq!(
        entries[0].images[0].artifact.item_id,
        visible_source.item_id
    );
}

async fn continue_watching_projection_honors_role_policy_and_admin_source_less_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend + IdentityAccessRepository,
{
    let role_library = seed_contract_library(&store).await;
    let role_user_id = UserId::new();
    let role_principal_id =
        UserPrincipalId::new(format!("continue-watching-role:{role_user_id}")).unwrap();
    let role_user = User {
        id: role_user_id,
        principal_id: role_principal_id.clone(),
        username: format!("continue-watching-role-{role_user_id}"),
        display_name: "Role Policy Viewer".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 2_000,
        updated_at_ms: 2_000,
    };
    store.upsert_user(&role_user).await.unwrap();
    store
        .replace_role_assignments(
            role_user_id,
            &[RoleAssignment {
                user_id: role_user_id,
                role: UserRole::Viewer,
                granted_at_ms: 2_000,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::Role(UserRole::Viewer),
            library_id: role_library.id,
            access: LibraryAccessLevel::Browse,
            created_at_ms: 2_000,
            updated_at_ms: 2_000,
        })
        .await
        .unwrap();

    let role_source = seed_contract_media_item_with_source(
        &store,
        role_library.id,
        "Role Continue",
        "local:///Contract Movies/Role Continue.mkv",
    )
    .await;
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: role_principal_id.clone(),
            item_id: role_source.item_id,
            source_id: Some(role_source.id),
            resume_position_ms: Some(90_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(4_000),
            updated_at_ms: 4_000,
        })
        .await
        .unwrap();

    let role_principal = AuthenticatedPrincipal {
        user_id: role_user_id,
        principal_id: role_principal_id.clone(),
        roles: vec![UserRole::Viewer],
        bootstrap: false,
    };
    let role_entries = store
        .list_continue_watching_entries(&role_principal, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(role_entries.len(), 1);
    assert_eq!(role_entries[0].item.id, role_source.item_id);
    assert_eq!(role_entries[0].state.item_id, role_source.item_id);

    let admin_item = contract_media_item(MediaItemId::new(), "Admin Source-less Continue");
    store.upsert_media_item(&admin_item).await.unwrap();
    store
        .upsert_user_playback_state(UserPlaybackStateWrite {
            principal_id: UserPrincipalId::local_admin(),
            item_id: admin_item.id,
            source_id: None,
            resume_position_ms: Some(15_000),
            duration_ms: Some(600_000),
            watched: false,
            watched_at_ms: None,
            last_played_at_ms: Some(5_000),
            updated_at_ms: 5_000,
        })
        .await
        .unwrap();

    let admin_entries = store
        .list_continue_watching_entries(
            &AuthenticatedPrincipal::bootstrap_admin(),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(admin_entries.len(), 1);
    assert_eq!(admin_entries[0].item.id, admin_item.id);
    assert_eq!(admin_entries[0].state.source_id, None);
}

async fn user_playlist_membership_is_principal_scoped_ordered_and_idempotent_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let first_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "First Playlist Item",
        "local:///Contract Movies/First Playlist Item.mkv",
    )
    .await;
    let second_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Second Playlist Item",
        "local:///Contract Movies/Second Playlist Item.mkv",
    )
    .await;
    let principal = UserPrincipalId::local_admin();
    let other_principal = UserPrincipalId::new("contract-second-profile").unwrap();
    let playlist_id = UserPlaylistId::new();

    let created = store
        .create_user_playlist(NewUserPlaylist {
            id: playlist_id,
            principal_id: principal.clone(),
            name: "Weekend queue".to_owned(),
            created_at_ms: 1_000,
        })
        .await
        .unwrap();
    assert_eq!(created.version, 1);
    assert_eq!(created.item_count, 0);

    let first_add = store
        .add_user_playlist_item(UserPlaylistItemWrite {
            playlist_id,
            principal_id: principal.clone(),
            item_id: first_source.item_id,
            position: None,
            expected_version: Some(created.version),
            added_at_ms: 1_100,
            updated_at_ms: 1_100,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first_add.version, 2);
    assert_eq!(first_add.item_count, 1);

    let duplicate_add = store
        .add_user_playlist_item(UserPlaylistItemWrite {
            playlist_id,
            principal_id: principal.clone(),
            item_id: first_source.item_id,
            position: None,
            expected_version: Some(first_add.version),
            added_at_ms: 1_200,
            updated_at_ms: 1_200,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(duplicate_add.version, first_add.version);
    assert_eq!(duplicate_add.item_count, 1);

    let second_add = store
        .add_user_playlist_item(UserPlaylistItemWrite {
            playlist_id,
            principal_id: principal.clone(),
            item_id: second_source.item_id,
            position: Some(0),
            expected_version: Some(duplicate_add.version),
            added_at_ms: 1_300,
            updated_at_ms: 1_300,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(second_add.version, 3);
    assert_eq!(second_add.item_count, 2);

    assert_eq!(
        store
            .list_user_playlist_items(&principal, playlist_id, PageRequest::first_page())
            .await
            .unwrap()
            .iter()
            .map(|item| (item.item_id, item.position))
            .collect::<Vec<_>>(),
        vec![(second_source.item_id, 0), (first_source.item_id, 1)]
    );
    assert_eq!(
        store
            .get_user_playlist(&other_principal, playlist_id)
            .await
            .unwrap(),
        None
    );

    let reordered = store
        .replace_user_playlist_item_order(UserPlaylistReorder {
            playlist_id,
            principal_id: principal.clone(),
            item_ids: vec![first_source.item_id, second_source.item_id],
            expected_version: Some(second_add.version),
            updated_at_ms: 1_400,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reordered.version, 4);
    assert_eq!(
        store
            .list_user_playlist_items(&principal, playlist_id, PageRequest::first_page())
            .await
            .unwrap()
            .iter()
            .map(|item| (item.item_id, item.position))
            .collect::<Vec<_>>(),
        vec![(first_source.item_id, 0), (second_source.item_id, 1)]
    );

    assert!(
        store
            .replace_user_playlist_item_order(UserPlaylistReorder {
                playlist_id,
                principal_id: principal.clone(),
                item_ids: vec![second_source.item_id, first_source.item_id],
                expected_version: Some(1),
                updated_at_ms: 1_500,
            })
            .await
            .unwrap()
            .is_none()
    );

    let after_remove = store
        .remove_user_playlist_item(UserPlaylistItemRemoval {
            playlist_id,
            principal_id: principal.clone(),
            item_id: second_source.item_id,
            expected_version: Some(reordered.version),
            updated_at_ms: 1_600,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_remove.version, 5);
    assert_eq!(after_remove.item_count, 1);

    assert!(
        store
            .delete_user_playlist(&principal, playlist_id)
            .await
            .unwrap()
    );
    assert_eq!(
        store
            .list_user_playlist_items(&principal, playlist_id, PageRequest::first_page())
            .await
            .unwrap(),
        Vec::new()
    );
}

async fn transcode_session_lifecycle_filters_cancellation_and_stale_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Runtime Session",
        "local:///Contract Movies/Runtime Session.mkv",
    )
    .await;
    let other_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Other Runtime Session",
        "local:///Contract Movies/Other Runtime Session.mkv",
    )
    .await;
    let request_key = "contract-profile:remux-primary".to_owned();
    let session_id = TranscodeSessionId::new();

    let planned = store
        .create_transcode_session(NewTranscodeSession {
            id: session_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: request_key.clone(),
            output_path: "cache/remux/contract-primary.mp4".into(),
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
            .find_active_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
            .await
            .unwrap()
            .map(|session| session.id),
        Some(session_id)
    );

    let running = store
        .set_transcode_session_state(session_id, TranscodeSessionState::Running, None, None)
        .await
        .unwrap();
    assert_eq!(running.state, TranscodeSessionState::Running);
    assert!(running.started_at.is_some());

    let cancel_requested = store
        .request_transcode_session_cancellation(session_id, "user cancelled playback".to_owned())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        cancel_requested.state,
        TranscodeSessionState::CancelRequested
    );
    assert_eq!(
        cancel_requested.failure_category,
        Some(TranscodeFailureCategory::Cancelled)
    );

    let cancelled = store
        .set_transcode_session_state(
            session_id,
            TranscodeSessionState::Cancelled,
            Some(TranscodeFailureCategory::Cancelled),
            Some("runner acknowledged cancellation".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(cancelled.state, TranscodeSessionState::Cancelled);
    assert!(cancelled.completed_at.is_some());
    assert!(
        store
            .find_active_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        store
            .find_latest_transcode_session(source.id, TranscodeSessionKind::Remux, &request_key)
            .await
            .unwrap()
            .map(|session| session.id),
        Some(session_id)
    );
    assert!(
        store
            .request_transcode_session_cancellation(session_id, "late cancel".to_owned())
            .await
            .unwrap()
            .is_none()
    );

    let hls_id = TranscodeSessionId::new();
    store
        .create_transcode_session(NewTranscodeSession {
            id: hls_id,
            source_id: source.id,
            kind: TranscodeSessionKind::HlsTranscode,
            request_key: "contract-profile:hls-primary".to_owned(),
            output_path: "cache/hls/contract-primary/index.m3u8".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();
    store
        .set_transcode_session_state(
            hls_id,
            TranscodeSessionState::Failed,
            Some(TranscodeFailureCategory::Plan),
            Some("playback transcode planning failed".to_owned()),
        )
        .await
        .unwrap();

    let other_id = TranscodeSessionId::new();
    store
        .create_transcode_session(NewTranscodeSession {
            id: other_id,
            source_id: other_source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "contract-profile:remux-other".to_owned(),
            output_path: "cache/remux/contract-other.mp4".into(),
            state: TranscodeSessionState::Running,
        })
        .await
        .unwrap();

    let filtered = store
        .list_transcode_sessions(
            TranscodeSessionListFilter {
                source_id: Some(source.id),
                kind: Some(TranscodeSessionKind::HlsTranscode),
                state: Some(TranscodeSessionState::Failed),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, hls_id);
    assert_eq!(
        filtered[0].failure_category,
        Some(TranscodeFailureCategory::Plan)
    );
    assert_eq!(
        filtered[0].failure_message.as_deref(),
        Some("playback transcode planning failed")
    );

    let failed_stale = store
        .fail_stale_transcode_sessions(
            TranscodeFailureCategory::Stale,
            "active during server startup".to_owned(),
        )
        .await
        .unwrap();
    assert_eq!(failed_stale, 1);

    let stale = store
        .get_transcode_session(other_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stale.state, TranscodeSessionState::Failed);
    assert_eq!(
        stale.failure_category,
        Some(TranscodeFailureCategory::Stale)
    );
    assert_eq!(
        store
            .get_transcode_session(session_id)
            .await
            .unwrap()
            .unwrap()
            .state,
        TranscodeSessionState::Cancelled
    );
}

async fn playback_session_tracks_user_attempt_independent_of_transcode_contract<S>(store: S)
where
    S: PlaybackRuntimeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Direct Runtime Session",
        "local:///Contract Movies/Direct Runtime Session.mkv",
    )
    .await;
    let principal_id = UserPrincipalId::local_admin();
    let now_ms = 1_779_814_400_000;
    let session_id = PlaybackSessionId::new();

    let created = store
        .create_playback_session(NewPlaybackSession {
            id: session_id,
            principal_id: principal_id.clone(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Direct,
            state: PlaybackSessionState::Active,
            client_capabilities_json: Some(
                r#"{"direct_play":true,"container":["mp4"],"video_codec":["h264"]}"#.to_owned(),
            ),
            started_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    assert_eq!(created.id, session_id);
    assert_eq!(created.mode, PlaybackSessionMode::Direct);
    assert_eq!(created.state, PlaybackSessionState::Active);
    assert_eq!(created.principal_id, principal_id);
    assert_eq!(created.source_id, source.id);
    assert_eq!(created.item_id, source.item_id);
    assert_eq!(created.transcode_session_id, None);

    let heartbeat = store
        .record_playback_session_heartbeat(PlaybackSessionHeartbeat {
            id: session_id,
            state: PlaybackSessionState::Paused,
            position_ms: Some(42_000),
            duration_ms: Some(600_000),
            heartbeat_at_ms: now_ms + 1_000,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(heartbeat.state, PlaybackSessionState::Paused);
    assert_eq!(heartbeat.position_ms, Some(42_000));
    assert_eq!(heartbeat.duration_ms, Some(600_000));
    assert_eq!(heartbeat.last_heartbeat_at_ms, Some(now_ms + 1_000));

    let listed = store
        .list_playback_sessions(
            PlaybackSessionListFilter {
                principal_id: Some(principal_id.clone()),
                source_id: Some(source.id),
                state: Some(PlaybackSessionState::Paused),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, session_id);

    let transcode_session_id = TranscodeSessionId::new();
    store
        .create_transcode_session(NewTranscodeSession {
            id: transcode_session_id,
            source_id: source.id,
            kind: TranscodeSessionKind::Remux,
            request_key: "contract-profile:linked-remux".to_owned(),
            output_path: "cache/remux/linked-remux.mp4".into(),
            state: TranscodeSessionState::Planned,
        })
        .await
        .unwrap();
    let linked = store
        .link_playback_session_transcode(session_id, transcode_session_id)
        .await
        .unwrap();
    assert_eq!(linked.transcode_session_id, Some(transcode_session_id));

    let ended = store
        .set_playback_session_state(
            session_id,
            PlaybackSessionState::Ended,
            Some(now_ms + 2_000),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ended.state, PlaybackSessionState::Ended);
    assert_eq!(ended.ended_at_ms, Some(now_ms + 2_000));

    assert!(
        store
            .record_playback_session_heartbeat(PlaybackSessionHeartbeat {
                id: session_id,
                state: PlaybackSessionState::Active,
                position_ms: Some(60_000),
                duration_ms: Some(600_000),
                heartbeat_at_ms: now_ms + 3_000,
            })
            .await
            .unwrap()
            .is_none()
    );
}

async fn renderer_session_and_command_queue_contract<S>(store: S)
where
    S: RendererRuntimeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Renderer Runtime Session",
        "local:///Contract Movies/Renderer Runtime Session.mkv",
    )
    .await;
    let owner_principal_id = UserPrincipalId::local_admin();
    let controlling_principal_id = UserPrincipalId::new("contract-controller").unwrap();
    let now_ms = 1_779_814_400_000;
    let renderer_session_id = RendererSessionId::new();

    let registered = store
        .upsert_renderer_session(NewRendererSession {
            id: renderer_session_id,
            owner_principal_id: owner_principal_id.clone(),
            target_kind: PlaybackTargetKind::NakoRemoteClient,
            display_name: "Contract Desktop".to_owned(),
            network_scope: PlaybackTargetNetworkScope::Local,
            transport_auth: PlaybackTargetTransportAuth::Bearer,
            media_capabilities_json: Some(
                r#"{"containers":["mp4"],"video_codecs":["h264"]}"#.to_owned(),
            ),
            control_capabilities: RendererControlCapabilities::basic_playback(),
            state: RendererSessionState::Online,
            last_seen_at_ms: now_ms,
            expires_at_ms: Some(now_ms + 60_000),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
        })
        .await
        .unwrap();

    assert_eq!(registered.id, renderer_session_id);
    assert_eq!(registered.owner_principal_id, owner_principal_id);
    assert_eq!(registered.target_kind, PlaybackTargetKind::NakoRemoteClient);
    assert_eq!(registered.state, RendererSessionState::Online);
    assert_eq!(registered.active_playback_session_id, None);
    assert!(
        registered
            .control_capabilities
            .supports(RendererControlCommand::Play)
    );

    let refreshed = store
        .record_renderer_session_heartbeat(RendererSessionHeartbeat {
            id: renderer_session_id,
            state: RendererSessionState::Online,
            last_seen_at_ms: now_ms + 1_000,
            expires_at_ms: Some(now_ms + 61_000),
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(refreshed.last_seen_at_ms, now_ms + 1_000);

    let listed = store
        .list_renderer_sessions(
            RendererSessionListFilter {
                owner_principal_id: Some(owner_principal_id.clone()),
                state: Some(RendererSessionState::Online),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, renderer_session_id);

    let playback_session_id = PlaybackSessionId::new();
    store
        .create_playback_session(NewPlaybackSession {
            id: playback_session_id,
            principal_id: controlling_principal_id.clone(),
            source_id: source.id,
            item_id: source.item_id,
            mode: PlaybackSessionMode::Direct,
            state: PlaybackSessionState::Active,
            client_capabilities_json: None,
            started_at_ms: now_ms + 2_000,
            updated_at_ms: now_ms + 2_000,
        })
        .await
        .unwrap();
    let attached = store
        .attach_renderer_playback_session(
            renderer_session_id,
            Some(playback_session_id),
            now_ms + 3_000,
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        attached.active_playback_session_id,
        Some(playback_session_id)
    );

    let play_command_id = RendererCommandId::new();
    let queued = store
        .create_renderer_command(NewRendererCommand {
            id: play_command_id,
            renderer_session_id,
            controlling_principal_id: controlling_principal_id.clone(),
            command: RendererControlCommand::Play,
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            playback_session_id: Some(playback_session_id),
            position_ms: Some(12_000),
            volume_percent: None,
            payload_json: Some(r#"{"reason":"contract"}"#.to_owned()),
            created_at_ms: now_ms + 4_000,
            updated_at_ms: now_ms + 4_000,
        })
        .await
        .unwrap();
    assert_eq!(queued.state, RendererCommandState::Queued);
    assert_eq!(queued.playback_session_id, Some(playback_session_id));

    let pause_command_id = RendererCommandId::new();
    store
        .create_renderer_command(NewRendererCommand {
            id: pause_command_id,
            renderer_session_id,
            controlling_principal_id: controlling_principal_id.clone(),
            command: RendererControlCommand::Pause,
            item_id: None,
            source_id: None,
            playback_session_id: Some(playback_session_id),
            position_ms: None,
            volume_percent: None,
            payload_json: None,
            created_at_ms: now_ms + 5_000,
            updated_at_ms: now_ms + 5_000,
        })
        .await
        .unwrap();

    let queued_commands = store
        .list_renderer_commands(
            RendererCommandListFilter {
                renderer_session_id: Some(renderer_session_id),
                state: Some(RendererCommandState::Queued),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(
        queued_commands
            .iter()
            .map(|command| command.id)
            .collect::<Vec<_>>(),
        vec![play_command_id, pause_command_id]
    );

    let claimed = store
        .claim_next_renderer_command(renderer_session_id, now_ms + 6_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claimed.id, play_command_id);
    assert_eq!(claimed.state, RendererCommandState::Delivered);
    assert_eq!(claimed.delivered_at_ms, Some(now_ms + 6_000));

    let completed = store
        .complete_renderer_command(RendererCommandCompletion {
            id: play_command_id,
            state: RendererCommandState::Acknowledged,
            completed_at_ms: now_ms + 7_000,
            failure_message: None,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(completed.state, RendererCommandState::Acknowledged);
    assert_eq!(completed.completed_at_ms, Some(now_ms + 7_000));
    assert!(
        store
            .complete_renderer_command(RendererCommandCompletion {
                id: play_command_id,
                state: RendererCommandState::Cancelled,
                completed_at_ms: now_ms + 8_000,
                failure_message: Some("late cancel".to_owned()),
            })
            .await
            .unwrap()
            .is_none()
    );

    let next = store
        .claim_next_renderer_command(renderer_session_id, now_ms + 9_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.id, pause_command_id);
    assert_eq!(next.state, RendererCommandState::Delivered);
}

async fn event_outbox_and_webhook_delivery_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Webhook Contract Movie",
        "local:///Contract Movies/webhook-contract.mkv",
    )
    .await;

    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: nako_core::EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: Some(source.id),
            idempotency_key: "library-scan:webhook-contract".to_owned(),
            payload_json: format!(
                r#"{{"library_id":"{}","source_id":"{}"}}"#,
                library.id, source.id
            ),
        })
        .await
        .unwrap();
    assert_eq!(event.status, OutboxEventStatus::Pending);
    assert_eq!(event.attempts, 0);
    assert_eq!(event.library_id, Some(library.id));
    assert_eq!(event.source_id, Some(source.id));

    let duplicate = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: nako_core::EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: Some(source.id),
            idempotency_key: "library-scan:webhook-contract".to_owned(),
            payload_json: r#"{"ignored":true}"#.to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(duplicate.id, event.id);
    assert_eq!(duplicate.payload_json, event.payload_json);

    let by_key = store
        .find_outbox_event_by_idempotency_key(
            DomainEventKind::LibraryScanned,
            "library-scan:webhook-contract",
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_key.id, event.id);

    let filtered = store
        .list_outbox_events(
            OutboxEventListFilter {
                kind: Some(DomainEventKind::LibraryScanned),
                status: Some(OutboxEventStatus::Pending),
                library_id: Some(library.id),
                source_id: Some(source.id),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, event.id);

    let endpoint_id = nako_core::WebhookEndpointId::new();
    let endpoint = store
        .upsert_webhook_endpoint(NewWebhookEndpoint {
            id: endpoint_id,
            name: "Library Scan Hook".to_owned(),
            url: "https://hooks.example.test/nako".to_owned(),
            secret_env: Some("NAKO_WEBHOOK_SECRET".to_owned()),
            subscribed_event_kinds: vec![
                DomainEventKind::LibraryScanned.as_str().to_owned(),
                DomainEventKind::NfoImported.as_str().to_owned(),
            ],
            timeout_ms: 1_500,
            max_attempts: 3,
            status: WebhookEndpointStatus::Enabled,
        })
        .await
        .unwrap();
    assert_eq!(endpoint.id, endpoint_id);
    assert_eq!(endpoint.secret_env.as_deref(), Some("NAKO_WEBHOOK_SECRET"));
    assert_eq!(endpoint.status, WebhookEndpointStatus::Enabled);

    let disabled = store
        .upsert_webhook_endpoint(NewWebhookEndpoint {
            id: nako_core::WebhookEndpointId::new(),
            name: "Disabled Hook".to_owned(),
            url: "https://hooks.example.test/disabled".to_owned(),
            secret_env: None,
            subscribed_event_kinds: vec![DomainEventKind::LibraryScanned.as_str().to_owned()],
            timeout_ms: 2_000,
            max_attempts: 1,
            status: WebhookEndpointStatus::Disabled,
        })
        .await
        .unwrap();

    let enabled = store.list_enabled_webhook_endpoints().await.unwrap();
    assert_eq!(
        enabled
            .iter()
            .map(|endpoint| endpoint.id)
            .collect::<Vec<_>>(),
        vec![endpoint_id]
    );
    assert!(!enabled.iter().any(|endpoint| endpoint.id == disabled.id));

    let attempt = store
        .create_webhook_delivery_attempt(NewWebhookDeliveryAttempt {
            id: nako_core::WebhookDeliveryAttemptId::new(),
            endpoint_id,
            event_id: event.id,
            attempt_number: 1,
        })
        .await
        .unwrap();
    assert_eq!(attempt.status, WebhookDeliveryStatus::Pending);
    assert_eq!(attempt.completed_at, None);

    let failed = store
        .set_webhook_delivery_attempt_result(
            attempt.id,
            WebhookDeliveryStatus::Failed,
            Some(503),
            Some("temporarily unavailable".to_owned()),
            Some("2026-05-20T00:01:00.000Z".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(failed.status, WebhookDeliveryStatus::Failed);
    assert_eq!(failed.http_status, Some(503));
    assert_eq!(failed.error.as_deref(), Some("temporarily unavailable"));
    assert!(failed.completed_at.is_some());
    assert_eq!(
        failed.next_retry_at.as_deref(),
        Some("2026-05-20T00:01:00.000Z")
    );

    let attempts = store
        .list_webhook_delivery_attempts(event.id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0].id, attempt.id);
}

async fn addon_event_delivery_attempt_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Addon Event Contract Movie",
        "local:///Contract Movies/addon-event-contract.mkv",
    )
    .await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: nako_core::EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: Some(source.id),
            idempotency_key: "library-scan:addon-event-contract".to_owned(),
            payload_json: format!(
                r#"{{"library_id":"{}","source_id":"{}"}}"#,
                library.id, source.id
            ),
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.event-delivery".to_owned(),
            name: "Contract Event Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.event-delivery"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["webhook_event_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let attempt = store
        .create_addon_event_delivery_attempt(NewAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            attempt_number: 1,
        })
        .await
        .unwrap();
    assert_eq!(attempt.addon_id, addon_id);
    assert_eq!(attempt.event_id, event.id);
    assert_eq!(attempt.declaration_id, "library-scanned");
    assert_eq!(attempt.status, AddonEventDeliveryStatus::Pending);
    assert_eq!(attempt.completed_at, None);

    let failed = store
        .set_addon_event_delivery_attempt_result(
            attempt.id,
            AddonEventDeliveryStatus::Failed,
            Some(503),
            Some("sidecar unavailable".to_owned()),
            Some("2026-05-25T00:01:00.000Z".to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(failed.status, AddonEventDeliveryStatus::Failed);
    assert_eq!(failed.http_status, Some(503));
    assert_eq!(failed.error.as_deref(), Some("sidecar unavailable"));
    assert_eq!(
        failed.next_retry_at.as_deref(),
        Some("2026-05-25T00:01:00.000Z")
    );
    assert!(failed.completed_at.is_some());

    let second_attempt = store
        .create_addon_event_delivery_attempt(NewAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            attempt_number: 2,
        })
        .await
        .unwrap();
    let succeeded = store
        .set_addon_event_delivery_attempt_result(
            second_attempt.id,
            AddonEventDeliveryStatus::Succeeded,
            Some(202),
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(succeeded.status, AddonEventDeliveryStatus::Succeeded);
    assert_eq!(succeeded.http_status, Some(202));
    assert_eq!(succeeded.error, None);

    let attempts = store
        .list_addon_event_delivery_attempts(event.id)
        .await
        .unwrap();
    assert_eq!(attempts.len(), 2);
    assert_eq!(
        attempts
            .iter()
            .map(|attempt| attempt.attempt_number)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );

    let addon_attempts = store
        .list_addon_event_delivery_attempts_for_addon(addon_id, event.id, "library-scanned")
        .await
        .unwrap();
    assert_eq!(addon_attempts, attempts);
}

async fn addon_event_scheduler_due_work_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Addon Event Scheduler Movie",
        "local:///Contract Movies/addon-event-scheduler.mkv",
    )
    .await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: nako_core::EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: Some(source.id),
            idempotency_key: "library-scan:addon-event-scheduler".to_owned(),
            payload_json: r#"{"secret":"nako_at_should_not_echo","safe":true}"#.to_owned(),
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    let manifest_json = r#"{"id":"dev.nako.contract.event-scheduler"}"#.to_owned();
    let fingerprint = AddonManifestFingerprint::new(&manifest_json);
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.event-scheduler".to_owned(),
            name: "Contract Event Scheduler Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json,
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["webhook_event_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    let disabled_addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: disabled_addon_id,
            manifest_id: "dev.nako.contract.disabled-event-scheduler".to_owned(),
            name: "Disabled Event Scheduler Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/disabled-addon".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.disabled-event-scheduler"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["webhook_event_read".to_owned()],
            status: AddonStatus::Disabled,
        })
        .await
        .unwrap();
    store
        .replace_addon_routing_plans(
            addon_id,
            vec![
                NewAddonRoutingPlan {
                    id: AddonRoutingPlanId::new(),
                    addon_id,
                    manifest_id: "dev.nako.contract.event-scheduler".to_owned(),
                    manifest_version: "0.1.0".to_owned(),
                    manifest_fingerprint: fingerprint.clone(),
                    declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                    declaration_id: "deferred-library-scanned".to_owned(),
                    status: AddonRoutingPlanStatus::Deferred,
                    target: AddonRoutingPlanTarget::None,
                    safe_reason_code: Some("missing_grant".to_owned()),
                    job_kind: None,
                    event_kind: Some(DomainEventKind::LibraryScanned.as_str().to_owned()),
                    plan_json: r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"deferred-library-scanned"}"#.to_owned(),
                },
                NewAddonRoutingPlan {
                    id: AddonRoutingPlanId::new(),
                    addon_id,
                    manifest_id: "dev.nako.contract.event-scheduler".to_owned(),
                    manifest_version: "0.1.0".to_owned(),
                    manifest_fingerprint: fingerprint,
                    declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                    declaration_id: "library-scanned".to_owned(),
                    status: AddonRoutingPlanStatus::Executable,
                    target: AddonRoutingPlanTarget::EventOutbox,
                    safe_reason_code: None,
                    job_kind: None,
                    event_kind: Some(DomainEventKind::LibraryScanned.as_str().to_owned()),
                    plan_json: r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"library-scanned"}"#.to_owned(),
                },
            ],
        )
        .await
        .unwrap();
    store
        .replace_addon_routing_plans(
            disabled_addon_id,
            vec![NewAddonRoutingPlan {
                id: AddonRoutingPlanId::new(),
                addon_id: disabled_addon_id,
                manifest_id: "dev.nako.contract.disabled-event-scheduler".to_owned(),
                manifest_version: "0.1.0".to_owned(),
                manifest_fingerprint: AddonManifestFingerprint::new(
                    r#"{"id":"dev.nako.contract.disabled-event-scheduler"}"#,
                ),
                declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                declaration_id: "library-scanned".to_owned(),
                status: AddonRoutingPlanStatus::Executable,
                target: AddonRoutingPlanTarget::EventOutbox,
                safe_reason_code: None,
                job_kind: None,
                event_kind: Some(DomainEventKind::LibraryScanned.as_str().to_owned()),
                plan_json:
                    r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"library-scanned"}"#
                        .to_owned(),
            }],
        )
        .await
        .unwrap();

    let initial_work = store
        .list_addon_event_scheduler_work(event.id)
        .await
        .unwrap();
    assert_eq!(
        initial_work
            .iter()
            .map(|work| work.declaration_id.as_str())
            .collect::<Vec<_>>(),
        vec!["deferred-library-scanned", "library-scanned"]
    );
    assert!(
        !initial_work
            .iter()
            .any(|work| work.addon_id == disabled_addon_id)
    );
    let executable = initial_work
        .iter()
        .find(|work| work.declaration_id == "library-scanned")
        .unwrap();
    assert_eq!(executable.addon_id, addon_id);
    assert_eq!(executable.event_id, event.id);
    assert_eq!(
        executable.event_kind,
        DomainEventKind::LibraryScanned.as_str()
    );
    assert_eq!(
        executable.routing_plan_status,
        AddonRoutingPlanStatus::Executable
    );
    assert_eq!(
        executable.routing_plan_target,
        AddonRoutingPlanTarget::EventOutbox
    );
    assert_eq!(executable.attempt_count, 0);
    assert_eq!(executable.next_attempt_number, 1);
    assert_eq!(executable.latest_attempt_status, None);
    assert!(!executable.has_succeeded);
    assert!(!executable.has_in_flight);
    let deferred = initial_work
        .iter()
        .find(|work| work.declaration_id == "deferred-library-scanned")
        .unwrap();
    assert_eq!(
        deferred.routing_plan_status,
        AddonRoutingPlanStatus::Deferred
    );
    assert_eq!(deferred.routing_plan_target, AddonRoutingPlanTarget::None);
    assert_eq!(
        deferred.routing_plan_safe_reason_code.as_deref(),
        Some("missing_grant")
    );

    let attempt = store
        .create_addon_event_delivery_attempt(NewAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            attempt_number: 1,
        })
        .await
        .unwrap();
    store
        .set_addon_event_delivery_attempt_result(
            attempt.id,
            AddonEventDeliveryStatus::Failed,
            Some(503),
            Some(r#"{"safe_error_code":"retryable_http_failure"}"#.to_owned()),
            Some("2026-05-25T00:01:00.000Z".to_owned()),
        )
        .await
        .unwrap();

    let retry_work = store
        .list_addon_event_scheduler_work(event.id)
        .await
        .unwrap();
    let executable = retry_work
        .iter()
        .find(|work| work.declaration_id == "library-scanned")
        .unwrap();
    assert_eq!(executable.attempt_count, 1);
    assert_eq!(executable.next_attempt_number, 2);
    assert_eq!(
        executable.latest_attempt_status,
        Some(AddonEventDeliveryStatus::Failed)
    );
    assert_eq!(executable.latest_http_status, Some(503));
    assert_eq!(
        executable.latest_next_retry_at.as_deref(),
        Some("2026-05-25T00:01:00.000Z")
    );
    assert!(!executable.has_succeeded);
    assert!(!executable.has_in_flight);
}

async fn addon_event_scheduler_claim_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let event = store
        .enqueue_outbox_event(NewOutboxEvent {
            id: nako_core::EventId::new(),
            kind: DomainEventKind::LibraryScanned,
            subject: DomainEventSubject::Library(library.id),
            library_id: Some(library.id),
            source_id: None,
            idempotency_key: "library-scan:addon-event-scheduler-claim".to_owned(),
            payload_json: r#"{"safe":true}"#.to_owned(),
        })
        .await
        .unwrap();

    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.event-scheduler-claim".to_owned(),
            name: "Contract Event Scheduler Claim Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.event-scheduler-claim"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["webhook_event_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let first = store
        .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            max_attempts: 2,
            now: "2026-05-25T00:00:00.000Z".to_owned(),
            lease_expires_at: "2026-05-25T00:05:00.000Z".to_owned(),
            forced_replay: false,
            replay_reason_code: None,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.status, AddonEventDeliveryStatus::Running);
    assert_eq!(first.attempt_number, 1);
    assert_eq!(
        first.lease_expires_at.as_deref(),
        Some("2026-05-25T00:05:00.000Z")
    );

    let duplicate_in_flight = store
        .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            max_attempts: 2,
            now: "2026-05-25T00:01:00.000Z".to_owned(),
            lease_expires_at: "2026-05-25T00:06:00.000Z".to_owned(),
            forced_replay: false,
            replay_reason_code: None,
        })
        .await
        .unwrap();
    assert_eq!(duplicate_in_flight, None);

    store
        .set_addon_event_delivery_attempt_result(
            first.id,
            AddonEventDeliveryStatus::Failed,
            Some(503),
            Some(r#"{"safe_error_code":"retryable_http_failure"}"#.to_owned()),
            Some("2026-05-25T00:10:00.000Z".to_owned()),
        )
        .await
        .unwrap();
    let retry_waiting = store
        .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            max_attempts: 2,
            now: "2026-05-25T00:09:00.000Z".to_owned(),
            lease_expires_at: "2026-05-25T00:14:00.000Z".to_owned(),
            forced_replay: false,
            replay_reason_code: None,
        })
        .await
        .unwrap();
    assert_eq!(retry_waiting, None);

    let second = store
        .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            max_attempts: 2,
            now: "2026-05-25T00:10:00.000Z".to_owned(),
            lease_expires_at: "2026-05-25T00:15:00.000Z".to_owned(),
            forced_replay: false,
            replay_reason_code: None,
        })
        .await
        .unwrap()
        .unwrap();
    assert_eq!(second.status, AddonEventDeliveryStatus::Running);
    assert_eq!(second.attempt_number, 2);

    let exhausted = store
        .claim_addon_event_delivery_attempt(ClaimAddonEventDeliveryAttempt {
            id: AddonEventDeliveryAttemptId::new(),
            addon_id,
            event_id: event.id,
            declaration_id: "library-scanned".to_owned(),
            max_attempts: 2,
            now: "2026-05-25T00:16:00.000Z".to_owned(),
            lease_expires_at: "2026-05-25T00:21:00.000Z".to_owned(),
            forced_replay: false,
            replay_reason_code: None,
        })
        .await
        .unwrap();
    assert_eq!(exhausted, None);
}

async fn addon_registration_token_grant_and_side_effect_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Addon Contract Movie",
        "local:///Contract Movies/addon-contract.mkv",
    )
    .await;

    let addon_id = nako_core::AddonId::new();
    let addon = store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.addon".to_owned(),
            name: "Contract Addon".to_owned(),
            version: "1.0.0".to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "http://127.0.0.1:43123".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.addon","name":"Contract Addon"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["metadata.write".to_owned(), "artwork.write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    assert_eq!(addon.id, addon_id);
    assert_eq!(addon.status, AddonStatus::Enabled);

    let by_manifest = store
        .find_addon_registration_by_manifest_id("dev.nako.contract.addon")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_manifest.id, addon_id);
    assert_eq!(
        store
            .list_addon_registrations(Some(AddonStatus::Enabled))
            .await
            .unwrap()
            .iter()
            .map(|addon| addon.id)
            .collect::<Vec<_>>(),
        vec![addon_id]
    );
    let disabled_addon = store
        .update_addon_registration_status(addon_id, AddonStatus::Disabled)
        .await
        .unwrap()
        .expect("addon status update returns registration");
    assert_eq!(disabled_addon.id, addon_id);
    assert_eq!(disabled_addon.status, AddonStatus::Disabled);
    assert!(
        store
            .list_addon_registrations(Some(AddonStatus::Enabled))
            .await
            .unwrap()
            .is_empty()
    );
    let addon = store
        .update_addon_registration_status(addon_id, AddonStatus::Enabled)
        .await
        .unwrap()
        .expect("addon status update returns registration");
    assert_eq!(addon.status, AddonStatus::Enabled);
    assert!(
        store
            .update_addon_registration_status(AddonId::new(), AddonStatus::Disabled)
            .await
            .unwrap()
            .is_none()
    );

    let first_token = store
        .create_addon_token(NewAddonToken {
            id: AddonTokenId::new(),
            addon_id,
            label: "initial".to_owned(),
            token_prefix: "nako_at_initial".to_owned(),
            token_hash: "hash-initial".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(first_token.token_hash, "hash-initial");

    let loaded_token = store
        .find_addon_token_by_hash("hash-initial")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(loaded_token.id, first_token.id);

    let used = store
        .mark_addon_token_used(first_token.id)
        .await
        .unwrap()
        .unwrap();
    assert!(used.last_used_at.is_some());

    let (rotated, active) = store
        .rotate_addon_token(
            first_token.id,
            NewAddonToken {
                id: AddonTokenId::new(),
                addon_id,
                label: "rotated".to_owned(),
                token_prefix: "nako_at_rotated".to_owned(),
                token_hash: "hash-rotated".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(rotated.status.as_str(), "rotated");
    assert!(rotated.rotated_at.is_some());
    assert_eq!(active.token_hash, "hash-rotated");

    let grants = store
        .replace_addon_grants(
            addon_id,
            vec![
                NewAddonGrant {
                    id: nako_core::AddonGrantId::new(),
                    addon_id,
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library.id),
                },
                NewAddonGrant {
                    id: nako_core::AddonGrantId::new(),
                    addon_id,
                    permission: AddonPermission::ArtworkWrite,
                    library_id: None,
                },
            ],
        )
        .await
        .unwrap();
    assert_eq!(grants.len(), 2);
    assert!(grants.iter().any(|grant| {
        grant.permission == AddonPermission::MetadataWrite && grant.library_id == Some(library.id)
    }));

    let side_effect = NewAddonSideEffect {
        id: nako_core::AddonSideEffectId::new(),
        addon_id,
        token_id: active.id,
        permission: AddonPermission::MetadataWrite,
        library_id: library.id,
        target: AddonSideEffectTarget::media_item(source.item_id),
        idempotency_key: "addon:metadata-write:contract".to_owned(),
        provenance_json: r#"{"addon":"contract"}"#.to_owned(),
        payload_json: r#"{"title":"Addon Contract Movie"}"#.to_owned(),
        validation_status: AddonSideEffectValidationStatus::Accepted,
        safe_error_code: None,
    };
    let created = store
        .create_addon_side_effect(side_effect.clone())
        .await
        .unwrap();
    assert_eq!(created.apply_status, AddonSideEffectApplyStatus::Pending);
    assert_eq!(created.provenance_json, r#"{"addon":"contract"}"#);
    assert_eq!(created.payload_json, r#"{"title":"Addon Contract Movie"}"#);
    assert_eq!(
        created.request_fingerprint,
        AddonSideEffectRequestFingerprint::new(
            AddonPermission::MetadataWrite,
            library.id,
            &AddonSideEffectTarget::media_item(source.item_id),
            r#"{"addon":"contract"}"#,
            r#"{"title":"Addon Contract Movie"}"#,
        )
    );

    let duplicate = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: nako_core::AddonSideEffectId::new(),
            payload_json: r#"{"ignored":true}"#.to_owned(),
            ..side_effect
        })
        .await
        .unwrap();
    assert_eq!(duplicate.id, created.id);
    assert_eq!(duplicate.payload_json, created.payload_json);

    let applied = store
        .set_addon_side_effect_apply_outcome(
            created.id,
            AddonSideEffectApplyOutcome {
                status: AddonSideEffectApplyStatus::Applied,
                error_code: None,
                item_id: Some(source.item_id),
                source: Some("addon:contract".to_owned()),
                report_json: Some(r#"{"changed":["title"]}"#.to_owned()),
            },
        )
        .await
        .unwrap();
    assert_eq!(applied.apply_status, AddonSideEffectApplyStatus::Applied);
    assert_eq!(applied.applied_item_id, Some(source.item_id));
    assert!(applied.applied_at.is_some());

    let revoked = store.revoke_addon_token(active.id).await.unwrap().unwrap();
    assert_eq!(revoked.status.as_str(), "revoked");
    assert!(revoked.revoked_at.is_some());

    let active_for_unregister = store
        .create_addon_token(NewAddonToken {
            id: AddonTokenId::new(),
            addon_id,
            label: "unregister active".to_owned(),
            token_prefix: "nako_at_unregister".to_owned(),
            token_hash: "hash-unregister-active".to_owned(),
        })
        .await
        .unwrap();
    store
        .replace_addon_grants(
            addon_id,
            vec![NewAddonGrant {
                id: nako_core::AddonGrantId::new(),
                addon_id,
                permission: AddonPermission::MetadataWrite,
                library_id: Some(library.id),
            }],
        )
        .await
        .unwrap();
    let unregistered = store
        .unregister_addon_registration(addon_id)
        .await
        .unwrap()
        .expect("addon unregister returns registration");
    assert_eq!(unregistered.status, AddonStatus::Unregistered);
    assert!(
        store
            .list_addon_registrations(Some(AddonStatus::Unregistered))
            .await
            .unwrap()
            .iter()
            .any(|addon| addon.id == addon_id)
    );
    assert_eq!(
        store
            .get_addon_token(active_for_unregister.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        nako_core::AddonTokenStatus::Revoked
    );
    assert!(store.list_addon_grants(addon_id).await.unwrap().is_empty());
    assert!(
        store
            .unregister_addon_registration(AddonId::new())
            .await
            .unwrap()
            .is_none()
    );
}

async fn addon_routing_plan_replaces_manifest_declarations_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let addon_id = AddonId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.nako.contract.routing".to_owned(),
            name: "Contract Routing Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.routing","version":"0.1.0"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["automation_run".to_owned(), "webhook_event_read".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let first_fingerprint =
        AddonManifestFingerprint::new(r#"{"id":"dev.nako.contract.routing","version":"0.1.0"}"#);
    let first_plans = vec![
        NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: "dev.nako.contract.routing".to_owned(),
            manifest_version: "0.1.0".to_owned(),
            manifest_fingerprint: first_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::Task,
            declaration_id: "bulk-refresh".to_owned(),
            status: AddonRoutingPlanStatus::Executable,
            target: AddonRoutingPlanTarget::AddonTaskJob,
            safe_reason_code: None,
            job_kind: Some(JobKind::AddonTask),
            event_kind: None,
            plan_json: r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"bulk-refresh"}"#
                .to_owned(),
        },
        NewAddonRoutingPlan {
            id: AddonRoutingPlanId::new(),
            addon_id,
            manifest_id: "dev.nako.contract.routing".to_owned(),
            manifest_version: "0.1.0".to_owned(),
            manifest_fingerprint: first_fingerprint.clone(),
            declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
            declaration_id: "library-scanned".to_owned(),
            status: AddonRoutingPlanStatus::Executable,
            target: AddonRoutingPlanTarget::EventOutbox,
            safe_reason_code: None,
            job_kind: None,
            event_kind: Some(DomainEventKind::LibraryScanned.as_str().to_owned()),
            plan_json:
                r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"library-scanned"}"#
                    .to_owned(),
        },
    ];

    let created = store
        .replace_addon_routing_plans(addon_id, first_plans.clone())
        .await
        .unwrap();
    assert_eq!(created.len(), 2);
    assert!(created.iter().any(|plan| {
        plan.declaration_id == "bulk-refresh"
            && plan.status == AddonRoutingPlanStatus::Executable
            && plan.target == AddonRoutingPlanTarget::AddonTaskJob
            && plan.job_kind == Some(JobKind::AddonTask)
    }));
    assert!(created.iter().any(|plan| {
        plan.declaration_id == "library-scanned"
            && plan.event_kind.as_deref() == Some(DomainEventKind::LibraryScanned.as_str())
    }));

    let idempotent = store
        .replace_addon_routing_plans(addon_id, first_plans)
        .await
        .unwrap();
    assert_eq!(idempotent.len(), 2);
    assert_eq!(
        idempotent
            .iter()
            .filter(|plan| plan.declaration_id == "bulk-refresh")
            .count(),
        1
    );

    let next_fingerprint =
        AddonManifestFingerprint::new(r#"{"id":"dev.nako.contract.routing","version":"0.2.0"}"#);
    let replaced = store
        .replace_addon_routing_plans(
            addon_id,
            vec![NewAddonRoutingPlan {
                id: AddonRoutingPlanId::new(),
                addon_id,
                manifest_id: "dev.nako.contract.routing".to_owned(),
                manifest_version: "0.2.0".to_owned(),
                manifest_fingerprint: next_fingerprint.clone(),
                declaration_kind: AddonRoutingDeclarationKind::EventSubscription,
                declaration_id: "metadata-refreshed".to_owned(),
                status: AddonRoutingPlanStatus::Deferred,
                target: AddonRoutingPlanTarget::None,
                safe_reason_code: Some("missing_grant".to_owned()),
                job_kind: None,
                event_kind: Some(DomainEventKind::ItemMetadataRefreshed.as_str().to_owned()),
                plan_json: r#"{"schema":"nako.addon.routing_plan.v1","declaration_id":"metadata-refreshed","reason":"missing_grant"}"#.to_owned(),
            }],
        )
        .await
        .unwrap();

    assert_eq!(replaced.len(), 1);
    assert_eq!(replaced[0].declaration_id, "metadata-refreshed");
    assert_eq!(replaced[0].manifest_fingerprint, next_fingerprint);
    assert_eq!(
        replaced[0].safe_reason_code.as_deref(),
        Some("missing_grant")
    );
    assert!(
        store
            .list_addon_routing_plans(addon_id)
            .await
            .unwrap()
            .iter()
            .all(|plan| plan.declaration_id != "bulk-refresh")
    );

    assert!(
        store
            .list_jobs(JobListFilter::default(), PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        store
            .list_outbox_events(OutboxEventListFilter::default(), PageRequest::first_page())
            .await
            .unwrap()
            .is_empty()
    );

    let mismatched = store
        .replace_addon_routing_plans(
            addon_id,
            vec![NewAddonRoutingPlan {
                id: AddonRoutingPlanId::new(),
                addon_id: AddonId::new(),
                manifest_id: "dev.nako.contract.routing".to_owned(),
                manifest_version: "0.2.0".to_owned(),
                manifest_fingerprint: next_fingerprint,
                declaration_kind: AddonRoutingDeclarationKind::Task,
                declaration_id: "bad".to_owned(),
                status: AddonRoutingPlanStatus::Deferred,
                target: AddonRoutingPlanTarget::None,
                safe_reason_code: Some("missing_grant".to_owned()),
                job_kind: None,
                event_kind: None,
                plan_json: "{}".to_owned(),
            }],
        )
        .await;
    assert!(matches!(mismatched, Err(NakoError::InvalidInput { .. })));
    assert_eq!(
        store
            .list_addon_routing_plans(addon_id)
            .await
            .unwrap()
            .len(),
        1
    );

    let _ = library;
}

async fn addon_task_run_idempotency_fingerprint_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Addon Task Contract Movie",
        "local:///Contract Movies/addon-task-contract.mkv",
    )
    .await;
    let addon_id = AddonId::new();
    let manifest_id = "dev.nako.contract.addon-task";
    let manifest_version = "0.1.0";
    let manifest_json = r#"{"id":"dev.nako.contract.addon-task","version":"0.1.0"}"#;
    let manifest_fingerprint = AddonManifestFingerprint::new(manifest_json);
    let declaration_id = "bulk-refresh";
    let declaration_path = "/tasks/bulk-refresh";
    let idempotency_key = "addon-task-contract-key";
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: manifest_id.to_owned(),
            name: "Contract Addon Task".to_owned(),
            version: manifest_version.to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "https://example.test/addon-task".to_owned(),
            manifest_json: manifest_json.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["automation_run".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();

    let input_json = serde_json::json!({
        "schema": "nako.addon.task_run.input.v1",
        "library_id": library.id,
        "source_id": source.id,
        "payload": {
            "mode": "missing_only"
        },
        "idempotency_key": idempotency_key
    })
    .to_string();
    let first = contract_addon_task_run(
        addon_id,
        manifest_id,
        manifest_version,
        &manifest_fingerprint,
        declaration_id,
        declaration_path,
        idempotency_key,
        library.id,
        source.id,
        &input_json,
    );
    let created = store
        .create_addon_task_run(first.0, first.1.clone())
        .await
        .unwrap();
    assert!(!created.idempotent_replay);
    assert_eq!(created.run.addon_id, addon_id);
    assert_eq!(created.run.idempotency_key, idempotency_key);
    assert_eq!(created.run.request_fingerprint, first.1.request_fingerprint);
    assert_eq!(created.run.input_json, input_json);

    let replay = contract_addon_task_run(
        addon_id,
        manifest_id,
        manifest_version,
        &manifest_fingerprint,
        declaration_id,
        declaration_path,
        idempotency_key,
        library.id,
        source.id,
        &input_json,
    );
    let replayed = store
        .create_addon_task_run(replay.0, replay.1)
        .await
        .unwrap();
    assert!(replayed.idempotent_replay);
    assert_eq!(replayed.run.job.id, created.run.job.id);
    assert_eq!(
        replayed.run.request_fingerprint,
        created.run.request_fingerprint
    );

    let conflicting_input_json = serde_json::json!({
        "schema": "nako.addon.task_run.input.v1",
        "library_id": library.id,
        "source_id": source.id,
        "payload": {
            "mode": "force"
        },
        "idempotency_key": idempotency_key
    })
    .to_string();
    let conflict = contract_addon_task_run(
        addon_id,
        manifest_id,
        manifest_version,
        &manifest_fingerprint,
        declaration_id,
        declaration_path,
        idempotency_key,
        library.id,
        source.id,
        &conflicting_input_json,
    );
    let err = store
        .create_addon_task_run(conflict.0, conflict.1)
        .await
        .unwrap_err();
    assert!(matches!(err, NakoError::Conflict { .. }));
    assert!(err.to_string().contains("different request"));
    assert_eq!(
        store
            .list_addon_task_runs(
                AddonTaskRunListFilter {
                    addon_id: Some(addon_id),
                    ..AddonTaskRunListFilter::default()
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .len(),
        1
    );
}

fn contract_addon_task_run(
    addon_id: AddonId,
    manifest_id: &str,
    manifest_version: &str,
    manifest_fingerprint: &AddonManifestFingerprint,
    declaration_id: &str,
    declaration_path: &str,
    idempotency_key: &str,
    library_id: LibraryId,
    source_id: MediaSourceId,
    input_json: &str,
) -> (NewJob, NewAddonTaskRun) {
    let job_id = JobId::new();
    let request_fingerprint = AddonTaskRunRequestFingerprint::new(
        manifest_id,
        manifest_version,
        manifest_fingerprint,
        declaration_id,
        declaration_path,
        input_json,
    );

    (
        NewJob {
            id: job_id,
            kind: JobKind::AddonTask,
            resource_class: "addon.task.bulk-refresh".to_owned(),
            priority: JobPriority::Normal,
            library_id: Some(library_id),
            source_id: Some(source_id),
            input_json: Some(input_json.to_owned()),
        },
        NewAddonTaskRun {
            job_id,
            addon_id,
            manifest_id: manifest_id.to_owned(),
            manifest_version: manifest_version.to_owned(),
            manifest_fingerprint: manifest_fingerprint.clone(),
            declaration_id: declaration_id.to_owned(),
            declaration_name: "Bulk refresh".to_owned(),
            declaration_path: declaration_path.to_owned(),
            idempotency_key: idempotency_key.to_owned(),
            request_fingerprint,
            attempt: 1,
            max_attempts: Some(3),
            retry_of_job_id: None,
            input_json: input_json.to_owned(),
        },
    )
}

async fn automation_provider_and_artifact_contract<S>(store: S)
where
    S: EventAddonAutomationContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Automation Contract Movie",
        "local:///Contract Movies/automation-contract.mkv",
    )
    .await;
    let job = enqueue_contract_job(
        &store,
        JobKind::Automation,
        "automation.external_api",
        Some(library.id),
        Some(r#"{"capability":"summary"}"#),
    )
    .await;

    let provider_id = nako_core::AutomationProviderId::new();
    let provider = store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Contract AI".to_owned(),
            base_url: "https://automation.example.test".to_owned(),
            secret_env: Some("NAKO_AUTOMATION_TOKEN".to_owned()),
            capabilities: vec![
                AutomationCapability::Summary,
                AutomationCapability::TitleMatch,
            ],
            timeout_ms: 2_500,
            max_attempts: 2,
            status: AutomationProviderStatus::Enabled,
        })
        .await
        .unwrap();
    assert_eq!(provider.id, provider_id);
    assert_eq!(
        provider.secret_env.as_deref(),
        Some("NAKO_AUTOMATION_TOKEN")
    );
    assert_eq!(
        provider.capabilities,
        vec![
            AutomationCapability::Summary,
            AutomationCapability::TitleMatch
        ]
    );

    let disabled = store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: nako_core::AutomationProviderId::new(),
            name: "Disabled AI".to_owned(),
            base_url: "https://disabled.example.test".to_owned(),
            secret_env: None,
            capabilities: vec![AutomationCapability::Recommendation],
            timeout_ms: 1_000,
            max_attempts: 1,
            status: AutomationProviderStatus::Disabled,
        })
        .await
        .unwrap();

    let enabled = store.list_enabled_automation_providers().await.unwrap();
    assert_eq!(
        enabled
            .iter()
            .map(|provider| provider.id)
            .collect::<Vec<_>>(),
        vec![provider_id]
    );
    assert!(!enabled.iter().any(|provider| provider.id == disabled.id));

    let artifact = store
        .create_automation_artifact(NewAutomationArtifact {
            id: nako_core::AutomationArtifactId::new(),
            job_id: job.id,
            provider_id,
            capability: AutomationCapability::Summary,
            kind: AutomationArtifactKind::Summary,
            library_id: Some(library.id),
            item_id: Some(source.item_id),
            source_id: Some(source.id),
            artifact_json: r#"{"summary":"A contract artifact."}"#.to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(artifact.status, AutomationArtifactStatus::Proposed);
    assert_eq!(artifact.accepted_at, None);

    let accepted = store
        .set_automation_artifact_status(artifact.id, AutomationArtifactStatus::Accepted)
        .await
        .unwrap();
    assert_eq!(accepted.status, AutomationArtifactStatus::Accepted);
    assert!(accepted.accepted_at.is_some());

    let job_artifacts = store
        .list_automation_artifacts_for_job(job.id)
        .await
        .unwrap();
    assert_eq!(job_artifacts.len(), 1);
    assert_eq!(job_artifacts[0].id, artifact.id);

    let item_artifacts = store
        .list_automation_artifacts_for_item(source.item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(item_artifacts.len(), 1);
    assert_eq!(item_artifacts[0].id, artifact.id);

    let rejected = store
        .set_automation_artifact_status(artifact.id, AutomationArtifactStatus::Rejected)
        .await
        .unwrap();
    assert_eq!(rejected.status, AutomationArtifactStatus::Rejected);
    assert_eq!(rejected.accepted_at, None);
}

async fn seed_managed_artwork_contract_item<S>(store: &S) -> (LibraryId, MediaItemId)
where
    S: LibraryRepository + LibraryItemRepository + MediaRepository + ?Sized,
{
    let library = seed_contract_library(store).await;
    let item = contract_media_item(MediaItemId::new(), "Managed Artwork Contract");

    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id: library.id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();

    (library.id, item.id)
}

async fn seed_managed_artwork_addon_side_effect<S>(
    store: &S,
    library_id: LibraryId,
    item_id: MediaItemId,
    idempotency_key: &str,
) -> (AddonId, AddonSideEffectId)
where
    S: AddonRepository + ?Sized,
{
    let addon_id = AddonId::new();
    let token_id = AddonTokenId::new();
    store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: format!("dev.nako.contract.{idempotency_key}"),
            name: "Managed Artwork Contract Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "0.1.0-alpha.1".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: r#"{"id":"dev.nako.contract.managed-artwork"}"#.to_owned(),
            outbound_task_dispatch_secret_env: None,
            granted_scopes: vec!["artwork_write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    store
        .create_addon_token(NewAddonToken {
            id: token_id,
            addon_id,
            label: "managed artwork contract".to_owned(),
            token_prefix: "nako_at_managed_artwork".to_owned(),
            token_hash: format!("sha256:{idempotency_key}"),
        })
        .await
        .unwrap();
    let side_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: AddonSideEffectId::new(),
            addon_id,
            token_id,
            permission: AddonPermission::ArtworkWrite,
            library_id,
            target: AddonSideEffectTarget::media_item(item_id),
            idempotency_key: idempotency_key.to_owned(),
            provenance_json: r#"{"origin":"contract"}"#.to_owned(),
            payload_json: r#"{"intent":"propose_artwork"}"#.to_owned(),
            validation_status: AddonSideEffectValidationStatus::Accepted,
            safe_error_code: None,
        })
        .await
        .unwrap();

    (addon_id, side_effect.id)
}

async fn addon_artwork_candidate_intake_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (library_id, item_id) = seed_managed_artwork_contract_item(&store).await;
    let (addon_id, side_effect_id) = seed_managed_artwork_addon_side_effect(
        &store,
        library_id,
        item_id,
        "managed-artwork-candidate-intake",
    )
    .await;
    let source_uri = "https://cdn.example.test/managed-artwork/poster.png";

    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id,
            library_id,
            item_id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: source_uri.to_owned(),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(candidate.addon_id, addon_id);
    assert_eq!(candidate.side_effect_id, side_effect_id);
    assert_eq!(candidate.library_id, library_id);
    assert_eq!(candidate.item_id, item_id);
    assert_eq!(candidate.kind, ImageKind::Poster);
    assert_eq!(candidate.source_kind, ArtworkCandidateSourceKind::RemoteUrl);
    assert_eq!(candidate.source_uri, source_uri);
    assert_eq!(candidate.width, Some(1000));
    assert_eq!(candidate.height, Some(1500));
    assert_eq!(candidate.language.as_deref(), Some("en"));
    assert_eq!(candidate.status, ArtworkCandidateStatus::Proposed);
    assert!(!candidate.created_at.is_empty());
    assert!(!candidate.updated_at.is_empty());

    let duplicate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id,
            library_id,
            item_id,
            kind: ImageKind::Poster,
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: source_uri.to_owned(),
            width: Some(999),
            height: Some(1499),
            language: Some("fr".to_owned()),
        })
        .await
        .unwrap();

    assert_eq!(duplicate, candidate);
    assert_eq!(
        store.get_artwork_candidate(candidate.id).await.unwrap(),
        Some(candidate.clone())
    );
    assert_eq!(
        store
            .find_artwork_candidate_by_source(
                addon_id,
                library_id,
                item_id,
                &ImageKind::Poster,
                ArtworkCandidateSourceKind::RemoteUrl,
                source_uri,
            )
            .await
            .unwrap(),
        Some(candidate.clone())
    );
    assert_eq!(
        store
            .list_artwork_candidates_for_item(item_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![candidate.clone()]
    );

    let rejected = store
        .set_artwork_candidate_status(candidate.id, ArtworkCandidateStatus::Rejected)
        .await
        .unwrap();
    assert_eq!(rejected.status, ArtworkCandidateStatus::Rejected);
    assert_eq!(
        store.get_artwork_candidate(candidate.id).await.unwrap(),
        Some(rejected)
    );
}

async fn artwork_task_queue_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (_library_id, item_id) = seed_managed_artwork_contract_item(&store).await;
    let image = ImageAsset {
        id: ImageAssetId::new(),
        owner: ImageOwner::Item(item_id),
        kind: ImageKind::Thumbnail,
        source_uri: "local:///Contract Movies/Managed Artwork Contract.mkv#preview=10".to_owned(),
        provider: ExternalProvider::Local,
        cache_uri: None,
        width: Some(320),
        height: Some(180),
        language: None,
        selected: false,
        content_hash: None,
        etag: Some("preview-etag".to_owned()),
    };
    store.upsert_image_asset(&image).await.unwrap();

    let queued = ArtworkTask {
        id: ArtworkTaskId::new(),
        image_id: image.id,
        kind: ArtworkTaskKind::Preview,
        status: JobStatus::Queued,
        resource_class: ArtworkTaskKind::Preview.resource_class().to_owned(),
        attempts: 0,
        max_attempts: 3,
        error: None,
    };
    store.enqueue_artwork_task(&queued).await.unwrap();
    assert_eq!(
        store.get_artwork_task(queued.id).await.unwrap(),
        Some(queued.clone())
    );
    assert_eq!(
        store
            .list_artwork_tasks(PageRequest::first_page())
            .await
            .unwrap(),
        vec![queued.clone()]
    );

    let running = ArtworkTask {
        status: JobStatus::Running,
        attempts: 1,
        error: Some("retrying after transient image decode failure".to_owned()),
        ..queued
    };
    store.enqueue_artwork_task(&running).await.unwrap();

    assert_eq!(
        store.get_artwork_task(running.id).await.unwrap(),
        Some(running.clone())
    );
    assert_eq!(
        store
            .list_artwork_tasks(PageRequest::first_page())
            .await
            .unwrap(),
        vec![running]
    );
}

async fn seed_accepted_managed_artwork_ingest<S>(
    store: &S,
    idempotency_key: &str,
    kind: ImageKind,
    source_uri: &str,
) -> (
    LibraryId,
    MediaItemId,
    ArtworkCandidateRecord,
    ManagedArtworkAcceptanceRecord,
)
where
    S: ManagedArtworkContractBackend,
{
    let (library_id, item_id) = seed_managed_artwork_contract_item(store).await;
    let (candidate, accepted) = seed_accepted_managed_artwork_ingest_for_item(
        store,
        library_id,
        item_id,
        idempotency_key,
        kind,
        source_uri,
    )
    .await;

    (library_id, item_id, candidate, accepted)
}

async fn seed_accepted_managed_artwork_ingest_for_item<S>(
    store: &S,
    library_id: LibraryId,
    item_id: MediaItemId,
    idempotency_key: &str,
    kind: ImageKind,
    source_uri: &str,
) -> (ArtworkCandidateRecord, ManagedArtworkAcceptanceRecord)
where
    S: ManagedArtworkContractBackend,
{
    let (addon_id, side_effect_id) =
        seed_managed_artwork_addon_side_effect(store, library_id, item_id, idempotency_key).await;
    let candidate = store
        .create_artwork_candidate(NewArtworkCandidate {
            id: ArtworkCandidateId::new(),
            addon_id,
            side_effect_id,
            library_id,
            item_id,
            kind: kind.clone(),
            source_kind: ArtworkCandidateSourceKind::RemoteUrl,
            source_uri: source_uri.to_owned(),
            width: Some(1000),
            height: Some(1500),
            language: Some("en".to_owned()),
        })
        .await
        .unwrap();
    let ingest_id = ManagedArtworkIngestId::new();
    let job_id = JobId::new();

    let accepted = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ingest_id,
                candidate_id: candidate.id,
                job_id,
                library_id,
                item_id,
                kind: kind.clone(),
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: job_id,
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                priority: JobPriority::Normal,
                library_id: Some(library_id),
                source_id: None,
                input_json: Some(
                    serde_json::json!({
                        "candidate_id": candidate.id,
                        "library_id": library_id,
                        "item_id": item_id,
                        "image_kind": image_kind_contract_label(&kind)
                    })
                    .to_string(),
                ),
            },
        )
        .await
        .unwrap();

    (candidate, accepted)
}

async fn seed_published_selected_artwork_for_item<S>(
    store: &S,
    library_id: LibraryId,
    item_id: MediaItemId,
    idempotency_key: &str,
    kind: ImageKind,
    source_uri: &str,
) -> SelectedArtworkRecord
where
    S: ManagedArtworkContractBackend,
{
    let (_, accepted) = seed_accepted_managed_artwork_ingest_for_item(
        store,
        library_id,
        item_id,
        idempotency_key,
        kind.clone(),
        source_uri,
    )
    .await;
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claim.ingest.id, accepted.ingest.id);
    let artifact_id = ManagedArtworkArtifactId::new();
    store
        .commit_managed_artwork_artifact(
            claim.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id,
                kind,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some(format!("sha256-{artifact_id}")),
                width: Some(2),
                height: Some(3),
                byte_len: Some(64),
                media_type: Some("image/jpeg".to_owned()),
            },
            Some(r#"{"status":"stored"}"#.to_owned()),
        )
        .await
        .unwrap();
    store
        .publish_selected_artwork(artifact_id)
        .await
        .unwrap()
        .selected_artwork
}

fn image_kind_contract_label(kind: &ImageKind) -> &'static str {
    match kind {
        ImageKind::Poster => "poster",
        ImageKind::Backdrop => "backdrop",
        ImageKind::Logo => "logo",
        ImageKind::Thumbnail => "thumbnail",
        ImageKind::Banner => "banner",
        ImageKind::Other(_) => "other",
    }
}

async fn managed_artwork_acceptance_creates_ingest_job_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (library_id, item_id, candidate, accepted) = seed_accepted_managed_artwork_ingest(
        &store,
        "managed-artwork-acceptance",
        ImageKind::Poster,
        "https://cdn.example.test/poster.jpg?token=secret",
    )
    .await;
    let ingest_id = accepted.ingest.id;
    let job_id = accepted.job.id;

    assert_eq!(accepted.candidate.id, candidate.id);
    assert_eq!(accepted.candidate.status, ArtworkCandidateStatus::Accepted);
    assert_eq!(accepted.ingest.id, ingest_id);
    assert_eq!(accepted.ingest.candidate_id, candidate.id);
    assert_eq!(accepted.ingest.job_id, job_id);
    assert_eq!(accepted.ingest.library_id, library_id);
    assert_eq!(accepted.ingest.item_id, item_id);
    assert_eq!(accepted.ingest.kind, ImageKind::Poster);
    assert_eq!(accepted.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(accepted.ingest.artifact_id, None);
    assert_eq!(accepted.ingest.failure_code, None);
    assert_eq!(accepted.job.id, job_id);
    assert_eq!(accepted.job.kind, JobKind::ManagedArtworkIngest);
    assert_eq!(accepted.job.status, JobStatus::Queued);
    assert_eq!(accepted.job.resource_class, "artwork.ingest");
    assert!(
        !accepted
            .job
            .input_json
            .as_ref()
            .unwrap()
            .contains("token=secret")
    );
    assert_eq!(
        store
            .find_managed_artwork_ingest_by_candidate(candidate.id)
            .await
            .unwrap(),
        Some(accepted.ingest.clone())
    );
    assert_eq!(
        store.get_managed_artwork_ingest(ingest_id).await.unwrap(),
        Some(accepted.ingest.clone())
    );
    assert_eq!(
        store.get_job(job_id).await.unwrap(),
        Some(accepted.job.clone())
    );

    let replay = store
        .accept_managed_artwork_candidate_ingest(
            candidate.id,
            NewManagedArtworkIngest {
                id: ManagedArtworkIngestId::new(),
                candidate_id: candidate.id,
                job_id: JobId::new(),
                library_id,
                item_id,
                kind: ImageKind::Poster,
                status: ManagedArtworkIngestStatus::Queued,
                artifact_id: None,
                failure_code: None,
            },
            NewJob {
                id: JobId::new(),
                kind: JobKind::ManagedArtworkIngest,
                resource_class: "artwork.ingest".to_owned(),
                priority: JobPriority::Normal,
                library_id: Some(library_id),
                source_id: None,
                input_json: Some("{}".to_owned()),
            },
        )
        .await
        .unwrap();

    assert_eq!(replay.ingest.id, accepted.ingest.id);
    assert_eq!(replay.job.id, accepted.job.id);
    assert_eq!(replay.candidate.status, ArtworkCandidateStatus::Accepted);
}

async fn managed_artwork_ingest_processing_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (library_id, item_id, candidate, accepted) = seed_accepted_managed_artwork_ingest(
        &store,
        "managed-artwork-processing",
        ImageKind::Poster,
        "https://cdn.example.test/processing.jpg",
    )
    .await;

    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(claim.candidate.id, candidate.id);
    assert_eq!(claim.ingest.id, accepted.ingest.id);
    assert_eq!(claim.ingest.status, ManagedArtworkIngestStatus::Fetching);
    assert_eq!(claim.job.id, accepted.job.id);
    assert_eq!(claim.job.status, JobStatus::Running);
    assert!(
        store
            .claim_next_queued_managed_artwork_ingest()
            .await
            .unwrap()
            .is_none()
    );

    let artifact_id = ManagedArtworkArtifactId::new();
    let committed = store
        .commit_managed_artwork_artifact(
            claim.ingest.id,
            NewManagedArtworkArtifact {
                id: artifact_id,
                ingest_id: claim.ingest.id,
                library_id,
                item_id,
                kind: ImageKind::Poster,
                storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                content_hash: Some("sha256-processing".to_owned()),
                width: Some(1),
                height: Some(1),
                byte_len: Some(68),
                media_type: Some("image/png".to_owned()),
            },
            Some(r#"{"status":"stored","byte_len":68}"#.to_owned()),
        )
        .await
        .unwrap();

    assert_eq!(committed.ingest.status, ManagedArtworkIngestStatus::Stored);
    assert_eq!(committed.ingest.artifact_id, Some(artifact_id));
    assert_eq!(committed.artifact.as_ref().unwrap().id, artifact_id);
    assert_eq!(committed.job.status, JobStatus::Succeeded);
    assert_eq!(
        store
            .get_managed_artwork_artifact(artifact_id)
            .await
            .unwrap()
            .unwrap()
            .ingest_id,
        claim.ingest.id
    );
}

async fn managed_artwork_failure_recovery_requeue_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (_, _, _, accepted) = seed_accepted_managed_artwork_ingest(
        &store,
        "managed-artwork-requeue",
        ImageKind::Poster,
        "https://cdn.example.test/requeue.jpg",
    )
    .await;
    let claim = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    let failed = store
        .fail_managed_artwork_ingest(
            claim.ingest.id,
            "fetch_failed".to_owned(),
            "safe fetch failure".to_owned(),
            Some(r#"{"status":"failed"}"#.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(failed.ingest.status, ManagedArtworkIngestStatus::Failed);
    assert_eq!(failed.ingest.failure_code.as_deref(), Some("fetch_failed"));
    assert_eq!(failed.job.status, JobStatus::Failed);

    let requeued = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap();
    assert_eq!(requeued.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert_eq!(requeued.job.status, JobStatus::Queued);
    assert!(requeued.requeued);
    assert!(requeued.had_failure);

    let replay = store
        .requeue_managed_artwork_ingest(accepted.ingest.id)
        .await
        .unwrap();
    assert_eq!(replay.ingest.status, ManagedArtworkIngestStatus::Queued);
    assert!(!replay.requeued);
    assert!(!replay.had_failure);

    let refetched = store
        .claim_next_queued_managed_artwork_ingest()
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        refetched.ingest.status,
        ManagedArtworkIngestStatus::Fetching
    );
    let recovered = store
        .fail_unfinished_managed_artwork_ingests(
            "startup_recovery".to_owned(),
            "managed artwork ingest was unfinished during server startup".to_owned(),
            Some(r#"{"status":"failed"}"#.to_owned()),
        )
        .await
        .unwrap();
    assert_eq!(recovered, 1);
    assert_eq!(
        store
            .get_managed_artwork_ingest(refetched.ingest.id)
            .await
            .unwrap()
            .unwrap()
            .status,
        ManagedArtworkIngestStatus::Failed
    );
}

async fn selected_artwork_gallery_lifecycle_contract<S>(store: S)
where
    S: ManagedArtworkContractBackend,
{
    let (library_id, item_id, first_candidate, first_accepted) =
        seed_accepted_managed_artwork_ingest(
            &store,
            "managed-artwork-selected-first",
            ImageKind::Poster,
            "https://cdn.example.test/selected-first.jpg",
        )
        .await;
    let (_, second_accepted) = seed_accepted_managed_artwork_ingest_for_item(
        &store,
        library_id,
        item_id,
        "managed-artwork-selected-second",
        ImageKind::Backdrop,
        "https://cdn.example.test/selected-second.jpg",
    )
    .await;

    let mut artifact_ids = Vec::new();
    for (accepted, kind, byte_len) in [
        (first_accepted, ImageKind::Poster, 68_u64),
        (second_accepted, ImageKind::Backdrop, 102_u64),
    ] {
        let claim = store
            .claim_next_queued_managed_artwork_ingest()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(claim.ingest.id, accepted.ingest.id);
        let artifact_id = ManagedArtworkArtifactId::new();
        store
            .commit_managed_artwork_artifact(
                claim.ingest.id,
                NewManagedArtworkArtifact {
                    id: artifact_id,
                    ingest_id: claim.ingest.id,
                    library_id,
                    item_id,
                    kind,
                    storage_uri: format!("managed-artwork://artifact/{artifact_id}"),
                    content_hash: Some(format!("sha256-{artifact_id}")),
                    width: Some(1),
                    height: Some(1),
                    byte_len: Some(byte_len),
                    media_type: Some("image/png".to_owned()),
                },
                Some(r#"{"status":"stored"}"#.to_owned()),
            )
            .await
            .unwrap();
        artifact_ids.push(artifact_id);
    }

    let published = store
        .publish_selected_artwork(artifact_ids[0])
        .await
        .unwrap();
    assert_eq!(published.selected_artwork.item_id, item_id);
    assert_eq!(published.selected_artwork.kind, ImageKind::Poster);
    assert_eq!(published.selected_artwork.artifact_id, artifact_ids[0]);
    assert!(published.changed);
    let replay = store
        .publish_selected_artwork(artifact_ids[0])
        .await
        .unwrap();
    assert_eq!(replay.selected_artwork.id, published.selected_artwork.id);
    assert!(!replay.changed);
    assert_eq!(
        store.list_selected_artwork_for_item(item_id).await.unwrap(),
        vec![published.selected_artwork.clone()]
    );

    let gallery = store
        .get_managed_artwork_gallery_for_item(item_id, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(gallery.summary.candidates, 2);
    assert_eq!(gallery.summary.artifacts, 2);
    assert_eq!(gallery.summary.selected, 1);
    assert!(
        gallery
            .candidates
            .iter()
            .any(|candidate| candidate.id == first_candidate.id && candidate.selected())
    );
    assert!(
        !serde_json::to_string(&nako_api_safety_projection(&gallery))
            .unwrap()
            .contains("managed-artwork://")
    );

    let lifecycle = store
        .list_managed_artwork_artifact_lifecycle(
            ManagedArtworkArtifactLifecycleFilter::All,
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(lifecycle.summary.total_artifacts, 2);
    assert_eq!(lifecycle.summary.protected_artifacts, 1);
    assert_eq!(lifecycle.summary.cleanup_candidate_artifacts, 1);
    assert_eq!(lifecycle.summary.known_total_bytes, 170);

    let cleanup = store
        .cleanup_unselected_managed_artwork_artifacts(PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(cleanup.cleaned_artifacts.len(), 1);
    assert_eq!(cleanup.cleaned_artifacts[0].id, artifact_ids[1]);
    assert!(
        store
            .get_managed_artwork_artifact(artifact_ids[0])
            .await
            .unwrap()
            .is_some()
    );
    assert_eq!(
        store
            .get_managed_artwork_artifact(artifact_ids[1])
            .await
            .unwrap(),
        None
    );
}

fn nako_api_safety_projection(
    gallery: &nako_core::ManagedArtworkGallerySnapshot,
) -> (u32, u32, u32) {
    (
        gallery.summary.candidates,
        gallery.summary.artifacts,
        gallery.summary.selected,
    )
}

async fn runtime_promotion_contract_covers_facade_dispatch_gap_surfaces<S>(store: S)
where
    S: RuntimePromotionContractBackend,
{
    let library = seed_contract_library(&store).await;
    let first_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Uncertain Movie",
        "local:///Contract/uncertain-a.mkv",
    )
    .await;
    let second_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Uncertain Movie Duplicate",
        "local:///Contract/uncertain-b.mkv",
    )
    .await;
    let evidence = LocalInferenceEvidence {
        id: LocalInferenceEvidenceId::new(),
        source_id: first_source.id,
        inferred_kind: MediaKind::Movie,
        inferred_title: Some("Uncertain Movie".to_owned()),
        inferred_year: Some(2026),
        inferred_season: None,
        inferred_episode: None,
        confidence_milli: Some(650),
        evidence_source: LocalInferenceEvidenceSource::FileName,
        evidence_value: "uncertain-a.mkv".to_owned(),
        inference_version: "runtime-promotion".to_owned(),
    };
    let subject = ProviderSubject {
        id: ProviderSubjectId::new(),
        provider: ExternalProvider::Tmdb,
        subject_kind: ProviderSubjectKind::Movie,
        subject_key: "650".to_owned(),
        title: Some("Uncertain Movie".to_owned()),
        release_year: Some(2026),
        locale: Some("en-US".to_owned()),
    };
    let mapping = ProviderMapping {
        id: ProviderMappingId::new(),
        item_id: first_source.item_id,
        subject_id: subject.id,
        status: ProviderMappingStatus::Candidate,
        confidence_milli: Some(650),
        source: MetadataSource::Provider(ExternalProvider::Tmdb),
    };
    let duplicate = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: second_source.id,
        duplicate_source_id: first_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
        evidence_value: Some("sha256:contract".to_owned()),
        status: SourceDuplicateRelationshipStatus::Suggested,
        confidence_milli: Some(990),
    };
    let expected_duplicate = duplicate.canonicalized();

    store
        .upsert_local_inference_evidence(&evidence)
        .await
        .unwrap();
    store.upsert_provider_subject(&subject).await.unwrap();
    store.upsert_provider_mapping(&mapping).await.unwrap();
    store
        .upsert_source_duplicate_relationship(&duplicate)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_source_duplicate_relationship(duplicate.id)
            .await
            .unwrap(),
        Some(expected_duplicate.clone())
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(first_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected_duplicate.clone()]
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(second_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected_duplicate]
    );

    let governance = store
        .list_catalog_governance_items(
            CatalogGovernanceItemListFilter {
                library_id: Some(library.id),
                max_confidence_milli: 700,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    let governed = governance
        .iter()
        .find(|record| record.item.id == first_source.item_id)
        .expect("governance should include low-confidence inferred item");

    assert_eq!(governed.library_id, library.id);
    assert_eq!(governed.source_count, 1);
    assert_eq!(governed.representative_source_id, Some(first_source.id));
    assert_eq!(
        governed.representative_file_name.as_deref(),
        Some("uncertain-a.mkv")
    );
    assert_eq!(governed.provider_mapping_count, 1);
    assert_eq!(governed.accepted_provider_mapping_count, 0);
    assert_eq!(governed.duplicate_relationship_count, 1);
    assert_eq!(governed.best_local_inference, Some(evidence));

    let duplicate_only = governance
        .iter()
        .find(|record| record.item.id == second_source.item_id)
        .expect("governance should include high-confidence duplicate-only item");

    assert_eq!(duplicate_only.library_id, library.id);
    assert_eq!(duplicate_only.source_count, 1);
    assert_eq!(
        duplicate_only.representative_source_id,
        Some(second_source.id)
    );
    assert_eq!(duplicate_only.provider_mapping_count, 0);
    assert_eq!(duplicate_only.accepted_provider_mapping_count, 0);
    assert_eq!(duplicate_only.duplicate_relationship_count, 1);
    assert_eq!(duplicate_only.best_local_inference, None);
}

async fn source_duplicate_contract_upsert_is_idempotent_by_canonical_pair<S>(store: S)
where
    S: SourceDuplicateContractBackend,
{
    let library = seed_contract_library(&store).await;
    let first_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Pair A",
        "local:///Contract/duplicate-pair-a.mkv",
    )
    .await;
    let second_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Pair B",
        "local:///Contract/duplicate-pair-b.mkv",
    )
    .await;

    let initial = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: second_source.id,
        duplicate_source_id: first_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::SizeAndEtag,
        evidence_value: Some("size=19;etag=weak-contract".to_owned()),
        status: SourceDuplicateRelationshipStatus::Suggested,
        confidence_milli: Some(720),
    };
    let replacement = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: first_source.id,
        duplicate_source_id: second_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
        evidence_value: Some("sha256:source-duplicate-contract".to_owned()),
        status: SourceDuplicateRelationshipStatus::Confirmed,
        confidence_milli: Some(995),
    };
    let mut expected = replacement.canonicalized();
    expected.id = initial.id;

    store
        .upsert_source_duplicate_relationship(&initial)
        .await
        .unwrap();
    store
        .upsert_source_duplicate_relationship(&replacement)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_source_duplicate_relationship(initial.id)
            .await
            .unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        store
            .get_source_duplicate_relationship(replacement.id)
            .await
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(first_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected.clone()]
    );
    assert_eq!(
        store
            .list_source_duplicate_relationships(second_source.id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![expected]
    );
}

async fn source_duplicate_contract_lists_fingerprint_matches_and_pair_lookup<S>(store: S)
where
    S: SourceDuplicateContractBackend,
{
    let library = seed_contract_library(&store).await;
    let other_library = seed_contract_library(&store).await;
    let fingerprint = "source:v1:content_hash:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let mut target_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Plan Target",
        "local:///Contract/duplicate-plan-target.mkv",
    )
    .await;
    let mut stale_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Plan Stale Candidate",
        "local:///Contract/duplicate-plan-stale.mkv",
    )
    .await;
    let mut fresh_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Plan Fresh Candidate",
        "local:///Contract/duplicate-plan-fresh.mkv",
    )
    .await;
    let mut another_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Plan Another Candidate",
        "local:///Contract/duplicate-plan-another.mkv",
    )
    .await;
    let mut other_library_source = seed_contract_media_item_with_source(
        &store,
        other_library.id,
        "Duplicate Plan Other Library",
        "local:///Contract/duplicate-plan-other-library.mkv",
    )
    .await;
    let other_fingerprint_source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "Duplicate Plan Other Fingerprint",
        "local:///Contract/duplicate-plan-other-fingerprint.mkv",
    )
    .await;

    target_source.fingerprint = Some(fingerprint.to_owned());
    stale_source.fingerprint = Some(fingerprint.to_owned());
    fresh_source.fingerprint = Some(fingerprint.to_owned());
    another_source.fingerprint = Some(fingerprint.to_owned());
    other_library_source.fingerprint = Some(fingerprint.to_owned());

    for source in [
        &target_source,
        &stale_source,
        &fresh_source,
        &another_source,
        &other_library_source,
    ] {
        store.upsert_media_source(source).await.unwrap();
    }

    let scan_id = ScanSnapshotId::new();
    store
        .begin_scan_snapshot(scan_id, library.id, "local:///Contract")
        .await
        .unwrap();
    let mut stale_state =
        contract_source_state(library.id, stale_source.id, scan_id, &stale_source.locator);
    stale_state.fingerprint = Some(fingerprint.to_owned());
    stale_state.tombstoned = true;
    store.upsert_source_state(&stale_state).await.unwrap();
    store
        .complete_scan_snapshot(scan_id, ScanStatus::Succeeded, None)
        .await
        .unwrap();

    let first_page = store
        .list_media_sources_by_fingerprint(
            library.id,
            fingerprint,
            Some(target_source.id),
            PageRequest::new(2, 0),
        )
        .await
        .unwrap();
    let second_page = store
        .list_media_sources_by_fingerprint(
            library.id,
            fingerprint,
            Some(target_source.id),
            PageRequest::new(2, 2),
        )
        .await
        .unwrap();
    let all_matches = store
        .list_media_sources_by_fingerprint(
            library.id,
            fingerprint,
            Some(target_source.id),
            PageRequest::new(10, 0),
        )
        .await
        .unwrap();

    assert_eq!(first_page.len(), 2);
    assert_eq!(second_page.len(), 1);
    assert_eq!(all_matches.len(), 3);
    assert!(
        all_matches
            .iter()
            .all(|matched| matched.source.library_id == library.id)
    );
    assert!(
        !all_matches
            .iter()
            .any(|matched| matched.source.id == target_source.id)
    );
    assert!(
        !all_matches
            .iter()
            .any(|matched| matched.source.id == other_library_source.id)
    );
    assert!(
        !all_matches
            .iter()
            .any(|matched| matched.source.id == other_fingerprint_source.id)
    );
    assert!(
        all_matches
            .iter()
            .find(|matched| matched.source.id == stale_source.id)
            .expect("stale source match")
            .stale
    );
    assert!(
        !all_matches
            .iter()
            .find(|matched| matched.source.id == fresh_source.id)
            .expect("fresh source match")
            .stale
    );

    let relationship = SourceDuplicateRelationship {
        id: SourceDuplicateRelationshipId::new(),
        source_id: fresh_source.id,
        duplicate_source_id: stale_source.id,
        evidence_kind: SourceDuplicateEvidenceKind::StrongFingerprint,
        evidence_value: Some("redacted-contract-evidence".to_owned()),
        status: SourceDuplicateRelationshipStatus::Suggested,
        confidence_milli: Some(1_000),
    };
    let expected = relationship.canonicalized();
    store
        .upsert_source_duplicate_relationship(&relationship)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_source_duplicate_relationship_by_pair(stale_source.id, fresh_source.id)
            .await
            .unwrap(),
        Some(expected.clone())
    );
    assert_eq!(
        store
            .get_source_duplicate_relationship_by_pair(target_source.id, fresh_source.id)
            .await
            .unwrap(),
        None
    );
}

async fn vfs_cache_contract_round_trips_listing_failures_and_summary<S>(store: S)
where
    S: VfsStagingContractBackend,
{
    let directory = VfsCachedObject {
        uri: "webdav:///Contract/Movies/".to_owned(),
        scheme: "webdav".to_owned(),
        kind: VfsCachedObjectKind::Directory,
        len: None,
        modified_at: Some("2026-05-20T00:00:00.000Z".to_owned()),
        etag: Some("movies".to_owned()),
        fingerprint: Some("webdav:etag=movies".to_owned()),
        capabilities_bits: 0b111,
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };
    let movie = VfsCachedObject {
        uri: "webdav:///Contract/Movies/Demo.mkv".to_owned(),
        scheme: "webdav".to_owned(),
        kind: VfsCachedObjectKind::File,
        len: Some(4),
        modified_at: Some("2026-05-20T00:00:01.000Z".to_owned()),
        etag: Some("demo".to_owned()),
        fingerprint: Some("webdav:etag=demo".to_owned()),
        capabilities_bits: 0b101,
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };
    let subtitle = VfsCachedObject {
        uri: "webdav:///Contract/Movies/Demo.zh.srt".to_owned(),
        scheme: "webdav".to_owned(),
        kind: VfsCachedObjectKind::File,
        len: Some(2),
        modified_at: Some("2026-05-20T00:00:02.000Z".to_owned()),
        etag: Some("subtitle".to_owned()),
        fingerprint: Some("webdav:etag=subtitle".to_owned()),
        capabilities_bits: 0b001,
        fetched_at_ms: 150,
        fresh_until_ms: 500,
    };
    let listing = VfsCachedListing {
        directory: directory.clone(),
        entries: vec![movie.clone(), subtitle.clone()],
        fetched_at_ms: 100,
        fresh_until_ms: 200,
    };

    store.upsert_vfs_cache_listing(&listing).await.unwrap();

    assert_eq!(
        store
            .get_vfs_cache_object("webdav:///Contract/Movies/Demo.mkv")
            .await
            .unwrap(),
        Some(movie.clone())
    );
    assert_eq!(
        store
            .get_vfs_cache_listing("webdav:///Contract/Movies/")
            .await
            .unwrap(),
        Some(listing.clone())
    );

    let replacement_listing = VfsCachedListing {
        directory: VfsCachedObject {
            fresh_until_ms: 800,
            ..directory
        },
        entries: vec![subtitle.clone()],
        fetched_at_ms: 300,
        fresh_until_ms: 800,
    };
    store
        .upsert_vfs_cache_listing(&replacement_listing)
        .await
        .unwrap();
    assert_eq!(
        store
            .get_vfs_cache_listing("webdav:///Contract/Movies/")
            .await
            .unwrap(),
        Some(replacement_listing)
    );

    let first_failure_library_id = LibraryId::new();
    let first_failure_backend_key = format!("library:{first_failure_library_id}:webdav");
    let second_failure_library_id = LibraryId::new();
    let second_failure_backend_key = format!("library:{second_failure_library_id}:webdav");
    let first_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Contract/Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 900,
            error: "timeout".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                first_failure_library_id,
                first_failure_backend_key,
            ),
        })
        .await
        .unwrap();
    let second_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Contract/Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 950,
            error: "rate limited".to_owned(),
            authority: VfsCacheFailureAuthority::attributed(
                second_failure_library_id,
                second_failure_backend_key.clone(),
            ),
        })
        .await
        .unwrap();
    let older_distinct_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Contract/Movies/Older.mkv".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::Stat,
            failed_at_ms: 925,
            error: "storage backend unavailable".to_owned(),
            authority: VfsCacheFailureAuthority::default(),
        })
        .await
        .unwrap();

    assert_eq!(first_failure.failure_count, 1);
    assert_eq!(
        first_failure.authority.library_id,
        Some(first_failure_library_id)
    );
    assert_eq!(second_failure.failure_count, 2);
    assert_eq!(second_failure.failed_at_ms, 950);
    assert_eq!(second_failure.error, "rate limited");
    assert_eq!(
        second_failure.authority,
        VfsCacheFailureAuthority::attributed(second_failure_library_id, second_failure_backend_key)
    );
    assert_eq!(
        store
            .get_vfs_cache_failure("webdav:///Contract/Movies/", VfsCacheOperation::List)
            .await
            .unwrap(),
        Some(second_failure.clone())
    );

    let summary = store.summarize_vfs_cache(700).await.unwrap();
    assert_eq!(summary.object_count, 3);
    assert_eq!(summary.listing_count, 1);
    assert_eq!(summary.failure_count, 2);
    assert_eq!(summary.stale_object_count, 2);
    assert_eq!(summary.stale_listing_count, 0);
    assert_eq!(summary.last_failure_at_ms, Some(950));

    assert_eq!(
        store
            .list_vfs_cache_failures(PageRequest::new(10, 0))
            .await
            .unwrap(),
        vec![second_failure.clone(), older_distinct_failure.clone()]
    );
    assert_eq!(
        store
            .list_vfs_cache_failures(PageRequest::new(1, 1))
            .await
            .unwrap(),
        vec![older_distinct_failure]
    );

    assert_eq!(
        store.get_latest_vfs_cache_failure().await.unwrap(),
        Some(second_failure)
    );
}

async fn storage_backend_health_contract_records_recovery_and_reset<S>(store: S)
where
    S: StorageBackendHealthContractBackend,
{
    let library_id = LibraryId::new();
    let backend_key = format!("library:{library_id}:webdav");
    let unhealthy = StorageBackendHealthRecord {
        backend_key: backend_key.clone(),
        library_id: Some(library_id),
        scheme: "webdav".to_owned(),
        status: StorageBackendHealthStatus::Unavailable,
        circuit_breaker_state: StorageCircuitBreakerState::Open,
        consecutive_failures: 3,
        last_success_at_ms: Some(500),
        last_failure_at_ms: Some(1_000),
        last_failure_class: Some(StorageFailureClass::Timeout),
        last_failure_safe_message: Some(StorageFailureClass::Timeout.safe_message().to_owned()),
        circuit_opened_at_ms: Some(1_000),
        backoff_until_ms: Some(2_000),
        updated_at_ms: 1_000,
    };
    let healthy_local = StorageBackendHealthRecord {
        backend_key: "library:local:default".to_owned(),
        library_id: None,
        scheme: "local".to_owned(),
        status: StorageBackendHealthStatus::Healthy,
        circuit_breaker_state: StorageCircuitBreakerState::Closed,
        consecutive_failures: 0,
        last_success_at_ms: Some(900),
        last_failure_at_ms: None,
        last_failure_class: None,
        last_failure_safe_message: None,
        circuit_opened_at_ms: None,
        backoff_until_ms: None,
        updated_at_ms: 900,
    };

    assert_eq!(
        store
            .get_storage_backend_health(&backend_key)
            .await
            .unwrap(),
        None
    );

    let saved = store
        .upsert_storage_backend_health(unhealthy.clone())
        .await
        .unwrap();
    assert_eq!(saved, unhealthy);
    store
        .upsert_storage_backend_health(healthy_local)
        .await
        .unwrap();

    assert_eq!(
        store
            .get_storage_backend_health(&backend_key)
            .await
            .unwrap(),
        Some(unhealthy.clone())
    );
    assert_eq!(
        store
            .list_storage_backend_health(
                StorageBackendHealthListFilter {
                    library_id: Some(library_id),
                    scheme: Some("webdav".to_owned()),
                    status: Some(StorageBackendHealthStatus::Unavailable),
                    circuit_breaker_state: Some(StorageCircuitBreakerState::Open),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![unhealthy.clone()]
    );

    let recovering = store
        .upsert_storage_backend_health(StorageBackendHealthRecord {
            status: StorageBackendHealthStatus::Recovering,
            circuit_breaker_state: StorageCircuitBreakerState::HalfOpen,
            backoff_until_ms: None,
            updated_at_ms: 2_100,
            ..unhealthy.clone()
        })
        .await
        .unwrap();

    assert_eq!(recovering.status, StorageBackendHealthStatus::Recovering);
    assert_eq!(
        recovering.circuit_breaker_state,
        StorageCircuitBreakerState::HalfOpen
    );
    assert_eq!(recovering.backoff_until_ms, None);

    let reset = store
        .clear_storage_backend_health(&backend_key, 2_200)
        .await
        .unwrap()
        .expect("health reset should return existing backend record");
    assert_eq!(reset.status, StorageBackendHealthStatus::Healthy);
    assert_eq!(
        reset.circuit_breaker_state,
        StorageCircuitBreakerState::Closed
    );
    assert_eq!(reset.consecutive_failures, 0);
    assert_eq!(reset.last_success_at_ms, Some(500));
    assert_eq!(reset.last_failure_at_ms, None);
    assert_eq!(reset.last_failure_class, None);
    assert_eq!(reset.last_failure_safe_message, None);
    assert_eq!(reset.circuit_opened_at_ms, None);
    assert_eq!(reset.backoff_until_ms, None);
    assert_eq!(reset.updated_at_ms, 2_200);
    assert_eq!(
        store
            .clear_storage_backend_health("missing-backend", 2_300)
            .await
            .unwrap(),
        None
    );
}

async fn staging_manifest_contract_round_trips_attribution_variants<S>(store: S)
where
    S: VfsStagingContractBackend,
{
    let library_id = LibraryId::new();
    let cases = [
        (
            StagingManifestId::new(),
            "webdav:///Contract/Movies/Attributed.mkv",
            "webdav",
            "/var/cache/nako/staging/attributed.mkv",
            StagingAttribution::attributed(library_id),
        ),
        (
            StagingManifestId::new(),
            "webdav:///Contract/Shared/Ambiguous.mkv",
            "webdav",
            "/var/cache/nako/staging/ambiguous.mkv",
            StagingAttribution::ambiguous(),
        ),
        (
            StagingManifestId::new(),
            "local:///Contract/Unknown.mkv",
            "local",
            "/var/cache/nako/staging/unknown.mkv",
            StagingAttribution::unknown(),
        ),
    ];

    for (index, (id, source_uri, source_scheme, local_path, attribution)) in
        cases.iter().enumerate()
    {
        let now_ms = 1_000 + index as i64;
        let saved = store
            .upsert_staging_manifest_record(NewStagingManifestRecord {
                id: *id,
                attribution: *attribution,
                source_uri: (*source_uri).to_owned(),
                source_scheme: (*source_scheme).to_owned(),
                purpose: StagingPurpose::ProbeInput,
                local_path: (*local_path).to_owned(),
                size_bytes: Some(10 + index as u64),
                etag: None,
                fingerprint: None,
                state: StagingState::Ready,
                created_at_ms: now_ms,
                updated_at_ms: now_ms,
                last_accessed_at_ms: now_ms,
                expires_at_ms: Some(10_000),
                active_leases: 0,
                validation_error: None,
            })
            .await
            .unwrap();

        assert_eq!(saved.attribution, *attribution);
        assert_eq!(
            store
                .get_staging_manifest_record(*id)
                .await
                .unwrap()
                .expect("staging attribution record")
                .attribution,
            *attribution
        );
    }

    let listed = store
        .list_staging_manifest_records(
            Some(StagingPurpose::ProbeInput),
            Some(StagingState::Ready),
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(
        listed
            .iter()
            .map(|record| record.attribution)
            .collect::<Vec<_>>(),
        vec![
            StagingAttribution::attributed(library_id),
            StagingAttribution::ambiguous(),
            StagingAttribution::unknown(),
        ]
    );

    let reattributed = store
        .upsert_staging_manifest_record(NewStagingManifestRecord {
            id: cases[0].0,
            attribution: StagingAttribution::ambiguous(),
            source_uri: cases[0].1.to_owned(),
            source_scheme: cases[0].2.to_owned(),
            purpose: StagingPurpose::ProbeInput,
            local_path: cases[0].3.to_owned(),
            size_bytes: Some(10),
            etag: None,
            fingerprint: None,
            state: StagingState::Ready,
            created_at_ms: 1_000,
            updated_at_ms: 2_000,
            last_accessed_at_ms: 2_000,
            expires_at_ms: Some(10_000),
            active_leases: 0,
            validation_error: None,
        })
        .await
        .unwrap();
    assert_eq!(reattributed.attribution, StagingAttribution::ambiguous());
    assert_eq!(
        reattributed.attribution.library_id(),
        None,
        "ambiguous attribution must not retain a stale library id"
    );
}

async fn staging_manifest_contract_preserves_reservation_budget_and_leases<S>(store: S)
where
    S: VfsStagingContractBackend,
{
    let library_id = LibraryId::new();
    let id = StagingManifestId::new();
    let base_record = NewStagingManifestRecord {
        id,
        attribution: StagingAttribution::attributed(library_id),
        source_uri: "webdav:///Contract/Movies/Demo.mkv".to_owned(),
        source_scheme: "webdav".to_owned(),
        purpose: StagingPurpose::FfmpegInput,
        local_path: "/var/cache/nako/staging/demo.mkv".to_owned(),
        size_bytes: Some(40),
        etag: Some("etag-demo".to_owned()),
        fingerprint: Some("fingerprint-demo".to_owned()),
        state: StagingState::Reserved,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
        last_accessed_at_ms: 1_000,
        expires_at_ms: Some(5_000),
        active_leases: 0,
        validation_error: None,
    };

    let reserved = store
        .reserve_staging_manifest_record(base_record.clone(), 100, 1_000)
        .await
        .unwrap();
    assert_eq!(reserved.id, id);
    assert_eq!(
        reserved.attribution,
        StagingAttribution::attributed(library_id)
    );
    assert_eq!(reserved.state, StagingState::Reserved);
    assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 40);

    let duplicate = store
        .reserve_staging_manifest_record(
            NewStagingManifestRecord {
                id: StagingManifestId::new(),
                source_uri: "webdav:///Contract/Movies/Demo-copy.mkv".to_owned(),
                ..base_record.clone()
            },
            100,
            2_000,
        )
        .await
        .unwrap_err();
    assert!(matches!(
        duplicate,
        NakoError::Storage { .. } | NakoError::Conflict { .. }
    ));

    let budget_err = store
        .reserve_staging_manifest_record(
            NewStagingManifestRecord {
                id: StagingManifestId::new(),
                source_uri: "webdav:///Contract/Movies/Large.mkv".to_owned(),
                local_path: "/var/cache/nako/staging/large.mkv".to_owned(),
                size_bytes: Some(80),
                ..base_record.clone()
            },
            100,
            2_000,
        )
        .await
        .unwrap_err();
    assert!(matches!(budget_err, NakoError::Storage { .. }));

    let staging = store
        .start_staging_manifest_record(id, 1_100)
        .await
        .unwrap();
    assert_eq!(staging.state, StagingState::Staging);

    let ready = store
        .complete_staging_manifest_record(NewStagingManifestRecord {
            state: StagingState::Ready,
            updated_at_ms: 1_200,
            last_accessed_at_ms: 1_200,
            ..base_record.clone()
        })
        .await
        .unwrap();
    assert_eq!(ready.state, StagingState::Ready);

    let leased = store
        .acquire_staging_manifest_lease(id, 1_300)
        .await
        .unwrap();
    assert_eq!(leased.state, StagingState::Leased);
    assert_eq!(leased.active_leases, 1);

    let after_expire_attempt = store
        .expire_staging_manifest_record(id, 6_000)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after_expire_attempt.state, StagingState::Leased);
    assert_eq!(after_expire_attempt.active_leases, 1);

    let ready_again = store
        .release_staging_manifest_lease(id, 6_100)
        .await
        .unwrap();
    assert_eq!(ready_again.state, StagingState::Ready);
    assert_eq!(ready_again.active_leases, 0);

    let cleanup = store
        .list_staging_cleanup_candidates(6_200, PageRequest::first_page())
        .await
        .unwrap();
    assert_eq!(cleanup, vec![ready_again.clone()]);

    let touched = store
        .touch_staging_manifest_record(id, 6_300)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(touched.last_accessed_at_ms, 6_300);

    assert_eq!(
        store
            .find_staging_manifest_record_by_path("/var/cache/nako/staging/demo.mkv")
            .await
            .unwrap()
            .map(|record| record.id),
        Some(id)
    );
    assert_eq!(
        store
            .list_staging_manifest_records(
                Some(StagingPurpose::FfmpegInput),
                Some(StagingState::Ready),
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .into_iter()
            .map(|record| record.id)
            .collect::<Vec<_>>(),
        vec![id]
    );

    let failed_id = StagingManifestId::new();
    let failed_ready = store
        .complete_staging_manifest_record(NewStagingManifestRecord {
            id: failed_id,
            attribution: StagingAttribution::ambiguous(),
            source_uri: "webdav:///Contract/Movies/Corrupt.mkv".to_owned(),
            local_path: "/var/cache/nako/staging/corrupt.mkv".to_owned(),
            state: StagingState::Ready,
            size_bytes: Some(10),
            created_at_ms: 7_000,
            updated_at_ms: 7_000,
            last_accessed_at_ms: 7_000,
            expires_at_ms: Some(8_000),
            ..base_record.clone()
        })
        .await
        .unwrap();
    assert_eq!(failed_ready.state, StagingState::Ready);
    assert_eq!(failed_ready.attribution, StagingAttribution::ambiguous());

    let failed = store
        .fail_staging_manifest_record(failed_id, 7_100, "checksum mismatch".to_owned())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(failed.state, StagingState::Failed);
    assert_eq!(
        failed.validation_error.as_deref(),
        Some("checksum mismatch")
    );

    let deleted = store
        .mark_deleted_staging_manifest_record(failed_id, 7_200)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(deleted.state, StagingState::Deleted);
    assert_eq!(deleted.expires_at_ms, None);

    store.delete_staging_manifest_record(id).await.unwrap();
    store
        .delete_staging_manifest_record(failed_id)
        .await
        .unwrap();
    assert_eq!(store.get_staging_manifest_record(id).await.unwrap(), None);
    assert_eq!(store.sum_staging_manifest_bytes().await.unwrap(), 0);
}

async fn managed_import_contract_round_trips_artifacts_and_state<S>(store: S)
where
    S: ManagedImportContractBackend,
{
    let library = seed_contract_library(&store).await;
    let staging_manifest_id = StagingManifestId::new();
    let staging_manifest = NewStagingManifestRecord {
        id: staging_manifest_id,
        attribution: StagingAttribution::unknown(),
        source_uri: "local:///incoming/Demo.mkv".to_owned(),
        source_scheme: "local".to_owned(),
        purpose: StagingPurpose::ProbeInput,
        local_path: "/var/cache/nako/import/demo.mkv".to_owned(),
        size_bytes: Some(120),
        etag: Some("etag-demo".to_owned()),
        fingerprint: Some("fingerprint-demo".to_owned()),
        state: StagingState::Ready,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
        last_accessed_at_ms: 1_000,
        expires_at_ms: Some(50_000),
        active_leases: 0,
        validation_error: None,
    };
    store
        .upsert_staging_manifest_record(staging_manifest)
        .await
        .unwrap();

    let id = ManagedImportArtifactId::new();
    let source_kind = ManagedImportSourceKind::WatchedCandidate;
    let artifact = NewManagedImportArtifact {
        id,
        target_library_id: library.id,
        source_kind: source_kind.clone(),
        source_uri: "file:///incoming/Demo.mkv".to_owned(),
        staging_manifest_id: Some(staging_manifest_id),
        artifact_uri: Some("staging:///managed-import/demo.mkv".to_owned()),
        original_file_name: Some("Demo.mkv".to_owned()),
        intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
        size_bytes: Some(120),
        fingerprint: Some("fingerprint-demo".to_owned()),
        state: ManagedImportArtifactState::Staged,
        diagnostics_json: Some(r#"{"redacted":true}"#.to_owned()),
        created_at_ms: 1_100,
        updated_at_ms: 1_200,
    };

    let saved = store
        .upsert_managed_import_artifact(artifact.clone())
        .await
        .unwrap();
    assert_eq!(saved.id, id);
    assert_eq!(saved.target_library_id, library.id);
    assert_eq!(saved.source_kind, source_kind);
    assert_eq!(saved.staging_manifest_id, Some(staging_manifest_id));
    assert_eq!(saved.state, ManagedImportArtifactState::Staged);
    assert_eq!(saved.size_bytes, Some(120));
    assert_eq!(
        store.get_managed_import_artifact(id).await.unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .find_managed_import_artifact_by_source(
                library.id,
                &ManagedImportSourceKind::WatchedCandidate,
                "file:///incoming/Demo.mkv",
            )
            .await
            .unwrap(),
        Some(saved.clone())
    );

    let planned = store
        .set_managed_import_artifact_state(
            id,
            ManagedImportArtifactState::Planned,
            1_300,
            Some(r#"{"plan":"copy","writes_library":false}"#.to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(planned.state, ManagedImportArtifactState::Planned);
    assert_eq!(planned.updated_at_ms, 1_300);
    assert_eq!(
        planned.diagnostics_json.as_deref(),
        Some(r#"{"plan":"copy","writes_library":false}"#)
    );

    let proposed_id = ManagedImportArtifactId::new();
    let proposed = store
        .upsert_managed_import_artifact(NewManagedImportArtifact {
            id: proposed_id,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::AddonProposed,
            source_uri: "addon://demo/artifact/1".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: Some("Addon Demo.mkv".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            state: ManagedImportArtifactState::Proposed,
            diagnostics_json: None,
            created_at_ms: 1_400,
            updated_at_ms: 1_400,
        })
        .await
        .unwrap();

    let planned_items = store
        .list_managed_import_artifacts(
            ManagedImportArtifactListFilter {
                target_library_id: Some(library.id),
                state: Some(ManagedImportArtifactState::Planned),
                source_kind: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(planned_items, vec![planned.clone()]);

    let proposed_items = store
        .list_managed_import_artifacts(
            ManagedImportArtifactListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(ManagedImportSourceKind::AddonProposed),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(proposed_items, vec![proposed]);

    let resource_search_selection = store
        .upsert_managed_import_artifact(NewManagedImportArtifact {
            id: ManagedImportArtifactId::new(),
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::ResourceSearchSelection,
            source_uri: "https://pan.example/resource-search-selection".to_owned(),
            staging_manifest_id: None,
            artifact_uri: None,
            original_file_name: Some("Selected Resource.mkv".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            state: ManagedImportArtifactState::Proposed,
            diagnostics_json: Some(r#"{"resource_search_selection":true}"#.to_owned()),
            created_at_ms: 1_450,
            updated_at_ms: 1_450,
        })
        .await
        .unwrap();
    let resource_search_items = store
        .list_managed_import_artifacts(
            ManagedImportArtifactListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(ManagedImportSourceKind::ResourceSearchSelection),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(resource_search_items, vec![resource_search_selection]);

    let missing = store
        .set_managed_import_artifact_state(
            ManagedImportArtifactId::new(),
            ManagedImportArtifactState::Rejected,
            1_500,
            None,
        )
        .await
        .unwrap();
    assert_eq!(missing, None);
}

async fn promotion_apply_contract_round_trips_acceptance_and_audit<S>(store: S)
where
    S: ManagedImportContractBackend,
{
    let library = seed_contract_library(&store).await;
    let artifact_id = ManagedImportArtifactId::new();
    let artifact = store
        .upsert_managed_import_artifact(NewManagedImportArtifact {
            id: artifact_id,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::WatchedCandidate,
            source_uri: "file:///incoming/Demo.mkv".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Demo.mkv".to_owned()),
            original_file_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(120),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: ManagedImportArtifactState::Planned,
            diagnostics_json: Some(r#"{"preview":"ready"}"#.to_owned()),
            created_at_ms: 2_000,
            updated_at_ms: 2_000,
        })
        .await
        .unwrap();
    assert_eq!(artifact.state, ManagedImportArtifactState::Planned);

    let apply_id = ManagedImportPromotionApplyId::new();
    let apply = NewManagedImportPromotionApply {
        id: apply_id,
        artifact_id,
        target_library_id: library.id,
        requested_by: UserPrincipalId::local_admin(),
        idempotency_key: "promotion-demo-1".to_owned(),
        operation_kind: ManagedImportPromotionOperationKind::Hardlink,
        source_artifact_uri: Some("local:///incoming/Demo.mkv".to_owned()),
        destination_locator: "local:///Contract Movies/Movies/Demo (2026)/Demo.mkv".to_owned(),
        accepted_plan_json: r#"{"plan_version":1,"operation":"hardlink"}"#.to_owned(),
        accepted_warnings_json: Some(r#"["duplicate_hint"]"#.to_owned()),
        state: ManagedImportPromotionApplyState::Requested,
        outcome_json: None,
        safe_error_code: None,
        safe_message: None,
        created_at_ms: 2_100,
        updated_at_ms: 2_100,
    };

    let saved = store
        .upsert_managed_import_promotion_apply(apply)
        .await
        .unwrap();
    assert_eq!(saved.id, apply_id);
    assert_eq!(saved.artifact_id, artifact_id);
    assert_eq!(saved.target_library_id, library.id);
    assert_eq!(saved.requested_by, UserPrincipalId::local_admin());
    assert_eq!(
        saved.operation_kind,
        ManagedImportPromotionOperationKind::Hardlink
    );
    assert_eq!(saved.state, ManagedImportPromotionApplyState::Requested);
    assert_eq!(
        saved.accepted_plan_json,
        r#"{"plan_version":1,"operation":"hardlink"}"#
    );
    assert_eq!(saved.safe_error_code, None);
    assert_eq!(saved.safe_message, None);
    assert_eq!(
        store
            .get_managed_import_promotion_apply(apply_id)
            .await
            .unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .find_managed_import_promotion_apply_by_idempotency_key(library.id, "promotion-demo-1",)
            .await
            .unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .list_managed_import_promotion_applies_for_artifact(
                artifact_id,
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![saved.clone()]
    );

    let accepted = store
        .set_managed_import_promotion_apply_state(
            apply_id,
            ManagedImportPromotionApplyState::Accepted,
            2_200,
            Some(r#"{"validated":true,"writes_library":false}"#.to_owned()),
            None,
            Some("accepted for storage apply".to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(accepted.state, ManagedImportPromotionApplyState::Accepted);
    assert_eq!(
        accepted.outcome_json.as_deref(),
        Some(r#"{"validated":true,"writes_library":false}"#)
    );
    assert_eq!(
        accepted.safe_message.as_deref(),
        Some("accepted for storage apply")
    );

    let cleanup_pending = store
        .set_managed_import_promotion_apply_state(
            apply_id,
            ManagedImportPromotionApplyState::CleanupPending,
            2_300,
            Some(r#"{"storage_target_created":true,"catalog_committed":false}"#.to_owned()),
            Some("catalog_commit_failed".to_owned()),
            Some("cleanup required".to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        cleanup_pending.state,
        ManagedImportPromotionApplyState::CleanupPending
    );
    assert_eq!(cleanup_pending.updated_at_ms, 2_300);
    assert_eq!(
        cleanup_pending.safe_error_code.as_deref(),
        Some("catalog_commit_failed")
    );

    let missing = store
        .set_managed_import_promotion_apply_state(
            ManagedImportPromotionApplyId::new(),
            ManagedImportPromotionApplyState::Rejected,
            2_400,
            None,
            Some("missing_apply".to_owned()),
            None,
        )
        .await
        .unwrap();
    assert_eq!(missing, None);
}

async fn acquisition_intake_contract_round_trips_candidates_and_state<S>(store: S)
where
    S: AcquisitionIntakeContractBackend,
{
    let library = seed_contract_library(&store).await;
    let artifact_id = ManagedImportArtifactId::new();
    store
        .upsert_managed_import_artifact(NewManagedImportArtifact {
            id: artifact_id,
            target_library_id: library.id,
            source_kind: ManagedImportSourceKind::WatchedCandidate,
            source_uri: "file:///incoming/Demo.mkv".to_owned(),
            staging_manifest_id: None,
            artifact_uri: Some("local:///incoming/Demo.mkv".to_owned()),
            original_file_name: Some("Demo.mkv".to_owned()),
            intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
            size_bytes: Some(120),
            fingerprint: Some("fingerprint-demo".to_owned()),
            state: ManagedImportArtifactState::Staged,
            diagnostics_json: Some(r#"{"redacted":true}"#.to_owned()),
            created_at_ms: 1_000,
            updated_at_ms: 1_000,
        })
        .await
        .unwrap();

    let id = AcquisitionIntakeCandidateId::new();
    let source_kind = AcquisitionIntakeSourceKind::WatchFolder;
    let candidate = NewAcquisitionIntakeCandidate {
        id,
        target_library_id: library.id,
        source_kind: source_kind.clone(),
        source_key: "watch:incoming/demo.mkv:fingerprint-demo".to_owned(),
        source_uri: "local:///incoming/Demo.mkv".to_owned(),
        display_name: Some("Demo.mkv".to_owned()),
        intended_locator: Some("Movies/Demo (2026)/Demo.mkv".to_owned()),
        size_bytes: Some(120),
        fingerprint: Some("fingerprint-demo".to_owned()),
        managed_import_artifact_id: None,
        state: AcquisitionIntakeCandidateState::Discovered,
        diagnostics_json: Some(r#"{"redacted":true,"writes_library":false}"#.to_owned()),
        first_seen_at_ms: 1_100,
        last_seen_at_ms: 1_100,
        created_at_ms: 1_100,
        updated_at_ms: 1_100,
    };

    let saved = store
        .upsert_acquisition_intake_candidate(candidate)
        .await
        .unwrap();
    assert_eq!(saved.id, id);
    assert_eq!(saved.target_library_id, library.id);
    assert_eq!(saved.source_kind, source_kind);
    assert_eq!(saved.source_key, "watch:incoming/demo.mkv:fingerprint-demo");
    assert_eq!(saved.managed_import_artifact_id, None);
    assert_eq!(saved.state, AcquisitionIntakeCandidateState::Discovered);
    assert_eq!(
        store.get_acquisition_intake_candidate(id).await.unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .find_acquisition_intake_candidate_by_source_key(
                library.id,
                &AcquisitionIntakeSourceKind::WatchFolder,
                "watch:incoming/demo.mkv:fingerprint-demo",
            )
            .await
            .unwrap(),
        Some(saved.clone())
    );

    let ready = store
        .set_acquisition_intake_candidate_state(
            id,
            AcquisitionIntakeCandidateState::Ready,
            1_200,
            Some(r#"{"ready":true,"writes_library":false}"#.to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ready.state, AcquisitionIntakeCandidateState::Ready);
    assert_eq!(ready.updated_at_ms, 1_200);
    assert_eq!(
        ready.diagnostics_json.as_deref(),
        Some(r#"{"ready":true,"writes_library":false}"#)
    );

    let accepted = store
        .link_acquisition_intake_candidate_managed_import_artifact(
            id,
            artifact_id,
            1_300,
            Some(r#"{"accepted":true,"managed_import_linked":true}"#.to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(accepted.state, AcquisitionIntakeCandidateState::Accepted);
    assert_eq!(accepted.managed_import_artifact_id, Some(artifact_id));
    assert_eq!(accepted.updated_at_ms, 1_300);

    let operator_candidate = store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: AcquisitionIntakeCandidateId::new(),
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::OperatorSubmitted,
            source_key: "operator:manual-demo".to_owned(),
            source_uri: "upload://manual-demo".to_owned(),
            display_name: Some("Manual Demo.mkv".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Blocked,
            diagnostics_json: Some(r#"{"blocked":["missing_artifact"]}"#.to_owned()),
            first_seen_at_ms: 1_400,
            last_seen_at_ms: 1_400,
            created_at_ms: 1_400,
            updated_at_ms: 1_400,
        })
        .await
        .unwrap();

    let accepted_items = store
        .list_acquisition_intake_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: Some(AcquisitionIntakeCandidateState::Accepted),
                source_kind: None,
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(accepted_items, vec![accepted.clone()]);

    let operator_items = store
        .list_acquisition_intake_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(AcquisitionIntakeSourceKind::OperatorSubmitted),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(operator_items, vec![operator_candidate]);

    let resource_search_candidate = store
        .upsert_acquisition_intake_candidate(NewAcquisitionIntakeCandidate {
            id: AcquisitionIntakeCandidateId::new(),
            target_library_id: library.id,
            source_kind: AcquisitionIntakeSourceKind::ResourceSearchSelection,
            source_key: "resource_search_selection:sha256:demo".to_owned(),
            source_uri: "https://pan.example/resource-search-selection".to_owned(),
            display_name: Some("Selected Resource.mkv".to_owned()),
            intended_locator: None,
            size_bytes: None,
            fingerprint: None,
            managed_import_artifact_id: None,
            state: AcquisitionIntakeCandidateState::Ready,
            diagnostics_json: Some(r#"{"resource_search_selection":true}"#.to_owned()),
            first_seen_at_ms: 1_450,
            last_seen_at_ms: 1_450,
            created_at_ms: 1_450,
            updated_at_ms: 1_450,
        })
        .await
        .unwrap();
    let resource_search_items = store
        .list_acquisition_intake_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: Some(AcquisitionIntakeSourceKind::ResourceSearchSelection),
                managed_import_artifact_id: None,
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(resource_search_items, vec![resource_search_candidate]);

    let linked_items = store
        .list_acquisition_intake_candidates(
            AcquisitionIntakeCandidateListFilter {
                target_library_id: Some(library.id),
                state: None,
                source_kind: None,
                managed_import_artifact_id: Some(artifact_id),
            },
            PageRequest::first_page(),
        )
        .await
        .unwrap();
    assert_eq!(linked_items, vec![accepted]);

    let missing = store
        .set_acquisition_intake_candidate_state(
            AcquisitionIntakeCandidateId::new(),
            AcquisitionIntakeCandidateState::Rejected,
            1_500,
            None,
        )
        .await
        .unwrap();
    assert_eq!(missing, None);
}

async fn nfo_sidecar_apply_contract_round_trips_acceptance_and_audit<S>(store: S)
where
    S: NfoSidecarApplyContractBackend,
{
    let library = seed_contract_library(&store).await;
    let source = seed_contract_media_item_with_source(
        &store,
        library.id,
        "NFO Apply Demo",
        "local:///Contract Movies/NFO Apply Demo (2026)/NFO Apply Demo.mkv",
    )
    .await;

    let apply_id = NfoSidecarApplyId::new();
    let apply = NewNfoSidecarApply {
        id: apply_id,
        target_library_id: library.id,
        media_item_id: source.item_id,
        media_source_id: Some(source.id),
        requested_by: UserPrincipalId::local_admin(),
        idempotency_key: "nfo-sidecar-demo-1".to_owned(),
        operation_kind: NfoSidecarApplyOperationKind::ExportSidecar,
        sidecar_locator: "local:///Contract Movies/NFO Apply Demo (2026)/NFO Apply Demo.nfo"
            .to_owned(),
        accepted_preview_json: r#"{"preview_version":1,"operation":"export_sidecar"}"#.to_owned(),
        accepted_warnings_json: Some(r#"["backup_required"]"#.to_owned()),
        policy_version: "nfo-sidecar-policy-v1".to_owned(),
        state: NfoSidecarApplyState::Requested,
        outcome_json: None,
        safe_error_code: None,
        safe_message: None,
        created_at_ms: 3_100,
        updated_at_ms: 3_100,
    };

    let saved = store.upsert_nfo_sidecar_apply(apply).await.unwrap();
    assert_eq!(saved.id, apply_id);
    assert_eq!(saved.target_library_id, library.id);
    assert_eq!(saved.media_item_id, source.item_id);
    assert_eq!(saved.media_source_id, Some(source.id));
    assert_eq!(saved.requested_by, UserPrincipalId::local_admin());
    assert_eq!(
        saved.operation_kind,
        NfoSidecarApplyOperationKind::ExportSidecar
    );
    assert_eq!(saved.state, NfoSidecarApplyState::Requested);
    assert_eq!(
        saved.accepted_preview_json,
        r#"{"preview_version":1,"operation":"export_sidecar"}"#
    );
    assert_eq!(saved.policy_version, "nfo-sidecar-policy-v1");
    assert_eq!(saved.safe_error_code, None);
    assert_eq!(saved.safe_message, None);
    assert_eq!(
        store.get_nfo_sidecar_apply(apply_id).await.unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .find_nfo_sidecar_apply_by_idempotency_key(library.id, "nfo-sidecar-demo-1")
            .await
            .unwrap(),
        Some(saved.clone())
    );
    assert_eq!(
        store
            .list_nfo_sidecar_applies_for_item(source.item_id, PageRequest::first_page())
            .await
            .unwrap(),
        vec![saved.clone()]
    );

    let accepted = store
        .set_nfo_sidecar_apply_state(
            apply_id,
            NfoSidecarApplyState::Accepted,
            3_200,
            Some(r#"{"preview_revalidated":true,"writes_library":false}"#.to_owned()),
            None,
            Some("accepted for nfo sidecar apply".to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(accepted.state, NfoSidecarApplyState::Accepted);
    assert_eq!(accepted.updated_at_ms, 3_200);
    assert_eq!(
        accepted.outcome_json.as_deref(),
        Some(r#"{"preview_revalidated":true,"writes_library":false}"#)
    );
    assert_eq!(
        accepted.safe_message.as_deref(),
        Some("accepted for nfo sidecar apply")
    );

    let repair_pending = store
        .set_nfo_sidecar_apply_state(
            apply_id,
            NfoSidecarApplyState::RepairPending,
            3_300,
            Some(r#"{"sidecar_written":true,"audit_committed":false}"#.to_owned()),
            Some("audit_commit_failed".to_owned()),
            Some("repair pending; restore from recorded backup if needed".to_owned()),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(repair_pending.state, NfoSidecarApplyState::RepairPending);
    assert_eq!(repair_pending.updated_at_ms, 3_300);
    assert_eq!(
        repair_pending.safe_error_code.as_deref(),
        Some("audit_commit_failed")
    );
    assert_eq!(
        repair_pending.safe_message.as_deref(),
        Some("repair pending; restore from recorded backup if needed")
    );

    let missing = store
        .set_nfo_sidecar_apply_state(
            NfoSidecarApplyId::new(),
            NfoSidecarApplyState::Rejected,
            3_400,
            None,
            Some("missing_apply".to_owned()),
            None,
        )
        .await
        .unwrap();
    assert_eq!(missing, None);
}

async fn migrate_contract<S>(store: S)
where
    S: LifecycleContractBackend,
{
    store.migrate().await.unwrap();
    store.migrate().await.unwrap();
}

async fn admin_metadata_raw_cache_settings_contract<S>(store: S)
where
    S: AdminSettingsContractBackend,
{
    assert!(
        store
            .get_admin_metadata_raw_cache_settings()
            .await
            .unwrap()
            .is_none()
    );

    let first = AdminMetadataRawCacheSettingsRecord {
        settings: AdminMetadataRawCacheSettings {
            retention_ms: 3_600_000,
            cleanup_on_startup: false,
        },
        source: AdminSettingsSource::Admin,
        effect: AdminSettingsEffect::RequiresRestart,
        updated_at_ms: 1_000,
    };
    store
        .upsert_admin_metadata_raw_cache_settings(first.clone())
        .await
        .unwrap();
    assert_eq!(
        store
            .get_admin_metadata_raw_cache_settings()
            .await
            .unwrap()
            .unwrap(),
        first
    );

    let replacement = AdminMetadataRawCacheSettingsRecord {
        settings: AdminMetadataRawCacheSettings {
            retention_ms: 7_200_000,
            cleanup_on_startup: true,
        },
        source: AdminSettingsSource::Admin,
        effect: AdminSettingsEffect::RequiresRestart,
        updated_at_ms: 2_000,
    };
    store
        .upsert_admin_metadata_raw_cache_settings(replacement.clone())
        .await
        .unwrap();
    assert_eq!(
        store
            .get_admin_metadata_raw_cache_settings()
            .await
            .unwrap()
            .unwrap(),
        replacement
    );

    assert!(
        store
            .get_admin_settings_document(AdminSettingsDocumentKey::PlaybackRuntime)
            .await
            .unwrap()
            .is_none()
    );
    let document = AdminSettingsDocumentRecord {
        key: AdminSettingsDocumentKey::PlaybackRuntime,
        payload_json: r#"{"cpu_concurrency":2}"#.to_owned(),
        source: AdminSettingsSource::Admin,
        effect: AdminSettingsEffect::RequiresRestart,
        updated_at_ms: 3_000,
    };
    store
        .upsert_admin_settings_document(document.clone())
        .await
        .unwrap();
    let stored_document = store
        .get_admin_settings_document(AdminSettingsDocumentKey::PlaybackRuntime)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored_document.key, document.key);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&stored_document.payload_json).unwrap(),
        serde_json::from_str::<serde_json::Value>(&document.payload_json).unwrap()
    );
    assert_eq!(stored_document.source, document.source);
    assert_eq!(stored_document.effect, document.effect);
    assert_eq!(stored_document.updated_at_ms, document.updated_at_ms);
}

async fn identity_access_user_roles_and_library_policies_contract<S>(store: S)
where
    S: IdentityAccessContractBackend,
{
    let library = seed_contract_library(&store).await;
    let user = User {
        id: UserId::new(),
        principal_id: UserPrincipalId::new("contract-viewer").unwrap(),
        username: "Contract Viewer".to_owned(),
        display_name: "Contract Viewer".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
    };

    store.upsert_user(&user).await.unwrap();

    assert_eq!(store.get_user(user.id).await.unwrap(), Some(user.clone()));
    assert_eq!(
        store
            .get_user_by_principal(&user.principal_id)
            .await
            .unwrap(),
        Some(user.clone())
    );
    assert_eq!(
        store
            .list_users(PageRequest::first_page())
            .await
            .unwrap()
            .into_iter()
            .map(|user| user.id)
            .collect::<Vec<_>>(),
        vec![user.id]
    );

    store
        .replace_role_assignments(
            user.id,
            &[
                RoleAssignment {
                    user_id: user.id,
                    role: UserRole::Viewer,
                    granted_at_ms: 1_100,
                },
                RoleAssignment {
                    user_id: user.id,
                    role: UserRole::LibraryManager,
                    granted_at_ms: 1_200,
                },
            ],
        )
        .await
        .unwrap();
    assert_eq!(
        store.list_role_assignments(user.id).await.unwrap(),
        vec![
            RoleAssignment {
                user_id: user.id,
                role: UserRole::LibraryManager,
                granted_at_ms: 1_200,
            },
            RoleAssignment {
                user_id: user.id,
                role: UserRole::Viewer,
                granted_at_ms: 1_100,
            },
        ]
    );

    let role_policy = LibraryAccessPolicy {
        scope: LibraryAccessPolicyScope::Role(UserRole::Viewer),
        library_id: library.id,
        access: LibraryAccessLevel::Play,
        created_at_ms: 2_000,
        updated_at_ms: 2_000,
    };
    let user_policy = LibraryAccessPolicy {
        scope: LibraryAccessPolicyScope::User(user.id),
        library_id: library.id,
        access: LibraryAccessLevel::Browse,
        created_at_ms: 2_100,
        updated_at_ms: 2_100,
    };
    store
        .upsert_library_access_policy(&role_policy)
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&user_policy)
        .await
        .unwrap();

    let effective = store
        .resolve_effective_library_access(user.id, library.id)
        .await
        .unwrap();
    assert_eq!(effective.access, LibraryAccessLevel::Play);

    let upgraded_user_policy = LibraryAccessPolicy {
        access: LibraryAccessLevel::Manage,
        updated_at_ms: 2_200,
        ..user_policy
    };
    store
        .upsert_library_access_policy(&upgraded_user_policy)
        .await
        .unwrap();
    let effective = store
        .resolve_effective_library_access(user.id, library.id)
        .await
        .unwrap();
    assert_eq!(effective.access, LibraryAccessLevel::Manage);

    assert_eq!(
        store
            .list_library_access_policies(
                LibraryAccessPolicyFilter {
                    user_id: Some(user.id),
                    role: None,
                    library_id: Some(library.id),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![upgraded_user_policy]
    );

    store
        .delete_library_access_policy(LibraryAccessPolicyScope::Role(UserRole::Viewer), library.id)
        .await
        .unwrap();
    assert!(
        store
            .list_library_access_policies(
                LibraryAccessPolicyFilter {
                    user_id: None,
                    role: Some(UserRole::Viewer),
                    library_id: Some(library.id),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );

    let mut viewer_playback = PlaybackPermissionPolicy::current_playback_defaults();
    viewer_playback.allow_remux = false;
    viewer_playback.max_remote_bitrate = Some(4_000_000);
    let role_playback_policy =
        PlaybackPolicy::role(UserRole::Viewer, library.id, viewer_playback, 3_000);
    store
        .upsert_playback_policy(&role_playback_policy)
        .await
        .unwrap();

    let effective_playback = store
        .resolve_effective_playback_policy(user.id, library.id)
        .await
        .unwrap();
    assert!(!effective_playback.permissions.allow_remux);
    assert_eq!(
        effective_playback.permissions.max_remote_bitrate,
        Some(4_000_000)
    );

    let mut user_playback = PlaybackPermissionPolicy::current_playback_defaults();
    user_playback.allow_video_transcode = false;
    user_playback.max_streaming_bitrate = Some(8_000_000);
    let user_playback_policy = PlaybackPolicy::user(user.id, library.id, user_playback, 3_100);
    store
        .upsert_playback_policy(&user_playback_policy)
        .await
        .unwrap();

    let effective_playback = store
        .resolve_effective_playback_policy(user.id, library.id)
        .await
        .unwrap();
    assert!(effective_playback.permissions.allow_remux);
    assert!(!effective_playback.permissions.allow_video_transcode);
    assert_eq!(
        store
            .list_playback_policies(
                PlaybackPolicyFilter {
                    user_id: Some(user.id),
                    role: None,
                    library_id: Some(library.id),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap(),
        vec![user_playback_policy]
    );

    store
        .delete_playback_policy(PlaybackPolicyScope::Role(UserRole::Viewer), library.id)
        .await
        .unwrap();
    assert!(
        store
            .list_playback_policies(
                PlaybackPolicyFilter {
                    user_id: None,
                    role: Some(UserRole::Viewer),
                    library_id: Some(library.id),
                },
                PageRequest::first_page(),
            )
            .await
            .unwrap()
            .is_empty()
    );
}

async fn credential_session_lifecycle_contract<S>(store: S)
where
    S: CredentialSessionContractBackend,
{
    let user = User {
        id: UserId::new(),
        principal_id: UserPrincipalId::new("credential-session-user").unwrap(),
        username: "Credential User".to_owned(),
        display_name: "Credential User".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
    };
    store.upsert_user(&user).await.unwrap();

    let credential = LocalCredentialRecord {
        user_id: user.id,
        password_hash: "$argon2id$v=19$m=65536,t=3,p=1$first$safe-hash".to_owned(),
        updated_at_ms: 2_000,
    };
    store.upsert_local_credential(&credential).await.unwrap();

    assert_eq!(
        store.get_local_credential_by_user(user.id).await.unwrap(),
        Some(credential.clone())
    );
    assert_eq!(
        store
            .get_local_credential_by_username(" credential USER ")
            .await
            .unwrap(),
        Some(credential.clone())
    );

    let rotated = LocalCredentialRecord {
        password_hash: "$argon2id$v=19$m=65536,t=3,p=1$second$safe-hash".to_owned(),
        updated_at_ms: 3_000,
        ..credential
    };
    store.upsert_local_credential(&rotated).await.unwrap();
    assert_eq!(
        store.get_local_credential_by_user(user.id).await.unwrap(),
        Some(rotated)
    );

    let session = UserSessionRecord {
        id: UserSessionId::new(),
        user_id: user.id,
        token_hash: "sha256:session-token-hash".to_owned(),
        created_at_ms: 4_000,
        last_seen_at_ms: 4_000,
        expires_at_ms: 8_000,
        revoked_at_ms: None,
    };
    store.create_user_session(&session).await.unwrap();
    assert_eq!(
        store
            .get_user_session_by_token_hash("sha256:session-token-hash")
            .await
            .unwrap(),
        Some(session.clone())
    );

    let touched = store
        .touch_user_session(session.id, 4_500)
        .await
        .unwrap()
        .expect("session should exist");
    assert_eq!(touched.last_seen_at_ms, 4_500);
    assert_eq!(touched.revoked_at_ms, None);

    let revoked = store
        .revoke_user_session(session.id, 5_000)
        .await
        .unwrap()
        .expect("session should exist");
    assert_eq!(revoked.revoked_at_ms, Some(5_000));
    assert_eq!(
        store
            .get_user_session_by_token_hash("sha256:session-token-hash")
            .await
            .unwrap()
            .unwrap()
            .revoked_at_ms,
        Some(5_000)
    );
}

async fn invitation_lifecycle_contract<S>(store: S)
where
    S: CredentialSessionContractBackend,
{
    let inviter = User {
        id: UserId::new(),
        principal_id: UserPrincipalId::new("invitation-admin").unwrap(),
        username: "Invitation Admin".to_owned(),
        display_name: "Invitation Admin".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1_000,
        updated_at_ms: 1_000,
    };
    store.upsert_user(&inviter).await.unwrap();

    let invitation = UserInvitationRecord {
        id: UserInvitationId::new(),
        created_by_user_id: inviter.id,
        email_or_username: Some("invitee@example.test".to_owned()),
        token_hash: "sha256:invitation-token-hash".to_owned(),
        roles: vec![UserRole::Viewer],
        status: UserInvitationStatus::Pending,
        expires_at_ms: 86_400_000,
        redeemed_at_ms: None,
        redeemed_by_user_id: None,
        revoked_at_ms: None,
        created_at_ms: 2_000,
        updated_at_ms: 2_000,
    };
    store.create_user_invitation(&invitation).await.unwrap();

    assert_eq!(
        store.get_user_invitation(invitation.id).await.unwrap(),
        Some(invitation.clone())
    );
    assert_eq!(
        store
            .get_user_invitation_by_token_hash("sha256:invitation-token-hash")
            .await
            .unwrap(),
        Some(invitation.clone())
    );
    assert_eq!(
        store
            .list_user_invitations(PageRequest::first_page())
            .await
            .unwrap(),
        vec![invitation.clone()]
    );

    let invitee = User {
        id: UserId::new(),
        principal_id: UserPrincipalId::new("invited-user").unwrap(),
        username: "Invited User".to_owned(),
        display_name: "Invited User".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 3_000,
        updated_at_ms: 3_000,
    };
    let redeemed = store
        .redeem_user_invitation(
            invitation.id,
            &invitee,
            &LocalCredentialRecord {
                user_id: invitee.id,
                password_hash: "$argon2id$v=19$m=65536,t=3,p=1$invited$safe-hash".to_owned(),
                updated_at_ms: 4_000,
            },
            &[RoleAssignment {
                user_id: invitee.id,
                role: UserRole::Viewer,
                granted_at_ms: 4_000,
            }],
            4_000,
        )
        .await
        .unwrap()
        .expect("invitation should exist");
    assert_eq!(redeemed.status, UserInvitationStatus::Redeemed);
    assert_eq!(redeemed.redeemed_at_ms, Some(4_000));
    assert_eq!(redeemed.redeemed_by_user_id, Some(invitee.id));
    assert_eq!(redeemed.revoked_at_ms, None);
    assert!(!redeemed.is_redeemable_at(4_001));
    assert_eq!(
        store.get_user(invitee.id).await.unwrap(),
        Some(invitee.clone())
    );
    assert_eq!(
        store.list_role_assignments(invitee.id).await.unwrap(),
        vec![RoleAssignment {
            user_id: invitee.id,
            role: UserRole::Viewer,
            granted_at_ms: 4_000,
        }]
    );
    assert!(
        store
            .get_local_credential_by_user(invitee.id)
            .await
            .unwrap()
            .is_some()
    );

    let unchanged = store
        .revoke_user_invitation(invitation.id, 5_000)
        .await
        .unwrap()
        .expect("invitation should still exist");
    assert_eq!(unchanged.status, UserInvitationStatus::Redeemed);
    assert_eq!(unchanged.revoked_at_ms, None);

    let revoke_only = UserInvitationRecord {
        id: UserInvitationId::new(),
        token_hash: "sha256:revoke-token-hash".to_owned(),
        created_at_ms: 6_000,
        updated_at_ms: 6_000,
        ..invitation
    };
    store.create_user_invitation(&revoke_only).await.unwrap();
    let revoked = store
        .revoke_user_invitation(revoke_only.id, 7_000)
        .await
        .unwrap()
        .expect("invitation should exist");
    assert_eq!(revoked.status, UserInvitationStatus::Revoked);
    assert_eq!(revoked.revoked_at_ms, Some(7_000));
    assert!(!revoked.is_redeemable_at(7_001));
}

database_contract_pair!(
    sqlite = sqlite_lifecycle_contract_migrate_is_idempotent,
    postgres = postgres_lifecycle_contract_migrate_is_idempotent,
    case = ContractCase::fresh(ContractFamily::Lifecycle, "migrate_is_idempotent"),
    contract = migrate_contract,
);

database_contract_pair!(
    sqlite = sqlite_admin_settings_contract_metadata_raw_cache_settings_round_trip,
    postgres = postgres_admin_settings_contract_metadata_raw_cache_settings_round_trip,
    case = ContractCase::migrated(
        ContractFamily::AdminSettings,
        "metadata_raw_cache_settings_round_trip"
    ),
    contract = admin_metadata_raw_cache_settings_contract,
);

database_contract_pair!(
    sqlite = sqlite_identity_access_contract_user_roles_and_library_policies,
    postgres = postgres_identity_access_contract_user_roles_and_library_policies,
    case = ContractCase::migrated(
        ContractFamily::IdentityAccess,
        "user_roles_and_library_policies"
    ),
    contract = identity_access_user_roles_and_library_policies_contract,
);

database_contract_pair!(
    sqlite = sqlite_credential_session_contract_lifecycle,
    postgres = postgres_credential_session_contract_lifecycle,
    case = ContractCase::migrated(ContractFamily::CredentialSession, "lifecycle"),
    contract = credential_session_lifecycle_contract,
);

database_contract_pair!(
    sqlite = sqlite_credential_session_contract_invitation_lifecycle,
    postgres = postgres_credential_session_contract_invitation_lifecycle,
    case = ContractCase::migrated(ContractFamily::CredentialSession, "invitation_lifecycle"),
    contract = invitation_lifecycle_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_lease_contract_claims_next_with_worker_token_and_filter,
    postgres = postgres_job_lease_contract_claims_next_with_worker_token_and_filter,
    case = ContractCase::migrated(
        ContractFamily::JobLease,
        "claims_next_with_worker_token_and_filter"
    ),
    contract = claim_next_job_lease_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_lease_contract_heartbeats_and_completes_with_run_token_fence,
    postgres = postgres_job_lease_contract_heartbeats_and_completes_with_run_token_fence,
    case = ContractCase::migrated(
        ContractFamily::JobLease,
        "heartbeats_and_completes_with_run_token_fence"
    ),
    contract = job_lease_run_token_fence_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner,
    postgres = postgres_job_lease_contract_cancel_requests_are_durable_and_acknowledged_by_owner,
    case = ContractCase::migrated(
        ContractFamily::JobLease,
        "cancel_requests_are_durable_and_acknowledged_by_owner"
    ),
    contract = job_cancellation_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_lease_contract_recovers_only_expired_running_leases,
    postgres = postgres_job_lease_contract_recovers_only_expired_running_leases,
    case = ContractCase::migrated(
        ContractFamily::JobLease,
        "recovers_only_expired_running_leases"
    ),
    contract = recover_expired_job_leases_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_retry_contract_persists_backoff_and_redacted_queue_pressure,
    postgres = postgres_job_retry_contract_persists_backoff_and_redacted_queue_pressure,
    case = ContractCase::migrated(
        ContractFamily::JobRetry,
        "persists_backoff_and_redacted_queue_pressure"
    ),
    contract = job_retry_backoff_contract,
);

database_contract_pair!(
    sqlite = sqlite_job_retry_contract_priority_policy_orders_fairly_and_recovers,
    postgres = postgres_job_retry_contract_priority_policy_orders_fairly_and_recovers,
    case = ContractCase::migrated(
        ContractFamily::JobRetry,
        "priority_policy_orders_fairly_and_recovers"
    ),
    contract = job_priority_policy_contract,
);

database_contract_pair!(
    sqlite = sqlite_library_media_contract_preserves_library_scoped_source_identity,
    postgres = postgres_library_media_contract_preserves_library_scoped_source_identity,
    case = ContractCase::migrated(
        ContractFamily::LibraryMedia,
        "preserves_library_scoped_source_identity"
    ),
    contract = library_media_identity_contract,
);

database_contract_pair!(
    sqlite = sqlite_library_media_contract_source_inventory_hydrates_sources_items_and_probes,
    postgres = postgres_library_media_contract_source_inventory_hydrates_sources_items_and_probes,
    case = ContractCase::migrated(
        ContractFamily::LibraryMedia,
        "source_inventory_hydrates_sources_items_and_probes"
    ),
    contract = library_source_inventory_projection_contract,
);

database_contract_pair!(
    sqlite = sqlite_library_media_contract_browse_query_filters_sorts_and_pages,
    postgres = postgres_library_media_contract_browse_query_filters_sorts_and_pages,
    case = ContractCase::migrated(
        ContractFamily::LibraryMedia,
        "browse_query_filters_sorts_and_pages"
    ),
    contract = library_item_browse_query_contract,
);

database_contract_pair!(
    sqlite = sqlite_library_media_contract_browse_sort_edges,
    postgres = postgres_library_media_contract_browse_sort_edges,
    case = ContractCase::migrated(ContractFamily::LibraryMedia, "browse_sort_edges"),
    contract = library_item_browse_sort_edges_contract,
);

database_contract_pair!(
    sqlite = sqlite_scan_commit_contract_writes_full_source_unit_and_resolves_failure,
    postgres = postgres_scan_commit_contract_writes_full_source_unit_and_resolves_failure,
    case = ContractCase::migrated(
        ContractFamily::ScanCommit,
        "writes_full_source_unit_and_resolves_failure"
    ),
    contract = scan_commit_writes_full_source_unit_and_resolves_failure_contract,
);

database_contract_pair!(
    sqlite = sqlite_scan_commit_contract_rolls_back_when_search_projection_fails,
    postgres = postgres_scan_commit_contract_rolls_back_when_search_projection_fails,
    case = ContractCase::migrated(
        ContractFamily::ScanCommit,
        "rolls_back_when_search_projection_fails"
    ),
    contract = scan_commit_rolls_back_when_search_projection_fails_contract,
);

database_contract_pair!(
    sqlite = sqlite_metadata_catalog_contract_metadata_refresh_confirms_provider_mapping_and_library_state,
    postgres = postgres_metadata_catalog_contract_metadata_refresh_confirms_provider_mapping_and_library_state,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "metadata_refresh_confirms_provider_mapping_and_library_state"
    ),
    contract = metadata_refresh_commit_confirms_provider_mapping_and_library_state_contract,
);

database_contract_pair!(
    sqlite = sqlite_metadata_catalog_contract_nfo_import_writes_graph_search_and_rolls_back,
    postgres = postgres_metadata_catalog_contract_nfo_import_writes_graph_search_and_rolls_back,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "nfo_import_writes_graph_search_and_rolls_back"
    ),
    contract = nfo_import_commit_writes_catalog_graph_search_and_rolls_back_contract,
);

database_contract_pair!(
    sqlite =
        sqlite_metadata_catalog_contract_addon_metadata_write_updates_projection_apply_outcome_and_rolls_back,
    postgres =
        postgres_metadata_catalog_contract_addon_metadata_write_updates_projection_apply_outcome_and_rolls_back,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "addon_metadata_write_updates_projection_apply_outcome_and_rolls_back"
    ),
    contract = addon_metadata_write_commit_updates_projection_apply_outcome_and_rolls_back_contract,
);

database_contract_pair!(
    sqlite = sqlite_metadata_catalog_contract_metadata_application_updates_item_projection_and_rolls_back,
    postgres =
        postgres_metadata_catalog_contract_metadata_application_updates_item_projection_and_rolls_back,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "metadata_application_updates_item_projection_and_rolls_back"
    ),
    contract = metadata_application_commit_updates_item_projection_and_rolls_back_contract,
);

database_contract_pair!(
    sqlite =
        sqlite_metadata_catalog_contract_generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic,
    postgres =
        postgres_metadata_catalog_contract_generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic"
    ),
    contract = generated_artifact_metadata_apply_outcome_is_idempotent_and_atomic_contract,
);

database_contract_pair!(
    sqlite =
        sqlite_metadata_catalog_contract_generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic,
    postgres =
        postgres_metadata_catalog_contract_generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic,
    case = ContractCase::migrated(
        ContractFamily::MetadataCatalog,
        "generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic"
    ),
    contract = generated_artifact_bulk_metadata_apply_batch_is_idempotent_and_atomic_contract,
);

database_contract_pair!(
    sqlite = sqlite_metadata_candidate_review_batch_contract_is_idempotent_and_atomic,
    postgres = postgres_metadata_candidate_review_batch_contract_is_idempotent_and_atomic,
    case = ContractCase::migrated(
        ContractFamily::MetadataCandidateReviewBatch,
        "is_idempotent_and_atomic"
    ),
    contract = metadata_candidate_review_batch_is_idempotent_and_atomic_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_addon_artwork_candidate_intake,
    postgres = postgres_managed_artwork_contract_addon_artwork_candidate_intake,
    case = ContractCase::migrated(
        ContractFamily::ManagedArtwork,
        "addon_artwork_candidate_intake"
    ),
    contract = addon_artwork_candidate_intake_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_artwork_task_queue,
    postgres = postgres_managed_artwork_contract_artwork_task_queue,
    case = ContractCase::migrated(ContractFamily::ManagedArtwork, "artwork_task_queue"),
    contract = artwork_task_queue_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_acceptance_creates_ingest_job,
    postgres = postgres_managed_artwork_contract_acceptance_creates_ingest_job,
    case = ContractCase::migrated(
        ContractFamily::ManagedArtwork,
        "acceptance_creates_ingest_job"
    ),
    contract = managed_artwork_acceptance_creates_ingest_job_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_ingest_processing,
    postgres = postgres_managed_artwork_contract_ingest_processing,
    case = ContractCase::migrated(ContractFamily::ManagedArtwork, "ingest_processing"),
    contract = managed_artwork_ingest_processing_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_failure_recovery_requeue,
    postgres = postgres_managed_artwork_contract_failure_recovery_requeue,
    case = ContractCase::migrated(ContractFamily::ManagedArtwork, "failure_recovery_requeue"),
    contract = managed_artwork_failure_recovery_requeue_contract,
);

database_contract_pair!(
    sqlite = sqlite_managed_artwork_contract_selected_gallery_lifecycle,
    postgres = postgres_managed_artwork_contract_selected_gallery_lifecycle,
    case = ContractCase::migrated(ContractFamily::ManagedArtwork, "selected_gallery_lifecycle"),
    contract = selected_artwork_gallery_lifecycle_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_user_playback_state_is_principal_scoped_and_continue_watching,
    postgres = postgres_playback_runtime_contract_user_playback_state_is_principal_scoped_and_continue_watching,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "user_playback_state_is_principal_scoped_and_continue_watching"
    ),
    contract = user_playback_state_is_principal_scoped_and_continue_watching_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_continue_watching_projection_filters_access_before_pagination,
    postgres = postgres_playback_runtime_contract_continue_watching_projection_filters_access_before_pagination,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "continue_watching_projection_filters_access_before_pagination"
    ),
    contract = continue_watching_projection_filters_access_before_pagination_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_continue_watching_projection_honors_role_policy_and_admin_source_less,
    postgres = postgres_playback_runtime_contract_continue_watching_projection_honors_role_policy_and_admin_source_less,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "continue_watching_projection_honors_role_policy_and_admin_source_less"
    ),
    contract = continue_watching_projection_honors_role_policy_and_admin_source_less_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_user_playlist_membership_is_principal_scoped_ordered_and_idempotent,
    postgres = postgres_playback_runtime_contract_user_playlist_membership_is_principal_scoped_ordered_and_idempotent,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "user_playlist_membership_is_principal_scoped_ordered_and_idempotent"
    ),
    contract = user_playlist_membership_is_principal_scoped_ordered_and_idempotent_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale,
    postgres = postgres_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "transcode_session_lifecycle_filters_cancellation_and_stale"
    ),
    contract = transcode_session_lifecycle_filters_cancellation_and_stale_contract,
);

database_contract_pair!(
    sqlite = sqlite_playback_runtime_contract_playback_session_tracks_user_attempt_independent_of_transcode,
    postgres = postgres_playback_runtime_contract_playback_session_tracks_user_attempt_independent_of_transcode,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "playback_session_tracks_user_attempt_independent_of_transcode"
    ),
    contract = playback_session_tracks_user_attempt_independent_of_transcode_contract,
);

database_contract_pair!(
    sqlite = sqlite_renderer_runtime_contract_renderer_session_and_command_queue,
    postgres = postgres_renderer_runtime_contract_renderer_session_and_command_queue,
    case = ContractCase::migrated(
        ContractFamily::RendererRuntime,
        "renderer_session_and_command_queue"
    ),
    contract = renderer_session_and_command_queue_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_event_outbox_and_webhook_delivery,
    postgres = postgres_event_addon_automation_contract_event_outbox_and_webhook_delivery,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "event_outbox_and_webhook_delivery"
    ),
    contract = event_outbox_and_webhook_delivery_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_addon_event_delivery_attempt,
    postgres = postgres_event_addon_automation_contract_addon_event_delivery_attempt,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_event_delivery_attempt"
    ),
    contract = addon_event_delivery_attempt_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_addon_event_scheduler_due_work,
    postgres = postgres_event_addon_automation_contract_addon_event_scheduler_due_work,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_event_scheduler_due_work"
    ),
    contract = addon_event_scheduler_due_work_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_addon_event_scheduler_claim,
    postgres = postgres_event_addon_automation_contract_addon_event_scheduler_claim,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_event_scheduler_claim"
    ),
    contract = addon_event_scheduler_claim_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_addon_registration_token_grant_and_side_effect,
    postgres =
        postgres_event_addon_automation_contract_addon_registration_token_grant_and_side_effect,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_registration_token_grant_and_side_effect"
    ),
    contract = addon_registration_token_grant_and_side_effect_contract,
);

database_contract_pair!(
    sqlite =
        sqlite_event_addon_automation_contract_addon_routing_plan_replaces_manifest_declarations,
    postgres =
        postgres_event_addon_automation_contract_addon_routing_plan_replaces_manifest_declarations,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_routing_plan_replaces_manifest_declarations"
    ),
    contract = addon_routing_plan_replaces_manifest_declarations_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_addon_task_run_idempotency_fingerprint,
    postgres = postgres_event_addon_automation_contract_addon_task_run_idempotency_fingerprint,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "addon_task_run_idempotency_fingerprint"
    ),
    contract = addon_task_run_idempotency_fingerprint_contract,
);

database_contract_pair!(
    sqlite = sqlite_event_addon_automation_contract_automation_provider_and_artifact,
    postgres = postgres_event_addon_automation_contract_automation_provider_and_artifact,
    case = ContractCase::migrated(
        ContractFamily::EventAddonAutomation,
        "automation_provider_and_artifact"
    ),
    contract = automation_provider_and_artifact_contract,
);

database_contract_pair!(
    sqlite = sqlite_runtime_promotion_contract_covers_facade_dispatch_gap_surfaces,
    postgres = postgres_runtime_promotion_contract_covers_facade_dispatch_gap_surfaces,
    case = ContractCase::migrated(
        ContractFamily::RuntimePromotion,
        "covers_facade_dispatch_gap_surfaces"
    ),
    contract = runtime_promotion_contract_covers_facade_dispatch_gap_surfaces,
);

database_contract_pair!(
    sqlite = sqlite_source_duplicate_contract_upsert_is_idempotent_by_canonical_pair,
    postgres = postgres_source_duplicate_contract_upsert_is_idempotent_by_canonical_pair,
    case = ContractCase::migrated(
        ContractFamily::SourceDuplicate,
        "upsert_is_idempotent_by_canonical_pair"
    ),
    contract = source_duplicate_contract_upsert_is_idempotent_by_canonical_pair,
);

database_contract_pair!(
    sqlite = sqlite_source_duplicate_contract_lists_fingerprint_matches_and_pair_lookup,
    postgres = postgres_source_duplicate_contract_lists_fingerprint_matches_and_pair_lookup,
    case = ContractCase::migrated(
        ContractFamily::SourceDuplicate,
        "lists_fingerprint_matches_and_pair_lookup"
    ),
    contract = source_duplicate_contract_lists_fingerprint_matches_and_pair_lookup,
);

database_contract_pair!(
    sqlite = sqlite_vfs_staging_contract_round_trips_listing_failures_and_summary,
    postgres = postgres_vfs_staging_contract_round_trips_listing_failures_and_summary,
    case = ContractCase::migrated(
        ContractFamily::VfsStaging,
        "round_trips_listing_failures_and_summary"
    ),
    contract = vfs_cache_contract_round_trips_listing_failures_and_summary,
);

database_contract_pair!(
    sqlite = sqlite_storage_backend_health_contract_records_recovery_and_reset,
    postgres = postgres_storage_backend_health_contract_records_recovery_and_reset,
    case = ContractCase::migrated(
        ContractFamily::StorageBackendHealth,
        "records_recovery_and_reset"
    ),
    contract = storage_backend_health_contract_records_recovery_and_reset,
);

database_contract_pair!(
    sqlite = sqlite_vfs_staging_contract_round_trips_attribution_variants,
    postgres = postgres_vfs_staging_contract_round_trips_attribution_variants,
    case = ContractCase::migrated(
        ContractFamily::VfsStaging,
        "round_trips_attribution_variants"
    ),
    contract = staging_manifest_contract_round_trips_attribution_variants,
);

database_contract_pair!(
    sqlite = sqlite_vfs_staging_contract_preserves_reservation_budget_and_leases,
    postgres = postgres_vfs_staging_contract_preserves_reservation_budget_and_leases,
    case = ContractCase::migrated(
        ContractFamily::VfsStaging,
        "preserves_reservation_budget_and_leases"
    ),
    contract = staging_manifest_contract_preserves_reservation_budget_and_leases,
);

database_contract_pair!(
    sqlite = sqlite_managed_import_contract_round_trips_artifacts_and_state,
    postgres = postgres_managed_import_contract_round_trips_artifacts_and_state,
    case = ContractCase::migrated(
        ContractFamily::ManagedImport,
        "round_trips_artifacts_and_state"
    ),
    contract = managed_import_contract_round_trips_artifacts_and_state,
);

database_contract_pair!(
    sqlite = sqlite_promotion_apply_contract_round_trips_acceptance_and_audit,
    postgres = postgres_promotion_apply_contract_round_trips_acceptance_and_audit,
    case = ContractCase::migrated(
        ContractFamily::ManagedImport,
        "promotion_apply_round_trips_acceptance_and_audit"
    ),
    contract = promotion_apply_contract_round_trips_acceptance_and_audit,
);

database_contract_pair!(
    sqlite = sqlite_acquisition_intake_contract_round_trips_candidates_and_state,
    postgres = postgres_acquisition_intake_contract_round_trips_candidates_and_state,
    case = ContractCase::migrated(
        ContractFamily::AcquisitionIntake,
        "round_trips_candidates_and_state"
    ),
    contract = acquisition_intake_contract_round_trips_candidates_and_state,
);

database_contract_pair!(
    sqlite = sqlite_nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
    postgres = postgres_nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
    case = ContractCase::migrated(
        ContractFamily::NfoSidecarApply,
        "round_trips_acceptance_and_audit"
    ),
    contract = nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
);
