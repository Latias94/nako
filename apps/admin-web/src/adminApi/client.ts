import type {
  AddonGrantsResponse,
  AddonTokensResponse,
  AdminCatalogGovernanceItemListResponse,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AdminAddonHealthCheckResponse,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticRequest,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAddonSurfacesResponse,
  AdminAddonsQuery,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
  AdminJobListResponse,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  UpdateAddonStatusRequest,
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

  async getAddons(query: AdminAddonsQuery = {}): Promise<AdminAddonRegistrationsResponse> {
    return this.getJson<AdminAddonRegistrationsResponse>(withQuery(TARU_ADMIN_ROUTES.addons, query));
  }

  async getAddonDetail(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.getJson<AdminAddonRegistrationResponse>(addonPath(TARU_ADMIN_ROUTES.addonDetail, addonId));
  }

  async updateAddonStatus(
    addonId: string,
    request: UpdateAddonStatusRequest,
  ): Promise<AdminAddonRegistrationResponse> {
    return this.patchJson<AdminAddonRegistrationResponse>(
      addonPath(TARU_ADMIN_ROUTES.addonStatus, addonId),
      request,
    );
  }

  async unregisterAddon(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.postJson<AdminAddonRegistrationResponse>(
      addonPath(TARU_ADMIN_ROUTES.addonUnregister, addonId),
      {},
    );
  }

  async checkAddonHealth(addonId: string): Promise<AdminAddonHealthCheckResponse> {
    return this.postJson<AdminAddonHealthCheckResponse>(
      addonPath(TARU_ADMIN_ROUTES.addonHealthCheck, addonId),
      {},
    );
  }

  async getAddonSurfaces(addonId: string): Promise<AdminAddonSurfacesResponse> {
    return this.getJson<AdminAddonSurfacesResponse>(addonPath(TARU_ADMIN_ROUTES.addonSurfaces, addonId));
  }

  async diagnoseAddonResourceCall(
    addonId: string,
    request: AdminAddonResourceCallDiagnosticRequest,
  ): Promise<AdminAddonResourceCallDiagnosticResponse> {
    return this.postJson<AdminAddonResourceCallDiagnosticResponse>(
      addonPath(TARU_ADMIN_ROUTES.addonResourceCallDiagnostic, addonId),
      request,
    );
  }

  async getAddonTokens(addonId: string): Promise<AddonTokensResponse> {
    return this.getJson<AddonTokensResponse>(`${addonPath(TARU_ADMIN_ROUTES.addonDetail, addonId)}/tokens`);
  }

  async getAddonGrants(addonId: string): Promise<AddonGrantsResponse> {
    return this.getJson<AddonGrantsResponse>(`${addonPath(TARU_ADMIN_ROUTES.addonDetail, addonId)}/grants`);
  }

  async getCatalogGovernanceItems(): Promise<AdminCatalogGovernanceItemListResponse> {
    return this.getJson<AdminCatalogGovernanceItemListResponse>(
      TARU_ADMIN_ROUTES.catalogGovernanceItems,
    );
  }

  async getAcquisitionIntakeCandidates(
    query: AdminAcquisitionIntakeCandidatesQuery = {},
  ): Promise<AdminAcquisitionIntakeCandidateListResponse> {
    return this.getJson<AdminAcquisitionIntakeCandidateListResponse>(
      withQuery(TARU_ADMIN_ROUTES.acquisitionIntakeCandidates, query),
    );
  }

  async discoverWatchFolderCandidates(
    request: AdminWatchFolderDiscoveryRequest,
  ): Promise<AdminWatchFolderDiscoveryResponse> {
    return this.postJson<AdminWatchFolderDiscoveryResponse>(
      TARU_ADMIN_ROUTES.acquisitionIntakeWatchFolderDiscovery,
      request,
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

  private async postJson<T>(path: string, body: unknown): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      method: "POST",
      headers: {
        ...this.headers(),
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
    }

    return (await response.json()) as T;
  }

  private async patchJson<T>(path: string, body: unknown): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      method: "PATCH",
      headers: {
        ...this.headers(),
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
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

function withQuery(
  path: string,
  query: AdminPlaybackSupportQuery | AdminAcquisitionIntakeCandidatesQuery | AdminAddonsQuery,
) {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(query)) {
    if (value) {
      params.set(key, value);
    }
  }

  const suffix = params.toString();
  return suffix ? `${path}?${suffix}` : path;
}

function addonPath(template: string, addonId: string) {
  return template.replace(":addon_id", encodeURIComponent(addonId));
}
