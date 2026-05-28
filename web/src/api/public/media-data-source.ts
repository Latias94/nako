import {
  NakoClient,
  type BrowserPlaybackTicketResponse,
  type BrowserPlaybackUrlDto,
  type FetchLike,
  type MediaStreamDto,
} from "@nako/sdk"
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

export type PublicPlaybackSubtitleTrack = {
  id: string
  streamIndex: number
  language: string
  srcLang: string
  url: string
  contentType: string
  default: boolean
  forced: boolean
  sdh: boolean
}

export type PublicPlaybackPlan = {
  itemId: string
  sourceId?: string
  mode?: "direct" | "remux" | "hls"
  mediaUrl?: string
  mediaContentType?: string
  subtitles: PublicPlaybackSubtitleTrack[]
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicMediaDataSource = {
  listMedia(): Promise<PublicMediaItemsPayload>
  searchMedia(query: string): Promise<PublicMediaItemsPayload>
  getMediaDetails(id: string, mediaType: MediaItem["type"]): Promise<PublicMediaDetailPayload>
  loadPlaybackPlan(
    itemId: string,
    mediaType: MediaItem["type"],
    sourceId?: string,
  ): Promise<PublicPlaybackPlan>
}

const BROWSER_PLAYBACK_CAPABILITIES = {
  direct_play: true,
  supports_subtitles: true,
  hls_variant_policy: "single_variant",
  hls_segment_container: "mpeg_ts",
} as const

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
    async loadPlaybackPlan(itemId, mediaType, requestedSourceId) {
      try {
        const detail = await client.getItem(itemId)
        const selectedSource =
          detail.sources.find((source) => source.id === requestedSourceId) ?? detail.sources[0]

        if (!selectedSource) {
          return fixturePlaybackPlan(
            itemId,
            requestedSourceId,
            mediaType,
            new Error("No playable media source is available"),
          )
        }

        const [decision, probe] = await Promise.all([
          client.getPlaybackDecision(selectedSource.id, BROWSER_PLAYBACK_CAPABILITIES),
          client.getSourceProbe(selectedSource.id),
        ])
        const mode = browserModeForDecision(decision.decision.mode)
        const mediaTicket = await client.createBrowserPlaybackTicket(selectedSource.id, {
          mode,
          capabilities: BROWSER_PLAYBACK_CAPABILITIES,
        })
        const subtitleStreams = playableSubtitleStreams(probe.probe.streams)
        const subtitleTickets = await Promise.all(
          subtitleStreams.map((stream) =>
            client.createBrowserPlaybackTicket(selectedSource.id, {
              mode: "subtitle",
              subtitle_stream_index: stream.index,
            }),
          ),
        )

        return livePlaybackPlan({
          itemId,
          sourceId: selectedSource.id,
          mode,
          baseUrl: connection.baseUrl,
          mediaTicket,
          subtitleStreams,
          subtitleTickets,
        })
      } catch (error) {
        return fixturePlaybackPlan(itemId, requestedSourceId, mediaType, error)
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
    async loadPlaybackPlan(itemId, mediaType, sourceId) {
      return fixturePlaybackPlan(itemId, sourceId, mediaType)
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

function livePlaybackPlan(input: {
  itemId: string
  sourceId: string
  mode: PublicPlaybackPlan["mode"]
  baseUrl: string
  mediaTicket: BrowserPlaybackTicketResponse
  subtitleStreams: MediaStreamDto[]
  subtitleTickets: BrowserPlaybackTicketResponse[]
}): PublicPlaybackPlan {
  const mediaUrl = playbackUrlByKind(input.mediaTicket, input.mode === "hls" ? "playlist" : "stream")

  return {
    itemId: input.itemId,
    sourceId: input.sourceId,
    mode: input.mode,
    mediaUrl: mediaUrl ? absolutePublicUrl(input.baseUrl, mediaUrl.url) : undefined,
    mediaContentType: mediaUrl?.content_type,
    subtitles: input.subtitleStreams.flatMap((stream, index) => {
      const url = playbackUrlByKind(input.subtitleTickets[index], "subtitle")
      if (!url) {
        return []
      }

      return [
        {
          id: `subtitle-${stream.index}`,
          streamIndex: stream.index,
          language: subtitleLabel(stream),
          srcLang: stream.language ?? "und",
          url: absolutePublicUrl(input.baseUrl, url.url),
          contentType: url.content_type,
          default: stream.disposition.default,
          forced: stream.disposition.forced,
          sdh: stream.disposition.hearing_impaired,
        },
      ]
    }),
    fallback: false,
    source: "live",
  }
}

function fixturePlaybackPlan(
  itemId: string,
  sourceId: string | undefined,
  _mediaType: MediaItem["type"],
  error?: unknown,
): PublicPlaybackPlan {
  return {
    itemId,
    sourceId,
    subtitles: [],
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function browserModeForDecision(
  mode: "direct_play" | "remux" | "transcode" | "denied",
): "direct" | "remux" | "hls" {
  switch (mode) {
    case "remux":
      return "remux"
    case "transcode":
      return "hls"
    case "direct_play":
    case "denied":
      return "direct"
  }
}

function playableSubtitleStreams(streams: MediaStreamDto[]) {
  return streams.filter(
    (stream) =>
      stream.kind === "subtitle" &&
      stream.origin === "sidecar" &&
      isPlayableSubtitleCodec(stream.codec),
  )
}

function isPlayableSubtitleCodec(codec: string | null) {
  return codec === "srt" || codec === "vtt" || codec === "webvtt"
}

function playbackUrlByKind(
  ticket: BrowserPlaybackTicketResponse,
  kind: BrowserPlaybackUrlDto["kind"],
) {
  return ticket.urls.find((url) => url.kind === kind)
}

function absolutePublicUrl(baseUrl: string, pathOrUrl: string) {
  return new URL(pathOrUrl, baseUrl).toString()
}

function subtitleLabel(stream: MediaStreamDto) {
  const language = stream.language ?? "Unknown"
  const badges = [
    stream.disposition.forced ? "Forced" : null,
    stream.disposition.hearing_impaired ? "SDH" : null,
    stream.disposition.commentary ? "Commentary" : null,
  ].filter(Boolean)

  return badges.length > 0 ? `${language} (${badges.join(", ")})` : language
}

function errorMessage(error: unknown) {
  if (!error) {
    return undefined
  }

  return error instanceof Error ? error.message : "Public Client API request failed"
}
