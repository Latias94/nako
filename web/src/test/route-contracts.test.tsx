import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen, waitFor } from "@testing-library/react"
import { describe, expect, it, vi } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { CONNECTION_PROFILE_STORAGE_KEY, CONNECTION_SESSION_STORAGE_KEY } from "@/src/api/connection-profile"

interface RouteContract {
  path: string
  assert: () => Promise<void>
}

function renderRoute(path: string) {
  const router = createNakoRouter({
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  return render(
    <ThemeProvider defaultTheme="dark">
      <QueryProvider>
        <NakoRouter router={router} />
      </QueryProvider>
    </ThemeProvider>,
  )
}

const routeContracts: RouteContract[] = [
  {
    path: "/",
    assert: async () => {
      expect(await screen.findByText(/继续观看/, {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media",
    assert: async () => {
      expect(await screen.findByText(/继续观看/, {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/search?q=dune",
    assert: async () => {
      expect(await screen.findByDisplayValue("dune", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/detail?id=1&type=movie",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "沙丘2" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/watch?id=1&type=movie",
    assert: async () => {
      expect(await screen.findByText("视频播放区域", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/library?id=movies",
    assert: async () => {
      expect(await screen.findByRole("button", { name: /按 日期已添加 排序/ }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/library?id=movies&view=table&sort=title&order=asc&filter=unwatched",
    assert: async () => {
      expect(await screen.findByRole("button", { name: /按 标题 排序/ }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/media/my-list",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "我的列表" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "仪表盘" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/libraries",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "媒体库管理" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/users",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "用户管理" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/tasks",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "计划任务" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/logs",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "系统日志" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/logs?q=database&levels=error,warn&sources=database&tab=errors&time=7d",
    assert: async () => {
      expect(await screen.findByDisplayValue("database", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("最近7天", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/acquisition/intake?state=ready",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "采集入口" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("fixture-intake-1", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/automation/generated-artifacts?limit=25",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "生成产物" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/automation/generated-artifacts/review?artifact_id=fixture-generated-artifact-1&decision=accept",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "生成产物审核" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("Metadata Authority apply", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/automation/generated-artifacts/metadata-apply?artifact_id=fixture-generated-artifact-1",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "Metadata Authority apply" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("field_locked", {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("Provider Mapping 计划", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/automation/generated-artifacts/recovery?attention=needs_repair",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "生成产物恢复" }, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText(/fixture-generated-outcome-1/, {}, { timeout: 10000 })).toBeInTheDocument()
      expect(await screen.findByText("只读恢复队列", {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/admin/settings",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "高级设置" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/notifications",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "通知中心" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/settings",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: "设置" }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/setup",
    assert: async () => {
      expect(await screen.findByText(/Welcome to Nako/, {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/account",
    assert: async () => {
      expect(await screen.findByText(/谁在观看？/, {}, { timeout: 10000 })).toBeInTheDocument()
    },
  },
  {
    path: "/tv",
    assert: async () => {
      expect(await screen.findByRole("heading", { name: /沙丘2/ }, { timeout: 10000 })).toBeInTheDocument()
    },
  },
]

describe("top-level route contracts", () => {
  it.each(routeContracts)("$path renders expected surface", async ({ path, assert }) => {
    renderRoute(path)

    await assert()
  })

  it("renders live scoped library items from the Public Client browse route", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = new URL(String(input))

      if (url.pathname === "/libraries/movies") {
        return jsonResponse({
          library: publicLibrary("movies"),
        })
      }

      if (url.pathname === "/libraries/movies/sources") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page,
          sources: [],
        })
      }

      if (url.pathname === "/libraries/movies/items") {
        return jsonResponse({
          library: publicLibrary("movies"),
          page,
          items: [publicMediaItem()],
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/media/library?id=movies")

      expect(await screen.findByText("Live Movie", {}, { timeout: 10000 })).toBeInTheDocument()
      const calledTargets = fetcher.mock.calls.map(([input]) => {
        const url = new URL(String(input))
        return `${url.pathname}${url.search}`
      })

      expect(calledTargets).toContain(
        "/libraries/movies/items?limit=50&offset=0&sort=date_added&order=desc&watch_state=any",
      )
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      if (previousProfile === null) {
        window.localStorage.removeItem(CONNECTION_PROFILE_STORAGE_KEY)
      } else {
        window.localStorage.setItem(CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      }
    }
  })

  it("renders live user playlist items from the Public Client playlist route", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      const url = new URL(String(input))

      if (url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page,
        })
      }

      if (url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist(),
          page,
          items: [
            {
              playlist_id: "playlist-live",
              item_id: "live-movie",
              position: 0,
              added_at: "2026-05-29T01:00:00Z",
              item: publicMediaItem(),
              images: [],
            },
          ],
        })
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/media/my-list?playlist=playlist-live&view=list")

      expect(await screen.findAllByText("Live Playlist", {}, { timeout: 10000 })).toHaveLength(2)
      expect(await screen.findByText("Live Movie", {}, { timeout: 10000 })).toBeInTheDocument()
      const calledTargets = fetcher.mock.calls
        .map(([input]) => {
          const url = new URL(String(input))
          return `${url.pathname}${url.search}`
        })
        .filter((target) => target.startsWith("/users/me/playlists"))

      expect(calledTargets).toEqual([
        "/users/me/playlists?limit=20&offset=0",
        "/users/me/playlists/playlist-live/items?limit=50&offset=0",
      ])
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      if (previousProfile === null) {
        window.localStorage.removeItem(CONNECTION_PROFILE_STORAGE_KEY)
      } else {
        window.localStorage.setItem(CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      }
    }
  })

  it("starts live Media watch through Public Client playback ticket routes", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{
      method: string
      path: string
      authorization: string | null
      body?: unknown
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method,
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
        body,
      })

      switch (`${method} ${url.pathname}`) {
        case "GET /items/live-movie":
          return jsonResponse({
            item: publicMediaItem(),
            sources: [publicMediaSource()],
            images: [],
          })
        case "GET /sources/source-live/playback/decision":
          return jsonResponse(playbackDecisionResponse())
        case "GET /sources/source-live/probe":
          return jsonResponse(sourceProbeResponse())
        case "POST /sources/source-live/playback/browser-ticket":
          return jsonResponse(browserTicketResponse())
        default:
          return jsonResponse({ code: "not_found", message: "not found" }, 404)
      }
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "public-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/media/watch?id=live-movie&type=movie&source_id=source-live")

      const source = await screen.findByTestId("nako-video-source", {}, { timeout: 10000 })
      expect(source).toHaveAttribute(
        "src",
        "http://nako.test/sources/source-live/stream?ticket=video-ticket",
      )
      expect(source).toHaveAttribute("type", "video/x-matroska")

      await waitFor(() => {
        expect(calls.some((call) => call.path === "/sources/source-live/playback/browser-ticket")).toBe(true)
      })
      expect(calls.map((call) => `${call.method} ${call.path}`)).toEqual(
        expect.arrayContaining([
          "GET /items/live-movie",
          "GET /sources/source-live/probe",
          "POST /sources/source-live/playback/browser-ticket",
        ]),
      )
      const decisionCall = calls.find((call) => call.path.startsWith("/sources/source-live/playback/decision?"))
      const decisionUrl = new URL(decisionCall?.path ?? "/", "http://nako.test")
      expect(decisionCall?.method).toBe("GET")
      expect(decisionUrl.searchParams.get("direct_play")).toBe("true")
      expect(decisionUrl.searchParams.get("supports_subtitles")).toBe("true")
      expect(decisionUrl.searchParams.get("hls_variant_policy")).toBe("single_variant")
      expect(decisionUrl.searchParams.get("hls_segment_container")).toBe("mpeg_ts")
      expect(calls.every((call) => !call.path.startsWith("/admin"))).toBe(true)
      expect(calls.find((call) => call.method === "POST")?.authorization).toBe("Bearer public-token")
      expect(JSON.stringify(calls.find((call) => call.method === "POST")?.body)).not.toContain("public-token")
      expect(document.body.textContent).not.toContain("public-token")
      expect(document.body.textContent).not.toContain("video-ticket")
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })
})

const page = {
  limit: 50,
  offset: 0,
  returned: 1,
}

function restoreStorage(storage: Storage, key: string, value: string | null) {
  if (value === null) {
    storage.removeItem(key)
  } else {
    storage.setItem(key, value)
  }
}

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
}

function publicLibrary(id: string) {
  return {
    id,
    name: "Live Library",
    roots: ["/media/live"],
    options: {
      domain: "video",
      preset: "movies",
      naming_strategy: "movie",
      scan: {
        max_depth: null,
        realtime_monitor: true,
      },
      metadata_profile: {
        country: null,
        image_providers: [],
        item_kinds: ["movie"],
        language: null,
        local_metadata_policy: "read_only",
        local_readers: [],
        metadata_providers: [],
        refresh_mode: "default",
        scan: {
          addon_scrape: true,
          addon_writeback: false,
          enabled: true,
        },
      },
    },
  }
}

function publicMediaItem() {
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
      overview: "A routed public API item.",
      ratings: [{ source: "tmdb", value: "8.1" }],
      release_date: "2026-01-02",
      runtime_minutes: 125,
      sort_title: null,
      studios: [],
      tagline: null,
      tags: [],
      title: "Live Movie",
    },
  }
}

function publicMediaSource() {
  return {
    id: "source-live",
    item_id: "live-movie",
    library_id: "library-a",
    file_name: "Live Movie.mkv",
    fingerprint: null,
    size_bytes: 1024,
  }
}

function playbackDecisionResponse() {
  return {
    source: publicMediaSource(),
    probe: null,
    target: {
      kind: "browser",
      network_scope: "local",
      transport_auth: "ticket",
      control_capabilities: {
        can_pause: true,
        can_seek: true,
        can_set_volume: true,
        can_stop: true,
      },
      media_capabilities: {
        direct_play: true,
      },
    },
    decision: {
      mode: "direct_play",
      reason: "compatible",
      direct_play: {},
      transcode_plan: null,
      denial: null,
      report: {
        selected_mode: "direct_play",
        direct_play: { supported: true, conditions: ["compatible"] },
        remux: { supported: true, conditions: ["compatible"] },
        transcode: { supported: false, conditions: ["requested_transcode_output"] },
      },
    },
  }
}

function sourceProbeResponse() {
  return {
    source_id: "source-live",
    probe: {
      container: "matroska",
      duration_ms: 60000,
      bit_rate: null,
      streams: [],
    },
  }
}

function browserTicketResponse() {
  return {
    source_id: "source-live",
    item_id: "live-movie",
    playback_session_id: "playback-session-live",
    mode: "direct",
    expires_at: "2026-05-28T10:00:00Z",
    urls: [
      {
        kind: "stream",
        url: "/sources/source-live/stream?ticket=video-ticket",
        content_type: "video/x-matroska",
        supports_range_requests: true,
      },
    ],
  }
}

function publicUserPlaylist() {
  return {
    id: "playlist-live",
    name: "Live Playlist",
    visibility: "private",
    item_count: 1,
    created_at: "2026-05-29T00:00:00Z",
    updated_at: "2026-05-29T01:00:00Z",
    version: 2,
  }
}
