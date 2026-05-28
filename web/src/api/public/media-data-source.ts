import { NakoClient, type FetchLike } from "@nako/sdk"
import { mapPublicMediaItem, type MediaItem } from "@/lib/media-types"
import { loadPublicClientConnection, type PublicClientConnection } from "./connection"
import { LOCAL_MEDIA_ITEMS, matchesLocalMediaQuery } from "./media-fixtures"

export type PublicMediaSourceMode = "live" | "fixture"

export type PublicMediaItemsPayload = {
  items: MediaItem[]
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicMediaDetailPayload = {
  item: MediaItem | null
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicMediaDataSource = {
  listMedia(): Promise<PublicMediaItemsPayload>
  searchMedia(query: string): Promise<PublicMediaItemsPayload>
  getMediaDetails(id: string, mediaType: MediaItem["type"]): Promise<PublicMediaDetailPayload>
}

export function createPublicMediaDataSource(
  connection: PublicClientConnection = loadPublicClientConnection(),
  fetcher?: FetchLike,
): PublicMediaDataSource {
  if (connection.mode === "fixture") {
    return createFixtureMediaDataSource()
  }

  return createLiveMediaDataSource(connection, fetcher)
}

function createLiveMediaDataSource(
  connection: Extract<PublicClientConnection, { mode: "live" }>,
  fetcher?: FetchLike,
): PublicMediaDataSource {
  const client = new NakoClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetch: fetcher,
  })

  return {
    async listMedia() {
      try {
        const response = await client.listItems({ limit: 40, offset: 0 })
        return liveItems(response.items.map(mapPublicMediaItem))
      } catch (error) {
        return fixtureItems(LOCAL_MEDIA_ITEMS, error)
      }
    },
    async searchMedia(query) {
      if (!query.trim()) {
        return liveItems([])
      }

      try {
        const response = await client.searchItems({ q: query, limit: 20, offset: 0 })
        return liveItems(response.hits.map((hit) => mapPublicMediaItem(hit.item)))
      } catch (error) {
        return fixtureItems(
          LOCAL_MEDIA_ITEMS.filter((item) => matchesLocalMediaQuery(item, query)),
          error,
        )
      }
    },
    async getMediaDetails(id, mediaType) {
      try {
        const response = await client.getItem(id)
        return liveDetail(mapPublicMediaItem(response.item))
      } catch (error) {
        return fixtureDetail(
          LOCAL_MEDIA_ITEMS.find((entry) => entry.id === id && entry.type === mediaType) ?? null,
          error,
        )
      }
    },
  }
}

function createFixtureMediaDataSource(): PublicMediaDataSource {
  return {
    async listMedia() {
      return fixtureItems(LOCAL_MEDIA_ITEMS)
    },
    async searchMedia(query) {
      if (!query.trim()) {
        return fixtureItems([])
      }

      return fixtureItems(LOCAL_MEDIA_ITEMS.filter((item) => matchesLocalMediaQuery(item, query)))
    },
    async getMediaDetails(id, mediaType) {
      return fixtureDetail(
        LOCAL_MEDIA_ITEMS.find((entry) => entry.id === id && entry.type === mediaType) ?? null,
      )
    },
  }
}

function liveItems(items: MediaItem[]): PublicMediaItemsPayload {
  return {
    items,
    fallback: false,
    source: "live",
  }
}

function liveDetail(item: MediaItem): PublicMediaDetailPayload {
  return {
    item,
    fallback: false,
    source: "live",
  }
}

function fixtureItems(items: MediaItem[], error?: unknown): PublicMediaItemsPayload {
  return {
    items,
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function fixtureDetail(item: MediaItem | null, error?: unknown): PublicMediaDetailPayload {
  return {
    item,
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function errorMessage(error: unknown) {
  if (!error) {
    return undefined
  }

  return error instanceof Error ? error.message : "Public Client API request failed"
}
