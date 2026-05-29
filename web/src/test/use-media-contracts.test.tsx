import { QueryClient, QueryClientProvider } from "@tanstack/react-query"
import { renderHook } from "@testing-library/react"
import type { ReactNode } from "react"
import { afterEach, describe, expect, it, vi } from "vitest"
import { savePublicClientConnection } from "@/src/api/public/connection"
import {
  useAddUserPlaylistItemMutation,
  useCreateUserPlaylistMutation,
  useDeleteUserPlaylistMutation,
  useRemoveUserPlaylistItemMutation,
  useReorderUserPlaylistItemsMutation,
  useUpdateUserPlaylistMutation,
  userPlaylistItemsQueryKey,
  userPlaylistsQueryKey,
} from "@/lib/use-media"

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

function createQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  })
}

function queryWrapper(queryClient: QueryClient) {
  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  }
}

afterEach(() => {
  vi.unstubAllGlobals()
})

describe("public media query hooks", () => {
  it("runs User Playlist mutation hooks through Public Client and invalidates playlist caches", async () => {
    const queryClient = createQueryClient()
    const invalidateQueries = vi.spyOn(queryClient, "invalidateQueries")
    const removeQueries = vi.spyOn(queryClient, "removeQueries")
    const calls: Array<{
      method: string
      path: string
      body?: unknown
      authorization: string | null
    }> = []
    const fetcher = vi.fn<typeof fetch>(async (input, init) => {
      const url = new URL(String(input))
      const method = init?.method ?? "GET"
      const body = typeof init?.body === "string" ? JSON.parse(init.body) : undefined
      calls.push({
        method,
        path: url.pathname,
        body,
        authorization: new Headers(init?.headers).get("Authorization"),
      })

      switch (`${method} ${url.pathname}`) {
        case "POST /users/me/playlists":
          return jsonResponse({
            playlist: publicUserPlaylist({
              id: "playlist-new",
              name: (body as { name: string }).name,
              item_count: 0,
              version: 1,
            }),
          })
        case "PATCH /users/me/playlists/playlist-live":
          return jsonResponse({
            playlist: publicUserPlaylist({
              name: (body as { name: string }).name,
              version: 3,
            }),
          })
        case "DELETE /users/me/playlists/playlist-live":
          return jsonResponse({ playlist_id: "playlist-live", deleted: true })
        case "PUT /users/me/playlists/playlist-live/items/live-movie":
          return jsonResponse({
            playlist: publicUserPlaylist({
              item_count: 2,
              version: 4,
            }),
          })
        case "DELETE /users/me/playlists/playlist-live/items/live-movie":
          return jsonResponse({
            playlist: publicUserPlaylist({
              item_count: 0,
              version: 5,
            }),
          })
        case "PUT /users/me/playlists/playlist-live/items/reorder":
          return jsonResponse({
            playlist: publicUserPlaylist({
              version: 6,
            }),
          })
        default:
          return jsonResponse({ message: "not found" }, 404)
      }
    })
    vi.stubGlobal("fetch", fetcher)
    savePublicClientConnection({
      mode: "live",
      baseUrl: "http://nako.test",
      bearerToken: "public-token",
    })

    const { result } = renderHook(
      () => ({
        createPlaylist: useCreateUserPlaylistMutation(),
        updatePlaylist: useUpdateUserPlaylistMutation(),
        deletePlaylist: useDeleteUserPlaylistMutation(),
        addItem: useAddUserPlaylistItemMutation(),
        removeItem: useRemoveUserPlaylistItemMutation(),
        reorderItems: useReorderUserPlaylistItemsMutation(),
      }),
      { wrapper: queryWrapper(queryClient) },
    )

    await result.current.createPlaylist.mutateAsync({ name: "New Queue" })
    await result.current.updatePlaylist.mutateAsync({
      playlistId: "playlist-live",
      body: { name: "Renamed Queue", expected_version: 2 },
    })
    await result.current.addItem.mutateAsync({
      playlistId: "playlist-live",
      itemId: "live-movie",
      body: { position: 1, expected_version: 3 },
    })
    await result.current.removeItem.mutateAsync({
      playlistId: "playlist-live",
      itemId: "live-movie",
    })
    await result.current.reorderItems.mutateAsync({
      playlistId: "playlist-live",
      body: { item_ids: ["live-movie-b", "live-movie-a"], expected_version: 5 },
    })
    await result.current.deletePlaylist.mutateAsync("playlist-live")

    expect(calls).toEqual([
      {
        method: "POST",
        path: "/users/me/playlists",
        body: { name: "New Queue" },
        authorization: "Bearer public-token",
      },
      {
        method: "PATCH",
        path: "/users/me/playlists/playlist-live",
        body: { name: "Renamed Queue", expected_version: 2 },
        authorization: "Bearer public-token",
      },
      {
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: { position: 1, expected_version: 3 },
        authorization: "Bearer public-token",
      },
      {
        method: "DELETE",
        path: "/users/me/playlists/playlist-live/items/live-movie",
        body: undefined,
        authorization: "Bearer public-token",
      },
      {
        method: "PUT",
        path: "/users/me/playlists/playlist-live/items/reorder",
        body: { item_ids: ["live-movie-b", "live-movie-a"], expected_version: 5 },
        authorization: "Bearer public-token",
      },
      {
        method: "DELETE",
        path: "/users/me/playlists/playlist-live",
        body: undefined,
        authorization: "Bearer public-token",
      },
    ])
    expect(invalidateQueries).toHaveBeenCalledWith({ queryKey: userPlaylistsQueryKey })
    expect(invalidateQueries).toHaveBeenCalledWith({
      queryKey: userPlaylistItemsQueryKey("playlist-live"),
    })
    expect(removeQueries).toHaveBeenCalledWith({
      queryKey: userPlaylistItemsQueryKey("playlist-live"),
    })
  })
})
