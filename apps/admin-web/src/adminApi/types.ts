import type { AdminOverviewResponse, PageInfo } from "./generated/contract";

export type {
  AdminCatalogGovernanceItem,
  AdminCatalogGovernanceItemListResponse,
  AdminJobListItem,
  AdminJobListResponse,
  AdminLocalInferenceSummary,
  AdminOutboxEventListItem,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminOverviewStatus,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListItem,
  AdminPlaybackSessionListResponse,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  PageInfo,
} from "./generated/contract";

export type DataSourceMode = "live" | "hybrid" | "mock" | "planned";

export type AdminSectionKey =
  | "overview"
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
  events: EventSummary;
  jobs: JobRow[];
  playback: PlaybackSummary;
  storage: StorageSummary;
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

export type SettingRow = {
  label: string;
  value: string;
};
