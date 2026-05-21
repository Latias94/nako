use std::{future::Future, sync::OnceLock};

use taru_core::{
    AddonId, AddonMetadataWriteCatalogCommit, AddonMetadataWritePersistenceCommit, AddonPermission,
    AddonRepository, AddonSideEffectApplyOutcome, AddonSideEffectApplyStatus, AddonSideEffectId,
    AddonSideEffectRequestFingerprint, AddonSideEffectTarget, AddonSideEffectValidationStatus,
    AddonStatus, AddonTokenId, ArtworkCandidateId, ArtworkCandidateRecord,
    ArtworkCandidateRepository, ArtworkCandidateSourceKind, ArtworkCandidateStatus, ArtworkTask,
    ArtworkTaskId, ArtworkTaskKind, ArtworkTaskRepository, AutomationArtifactKind,
    AutomationArtifactStatus, AutomationCapability, AutomationProviderStatus, AutomationRepository,
    CancelLeasedJob, CanonicalMetadata, CatalogGovernanceItemListFilter,
    CatalogGovernanceRepository, CatalogItemGraphReplacement, CatalogItemProjectionCommit,
    CatalogRepository, CatalogSearchProjection, Collection, CollectionId, CollectionItem,
    CompleteLeasedJob, CreditRole, DatabaseLifecycle, DirectorySnapshot, DomainEventKind,
    DomainEventSubject, EventOutboxRepository, ExternalId, ExternalProvider, FailLeasedJob, Genre,
    GenreId, ImageAsset, ImageAssetId, ImageKind, ImageOwner, IngestionFailureClass,
    IngestionFailureFilter, IngestionFailurePhase, IngestionFailureRepository,
    IngestionFailureResolution, IngestionFailureStatus, ItemCredit, ItemGenre, ItemStudio, ItemTag,
    Job, JobId, JobKind, JobLeaseClaimFilter, JobLeaseClaimRequest, JobLeaseGuard,
    JobLeaseHeartbeat, JobLeaseRepository, JobRepository, JobRunToken, JobStatus, JobWorkerId,
    Library, LibraryId, LibraryItemRepository, LibraryItemState, LibraryOptions, LibraryPreset,
    LibraryRepository, LibraryScanSourcePersistenceCommit, LocalInferenceEvidence,
    LocalInferenceEvidenceId, LocalInferenceEvidenceSource, LocalInferenceRepository,
    ManagedArtworkAcceptanceRecord, ManagedArtworkArtifactId,
    ManagedArtworkArtifactLifecycleFilter, ManagedArtworkIngestId, ManagedArtworkIngestStatus,
    ManagedArtworkRepository, ManagedImportArtifactId, ManagedImportArtifactListFilter,
    ManagedImportArtifactState, ManagedImportPromotionApplyId, ManagedImportPromotionApplyState,
    ManagedImportPromotionOperationKind, ManagedImportRepository, ManagedImportSourceKind,
    MediaItem, MediaItemId, MediaKind, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, MetadataAttemptFilter, MetadataField, MetadataFieldLock, MetadataMatchKind,
    MetadataProviderAttemptId, MetadataProviderAttemptStatus, MetadataRefreshPersistenceCommit,
    MetadataRefreshProviderMappingCommit, MetadataRepository, MetadataSource, NewAddonGrant,
    NewAddonRegistration, NewAddonSideEffect, NewAddonToken, NewArtworkCandidate,
    NewAutomationArtifact, NewAutomationProviderConfig, NewIngestionFailure, NewJob,
    NewManagedArtworkArtifact, NewManagedArtworkIngest, NewManagedImportArtifact,
    NewManagedImportPromotionApply, NewMetadataProviderAttempt, NewNfoSidecarApply, NewOutboxEvent,
    NewStagingManifestRecord, NewTranscodeSession, NewVfsCacheFailure, NewWebhookDeliveryAttempt,
    NewWebhookEndpoint, NfoImportPersistenceCommit, NfoSidecarApplyId,
    NfoSidecarApplyOperationKind, NfoSidecarApplyRepository, NfoSidecarApplyState,
    OutboxEventListFilter, OutboxEventStatus, PageRequest, Person, PersonId, ProviderMapping,
    ProviderMappingId, ProviderMappingRepository, ProviderMappingStatus, ProviderRawResponse,
    ProviderSubject, ProviderSubjectId, ProviderSubjectKind, RecoverExpiredJobLeases,
    RequestJobCancellation, ScanRepository, ScanSnapshotId, ScanStatus,
    SourceDuplicateEvidenceKind, SourceDuplicateRelationship, SourceDuplicateRelationshipId,
    SourceDuplicateRelationshipStatus, SourceDuplicateRepository, SourceState, StagingManifestId,
    StagingManifestRepository, StagingPurpose, StagingState, Studio, StudioId, Tag, TagId,
    TaruError, TranscodeFailureCategory, TranscodeSessionId, TranscodeSessionKind,
    TranscodeSessionListFilter, TranscodeSessionRepository, TranscodeSessionState,
    UserPlaybackStateRepository, UserPlaybackStateWrite, UserPrincipalId, VfsCacheOperation,
    VfsCacheRepository, VfsCachedListing, VfsCachedObject, VfsCachedObjectKind,
    WebhookDeliveryStatus, WebhookEndpointStatus, WebhookRepository,
};
use taru_search::{SearchIndex, SearchQuery};

use crate::{TaruDatabase, postgres::PostgresStore};

const TARU_TEST_POSTGRES_URL: &str = "TARU_TEST_POSTGRES_URL";

static POSTGRES_CONTRACT_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ContractFamily {
    Lifecycle,
    JobLease,
    LibraryMedia,
    ScanCommit,
    MetadataCatalog,
    ManagedArtwork,
    ManagedImport,
    NfoSidecarApply,
    PlaybackRuntime,
    EventAddonAutomation,
    RuntimePromotion,
    VfsStaging,
}

impl ContractFamily {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::JobLease => "job_lease",
            Self::LibraryMedia => "library_media",
            Self::ScanCommit => "scan_commit",
            Self::MetadataCatalog => "metadata_catalog",
            Self::ManagedArtwork => "managed_artwork",
            Self::ManagedImport => "managed_import",
            Self::NfoSidecarApply => "nfo_sidecar_apply",
            Self::PlaybackRuntime => "playback_runtime",
            Self::EventAddonAutomation => "event_addon_automation",
            Self::RuntimePromotion => "runtime_promotion",
            Self::VfsStaging => "vfs_staging",
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

trait JobLeaseContractBackend:
    LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

impl<T> JobLeaseContractBackend for T where
    T: LifecycleContractBackend + JobRepository + JobLeaseRepository + LibraryRepository
{
}

trait LibraryMediaContractBackend:
    LifecycleContractBackend + LibraryRepository + LibraryItemRepository + MediaRepository
{
}

impl<T> LibraryMediaContractBackend for T where
    T: LifecycleContractBackend + LibraryRepository + LibraryItemRepository + MediaRepository
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
    + TranscodeSessionRepository
    + UserPlaybackStateRepository
{
}

impl<T> PlaybackRuntimeContractBackend for T where
    T: LifecycleContractBackend
        + LibraryRepository
        + MediaRepository
        + TranscodeSessionRepository
        + UserPlaybackStateRepository
{
}

trait EventAddonAutomationContractBackend:
    LifecycleContractBackend
    + AddonRepository
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
        + AddonRepository
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

trait VfsStagingContractBackend:
    LifecycleContractBackend + VfsCacheRepository + StagingManifestRepository
{
}

impl<T> VfsStagingContractBackend for T where
    T: LifecycleContractBackend + VfsCacheRepository + StagingManifestRepository
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
        #[ignore = "requires TARU_TEST_POSTGRES_URL"]
        async fn $postgres_test() {
            run_postgres_contract($case, $contract).await;
        }
    };
}

async fn sqlite_contract_database(setup: ContractSetup) -> TaruDatabase {
    let store = TaruDatabase::connect_in_memory().await.unwrap();
    apply_contract_setup(&store, setup).await;
    store
}

async fn postgres_contract_database(database_url: &str, setup: ContractSetup) -> PostgresStore {
    let schema_name = format!(
        "taru_contract_{}",
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
    F: FnOnce(TaruDatabase) -> Fut,
    Fut: Future<Output = ()>,
{
    contract(sqlite_contract_database(case.setup).await).await;
}

async fn run_postgres_contract<F, Fut>(case: ContractCase, contract: F)
where
    F: FnOnce(PostgresStore) -> Fut,
    Fut: Future<Output = ()>,
{
    let database_url = std::env::var(TARU_TEST_POSTGRES_URL).unwrap_or_else(|_| {
        panic!(
            "PostgreSQL {} contract requires {TARU_TEST_POSTGRES_URL}; do not run ignored PostgreSQL contract gates without a test database URL",
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
    let library = Library {
        id: LibraryId::new(),
        name: "Contract Movies".to_owned(),
        roots: vec!["local:///Contract Movies".to_owned()],
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
    store
        .enqueue_job(NewJob {
            id: JobId::new(),
            kind,
            resource_class: resource_class.to_owned(),
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
) -> Option<taru_core::LeasedJob>
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
    assert!(matches!(stale_heartbeat, TaruError::Conflict { .. }));

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
    assert!(matches!(stale_success, TaruError::Conflict { .. }));
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
    assert!(matches!(stale_failure, TaruError::Conflict { .. }));
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
    assert!(matches!(stale_cancel, TaruError::Conflict { .. }));

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
    assert!(matches!(terminal_cancel, TaruError::Conflict { .. }));
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
        fingerprint: Some("fingerprint-local".to_owned()),
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
        vec![episode]
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
    let expected_attempt = taru_core::MetadataProviderAttemptRecord {
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
        cache_uri: Some("taru://artwork/poster-contract".to_owned()),
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
    let addon_id = taru_core::AddonId::new();
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
            manifest_id: "dev.taru.contract.addon-metadata-write".to_owned(),
            name: "Contract Metadata Addon".to_owned(),
            version: "1.0.0".to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "http://127.0.0.1:43124".to_owned(),
            manifest_json: r#"{"id":"dev.taru.contract.addon-metadata-write"}"#.to_owned(),
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
            token_prefix: "taru_at_metadata".to_owned(),
            token_hash: "hash-addon-metadata".to_owned(),
        })
        .await
        .unwrap();

    let search_only_effect = store
        .create_addon_side_effect(NewAddonSideEffect {
            id: taru_core::AddonSideEffectId::new(),
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
            id: taru_core::AddonSideEffectId::new(),
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
            id: taru_core::AddonSideEffectId::new(),
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
            Some(TranscodeFailureCategory::Runner),
            Some("ffmpeg failed".to_owned()),
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
        Some(TranscodeFailureCategory::Runner)
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
            id: taru_core::EventId::new(),
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
            id: taru_core::EventId::new(),
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

    let endpoint_id = taru_core::WebhookEndpointId::new();
    let endpoint = store
        .upsert_webhook_endpoint(NewWebhookEndpoint {
            id: endpoint_id,
            name: "Library Scan Hook".to_owned(),
            url: "https://hooks.example.test/taru".to_owned(),
            secret_env: Some("TARU_WEBHOOK_SECRET".to_owned()),
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
    assert_eq!(endpoint.secret_env.as_deref(), Some("TARU_WEBHOOK_SECRET"));
    assert_eq!(endpoint.status, WebhookEndpointStatus::Enabled);

    let disabled = store
        .upsert_webhook_endpoint(NewWebhookEndpoint {
            id: taru_core::WebhookEndpointId::new(),
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
            id: taru_core::WebhookDeliveryAttemptId::new(),
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

    let addon_id = taru_core::AddonId::new();
    let addon = store
        .upsert_addon_registration(NewAddonRegistration {
            id: addon_id,
            manifest_id: "dev.taru.contract.addon".to_owned(),
            name: "Contract Addon".to_owned(),
            version: "1.0.0".to_owned(),
            protocol_version: "2026-05".to_owned(),
            base_url: "http://127.0.0.1:43123".to_owned(),
            manifest_json: r#"{"id":"dev.taru.contract.addon","name":"Contract Addon"}"#.to_owned(),
            granted_scopes: vec!["metadata.write".to_owned(), "artwork.write".to_owned()],
            status: AddonStatus::Enabled,
        })
        .await
        .unwrap();
    assert_eq!(addon.id, addon_id);
    assert_eq!(addon.status, AddonStatus::Enabled);

    let by_manifest = store
        .find_addon_registration_by_manifest_id("dev.taru.contract.addon")
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

    let first_token = store
        .create_addon_token(NewAddonToken {
            id: AddonTokenId::new(),
            addon_id,
            label: "initial".to_owned(),
            token_prefix: "taru_at_initial".to_owned(),
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
                token_prefix: "taru_at_rotated".to_owned(),
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
                    id: taru_core::AddonGrantId::new(),
                    addon_id,
                    permission: AddonPermission::MetadataWrite,
                    library_id: Some(library.id),
                },
                NewAddonGrant {
                    id: taru_core::AddonGrantId::new(),
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
        id: taru_core::AddonSideEffectId::new(),
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
            id: taru_core::AddonSideEffectId::new(),
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

    let provider_id = taru_core::AutomationProviderId::new();
    let provider = store
        .upsert_automation_provider(NewAutomationProviderConfig {
            id: provider_id,
            name: "Contract AI".to_owned(),
            base_url: "https://automation.example.test".to_owned(),
            secret_env: Some("TARU_AUTOMATION_TOKEN".to_owned()),
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
        Some("TARU_AUTOMATION_TOKEN")
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
            id: taru_core::AutomationProviderId::new(),
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
            id: taru_core::AutomationArtifactId::new(),
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
            manifest_id: format!("dev.taru.contract.{idempotency_key}"),
            name: "Managed Artwork Contract Addon".to_owned(),
            version: "0.1.0".to_owned(),
            protocol_version: "2026-05-15".to_owned(),
            base_url: "https://example.test/addon".to_owned(),
            manifest_json: r#"{"id":"dev.taru.contract.managed-artwork"}"#.to_owned(),
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
            token_prefix: "taru_at_managed_artwork".to_owned(),
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
        !serde_json::to_string(&taru_api_safety_projection(&gallery))
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

fn taru_api_safety_projection(
    gallery: &taru_core::ManagedArtworkGallerySnapshot,
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

    let first_failure = store
        .record_vfs_cache_failure(NewVfsCacheFailure {
            uri: "webdav:///Contract/Movies/".to_owned(),
            scheme: "webdav".to_owned(),
            operation: VfsCacheOperation::List,
            failed_at_ms: 900,
            error: "timeout".to_owned(),
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
        })
        .await
        .unwrap();

    assert_eq!(first_failure.failure_count, 1);
    assert_eq!(second_failure.failure_count, 2);
    assert_eq!(second_failure.failed_at_ms, 950);
    assert_eq!(second_failure.error, "rate limited");
    assert_eq!(
        store
            .get_vfs_cache_failure("webdav:///Contract/Movies/", VfsCacheOperation::List)
            .await
            .unwrap(),
        Some(second_failure)
    );

    let summary = store.summarize_vfs_cache(700).await.unwrap();
    assert_eq!(summary.object_count, 3);
    assert_eq!(summary.listing_count, 1);
    assert_eq!(summary.failure_count, 1);
    assert_eq!(summary.stale_object_count, 2);
    assert_eq!(summary.stale_listing_count, 0);
    assert_eq!(summary.last_failure_at_ms, Some(950));
}

async fn staging_manifest_contract_preserves_reservation_budget_and_leases<S>(store: S)
where
    S: VfsStagingContractBackend,
{
    let id = StagingManifestId::new();
    let base_record = NewStagingManifestRecord {
        id,
        source_uri: "webdav:///Contract/Movies/Demo.mkv".to_owned(),
        source_scheme: "webdav".to_owned(),
        purpose: StagingPurpose::FfmpegInput,
        local_path: "/var/cache/taru/staging/demo.mkv".to_owned(),
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
        TaruError::Storage { .. } | TaruError::Conflict { .. }
    ));

    let budget_err = store
        .reserve_staging_manifest_record(
            NewStagingManifestRecord {
                id: StagingManifestId::new(),
                source_uri: "webdav:///Contract/Movies/Large.mkv".to_owned(),
                local_path: "/var/cache/taru/staging/large.mkv".to_owned(),
                size_bytes: Some(80),
                ..base_record.clone()
            },
            100,
            2_000,
        )
        .await
        .unwrap_err();
    assert!(matches!(budget_err, TaruError::Storage { .. }));

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
            .find_staging_manifest_record_by_path("/var/cache/taru/staging/demo.mkv")
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
            source_uri: "webdav:///Contract/Movies/Corrupt.mkv".to_owned(),
            local_path: "/var/cache/taru/staging/corrupt.mkv".to_owned(),
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
        source_uri: "local:///incoming/Demo.mkv".to_owned(),
        source_scheme: "local".to_owned(),
        purpose: StagingPurpose::ProbeInput,
        local_path: "/var/cache/taru/import/demo.mkv".to_owned(),
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

database_contract_pair!(
    sqlite = sqlite_lifecycle_contract_migrate_is_idempotent,
    postgres = postgres_lifecycle_contract_migrate_is_idempotent,
    case = ContractCase::fresh(ContractFamily::Lifecycle, "migrate_is_idempotent"),
    contract = migrate_contract,
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
    sqlite = sqlite_library_media_contract_preserves_library_scoped_source_identity,
    postgres = postgres_library_media_contract_preserves_library_scoped_source_identity,
    case = ContractCase::migrated(
        ContractFamily::LibraryMedia,
        "preserves_library_scoped_source_identity"
    ),
    contract = library_media_identity_contract,
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
    sqlite = sqlite_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale,
    postgres = postgres_playback_runtime_contract_transcode_session_lifecycle_filters_cancellation_and_stale,
    case = ContractCase::migrated(
        ContractFamily::PlaybackRuntime,
        "transcode_session_lifecycle_filters_cancellation_and_stale"
    ),
    contract = transcode_session_lifecycle_filters_cancellation_and_stale_contract,
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
    sqlite = sqlite_vfs_staging_contract_round_trips_listing_failures_and_summary,
    postgres = postgres_vfs_staging_contract_round_trips_listing_failures_and_summary,
    case = ContractCase::migrated(
        ContractFamily::VfsStaging,
        "round_trips_listing_failures_and_summary"
    ),
    contract = vfs_cache_contract_round_trips_listing_failures_and_summary,
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
    sqlite = sqlite_nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
    postgres = postgres_nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
    case = ContractCase::migrated(
        ContractFamily::NfoSidecarApply,
        "round_trips_acceptance_and_audit"
    ),
    contract = nfo_sidecar_apply_contract_round_trips_acceptance_and_audit,
);
