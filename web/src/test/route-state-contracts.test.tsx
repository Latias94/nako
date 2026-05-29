import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { CONNECTION_PROFILE_STORAGE_KEY, CONNECTION_SESSION_STORAGE_KEY } from "@/src/api/connection-profile"

function renderRoute(path: string) {
  const router = createNakoRouter({
    history: createMemoryHistory({ initialEntries: [path] }),
  })

  const result = render(
    <ThemeProvider defaultTheme="dark">
      <QueryProvider>
        <NakoRouter router={router} />
      </QueryProvider>
    </ThemeProvider>,
  )

  return { router, ...result }
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("route state contracts", () => {
  it("writes Admin log search state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/logs")

    const input = await screen.findByPlaceholderText("搜索日志内容...", {}, { timeout: 5000 })
    await user.type(input, "database")

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "database" })
    })
  })

  it("renders live Admin acquisition intake candidates without exposing raw sensitive fields", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/acquisition/intake/candidates") {
        return jsonResponse(adminAcquisitionIntakeCandidatesResponse())
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute(
        "/admin/acquisition/intake?library_id=library-a&state=ready&source_kind=watch_folder&managed_import_artifact_id=artifact-live&limit=25&offset=50",
      )

      expect(await screen.findByText("candidate-live", {}, { timeout: 5000 })).toBeInTheDocument()
      expect(screen.getByText("file://<redacted>/Live.mkv")).toBeInTheDocument()
      expect(screen.queryByText("/mnt/private/raw/Live.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()
      expect(calls).toContainEqual({
        path: [
          "/admin/v1/acquisition/intake/candidates",
          "?library_id=library-a&state=ready&source_kind=watch_folder",
          "&managed_import_artifact_id=artifact-live&limit=25&offset=50",
        ].join(""),
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("writes Admin acquisition intake filter state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/acquisition/intake")

    await user.type(await screen.findByLabelText("媒体库 ID", {}, { timeout: 5000 }), "library-anime")
    await user.click(screen.getByRole("button", { name: "应用筛选" }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/acquisition/intake")
      expect(router.state.location.search).toMatchObject({ library_id: "library-anime" })
    })
  })

  it("renders live Admin generated artifact proposals without exposing raw sensitive fields", async () => {
    const originalFetch = globalThis.fetch
    const previousProfile = window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)
    const previousSession = window.sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)
    const calls: Array<{ path: string; authorization: string | null }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      calls.push({
        path: `${url.pathname}${url.search}`,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      if (url.pathname === "/admin/v1/automation/generated-artifacts/proposals") {
        return jsonResponse(adminGeneratedArtifactProposalsResponse())
      }

      return jsonResponse({ code: "not_found", message: "not found" }, 404)
    })

    window.localStorage.setItem(
      CONNECTION_PROFILE_STORAGE_KEY,
      JSON.stringify({
        mode: "live",
        runtime: "browser",
        baseUrl: "http://nako-admin.test",
      }),
    )
    window.sessionStorage.setItem(
      CONNECTION_SESSION_STORAGE_KEY,
      JSON.stringify({
        bearerToken: "admin-token",
      }),
    )
    vi.stubGlobal("fetch", fetcher)

    try {
      renderRoute("/admin/automation/generated-artifacts?limit=25&offset=50")

      expect(await screen.findByText("artifact-live", {}, { timeout: 5000 })).toBeInTheDocument()
      expect(screen.getByText("Live Automation Provider")).toBeInTheDocument()
      expect(screen.queryByText("unsafe prompt body")).not.toBeInTheDocument()
      expect(screen.queryByText("unsafe generated payload title")).not.toBeInTheDocument()
      expect(screen.queryByText("provider secret response")).not.toBeInTheDocument()
      expect(screen.queryByText("F:\\private\\source\\Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("file:///mnt/private/source/Movie.mkv")).not.toBeInTheDocument()
      expect(screen.queryByText("admin-token")).not.toBeInTheDocument()
      expect(calls).toContainEqual({
        path: "/admin/v1/automation/generated-artifacts/proposals?limit=25&offset=50",
        authorization: "Bearer admin-token",
      })
    } finally {
      vi.stubGlobal("fetch", originalFetch)
      restoreStorage(window.localStorage, CONNECTION_PROFILE_STORAGE_KEY, previousProfile)
      restoreStorage(window.sessionStorage, CONNECTION_SESSION_STORAGE_KEY, previousSession)
    }
  })

  it("writes Admin generated artifact pagination state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/admin/automation/generated-artifacts?limit=1")

    expect(await screen.findByText("fixture-generated-artifact-1", {}, { timeout: 5000 })).toBeInTheDocument()
    await user.click(screen.getByRole("button", { name: /下一页/ }))

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/admin/automation/generated-artifacts")
      expect(router.state.location.search).toMatchObject({ limit: 1, offset: 1 })
    })
  })

  it("writes Media search submits to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/search")

    const input = await screen.findByPlaceholderText("搜索电影、剧集、演员...", {}, { timeout: 5000 })
    await user.type(input, "dune")
    const searchButtons = screen.getAllByRole("button", { name: "搜索" })
    await user.click(searchButtons[searchButtons.length - 1])

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ q: "dune" })
    })
  })

  it("writes Media playlist tab and view state to the URL", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/my-list")

    const favoritesTab = await screen.findByRole("tab", { name: /收藏/ }, { timeout: 5000 })
    await user.click(favoritesTab)

    await waitFor(() => {
      expect(router.state.location.pathname).toBe("/media/my-list")
      expect(router.state.location.search).toMatchObject({ playlist: "fixture-favorites" })
    })

    await user.click(await screen.findByRole("button", { name: "列表视图" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        playlist: "fixture-favorites",
        view: "list",
      })
    })
  })

  it("creates a Media playlist and writes the selected playlist to the URL", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlists = [publicUserPlaylist()]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists,
          page: {
            limit: 20,
            offset: 0,
            returned: playlists.length,
          },
        })
      }

      if (method === "POST" && url.pathname === "/users/me/playlists") {
        const created = publicUserPlaylist({
          id: "playlist-new",
          name: (body as { name: string }).name,
          item_count: 0,
          version: 1,
        })
        playlists = [...playlists, created]

        return jsonResponse({ playlist: created })
      }

      if (method === "GET" && url.pathname.startsWith("/users/me/playlists/")) {
        const playlistId = url.pathname.split("/")[4]
        const playlist = playlists.find((entry) => entry.id === playlistId) ?? playlists[0]

        return jsonResponse({
          playlist,
          items: [],
          page: {
            limit: 50,
            offset: 0,
            returned: 0,
          },
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
    const { router } = renderRoute("/media/my-list?playlist=playlist-live")

    expect(await screen.findAllByText("Live Playlist", {}, { timeout: 5000 })).toHaveLength(2)
    await user.click(await screen.findByRole("button", { name: "新建播放列表" }))
    await user.type(await screen.findByLabelText("播放列表名称"), "Weekend Queue")
    await user.click(screen.getByRole("button", { name: "创建播放列表" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({ playlist: "playlist-new" })
    })
    expect(calls).toContainEqual({
      method: "POST",
      path: "/users/me/playlists",
      body: { name: "Weekend Queue" },
    })
  })

  it("renames and deletes the active Media playlist without leaving stale URL state", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlists = [
      publicUserPlaylist(),
      publicUserPlaylist({
        id: "playlist-archive",
        name: "Archive Playlist",
        item_count: 0,
        version: 1,
      }),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists,
          page: {
            limit: 20,
            offset: 0,
            returned: playlists.length,
          },
        })
      }

      if (method === "PATCH" && url.pathname === "/users/me/playlists/playlist-live") {
        const renamed = publicUserPlaylist({
          name: (body as { name: string }).name,
          version: 3,
        })
        playlists = playlists.map((playlist) => (playlist.id === renamed.id ? renamed : playlist))

        return jsonResponse({ playlist: renamed })
      }

      if (method === "DELETE" && url.pathname === "/users/me/playlists/playlist-live") {
        playlists = playlists.filter((playlist) => playlist.id !== "playlist-live")

        return jsonResponse({ playlist_id: "playlist-live", deleted: true })
      }

      if (method === "GET" && url.pathname.startsWith("/users/me/playlists/")) {
        const playlistId = url.pathname.split("/")[4]
        const playlist = playlists.find((entry) => entry.id === playlistId) ?? playlists[0]

        return jsonResponse({
          playlist,
          items: [],
          page: {
            limit: 50,
            offset: 0,
            returned: 0,
          },
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
    const { router } = renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findAllByText("Live Playlist", {}, { timeout: 5000 })).toHaveLength(2)
    await user.click(await screen.findByRole("button", { name: "重命名播放列表" }))
    const nameInput = await screen.findByLabelText("播放列表名称")
    await user.clear(nameInput)
    await user.type(nameInput, "Renamed Playlist")
    await user.click(screen.getByRole("button", { name: "保存名称" }))

    await waitFor(() => {
      expect(screen.getAllByText("Renamed Playlist")).toHaveLength(2)
    })
    expect(calls).toContainEqual({
      method: "PATCH",
      path: "/users/me/playlists/playlist-live",
      body: { name: "Renamed Playlist", expected_version: 2 },
    })

    await user.click(screen.getByRole("button", { name: "删除播放列表" }))
    await user.click(await screen.findByRole("button", { name: "确认删除" }))

    await waitFor(() => {
      expect(router.state.location.search).toMatchObject({
        playlist: "playlist-archive",
        view: "list",
      })
    })
    expect(calls).toContainEqual({
      method: "DELETE",
      path: "/users/me/playlists/playlist-live",
      body: undefined,
    })
  })

  it("keeps fixture Media playlist mutations truthful and does not update URL state", async () => {
    const user = userEvent.setup()
    const { router } = renderRoute("/media/my-list")

    await user.click(await screen.findByRole("button", { name: "新建播放列表" }, { timeout: 5000 }))
    await user.type(await screen.findByLabelText("播放列表名称"), "Fixture Queue")
    await user.click(screen.getByRole("button", { name: "创建播放列表" }))

    expect(
      await screen.findByText("Fixture mode does not persist playlist mutations.", {}, { timeout: 5000 }),
    ).toBeInTheDocument()
    expect(router.state.location.search).not.toMatchObject({ playlist: "Fixture Queue" })
  })

  it("removes an item from the active Media playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistItems = [
      {
        playlist_id: "playlist-live",
        item_id: "live-movie",
        position: 0,
        added_at: "2026-05-29T01:00:00Z",
        item: publicMediaItem(),
        images: [],
      },
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "DELETE" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        playlistItems = []

        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 0,
            version: 3,
          }),
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
    renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie", {}, { timeout: 5000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "从播放列表移除 Live Movie" }))

    await waitFor(() => {
      expect(screen.getByText("列表为空")).toBeInTheDocument()
    })
    expect(calls).toContainEqual({
      method: "DELETE",
      path: "/users/me/playlists/playlist-live/items/live-movie",
      body: undefined,
    })
  })

  it("reorders Media playlist items through the Public Client route without changing URL state", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistVersion = 2
    let playlistItems = [
      publicPlaylistItem("live-movie-a", "Live Movie A", 0),
      publicPlaylistItem("live-movie-b", "Live Movie B", 1),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/reorder") {
        const itemIds = (body as { item_ids: string[] }).item_ids
        playlistItems = itemIds.map((itemId, index) => {
          const item = playlistItems.find((entry) => entry.item_id === itemId)
          if (!item) {
            throw new Error(`unknown playlist item: ${itemId}`)
          }

          return { ...item, position: index }
        })
        playlistVersion = 3

        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: playlistItems.length,
            version: playlistVersion,
          }),
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
    const { router } = renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie A", {}, { timeout: 5000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "将 Live Movie B 上移" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: {
          item_ids: ["live-movie-b", "live-movie-a"],
          expected_version: 2,
        },
      })
    })
    await waitFor(() => {
      expect(
        screen
          .getAllByRole("heading", { level: 3 })
          .map((heading) => heading.textContent),
      ).toEqual(["Live Movie B", "Live Movie A"])
    })
    expect(router.state.location.search).toMatchObject({
      playlist: "playlist-live",
      view: "list",
    })
  })

  it("refetches playlist items when reorder returns a stale version conflict", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    let playlistVersion = 2
    let playlistItems = [
      publicPlaylistItem("live-movie-a", "Live Movie A", 0),
      publicPlaylistItem("live-movie-b", "Live Movie B", 1),
    ]
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion })],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists/playlist-live/items") {
        return jsonResponse({
          playlist: publicUserPlaylist({ item_count: playlistItems.length, version: playlistVersion }),
          items: playlistItems,
          page: {
            limit: 50,
            offset: 0,
            returned: playlistItems.length,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/reorder") {
        playlistItems = [
          publicPlaylistItem("live-movie-b", "Live Movie B", 0),
          publicPlaylistItem("live-movie-a", "Live Movie A", 1),
        ]
        playlistVersion = 4

        return jsonResponse(
          {
            code: "conflict",
            message: "Playlist version conflict",
          },
          409,
        )
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
    renderRoute("/media/my-list?playlist=playlist-live&view=list")

    expect(await screen.findByText("Live Movie A", {}, { timeout: 5000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "将 Live Movie B 上移" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: {
          item_ids: ["live-movie-b", "live-movie-a"],
          expected_version: 2,
        },
      })
    })
    expect(await screen.findByText("Playlist version conflict", {}, { timeout: 5000 })).toBeInTheDocument()
    await waitFor(() => {
      expect(
        calls.filter((call) => call.method === "GET" && call.path === "/users/me/playlists/playlist-live/items")
          .length,
      ).toBeGreaterThan(1)
    })
    await waitFor(() => {
      expect(
        screen
          .getAllByRole("heading", { level: 3 })
          .map((heading) => heading.textContent),
      ).toEqual(["Live Movie B", "Live Movie A"])
    })
  })

  it("adds detail media to a selected playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/items/live-movie") {
        return jsonResponse({
          item: publicMediaItem(),
          sources: [],
          images: [],
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 2,
            version: 3,
          }),
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
    renderRoute("/media/detail?id=live-movie&type=movie")

    expect(await screen.findByRole("heading", { name: "Live Movie" }, { timeout: 5000 })).toBeInTheDocument()
    await user.click(await screen.findByRole("button", { name: "添加到播放列表" }))
    await user.click(await screen.findByRole("menuitem", { name: "添加到 Live Playlist" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: {},
      })
    })
  })

  it("adds browse media cards to a selected playlist through the Public Client route", async () => {
    const user = userEvent.setup()
    const calls: Array<{ method: string; path: string; body?: unknown }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({ method, path: url.pathname, body })

      if (method === "GET" && url.pathname === "/items") {
        return jsonResponse({
          items: [publicMediaItem()],
          page: {
            limit: 40,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "GET" && url.pathname === "/users/me/playlists") {
        return jsonResponse({
          playlists: [publicUserPlaylist()],
          page: {
            limit: 20,
            offset: 0,
            returned: 1,
          },
        })
      }

      if (method === "PUT" && url.pathname === "/users/me/playlists/playlist-live/items/live-movie") {
        return jsonResponse({
          playlist: publicUserPlaylist({
            item_count: 2,
            version: 3,
          }),
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
    renderRoute("/media")

    await user.click(await screen.findByRole("button", { name: "添加 Live Movie 到播放列表" }, { timeout: 5000 }))
    await user.click(await screen.findByRole("menuitem", { name: "添加到 Live Playlist" }))

    await waitFor(() => {
      expect(calls).toContainEqual({
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: {},
      })
    })
  })
})

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
}

function restoreStorage(storage: Storage, key: string, value: string | null) {
  if (value === null) {
    storage.removeItem(key)
    return
  }

  storage.setItem(key, value)
}

function adminAcquisitionIntakeCandidatesResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    candidates: [
      {
        id: "candidate-live",
        target_library_id: "library-a",
        source_kind: "watch_folder",
        custom_source_kind: false,
        source_scheme: "file",
        source_ref_redacted: "file://<redacted>/Live.mkv",
        source_key_fingerprint: "sha256:candidate-live",
        has_display_name: true,
        has_intended_locator: true,
        size_bytes: 123456789,
        has_fingerprint: true,
        managed_import_artifact_id: "artifact-live",
        state: "ready",
        has_diagnostics: true,
        first_seen_at_ms: 1710468000000,
        last_seen_at_ms: 1710468300000,
        created_at_ms: 1710468000000,
        updated_at_ms: 1710468300000,
        intended_locator: "file:///mnt/private/raw/Live.mkv",
        prompt_body: "unsafe prompt body",
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function adminGeneratedArtifactProposalsResponse() {
  return {
    admin_api_version: "v1",
    public_api_version: "v1",
    proposals: [
      {
        id: "artifact-live",
        kind: "metadata_suggestion",
        capability: "item_metadata_suggest",
        status: "pending_review",
        target: {
          kind: "media_item",
          library_id: "library-a",
          item_id: "item-live",
          source_id: "source-live",
          local_path: "F:\\private\\source\\Movie.mkv",
          source_locator: "file:///mnt/private/source/Movie.mkv",
        },
        provenance: {
          provider_id: "provider-live",
          provider_name: "Live Automation Provider",
          job_id: "job-live",
          capability: "item_metadata_suggest",
          idempotency_key_fingerprint: "sha256:idempotency-live",
          prompt_fingerprint: "sha256:prompt-live",
          attempt_count: 2,
          artifact_created_at: "2026-05-29T01:00:00Z",
          raw_prompt: "unsafe prompt body",
          provider_raw_response: "provider secret response",
        },
        payload: {
          valid_json: true,
          shape: "object",
          payload_fingerprint: "sha256:payload-live",
          payload_bytes: 4096,
          object_field_count: 9,
          array_item_count: null,
          has_textual_values: true,
          has_explanation: true,
          confidence_milli: 910,
          raw_payload: {
            title: "unsafe generated payload title",
          },
        },
        readiness: {
          status: "ready",
          actionable: true,
          reasons: ["ready_for_review"],
        },
        created_at: "2026-05-29T01:01:00Z",
        updated_at: "2026-05-29T01:05:00Z",
        accepted_at: null,
      },
    ],
    page: {
      limit: 25,
      offset: 50,
      returned: 1,
    },
  }
}

function publicUserPlaylist(overrides: Record<string, unknown> = {}) {
  return {
    id: "playlist-live",
    name: "Live Playlist",
    visibility: "private",
    item_count: 1,
    created_at: "2026-05-29T00:00:00Z",
    updated_at: "2026-05-29T01:00:00Z",
    version: 2,
    ...overrides,
  }
}

function publicPlaylistItem(itemId: string, title: string, position: number) {
  const baseItem = publicMediaItem()

  return {
    playlist_id: "playlist-live",
    item_id: itemId,
    position,
    added_at: "2026-05-29T01:00:00Z",
    item: publicMediaItem({
      id: itemId,
      metadata: {
        ...baseItem.metadata,
        original_title: title,
        title,
      },
    }),
    images: [],
  }
}

function publicMediaItem(overrides: Record<string, unknown> = {}) {
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
    ...overrides,
  }
}
