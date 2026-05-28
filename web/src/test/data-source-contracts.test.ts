import type { FetchLike, MediaItemDto } from "@nako/sdk"
import { describe, expect, it, vi } from "vitest"
import {
  ADMIN_DASHBOARD_FIXTURE,
  createAdminDashboardDataSource,
} from "@/src/api/admin/dashboard-data-source"
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
