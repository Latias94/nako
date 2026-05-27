import type {
  AdminAccessSummaryResponse,
  AdminAddonRegistrationsResponse,
  AdminJobListResponse,
  AdminOverviewResponse,
} from "@/api/admin/generated/contract";

export const fixtureAdminOverview: AdminOverviewResponse = {
  admin_api_version: "v1",
  public_api_version: "v1",
  status: "degraded",
  metadata: {
    available_providers: 0,
    disabled_providers: 0,
    providers: [],
    total_providers: 0,
    unavailable_providers: 0,
  },
  runtime: {
    active_tasks: 0,
    cancelled_jobs: 0,
    completed_tasks: 0,
    failed_jobs: 0,
    failed_tasks: 0,
    shutdown_requested: false,
    succeeded_jobs: 0,
  },
  startup: {
    artwork_ingest_worker_started: false,
    configured_libraries: 0,
    metadata_lifecycle_tasks_started: 0,
    metadata_raw_cache_deleted: 0,
    recovered_jobs: 0,
    recovered_transcode_sessions: 0,
    staging_deleted_files: 0,
    staging_deleted_records: 0,
  },
  storage: {
    backends: [],
    degraded_backends: 0,
    ready_backends: 0,
    total_backends: 0,
    unavailable_backends: 0,
  },
};

export const fixtureAdminJobs: AdminJobListResponse = {
  jobs: [],
  page: {
    limit: 20,
    offset: 0,
    returned: 0,
  },
};

export const fixtureAdminAccessSummary: AdminAccessSummaryResponse = {
  admin_api_version: "v1",
  public_api_version: "v1",
  auth: {
    enabled: false,
    token_reference_configured: false,
  },
  library_access: {
    configured_libraries: 0,
    libraries: [],
  },
  mode: "single_admin",
  principal: {
    display_name: "Bootstrap administrator",
    principal_id: "bootstrap",
    principal_kind: "local_admin",
  },
  readiness: {
    library_access_policy: "planned",
    roles: "planned",
    single_admin_mode: "active",
    user_accounts: "planned",
  },
};

export const fixtureAdminAddons: AdminAddonRegistrationsResponse = {
  addons: [
    {
      base_url: "http://127.0.0.1:9000",
      created_at: "2026-05-28T01:00:00Z",
      granted_scopes: ["catalog_read"],
      id: "local-metadata-sidecar",
      manifest_id: "dev.nako.local-metadata",
      name: "Local Metadata Sidecar",
      protocol_version: "v1",
      status: "disabled",
      updated_at: "2026-05-28T01:00:00Z",
      version: "0.1.0",
    },
  ],
};
