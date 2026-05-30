import {
  NakoClient,
  type FetchLike,
  type ManagementContextDto,
  type ManagementContextLinkDto,
  type ManagementContextQuery,
} from "@nako/sdk"
import { loadPublicClientConnection, type PublicClientConnection } from "./connection"
import {
  normalizePublicManagementContext,
  type PublicManagementContext,
} from "./management-context-model"

export { normalizePublicManagementContext } from "./management-context-model"
export type { PublicManagementContext } from "./management-context-model"

export type PublicManagementContextSourceMode = "live" | "fixture"

export type PublicManagementContextQuery = PublicManagementContext

export type PublicManagementContextLink = {
  action: ManagementContextLinkDto["action"]
  disabledReason: ManagementContextLinkDto["disabled_reason"]
  enabled: boolean
  method: ManagementContextLinkDto["method"]
  requiredAccess: ManagementContextLinkDto["required_access"]
  routeName: string
  surface: ManagementContextLinkDto["surface"]
  target: PublicManagementContext
}

export type PublicManagementContextLinksPayload = {
  context: PublicManagementContext
  links: PublicManagementContextLink[]
  fallback: boolean
  source: PublicManagementContextSourceMode
  error?: string
}

export type PublicManagementContextDataSource = {
  loadManagementContextLinks(
    query?: PublicManagementContextQuery,
  ): Promise<PublicManagementContextLinksPayload>
}

type ManagementRouteFixture = Pick<
  PublicManagementContextLink,
  "action" | "method" | "requiredAccess" | "routeName"
> & {
  requires: "library" | "item" | "playback" | "none"
}

const MANAGEMENT_ROUTE_FIXTURES: ManagementRouteFixture[] = [
  {
    routeName: "library.scan",
    method: "POST",
    action: "scan_library",
    requiredAccess: "library_manage",
    requires: "library",
  },
  {
    routeName: "library.metadata_profile",
    method: "GET",
    action: "update_library_metadata_profile",
    requiredAccess: "administrator",
    requires: "library",
  },
  {
    routeName: "item.metadata_refresh",
    method: "POST",
    action: "refresh_item_metadata",
    requiredAccess: "library_manage",
    requires: "item",
  },
  {
    routeName: "jobs.filtered",
    method: "GET",
    action: "view_jobs",
    requiredAccess: "administrator",
    requires: "none",
  },
  {
    routeName: "playback.support",
    method: "GET",
    action: "view_playback_diagnostics",
    requiredAccess: "administrator",
    requires: "playback",
  },
  {
    routeName: "playback.runtime",
    method: "GET",
    action: "view_playback_runtime",
    requiredAccess: "administrator",
    requires: "none",
  },
  {
    routeName: "access.library_policies",
    method: "GET",
    action: "manage_library_access",
    requiredAccess: "administrator",
    requires: "library",
  },
]

export function createPublicManagementContextDataSource(
  connection: PublicClientConnection = loadPublicClientConnection(),
  fetcher?: FetchLike,
): PublicManagementContextDataSource {
  if (connection.mode === "fixture") {
    return createFixtureManagementContextDataSource()
  }

  return createLiveManagementContextDataSource(connection, fetcher)
}

function createLiveManagementContextDataSource(
  connection: Extract<PublicClientConnection, { mode: "live" }>,
  fetcher?: FetchLike,
): PublicManagementContextDataSource {
  const client = new NakoClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetch: fetcher,
  })

  return {
    async loadManagementContextLinks(query = {}) {
      try {
        const response = await client.managementContextLinks(toSdkQuery(query))

        return liveManagementContextLinks(response.context, response.links)
      } catch (error) {
        return fixtureManagementContextLinks(query, error)
      }
    },
  }
}

function createFixtureManagementContextDataSource(): PublicManagementContextDataSource {
  return {
    async loadManagementContextLinks(query = {}) {
      return fixtureManagementContextLinks(query)
    },
  }
}

function liveManagementContextLinks(
  context: ManagementContextDto,
  links: ManagementContextLinkDto[],
): PublicManagementContextLinksPayload {
  return {
    context: mapManagementContext(context),
    links: links.map(mapManagementContextLink),
    fallback: false,
    source: "live",
  }
}

function fixtureManagementContextLinks(
  query: PublicManagementContextQuery,
  error?: unknown,
): PublicManagementContextLinksPayload {
  const context = normalizePublicManagementContext(query)

  return {
    context,
    links: MANAGEMENT_ROUTE_FIXTURES.map((fixture) => fixtureManagementContextLink(fixture, context)),
    fallback: true,
    source: "fixture",
    error: errorMessage(error),
  }
}

function fixtureManagementContextLink(
  fixture: ManagementRouteFixture,
  context: PublicManagementContext,
): PublicManagementContextLink {
  const hasRequiredContext = hasFixtureContext(fixture.requires, context)

  return {
    action: fixture.action,
    disabledReason: hasRequiredContext ? null : "missing_context",
    enabled: hasRequiredContext,
    method: fixture.method,
    requiredAccess: fixture.requiredAccess,
    routeName: fixture.routeName,
    surface: "management",
    target: context,
  }
}

function hasFixtureContext(
  requirement: ManagementRouteFixture["requires"],
  context: PublicManagementContext,
) {
  switch (requirement) {
    case "library":
      return Boolean(context.libraryId)
    case "item":
      return Boolean(context.itemId)
    case "playback":
      return Boolean(context.sourceId || context.playbackSessionId)
    case "none":
      return true
  }
}

function mapManagementContextLink(
  link: ManagementContextLinkDto,
): PublicManagementContextLink {
  return {
    action: link.action,
    disabledReason: link.disabled_reason,
    enabled: link.enabled,
    method: link.method,
    requiredAccess: link.required_access,
    routeName: link.route_name,
    surface: link.surface,
    target: mapManagementContext(link.target),
  }
}

function mapManagementContext(context: ManagementContextDto): PublicManagementContext {
  return normalizePublicManagementContext({
    libraryId: context.library_id ?? undefined,
    itemId: context.item_id ?? undefined,
    sourceId: context.source_id ?? undefined,
    playbackSessionId: context.playback_session_id ?? undefined,
  })
}

function toSdkQuery(query: PublicManagementContextQuery): ManagementContextQuery | undefined {
  const context = normalizePublicManagementContext(query)
  const sdkQuery: ManagementContextQuery = {
    library_id: context.libraryId,
    item_id: context.itemId,
    source_id: context.sourceId,
    playback_session_id: context.playbackSessionId,
  }

  return Object.values(sdkQuery).some(Boolean) ? sdkQuery : undefined
}

function errorMessage(error: unknown) {
  if (!error) {
    return undefined
  }

  return error instanceof Error ? error.message : "Management Context Links request failed"
}
