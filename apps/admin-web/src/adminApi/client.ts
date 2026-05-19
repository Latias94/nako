import type {
  AdminCatalogGovernanceItemListResponse,
  AdminJobListResponse,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
} from "./types";

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
    return this.getJson<AdminOverviewResponse>("/admin/v1/overview");
  }

  async getCatalogGovernanceItems(): Promise<AdminCatalogGovernanceItemListResponse> {
    return this.getJson<AdminCatalogGovernanceItemListResponse>(
      "/admin/v1/catalog/governance/items",
    );
  }

  async getEvents(): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>("/admin/v1/events");
  }

  async getJobs(): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>("/admin/v1/jobs");
  }

  async getPlaybackSessions(): Promise<AdminPlaybackSessionListResponse> {
    return this.getJson<AdminPlaybackSessionListResponse>("/admin/v1/playback/sessions");
  }

  async getPlaybackRuntime(): Promise<AdminPlaybackRuntimeDiagnosticsResponse> {
    return this.getJson<AdminPlaybackRuntimeDiagnosticsResponse>("/admin/v1/playback/runtime");
  }

  async getStorageStaging(): Promise<AdminStorageStagingDiagnosticsResponse> {
    return this.getJson<AdminStorageStagingDiagnosticsResponse>("/admin/v1/storage/staging");
  }

  async getSystemConfig(): Promise<AdminServerConfigDiagnosticsResponse> {
    return this.getJson<AdminServerConfigDiagnosticsResponse>("/admin/v1/system/config");
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
