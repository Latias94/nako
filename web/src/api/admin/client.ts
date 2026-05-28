import {
  NAKO_ADMIN_ROUTES,
  type AdminJobListResponse,
  type AdminJobsQuery,
  type AdminOverviewResponse,
  type AdminPlaybackRuntimeDiagnosticsResponse,
  type AdminPlaybackSessionListResponse,
  type AdminPlaybackSessionsQuery,
  type AdminServerConfigDiagnosticsResponse,
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

  getJobs(query: AdminJobsQuery = {}): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(withQuery(NAKO_ADMIN_ROUTES.jobs, query))
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

  private async getJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: this.headers(),
    })

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
