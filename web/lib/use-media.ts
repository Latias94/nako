import { useQuery } from "@tanstack/react-query"
import {
  createPublicMediaDataSource,
  type PublicLibraryItemsQuery,
  type PublicMediaDetailPayload,
  type PublicMediaItemsPayload,
  type PublicUserPlaylistItemsPayload,
  type PublicUserPlaylistsPayload,
} from "@/src/api/public/media-data-source"
import type { PlaybackSessionHeartbeatRequest } from "@nako/sdk"
import type { MediaItem } from "./media-types"

export function useTrendingMedia() {
  return useQuery({
    queryKey: ["nako", "media", "trending"],
    queryFn: () => createPublicMediaDataSource().listMedia(),
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useCategoryMedia() {
  const { data, isLoading, error } = useTrendingMedia()

  const items = data?.items || []
  const movies = items.filter((item) => item.type === "movie")
  const series = items.filter((item) => item.type === "series")

  const categories = [
    {
      title: "为你推荐",
      items: items.slice(0, 8),
    },
    {
      title: "热门电影",
      items: movies.slice(0, 8),
    },
    {
      title: "热门剧集",
      items: series.slice(0, 8),
    },
  ].filter((category) => category.items.length > 0)

  return {
    categories,
    isLoading,
    error,
    fallback: data?.fallback || false,
  }
}

export function useSearchMedia(query: string) {
  return useQuery({
    queryKey: ["nako", "media", "search", query],
    queryFn: () => createPublicMediaDataSource().searchMedia(query),
    enabled: query.trim().length > 0,
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useMediaDetails(id: string, mediaType: "movie" | "series") {
  return useQuery({
    queryKey: ["nako", "media", "details", mediaType, id],
    queryFn: async (): Promise<PublicMediaDetailPayload> =>
      createPublicMediaDataSource().getMediaDetails(id, mediaType),
    enabled: !!id,
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useContinueWatchingMedia() {
  return useQuery({
    queryKey: ["nako", "media", "continue-watching"],
    queryFn: () => createPublicMediaDataSource().listContinueWatching(),
    staleTime: 60 * 1000,
    retry: 0,
  })
}

export function useUserPlaylists() {
  return useQuery({
    queryKey: ["nako", "media", "user-playlists"],
    queryFn: (): Promise<PublicUserPlaylistsPayload> =>
      createPublicMediaDataSource().listUserPlaylists(),
    staleTime: 60 * 1000,
    retry: 0,
  })
}

export function useUserPlaylistItems(playlistId?: string) {
  return useQuery({
    queryKey: ["nako", "media", "user-playlists", playlistId, "items"],
    queryFn: (): Promise<PublicUserPlaylistItemsPayload> =>
      createPublicMediaDataSource().listUserPlaylistItems(playlistId ?? ""),
    enabled: !!playlistId,
    staleTime: 60 * 1000,
    retry: 0,
  })
}

export function useLibraryReadiness(libraryId: string) {
  return useQuery({
    queryKey: ["nako", "media", "library", libraryId, "readiness"],
    queryFn: () => createPublicMediaDataSource().getLibraryReadiness(libraryId),
    enabled: !!libraryId,
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useLibraryItems(
  libraryId: string,
  query: PublicLibraryItemsQuery,
  enabled = true,
) {
  return useQuery({
    queryKey: ["nako", "media", "library", libraryId, "items", query],
    queryFn: (): Promise<PublicMediaItemsPayload> =>
      createPublicMediaDataSource().listLibraryItems(libraryId, query),
    enabled: !!libraryId && enabled,
    staleTime: 60 * 1000,
    retry: 0,
  })
}

export async function searchPublicMedia(query: string): Promise<PublicMediaItemsPayload> {
  return createPublicMediaDataSource().searchMedia(query)
}

export async function heartbeatPublicPlaybackSession(
  sessionId: string,
  body: PlaybackSessionHeartbeatRequest,
) {
  await createPublicMediaDataSource().heartbeatPlaybackSession(sessionId, body)
}

export function usePlaybackPlan(
  itemId: string,
  mediaType: "movie" | "series",
  sourceId?: string,
) {
  return useQuery({
    queryKey: ["nako", "media", "playback-plan", mediaType, itemId, sourceId ?? "auto"],
    queryFn: () => createPublicMediaDataSource().loadPlaybackPlan(itemId, mediaType, sourceId),
    enabled: !!itemId,
    staleTime: 30 * 1000,
    retry: 0,
  })
}
