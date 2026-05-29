import {
  NakoClient,
  type BrowserPlaybackTicketResponse,
  type BrowserPlaybackUrlDto,
  type ClientBrowseSortKey,
  type ClientSortOrder,
  type ClientWatchStateFilter,
  type ContinueWatchingItemDto,
  type FetchLike,
  type LibraryDto,
  type LibrarySourceResponse,
  type MediaStreamDto,
  type MediaSourceDto,
  type PageInfo,
  type PlaybackSessionHeartbeatRequest,
  type PublicImageRefDto,
  type SetWatchedStateRequest,
  type UpdatePlaybackProgressRequest,
  type UserPlaybackStateDto,
} from "@nako/sdk"
import { mapPublicMediaItem, type MediaItem } from "@/lib/media-types"
import { loadPublicClientConnection, type PublicClientConnection } from "./connection"
import { LOCAL_MEDIA_ITEMS, matchesLocalMediaQuery } from "./media-fixtures"

export type PublicMediaSourceMode = "live" | "fixture"

export type PublicMediaItemsPayload = {
  items: MediaItem[]
  page?: PublicPage
  readiness: PublicReadinessState[]
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicLibraryItemsQuery = {
  limit?: number
  offset?: number
  sort?: ClientBrowseSortKey
  order?: ClientSortOrder
  facet?: string | string[]
  watchState?: ClientWatchStateFilter
}

export type PublicMediaDetailPayload = {
  item: MediaItem | null
  sources: PublicMediaSourceRef[]
  images: PublicImageRef[]
  readiness: PublicReadinessState[]
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicReadinessState = {
  id: string
  status: "ready" | "missing_contract" | "fallback" | "error"
  message: string
  contract?: string
}

export type PublicPage = {
  limit: number
  offset: number
  returned: number
}

export type PublicImageRef = {
  id: string
  kind: string
  url: string
  width: number | null
  height: number | null
}

export type PublicMediaSourceRef = {
  id: string
  itemId: string
  libraryId: string
  fileName: string
  sizeBytes: number | null
}

export type PublicLibrarySummary = {
  id: string
  name: string
  domain: LibraryDto["options"]["domain"]
  preset: LibraryDto["options"]["preset"]
  namingStrategy: LibraryDto["options"]["naming_strategy"]
  roots: string[]
}

export type PublicLibrarySourceSummary = {
  source: PublicMediaSourceRef
  item: MediaItem | null
}

export type PublicLibrariesPayload = {
  libraries: PublicLibrarySummary[]
  page?: PublicPage
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicLibraryReadinessPayload = {
  library: PublicLibrarySummary | null
  sources: PublicLibrarySourceSummary[]
  itemBrowse: PublicReadinessState
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
  playbackSessionId?: string
  mode?: "direct" | "remux" | "hls"
  mediaUrl?: string
  mediaContentType?: string
  subtitles: PublicPlaybackSubtitleTrack[]
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicPlaybackState = {
  itemId: string
  sourceId: string | null
  resumePositionMs: number | null
  durationMs: number | null
  progressPercent: number | null
  watched: boolean
  watchedAt: string | null
  lastPlayedAt: string | null
  updatedAt: string | null
  version: number
}

export type PublicContinueWatchingItem = {
  item: MediaItem
  state: PublicPlaybackState
  images: PublicImageRef[]
}

export type PublicContinueWatchingPayload = {
  items: PublicContinueWatchingItem[]
  page?: PublicPage
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicPlaybackStatePayload = {
  state: PublicPlaybackState | null
  fallback: boolean
  source: PublicMediaSourceMode
  error?: string
}

export type PublicMediaDataSource = {
  listMedia(): Promise<PublicMediaItemsPayload>
  listLibraryItems(
    libraryId: string,
    query?: PublicLibraryItemsQuery,
  ): Promise<PublicMediaItemsPayload>
  searchMedia(query: string): Promise<PublicMediaItemsPayload>
  getMediaDetails(id: string, mediaType: MediaItem["type"]): Promise<PublicMediaDetailPayload>
  listLibraries(): Promise<PublicLibrariesPayload>
  getLibraryReadiness(libraryId: string): Promise<PublicLibraryReadinessPayload>
  listContinueWatching(): Promise<PublicContinueWatchingPayload>
  getPlaybackState(itemId: string): Promise<PublicPlaybackStatePayload>
  updatePlaybackProgress(
    itemId: string,
    body: UpdatePlaybackProgressRequest,
  ): Promise<PublicPlaybackStatePayload>
  setWatchedState(
    itemId: string,
    body: SetWatchedStateRequest,
  ): Promise<PublicPlaybackStatePayload>
  heartbeatPlaybackSession(
    sessionId: string,
    body: PlaybackSessionHeartbeatRequest,
  ): Promise<void>
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

const RECENTLY_ADDED_CONTRACT_GAP: PublicReadinessState = {
  id: "recently-added-sort",
  status: "missing_contract",
  message: "Public Client listItems does not expose a stable Recently Added sort/filter contract yet.",
  contract: "listItems({ sort: 'recently_added' })",
}

const LIBRARY_ITEM_BROWSE_READY: PublicReadinessState = {
  id: "library-scoped-item-browse",
  status: "ready",
  message: "Public Client exposes library-scoped item browse.",
  contract: "GET /libraries/{library_id}/items",
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
        return liveItems(response.items.map(mapPublicMediaItem), response.page, [
          RECENTLY_ADDED_CONTRACT_GAP,
        ])
      } catch (error) {
        return fixtureItems(LOCAL_MEDIA_ITEMS, error)
      }
    },
    async listLibraryItems(libraryId, query = {}) {
      try {
        const response = await client.listLibraryItems(libraryId, {
          limit: query.limit ?? 50,
          offset: query.offset ?? 0,
          sort: query.sort ?? "date_added",
          order: query.order ?? "desc",
          facet: query.facet,
          watch_state: query.watchState ?? "any",
        })

        return liveItems(response.items.map(mapPublicMediaItem), response.page)
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
        return liveItems(
          response.hits.map((hit) => mapPublicMediaItem(hit.item)),
          response.page,
        )
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
        return liveDetail(mapPublicMediaItem(response.item), {
          images: response.images.map(mapPublicImage),
          sources: response.sources.map(mapPublicMediaSource),
        })
      } catch (error) {
        return fixtureDetail(
          LOCAL_MEDIA_ITEMS.find((entry) => entry.id === id && entry.type === mediaType) ?? null,
          error,
        )
      }
    },
    async listLibraries() {
      try {
        const response = await client.listLibraries({ limit: 50, offset: 0 })
        return liveLibraries(response.libraries.map(mapPublicLibrary), response.page)
      } catch (error) {
        return fixtureLibraries(error)
      }
    },
    async getLibraryReadiness(libraryId) {
      try {
        const [libraryResponse, sourcesResponse] = await Promise.all([
          client.getLibrary(libraryId),
          client.listLibrarySources(libraryId, { limit: 20, offset: 0 }),
        ])

        return liveLibraryReadiness({
          library: mapPublicLibrary(libraryResponse.library),
          sources: sourcesResponse.sources.map(mapPublicLibrarySource),
        })
      } catch (error) {
        return fixtureLibraryReadiness(libraryId, error)
      }
    },
    async listContinueWatching() {
      try {
        const response = await client.listContinueWatching({ limit: 12, offset: 0 })
        return liveContinueWatching(response.items.map(mapPublicContinueWatchingItem), response.page)
      } catch (error) {
        return fixtureContinueWatching(error)
      }
    },
    async getPlaybackState(itemId) {
      try {
        const response = await client.getUserPlaybackState(itemId)
        return livePlaybackState(mapPublicPlaybackState(response.state))
      } catch (error) {
        return fixturePlaybackState(itemId, error)
      }
    },
    async updatePlaybackProgress(itemId, body) {
      try {
        const response = await client.updateUserPlaybackProgress(itemId, body)
        return livePlaybackState(mapPublicPlaybackState(response.state))
      } catch (error) {
        return fixturePlaybackState(itemId, error)
      }
    },
    async setWatchedState(itemId, body) {
      try {
        const response = await client.setUserWatchedState(itemId, body)
        return livePlaybackState(mapPublicPlaybackState(response.state))
      } catch (error) {
        return fixturePlaybackState(itemId, error)
      }
    },
    async heartbeatPlaybackSession(sessionId, body) {
      await client.heartbeatPlaybackSession(sessionId, body)
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
    async listLibraryItems() {
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
    async listLibraries() {
      return fixtureLibraries()
    },
    async getLibraryReadiness(libraryId) {
      return fixtureLibraryReadiness(libraryId)
    },
    async listContinueWatching() {
      return fixtureContinueWatching()
    },
    async getPlaybackState(itemId) {
      return fixturePlaybackState(itemId)
    },
    async updatePlaybackProgress(itemId, body) {
      return fixturePlaybackState(itemId, undefined, body.position_ms, body.duration_ms)
    },
    async setWatchedState(itemId, body) {
      return fixturePlaybackState(itemId, undefined, body.position_ms ?? null, body.duration_ms, body.watched)
    },
    async heartbeatPlaybackSession() {
      return undefined
    },
    async loadPlaybackPlan(itemId, mediaType, sourceId) {
      return fixturePlaybackPlan(itemId, sourceId, mediaType)
    },
  }
}

function liveItems(
  items: MediaItem[],
  page?: PageInfo,
  readiness: PublicReadinessState[] = [],
): PublicMediaItemsPayload {
  return {
    items,
    page: mapPage(page),
    readiness,
    fallback: false,
    source: "live",
  }
}

function liveDetail(
  item: MediaItem,
  extra: {
    images?: PublicImageRef[]
    sources?: PublicMediaSourceRef[]
    readiness?: PublicReadinessState[]
  } = {},
): PublicMediaDetailPayload {
  return {
    item,
    sources: extra.sources ?? [],
    images: extra.images ?? [],
    readiness: extra.readiness ?? [],
    fallback: false,
    source: "live",
  }
}

function fixtureItems(items: MediaItem[], error?: unknown): PublicMediaItemsPayload {
  return {
    items,
    readiness: [],
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function fixtureDetail(item: MediaItem | null, error?: unknown): PublicMediaDetailPayload {
  return {
    item,
    sources: [],
    images: [],
    readiness: [],
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function liveLibraries(libraries: PublicLibrarySummary[], page?: PageInfo): PublicLibrariesPayload {
  return {
    libraries,
    page: mapPage(page),
    fallback: false,
    source: "live",
  }
}

function fixtureLibraries(error?: unknown): PublicLibrariesPayload {
  return {
    libraries: fixtureLibrarySummaries(),
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function liveLibraryReadiness(input: {
  library: PublicLibrarySummary
  sources: PublicLibrarySourceSummary[]
}): PublicLibraryReadinessPayload {
  return {
    library: input.library,
    sources: input.sources,
    itemBrowse: LIBRARY_ITEM_BROWSE_READY,
    fallback: false,
    source: "live",
  }
}

function fixtureLibraryReadiness(
  libraryId: string,
  error?: unknown,
): PublicLibraryReadinessPayload {
  const libraries = fixtureLibrarySummaries()

  return {
    library: libraries.find((library) => library.id === libraryId) ?? libraries[0] ?? null,
    sources: LOCAL_MEDIA_ITEMS.map((item) => ({
      source: {
        id: `fixture-source-${item.id}`,
        itemId: item.id,
        libraryId,
        fileName: `${item.originalTitle || item.title}.mkv`,
        sizeBytes: null,
      },
      item,
    })),
    itemBrowse: {
      ...LIBRARY_ITEM_BROWSE_READY,
      status: error ? "fallback" : LIBRARY_ITEM_BROWSE_READY.status,
    },
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function mapPublicLibrary(library: LibraryDto): PublicLibrarySummary {
  return {
    id: library.id,
    name: library.name,
    domain: library.options.domain,
    preset: library.options.preset,
    namingStrategy: library.options.naming_strategy,
    roots: library.roots,
  }
}

function mapPublicLibrarySource(source: LibrarySourceResponse): PublicLibrarySourceSummary {
  return {
    source: mapPublicMediaSource(source.source),
    item: source.item ? mapPublicMediaItem(source.item) : null,
  }
}

function mapPublicMediaSource(source: MediaSourceDto): PublicMediaSourceRef {
  return {
    id: source.id,
    itemId: source.item_id,
    libraryId: source.library_id,
    fileName: source.file_name,
    sizeBytes: source.size_bytes,
  }
}

function mapPublicImage(image: PublicImageRefDto): PublicImageRef {
  return {
    id: image.id,
    kind: image.kind,
    url: image.url,
    width: image.width,
    height: image.height,
  }
}

function mapPage(page?: PageInfo): PublicPage | undefined {
  if (!page) {
    return undefined
  }

  return {
    limit: page.limit,
    offset: page.offset,
    returned: page.returned,
  }
}

function fixtureLibrarySummaries(): PublicLibrarySummary[] {
  return [
    {
      id: "movies",
      name: "电影",
      domain: "video",
      preset: "movies",
      namingStrategy: "movie",
      roots: ["/media/movies"],
    },
    {
      id: "series",
      name: "剧集",
      domain: "video",
      preset: "tv",
      namingStrategy: "series",
      roots: ["/media/series"],
    },
  ]
}

function liveContinueWatching(
  items: PublicContinueWatchingItem[],
  page?: PageInfo,
): PublicContinueWatchingPayload {
  return {
    items,
    page: mapPage(page),
    fallback: false,
    source: "live",
  }
}

function fixtureContinueWatching(error?: unknown): PublicContinueWatchingPayload {
  return {
    items: LOCAL_MEDIA_ITEMS.slice(0, 3).map((item, index) => ({
      item,
      state: fixturePlaybackStateRecord(item.id, [65, 40, 25][index] ?? 50),
      images: [],
    })),
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function livePlaybackState(state: PublicPlaybackState): PublicPlaybackStatePayload {
  return {
    state,
    fallback: false,
    source: "live",
  }
}

function fixturePlaybackState(
  itemId: string,
  error?: unknown,
  positionMs?: number | null,
  durationMs?: number | null,
  watched = false,
): PublicPlaybackStatePayload {
  return {
    state: fixturePlaybackStateRecord(
      itemId,
      durationMs && positionMs ? Math.round((positionMs / durationMs) * 100) : 0,
      positionMs,
      durationMs,
      watched,
    ),
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function fixturePlaybackStateRecord(
  itemId: string,
  progressPercent: number,
  resumePositionMs: number | null = null,
  durationMs: number | null = null,
  watched = false,
): PublicPlaybackState {
  return {
    itemId,
    sourceId: null,
    resumePositionMs,
    durationMs,
    progressPercent,
    watched,
    watchedAt: watched ? new Date(0).toISOString() : null,
    lastPlayedAt: null,
    updatedAt: null,
    version: 0,
  }
}

function mapPublicContinueWatchingItem(item: ContinueWatchingItemDto): PublicContinueWatchingItem {
  return {
    item: mapPublicMediaItem(item.item),
    state: mapPublicPlaybackState(item.state),
    images: item.images.map(mapPublicImage),
  }
}

function mapPublicPlaybackState(state: UserPlaybackStateDto): PublicPlaybackState {
  return {
    itemId: state.item_id,
    sourceId: state.source_id,
    resumePositionMs: state.resume_position_ms,
    durationMs: state.duration_ms,
    progressPercent: state.progress_percent,
    watched: state.watched,
    watchedAt: state.watched_at,
    lastPlayedAt: state.last_played_at,
    updatedAt: state.updated_at,
    version: state.version,
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
    playbackSessionId: input.mediaTicket.playback_session_id ?? undefined,
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
