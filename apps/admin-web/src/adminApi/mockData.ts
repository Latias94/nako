import type { AdminConsoleData, AdminOverviewResponse } from "./types";

export const mockOverview: AdminOverviewResponse = {
  admin_api_version: "v1",
  public_api_version: "v1",
  status: "healthy",
  storage: {
    total_backends: 3,
    ready_backends: 2,
    degraded_backends: 1,
    unavailable_backends: 0,
    backends: [
      {
        library_id: "library-anime",
        library_name: "Anime Vault",
        backend_kind: "local",
        status: "ready",
      },
      {
        library_id: "library-films",
        library_name: "Films",
        backend_kind: "webdav",
        status: "ready",
      },
      {
        library_id: "library-archive",
        library_name: "Archive",
        backend_kind: "local",
        status: "degraded",
      },
    ],
  },
  metadata: {
    total_providers: 4,
    available_providers: 3,
    disabled_providers: 1,
    unavailable_providers: 0,
    providers: [
      { provider: "tmdb", status: "available" },
      { provider: "douban", status: "available" },
      { provider: "bangumi", status: "available" },
      { provider: "nfo", status: "disabled" },
    ],
  },
  runtime: {
    active_tasks: 4,
    completed_tasks: 186,
    failed_tasks: 2,
    succeeded_jobs: 72,
    cancelled_jobs: 1,
    failed_jobs: 3,
    shutdown_requested: false,
  },
  startup: {
    configured_libraries: 3,
    recovered_transcode_sessions: 0,
    recovered_jobs: 1,
    staging_deleted_records: 4,
    staging_deleted_files: 4,
    metadata_raw_cache_deleted: 18,
    metadata_lifecycle_tasks_started: 2,
    artwork_ingest_worker_started: true,
  },
};

export const mockAdminConsoleData: AdminConsoleData = {
  overview: mockOverview,
  overviewSource: "mock",
  libraries: [
    {
      id: "library-anime",
      name: "Anime Vault",
      backendKind: "local",
      status: "ready",
      itemCount: 1248,
      lastScan: "14 min ago",
    },
    {
      id: "library-films",
      name: "Films",
      backendKind: "webdav",
      status: "ready",
      itemCount: 382,
      lastScan: "2 hr ago",
    },
    {
      id: "library-archive",
      name: "Archive",
      backendKind: "local",
      status: "degraded",
      itemCount: 91,
      lastScan: "needs review",
    },
  ],
  jobs: [
    {
      id: "job-scan",
      kind: "LibraryScan",
      status: "running",
      resourceClass: "library",
    },
    {
      id: "job-artwork",
      kind: "ManagedArtworkIngest",
      status: "queued",
      resourceClass: "artwork",
    },
    {
      id: "job-metadata",
      kind: "MetadataRefresh",
      status: "failed",
      resourceClass: "metadata",
    },
  ],
  playback: {
    hardwarePolicy: "Prefer NVENC, fall back to CPU",
    accelerators: [
      { name: "VAAPI", available: false },
      { name: "NVENC", available: true },
      { name: "QuickSync", available: false },
    ],
    sessions: [
      {
        id: "session-hls",
        kind: "HLS transcode",
        sourceTitle: "Episode 08, high bitrate source",
        state: "running",
      },
      {
        id: "session-remux",
        kind: "Remux",
        sourceTitle: "Film archive preview",
        state: "starting",
      },
    ],
  },
  settings: [
    { label: "Admin auth", value: "Auth configured" },
    { label: "FFmpeg", value: "Configured, diagnostics only" },
    { label: "Transcode policy", value: "GPU preferred" },
    { label: "Settings edits", value: "Planned Admin API" },
  ],
};
