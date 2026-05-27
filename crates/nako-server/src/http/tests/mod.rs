use std::{
    fs,
    path::{Path as FsPath, PathBuf},
    time::Duration,
};

use axum::{
    Extension, Json,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use nako_addon_client::{ReqwestAddonTransport, call_addon_resource};
use nako_addon_protocol::{
    ADDON_PROTOCOL_VERSION, AddonAuth, AddonConfigurationSchema, AddonEntryPointDeclaration,
    AddonEntryPointKind, AddonEventRequest, AddonEventResponse, AddonEventSubscriptionDeclaration,
    AddonHealthCheckRequest, AddonHealthCheckResponse as ProtocolAddonHealthCheckResponse,
    AddonHealthManifestFacts, AddonHealthStatus, AddonHostedPageDeclaration,
    AddonInstallDescriptor, AddonManifest, AddonResource, AddonResourceDeclaration,
    AddonRuntimeKind, AddonRuntimeRequirement, AddonScope, AddonSecretReferenceBinding,
    AddonSecretReferenceFieldDeclaration, AddonTaskDeclaration, AddonTaskRequest,
    AddonTaskResponse,
};
use nako_api::{
    admin::{
        AcceptManagedArtworkCandidateResponse, AdminAccessCapabilityState, AdminAccessMode,
        AdminAccessPrincipalKind, AdminAccessSummaryResponse, AdminAccessUserListResponse,
        AdminAccessUserResponse, AdminAcquisitionIntakeCandidateListResponse,
        AdminCatalogGovernanceItemDetailResponse, AdminCatalogGovernanceItemListResponse,
        AdminCatalogGovernanceProviderMappingReviewDecision,
        AdminCatalogGovernanceProviderMappingReviewPlanResponse,
        AdminCatalogGovernanceProviderMappingReviewRequest,
        AdminCatalogGovernanceProviderMappingReviewResponse, AdminCatalogGovernanceRepairAction,
        AdminCatalogGovernanceRepairPlanStatus, AdminCreateInvitationRequest,
        AdminCreateInvitationResponse, AdminCreateUserRequest,
        AdminGeneratedArtifactProposalListResponse, AdminGeneratedArtifactReviewPlanResponse,
        AdminGeneratedArtifactReviewRequest, AdminGeneratedArtifactReviewResponse,
        AdminInvitationListResponse, AdminInvitationResponse, AdminJobCancelRequestResponse,
        AdminJobListResponse, AdminLibraryAccessLevel, AdminLibraryAccessPolicyDeleteResponse,
        AdminLibraryAccessPolicyListResponse, AdminLibraryAccessPolicyScope,
        AdminLibraryAccessReason, AdminManagedArtworkArtifactCleanupResponse,
        AdminManagedArtworkArtifactLifecycleResponse,
        AdminManagedArtworkArtifactRemediationPlanResponse,
        AdminManagedArtworkArtifactStorageDriftArtifactIssue,
        AdminManagedArtworkArtifactStorageDriftFileReason,
        AdminManagedArtworkArtifactStorageDriftResponse,
        AdminManagedArtworkArtifactStrayFileCleanupResponse,
        AdminManagedArtworkArtifactStrayFileCleanupStatus,
        AdminManagedArtworkArtifactStrayFileRemediationAction, AdminManagedArtworkGalleryResponse,
        AdminMetadataRawCacheSettingsResponse, AdminOutboxEventListResponse, AdminOverviewResponse,
        AdminOverviewStatus, AdminPlaybackReadinessCheckName, AdminPlaybackReadinessReason,
        AdminPlaybackReadinessStatus, AdminPlaybackRuntimeDiagnosticsResponse,
        AdminPlaybackRuntimeSettingsPayload, AdminPlaybackRuntimeSettingsResponse,
        AdminPlaybackRuntimeStatus, AdminPlaybackSessionListResponse,
        AdminPlaybackSupportEvidenceResponse, AdminReplaceUserRolesRequest,
        AdminServerConfigDiagnosticsResponse, AdminStorageStagingDiagnosticsResponse,
        AdminUpdateMetadataRawCacheSettingsRequest, AdminUpdatePlaybackRuntimeSettingsRequest,
        AdminUpdateUserStatusRequest, AdminUpsertLibraryAccessPolicyRequest,
        AdminWatchFolderDiscoveryRequest, AdminWatchFolderDiscoveryResponse,
        IgnoreIngestionFailureRequest, IngestionFailuresResponse, JobResponse,
        ProcessManagedArtworkIngestResponse, PublishSelectedArtworkResponse,
        RequeueManagedArtworkIngestResponse, StorageBackendDiagnosticsResponse, StorageBackendKind,
        StorageBackendRuntimeStateScope, StorageBackendStatus, UnpublishSelectedArtworkResponse,
    },
    extension::{
        AddonAccessCheckRequest, AddonAccessCheckResponse, AddonAcquisitionCandidateResponse,
        AddonEventDeliveryAttemptsResponse, AddonEventDispatchResponse, AddonEventReplayResponse,
        AddonEventSchedulerWorkResponse, AddonEventSchedulerWorkStatus,
        AddonGeneratedArtifactResponse, AddonGrantAssignment, AddonGrantsResponse,
        AddonSideEffectResponse, AddonSideEffectTargetRequest, AddonTaskRunDispatchMode,
        AddonTaskRunResponse, AddonTokenIssuedResponse, AddonTokenResponse,
        AddonTokenRotationResponse, AddonTokensResponse, AdminAddonHealthCheckResponse,
        AdminAddonHealthCheckStatus, AdminAddonInstallGuidePreviewRequest,
        AdminAddonInstallGuidePreviewResponse, AdminAddonInstallGuideResponse,
        AdminAddonLifecycleIntent, AdminAddonManagerPlanRequest, AdminAddonManagerPlanResponse,
        AdminAddonRegistrationResponse, AdminAddonRegistrationsResponse,
        AdminAddonResourceCallDiagnosticRequest, AdminAddonResourceCallDiagnosticResponse,
        AdminAddonResourceCallDiagnosticStatus, AdminAddonRoutingPlansResponse,
        AdminAddonRuntimeReadinessReason, AdminAddonRuntimeReadinessResponse,
        AdminAddonRuntimeReadinessStatus, AdminAddonSourceCatalogEntriesResponse,
        AdminAddonSourceCatalogResolveResponse, AdminAddonSourceCatalogSourceKind,
        AdminAddonSourceCatalogSourcesResponse, AdminAddonSurfacesResponse,
        AutomationArtifactsResponse, AutomationProviderResponse, AutomationProvidersResponse,
        CancelAddonTaskRunRequest, ClaimAddonTaskRunRequest, ClaimAddonTaskRunResponse,
        CompleteAddonTaskRunRequest, CreateAddonTaskRunRequest, EnqueueAutomationJobRequest,
        FailAddonTaskRunRequest, IssueAddonTokenRequest, RegisterAddonRequest,
        ReplaceAddonGrantsRequest, ReplayAddonEventRequest, ReportAddonTaskRunProgressRequest,
        RetryAddonTaskRunRequest, SubmitAddonAcquisitionCandidateRequest,
        SubmitAddonGeneratedArtifactRequest, SubmitAddonSideEffectRequest,
        UpdateAddonStatusRequest, UpsertAutomationProviderRequest, UpsertWebhookEndpointRequest,
        WebhookDeliveryAttemptsResponse, WebhookEndpointResponse, WebhookEndpointsResponse,
    },
    metadata_diagnostics::{
        EnqueueMetadataMaintenanceRequest, MetadataCandidateReviewDecisionKind,
        MetadataCandidateReviewResponse, MetadataCandidateReviewStatus,
        MetadataMaintenancePlanResponse, MetadataProviderAttemptsResponse,
        MetadataProviderDiagnosticStatus, MetadataProviderDiagnosticsResponse,
        MetadataRawCleanupResponse, MetadataRawResponsesResponse,
    },
    public_client::{
        CurrentUserResponse, ErrorResponse, HealthResponse, LibraryListResponse, LibraryResponse,
        LoginRequest, LoginResponse, LogoutResponse, ManagementContextLinkDto,
        ManagementContextLinksResponse, PLAYBACK_SESSION_ID_HEADER, PlaybackSessionResponse,
        RedeemInvitationRequest,
    },
};
use nako_core::{
    AcquisitionIntakeCandidateState, AcquisitionIntakeRepository, AddonEventDeliveryRepository,
    AddonId, AddonPermission, AddonRepository, AddonRoutingPlanStatus, AddonRoutingPlanTarget,
    AddonSideEffectApplyStatus, AddonSideEffectTargetKind, AddonSideEffectValidationStatus,
    AddonStatus, AddonTokenStatus, ArtworkCandidateRepository, ArtworkCandidateSourceKind,
    ArtworkCandidateStatus, AuthenticatedPrincipal, AutomationArtifactId, AutomationArtifactKind,
    AutomationArtifactStatus, AutomationCapability, AutomationJobInput, AutomationProviderId,
    AutomationProviderStatus, AutomationRepository, CanonicalMetadata, CatalogRepository,
    CreditRole, DomainEventKind, DomainEventSubject, EventId, EventOutboxRepository,
    ExternalProvider, GeneratedArtifactReviewDecision, Genre, GenreId, IdentityAccessRepository,
    ImageAsset, ImageAssetId, ImageKind, ImageOwner, IngestionFailureClass, IngestionFailurePhase,
    IngestionFailureRepository, IngestionFailureStatus, ItemCredit, ItemGenre, ItemTag, JobId,
    JobKind, JobRepository, JobStatus, Library, LibraryAccessLevel, LibraryAccessPolicy,
    LibraryAccessPolicyScope, LibraryId, LibraryItemRepository, LibraryItemState, LibraryOptions,
    LibraryRepository, LocalInferenceEvidence, LocalInferenceEvidenceId,
    LocalInferenceEvidenceSource, LocalInferenceRepository, LocalMetadataPolicy,
    ManagedArtworkArtifactId, ManagedArtworkIngestStatus, ManagedArtworkRepository,
    ManagedImportRepository, MediaItem, MediaItemId, MediaKind, MediaProbeRepository,
    MediaProbeResult, MediaRepository, MediaSource, MediaSourceId, MediaStreamInfo,
    MediaStreamKind, MetadataField, MetadataFieldLock, MetadataMatchKind,
    MetadataProviderAttemptId, MetadataProviderAttemptStatus, MetadataProviderErrorClass,
    MetadataRefreshMode, MetadataRepository, MetadataSource, NakoError, NewAddonRegistration,
    NewAutomationArtifact, NewAutomationProviderConfig, NewIngestionFailure, NewJob,
    NewMetadataProviderAttempt, NewOutboxEvent, NewPlaybackSession, NewStagingManifestRecord,
    NewTranscodeSession, NewVfsCacheFailure, OutboxEventStatus, PageRequest, Person, PersonId,
    PlaybackPermissionPolicy, PlaybackPolicy, PlaybackPolicyRepository, PlaybackSessionId,
    PlaybackSessionListFilter, PlaybackSessionMode, PlaybackSessionRecord,
    PlaybackSessionRepository, PlaybackSessionState, ProviderMapping, ProviderMappingId,
    ProviderMappingRepository, ProviderMappingStatus, ProviderRawResponse,
    ProviderRawResponseFilter, ProviderSubject, ProviderSubjectId, ProviderSubjectKind,
    RoleAssignment, StagingManifestId, StagingManifestRepository, StagingPurpose, StagingState,
    StorageErrorKind, Tag, TagId, TranscodeFailureCategory, TranscodeSessionId,
    TranscodeSessionKind, TranscodeSessionListFilter, TranscodeSessionRepository,
    TranscodeSessionState, User, UserId, UserPrincipalId, UserRole, UserStatus, VfsCacheOperation,
    VfsCacheRepository, VfsCachedObject, VfsCachedObjectKind, WebhookEndpointStatus,
};
use nako_db::NakoDatabase;
use nako_playback::{
    ClientPlaybackCapabilities, PlaybackPreferenceContext, PlaybackProfile,
    PlaybackSelectionContext, PlaybackStorageContext,
};
use nako_search::{SearchDocument, SearchIndex, SearchQuery};
use nako_streaming::{DirectPlayRangeRequest, RequestedByteRange, plan_direct_play_response};
use nako_transcode::{HardwareAcceleration, OutputContainer, RemuxContainer, TranscodePlan};
use nako_vfs::{ByteRange, ReadStream, StorageUri};
use serde::{Serialize, de::DeserializeOwned};
use tokio::{net::TcpListener, task::yield_now, time::sleep};
use tower::ServiceExt;

use super::error::ApiError;
use super::*;
use crate::config::{
    LocalLibraryConfig, MetadataConfig, MetadataProviderConfig, MetadataProviderHeaderConfig,
    MetadataProviderRuntimeConfig, NakoServerConfig, NetworkAccessConfig, NetworkExposureMode,
    PlaybackConfig, StagingConfig, TranscodeConfig,
};
use crate::http::playback::stream_direct_play_response;

mod addons;
mod automation;
mod catalog;
mod library;
mod management_context;
mod metadata;
mod playback;
mod renderer;
mod self_host_smoke;
mod system;
mod user_playback;
mod webhooks;

async fn router_with_media_source(
    file_name: &str,
    content: &[u8],
) -> (tempfile::TempDir, Router, MediaSource, NakoDatabase) {
    router_with_media_source_config(file_name, content, |_| {}).await
}

fn public_client_router_with_principal(app: NakoApp, principal: AuthenticatedPrincipal) -> Router {
    let principal_id = principal.principal_id.clone();

    Router::new()
        .merge(super::library::routes())
        .merge(super::catalog::routes())
        .merge(super::management_context::routes())
        .merge(super::metadata::routes())
        .merge(super::playback::routes())
        .merge(super::renderer::routes())
        .merge(super::user_playback::routes())
        .layer(Extension(principal_id))
        .layer(Extension(principal))
        .with_state(app)
}

async fn local_viewer_with_library_access(
    store: &NakoDatabase,
    library_id: LibraryId,
    access: LibraryAccessLevel,
) -> AuthenticatedPrincipal {
    local_principal_with_library_access(store, library_id, UserRole::Viewer, access).await
}

async fn local_principal_with_library_access(
    store: &NakoDatabase,
    library_id: LibraryId,
    role: UserRole,
    access: LibraryAccessLevel,
) -> AuthenticatedPrincipal {
    let user_id = UserId::new();
    let principal_id = UserPrincipalId::new(format!("local-user:{user_id}")).unwrap();
    let user = User {
        id: user_id,
        principal_id: principal_id.clone(),
        username: format!("{}-{}", role.as_str(), user_id),
        display_name: "Library principal".to_owned(),
        status: UserStatus::Active,
        created_at_ms: 1,
        updated_at_ms: 1,
    };

    store.upsert_user(&user).await.unwrap();
    store
        .replace_role_assignments(
            user_id,
            &[RoleAssignment {
                user_id,
                role,
                granted_at_ms: 1,
            }],
        )
        .await
        .unwrap();
    store
        .upsert_library_access_policy(&LibraryAccessPolicy {
            scope: LibraryAccessPolicyScope::User(user_id),
            library_id,
            access,
            created_at_ms: 1,
            updated_at_ms: 1,
        })
        .await
        .unwrap();

    AuthenticatedPrincipal {
        user_id,
        principal_id,
        roles: vec![role],
        bootstrap: false,
    }
}

async fn router_with_media_source_config(
    file_name: &str,
    content: &[u8],
    configure: impl FnOnce(&mut NakoServerConfig),
) -> (tempfile::TempDir, Router, MediaSource, NakoDatabase) {
    let (temp, app, source, store) =
        app_with_media_source_config(file_name, content, configure).await;
    let router = build_router(app);

    (temp, router, source, store)
}

async fn app_with_media_source_config(
    file_name: &str,
    content: &[u8],
    configure: impl FnOnce(&mut NakoServerConfig),
) -> (tempfile::TempDir, NakoApp, MediaSource, NakoDatabase) {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join(file_name), content).unwrap();
    let library_id = LibraryId::new();
    let mut config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: temp.path().join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig {
            artifact_root: temp.path().join("nako-cache").join("artwork"),
            ..crate::config::ArtworkConfig::default()
        },
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: temp.path().to_path_buf(),
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    configure(&mut config);
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: file_name.to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: format!("local:///{file_name}"),
        file_name: file_name.to_owned(),
        size_bytes: Some(content.len() as u64),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store
        .upsert_library_item_state(&LibraryItemState {
            library_id,
            item_id: item.id,
            provisional: false,
        })
        .await
        .unwrap();
    store.upsert_media_source(&source).await.unwrap();

    (temp, app, source, store)
}

async fn router_with_remux_source(
    slow: bool,
) -> (
    tempfile::TempDir,
    Router,
    MediaSource,
    PathBuf,
    PathBuf,
    PathBuf,
    NakoDatabase,
) {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("remux.started");
    let ffmpeg_path = fake_ffmpeg_script(temp.path(), "remux", slow, &marker);
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: ffmpeg_path.clone(),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: staging_root.clone(),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);

    (
        temp,
        router,
        source,
        staging_root,
        ffmpeg_path,
        marker,
        store,
    )
}

async fn router_with_hls_source() -> (tempfile::TempDir, Router, MediaSource, NakoDatabase) {
    let temp = tempfile::tempdir().unwrap();
    let ffmpeg_path = fake_hls_ffmpeg_script(temp.path(), "hls");
    let library_root = temp.path().join("library");
    let staging_root = temp.path().join("cache").join("remux");
    fs::create_dir_all(&library_root).unwrap();
    fs::write(library_root.join("demo.mkv"), b"media").unwrap();
    let library_id = LibraryId::new();
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path,
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: staging_root,
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root: library_root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store.clone())
        .await
        .unwrap();
    let item = MediaItem {
        id: nako_core::MediaItemId::new(),
        kind: MediaKind::Movie,
        parent_id: None,
        metadata: CanonicalMetadata {
            title: "Demo".to_owned(),
            ..CanonicalMetadata::default()
        },
    };
    let source = MediaSource {
        id: MediaSourceId::new(),
        library_id,
        item_id: item.id,
        locator: "local:///demo.mkv".to_owned(),
        file_name: "demo.mkv".to_owned(),
        size_bytes: Some(5),
        fingerprint: None,
    };
    store.upsert_media_item(&item).await.unwrap();
    store.upsert_media_source(&source).await.unwrap();
    store
        .upsert_media_probe(source.id, &compatible_probe())
        .await
        .unwrap();
    let router = build_router(app);

    (temp, router, source, store)
}

fn compatible_probe() -> MediaProbeResult {
    MediaProbeResult {
        duration_ms: Some(1_000),
        container: Some("matroska,webm".to_owned()),
        bit_rate: None,
        streams: vec![
            MediaStreamInfo {
                index: 0,
                kind: MediaStreamKind::Video,
                codec: Some("h264".to_owned()),
                language: None,
                duration_ms: None,
                bit_rate: None,
                width: Some(1920),
                height: Some(1080),
                channels: None,
                sample_rate: None,
            },
            MediaStreamInfo {
                index: 1,
                kind: MediaStreamKind::Audio,
                codec: Some("aac".to_owned()),
                language: None,
                duration_ms: None,
                bit_rate: None,
                width: None,
                height: None,
                channels: Some(2),
                sample_rate: Some(48_000),
            },
        ],
    }
}

fn local_remux_request_key(source: &MediaSource, container: RemuxContainer) -> String {
    let profile = PlaybackProfile::from_context(
        &ClientPlaybackCapabilities::default(),
        PlaybackSelectionContext {
            storage: PlaybackStorageContext {
                remote: false,
                range_readable: Some(true),
            },
            preferences: PlaybackPreferenceContext {
                remux_output_container: Some(container),
                ..Default::default()
            },
        },
    );

    profile
        .remux_transcode_profile(container)
        .identity()
        .bind_source(&nako_transcode::TranscodeSourceIdentity::from_media_source(
            source,
        ))
        .persisted_request_key()
        .to_owned()
}

fn local_hls_request_key(source: &MediaSource, acceleration: HardwareAcceleration) -> String {
    let profile = PlaybackProfile::from_context(
        &ClientPlaybackCapabilities::default(),
        PlaybackSelectionContext {
            storage: PlaybackStorageContext {
                remote: false,
                range_readable: Some(true),
            },
            preferences: PlaybackPreferenceContext {
                transcode_output_container: Some(OutputContainer::Hls),
                ..Default::default()
            },
        },
    );
    let plan = TranscodePlan {
        input_locator: "local:///demo.mkv".to_owned(),
        output_container: OutputContainer::Hls,
        video_codec: Some("h264".to_owned()),
        audio_codec: Some("aac".to_owned()),
    };

    profile
        .hls_transcode_profile(
            &plan,
            nako_transcode::TranscodeExecutionPolicy::hls_single_variant(
                nako_transcode::TranscodeAccelerationPlan::for_selected_hardware(acceleration),
                profile.track_selection(),
                nako_transcode::TranscodeOutputConstraints::default(),
            ),
        )
        .identity()
        .bind_source(&nako_transcode::TranscodeSourceIdentity::from_media_source(
            source,
        ))
        .persisted_request_key()
        .to_owned()
}

#[cfg(unix)]
fn push_unix_ffmpeg_probe_handlers(content: &mut String, encoder_lines: &[&str]) {
    content.push_str("if [ \"$1\" = \"-hide_banner\" ]; then\n");
    content.push_str("case \"$2\" in\n");
    content.push_str("-encoders)\n");
    content.push_str("cat <<'EOF'\n");
    for line in encoder_lines {
        content.push_str(line);
        content.push('\n');
    }
    content.push_str("EOF\nexit 0\n;;\n");
    content.push_str("-decoders)\n");
    content.push_str("cat <<'EOF'\n VFS..D h264\n V..... h264_qsv\nEOF\nexit 0\n;;\n");
    content.push_str("-hwaccels)\n");
    content.push_str("cat <<'EOF'\nvaapi\nqsv\nvideotoolbox\nEOF\nexit 0\n;;\n");
    content.push_str("-filters)\n");
    content.push_str("cat <<'EOF'\n ... hwupload\n ... scale_vaapi\nEOF\nexit 0\n;;\n");
    content.push_str("-bsfs)\n");
    content.push_str("cat <<'EOF'\nh264_mp4toannexb\nEOF\nexit 0\n;;\n");
    content.push_str("esac\nfi\n");
}

#[cfg(windows)]
fn push_windows_ffmpeg_probe_handlers(content: &mut String) {
    content.push_str("if \"%~1\"==\"-hide_banner\" if \"%~2\"==\"-encoders\" goto encoders\r\n");
    content.push_str("if \"%~1\"==\"-hide_banner\" if \"%~2\"==\"-decoders\" goto decoders\r\n");
    content.push_str("if \"%~1\"==\"-hide_banner\" if \"%~2\"==\"-hwaccels\" goto hwaccels\r\n");
    content.push_str("if \"%~1\"==\"-hide_banner\" if \"%~2\"==\"-filters\" goto filters\r\n");
    content.push_str("if \"%~1\"==\"-hide_banner\" if \"%~2\"==\"-bsfs\" goto bsfs\r\n");
}

#[cfg(windows)]
fn push_windows_ffmpeg_probe_labels(content: &mut String, encoder_lines: &[&str]) {
    content.push_str(":encoders\r\n");
    for line in encoder_lines {
        content.push_str(&format!("echo {line}\r\n"));
    }
    content.push_str("exit /b 0\r\n");
    content.push_str(":decoders\r\n");
    content.push_str("echo  VFS..D h264\r\n");
    content.push_str("echo  V..... h264_qsv\r\n");
    content.push_str("exit /b 0\r\n");
    content.push_str(":hwaccels\r\n");
    content.push_str("echo vaapi\r\n");
    content.push_str("echo qsv\r\n");
    content.push_str("echo videotoolbox\r\n");
    content.push_str("exit /b 0\r\n");
    content.push_str(":filters\r\n");
    content.push_str("echo  ... hwupload\r\n");
    content.push_str("echo  ... scale_vaapi\r\n");
    content.push_str("exit /b 0\r\n");
    content.push_str(":bsfs\r\n");
    content.push_str("echo h264_mp4toannexb\r\n");
    content.push_str("exit /b 0\r\n");
}

fn fake_ffmpeg_script(root: &FsPath, name: &str, slow: bool, marker: &FsPath) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        push_unix_ffmpeg_probe_handlers(
            &mut content,
            &[
                " V..... libx264",
                " A..... aac",
                " V..... h264_nvenc",
                " V..... h264_vaapi",
                " V..... h264_qsv",
            ],
        );
        content.push_str("for arg do out=\"$arg\"; done\n");
        if slow {
            content.push_str(&format!("printf started > \"{}\"\n", marker.display()));
        }
        content.push_str("printf remuxed > \"$out\"\n");
        if slow {
            content.push_str("sleep 1\n");
        }
        content.push_str("exit 0\n");
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        push_windows_ffmpeg_probe_handlers(&mut content);
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        if slow {
            content.push_str(&format!(
                "<nul set /p dummy=started>\"{}\"\r\n",
                marker.display()
            ));
        }
        content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
        if slow {
            content.push_str("ping -n 3 127.0.0.1 > nul\r\n");
        }
        content.push_str("exit /b 0\r\n");
        push_windows_ffmpeg_probe_labels(
            &mut content,
            &[
                " V..... libx264",
                " A..... aac",
                " V..... h264_nvenc",
                " V..... h264_vaapi",
                " V..... h264_qsv",
            ],
        );
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_ffmpeg_encoder_script(root: &FsPath, name: &str, encoder_lines: &[&str]) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        push_unix_ffmpeg_probe_handlers(&mut content, encoder_lines);
        content.push_str("for arg do out=\"$arg\"; done\n");
        content.push_str("printf remuxed > \"$out\"\n");
        content.push_str("exit 0\n");
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        push_windows_ffmpeg_probe_handlers(&mut content);
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("<nul set /p dummy=remuxed>\"%out%\"\r\n");
        content.push_str("exit /b 0\r\n");
        push_windows_ffmpeg_probe_labels(&mut content, encoder_lines);
        fs::write(&path, content).unwrap();
        path
    }
}

fn fake_hls_ffmpeg_script(root: &FsPath, name: &str) -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let path = root.join(name);
        let mut content = String::from("#!/bin/sh\n");
        push_unix_ffmpeg_probe_handlers(
            &mut content,
            &[
                " V..... libx264",
                " A..... aac",
                " V..... h264_nvenc",
                " V..... h264_vaapi",
                " V..... h264_qsv",
            ],
        );
        content.push_str("for arg do out=\"$arg\"; done\n");
        content.push_str("dir=$(dirname \"$out\")\n");
        content.push_str("mkdir -p \"$dir\"\n");
        content.push_str(
            "printf '#EXTM3U\\n#EXTINF:1,\\nsegment_00000.ts\\n#EXT-X-ENDLIST\\n' > \"$out\"\n",
        );
        content.push_str("printf segment > \"$dir/segment_00000.ts\"\n");
        content.push_str("exit 0\n");
        fs::write(&path, content).unwrap();
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
        path
    }

    #[cfg(windows)]
    {
        let path = root.join(format!("{name}.cmd"));
        let mut content = String::from("@echo off\r\n");
        push_windows_ffmpeg_probe_handlers(&mut content);
        content.push_str("setlocal enabledelayedexpansion\r\n");
        content.push_str(":args\r\n");
        content.push_str("if \"%~1\"==\"\" goto run\r\n");
        content.push_str("set out=%~1\r\n");
        content.push_str("shift\r\n");
        content.push_str("goto args\r\n");
        content.push_str(":run\r\n");
        content.push_str("for %%I in (\"%out%\") do set dir=%%~dpI\r\n");
        content.push_str("if not exist \"%dir%\" mkdir \"%dir%\"\r\n");
        content.push_str(">\"%out%\" echo #EXTM3U\r\n");
        content.push_str(">>\"%out%\" echo #EXTINF:1,\r\n");
        content.push_str(">>\"%out%\" echo segment_00000.ts\r\n");
        content.push_str(">>\"%out%\" echo #EXT-X-ENDLIST\r\n");
        content.push_str("<nul set /p dummy=segment>\"%dir%segment_00000.ts\"\r\n");
        content.push_str("exit /b 0\r\n");
        push_windows_ffmpeg_probe_labels(
            &mut content,
            &[
                " V..... libx264",
                " A..... aac",
                " V..... h264_nvenc",
                " V..... h264_vaapi",
                " V..... h264_qsv",
            ],
        );
        fs::write(&path, content).unwrap();
        path
    }
}

async fn test_router(root: PathBuf, library_id: LibraryId) -> Router {
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network: crate::config::NetworkAccessConfig::default(),
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: root.join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    build_router(app)
}

async fn test_router_with_bearer_auth(root: PathBuf, library_id: LibraryId, token: &str) -> Router {
    test_router_with_bearer_auth_and_network(
        root,
        library_id,
        token,
        NetworkAccessConfig::default(),
    )
    .await
}

async fn test_router_with_bearer_auth_and_network(
    root: PathBuf,
    library_id: LibraryId,
    token: &str,
    network: NetworkAccessConfig,
) -> Router {
    let config = NakoServerConfig {
        database_backend: Default::default(),
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        database_url: "sqlite::memory:".to_owned(),
        database_url_env: None,
        auth: crate::config::AuthConfig::disabled(),
        network,
        ffprobe_path: PathBuf::from("ffprobe"),
        ffmpeg_path: PathBuf::from("ffmpeg"),
        scan_concurrency: 1,
        probe_concurrency: 1,
        metadata_concurrency: 1,
        remux_concurrency: 1,
        webhook_concurrency: 2,
        addon_event_scheduler: crate::config::AddonEventSchedulerConfig::default(),
        remux_timeout_ms: 30 * 60 * 1_000,
        remux_staging_root: root.join("nako-cache").join("remux"),
        metadata: MetadataConfig::default(),
        transcode: TranscodeConfig::default(),
        staging: StagingConfig::default(),
        playback: PlaybackConfig::default(),
        artwork: crate::config::ArtworkConfig::default(),
        libraries: vec![LocalLibraryConfig {
            id: library_id,
            name: "Movies".to_owned(),
            root,
            preset: nako_core::LibraryPreset::Movies,
            webdav: None,
        }],
    };
    let store = NakoDatabase::connect_in_memory().await.unwrap();
    let app = NakoApp::new_with_store(config, store).await.unwrap();
    build_router_with_auth(app, auth::InboundAuthState::bearer_token(token))
}

fn addon_manifest() -> AddonManifest {
    AddonManifest {
        id: "example.metadata".to_owned(),
        name: "Example Metadata".to_owned(),
        version: "0.1.0".to_owned(),
        protocol_version: ADDON_PROTOCOL_VERSION.to_owned(),
        base_url: "https://example.test/addon".to_owned(),
        description: Some("Metadata suggestion addon".to_owned()),
        resources: vec![AddonResourceDeclaration {
            kind: AddonResource::Metadata,
            path: "/metadata".to_owned(),
            input_schema: Some("nako.metadata.request.v1".to_owned()),
            output_schema: Some("nako.metadata.response.v1".to_owned()),
            required_scopes: vec![
                AddonScope::ItemMetadataRead,
                AddonScope::ItemMetadataSuggest,
            ],
            timeout_ms: Some(5_000),
            max_attempts: Some(2),
        }],
        entry_points: Vec::new(),
        hosted_pages: Vec::new(),
        configuration_schema: None,
        secret_reference_fields: Vec::new(),
        event_subscriptions: Vec::new(),
        tasks: Vec::new(),
        auth: AddonAuth::Bearer,
        default_timeout_ms: Some(10_000),
        default_max_attempts: Some(2),
        scopes: vec![
            AddonScope::ItemMetadataRead,
            AddonScope::ItemMetadataSuggest,
        ],
    }
}

async fn post_addon_registration(
    router: &Router,
    request: RegisterAddonRequest,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/admin/v1/addons")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn post_legacy_addon_registration(
    router: &Router,
    request: RegisterAddonRequest,
) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/addons")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(&request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn response_for(router: &Router, method: Method, uri: &str) -> axum::response::Response {
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn response_for_body_json<B>(
    router: &Router,
    method: Method,
    uri: &str,
    body: &B,
) -> axum::response::Response
where
    B: Serialize,
{
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn request_json<T>(router: &Router, method: Method, uri: &str) -> T
where
    T: DeserializeOwned,
{
    let response = response_for(router, method, uri).await;

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn request_json_with_bearer<T>(
    router: &Router,
    method: Method,
    uri: &str,
    bearer_token: &str,
) -> T
where
    T: DeserializeOwned,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn request_body_json<T, B>(router: &Router, method: Method, uri: &str, body: &B) -> T
where
    T: DeserializeOwned,
    B: Serialize,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn request_body_json_with_bearer<T, B>(
    router: &Router,
    method: Method,
    uri: &str,
    body: &B,
    bearer_token: &str,
) -> T
where
    T: DeserializeOwned,
    B: Serialize,
{
    let response = router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, format!("Bearer {bearer_token}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    body_json(response).await
}

async fn response_body_json<B>(router: &Router, method: Method, uri: &str, body: &B) -> Response
where
    B: Serialize,
{
    router
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(serde_json::to_vec(body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap()
}

async fn body_json<T>(response: axum::response::Response) -> T
where
    T: DeserializeOwned,
{
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

async fn response_text(response: axum::response::Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

struct MockWebDavServer {
    addr: std::net::SocketAddr,
}

impl MockWebDavServer {
    async fn start() -> Self {
        let router = Router::new().route("/{*path}", axum::routing::any(mock_webdav_handler));
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        Self { addr }
    }

    fn base_url(&self) -> String {
        format!("http://{}/dav", self.addr)
    }
}

async fn mock_webdav_handler(method: Method, uri: axum::http::Uri) -> Response {
    let path = uri.path();
    if method.as_str() == "PROPFIND" && path.ends_with("/Movies/Demo.mkv") {
        return (
                StatusCode::MULTI_STATUS,
                [(header::CONTENT_TYPE, "application/xml")],
                r#"<?xml version="1.0" encoding="utf-8"?><D:multistatus xmlns:D="DAV:"><D:response><D:href>/dav/Movies/Demo.mkv</D:href><D:propstat><D:prop><D:getcontentlength>4</D:getcontentlength><D:getetag>"etag-demo"</D:getetag></D:prop><D:status>HTTP/1.1 200 OK</D:status></D:propstat></D:response></D:multistatus>"#,
            )
                .into_response();
    }

    if method == Method::GET && path.ends_with("/Movies/Demo.mkv") {
        return (StatusCode::OK, [(header::CONTENT_LENGTH, "4")], "demo").into_response();
    }

    StatusCode::NOT_FOUND.into_response()
}
