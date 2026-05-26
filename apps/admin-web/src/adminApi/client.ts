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
  AdminAccessSummaryResponse,
  AdminAddonRoutingPlansResponse,
  AdminAddonRuntimeReadinessResponse,
  AdminAddonSurfacesResponse,
  AdminAddonsQuery,
  AdminCatalogGovernanceItemListResponse,
  AdminCatalogGovernanceItemsQuery,
  AdminCatalogGovernanceItemDetailResponse,
  AdminCatalogGovernanceProviderMappingReviewDecision,
  AdminCatalogGovernanceProviderMappingReviewPlanResponse,
  AdminCatalogGovernanceProviderMappingReviewRequest,
  AdminCatalogGovernanceProviderMappingReviewResponse,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminGeneratedArtifactReviewPlanResponse,
  AdminGeneratedArtifactReviewRequest,
  AdminGeneratedArtifactReviewResponse,
  AdminArtworkKind,
  AdminItemArtworkGalleryQuery,
  AdminManagedArtworkGalleryResponse,
  AdminSelectItemArtworkRequest,
  AdminJobCommandResponse,
  AdminLibraryMetadataProfileResponse,
  AdminMetadataRawCacheSettingsResponse,
  AdminMetadataProfile,
  AdminUpdateMetadataRawCacheSettingsRequest,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionsQuery,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingQuery,
  AdminStorageStagingDiagnosticsResponse,
  IssueAddonTokenRequest,
  RegisterAddonRequest,
  ReplaceAddonGrantsRequest,
  AdminUpdateLibraryMetadataProfileRequest,
  PublishSelectedArtworkResponse,
  UpdateAddonStatusRequest,
  UnpublishSelectedArtworkResponse,
} from "./generated/contract";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";
import type {
  PublicCatalogItemsQuery,
  PublicCatalogItemsResponse,
  PublicCatalogSearchQuery,
  PublicCatalogSearchResponse,
  PublicItemCreditsResponse,
  PublicItemDetailResponse,
  PublicItemImagesResponse,
  PublicLibrarySourcesResponse,
  PublicSourceProbeResponse,
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
    this.fetcher = options.fetcher ?? ((input, init) => fetch(input, init));
  }

  async getOverview(): Promise<AdminOverviewResponse> {
    return this.getJson<AdminOverviewResponse>(NAKO_ADMIN_ROUTES.overview);
  }

  async getAccessSummary(): Promise<AdminAccessSummaryResponse> {
    return this.getJson<AdminAccessSummaryResponse>(NAKO_ADMIN_ROUTES.accessSummary);
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

  async getCatalogGovernanceItems(
    query: AdminCatalogGovernanceItemsQuery = {},
  ): Promise<AdminCatalogGovernanceItemListResponse> {
    return this.getJson<AdminCatalogGovernanceItemListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.catalogGovernanceItems, query),
    );
  }

  async getCatalogGovernanceItemDetail(
    itemId: string,
  ): Promise<AdminCatalogGovernanceItemDetailResponse> {
    return this.getJson<AdminCatalogGovernanceItemDetailResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.catalogGovernanceItemDetail, "item_id", itemId),
    );
  }

  async planCatalogGovernanceProviderMappingReview(
    itemId: string,
    mappingId: string,
    decision: AdminCatalogGovernanceProviderMappingReviewDecision,
  ): Promise<AdminCatalogGovernanceProviderMappingReviewPlanResponse> {
    const path = routeWithParam(
      routeWithParam(
        NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReviewPlan,
        "item_id",
        itemId,
      ),
      "mapping_id",
      mappingId,
    );

    return this.postJson<AdminCatalogGovernanceProviderMappingReviewPlanResponse>(
      path,
      { decision } satisfies AdminCatalogGovernanceProviderMappingReviewRequest,
    );
  }

  async reviewCatalogGovernanceProviderMapping(
    itemId: string,
    mappingId: string,
    decision: AdminCatalogGovernanceProviderMappingReviewDecision,
  ): Promise<AdminCatalogGovernanceProviderMappingReviewResponse> {
    const path = routeWithParam(
      routeWithParam(
        NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReview,
        "item_id",
        itemId,
      ),
      "mapping_id",
      mappingId,
    );

    return this.postJson<AdminCatalogGovernanceProviderMappingReviewResponse>(
      path,
      { decision } satisfies AdminCatalogGovernanceProviderMappingReviewRequest,
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

  async planGeneratedArtifactReview(
    artifactId: string,
    decision: AdminGeneratedArtifactReviewRequest["decision"],
  ): Promise<AdminGeneratedArtifactReviewPlanResponse> {
    return this.postJson<AdminGeneratedArtifactReviewPlanResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.generatedArtifactReviewPlan, "artifact_id", artifactId),
      { decision } satisfies AdminGeneratedArtifactReviewRequest,
    );
  }

  async reviewGeneratedArtifact(
    artifactId: string,
    decision: AdminGeneratedArtifactReviewRequest["decision"],
  ): Promise<AdminGeneratedArtifactReviewResponse> {
    return this.postJson<AdminGeneratedArtifactReviewResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.generatedArtifactReview, "artifact_id", artifactId),
      { decision } satisfies AdminGeneratedArtifactReviewRequest,
    );
  }

  async getItemArtworkGallery(
    itemId: string,
    query: AdminItemArtworkGalleryQuery = {},
  ): Promise<AdminManagedArtworkGalleryResponse> {
    return this.getJson<AdminManagedArtworkGalleryResponse>(
      withQuery(routeWithParam(NAKO_ADMIN_ROUTES.itemArtworkGallery, "item_id", itemId), query),
    );
  }

  async selectItemArtwork(
    itemId: string,
    kind: AdminArtworkKind | string,
    artifactId: AdminSelectItemArtworkRequest["artifact_id"],
  ): Promise<PublishSelectedArtworkResponse> {
    return this.postJson<PublishSelectedArtworkResponse>(
      itemArtworkPath(NAKO_ADMIN_ROUTES.itemArtworkSelect, itemId, kind),
      { artifact_id: artifactId } satisfies AdminSelectItemArtworkRequest,
    );
  }

  async unpublishItemArtwork(
    itemId: string,
    kind: AdminArtworkKind | string,
  ): Promise<UnpublishSelectedArtworkResponse> {
    return this.deleteJson<UnpublishSelectedArtworkResponse>(
      itemArtworkPath(NAKO_ADMIN_ROUTES.itemArtworkSelection, itemId, kind),
    );
  }

  async getEvents(): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>(NAKO_ADMIN_ROUTES.events);
  }

  async getJobs(query: AdminJobsQuery = {}): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(withQuery(NAKO_ADMIN_ROUTES.jobs, query));
  }

  async getPlaybackSessions(
    query: AdminPlaybackSessionsQuery = {},
  ): Promise<AdminPlaybackSessionListResponse> {
    return this.getJson<AdminPlaybackSessionListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.playbackSessions, query),
    );
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

  async getStorageStaging(
    query: AdminStorageStagingQuery = {},
  ): Promise<AdminStorageStagingDiagnosticsResponse> {
    return this.getJson<AdminStorageStagingDiagnosticsResponse>(
      withQuery(NAKO_ADMIN_ROUTES.storageStaging, query),
    );
  }

  async getSystemConfig(): Promise<AdminServerConfigDiagnosticsResponse> {
    return this.getJson<AdminServerConfigDiagnosticsResponse>(NAKO_ADMIN_ROUTES.systemConfig);
  }

  async getMetadataRawCacheSettings(): Promise<AdminMetadataRawCacheSettingsResponse> {
    return this.getJson<AdminMetadataRawCacheSettingsResponse>(
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
    );
  }

  async updateMetadataRawCacheSettings(
    request: AdminUpdateMetadataRawCacheSettingsRequest,
  ): Promise<AdminMetadataRawCacheSettingsResponse> {
    return this.putJson<AdminMetadataRawCacheSettingsResponse>(
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
      request,
    );
  }

  async getLibraryMetadataProfile(libraryId: string): Promise<AdminLibraryMetadataProfileResponse> {
    return this.getJson<AdminLibraryMetadataProfileResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.libraryMetadataProfile, "library_id", libraryId),
    );
  }

  async updateLibraryMetadataProfile(
    libraryId: string,
    profile: AdminMetadataProfile,
  ): Promise<AdminLibraryMetadataProfileResponse> {
    return this.putJson<AdminLibraryMetadataProfileResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.libraryMetadataProfile, "library_id", libraryId),
      { profile } satisfies AdminUpdateLibraryMetadataProfileRequest,
    );
  }

  async enqueueLibraryScan(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.postJson<AdminJobCommandResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.libraryScan, "library_id", libraryId),
      {},
    );
  }

  async enqueueLibraryNfoImport(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.postJson<AdminJobCommandResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.libraryNfoImport, "library_id", libraryId),
      {},
    );
  }

  async enqueueLibraryNfoExport(libraryId: string): Promise<AdminJobCommandResponse> {
    return this.postJson<AdminJobCommandResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.libraryNfoExport, "library_id", libraryId),
      {},
    );
  }

  async getPublicLibrarySourceInventoryBridge(
    libraryId: string,
    query: { limit?: number; offset?: number } = {},
  ): Promise<PublicLibrarySourcesResponse> {
    return this.getJson<PublicLibrarySourcesResponse>(
      withQuery(`/libraries/${encodeURIComponent(libraryId)}/sources`, query),
    );
  }

  async getPublicCatalogItemsBridge(
    query: PublicCatalogItemsQuery = {},
  ): Promise<PublicCatalogItemsResponse> {
    return this.getJson<PublicCatalogItemsResponse>(withQuery("/items", query));
  }

  async getPublicCatalogSearchBridge(
    query: PublicCatalogSearchQuery = {},
  ): Promise<PublicCatalogSearchResponse> {
    return this.getJson<PublicCatalogSearchResponse>(withQuery("/search", query));
  }

  async getPublicItemDetailBridge(itemId: string): Promise<PublicItemDetailResponse> {
    return this.getJson<PublicItemDetailResponse>(`/items/${encodeURIComponent(itemId)}`);
  }

  async getPublicItemCreditsBridge(itemId: string): Promise<PublicItemCreditsResponse> {
    return this.getJson<PublicItemCreditsResponse>(`/items/${encodeURIComponent(itemId)}/credits`);
  }

  async getPublicItemImagesBridge(itemId: string): Promise<PublicItemImagesResponse> {
    return this.getJson<PublicItemImagesResponse>(`/items/${encodeURIComponent(itemId)}/images`);
  }

  async getPublicSourceProbeBridge(sourceId: string): Promise<PublicSourceProbeResponse> {
    return this.getJson<PublicSourceProbeResponse>(
      `/sources/${encodeURIComponent(sourceId)}/probe`,
    );
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

  private async deleteJson<T>(path: string): Promise<T> {
    const response = await this.fetcher(`${this.baseUrl}${path}`, {
      method: "DELETE",
      headers: this.headers(),
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

function itemArtworkPath(path: string, itemId: string, kind: string) {
  return routeWithParam(routeWithParam(path, "item_id", itemId), "kind", kind);
}

function withQuery(path: string, query: object) {
  const params = new URLSearchParams();

  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== "") {
      params.set(key, String(value));
    }
  }

  const suffix = params.toString();
  return suffix ? `${path}?${suffix}` : path;
}

function addonPath(template: string, addonId: string) {
  return template.replace(":addon_id", encodeURIComponent(addonId));
}
