import { createMemoryHistory } from "@tanstack/react-router"
import { render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, describe, expect, it, vi } from "vitest"
import { NakoRouter, createNakoRouter } from "@/src/shell"
import { ThemeProvider } from "@/components/theme-provider"
import { QueryProvider } from "@/lib/query-provider"
import { CONNECTION_PROFILE_STORAGE_KEY } from "@/src/api/connection-profile"

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
})

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json",
    },
  })
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
