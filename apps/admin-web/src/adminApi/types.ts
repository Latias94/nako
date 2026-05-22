import type {
  AdminNetworkAccessDiagnostics,
  AdminOverviewResponse,
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
  AdminCatalogGovernanceItem,
  AdminCatalogGovernanceItemListResponse,
  AdminGeneratedArtifactProposal,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminJobListItem,
  AdminJobListResponse,
  AdminLocalInferenceSummary,
  AdminNetworkAccessDiagnostics,
  AdminOutboxEventListItem,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminOverviewStatus,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListItem,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
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
  items: Array<{
    id: string;
    title: string;
    kind: string;
    issues: string[];
    sourceCount: number;
    providerMappingCount: number;
  }>;
  page: PageInfo;
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
