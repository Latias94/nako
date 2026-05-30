import {
  normalizePublicManagementContext,
  type PublicManagementContext,
} from "@/src/api/public/management-context-model"

export type AdminManagementContextIntent = "scan_library" | "refresh_item_metadata"

export type AdminManagementContextPanel =
  | "metadata_profile"
  | "support"
  | "runtime"
  | "library_access"

export type AdminManagementContextRouteState = PublicManagementContext & {
  context?: "management_link"
  intent?: AdminManagementContextIntent
  panel?: AdminManagementContextPanel
}

export type AdminManagementContextRouteSearch = {
  context?: "management_link"
  intent?: AdminManagementContextIntent
  panel?: AdminManagementContextPanel
  library_id?: string
  item_id?: string
  media_type?: PublicManagementContext["mediaType"]
  source_id?: string
  playback_session_id?: string
}

const ADMIN_MANAGEMENT_CONTEXT_INTENTS: AdminManagementContextIntent[] = [
  "scan_library",
  "refresh_item_metadata",
]

const ADMIN_MANAGEMENT_CONTEXT_PANELS: AdminManagementContextPanel[] = [
  "metadata_profile",
  "support",
  "runtime",
  "library_access",
]

export function adminManagementContextFromSearch(
  search: Record<string, unknown>,
): AdminManagementContextRouteState {
  const routeSearch = adminManagementContextRouteSearchFromSearch(search)
  const context = normalizePublicManagementContext({
    libraryId: routeSearch.library_id,
    itemId: routeSearch.item_id,
    mediaType: routeSearch.media_type,
    sourceId: routeSearch.source_id,
    playbackSessionId: routeSearch.playback_session_id,
  })

  return {
    ...context,
    ...(routeSearch.context ? { context: routeSearch.context } : {}),
    ...(routeSearch.intent ? { intent: routeSearch.intent } : {}),
    ...(routeSearch.panel ? { panel: routeSearch.panel } : {}),
  }
}

export function adminManagementContextRouteSearchFromSearch(
  search: Record<string, unknown>,
): AdminManagementContextRouteSearch {
  const context = normalizePublicManagementContext({
    libraryId: searchString(search.library_id),
    itemId: searchString(search.item_id),
    mediaType: searchMediaType(search.media_type),
    sourceId: searchString(search.source_id),
    playbackSessionId: searchString(search.playback_session_id),
  })
  const intent = allowedSearchValue(search.intent, ADMIN_MANAGEMENT_CONTEXT_INTENTS)
  const panel = allowedSearchValue(search.panel, ADMIN_MANAGEMENT_CONTEXT_PANELS)

  return {
    ...(search.context === "management_link" ? { context: "management_link" as const } : {}),
    ...(intent ? { intent } : {}),
    ...(panel ? { panel } : {}),
    ...(context.libraryId ? { library_id: context.libraryId } : {}),
    ...(context.itemId ? { item_id: context.itemId } : {}),
    ...(context.mediaType ? { media_type: context.mediaType } : {}),
    ...(context.sourceId ? { source_id: context.sourceId } : {}),
    ...(context.playbackSessionId ? { playback_session_id: context.playbackSessionId } : {}),
  }
}

export function hasAdminManagementContextState(
  state: AdminManagementContextRouteState | undefined,
) {
  return Boolean(
    state?.context ||
      state?.intent ||
      state?.panel ||
      state?.libraryId ||
      state?.itemId ||
      state?.sourceId ||
      state?.playbackSessionId,
  )
}

function searchString(value: unknown) {
  if (typeof value !== "string") {
    return undefined
  }

  const trimmed = value.trim()
  return trimmed || undefined
}

function allowedSearchValue<T extends string>(value: unknown, allowed: T[]) {
  return typeof value === "string" && allowed.includes(value as T) ? (value as T) : undefined
}

function searchMediaType(value: unknown) {
  return value === "movie" || value === "series" ? value : undefined
}
