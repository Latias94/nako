import {
  type PublicManagementContextLink,
} from "@/src/api/public/management-context-data-source"
import {
  normalizePublicManagementContext,
  type PublicManagementContext,
} from "@/src/api/public/management-context-model"

export type ManagementContextRoutePath =
  | "/admin/libraries"
  | "/admin/tasks"
  | "/admin/transcoding"
  | "/admin/users"

export type ManagementContextRouteSearch = Record<string, string>

export type ResolvedManagementContextRoute = {
  routeName: string
  action: PublicManagementContextLink["action"]
  path: ManagementContextRoutePath
  search: ManagementContextRouteSearch
}

export function resolveManagementContextLinks(
  links: PublicManagementContextLink[],
): ResolvedManagementContextRoute[] {
  return links.flatMap((link) => {
    const route = resolveManagementContextLink(link)

    return route ? [route] : []
  })
}

export function resolveManagementContextLink(
  link: PublicManagementContextLink,
): ResolvedManagementContextRoute | null {
  if (!link.enabled) {
    return null
  }

  const target = normalizePublicManagementContext(link.target)

  switch (link.routeName) {
    case "library.scan":
      return withRequiredLibrary(link, target, "/admin/libraries", {
        intent: "scan_library",
      })
    case "library.metadata_profile":
      return withRequiredLibrary(link, target, "/admin/libraries", {
        panel: "metadata_profile",
      })
    case "item.metadata_refresh":
      return withRequiredItem(link, target, "/admin/libraries", {
        intent: "refresh_item_metadata",
      })
    case "jobs.filtered":
      return resolvedRoute(link, "/admin/tasks", {
        context: "management_link",
        ...contextSearch(target),
      })
    case "playback.support":
      return withRequiredPlayback(link, target, "/admin/transcoding", {
        panel: "support",
      })
    case "playback.runtime":
      return resolvedRoute(link, "/admin/transcoding", {
        panel: "runtime",
        ...contextSearch(target),
      })
    case "access.library_policies":
      return withRequiredLibrary(link, target, "/admin/users", {
        panel: "library_access",
      })
    default:
      return null
  }
}

function withRequiredLibrary(
  link: PublicManagementContextLink,
  target: PublicManagementContext,
  path: ManagementContextRoutePath,
  extra: ManagementContextRouteSearch,
) {
  if (!target.libraryId) {
    return null
  }

  return resolvedRoute(link, path, {
    library_id: target.libraryId,
    ...extra,
  })
}

function withRequiredItem(
  link: PublicManagementContextLink,
  target: PublicManagementContext,
  path: ManagementContextRoutePath,
  extra: ManagementContextRouteSearch,
) {
  if (!target.itemId) {
    return null
  }

  return resolvedRoute(link, path, {
    ...contextSearch(target),
    ...extra,
  })
}

function withRequiredPlayback(
  link: PublicManagementContextLink,
  target: PublicManagementContext,
  path: ManagementContextRoutePath,
  extra: ManagementContextRouteSearch,
) {
  if (!target.sourceId && !target.playbackSessionId) {
    return null
  }

  return resolvedRoute(link, path, {
    ...contextSearch(target),
    ...extra,
  })
}

function resolvedRoute(
  link: PublicManagementContextLink,
  path: ManagementContextRoutePath,
  search: ManagementContextRouteSearch,
): ResolvedManagementContextRoute {
  return {
    routeName: link.routeName,
    action: link.action,
    path,
    search,
  }
}

function contextSearch(context: PublicManagementContext): ManagementContextRouteSearch {
  return {
    ...optionalSearch("library_id", context.libraryId),
    ...optionalSearch("item_id", context.itemId),
    ...optionalSearch("media_type", context.mediaType),
    ...optionalSearch("source_id", context.sourceId),
    ...optionalSearch("playback_session_id", context.playbackSessionId),
  }
}

function optionalSearch(key: string, value: string | undefined): ManagementContextRouteSearch {
  return value ? { [key]: value } : {}
}
