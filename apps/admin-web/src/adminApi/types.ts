import type {
  AdminNetworkAccessDiagnostics,
  AdminOverviewResponse,
  PageInfo,
} from "./generated/contract";

export type {
  AdminAcquisitionIntakeCandidateDiagnostic,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
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
