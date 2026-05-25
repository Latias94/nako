import type {
  AddonGrantsResponse,
  AddonTokenIssuedResponse,
  AddonTokenResponse,
  AddonTokenRotationResponse,
  AddonTokensResponse,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AdminAddonHealthCheckResponse,
  AdminAddonInstallGuideResponse,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticRequest,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAddonRoutingPlansResponse,
  AdminAddonRuntimeReadinessResponse,
  AdminAddonSurfacesResponse,
  AdminAddonsQuery,
  AdminCatalogGovernanceItemListResponse,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  IssueAddonTokenRequest,
  RegisterAddonRequest,
  ReplaceAddonGrantsRequest,
  UpdateAddonStatusRequest,
} from "./generated/contract";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";

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
    this.fetcher = options.fetcher ?? ((input, init) => fetch(input, init));
  }

  async getOverview(): Promise<AdminOverviewResponse> {
    return this.getJson<AdminOverviewResponse>(NAKO_ADMIN_ROUTES.overview);
  }

  async getAddons(query: AdminAddonsQuery = {}): Promise<AdminAddonRegistrationsResponse> {
    return this.getJson<AdminAddonRegistrationsResponse>(withQuery(NAKO_ADMIN_ROUTES.addons, query));
  }

  async getAddonDetail(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.getJson<AdminAddonRegistrationResponse>(addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId));
  }

  async registerAddon(
    manifest: RegisterAddonRequest["manifest"],
    options: {
      id?: string;
      grantedScopes?: RegisterAddonRequest["granted_scopes"];
      status?: RegisterAddonRequest["status"];
    } = {},
  ): Promise<AdminAddonRegistrationResponse> {
    return this.postJson<AdminAddonRegistrationResponse>(NAKO_ADMIN_ROUTES.addons, {
      id: options.id,
      manifest,
      granted_scopes: options.grantedScopes ?? [],
      status: options.status ?? "disabled",
    } satisfies RegisterAddonRequest);
  }

  async updateAddonStatus(
    addonId: string,
    request: UpdateAddonStatusRequest,
  ): Promise<AdminAddonRegistrationResponse> {
    return this.patchJson<AdminAddonRegistrationResponse>(
      addonPath(NAKO_ADMIN_ROUTES.addonStatus, addonId),
      request,
    );
  }

  async unregisterAddon(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.postJson<AdminAddonRegistrationResponse>(
      addonPath(NAKO_ADMIN_ROUTES.addonUnregister, addonId),
      {},
    );
  }

  async checkAddonHealth(addonId: string): Promise<AdminAddonHealthCheckResponse> {
    return this.postJson<AdminAddonHealthCheckResponse>(
      addonPath(NAKO_ADMIN_ROUTES.addonHealthCheck, addonId),
      {},
    );
  }

  async getAddonSurfaces(addonId: string): Promise<AdminAddonSurfacesResponse> {
    return this.getJson<AdminAddonSurfacesResponse>(addonPath(NAKO_ADMIN_ROUTES.addonSurfaces, addonId));
  }

  async getAddonInstallGuide(addonId: string): Promise<AdminAddonInstallGuideResponse> {
    return this.getJson<AdminAddonInstallGuideResponse>(addonPath(NAKO_ADMIN_ROUTES.addonInstallGuide, addonId));
  }

  async diagnoseAddonResourceCall(
    addonId: string,
    request: AdminAddonResourceCallDiagnosticRequest,
  ): Promise<AdminAddonResourceCallDiagnosticResponse> {
    return this.postJson<AdminAddonResourceCallDiagnosticResponse>(
      addonPath(NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic, addonId),
      request,
    );
  }

  async getAddonTokens(addonId: string): Promise<AddonTokensResponse> {
    return this.getJson<AddonTokensResponse>(`${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/tokens`);
  }

  async issueAddonToken(
    addonId: string,
    request: IssueAddonTokenRequest,
  ): Promise<AddonTokenIssuedResponse> {
    return this.postJson<AddonTokenIssuedResponse>(
      `${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/tokens`,
      request,
    );
  }

  async rotateAddonToken(
    addonId: string,
    tokenId: string,
    request: IssueAddonTokenRequest,
  ): Promise<AddonTokenRotationResponse> {
    return this.postJson<AddonTokenRotationResponse>(
      `${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/tokens/${encodeURIComponent(tokenId)}/rotate`,
      request,
    );
  }

  async revokeAddonToken(addonId: string, tokenId: string): Promise<AddonTokenResponse> {
    return this.postJson<AddonTokenResponse>(
      `${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/tokens/${encodeURIComponent(tokenId)}/revoke`,
      {},
    );
  }

  async getAddonGrants(addonId: string): Promise<AddonGrantsResponse> {
    return this.getJson<AddonGrantsResponse>(`${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/grants`);
  }

  async replaceAddonGrants(
    addonId: string,
    request: ReplaceAddonGrantsRequest,
  ): Promise<AddonGrantsResponse> {
    return this.putJson<AddonGrantsResponse>(
      `${addonPath(NAKO_ADMIN_ROUTES.addonDetail, addonId)}/grants`,
      request,
    );
  }

  async getCatalogGovernanceItems(): Promise<AdminCatalogGovernanceItemListResponse> {
    return this.getJson<AdminCatalogGovernanceItemListResponse>(
      NAKO_ADMIN_ROUTES.catalogGovernanceItems,
    );
  }

  async getAcquisitionIntakeCandidates(
    query: AdminAcquisitionIntakeCandidatesQuery = {},
  ): Promise<AdminAcquisitionIntakeCandidateListResponse> {
    return this.getJson<AdminAcquisitionIntakeCandidateListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates, query),
    );
  }

  async discoverWatchFolderCandidates(
    request: AdminWatchFolderDiscoveryRequest,
  ): Promise<AdminWatchFolderDiscoveryResponse> {
    return this.postJson<AdminWatchFolderDiscoveryResponse>(
      NAKO_ADMIN_ROUTES.acquisitionIntakeWatchFolderDiscovery,
      request,
    );
  }

  async getGeneratedArtifactProposals(
    query: AdminGeneratedArtifactProposalsQuery = {},
  ): Promise<AdminGeneratedArtifactProposalListResponse> {
    return this.getJson<AdminGeneratedArtifactProposalListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.generatedArtifactProposals, query),
    );
  }

  async getEvents(): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>(NAKO_ADMIN_ROUTES.events);
  }

  async getJobs(query: AdminJobsQuery = {}): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(withQuery(NAKO_ADMIN_ROUTES.jobs, query));
  }

  async getPlaybackSessions(): Promise<AdminPlaybackSessionListResponse> {
    return this.getJson<AdminPlaybackSessionListResponse>(NAKO_ADMIN_ROUTES.playbackSessions);
  }

  async getPlaybackRuntime(): Promise<AdminPlaybackRuntimeDiagnosticsResponse> {
    return this.getJson<AdminPlaybackRuntimeDiagnosticsResponse>(NAKO_ADMIN_ROUTES.playbackRuntime);
  }

  async getPlaybackSupport(
    query: AdminPlaybackSupportQuery = {},
  ): Promise<AdminPlaybackSupportEvidenceResponse> {
    return this.getJson<AdminPlaybackSupportEvidenceResponse>(
      withQuery(NAKO_ADMIN_ROUTES.playbackSupport, query),
    );
  }

  async getAddonRuntimeReadiness(addonId: string): Promise<AdminAddonRuntimeReadinessResponse> {
    return this.postJson<AdminAddonRuntimeReadinessResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonRuntimeReadiness, "addon_id", addonId),
      {},
    );
  }

  async getAddonRoutingPlans(addonId: string): Promise<AdminAddonRoutingPlansResponse> {
    return this.postJson<AdminAddonRoutingPlansResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonRoutingPlans, "addon_id", addonId),
      {},
    );
  }

  async getStorageStaging(): Promise<AdminStorageStagingDiagnosticsResponse> {
    return this.getJson<AdminStorageStagingDiagnosticsResponse>(NAKO_ADMIN_ROUTES.storageStaging);
  }

  async getSystemConfig(): Promise<AdminServerConfigDiagnosticsResponse> {
    return this.getJson<AdminServerConfigDiagnosticsResponse>(NAKO_ADMIN_ROUTES.systemConfig);
  }

  private async getJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
    }

    return this.parseJson<T>(response);
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

    return this.parseJson<T>(response);
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

    return this.parseJson<T>(response);
  }

  private async putJson<T>(path: string, body: unknown): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      method: "PUT",
      headers: {
        ...this.headers(),
        "Content-Type": "application/json",
      },
      body: JSON.stringify(body),
    });

    if (!response.ok) {
      throw new Error(`Admin API request failed with HTTP ${response.status}`);
    }

    return this.parseJson<T>(response);
  }

  private async parseJson<T>(response: Response): Promise<T> {
    const contentType = response.headers.get("content-type") ?? "";
    if (!contentType.toLowerCase().includes("application/json")) {
      throw new Error("Admin API request returned a non-JSON response");
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

function routeWithParam(path: string, name: string, value: string) {
  return path.replace(`{${name}}`, encodeURIComponent(value));
}

function withQuery(
  path: string,
  query:
    | AdminPlaybackSupportQuery
    | AdminAcquisitionIntakeCandidatesQuery
    | AdminGeneratedArtifactProposalsQuery
    | AdminAddonsQuery
    | AdminJobsQuery,
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
