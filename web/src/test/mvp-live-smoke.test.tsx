import { createMemoryHistory } from "@tanstack/react-router"
import { fireEvent, render, screen, waitFor } from "@testing-library/react"
import type { FetchLike, MediaItemDto } from "@nako/sdk"
import { afterEach, describe, expect, it, vi } from "vitest"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { createPublicMediaDataSource } from "@/src/api/public/media-data-source"
import { savePublicClientConnection } from "@/src/api/public/connection"
import { VideoPlayer } from "@/src/features/media/video-player"
import { NakoRouter, createNakoRouter } from "@/src/shell"

const UNSAFE_SURFACE_NEEDLES = [
  "public-token",
  "F:\\private",
  "/mnt/private",
  "file:///mnt/private",
  "source-locator-secret",
  "provider-secret",
]

afterEach(() => {
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

describe("MVP Web/Public Client live smoke", () => {
  it("renders browser route surfaces through live Public Client paths without unsafe text or console errors", async () => {
    const consoleError = vi.spyOn(console, "error").mockImplementation(() => {})
    const fetcher = vi.fn<FetchLike>(mvpFetch)
    vi.stubGlobal("fetch", fetcher)
    savePublicClientConnection({
      mode: "live",
      baseUrl: "http://nako.test",
      bearerToken: "public-token",
    })

    const browse = renderRoute("/media")
    expect(await screen.findAllByText("Live Movie", {}, { timeout: 5000 })).not.toHaveLength(0)
    expectNoUnsafeSurfaceText()
    browse.unmount()

    const library = renderRoute("/media/library?id=library-a")
    expect(await screen.findByText(/已通过 Public Client 读取 1 个媒体项/, {}, { timeout: 5000 })).toBeInTheDocument()
    expect(await screen.findAllByText("Live Movie", {}, { timeout: 5000 })).not.toHaveLength(0)
    expectNoUnsafeSurfaceText()
    library.unmount()

    renderRoute("/media/detail?id=live-movie&type=movie")
    expect(await screen.findByRole("heading", { name: "Live Movie" }, { timeout: 5000 })).toBeInTheDocument()
    expect(await screen.findByText("媒体源", {}, { timeout: 5000 })).toBeInTheDocument()
    expectNoUnsafeSurfaceText()

    await waitFor(() => {
      expect(consoleError).not.toHaveBeenCalled()
    })

    expect(requestTargets(fetcher)).toEqual(
      expect.arrayContaining([
        "/items?limit=40&offset=0",
        "/libraries/library-a",
        "/libraries/library-a/items?limit=50&offset=0&sort=date_added&order=desc&watch_state=any",
        "/items/live-movie",
        "/users/me/playlists?limit=20&offset=0",
      ]),
    )
  })

  it("creates browser playback tickets, renders native video tracks, and heartbeats through playback_session_id", async () => {
    vi.spyOn(HTMLMediaElement.prototype, "pause").mockImplementation(() => {})
    vi.spyOn(HTMLMediaElement.prototype, "play").mockResolvedValue(undefined)
    const fetcher = vi.fn<FetchLike>(mvpFetch)
    const source = createPublicMediaDataSource(
      {
        mode: "live",
        baseUrl: "http://nako.test/",
        bearerToken: "public-token",
      },
      fetcher,
    )

    const plan = await source.loadPlaybackPlan("live-movie", "movie", "source-1")

    expect(plan).toMatchObject({
      source: "live",
      fallback: false,
      sourceId: "source-1",
      playbackSessionId: "playback-session-1",
      mode: "direct",
      mediaUrl: "http://nako.test/sources/source-1/stream?ticket=video-ticket",
      subtitles: [
        {
          streamIndex: 2,
          url: "http://nako.test/sources/source-1/subtitles/2?ticket=subtitle-ticket",
          contentType: "application/x-subrip; charset=utf-8",
        },
      ],
    })
    expectSerializedToBeSafe(plan)

    await source.heartbeatPlaybackSession("playback-session-1", {
      state: "active",
      position_ms: 12000,
      duration_ms: 120000,
    })

    const onPlaybackHeartbeat = vi.fn()
    const { container } = render(
      <VideoPlayer
        onBack={() => {}}
        mediaTitle="Live Movie"
        playbackSessionId={plan.playbackSessionId}
        onPlaybackHeartbeat={onPlaybackHeartbeat}
        sources={[
          {
            quality: plan.mode?.toUpperCase() ?? "Auto",
            url: plan.mediaUrl ?? "",
            contentType: plan.mediaContentType,
          },
        ]}
        subtitles={plan.subtitles.map((subtitle) => ({
          id: subtitle.id,
          language: subtitle.language,
          srcLang: subtitle.srcLang,
          url: subtitle.url,
          contentType: subtitle.contentType,
          default: subtitle.default,
          forced: subtitle.forced,
        }))}
      />,
    )
    const video = container.querySelector("video")
    const mediaSource = container.querySelector("source")
    const subtitleTrack = container.querySelector("track")
    expect(video).not.toBeNull()
    expect(mediaSource).toHaveAttribute("src", plan.mediaUrl)
    expect(subtitleTrack).toHaveAttribute("src", plan.subtitles[0].url)
    expectSerializedToBeSafe({
      sourceUrl: mediaSource?.getAttribute("src"),
      subtitleUrl: subtitleTrack?.getAttribute("src"),
    })

    Object.defineProperty(video!, "currentTime", { configurable: true, value: 12 })
    Object.defineProperty(video!, "duration", { configurable: true, value: 120 })
    fireEvent.playing(video!)

    await waitFor(() =>
      expect(onPlaybackHeartbeat).toHaveBeenCalledWith("playback-session-1", {
        state: "active",
        position_ms: 12000,
        duration_ms: 120000,
      }),
    )
    expect(requestTargets(fetcher)).toEqual(
      expect.arrayContaining([
        "/items/live-movie",
        "/sources/source-1/playback/decision?direct_play=true&supports_subtitles=true&hls_variant_policy=single_variant&hls_segment_container=mpeg_ts",
        "/sources/source-1/probe",
        "/sources/source-1/playback/browser-ticket",
        "/playback/sessions/playback-session-1/heartbeat",
      ]),
    )
  })
})

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

async function mvpFetch(input: RequestInfo | URL, init?: RequestInit) {
  const url = new URL(String(input))
  const method = init?.method ?? "GET"

  switch (`${method} ${url.pathname}`) {
    case "GET /items":
      return jsonResponse({
        items: [publicMediaItem()],
        page: page(40),
      })
    case "GET /items/live-movie":
      return jsonResponse({
        item: publicMediaItem(),
        sources: [publicMediaSource()],
        images: [],
        collections: [],
        credits: [],
        genres: [],
        studios: [],
        tags: [],
      })
    case "GET /libraries":
      return jsonResponse({
        libraries: [publicLibrary()],
        page: page(50),
      })
    case "GET /libraries/library-a":
      return jsonResponse({
        library: publicLibrary(),
      })
    case "GET /libraries/library-a/sources":
      return jsonResponse({
        library: publicLibrary(),
        page: page(20),
        sources: [publicLibrarySource()],
      })
    case "GET /libraries/library-a/items":
      return jsonResponse({
        library: publicLibrary(),
        page: page(50),
        items: [publicMediaItem()],
      })
    case "GET /users/me/playback-state/continue-watching":
      return jsonResponse({
        page: page(12),
        items: [
          {
            item: publicMediaItem(),
            images: [],
            state: {
              item_id: "live-movie",
              source_id: "source-1",
              resume_position_ms: 12000,
              duration_ms: 120000,
              progress_percent: 10,
              watched: false,
              watched_at: null,
              last_played_at: "2026-06-01T00:00:00Z",
              updated_at: "2026-06-01T00:00:01Z",
              version: 1,
            },
          },
        ],
      })
    case "GET /users/me/playlists":
      return jsonResponse({
        playlists: [],
        page: page(20, 0),
      })
    case "GET /management/context-links":
      return jsonResponse(managementContextLinks(url))
    case "GET /sources/source-1/playback/decision":
      return jsonResponse(playbackDecision())
    case "GET /sources/source-1/probe":
      return jsonResponse(sourceProbe())
    case "POST /sources/source-1/playback/browser-ticket":
      return jsonResponse(browserTicket(init))
    case "POST /playback/sessions/playback-session-1/heartbeat":
      return jsonResponse({
        session: {
          id: "playback-session-1",
          source_id: "source-1",
          item_id: "live-movie",
          mode: "direct",
          state: "active",
          position_ms: 12000,
          duration_ms: 120000,
          started_at: "2026-06-01T00:00:00Z",
          updated_at: "2026-06-01T00:00:01Z",
          ended_at: null,
        },
      })
    default:
      return jsonResponse({ code: "not_found", message: "not found" }, 404)
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

function page(limit: number, returned = 1) {
  return {
    limit,
    offset: 0,
    returned,
  }
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
      overview: "A release-smoke media item.",
      ratings: [{ source: "tmdb", value: "8.1" }],
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

function publicLibrary() {
  return {
    id: "library-a",
    name: "Live Library",
    roots: ["/mnt/private/library-a"],
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

function publicMediaSource() {
  return {
    id: "source-1",
    item_id: "live-movie",
    library_id: "library-a",
    file_name: "Live Movie.mkv",
    fingerprint: null,
    size_bytes: 1024,
    source_locator: "file:///mnt/private/library-a/Live Movie.mkv",
  }
}

function publicLibrarySource() {
  return {
    source: publicMediaSource(),
    item: publicMediaItem(),
    probe: null,
  }
}

function managementContextLinks(url: URL) {
  const libraryId = url.searchParams.get("library_id")
  const itemId = url.searchParams.get("item_id")
  const sourceId = url.searchParams.get("source_id")
  const playbackSessionId = url.searchParams.get("playback_session_id")

  return {
    context: {
      library_id: libraryId,
      item_id: itemId,
      source_id: sourceId,
      playback_session_id: playbackSessionId,
    },
    links: [
      {
        action: libraryId && !itemId ? "scan_library" : "refresh_item_metadata",
        disabled_reason: null,
        enabled: true,
        method: "POST",
        required_access: "library_manage",
        route_name: libraryId && !itemId ? "library.scan" : "item.metadata_refresh",
        surface: "management",
        target: {
          library_id: libraryId,
          item_id: itemId,
          source_id: sourceId ?? "file:///mnt/private/library-a/Live Movie.mkv",
          playback_session_id: playbackSessionId,
        },
      },
    ],
  }
}

function playbackDecision() {
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

function sourceProbe() {
  const disposition = {
    attached_pic: false,
    captions: false,
    commentary: false,
    default: true,
    descriptions: false,
    forced: false,
    hearing_impaired: false,
    visual_impaired: false,
  }

  return {
    source_id: "source-1",
    probe: {
      container: "matroska",
      duration_ms: 120000,
      bit_rate: null,
      streams: [
        {
          index: 0,
          kind: "video",
          codec: "h264",
          language: null,
          duration_ms: 120000,
          bit_rate: null,
          width: 1920,
          height: 1080,
          channels: null,
          sample_rate: null,
          disposition,
          origin: null,
        },
        {
          index: 2,
          kind: "subtitle",
          codec: "srt",
          language: "en",
          duration_ms: null,
          bit_rate: null,
          width: null,
          height: null,
          channels: null,
          sample_rate: null,
          disposition,
          origin: "sidecar",
        },
      ],
    },
  }
}

function browserTicket(init?: RequestInit) {
  const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined

  if (body?.mode === "subtitle") {
    return {
      source_id: "source-1",
      item_id: "live-movie",
      playback_session_id: null,
      mode: "subtitle",
      expires_at: "2026-06-01T00:05:00Z",
      urls: [
        {
          kind: "subtitle",
          url: "/sources/source-1/subtitles/2?ticket=subtitle-ticket",
          content_type: "application/x-subrip; charset=utf-8",
          supports_range_requests: false,
        },
      ],
    }
  }

  return {
    source_id: "source-1",
    item_id: "live-movie",
    playback_session_id: "playback-session-1",
    mode: "direct",
    expires_at: "2026-06-01T00:05:00Z",
    urls: [
      {
        kind: "stream",
        url: "/sources/source-1/stream?ticket=video-ticket",
        content_type: "video/x-matroska",
        supports_range_requests: true,
      },
    ],
  }
}

function requestTargets(fetcher: ReturnType<typeof vi.fn<FetchLike>>) {
  return fetcher.mock.calls.map(([input]) => {
    const url = new URL(String(input))
    return `${url.pathname}${url.search}`
  })
}

function expectNoUnsafeSurfaceText() {
  expectSerializedToBeSafe(document.body.textContent ?? "")
}

function expectSerializedToBeSafe(value: unknown) {
  const serialized = typeof value === "string" ? value : JSON.stringify(value)

  for (const needle of UNSAFE_SURFACE_NEEDLES) {
    expect(serialized).not.toContain(needle)
  }
}
