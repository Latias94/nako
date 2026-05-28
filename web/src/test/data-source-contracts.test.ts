import type { FetchLike, MediaItemDto } from "@nako/sdk"
import { describe, expect, it, vi } from "vitest"
import {
  ADMIN_DASHBOARD_FIXTURE,
  createAdminDashboardDataSource,
} from "@/src/api/admin/dashboard-data-source"
import {
  ADMIN_LIBRARY_READ_MODEL_FIXTURE,
  ADMIN_LOGS_READ_MODEL_FIXTURE,
  ADMIN_SETTINGS_READ_MODEL_FIXTURE,
  ADMIN_TASKS_READ_MODEL_FIXTURE,
  ADMIN_USERS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
} from "@/src/api/admin/read-models-data-source"
import { createPublicMediaDataSource } from "@/src/api/public/media-data-source"

const page = {
  limit: 10,
  offset: 0,
  returned: 1,
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
}

function publicMediaItem(overrides: Partial<MediaItemDto> = {}): MediaItemDto {
  return {
    id: "live-movie",
    kind: "movie",
    parent_id: null,
    metadata: {
      collections: [],
      credits: [],
      external_ids: [],
      genres: ["Sci-Fi"],
      original_title: "Live Original",
      overview: "A mapped public API item.",
      ratings: [{ source: "tmdb", value: "7.86" }],
      release_date: "2026-01-02",
      runtime_minutes: 125,
      sort_title: null,
      studios: [],
      tagline: null,
      tags: [],
      title: "Live Movie",
    },
    ...overrides,
  }
}

function adminOverviewResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    status: "healthy",
    storage: {
      total_backends: 4,
      ready_backends: 3,
      degraded_backends: 1,
      unavailable_backends: 0,
      backends: [],
    },
    metadata: {
      total_providers: 2,
      available_providers: 2,
      disabled_providers: 0,
      unavailable_providers: 0,
      providers: [],
    },
    runtime: {
      active_tasks: 1,
      completed_tasks: 3,
      failed_tasks: 0,
      succeeded_jobs: 3,
      cancelled_jobs: 0,
      failed_jobs: 0,
      shutdown_requested: false,
    },
    startup: {
      configured_libraries: 7,
      recovered_transcode_sessions: 0,
      recovered_jobs: 0,
      staging_deleted_records: 0,
      staging_deleted_files: 0,
      metadata_raw_cache_deleted: 0,
      metadata_lifecycle_tasks_started: 1,
      artwork_ingest_worker_started: true,
    },
  }
}

function adminJobsResponse() {
  return {
    jobs: [
      {
        id: "job-1",
        kind: "library_scan",
        status: "running",
        resource_class: "library",
        library_id: "library-a",
        source_id: null,
        has_input: true,
        has_summary: false,
        has_error: false,
        queued_at: "2026-05-28T10:00:00Z",
        started_at: "2026-05-28T10:01:00Z",
        completed_at: null,
      },
    ],
    page,
  }
}

function adminPlaybackSessionsResponse() {
  return {
    sessions: [
      {
        id: "session-1",
        principal_id: "user-1",
        source_id: "source-1",
        item_id: "item-1",
        mode: "direct_play",
        state: "playing",
        transcode_session_id: null,
        has_client_capabilities: true,
        active: true,
        terminal: false,
        created_at: "2026-05-28T10:00:00Z",
        updated_at: "2026-05-28T10:10:00Z",
        started_at_ms: 1000,
        ended_at_ms: null,
        last_heartbeat_at_ms: 2000,
      },
      {
        id: "session-2",
        principal_id: "user-2",
        source_id: "source-2",
        item_id: "item-2",
        mode: "hls",
        state: "ended",
        transcode_session_id: "transcode-1",
        has_client_capabilities: true,
        active: false,
        terminal: true,
        created_at: "2026-05-28T09:00:00Z",
        updated_at: "2026-05-28T09:30:00Z",
        started_at_ms: 1000,
        ended_at_ms: 2000,
        last_heartbeat_at_ms: 2000,
      },
    ],
    page: {
      ...page,
      returned: 2,
    },
  }
}

function adminPlaybackRuntimeResponse(status = "ready") {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    readiness: {
      status,
      reason: status,
      checks: [],
    },
  }
}

function adminSystemConfigResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
  }
}

function adminReadModelOverviewResponse() {
  return {
    ...adminOverviewResponse(),
    storage: {
      ...adminOverviewResponse().storage,
      backends: [
        {
          library_id: "library-a",
          library_name: "Movies",
          backend_kind: "local",
          status: "ready",
        },
      ],
    },
  }
}

function adminReadModelSystemConfigResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    auth: {
      enabled: true,
      token_env: "NAKO_ADMIN_TOKEN",
    },
    network: {
      exposure_mode: "private_network",
      readiness: {
        status: "ready",
        reason: "ready",
        checks: [],
      },
      external_endpoint: {
        configured: true,
        scheme: "https",
        host_fingerprint: "fp",
      },
      trusted_proxy: {
        headers_enabled: true,
        source_count: 1,
      },
      origins: {
        allowed_origin_count: 2,
        configured: true,
      },
      tunnel_providers: [],
    },
    database: {
      configured_backend_kind: "sqlite",
      active_backend_kind: "sqlite",
      url_scheme: "sqlite",
      runtime_supported: true,
      migrated_on_startup: true,
      capabilities: {
        lifecycle: true,
        libraries: true,
        jobs: true,
        job_leases: true,
        media: true,
        scan_commits: true,
        metadata: true,
        catalog: true,
        playback_state: true,
        playback_sessions: true,
        transcode_sessions: true,
        event_outbox: true,
        addons: true,
        automation: true,
        managed_artwork: true,
        vfs_cache: true,
        webhooks: true,
        search_index: true,
      },
    },
    runtime: {
      listen_addr: "0.0.0.0:8096",
      scan_concurrency: 2,
      probe_concurrency: 4,
      metadata_concurrency: 3,
      remux_concurrency: 2,
      webhook_concurrency: 1,
      remux_timeout_ms: 120000,
    },
    libraries: [
      {
        id: "library-a",
        name: "Movies",
        preset: "movie",
        backend_kind: "local",
        root_scheme: "file",
        has_webdav_password_env: false,
        webdav_timeout_ms: null,
        webdav_max_attempts: null,
      },
    ],
    metadata: {
      raw_cache_retention_ms: 86400000,
      raw_cache_cleanup_on_startup: true,
      raw_cache_cleanup_interval_ms: 3600000,
      maintenance_policies: 1,
      providers: [
        {
          provider: "tmdb",
          enabled: true,
          token_env: null,
          api_key_env: "TMDB_API_KEY",
          has_api_base_url: true,
          has_image_base_url: true,
          language: "zh-CN",
          include_adult: false,
          header_count: 0,
          secret_header_count: 0,
          has_provider_runtime_override: false,
        },
      ],
      runtime: {
        timeout_ms: 10000,
        max_attempts: 3,
        min_interval_ms: 100,
        concurrency: 4,
        user_agent: "nako-test",
        has_proxy: false,
        circuit_breaker_failures: 3,
        circuit_breaker_backoff_ms: 1000,
      },
    },
    transcode: {
      hardware_policy: {},
      cpu_concurrency: 2,
      gpu_concurrency: 1,
    },
    staging: {
      max_bytes: 1024,
      retention_ms: 86400000,
      cleanup_on_startup: true,
    },
    playback: {
      remote_stream_concurrency: 4,
      remote_stage_concurrency: 2,
      transcode_artifact_retention_ms: 86400000,
      transcode_artifact_cleanup_on_startup: true,
      hls_segment_cleanup_enabled: true,
      hls_segment_keep_ms: 60000,
      transcode_throttle_enabled: false,
      transcode_throttle_delay_ms: 0,
    },
    artwork: {
      artifact_root_configured: true,
      fetch_timeout_ms: 10000,
      fetch_max_attempts: 3,
      fetch_max_bytes: 4096,
      fetch_concurrency: 2,
      ingest_worker_enabled: true,
      ingest_worker_idle_ms: 1000,
      fetch_user_agent: "nako-test",
      has_fetch_proxy: false,
      max_width: 3000,
      max_height: 3000,
    },
  }
}

function adminAccessSummaryResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    mode: "single_admin",
    principal: {
      principal_id: "principal-admin",
      display_name: "Admin",
      principal_kind: "local_admin",
    },
    auth: {
      enabled: true,
      token_reference_configured: true,
    },
    readiness: {
      single_admin_mode: "active",
      user_accounts: "active",
      roles: "active",
      library_access_policy: "planned",
    },
    library_access: {
      configured_libraries: 1,
      libraries: [
        {
          library_id: "library-a",
          library_name: "Movies",
          preset: "movie",
          backend_kind: "local",
          access: "manage",
          reason: "single_admin_mode",
        },
      ],
    },
  }
}

function adminAccessUsersResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    users: [
      {
        user_id: "user-1",
        principal_id: "user-1",
        username: "admin",
        display_name: "Admin",
        status: "active",
        roles: ["administrator"],
        bootstrap: true,
        local_password_configured: true,
        created_at_ms: 1000,
        updated_at_ms: 2000,
      },
    ],
    page,
  }
}

function adminEventsResponse() {
  return {
    events: [
      {
        id: "event-1",
        kind: "library.scan.completed",
        subject: "library-a",
        library_id: "library-a",
        source_id: null,
        status: "pending",
        attempts: 1,
        has_payload: true,
        has_error: false,
        occurred_at: "2026-05-28T10:00:00Z",
        updated_at: "2026-05-28T10:01:00Z",
        next_attempt_at: null,
      },
    ],
    page,
  }
}

function adminStorageStagingResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    summary: {
      configured_max_bytes: 1000,
      used_manifest_bytes: 250,
      cleanup_on_startup: true,
      retention_ms: 86400000,
      startup_deleted_records: 0,
      startup_deleted_files: 0,
      process_cached_backends: 0,
      vfs_cache: {
        object_count: 0,
        listing_count: 0,
        failure_count: 0,
        stale_object_count: 0,
        stale_listing_count: 0,
        last_failure_at_ms: null,
      },
    },
    records: [],
    page,
  }
}

function adminPlaybackRuntimeFullResponse() {
  return {
    ...adminPlaybackRuntimeResponse(),
    policy: {
      user_policy_rows_supported: true,
      role_policy_rows_supported: true,
      effective_resolution_supported: true,
      library_access_required: true,
      user_policy_overrides_role_policy: false,
      role_policy_merge: "restrictive",
      permissions: ["media_playback"],
    },
    ffmpeg: {
      probe_status: "ok",
      has_probe_error: false,
      hardware_capability_count: 1,
      available_gpu_capabilities: 1,
    },
    hardware: {
      policy: {},
      selection: {
        acceleration: "qsv",
        fallback_used: false,
        reason: "ready",
      },
      capabilities: [],
    },
    transcode: {
      configured_cpu_slots: 2,
      configured_gpu_slots: 1,
      effective_cpu_slots: 2,
      effective_gpu_slots: 1,
      selected_hls_slots: 1,
    },
    remux: {
      max_concurrent_sessions: 2,
      timeout_ms: 120000,
    },
    remote_playback: {
      backend_count: 1,
      stream_permits_available: 4,
      stream_permits_max: 4,
      stage_permits_available: 2,
      stage_permits_max: 2,
      state_scope: "server",
    },
    staging: {
      max_bytes: 1000,
      retention_ms: 86400000,
      cleanup_on_startup: true,
      startup_deleted_records: 0,
      startup_deleted_files: 0,
    },
    artifact_lifecycle: {
      transcode_artifact_retention_ms: 86400000,
      transcode_artifact_cleanup_on_startup: true,
      hls_segment_cleanup_enabled: true,
      hls_segment_keep_ms: 60000,
      startup_examined_artifacts: 0,
      startup_deleted_artifacts: 0,
      startup_deleted_files: 0,
      startup_deleted_directories: 0,
      startup_deleted_bytes: 0,
      startup_skipped_security: 0,
    },
    throttle: {
      enabled: false,
      delay_ms: 0,
    },
  }
}

function adminRawCacheSettingsResponse() {
  return {
    admin_api_version: "v1",
    retention_ms: 86400000,
    cleanup_on_startup: true,
    source: "configured",
    effect: "active",
    updated_at_ms: null,
  }
}

describe("public media data source contracts", () => {
  it("uses local fixtures when configured for fixture mode", async () => {
    const source = createPublicMediaDataSource({ mode: "fixture" })

    const payload = await source.listMedia()

    expect(payload.source).toBe("fixture")
    expect(payload.fallback).toBe(true)
    expect(payload.items[0]).toMatchObject({
      id: "1",
      title: "沙丘2",
      type: "movie",
    })
  })

  it("maps live Public Client DTOs into UI media items", async () => {
    const fetcher = vi.fn<FetchLike>(async () =>
      jsonResponse({
        items: [publicMediaItem()],
        page,
      }),
    )

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const payload = await source.listMedia()

    expect(payload).toMatchObject({
      source: "live",
      fallback: false,
      items: [
        {
          id: "live-movie",
          title: "Live Movie",
          originalTitle: "Live Original",
          year: 2026,
          rating: 7.9,
          duration: "2h 5m",
          type: "movie",
        },
      ],
    })
    expect(fetcher.mock.calls[0][0]).toBe("http://nako.test/items?limit=40&offset=0")
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer public-token",
    )
  })

  it("falls back to local search results when the live Public Client request fails", async () => {
    const fetcher = vi.fn<FetchLike>(async () => {
      throw new Error("public offline")
    })

    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test",
      },
      fetcher,
    )

    const payload = await source.searchMedia("Dune")

    expect(payload).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "public offline",
    })
    expect(payload.items.map((item) => item.title)).toEqual(["沙丘2"])
  })
})

describe("admin dashboard data source contracts", () => {
  it("uses the dashboard fixture when configured for fixture mode", async () => {
    const source = createAdminDashboardDataSource({ mode: "fixture" })

    await expect(source.loadDashboard()).resolves.toBe(ADMIN_DASHBOARD_FIXTURE)
  })

  it("maps live Admin API responses into dashboard data", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      switch (url.pathname) {
        case "/admin/v1/overview":
          return jsonResponse(adminOverviewResponse())
        case "/admin/v1/jobs":
          return jsonResponse(adminJobsResponse())
        case "/admin/v1/playback/sessions":
          return jsonResponse(adminPlaybackSessionsResponse())
        case "/admin/v1/playback/runtime":
          return jsonResponse(adminPlaybackRuntimeResponse())
        case "/admin/v1/system/config":
          return jsonResponse(adminSystemConfigResponse())
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminDashboardDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const dashboard = await source.loadDashboard()

    expect(dashboard).toMatchObject({
      source: "live",
      fallback: false,
      metrics: {
        storage: 75,
        totalLibraries: 7,
        activeStreams: 1,
        version: "Admin v1",
        latestVersion: "Public v1",
      },
      activeTasks: [
        {
          id: "job-1",
          type: "library_scan",
          status: "running",
          library: "library-a",
          progress: 50,
          startedAt: "2026-05-28T10:01:00Z",
        },
      ],
      playbackSessions: [
        {
          id: "session-1",
          user: "user-1",
          item: "item-1",
          playbackMethod: "direct_play",
          progress: 0,
          quality: "playing",
        },
      ],
    })

    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTargets).toEqual([
      "/admin/v1/overview",
      "/admin/v1/jobs?limit=3&offset=0",
      "/admin/v1/playback/sessions?limit=5&offset=0",
      "/admin/v1/playback/runtime",
      "/admin/v1/system/config",
    ])
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )
  })

  it("falls back to the dashboard fixture when a live Admin API request fails", async () => {
    const fetcher = vi.fn<typeof fetch>(async () => {
      throw new Error("admin offline")
    })

    const source = createAdminDashboardDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test",
      },
      fetcher,
    )

    const dashboard = await source.loadDashboard()

    expect(dashboard).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "admin offline",
    })
    expect(dashboard.metrics).toBe(ADMIN_DASHBOARD_FIXTURE.metrics)
  })
})

describe("admin read model data source contracts", () => {
  it("uses explicit fixtures for deeper Admin pages in fixture mode", async () => {
    const source = createAdminReadModelsDataSource({ mode: "fixture" })

    await expect(source.loadLibraries()).resolves.toBe(ADMIN_LIBRARY_READ_MODEL_FIXTURE)
    await expect(source.loadUsers()).resolves.toBe(ADMIN_USERS_READ_MODEL_FIXTURE)
    await expect(source.loadTasks()).resolves.toBe(ADMIN_TASKS_READ_MODEL_FIXTURE)
    await expect(source.loadLogs()).resolves.toBe(ADMIN_LOGS_READ_MODEL_FIXTURE)
    await expect(source.loadSettings()).resolves.toBe(ADMIN_SETTINGS_READ_MODEL_FIXTURE)
  })

  it("maps live Admin API responses into deeper Admin page read models", async () => {
    const fetcher = vi.fn<typeof fetch>(async (input) => {
      const url = new URL(String(input))

      switch (url.pathname) {
        case "/admin/v1/overview":
          return jsonResponse(adminReadModelOverviewResponse())
        case "/admin/v1/system/config":
          return jsonResponse(adminReadModelSystemConfigResponse())
        case "/admin/v1/access/summary":
          return jsonResponse(adminAccessSummaryResponse())
        case "/admin/v1/access/users":
          return jsonResponse(adminAccessUsersResponse())
        case "/admin/v1/playback/sessions":
          return jsonResponse(adminPlaybackSessionsResponse())
        case "/admin/v1/jobs":
          return jsonResponse(adminJobsResponse())
        case "/admin/v1/events":
          return jsonResponse(adminEventsResponse())
        case "/admin/v1/playback/runtime":
          return jsonResponse(adminPlaybackRuntimeFullResponse())
        case "/admin/v1/storage/staging":
          return jsonResponse(adminStorageStagingResponse())
        case "/admin/v1/settings/metadata/raw-cache":
          return jsonResponse(adminRawCacheSettingsResponse())
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })

    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test/",
        bearerToken: "admin-token",
      },
      fetcher,
    )

    const [libraries, users, tasks, logs, settings] = await Promise.all([
      source.loadLibraries(),
      source.loadUsers(),
      source.loadTasks(),
      source.loadLogs(),
      source.loadSettings(),
    ])

    expect(libraries).toMatchObject({
      source: "live",
      fallback: false,
      libraries: [
        {
          id: "library-a",
          name: "Movies",
          type: "movie",
          paths: [{ path: "file://local", available: true }],
          settings: {
            useNfo: true,
            downloadArt: true,
            metadataLanguage: "zh-CN",
          },
        },
      ],
    })
    expect(users).toMatchObject({
      source: "live",
      users: [
        {
          id: "user-1",
          role: "admin",
          status: "online",
          libraryAccess: ["all"],
        },
      ],
      activeSessions: [
        {
          id: "session-1",
          userId: "user-1",
          lastActivity: "正在播放: item-1",
        },
      ],
    })
    expect(tasks).toMatchObject({
      source: "live",
      tasks: [
        {
          id: "job-1",
          type: "scan",
          status: "running",
          progress: 50,
        },
      ],
      runningTask: {
        id: "job-1",
      },
      history: [
        {
          id: "event-1",
          status: "success",
        },
      ],
    })
    expect(logs.logs[0]).toMatchObject({
      id: "event-1",
      level: "warn",
      source: "scanner",
      requestId: "event-1",
    })
    expect(settings).toMatchObject({
      source: "live",
      general: {
        listenAddr: "0.0.0.0:8096",
        authEnabled: true,
      },
      metadata: {
        rawCacheRetentionMs: 86400000,
        enabledProviderCount: 1,
      },
      transcode: {
        hardwareAcceleration: "qsv",
        remuxConcurrency: 2,
      },
      storage: {
        stagingUsedBytes: 250,
      },
    })

    const calledTargets = fetcher.mock.calls.map(([input]) => {
      const url = new URL(String(input))
      return `${url.pathname}${url.search}`
    })
    expect(calledTargets).toEqual(
      expect.arrayContaining([
        "/admin/v1/access/users?limit=100&offset=0",
        "/admin/v1/events?limit=200&offset=0",
        "/admin/v1/settings/metadata/raw-cache",
        "/admin/v1/storage/staging?limit=100&offset=0",
      ]),
    )
    expect(new Headers(fetcher.mock.calls[0][1]?.headers).get("Authorization")).toBe(
      "Bearer admin-token",
    )
  })

  it("falls back to explicit Admin page fixtures when a live read model request fails", async () => {
    const fetcher = vi.fn<typeof fetch>(async () => {
      throw new Error("admin read model offline")
    })
    const source = createAdminReadModelsDataSource(
      {
        mode: "live",
        baseUrl: "http://nako-admin.test",
      },
      fetcher,
    )

    const libraries = await source.loadLibraries()

    expect(libraries).toMatchObject({
      source: "fixture",
      fallback: true,
      error: "admin read model offline",
    })
    expect(libraries.libraries).toBe(ADMIN_LIBRARY_READ_MODEL_FIXTURE.libraries)
  })
})
