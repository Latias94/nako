import type {
  AdminArtworkKind,
  AdminItemArtworkGalleryQuery,
  AdminNetworkAccessDiagnostics,
  AdminOverviewResponse,
  AdminCatalogGovernanceProviderMappingReviewRequest,
  AdminProviderMappingStatus,
  AdminGeneratedArtifactReviewRequest,
  AdminLibraryMetadataProfileResponse,
  AdminServerConfigDiagnosticsResponse,
  AddonPermission,
  AddonResource,
  AdminAddonManifest,
  PageInfo,
} from "./generated/contract";

export type {
  AddonGrantsResponse,
  AddonPermission,
  AddonResource,
  AddonScope,
  AddonStatus,
  AddonTokensResponse,
  AdminAcquisitionIntakeCandidateDiagnostic,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AdminAddonRoutingPlansResponse,
  AdminAddonHealthCheckResponse,
  AdminAddonInstallGuideResponse,
  AdminAddonManifest,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationSummary,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticRequest,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAddonResourceCallDiagnosticStatus,
  AdminAddonSurfacesResponse,
  AdminAddonsQuery,
  AdminAccessSummaryResponse,
  AdminArtworkKind,
  AdminCatalogGovernanceItem,
  AdminCatalogGovernanceItemDetailResponse,
  AdminCatalogGovernanceItemListResponse,
  AdminCatalogGovernanceItemsQuery,
  AdminCatalogGovernanceProviderMappingReviewDecision,
  AdminCatalogGovernanceProviderMappingReviewPlanResponse,
  AdminCatalogGovernanceProviderMappingReviewRequest,
  AdminCatalogGovernanceProviderMappingReviewResponse,
  AdminGeneratedArtifactProposal,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminGeneratedArtifactReviewPlanResponse,
  AdminGeneratedArtifactReviewRequest,
  AdminGeneratedArtifactReviewResponse,
  AdminItemArtworkGalleryQuery,
  AdminJobPriority,
  AdminJobListItem,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminJobCommandResponse,
  AdminLocalInferenceSummary,
  AdminMetadataProfile,
  AdminMetadataRawCacheSettingsResponse,
  AdminMetadataRefreshMode,
  AdminLocalMetadataPolicy,
  AdminMetadataScanPolicy,
  AdminNetworkAccessDiagnostics,
  AdminOutboxEventListItem,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminOverviewStatus,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionsQuery,
  AdminPlaybackSessionListItem,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminProviderMappingStatus,
  AdminServerConfigDiagnosticsResponse,
  AdminSourceDuplicateEvidenceKind,
  AdminSourceDuplicateReconciliationAction,
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationCandidate,
  AdminSourceDuplicateReconciliationPlanQuery,
  AdminSourceDuplicateReconciliationPlanResponse,
  AdminSourceDuplicateRelationshipStatus,
  AdminStorageStagingQuery,
  AdminStorageStagingDiagnosticsResponse,
  AdminVfsCacheRefreshResponse,
  AdminVfsCacheRepairActionPlanResponse,
  AdminVfsCacheRepairEnqueueRequest,
  AdminVfsCacheRepairEnqueueResponse,
  AdminVfsCacheRepairExecuteResponse,
  AdminVfsCacheRepairRemediationPlanResponse,
  AdminVfsCacheRepairRetryRequest,
  AdminVfsCacheRepairTargetListResponse,
  AdminVfsCacheRepairTargetPreviewResponse,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
  AdminUpdateMetadataRawCacheSettingsRequest,
  PageInfo,
} from "./generated/contract";

export type DataSourceMode = "live" | "hybrid" | "mock" | "planned";

export type AdminSectionKey =
  | "overview"
  | "addons"
  | "addonHealth"
  | "addonSurfaces"
  | "addonInstallGuide"
  | "addonTokens"
  | "addonGrants"
  | "acquisitionIntake"
  | "generatedArtifactProposals"
  | "catalogGovernance"
  | "events"
  | "jobs"
  | "playbackSessions"
  | "playbackRuntime"
  | "storageStaging"
  | "systemConfig";

export type AdminSourceMap = Record<AdminSectionKey, DataSourceMode>;

export type AdminErrorMap = Partial<Record<AdminSectionKey, string>>;

export type AdminConsoleData = {
  sources: AdminSourceMap;
  errors: AdminErrorMap;
  overview: AdminOverviewResponse;
  addons: AddonOperationsSummary;
  libraries: LibraryRow[];
  catalog: CatalogGovernanceSummary;
  acquisitionIntake: IntakeSummary;
  generatedArtifactProposals: GeneratedArtifactProposalSummary;
  events: EventSummary;
  jobs: JobRow[];
  playback: PlaybackSummary;
  storage: StorageSummary;
  network: NetworkSummary;
  settings: SettingRow[];
};

export type LibraryRow = {
  id: string;
  name: string;
  backendKind: string;
  status: "ready" | "degraded" | "unavailable";
  itemCount: number;
  lastScan: string;
};

export type LibraryConfigDiagnostics = AdminServerConfigDiagnosticsResponse["libraries"][number];

export type LibraryManagementDetail = {
  configuredLibraryCount: number;
  library: LibraryConfigDiagnostics | null;
  metadataProfile: AdminLibraryMetadataProfileResponse;
  sourceInventory: LibrarySourceInventorySummary;
};

export type LibrarySourceInventorySummary = {
  source: DataSourceMode;
  error?: string;
  sourceCount: number;
  linkedItemCount: number;
  probedSourceCount: number;
  returnedSourceCount: number;
  totalSizeBytes: number | null;
  latestScanJob: LibraryJobSummary | null;
  failedJobCount: number;
  page: PageInfo;
  samples: LibrarySourceSample[];
};

export type LibrarySourceSample = {
  id: string;
  fileName: string;
  itemTitle: string | null;
  sizeBytes: number | null;
  hasProbe: boolean;
};

export type PublicCatalogItemsQuery = {
  limit?: number;
  offset?: number;
};

export type PublicCatalogSearchQuery = PublicCatalogItemsQuery & {
  q?: string;
  facet?: string;
};

export type PublicCatalogItemsResponse = {
  items: PublicCatalogMediaItem[];
  page: PageInfo;
};

export type PublicCatalogSearchResponse = {
  hits: PublicCatalogSearchHit[];
  page: PageInfo;
};

export type PublicCatalogSearchHit = {
  item: PublicCatalogMediaItem;
  score: number;
};

export type PublicCatalogMediaItem = {
  id: string;
  kind: string;
  parent_id: string | null;
  metadata: PublicCanonicalMetadata;
};

export type PublicCanonicalMetadata = {
  title: string;
  original_title: string | null;
  sort_title: string | null;
  overview: string | null;
  release_date: string | null;
  runtime_minutes: number | null;
  tagline: string | null;
  genres: string[];
  tags: string[];
  ratings: Array<{
    source: string;
    value: string;
  }>;
  credits: Array<{
    name: string;
    role: string;
    character: string | null;
    order: number | null;
    external_ids: Array<{
      provider: string;
      value: string;
    }>;
  }>;
  collections: Array<{
    name: string;
    overview: string | null;
    sort_order: number | null;
    external_ids: Array<{
      provider: string;
      value: string;
    }>;
  }>;
  studios: Array<{
    name: string;
    external_ids: Array<{
      provider: string;
      value: string;
    }>;
  }>;
  external_ids: Array<{
    provider: string;
    value: string;
  }>;
};

export type PublicMediaSource = {
  id: string;
  library_id: string;
  item_id: string;
  file_name: string;
  size_bytes: number | null;
  fingerprint: string | null;
};

export type PublicItemDetailResponse = {
  item: PublicCatalogMediaItem;
  sources: PublicMediaSource[];
  credits: Array<{
    item_id: string;
    person_id: string;
    role: string;
    character: string | null;
    sort_order: number | null;
  }>;
  genres: Array<{
    item_id: string;
    genre_id: string;
  }>;
  tags: Array<{
    item_id: string;
    tag_id: string;
  }>;
  collections: Array<{
    collection_id: string;
    item_id: string;
    sort_order: number | null;
  }>;
  studios: Array<{
    item_id: string;
    studio_id: string;
  }>;
  images: PublicImageRef[];
};

export type PublicItemCreditsResponse = {
  item_id: string;
  credits: PublicItemDetailResponse["credits"];
  people: Array<{
    id: string;
    name: string;
    sort_name: string | null;
    overview: string | null;
    external_ids: Array<{
      provider: string;
      value: string;
    }>;
  }>;
};

export type PublicItemImagesResponse = {
  item_id: string;
  images: PublicImageRef[];
};

export type PublicImageRef = {
  id: string;
  owner: Record<string, unknown>;
  kind: string;
  url: string;
  width: number | null;
  height: number | null;
  language: string | null;
  media_type: string | null;
  etag: string | null;
};

export type PublicSourceProbeResponse = {
  source_id: string;
  probe: PublicMediaProbe;
};

export type PublicMediaProbe = {
  duration_ms: number | null;
  container: string | null;
  bit_rate: number | null;
  streams: PublicMediaStream[];
};

export type PublicMediaStream = {
  index: number;
  kind: string;
  codec: string | null;
  language: string | null;
  duration_ms: number | null;
  bit_rate: number | null;
  width: number | null;
  height: number | null;
  channels: number | null;
  sample_rate: number | null;
};

export type CatalogBrowseQuery = {
  q?: string;
  facet?: string;
  limit?: number;
  offset?: number;
};

export type CatalogBrowseSummary = {
  mode: "browse" | "search";
  items: CatalogBrowseItemSummary[];
  page: PageInfo;
};

export type CatalogBrowseItemSummary = {
  id: string;
  parentId: string | null;
  title: string;
  kind: string;
  releaseDate: string | null;
  runtimeMinutes: number | null;
  genreCount: number;
  tagCount: number;
  creditCount: number;
  collectionCount: number;
  studioCount: number;
  imageCount: number | null;
  sourceCount: number | null;
  score: number | null;
};

export type ItemDetailSummary = {
  item: {
    id: string;
    parentId: string | null;
    title: string;
    kind: string;
    releaseDate: string | null;
    runtimeMinutes: number | null;
    genreCount: number;
    tagCount: number;
    creditCount: number;
    collectionCount: number;
    studioCount: number;
    imageCount: number;
    sourceCount: number;
  };
  canonical: {
    genres: string[];
    tags: string[];
    credits: Array<{
      name: string;
      role: string;
      character: string | null;
    }>;
    collections: string[];
    studios: string[];
    ratingCount: number;
    externalIdCount: number;
  };
  sources: ItemSourceSummary[];
  images: ItemImageSummary[];
  readiness: ItemReadinessSummary[];
};

export type ItemSourceSummary = {
  id: string;
  libraryId: string;
  fileName: string;
  sizeBytes: number | null;
  hasFingerprint: boolean;
  probe: ItemSourceProbeSummary | null;
};

export type ItemSourceProbeSummary = {
  durationMs: number | null;
  container: string | null;
  bitRate: number | null;
  streamCount: number;
  videoStreamCount: number;
  audioStreamCount: number;
  subtitleStreamCount: number;
};

export type ItemImageSummary = {
  id: string;
  kind: string;
  routePath: string | null;
  width: number | null;
  height: number | null;
  language: string | null;
  mediaType: string | null;
  hasEtag: boolean;
};

export type ItemReadinessSummary = {
  label: string;
  status: "ready" | "planned" | "split";
  detail: string;
};

export type ItemArtworkGalleryQuery = AdminItemArtworkGalleryQuery;

export type ItemArtworkGallerySummary = {
  itemId: string;
  totals: {
    candidateCount: number;
    artifactCount: number;
    selectedCount: number;
  };
  candidates: ItemArtworkCandidateSummary[];
  artifacts: ItemArtworkArtifactSummary[];
  selected: ItemArtworkSelectedSummary[];
  page: PageInfo;
};

export type ItemArtworkCandidateSummary = {
  id: string;
  addonId: string;
  sideEffectId: string;
  libraryId: string;
  itemId: string;
  kind: AdminArtworkKind | string;
  sourceKind: string;
  status: string;
  width: number | null;
  height: number | null;
  language: string | null;
  ingestId: string | null;
  ingestStatus: string | null;
  hasIngestArtifact: boolean;
  hasIngestFailure: boolean;
  ingestFailureCode: string | null;
  artifactId: string | null;
  hasStoredArtifact: boolean;
  selectedArtworkCount: number;
  selected: boolean;
  updatedAt: string;
};

export type ItemArtworkArtifactSummary = {
  id: string;
  ingestId: string;
  candidateId: string;
  libraryId: string;
  itemId: string;
  kind: AdminArtworkKind | string;
  selectedArtworkCount: number;
  selected: boolean;
  width: number | null;
  height: number | null;
  byteLen: number | null;
  mediaType: string | null;
  hasContentHash: boolean;
  updatedAt: string;
};

export type ItemArtworkSelectedSummary = {
  selectedArtworkId: string;
  libraryId: string;
  itemId: string;
  kind: AdminArtworkKind | string;
  artifactId: string;
  imageId: string;
  routePath: string | null;
  width: number | null;
  height: number | null;
  language: string | null;
  mediaType: string | null;
  selectedAt: string;
  updatedAt: string;
};

export type ItemArtworkMutationResultSummary = {
  action: "select" | "unpublish";
  itemId: string;
  kind: AdminArtworkKind | string;
  changed: boolean;
  selectedArtworkId: string | null;
  artifactId: string | null;
  imageId: string | null;
  routePath: string | null;
  width: number | null;
  height: number | null;
  language: string | null;
  mediaType: string | null;
};

export type LibraryJobSummary = {
  id: string;
  kind: string;
  status: string;
  resourceClass: string;
  queuedAt: string;
  completedAt: string | null;
  hasError: boolean;
};

export type LibraryCommandAction = "scan" | "nfoImport" | "nfoExport";

export type LibraryCommandResult = {
  action: LibraryCommandAction;
  job: LibraryJobSummary;
};

export type PublicLibrarySourcesResponse = {
  library: {
    id: string;
    name: string;
  };
  sources: PublicLibrarySourceRecord[];
  page: PageInfo;
};

export type PublicLibrarySourceRecord = {
  source: {
    id: string;
    library_id: string;
    item_id: string;
    file_name: string;
    size_bytes: number | null;
    fingerprint: string | null;
  };
  item: {
    id: string;
    title?: string | null;
    kind?: string;
  } | null;
  probe: {
    duration_ms?: number | null;
    container?: string | null;
    streams?: unknown[];
  } | null;
};

export type AddonOperationsSummary = {
  selectedAddonId: string | null;
  addons: AddonRow[];
  selectedAddon: AddonDetail | null;
  health: AddonHealthSummary | null;
  surfaces: AddonSurfaceSummary | null;
  installGuide: AddonInstallGuideSummary | null;
  tokens: AddonTokenSummaryRow[];
  grants: AddonGrantSummaryRow[];
  diagnostic: AddonDiagnosticSummary | null;
};

export type AddonsRouteSummary = {
  addons: AddonsRouteRow[];
  selectedAddon: AddonsRouteDetail | null;
  statusCounts: {
    enabled: number;
    disabled: number;
    unregistered: number;
  };
  health: AddonHealthSummary | null;
  surfaceSummary: AddonsRouteSurfaceSummary | null;
  installBoundary: AddonsRouteInstallBoundary | null;
  tokens: AddonTokenSummaryRow[];
  grants: AddonGrantSummaryRow[];
};

export type AddonsRouteRow = {
  id: string;
  manifestId: string;
  name: string;
  version: string;
  protocolVersion: string;
  status: string;
  grantedScopeCount: number;
  updatedAt: string;
};

export type AddonsRouteDetail = AddonsRouteRow & {
  resourceCount: number;
  resourceKinds: AddonResource[];
  authMode: string;
  grantedScopes: string[];
  defaultTimeoutMs: number | null;
  defaultMaxAttempts: number | null;
};

export type AddonsRouteSurfaceSummary = {
  entryPointCount: number;
  hostedPageCount: number;
  configurationSchemaDeclared: boolean;
  secretReferenceFieldCount: number;
  taskCount: number;
  eventSubscriptionCount: number;
};

export type AddonsRouteInstallBoundary = {
  nakoManagesContainers: boolean;
  nakoManagesProcesses: boolean;
  nakoManagesPackages: boolean;
  secretReferenceCount: number;
  healthCheckStepCount: number;
  registrationVerificationStepCount: number;
};

export type AddonOnboardingResult =
  | {
      status: "registered";
      addon: AddonOnboardingRegistrationSummary;
      nextSteps: string[];
    }
  | {
      status: "invalid_json" | "server_error";
      error: string;
    };

export type AddonOnboardingRegistrationSummary = {
  id: string;
  manifestId: string;
  name: string;
  version: string;
  protocolVersion: string;
  baseUrl: string;
  status: string;
  resourceCount: number;
  grantedScopes: string[];
};

export type AddonManifestPreview = {
  status: "ready" | "invalid_json";
  manifest?: AdminAddonManifest;
  error?: string;
  summary?: {
    manifestId: string;
    name: string;
    version: string;
    protocolVersion: string;
    baseUrl: string;
    resourceCount: number;
    declaredScopes: string[];
    secretReferenceCount: number;
  };
};

export type AddonRow = {
  id: string;
  manifestId: string;
  name: string;
  version: string;
  protocolVersion: string;
  baseUrl: string;
  status: string;
  grantedScopes: string[];
  updatedAt: string;
};

export type AddonDetail = AddonRow & {
  description: string | null;
  resourceCount: number;
  resourceKinds: AddonResource[];
  authMode: string;
  defaultTimeoutMs: number | null;
  defaultMaxAttempts: number | null;
};

export type AddonHealthSummary = {
  addonId: string;
  status: string;
  latencyMs: number;
  protocolVersion: string | null;
  addonVersion: string | null;
  resourceCount: number | null;
  safeErrorCode: string | null;
};

export type AddonSurfaceSummary = {
  entryPoints: Array<{
    id: string;
    label: string;
    kind: string;
    path: string;
    hostedPageId: string | null;
  }>;
  hostedPages: Array<{
    id: string;
    title: string;
    path: string;
    url: string;
  }>;
  configurationSchemaId: string | null;
  secretReferenceFieldCount: number;
  tasks: Array<{
    id: string;
    name: string;
    path: string;
  }>;
  eventSubscriptions: Array<{
    id: string;
    eventKind: string;
    path: string;
  }>;
};

export type AddonTokenSummaryRow = {
  id: string;
  label: string;
  tokenPrefix: string;
  status: string;
  lastUsedAt: string | null;
};

export type AddonGrantSummaryRow = {
  id: string;
  permission: string;
  libraryId: string | null;
};

export type AddonGrantAssignmentInput = {
  permission: AddonPermission;
  libraryId: string | null;
};

export type AddonTokenActionResult = {
  token: AddonTokenSummaryRow;
  rawToken: string;
};

export type AddonDiagnosticSummary = {
  addonId: string;
  resource: AddonResource;
  status: string;
  latencyMs: number;
  attempts: number;
  httpStatus: number | null;
  safeErrorCode: string | null;
};

export type AddonInstallGuideSummary = {
  addonId: string;
  manifestId: string;
  addonName: string;
  addonVersion: string;
  protocolVersion: string;
  baseUrl: string;
  status: string;
  dockerCompose: AddonInstallGuideSnippetSummary;
  systemd: AddonInstallGuideSnippetSummary;
  secretReferences: AddonInstallGuideSecretReferenceSummary[];
  healthCheckSteps: AddonInstallGuideStepSummary[];
  registrationVerificationSteps: AddonInstallGuideStepSummary[];
  lifecycleBoundary: {
    nakoManagesContainers: boolean;
    nakoManagesProcesses: boolean;
    nakoManagesPackages: boolean;
    message: string;
  };
};

export type AddonInstallGuideSnippetSummary = {
  title: string;
  filename: string;
  content: string;
  notes: string[];
};

export type AddonInstallGuideSecretReferenceSummary = {
  id: string;
  label: string;
  description: string | null;
  required: boolean;
  envVar: string;
  placeholder: string;
};

export type AddonInstallGuideStepSummary = {
  title: string;
  command: string;
  expectedResult: string;
};

export type CatalogGovernanceSummary = {
  items: CatalogGovernanceItemSummary[];
  page: PageInfo;
};

export type CatalogGovernanceItemSummary = {
  id: string;
  libraryId: string;
  kind: string;
  parentId: string | null;
  title: string;
  releaseDate: string | null;
  issues: string[];
  sourceCount: number;
  representativeSourceId: string | null;
  representativeFileName: string | null;
  providerMappingCount: number;
  acceptedProviderMappingCount: number;
  duplicateRelationshipCount: number;
  localInference: {
    sourceId: string;
    inferredKind: string;
    inferredTitle: string | null;
    inferredYear: number | null;
    inferredSeason: number | null;
    inferredEpisode: number | null;
    confidenceMilli: number | null;
    evidenceSource: string;
    hasEvidence: boolean;
    inferenceVersion: string;
  } | null;
};

export type CatalogGovernanceItemDetailSummary = {
  item: CatalogGovernanceItemSummary;
  providerMappings: CatalogGovernanceProviderMappingSummary[];
  repairActions: string[];
};

export type CatalogGovernanceProviderMappingSummary = {
  id: string;
  itemId: string;
  status: AdminProviderMappingStatus;
  confidenceMilli: number | null;
  source: string;
  subject: {
    id: string;
    provider: string;
    kind: string;
    key: string;
    title: string | null;
    releaseYear: number | null;
    locale: string | null;
  };
};

export type CatalogGovernanceProviderMappingReviewDecision =
  AdminCatalogGovernanceProviderMappingReviewRequest["decision"];

export type CatalogGovernanceProviderMappingReviewPlanSummary = {
  item: CatalogGovernanceItemSummary;
  mapping: CatalogGovernanceProviderMappingSummary;
  decision: CatalogGovernanceProviderMappingReviewDecision;
  currentStatus: AdminProviderMappingStatus;
  targetStatus: AdminProviderMappingStatus;
  status: string;
  readiness: {
    status: string;
    actionable: boolean;
    reasons: string[];
  };
  boundary: {
    updatesProviderMappingStatus: boolean;
    updatesCanonicalMetadata: boolean;
    updatesProviderSubject: boolean;
    updatesLocalInference: boolean;
    updatesSourceDuplicates: boolean;
    updatesHierarchy: boolean;
    writesNfo: boolean;
    writesLibraryFiles: boolean;
    updatesArtwork: boolean;
    updatesPlaybackState: boolean;
  };
};

export type CatalogGovernanceProviderMappingReviewResultSummary = {
  itemId: string;
  mappingId: string;
  decision: CatalogGovernanceProviderMappingReviewDecision;
  previousStatus: AdminProviderMappingStatus;
  currentStatus: AdminProviderMappingStatus;
  changed: boolean;
  idempotentReplay: boolean;
  plan: CatalogGovernanceProviderMappingReviewPlanSummary;
};

export type EventSummary = {
  events: Array<{
    id: string;
    kind: string;
    status: string;
    attempts: number;
    hasError: boolean;
  }>;
  page: PageInfo;
};

export type IntakeSummary = {
  candidates: Array<{
    id: string;
    sourceKind: string;
    sourceScheme: string;
    state: string;
    sizeBytes: number | null;
    hasDiagnostics: boolean;
    linkedArtifactId: string | null;
  }>;
  page: PageInfo;
};

export type GeneratedArtifactProposalSummary = {
  proposals: Array<{
    id: string;
    capability: string;
    kind: string;
    status: string;
    targetKind: string;
    readinessStatus: string;
    actionable: boolean;
    confidenceMilli: number | null;
    payloadShape: string;
    providerName: string | null;
    promptFingerprint: string | null;
    payloadFingerprint: string;
  }>;
  page: PageInfo;
};

export type GeneratedArtifactReviewDecision = AdminGeneratedArtifactReviewRequest["decision"];

export type GeneratedArtifactReviewPlanSummary = {
  artifactId: string;
  decision: GeneratedArtifactReviewDecision;
  status: string;
  action: string;
  reasons: string[];
  capability: string;
  kind: string;
  target: {
    kind: string;
    libraryId: string | null;
    itemId: string | null;
    sourceId: string | null;
  };
  payload: {
    validJson: boolean;
    shape: string;
    payloadFingerprint: string;
    payloadBytes: number;
    objectFieldCount: number | null;
    arrayItemCount: number | null;
    hasTextualValues: boolean;
    hasExplanation: boolean;
    confidenceMilli: number | null;
  };
  readiness: {
    status: string;
    actionable: boolean;
    reasons: string[];
  };
  boundary: {
    acceptedIntoCanonicalMetadata: boolean;
    writesSidecar: boolean;
    writesLibraryFiles: boolean;
    appliesImmediately: boolean;
    requiresMetadataAuthorityApply: boolean;
  };
};

export type GeneratedArtifactReviewResultSummary = {
  artifactId: string;
  decision: GeneratedArtifactReviewDecision;
  artifactStatus: string;
  acceptedAt: string | null;
  idempotentReplay: boolean;
  plan: GeneratedArtifactReviewPlanSummary;
};

export type JobRow = {
  id: string;
  kind: string;
  status: string;
  resourceClass: string;
  hasError: boolean;
};

export type PlaybackSummary = {
  hardwarePolicy: string;
  ffmpegStatus: string;
  accelerators: Array<{
    name: string;
    available: boolean;
  }>;
  sessions: Array<{
    id: string;
    kind: string;
    sourceTitle: string;
    state: string;
  }>;
};

export type StorageSummary = {
  stagingUsedBytes: number;
  stagingMaxBytes: number;
  vfsObjectCount: number;
  records: Array<{
    id: string;
    sourceScheme: string;
    purpose: string;
    state: string;
    sizeBytes: number | null;
    hasValidationError: boolean;
  }>;
};

export type NetworkSummary = {
  exposureMode: AdminNetworkAccessDiagnostics["exposure_mode"];
  readinessStatus: AdminNetworkAccessDiagnostics["readiness"]["status"];
  readinessReason: AdminNetworkAccessDiagnostics["readiness"]["reason"];
  endpointConfigured: boolean;
  endpointScheme: string | null;
  trustedProxyHeaders: boolean;
  trustedProxySourceCount: number;
  allowedOriginCount: number;
  tunnelProviderCount: number;
};

export type SettingRow = {
  label: string;
  value: string;
};
