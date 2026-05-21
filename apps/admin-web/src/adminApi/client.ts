import type {
  AdminCatalogGovernanceItemListResponse,
  AdminJobListResponse,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
} from "./generated/contract";
import { TARU_ADMIN_ROUTES } from "./generated/contract";

export type AdminApiClientOptions = {
  baseUrl?: string;
  token?: string;
  fetcher?: typeof fetch;
};

export class AdminApiClient {
  private readonly baseUrl: string;
  private readonly token?: string;
  private readonly fetcher: typeof fetch;

  constructor(options: AdminApiClientOptions = {}) {
    this.baseUrl = normalizeBaseUrl(options.baseUrl);
    this.token = options.token;
    this.fetcher = options.fetcher ?? fetch;
  }

  async getOverview(): Promise<AdminOverviewResponse> {
    return this.getJson<AdminOverviewResponse>(TARU_ADMIN_ROUTES.overview);
  }

  async getCatalogGovernanceItems(): Promise<AdminCatalogGovernanceItemListResponse> {
    return this.getJson<AdminCatalogGovernanceItemListResponse>(
      TARU_ADMIN_ROUTES.catalogGovernanceItems,
    );
  }

  async getEvents(): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>(TARU_ADMIN_ROUTES.events);
  }

  async getJobs(): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(TARU_ADMIN_ROUTES.jobs);
  }

  async getPlaybackSessions(): Promise<AdminPlaybackSessionListResponse> {
    return this.getJson<AdminPlaybackSessionListResponse>(TARU_ADMIN_ROUTES.playbackSessions);
  }

  async getPlaybackRuntime(): Promise<AdminPlaybackRuntimeDiagnosticsResponse> {
    return this.getJson<AdminPlaybackRuntimeDiagnosticsResponse>(TARU_ADMIN_ROUTES.playbackRuntime);
  }

  async getPlaybackSupport(
    query: AdminPlaybackSupportQuery = {},
  ): Promise<AdminPlaybackSupportEvidenceResponse> {
    return this.getJson<AdminPlaybackSupportEvidenceResponse>(
      withQuery(TARU_ADMIN_ROUTES.playbackSupport, query),
    );
  }

  async getStorageStaging(): Promise<AdminStorageStagingDiagnosticsResponse> {
    return this.getJson<AdminStorageStagingDiagnosticsResponse>(TARU_ADMIN_ROUTES.storageStaging);
  }

  async getSystemConfig(): Promise<AdminServerConfigDiagnosticsResponse> {
    return this.getJson<AdminServerConfigDiagnosticsResponse>(TARU_ADMIN_ROUTES.systemConfig);
  }

  private async getJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
    }

    return (await response.json()) as T;
  }

  private headers(): HeadersInit {
    if (!this.token) {
      return {};
    }

    return {
      Authorization: `Bearer ${this.token}`,
    };
  }
}

function normalizeBaseUrl(baseUrl: string | undefined) {
  const value = baseUrl?.trim() || "";

  if (value.endsWith("/")) {
    return value.slice(0, -1);
  }

  return value;
}

function withQuery(path: string, query: AdminPlaybackSupportQuery) {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(query)) {
    if (value) {
      params.set(key, value);
    }
  }

  const suffix = params.toString();
  return suffix ? `${path}?${suffix}` : path;
}
