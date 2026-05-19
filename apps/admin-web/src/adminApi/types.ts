export type DataSourceMode = "live" | "hybrid" | "mock" | "planned";

export type PageInfo = {
  limit: number;
  offset: number;
  returned: number;
};

export type AdminOverviewStatus = "healthy" | "degraded";

export type AdminOverviewResponse = {
  admin_api_version: string;
  public_api_version: string;
  status: AdminOverviewStatus;
  storage: {
    total_backends: number;
    ready_backends: number;
    degraded_backends: number;
    unavailable_backends: number;
    backends: Array<{
      library_id: string;
      library_name: string;
      backend_kind: string;
      status: string;
    }>;
  };
  metadata: {
    total_providers: number;
    available_providers: number;
    disabled_providers: number;
    unavailable_providers: number;
    providers: Array<{
      provider: string;
      status: string;
    }>;
  };
  runtime: {
    active_tasks: number;
    completed_tasks: number;
    failed_tasks: number;
    succeeded_jobs: number;
    cancelled_jobs: number;
    failed_jobs: number;
    shutdown_requested: boolean;
  };
  startup: {
    configured_libraries: number;
    recovered_transcode_sessions: number;
    recovered_jobs: number;
    staging_deleted_records: number;
    staging_deleted_files: number;
    metadata_raw_cache_deleted: number;
    metadata_lifecycle_tasks_started: number;
    artwork_ingest_worker_started: boolean;
  };
};

export type AdminJobListItem = {
  id: string;
  kind: string;
  status: string;
  resource_class: string;
  library_id: string | null;
  source_id: string | null;
  has_input: boolean;
  has_summary: boolean;
  has_error: boolean;
  queued_at: string;
  started_at: string | null;
  completed_at: string | null;
};

export type AdminJobListResponse = {
  jobs: AdminJobListItem[];
  page: PageInfo;
};

export type AdminPlaybackSessionListItem = {
  id: string;
  source_id: string;
  kind: string;
  request_key: string;
  state: string;
  failure_category: string | null;
  has_failure_message: boolean;
  active: boolean;
  terminal: boolean;
  created_at: string;
  updated_at: string;
  started_at: string | null;
  completed_at: string | null;
};

export type AdminPlaybackSessionListResponse = {
  sessions: AdminPlaybackSessionListItem[];
  page: PageInfo;
};

export type AdminConsoleData = {
  overview: AdminOverviewResponse;
  overviewSource: DataSourceMode;
  libraries: LibraryRow[];
  jobs: JobRow[];
  playback: PlaybackSummary;
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

export type JobRow = {
  id: string;
  kind: string;
  status: "queued" | "running" | "succeeded" | "failed" | "cancelled";
  resourceClass: string;
};

export type PlaybackSummary = {
  hardwarePolicy: string;
  accelerators: Array<{
    name: string;
    available: boolean;
  }>;
  sessions: Array<{
    id: string;
    kind: string;
    sourceTitle: string;
    state: "running" | "starting" | "failed" | "finished" | "cancelled";
  }>;
};

export type SettingRow = {
  label: string;
  value: string;
};
