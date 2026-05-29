import {
  NAKO_ADMIN_ROUTES,
  type AdminAccessSummaryResponse,
  type AdminAccessUserListResponse,
  type AdminAccessUserResponse,
  type AdminAcquisitionIntakeCandidateListResponse,
  type AdminAcquisitionIntakeCandidatesQuery,
  type AdminAddonsQuery,
  type AdminAddonManagerPlanRequest,
  type AdminAddonManagerPlanResponse,
  type AdminAddonRegistrationsResponse,
  type AdminAddonRegistrationResponse,
  type AdminAddonSourceCatalogEntriesResponse,
  type AdminAddonSourceCatalogSourcesResponse,
  type AdminCreateUserRequest,
  type AdminJobCommandResponse,
  type AdminJobListResponse,
  type AdminJobsQuery,
  type AdminLocalPasswordResponse,
  type AdminMetadataRawCacheSettingsResponse,
  type AdminOutboxEventListResponse,
  type AdminOutboxEventsQuery,
  type AdminOverviewResponse,
  type AdminPageQuery,
  type AdminPlaybackRuntimeDiagnosticsResponse,
  type AdminPlaybackSessionListResponse,
  type AdminPlaybackSessionsQuery,
  type AdminReplaceUserRolesRequest,
  type AdminServerConfigDiagnosticsResponse,
  type AdminSetLocalPasswordRequest,
  type AdminStorageStagingDiagnosticsResponse,
  type AdminStorageStagingQuery,
  type AdminUpdateMetadataRawCacheSettingsRequest,
  type AdminUpdateUserStatusRequest,
  type UpdateAddonStatusRequest,
} from "./generated/contract"

export type AdminApiClientOptions = {
  baseUrl?: string
  bearerToken?: string
  fetcher?: typeof fetch
}

export class AdminApiClient {
  private readonly baseUrl: string
  private readonly bearerToken?: string
  private readonly fetcher: typeof fetch

  constructor(options: AdminApiClientOptions = {}) {
    this.baseUrl = normalizeBaseUrl(options.baseUrl)
    this.bearerToken = options.bearerToken
    this.fetcher = options.fetcher ?? ((input, init) => fetch(input, init))
  }

  getOverview(): Promise<AdminOverviewResponse> {
    return this.getJson<AdminOverviewResponse>(NAKO_ADMIN_ROUTES.overview)
  }

  getAccessSummary(): Promise<AdminAccessSummaryResponse> {
    return this.getJson<AdminAccessSummaryResponse>(NAKO_ADMIN_ROUTES.accessSummary)
  }

  getAccessUsers(query: AdminPageQuery = {}): Promise<AdminAccessUserListResponse> {
    return this.getJson<AdminAccessUserListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.accessUsers, query),
    )
  }

  createAccessUser(request: AdminCreateUserRequest): Promise<AdminAccessUserResponse> {
    return this.sendJson<AdminAccessUserResponse>("POST", NAKO_ADMIN_ROUTES.accessUsers, request)
  }

  replaceAccessUserRoles(
    userId: string,
    request: AdminReplaceUserRolesRequest,
  ): Promise<AdminAccessUserResponse> {
    return this.sendJson<AdminAccessUserResponse>(
      "PUT",
      pathParams(NAKO_ADMIN_ROUTES.accessUserRoles, { user_id: userId }),
      request,
    )
  }

  updateAccessUserStatus(
    userId: string,
    request: AdminUpdateUserStatusRequest,
  ): Promise<AdminAccessUserResponse> {
    return this.sendJson<AdminAccessUserResponse>(
      "PATCH",
      pathParams(NAKO_ADMIN_ROUTES.accessUserStatus, { user_id: userId }),
      request,
    )
  }

  setAccessUserLocalPassword(
    userId: string,
    request: AdminSetLocalPasswordRequest,
  ): Promise<AdminLocalPasswordResponse> {
    return this.sendJson<AdminLocalPasswordResponse>(
      "PUT",
      pathParams(NAKO_ADMIN_ROUTES.accessUserLocalPassword, { user_id: userId }),
      request,
    )
  }

  deleteAccessUserLocalPassword(userId: string): Promise<AdminLocalPasswordResponse> {
    return this.sendJson<AdminLocalPasswordResponse>(
      "DELETE",
      pathParams(NAKO_ADMIN_ROUTES.accessUserLocalPassword, { user_id: userId }),
    )
  }

  getEvents(query: AdminOutboxEventsQuery = {}): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>(withQuery(NAKO_ADMIN_ROUTES.events, query))
  }

  getAddons(query: AdminAddonsQuery = {}): Promise<AdminAddonRegistrationsResponse> {
    return this.getJson<AdminAddonRegistrationsResponse>(withQuery(NAKO_ADMIN_ROUTES.addons, query))
  }

  getAddonCatalogSources(): Promise<AdminAddonSourceCatalogSourcesResponse> {
    return this.getJson<AdminAddonSourceCatalogSourcesResponse>(
      NAKO_ADMIN_ROUTES.addonCatalogSources,
    )
  }

  getAddonCatalogEntries(): Promise<AdminAddonSourceCatalogEntriesResponse> {
    return this.getJson<AdminAddonSourceCatalogEntriesResponse>(
      NAKO_ADMIN_ROUTES.addonCatalogEntries,
    )
  }

  getAcquisitionIntakeCandidates(
    query: AdminAcquisitionIntakeCandidatesQuery = {},
  ): Promise<AdminAcquisitionIntakeCandidateListResponse> {
    return this.getJson<AdminAcquisitionIntakeCandidateListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates, query),
    )
  }

  updateAddonStatus(
    addonId: string,
    request: UpdateAddonStatusRequest,
  ): Promise<AdminAddonRegistrationResponse> {
    return this.sendJson<AdminAddonRegistrationResponse>(
      "PATCH",
      pathParams(NAKO_ADMIN_ROUTES.addonStatus, { addon_id: addonId }),
      request,
    )
  }

  getAddonManagerPlan(addonId: string): Promise<AdminAddonManagerPlanResponse> {
    return this.getJson<AdminAddonManagerPlanResponse>(
      pathParams(NAKO_ADMIN_ROUTES.addonManagerPlan, { addon_id: addonId }),
    )
  }

  planAddonManagerLifecycle(
    addonId: string,
    request: AdminAddonManagerPlanRequest,
  ): Promise<AdminAddonManagerPlanResponse> {
    return this.sendJson<AdminAddonManagerPlanResponse>(
      "POST",
      pathParams(NAKO_ADMIN_ROUTES.addonManagerPlan, { addon_id: addonId }),
      request,
    )
  }

  getJobs(query: AdminJobsQuery = {}): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(withQuery(NAKO_ADMIN_ROUTES.jobs, query))
  }

  requestLibraryScan(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.sendJson<AdminJobCommandResponse>(
      "POST",
      pathParams(NAKO_ADMIN_ROUTES.libraryScan, { library_id: libraryId }),
    )
  }

  requestLibraryNfoImport(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.sendJson<AdminJobCommandResponse>(
      "POST",
      pathParams(NAKO_ADMIN_ROUTES.libraryNfoImport, { library_id: libraryId }),
    )
  }

  requestLibraryNfoExport(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.sendJson<AdminJobCommandResponse>(
      "POST",
      pathParams(NAKO_ADMIN_ROUTES.libraryNfoExport, { library_id: libraryId }),
    )
  }

  getPlaybackSessions(
    query: AdminPlaybackSessionsQuery = {},
  ): Promise<AdminPlaybackSessionListResponse> {
    return this.getJson<AdminPlaybackSessionListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.playbackSessions, query),
    )
  }

  getPlaybackRuntime(): Promise<AdminPlaybackRuntimeDiagnosticsResponse> {
    return this.getJson<AdminPlaybackRuntimeDiagnosticsResponse>(NAKO_ADMIN_ROUTES.playbackRuntime)
  }

  getSystemConfig(): Promise<AdminServerConfigDiagnosticsResponse> {
    return this.getJson<AdminServerConfigDiagnosticsResponse>(NAKO_ADMIN_ROUTES.systemConfig)
  }

  getStorageStaging(
    query: AdminStorageStagingQuery = {},
  ): Promise<AdminStorageStagingDiagnosticsResponse> {
    return this.getJson<AdminStorageStagingDiagnosticsResponse>(
      withQuery(NAKO_ADMIN_ROUTES.storageStaging, query),
    )
  }

  getMetadataRawCacheSettings(): Promise<AdminMetadataRawCacheSettingsResponse> {
    return this.getJson<AdminMetadataRawCacheSettingsResponse>(
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
    )
  }

  updateMetadataRawCacheSettings(
    request: AdminUpdateMetadataRawCacheSettingsRequest,
  ): Promise<AdminMetadataRawCacheSettingsResponse> {
    return this.sendJson<AdminMetadataRawCacheSettingsResponse>(
      "PUT",
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
      request,
    )
  }

  private async getJson<T>(path: string): Promise<T> {
    return this.sendJson<T>("GET", path)
  }

  private async sendJson<T>(method: string, path: string, body?: unknown): Promise<T> {
    const headers = new Headers(this.headers())
    const init: RequestInit = {
      method,
      headers,
    }

    if (body !== undefined) {
      headers.set("content-type", "application/json")
      init.body = JSON.stringify(body)
    }

    const response = await this.fetcher(`${this.baseUrl}${path}`, init)

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`)
    }

    const contentType = response.headers.get("content-type") ?? ""
    if (!contentType.toLowerCase().includes("application/json")) {
      throw new Error("Admin API request returned a non-JSON response")
    }

    return (await response.json()) as T
  }

  private headers(): HeadersInit {
    if (!this.bearerToken) {
      return {}
    }

    return {
      Authorization: `Bearer ${this.bearerToken}`,
    }
  }
}

function normalizeBaseUrl(baseUrl: string | undefined) {
  const value = baseUrl?.trim() || ""

  return value.endsWith("/") ? value.slice(0, -1) : value
}

function withQuery(path: string, query: object) {
  const params = new URLSearchParams()

  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== "") {
      params.set(key, String(value))
    }
  }

  const suffix = params.toString()
  return suffix ? `${path}?${suffix}` : path
}

function pathParams(path: string, params: Record<string, string>) {
  return path.replace(/\{([^}]+)\}|:([A-Za-z_][A-Za-z0-9_]*)/g, (token, braced, colon) => {
    const key = braced ?? colon
    const value = params[key]
    if (value === undefined) {
      throw new Error(`Missing Admin API path parameter: ${key}`)
    }

    return encodeURIComponent(value)
  })
}
