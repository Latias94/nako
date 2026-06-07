import { AdminApiClient, type AdminApiClientOptions } from "./client";
import {
  mockAdminConsoleData,
  mockAcquisitionIntakeCandidates,
  mockAddonDetail,
  mockAddonDiagnostic,
  mockAddonGrants,
  mockAddonHealth,
  mockAddonInstallGuide,
  mockAccessSummary,
  mockAddons,
  mockAddonSurfaces,
  mockAddonTokens,
  mockCatalogGovernance,
  mockCatalogGovernanceItemDetail,
  mockCatalogGovernanceProviderMappingReviewPlan,
  mockPublicItemDetail,
  mockPublicCatalogItems,
  mockPublicCatalogSearch,
  mockEvents,
  mockGeneratedArtifactProposals,
  mockGeneratedArtifactReviewPlan,
  mockAdminItemArtworkGallery,
  mockJobs,
  mockLibraryMetadataProfile,
  mockMetadataRawCacheSettings,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackRuntimeSettings,
  mockPlaybackSessions,
  mockSourceDuplicateReconciliationPlan,
  mockStorageStaging,
  mockSystemConfig,
  mockVfsCacheRepairAutomationPlan,
  mockVfsCacheRepairActionPlan,
  mockVfsCacheRepairRemediationPlan,
  mockVfsCacheRepairTargetPreview,
  mockVfsCacheRepairTargets,
} from "./mockData";
import type {
  AddonGrantsResponse,
  AddonTokensResponse,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AdminAddonsQuery,
  AdminAddonHealthCheckResponse,
  AdminAddonInstallGuideResponse,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationSummary,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAddonSurfacesResponse,
  AdminAccessSummaryResponse,
  AdminCatalogGovernanceItem,
  AdminCatalogGovernanceItemDetailResponse,
  AdminCatalogGovernanceItemListResponse,
  AdminCatalogGovernanceItemsQuery,
  AdminCatalogGovernanceProviderMappingReviewPlanResponse,
  AdminCatalogGovernanceProviderMappingReviewResponse,
  AdminCatalogGovernanceProviderMappingSummary,
  AdminExternalProvider,
  AdminGeneratedArtifactProposalListResponse,
  AdminGeneratedArtifactProposalsQuery,
  AdminGeneratedArtifactReviewPlanResponse,
  AdminGeneratedArtifactReviewResponse,
  AdminMetadataSource,
  AdminArtworkKind,
  AdminItemArtworkGalleryQuery,
  AdminManagedArtworkGalleryResponse,
  AdminJobCancelRequestResponse,
  AdminJobCommandResponse,
  AdminJobListItem,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminLibraryMetadataProfileResponse,
  AdminMetadataRawCacheSettingsResponse,
  AdminUpdateMetadataRawCacheSettingsRequest,
  AdminOutboxEventListResponse,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackRuntimeSettingsResponse,
  AdminPlaybackSessionsQuery,
  AdminPlaybackSessionListResponse,
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationPlanQuery,
  AdminSourceDuplicateReconciliationPlanResponse,
  PublishSelectedArtworkResponse,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingQuery,
  AdminStorageStagingDiagnosticsResponse,
  AdminUpdatePlaybackRuntimeSettingsRequest,
  AdminVfsCacheRefreshResponse,
  AdminVfsCacheRepairActionPlanResponse,
  AdminVfsCacheRepairAutomationEnqueueRequest,
  AdminVfsCacheRepairAutomationEnqueueResponse,
  AdminVfsCacheRepairAutomationPlanResponse,
  AdminVfsCacheRepairAutomationPolicyRequest,
  AdminVfsCacheRepairEnqueueRequest,
  AdminVfsCacheRepairEnqueueResponse,
  AdminVfsCacheRepairExecuteResponse,
  AdminVfsCacheRepairRemediationPlanResponse,
  AdminVfsCacheRepairRetryRequest,
  AdminVfsCacheRepairTargetListResponse,
  AdminVfsCacheRepairTargetPreviewResponse,
  UnpublishSelectedArtworkResponse,
} from "./generated/contract";
import type {
  AddonResource,
  AdminConsoleData,
  AdminErrorMap,
  AdminSectionKey,
  AdminSourceMap,
  AddonManifestPreview,
  AddonGrantAssignmentInput,
  AddonDiagnosticSummary,
  AddonHealthSummary,
  AddonInstallGuideSummary,
  AddonOnboardingResult,
  AddonOperationsSummary,
  AddonsRouteSummary,
  AddonTokenActionResult,
  AddonTokenSummaryRow,
  AddonGrantSummaryRow,
  CatalogBrowseQuery,
  CatalogBrowseSummary,
  CatalogGovernanceItemDetailSummary,
  CatalogGovernanceItemSummary,
  CatalogGovernanceProviderMappingSummary,
  CatalogGovernanceProviderMappingReviewDecision,
  CatalogGovernanceProviderMappingReviewPlanSummary,
  CatalogGovernanceProviderMappingReviewResultSummary,
  CatalogGovernanceSummary,
  DataSourceMode,
  EventSummary,
  GeneratedArtifactReviewDecision,
  GeneratedArtifactProposalSummary,
  GeneratedArtifactReviewPlanSummary,
  GeneratedArtifactReviewResultSummary,
  IntakeSummary,
  ItemArtworkGallerySummary,
  ItemArtworkMutationResultSummary,
  ItemDetailSummary,
  JobRow,
  LibraryCommandAction,
  LibraryCommandResult,
  LibraryJobSummary,
  LibraryManagementDetail,
  LibrarySourceInventorySummary,
  NetworkSummary,
  PlaybackSummary,
  PublicCatalogItemsResponse,
  PublicCatalogMediaItem,
  PublicCatalogSearchResponse,
  PublicItemDetailResponse,
  PublicMediaProbe,
  PublicLibrarySourcesResponse,
  PublicSourceProbeResponse,
  AdminMetadataProfile,
  SettingRow,
  StorageSummary,
} from "./types";

export type { AdminConsoleData, AdminSourceMap, DataSourceMode };

export type AdminDataSource = {
  load(): Promise<AdminConsoleData>;
  loadAccessSummary?(): Promise<AdminSectionResult<AdminAccessSummaryResponse>>;
  loadOverview?(): Promise<AdminSectionResult<AdminOverviewResponse>>;
  loadAddons?(query?: AdminAddonsQuery): Promise<AdminSectionResult<AddonsRouteSummary>>;
  loadJobs?(query?: AdminJobsQuery): Promise<AdminSectionResult<AdminJobListResponse>>;
  cancelJob?(jobId: string): Promise<AdminJobCancelRequestResponse>;
  loadLibraries?(): Promise<AdminSectionResult<AdminServerConfigDiagnosticsResponse>>;
  loadLibraryDetail?(libraryId: string): Promise<AdminSectionResult<LibraryManagementDetail>>;
  loadSettings?(): Promise<AdminSectionResult<AdminServerConfigDiagnosticsResponse>>;
  loadMetadataRawCacheSettings?(): Promise<AdminSectionResult<AdminMetadataRawCacheSettingsResponse>>;
  updateMetadataRawCacheSettings?(
    request: AdminUpdateMetadataRawCacheSettingsRequest,
  ): Promise<AdminMetadataRawCacheSettingsResponse>;
  loadAcquisitionIntake?(
    query?: AdminAcquisitionIntakeCandidatesQuery,
  ): Promise<AdminSectionResult<AdminAcquisitionIntakeCandidateListResponse>>;
  loadGeneratedArtifacts?(
    query?: AdminGeneratedArtifactProposalsQuery,
  ): Promise<AdminSectionResult<AdminGeneratedArtifactProposalListResponse>>;
  loadGeneratedArtifactReviewPlan?(
    artifactId: string,
    decision: GeneratedArtifactReviewDecision,
  ): Promise<AdminSectionResult<GeneratedArtifactReviewPlanSummary>>;
  reviewGeneratedArtifact?(
    artifactId: string,
    decision: GeneratedArtifactReviewDecision,
  ): Promise<GeneratedArtifactReviewResultSummary>;
  loadItemArtworkGallery?(
    itemId: string,
    query?: AdminItemArtworkGalleryQuery,
  ): Promise<AdminSectionResult<ItemArtworkGallerySummary>>;
  selectItemArtwork?(
    itemId: string,
    kind: AdminArtworkKind | string,
    artifactId: string,
  ): Promise<ItemArtworkMutationResultSummary>;
  unpublishItemArtwork?(
    itemId: string,
    kind: AdminArtworkKind | string,
  ): Promise<ItemArtworkMutationResultSummary>;
  loadCatalog?(query?: CatalogBrowseQuery): Promise<AdminSectionResult<CatalogBrowseSummary>>;
  loadItemDetail?(itemId: string): Promise<AdminSectionResult<ItemDetailSummary>>;
  loadCatalogGovernance?(
    query?: AdminCatalogGovernanceItemsQuery,
  ): Promise<AdminSectionResult<AdminCatalogGovernanceItemListResponse>>;
  loadCatalogGovernanceItemDetail?(
    itemId: string,
  ): Promise<AdminSectionResult<CatalogGovernanceItemDetailSummary>>;
  loadCatalogGovernanceProviderMappingReviewPlan?(
    itemId: string,
    mappingId: string,
    decision: CatalogGovernanceProviderMappingReviewDecision,
  ): Promise<AdminSectionResult<CatalogGovernanceProviderMappingReviewPlanSummary>>;
  reviewCatalogGovernanceProviderMapping?(
    itemId: string,
    mappingId: string,
    decision: CatalogGovernanceProviderMappingReviewDecision,
  ): Promise<CatalogGovernanceProviderMappingReviewResultSummary>;
  loadPlaybackSessions?(
    query?: AdminPlaybackSessionsQuery,
  ): Promise<AdminSectionResult<AdminPlaybackSessionListResponse>>;
  loadPlaybackRuntimeSettings?(): Promise<AdminSectionResult<AdminPlaybackRuntimeSettingsResponse>>;
  updatePlaybackRuntimeSettings?(
    request: AdminUpdatePlaybackRuntimeSettingsRequest,
  ): Promise<AdminPlaybackRuntimeSettingsResponse>;
  loadSourceDuplicateReconciliationPlan?(
    libraryId: string,
    sourceId: string,
    query?: AdminSourceDuplicateReconciliationPlanQuery,
  ): Promise<AdminSectionResult<AdminSourceDuplicateReconciliationPlanResponse>>;
  applySourceDuplicateReconciliation?(
    libraryId: string,
    sourceId: string,
    duplicateSourceId: string,
  ): Promise<AdminSourceDuplicateReconciliationApplyResponse>;
  loadStorageStaging?(
    query?: AdminStorageStagingQuery,
  ): Promise<AdminSectionResult<AdminStorageStagingDiagnosticsResponse>>;
  loadVfsCacheRepairActionPlan?(): Promise<AdminSectionResult<AdminVfsCacheRepairActionPlanResponse>>;
  loadVfsCacheRepairRemediationPlan?(): Promise<AdminSectionResult<AdminVfsCacheRepairRemediationPlanResponse>>;
  loadVfsCacheRepairAutomationPlan?(
    request?: AdminVfsCacheRepairAutomationPolicyRequest,
  ): Promise<AdminSectionResult<AdminVfsCacheRepairAutomationPlanResponse>>;
  loadVfsCacheRepairTargets?(
    query?: { limit?: number; offset?: number },
  ): Promise<AdminSectionResult<AdminVfsCacheRepairTargetListResponse>>;
  loadVfsCacheRepairTargetPreview?(
    targetRef: string,
  ): Promise<AdminSectionResult<AdminVfsCacheRepairTargetPreviewResponse>>;
  refreshLatestVfsCacheRepair?(): Promise<AdminVfsCacheRefreshResponse>;
  refreshVfsCacheRepairTarget?(targetRef: string): Promise<AdminVfsCacheRefreshResponse>;
  enqueueVfsCacheRepairTarget?(
    targetRef: string,
    request?: AdminVfsCacheRepairEnqueueRequest,
  ): Promise<AdminVfsCacheRepairEnqueueResponse>;
  enqueueVfsCacheRepairAutomation?(
    request?: AdminVfsCacheRepairAutomationEnqueueRequest,
  ): Promise<AdminVfsCacheRepairAutomationEnqueueResponse>;
  executeVfsCacheRepairJob?(jobId: string): Promise<AdminVfsCacheRepairExecuteResponse>;
  retryVfsCacheRepairJob?(
    jobId: string,
    request?: AdminVfsCacheRepairRetryRequest,
  ): Promise<AdminJobListItem>;
  updateLibraryMetadataProfile?(
    libraryId: string,
    profile: AdminMetadataProfile,
  ): Promise<AdminLibraryMetadataProfileResponse>;
  runLibraryCommand?(
    libraryId: string,
    action: LibraryCommandAction,
  ): Promise<LibraryCommandResult>;
  setAddonStatus?(addonId: string, status: "enabled" | "disabled"): Promise<AddonOperationsSummary>;
  checkAddonHealth?(addonId: string): Promise<AddonHealthSummary>;
  diagnoseAddonResource?(addonId: string, resource: AddonResource): Promise<AddonDiagnosticSummary>;
  previewAddonManifestJson?(manifestJson: string): AddonManifestPreview;
  registerAddonManifestJson?(manifestJson: string): Promise<AddonOnboardingResult>;
  issueAddonToken?(addonId: string, label: string): Promise<AddonTokenActionResult>;
  rotateAddonToken?(addonId: string, tokenId: string, label: string): Promise<AddonTokenActionResult>;
  revokeAddonToken?(addonId: string, tokenId: string): Promise<AddonTokenSummaryRow>;
  replaceAddonGrants?(
    addonId: string,
    grants: AddonGrantAssignmentInput[],
  ): Promise<AddonGrantSummaryRow[]>;
};

type LoadResult<T> = {
  value: T;
  source: DataSourceMode;
  error?: string;
};

export type AdminSectionResult<T> = LoadResult<T>;

export function createAdminDataSource(options: AdminApiClientOptions = {}): AdminDataSource {
  const client = new AdminApiClient(options);

  return {
    async load() {
      const [
        overview,
        addons,
        addonDetail,
        addonHealth,
        addonSurfaces,
        addonInstallGuide,
        addonTokens,
        addonGrants,
        addonDiagnostic,
        catalogGovernance,
        acquisitionIntakeCandidates,
        generatedArtifactProposals,
        events,
        jobs,
        playbackSessions,
        playbackRuntime,
        storageStaging,
        systemConfig,
      ] = await Promise.all([
        loadSection(() => client.getOverview(), mockOverview),
        loadSection(() => client.getAddons(), mockAddons),
        loadSection(() => client.getAddonDetail(mockAddons.addons[0]?.id ?? ""), mockAddonDetail),
        loadSection(() => client.checkAddonHealth(mockAddons.addons[0]?.id ?? ""), mockAddonHealth),
        loadSection(() => client.getAddonSurfaces(mockAddons.addons[0]?.id ?? ""), mockAddonSurfaces),
        loadSection(
          () => client.getAddonInstallGuide(mockAddons.addons[0]?.id ?? ""),
          mockAddonInstallGuide,
        ),
        loadSection(() => client.getAddonTokens(mockAddons.addons[0]?.id ?? ""), mockAddonTokens),
        loadSection(() => client.getAddonGrants(mockAddons.addons[0]?.id ?? ""), mockAddonGrants),
        loadSection(
          () =>
            client.diagnoseAddonResourceCall(mockAddons.addons[0]?.id ?? "", {
              resource: mockAddonDiagnostic.resource,
              payload: {},
            }),
          mockAddonDiagnostic,
        ),
        loadSection(() => client.getCatalogGovernanceItems(), mockCatalogGovernance),
        loadSection(() => client.getAcquisitionIntakeCandidates(), mockAcquisitionIntakeCandidates),
        loadSection(() => client.getGeneratedArtifactProposals(), mockGeneratedArtifactProposals),
        loadSection(() => client.getEvents(), mockEvents),
        loadSection(() => client.getJobs(), mockJobs),
        loadSection(() => client.getPlaybackSessions(), mockPlaybackSessions),
        loadSection(() => client.getPlaybackRuntime(), mockPlaybackRuntime),
        loadSection(() => client.getStorageStaging(), mockStorageStaging),
        loadSection(() => client.getSystemConfig(), mockSystemConfig),
      ]);

      const sources: AdminSourceMap = {
        overview: overview.source,
        addons: addons.source,
        addonHealth: addonHealth.source,
        addonSurfaces: addonSurfaces.source,
        addonInstallGuide: addonInstallGuide.source,
        addonTokens: addonTokens.source,
        addonGrants: addonGrants.source,
        catalogGovernance: catalogGovernance.source,
        acquisitionIntake: acquisitionIntakeCandidates.source,
        generatedArtifactProposals: generatedArtifactProposals.source,
        events: events.source,
        jobs: jobs.source,
        playbackSessions: playbackSessions.source,
        playbackRuntime: playbackRuntime.source,
        storageStaging: storageStaging.source,
        systemConfig: systemConfig.source,
      };
      const errors: AdminErrorMap = {};

      recordError(errors, "overview", overview);
      recordError(errors, "addons", addons);
      recordError(errors, "addonHealth", addonHealth);
      recordError(errors, "addonSurfaces", addonSurfaces);
      recordError(errors, "addonInstallGuide", addonInstallGuide);
      recordError(errors, "addonTokens", addonTokens);
      recordError(errors, "addonGrants", addonGrants);
      recordError(errors, "catalogGovernance", catalogGovernance);
      recordError(errors, "acquisitionIntake", acquisitionIntakeCandidates);
      recordError(errors, "generatedArtifactProposals", generatedArtifactProposals);
      recordError(errors, "events", events);
      recordError(errors, "jobs", jobs);
      recordError(errors, "playbackSessions", playbackSessions);
      recordError(errors, "playbackRuntime", playbackRuntime);
      recordError(errors, "storageStaging", storageStaging);
      recordError(errors, "systemConfig", systemConfig);

      return {
        ...mockAdminConsoleData,
        sources,
        errors,
        overview: overview.value,
        addons: mapAddons(
          addons.value,
          addonDetail.value,
          addonHealth.value,
          addonSurfaces.value,
          addonInstallGuide.value,
          addonTokens.value,
          addonGrants.value,
          addonDiagnostic.value,
        ),
        catalog: mapCatalogGovernance(catalogGovernance.value),
        acquisitionIntake: mapAcquisitionIntake(acquisitionIntakeCandidates.value),
        generatedArtifactProposals: mapGeneratedArtifactProposals(
          generatedArtifactProposals.value,
        ),
        events: mapEvents(events.value),
        jobs: mapJobs(jobs.value),
        playback: mapPlayback(playbackSessions.value, playbackRuntime.value),
        storage: mapStorage(storageStaging.value),
        network: mapNetwork(systemConfig.value.network),
        settings: mapSettings(systemConfig.value),
      };
    },
    async loadJobs(query = {}) {
      return loadSection(() => client.getJobs(query), mockJobs);
    },
    async cancelJob(jobId) {
      return client.cancelJob(jobId);
    },
    async loadOverview() {
      return loadSection(() => client.getOverview(), mockOverview);
    },
    async loadAccessSummary() {
      return loadSection(() => client.getAccessSummary(), mockAccessSummary);
    },
    async loadAddons(query = {}) {
      return loadAddonsRouteSummary(client, query);
    },
    async loadLibraries() {
      return loadSection(() => client.getSystemConfig(), mockSystemConfig);
    },
    async loadLibraryDetail(libraryId) {
      return loadLibraryDetail(client, libraryId);
    },
    async updateLibraryMetadataProfile(libraryId, profile) {
      return client.updateLibraryMetadataProfile(libraryId, profile);
    },
    async runLibraryCommand(libraryId, action) {
      return {
        action,
        job: mapLibraryJob(await enqueueLibraryCommand(client, libraryId, action)),
      };
    },
    async loadSettings() {
      return loadSection(() => client.getSystemConfig(), mockSystemConfig);
    },
    async loadMetadataRawCacheSettings() {
      return loadSection(
        () => client.getMetadataRawCacheSettings(),
        mockMetadataRawCacheSettings,
      );
    },
    async updateMetadataRawCacheSettings(request) {
      return client.updateMetadataRawCacheSettings(request);
    },
    async loadAcquisitionIntake(query = {}) {
      return loadSection(
        () => client.getAcquisitionIntakeCandidates(query),
        mockAcquisitionIntakeCandidates,
      );
    },
    async loadGeneratedArtifacts(query = {}) {
      return loadSection(
        () => client.getGeneratedArtifactProposals(query),
        mockGeneratedArtifactProposals,
      );
    },
    async loadGeneratedArtifactReviewPlan(artifactId, decision) {
      return loadGeneratedArtifactReviewPlan(client, artifactId, decision);
    },
    async reviewGeneratedArtifact(artifactId, decision) {
      return mapGeneratedArtifactReviewResult(
        await client.reviewGeneratedArtifact(artifactId, decision),
      );
    },
    async loadItemArtworkGallery(itemId, query = {}) {
      return loadItemArtworkGallery(client, itemId, query);
    },
    async selectItemArtwork(itemId, kind, artifactId) {
      return mapPublishSelectedArtwork(
        await client.selectItemArtwork(itemId, kind, artifactId),
      );
    },
    async unpublishItemArtwork(itemId, kind) {
      return mapUnpublishSelectedArtwork(await client.unpublishItemArtwork(itemId, kind));
    },
    async loadCatalog(query = {}) {
      return loadCatalogBrowse(client, query);
    },
    async loadItemDetail(itemId) {
      return loadItemDetail(client, itemId);
    },
    async loadCatalogGovernance(query = {}) {
      return loadSection(() => client.getCatalogGovernanceItems(query), mockCatalogGovernance);
    },
    async loadCatalogGovernanceItemDetail(itemId) {
      return loadCatalogGovernanceItemDetail(client, itemId);
    },
    async loadCatalogGovernanceProviderMappingReviewPlan(itemId, mappingId, decision) {
      return loadCatalogGovernanceProviderMappingReviewPlan(
        client,
        itemId,
        mappingId,
        decision,
      );
    },
    async reviewCatalogGovernanceProviderMapping(itemId, mappingId, decision) {
      return mapCatalogGovernanceProviderMappingReviewResult(
        await client.reviewCatalogGovernanceProviderMapping(itemId, mappingId, decision),
      );
    },
    async loadPlaybackSessions(query = {}) {
      return loadSection(() => client.getPlaybackSessions(query), mockPlaybackSessions);
    },
    async loadPlaybackRuntimeSettings() {
      return loadSection(
        () => client.getPlaybackRuntimeSettings(),
        mockPlaybackRuntimeSettings,
      );
    },
    async updatePlaybackRuntimeSettings(request) {
      return client.updatePlaybackRuntimeSettings(request);
    },
    async loadSourceDuplicateReconciliationPlan(libraryId, sourceId, query = {}) {
      return loadSection(
        () => client.getSourceDuplicateReconciliationPlan(libraryId, sourceId, query),
        mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
      );
    },
    async applySourceDuplicateReconciliation(libraryId, sourceId, duplicateSourceId) {
      return client.applySourceDuplicateReconciliation(libraryId, sourceId, {
        duplicate_source_id: duplicateSourceId,
        expected_action: "suggest_relationship",
      });
    },
    async loadStorageStaging(query = {}) {
      return loadSection(() => client.getStorageStaging(query), mockStorageStaging);
    },
    async loadVfsCacheRepairActionPlan() {
      return loadSection(
        () => client.getVfsCacheRepairActionPlan(),
        mockVfsCacheRepairActionPlan,
      );
    },
    async loadVfsCacheRepairRemediationPlan() {
      return loadSection(
        () => client.getVfsCacheRepairRemediationPlan(),
        mockVfsCacheRepairRemediationPlan,
      );
    },
    async loadVfsCacheRepairAutomationPlan(request = { enabled: true }) {
      return loadSection(
        () => client.planVfsCacheRepairAutomation(request),
        mockVfsCacheRepairAutomationPlan,
      );
    },
    async loadVfsCacheRepairTargets(query = {}) {
      return loadSection(
        () => client.getVfsCacheRepairTargets(query),
        mockVfsCacheRepairTargets,
      );
    },
    async loadVfsCacheRepairTargetPreview(targetRef) {
      return loadSection(
        () => client.getVfsCacheRepairTargetPreview(targetRef),
        mockVfsCacheRepairTargetPreview,
      );
    },
    async refreshLatestVfsCacheRepair() {
      return client.refreshLatestVfsCacheRepair();
    },
    async refreshVfsCacheRepairTarget(targetRef) {
      return client.refreshVfsCacheRepairTarget(targetRef);
    },
    async enqueueVfsCacheRepairTarget(targetRef, request = {}) {
      return client.enqueueVfsCacheRepairTarget(targetRef, request);
    },
    async enqueueVfsCacheRepairAutomation(request = { enabled: true }) {
      return client.enqueueVfsCacheRepairAutomation(request);
    },
    async executeVfsCacheRepairJob(jobId) {
      return client.executeVfsCacheRepairJob(jobId);
    },
    async retryVfsCacheRepairJob(jobId, request = {}) {
      return client.retryVfsCacheRepairJob(jobId, request);
    },
    async setAddonStatus(addonId, status) {
      const updated = await client.updateAddonStatus(addonId, { status });
      return mapAddons(
        { addons: [updated.addon.summary] },
        updated,
        mockAddonHealth,
        mockAddonSurfaces,
        mockAddonInstallGuide,
        mockAddonTokens,
        mockAddonGrants,
        mockAddonDiagnostic,
      );
    },
    async checkAddonHealth(addonId) {
      return mapAddonHealth(await client.checkAddonHealth(addonId));
    },
    async diagnoseAddonResource(addonId, resource) {
      return mapAddonDiagnostic(
        await client.diagnoseAddonResourceCall(addonId, {
          resource,
          payload: {},
        }),
      );
    },
    previewAddonManifestJson(manifestJson) {
      return previewAddonManifestJson(manifestJson);
    },
    async registerAddonManifestJson(manifestJson) {
      const preview = previewAddonManifestJson(manifestJson);
      if (preview.status === "invalid_json" || !preview.manifest) {
        return {
          status: "invalid_json",
          error: preview.error ?? "Manifest JSON could not be parsed.",
        };
      }

      try {
        return mapAddonOnboardingResult(
          await client.registerAddon(preview.manifest, {
            grantedScopes: [],
            status: "disabled",
          }),
        );
      } catch (error: unknown) {
        return {
          status: "server_error",
          error: error instanceof Error ? error.message : "Addon registration failed.",
        };
      }
    },
    async issueAddonToken(addonId, label) {
      const response = await client.issueAddonToken(addonId, { label });
      return {
        token: mapAddonToken(response.token),
        rawToken: response.raw_token,
      };
    },
    async rotateAddonToken(addonId, tokenId, label) {
      const response = await client.rotateAddonToken(addonId, tokenId, { label });
      return {
        token: mapAddonToken(response.token),
        rawToken: response.raw_token,
      };
    },
    async revokeAddonToken(addonId, tokenId) {
      return mapAddonToken((await client.revokeAddonToken(addonId, tokenId)).token);
    },
    async replaceAddonGrants(addonId, grants) {
      const response = await client.replaceAddonGrants(addonId, {
        grants: grants.map((grant) => ({
          permission: grant.permission,
          library_id: grant.libraryId,
        })),
      });
      return response.grants.map(mapAddonGrant);
    },
  };
}

async function loadLibraryDetail(
  client: AdminApiClient,
  libraryId: string,
): Promise<AdminSectionResult<LibraryManagementDetail>> {
  const [systemConfig, metadataProfile, sourceInventory, jobs] = await Promise.all([
    loadSection(() => client.getSystemConfig(), mockSystemConfig),
    loadSection(() => client.getLibraryMetadataProfile(libraryId), mockLibraryMetadataProfile(libraryId)),
    loadSection(
      () => client.getPublicLibrarySourceInventoryBridge(libraryId, { limit: 50, offset: 0 }),
      emptyPublicLibrarySourcesResponse(libraryId),
    ),
    loadSection(() => client.getJobs({ library_id: libraryId, limit: 5, offset: 0 }), mockJobs),
  ]);
  const sourceInventoryResult = mapLibrarySourceInventory(
    sourceInventory.value,
    jobs.value,
    libraryId,
    combineLoadSources([sourceInventory.source, jobs.source]),
    combineLoadErrors([sourceInventory, jobs]),
  );

  return {
    value: {
      configuredLibraryCount: systemConfig.value.libraries.length,
      library:
        systemConfig.value.libraries.find((library) => library.id === libraryId) ?? null,
      metadataProfile: metadataProfile.value,
      sourceInventory: sourceInventoryResult,
    },
    source: combineLoadSources([
      systemConfig.source,
      metadataProfile.source,
      sourceInventory.source,
      jobs.source,
    ]),
    error: combineLoadErrors([systemConfig, metadataProfile, sourceInventory, jobs]),
  };
}

async function loadGeneratedArtifactReviewPlan(
  client: AdminApiClient,
  artifactId: string,
  decision: GeneratedArtifactReviewDecision,
): Promise<AdminSectionResult<GeneratedArtifactReviewPlanSummary>> {
  const result = await loadSection(
    () => client.planGeneratedArtifactReview(artifactId, decision),
    mockGeneratedArtifactReviewPlan(artifactId, decision),
  );

  return {
    value: mapGeneratedArtifactReviewPlan(result.value),
    source: result.source,
    error: result.error,
  };
}

async function loadCatalogGovernanceItemDetail(
  client: AdminApiClient,
  itemId: string,
): Promise<AdminSectionResult<CatalogGovernanceItemDetailSummary>> {
  const result = await loadSection(
    () => client.getCatalogGovernanceItemDetail(itemId),
    mockCatalogGovernanceItemDetail(itemId),
  );

  return {
    value: mapCatalogGovernanceItemDetail(result.value),
    source: result.source,
    error: result.error,
  };
}

async function loadCatalogGovernanceProviderMappingReviewPlan(
  client: AdminApiClient,
  itemId: string,
  mappingId: string,
  decision: CatalogGovernanceProviderMappingReviewDecision,
): Promise<AdminSectionResult<CatalogGovernanceProviderMappingReviewPlanSummary>> {
  const result = await loadSection(
    () => client.planCatalogGovernanceProviderMappingReview(itemId, mappingId, decision),
    mockCatalogGovernanceProviderMappingReviewPlan(itemId, mappingId, decision),
  );

  return {
    value: mapCatalogGovernanceProviderMappingReviewPlan(result.value),
    source: result.source,
    error: result.error,
  };
}

async function loadCatalogBrowse(
  client: AdminApiClient,
  query: CatalogBrowseQuery,
): Promise<AdminSectionResult<CatalogBrowseSummary>> {
  if (isCatalogSearch(query)) {
    const searchResult = await loadSection(
      () => client.getPublicCatalogSearchBridge(query),
      mockPublicCatalogSearch,
    );

    return {
      value: mapPublicCatalogSearch(searchResult.value),
      source: searchResult.source,
      error: searchResult.error,
    };
  }

  const browseResult = await loadSection(
    () =>
      client.getPublicCatalogItemsBridge({
        limit: query.limit,
        offset: query.offset,
      }),
    mockPublicCatalogItems,
  );

  return {
    value: mapPublicCatalogItems(browseResult.value),
    source: browseResult.source,
    error: browseResult.error,
  };
}

async function loadItemDetail(
  client: AdminApiClient,
  itemId: string,
): Promise<AdminSectionResult<ItemDetailSummary>> {
  const detail = await loadSection(
    () => client.getPublicItemDetailBridge(itemId),
    mockPublicItemDetail(itemId),
  );

  if (detail.error) {
    return {
      value: mapPublicItemDetail(detail.value, new Map()),
      source: detail.source,
      error: detail.error,
    };
  }

  const probedSourceIds = detail.value.sources.slice(0, 3).map((source) => source.id);
  const probeResults = await Promise.all(
    probedSourceIds.map((sourceId) =>
      loadSection(
        () => client.getPublicSourceProbeBridge(sourceId),
        emptyPublicSourceProbeResponse(sourceId),
      ),
    ),
  );
  const successfulProbes = new Map<string, PublicMediaProbe>();

  for (const result of probeResults) {
    if (!result.error) {
      successfulProbes.set(result.value.source_id, result.value.probe);
    }
  }

  const sections = [detail, ...probeResults];

  return {
    value: mapPublicItemDetail(detail.value, successfulProbes),
    source: combineLoadSources(sections.map((section) => section.source)),
    error: combineLoadErrors(sections),
  };
}

async function loadItemArtworkGallery(
  client: AdminApiClient,
  itemId: string,
  query: AdminItemArtworkGalleryQuery,
): Promise<AdminSectionResult<ItemArtworkGallerySummary>> {
  const gallery = await loadSection(
    () => client.getItemArtworkGallery(itemId, query),
    mockAdminItemArtworkGallery(itemId),
  );

  return {
    value: mapItemArtworkGallery(gallery.value),
    source: gallery.source,
    error: gallery.error,
  };
}

function isCatalogSearch(query: CatalogBrowseQuery) {
  return Boolean(query.q?.trim() || query.facet?.trim());
}

function mapPublicCatalogItems(response: PublicCatalogItemsResponse): CatalogBrowseSummary {
  return {
    mode: "browse",
    items: response.items.map((item) => mapPublicCatalogItem(item, null)),
    page: response.page,
  };
}

function mapPublicCatalogSearch(response: PublicCatalogSearchResponse): CatalogBrowseSummary {
  return {
    mode: "search",
    items: response.hits.map((hit) => mapPublicCatalogItem(hit.item, hit.score)),
    page: response.page,
  };
}

function mapPublicCatalogItem(
  item: PublicCatalogMediaItem,
  score: number | null,
): CatalogBrowseSummary["items"][number] {
  return {
    id: item.id,
    parentId: item.parent_id,
    title: item.metadata.title,
    kind: item.kind,
    releaseDate: item.metadata.release_date,
    runtimeMinutes: item.metadata.runtime_minutes,
    genreCount: item.metadata.genres.length,
    tagCount: item.metadata.tags.length,
    creditCount: item.metadata.credits.length,
    collectionCount: item.metadata.collections.length,
    studioCount: item.metadata.studios.length,
    imageCount: null,
    sourceCount: null,
    score,
  };
}

function mapPublicItemDetail(
  response: PublicItemDetailResponse,
  probes: Map<string, PublicMediaProbe>,
): ItemDetailSummary {
  const metadata = response.item.metadata;

  return {
    item: {
      id: response.item.id,
      parentId: response.item.parent_id,
      title: metadata.title,
      kind: response.item.kind,
      releaseDate: metadata.release_date,
      runtimeMinutes: metadata.runtime_minutes,
      genreCount: metadata.genres.length,
      tagCount: metadata.tags.length,
      creditCount: response.credits.length || metadata.credits.length,
      collectionCount: response.collections.length || metadata.collections.length,
      studioCount: response.studios.length || metadata.studios.length,
      imageCount: response.images.length,
      sourceCount: response.sources.length,
    },
    canonical: {
      genres: metadata.genres,
      tags: metadata.tags,
      credits: metadata.credits.slice(0, 5).map((credit) => ({
        name: credit.name,
        role: credit.role,
        character: credit.character,
      })),
      collections: metadata.collections.map((collection) => collection.name),
      studios: metadata.studios.map((studio) => studio.name),
      ratingCount: metadata.ratings.length,
      externalIdCount: metadata.external_ids.length,
    },
    sources: response.sources.map((source) => ({
      id: source.id,
      libraryId: source.library_id,
      fileName: source.file_name,
      sizeBytes: source.size_bytes,
      hasFingerprint: Boolean(source.fingerprint),
      probe: mapProbeSummary(probes.get(source.id) ?? null),
    })),
    images: response.images.map((image) => ({
      id: image.id,
      kind: image.kind,
      routePath: safeImageRoutePath(image.url),
      width: image.width,
      height: image.height,
      language: image.language,
      mediaType: image.media_type,
      hasEtag: Boolean(image.etag),
    })),
    readiness: [
      {
        label: "NFO sidecar status",
        status: "split",
        detail: "Admin NFO item status read model is split from MBG-040.",
      },
      {
        label: "Provider Mapping",
        status: "split",
        detail: "Provider Mapping decisions need a follow-on repair workflow.",
      },
      {
        label: "Local Inference",
        status: "split",
        detail: "Local Inference evidence diagnostics need an Admin read model.",
      },
      {
        label: "Generated Artifacts",
        status: "planned",
        detail: "Use route-level automation queue until per-item review is split.",
      },
      {
        label: "Artwork decisions",
        status: response.images.length > 0 ? "ready" : "planned",
        detail: response.images.length > 0
          ? "Public image refs are available for presentation readiness."
          : "Admin artwork selection workflow remains split.",
      },
      {
        label: "Catalog repair",
        status: "split",
        detail: "Repair/apply mutations stay out of the first item detail slice.",
      },
    ],
  };
}

function mapProbeSummary(probe: PublicMediaProbe | null): ItemDetailSummary["sources"][number]["probe"] {
  if (!probe) {
    return null;
  }

  return {
    durationMs: probe.duration_ms,
    container: probe.container,
    bitRate: probe.bit_rate,
    streamCount: probe.streams.length,
    videoStreamCount: probe.streams.filter((stream) => stream.kind === "video").length,
    audioStreamCount: probe.streams.filter((stream) => stream.kind === "audio").length,
    subtitleStreamCount: probe.streams.filter((stream) => stream.kind === "subtitle").length,
  };
}

function safeImageRoutePath(url: string | null): string | null {
  return url?.startsWith("/images/") ? url : null;
}

function mapItemArtworkGallery(
  response: AdminManagedArtworkGalleryResponse,
): ItemArtworkGallerySummary {
  return {
    itemId: response.item_id,
    totals: {
      candidateCount: response.summary.candidates,
      artifactCount: response.summary.artifacts,
      selectedCount: response.summary.selected,
    },
    candidates: response.candidates.map((candidate) => ({
      id: candidate.id,
      addonId: candidate.addon_id,
      sideEffectId: candidate.side_effect_id,
      libraryId: candidate.library_id,
      itemId: candidate.item_id,
      kind: candidate.kind,
      sourceKind: candidate.source_kind,
      status: candidate.status,
      width: candidate.width,
      height: candidate.height,
      language: candidate.language,
      ingestId: candidate.ingest?.id ?? null,
      ingestStatus: candidate.ingest?.status ?? null,
      hasIngestArtifact: candidate.ingest?.has_artifact ?? false,
      hasIngestFailure: candidate.ingest?.has_failure ?? false,
      ingestFailureCode: candidate.ingest?.failure_code ?? null,
      artifactId: candidate.artifact_id,
      hasStoredArtifact: candidate.has_stored_artifact,
      selectedArtworkCount: candidate.selected_artwork_count,
      selected: candidate.selected,
      updatedAt: candidate.updated_at,
    })),
    artifacts: response.artifacts.map((artifact) => ({
      id: artifact.id,
      ingestId: artifact.ingest_id,
      candidateId: artifact.candidate_id,
      libraryId: artifact.library_id,
      itemId: artifact.item_id,
      kind: artifact.kind,
      selectedArtworkCount: artifact.selected_artwork_count,
      selected: artifact.selected,
      width: artifact.width,
      height: artifact.height,
      byteLen: artifact.byte_len,
      mediaType: artifact.media_type,
      hasContentHash: artifact.has_content_hash,
      updatedAt: artifact.updated_at,
    })),
    selected: response.selected.map((selection) => ({
      selectedArtworkId: selection.selected_artwork.id,
      libraryId: selection.selected_artwork.library_id,
      itemId: selection.selected_artwork.item_id,
      kind: selection.selected_artwork.kind,
      artifactId: selection.selected_artwork.artifact_id,
      imageId: selection.image.id,
      routePath: safeImageRoutePath(selection.image.url),
      width: selection.image.width,
      height: selection.image.height,
      language: selection.image.language,
      mediaType: selection.image.media_type,
      selectedAt: selection.selected_artwork.created_at,
      updatedAt: selection.selected_artwork.updated_at,
    })),
    page: response.page,
  };
}

function mapPublishSelectedArtwork(
  response: PublishSelectedArtworkResponse,
): ItemArtworkMutationResultSummary {
  return {
    action: "select",
    itemId: response.selected_artwork.item_id,
    kind: response.selected_artwork.kind,
    changed: response.changed,
    selectedArtworkId: response.selected_artwork.id,
    artifactId: response.selected_artwork.artifact_id,
    imageId: response.image.id,
    routePath: safeImageRoutePath(response.image.url),
    width: response.image.width,
    height: response.image.height,
    language: response.image.language,
    mediaType: response.image.media_type,
  };
}

function mapUnpublishSelectedArtwork(
  response: UnpublishSelectedArtworkResponse,
): ItemArtworkMutationResultSummary {
  return {
    action: "unpublish",
    itemId: response.item_id,
    kind: response.kind,
    changed: response.changed,
    selectedArtworkId: response.unpublished?.selected_artwork.id ?? null,
    artifactId: response.unpublished?.selected_artwork.artifact_id ?? null,
    imageId: response.unpublished?.previous_image.id ?? null,
    routePath: safeImageRoutePath(response.unpublished?.previous_image.url ?? null),
    width: response.unpublished?.previous_image.width ?? null,
    height: response.unpublished?.previous_image.height ?? null,
    language: response.unpublished?.previous_image.language ?? null,
    mediaType: response.unpublished?.previous_image.media_type ?? null,
  };
}

function emptyPublicSourceProbeResponse(sourceId: string): PublicSourceProbeResponse {
  return {
    source_id: sourceId,
    probe: {
      duration_ms: null,
      container: null,
      bit_rate: null,
      streams: [],
    },
  };
}

function emptyPublicLibrarySourcesResponse(libraryId: string): PublicLibrarySourcesResponse {
  return {
    library: {
      id: libraryId,
      name: libraryId,
    },
    sources: [],
    page: {
      limit: 50,
      offset: 0,
      returned: 0,
    },
  };
}

function mapLibrarySourceInventory(
  response: PublicLibrarySourcesResponse,
  jobs: AdminJobListResponse,
  libraryId: string,
  source: DataSourceMode,
  error?: string,
): LibrarySourceInventorySummary {
  const libraryJobs = jobs.jobs.filter((job) => job.library_id === libraryId);
  const latestScanJob = libraryJobs.find(isLibraryScanJob) ?? null;
  const failedJobCount = libraryJobs.filter(
    (job) => job.has_error || job.status === "failed",
  ).length;
  const totalSizeBytes = response.sources.reduce<number | null>((total, row) => {
    if (typeof row.source.size_bytes !== "number") {
      return total;
    }

    return (total ?? 0) + row.source.size_bytes;
  }, null);

  return {
    source,
    error,
    sourceCount: response.page.returned,
    returnedSourceCount: response.sources.length,
    linkedItemCount: response.sources.filter((row) => row.item).length,
    probedSourceCount: response.sources.filter((row) => row.probe).length,
    totalSizeBytes,
    latestScanJob: latestScanJob ? mapLibraryJob(latestScanJob) : null,
    failedJobCount,
    page: response.page,
    samples: response.sources.slice(0, 4).map((row) => ({
      id: row.source.id,
      fileName: row.source.file_name,
      itemTitle: row.item?.title ?? null,
      sizeBytes: row.source.size_bytes,
      hasProbe: Boolean(row.probe),
    })),
  };
}

function isLibraryScanJob(job: AdminJobListResponse["jobs"][number]) {
  return job.kind === "library_scan" || job.resource_class === "disk.scan";
}

async function enqueueLibraryCommand(
  client: AdminApiClient,
  libraryId: string,
  action: LibraryCommandAction,
): Promise<AdminJobCommandResponse> {
  if (action === "scan") {
    return client.enqueueLibraryScan(libraryId);
  }

  if (action === "nfoImport") {
    return client.enqueueLibraryNfoImport(libraryId);
  }

  return client.enqueueLibraryNfoExport(libraryId);
}

function mapLibraryJob(job: AdminJobCommandResponse | AdminJobListResponse["jobs"][number]): LibraryJobSummary {
  return {
    id: job.id,
    kind: job.kind,
    status: job.status,
    resourceClass: job.resource_class,
    queuedAt: job.queued_at,
    completedAt: job.completed_at,
    hasError: job.has_error,
  };
}

function previewAddonManifestJson(manifestJson: string): AddonManifestPreview {
  try {
    const manifest = JSON.parse(manifestJson);

    return {
      status: "ready",
      manifest,
      summary: {
        manifestId: stringField(manifest.id),
        name: stringField(manifest.name),
        version: stringField(manifest.version),
        protocolVersion: stringField(manifest.protocol_version),
        baseUrl: stringField(manifest.base_url),
        resourceCount: Array.isArray(manifest.resources) ? manifest.resources.length : 0,
        declaredScopes: Array.isArray(manifest.scopes) ? manifest.scopes.map(String) : [],
        secretReferenceCount: Array.isArray(manifest.secret_reference_fields)
          ? manifest.secret_reference_fields.length
          : 0,
      },
    };
  } catch {
    return {
      status: "invalid_json",
      error: "Manifest JSON could not be parsed.",
    };
  }
}

function mapAddonOnboardingResult(
  response: AdminAddonRegistrationResponse,
): AddonOnboardingResult {
  const { summary, manifest } = response.addon;

  return {
    status: "registered",
    addon: {
      id: summary.id,
      manifestId: summary.manifest_id,
      name: summary.name,
      version: summary.version,
      protocolVersion: summary.protocol_version,
      baseUrl: summary.base_url,
      status: summary.status,
      resourceCount: manifest.resources.length,
      grantedScopes: summary.granted_scopes,
    },
    nextSteps: [
      "Open the generated Addon Install Guide",
      "Start the Addon Sidecar outside Nako",
      "Run Addon Health Check before enabling",
    ],
  };
}

function stringField(value: unknown) {
  return typeof value === "string" ? value : "";
}

async function loadAddonsRouteSummary(
  client: AdminApiClient,
  query: AdminAddonsQuery,
): Promise<AdminSectionResult<AddonsRouteSummary>> {
  const registrations = await loadSection(() => client.getAddons(query), mockAddonsForQuery(query));
  const selectedSummary = registrations.value.addons[0] ?? null;

  if (!selectedSummary) {
    return {
      value: mapAddonsRouteSummary(registrations.value, null, null, null, null, null, null),
      source: registrations.source,
      error: registrations.error,
    };
  }

  const fallback = fallbackAddonReadModels(selectedSummary);
  const [detail, health, surfaces, installGuide, tokens, grants] = await Promise.all([
    loadSection(() => client.getAddonDetail(selectedSummary.id), fallback.detail),
    loadSection(() => client.checkAddonHealth(selectedSummary.id), fallback.health),
    loadSection(() => client.getAddonSurfaces(selectedSummary.id), fallback.surfaces),
    loadSection(() => client.getAddonInstallGuide(selectedSummary.id), fallback.installGuide),
    loadSection(() => client.getAddonTokens(selectedSummary.id), fallback.tokens),
    loadSection(() => client.getAddonGrants(selectedSummary.id), fallback.grants),
  ]);
  const sections = [registrations, detail, health, surfaces, installGuide, tokens, grants];

  return {
    value: mapAddonsRouteSummary(
      registrations.value,
      detail.value,
      health.value,
      surfaces.value,
      installGuide.value,
      tokens.value,
      grants.value,
    ),
    source: combineLoadSources(sections.map((section) => section.source)),
    error: combineLoadErrors(sections),
  };
}

function mockAddonsForQuery(query: AdminAddonsQuery): AdminAddonRegistrationsResponse {
  if (!query.status) {
    return mockAddons;
  }

  return {
    addons: mockAddons.addons.filter((addon) => addon.status === query.status),
  };
}

function fallbackAddonReadModels(summary: AdminAddonRegistrationSummary) {
  if (summary.id === mockAddonDetail.addon.summary.id) {
    return {
      detail: mockAddonDetail,
      health: mockAddonHealth,
      surfaces: mockAddonSurfaces,
      installGuide: mockAddonInstallGuide,
      tokens: mockAddonTokens,
      grants: mockAddonGrants,
    };
  }

  return {
    detail: fallbackAddonDetail(summary),
    health: fallbackAddonHealth(summary),
    surfaces: emptyAddonSurfaces(summary),
    installGuide: fallbackAddonInstallGuide(summary),
    tokens: { tokens: [] },
    grants: { grants: [] },
  };
}

function fallbackAddonDetail(summary: AdminAddonRegistrationSummary): AdminAddonRegistrationResponse {
  return {
    addon: {
      summary,
      manifest: {
        id: summary.manifest_id,
        name: summary.name,
        version: summary.version,
        protocol_version: summary.protocol_version,
        base_url: summary.base_url,
        description: null,
        resources: [],
        auth: "none",
        default_timeout_ms: null,
        default_max_attempts: null,
        scopes: [],
      },
    },
  };
}

function fallbackAddonHealth(summary: AdminAddonRegistrationSummary): AdminAddonHealthCheckResponse {
  return {
    addon_id: summary.id,
    manifest_id: summary.manifest_id,
    status: "unreachable",
    latency_ms: 0,
    protocol_version: summary.protocol_version,
    addon_version: summary.version,
    safe_error_code: "route_fallback",
  };
}

function emptyAddonSurfaces(summary: AdminAddonRegistrationSummary): AdminAddonSurfacesResponse {
  return {
    addon_id: summary.id,
    manifest_id: summary.manifest_id,
    entry_points: [],
    hosted_pages: [],
    secret_reference_fields: [],
    tasks: [],
    event_subscriptions: [],
  };
}

function fallbackAddonInstallGuide(
  summary: AdminAddonRegistrationSummary,
): AdminAddonInstallGuideResponse {
  const emptySnippet = {
    title: "Unavailable",
    filename: "",
    content: "",
    notes: [],
  };

  return {
    addon_id: summary.id,
    manifest_id: summary.manifest_id,
    addon_name: summary.name,
    addon_version: summary.version,
    protocol_version: summary.protocol_version,
    base_url: summary.base_url,
    status: summary.status,
    docker_compose: emptySnippet,
    systemd: emptySnippet,
    secret_references: [],
    health_check_steps: [],
    registration_verification_steps: [],
    lifecycle_boundary: {
      nako_manages_containers: false,
      nako_manages_processes: false,
      nako_manages_packages: false,
      message:
        "Install guide unavailable. Nako does not manage Addon Sidecar lifecycle from this route.",
    },
  };
}

function mapAddonsRouteSummary(
  registrations: AdminAddonRegistrationsResponse,
  detail: AdminAddonRegistrationResponse | null,
  health: AdminAddonHealthCheckResponse | null,
  surfaces: AdminAddonSurfacesResponse | null,
  installGuide: AdminAddonInstallGuideResponse | null,
  tokens: AddonTokensResponse | null,
  grants: AddonGrantsResponse | null,
): AddonsRouteSummary {
  const selectedAddon = detail?.addon ?? null;

  return {
    addons: registrations.addons.map(mapAddonsRouteRow),
    selectedAddon: selectedAddon
      ? {
          ...mapAddonsRouteRow(selectedAddon.summary),
          resourceCount: selectedAddon.manifest.resources.length,
          resourceKinds: selectedAddon.manifest.resources.map((resource) => resource.kind),
          authMode: selectedAddon.manifest.auth,
          grantedScopes: selectedAddon.summary.granted_scopes,
          defaultTimeoutMs: selectedAddon.manifest.default_timeout_ms,
          defaultMaxAttempts: selectedAddon.manifest.default_max_attempts,
        }
      : null,
    statusCounts: countAddonStatuses(registrations.addons),
    health: health ? mapAddonHealth(health) : null,
    surfaceSummary: surfaces
      ? {
          entryPointCount: surfaces.entry_points.length,
          hostedPageCount: surfaces.hosted_pages.length,
          configurationSchemaDeclared: Boolean(surfaces.configuration_schema),
          secretReferenceFieldCount: surfaces.secret_reference_fields.length,
          taskCount: surfaces.tasks.length,
          eventSubscriptionCount: surfaces.event_subscriptions.length,
        }
      : null,
    installBoundary: installGuide
      ? {
          nakoManagesContainers: installGuide.lifecycle_boundary.nako_manages_containers,
          nakoManagesProcesses: installGuide.lifecycle_boundary.nako_manages_processes,
          nakoManagesPackages: installGuide.lifecycle_boundary.nako_manages_packages,
          secretReferenceCount: installGuide.secret_references.length,
          healthCheckStepCount: installGuide.health_check_steps.length,
          registrationVerificationStepCount:
            installGuide.registration_verification_steps.length,
        }
      : null,
    tokens: tokens?.tokens.map(mapAddonToken) ?? [],
    grants: grants?.grants.map(mapAddonGrant) ?? [],
  };
}

function mapAddonsRouteRow(addon: AdminAddonRegistrationSummary) {
  return {
    id: addon.id,
    manifestId: addon.manifest_id,
    name: addon.name,
    version: addon.version,
    protocolVersion: addon.protocol_version,
    status: addon.status,
    grantedScopeCount: addon.granted_scopes.length,
    updatedAt: addon.updated_at,
  };
}

function countAddonStatuses(addons: AdminAddonRegistrationSummary[]) {
  const counts = {
    enabled: 0,
    disabled: 0,
    unregistered: 0,
  };

  for (const addon of addons) {
    counts[addon.status] += 1;
  }

  return counts;
}

function mapAddons(
  registrations: AdminAddonRegistrationsResponse,
  detail: AdminAddonRegistrationResponse,
  health: AdminAddonHealthCheckResponse,
  surfaces: AdminAddonSurfacesResponse,
  installGuide: AdminAddonInstallGuideResponse,
  tokens: AddonTokensResponse,
  grants: AddonGrantsResponse,
  diagnostic: AdminAddonResourceCallDiagnosticResponse,
): AddonOperationsSummary {
  const addons = registrations.addons.map((addon) => ({
    id: addon.id,
    manifestId: addon.manifest_id,
    name: addon.name,
    version: addon.version,
    protocolVersion: addon.protocol_version,
    baseUrl: addon.base_url,
    status: addon.status,
    grantedScopes: addon.granted_scopes,
    updatedAt: addon.updated_at,
  }));
  const selectedAddon = detail.addon;

  return {
    selectedAddonId: selectedAddon.summary.id,
    addons,
    selectedAddon: {
      id: selectedAddon.summary.id,
      manifestId: selectedAddon.summary.manifest_id,
      name: selectedAddon.summary.name,
      version: selectedAddon.summary.version,
      protocolVersion: selectedAddon.summary.protocol_version,
      baseUrl: selectedAddon.summary.base_url,
      status: selectedAddon.summary.status,
      grantedScopes: selectedAddon.summary.granted_scopes,
      updatedAt: selectedAddon.summary.updated_at,
      description: selectedAddon.manifest.description,
      resourceCount: selectedAddon.manifest.resources.length,
      resourceKinds: selectedAddon.manifest.resources.map((resource) => resource.kind),
      authMode: selectedAddon.manifest.auth,
      defaultTimeoutMs: selectedAddon.manifest.default_timeout_ms,
      defaultMaxAttempts: selectedAddon.manifest.default_max_attempts,
    },
    health: mapAddonHealth(health),
    surfaces: {
      entryPoints: surfaces.entry_points.map((entryPoint) => ({
        id: entryPoint.id,
        label: entryPoint.label,
        kind: entryPoint.kind,
        path: entryPoint.path,
        hostedPageId: entryPoint.hosted_page_id ?? null,
      })),
      hostedPages: surfaces.hosted_pages.map((page) => ({
        id: page.id,
        title: page.title,
        path: page.path,
        url: page.url,
      })),
      configurationSchemaId: surfaces.configuration_schema?.schema_id ?? null,
      secretReferenceFieldCount: surfaces.secret_reference_fields.length,
      tasks: surfaces.tasks.map((task) => ({
        id: task.id,
        name: task.name,
        path: task.path,
      })),
      eventSubscriptions: surfaces.event_subscriptions.map((subscription) => ({
        id: subscription.id,
        eventKind: subscription.event_kind,
        path: subscription.path,
      })),
    },
    installGuide: mapAddonInstallGuide(installGuide),
    tokens: tokens.tokens.map(mapAddonToken),
    grants: grants.grants.map(mapAddonGrant),
    diagnostic: mapAddonDiagnostic(diagnostic),
  };
}

function mapAddonToken(token: AddonTokensResponse["tokens"][number]): AddonTokenSummaryRow {
  return {
    id: token.id,
    label: token.label,
    tokenPrefix: token.token_prefix,
    status: token.status,
    lastUsedAt: token.last_used_at,
  };
}

function mapAddonGrant(grant: AddonGrantsResponse["grants"][number]): AddonGrantSummaryRow {
  return {
    id: grant.id,
    permission: grant.permission,
    libraryId: grant.library_id,
  };
}

function mapAddonInstallGuide(response: AdminAddonInstallGuideResponse): AddonInstallGuideSummary {
  return {
    addonId: response.addon_id,
    manifestId: response.manifest_id,
    addonName: response.addon_name,
    addonVersion: response.addon_version,
    protocolVersion: response.protocol_version,
    baseUrl: response.base_url,
    status: response.status,
    dockerCompose: mapAddonInstallGuideSnippet(response.docker_compose),
    systemd: mapAddonInstallGuideSnippet(response.systemd),
    secretReferences: response.secret_references.map((secret) => ({
      id: secret.id,
      label: secret.label,
      description: secret.description ?? null,
      required: secret.required,
      envVar: secret.env_var,
      placeholder: secret.placeholder,
    })),
    healthCheckSteps: response.health_check_steps.map(mapAddonInstallGuideStep),
    registrationVerificationSteps: response.registration_verification_steps.map(mapAddonInstallGuideStep),
    lifecycleBoundary: {
      nakoManagesContainers: response.lifecycle_boundary.nako_manages_containers,
      nakoManagesProcesses: response.lifecycle_boundary.nako_manages_processes,
      nakoManagesPackages: response.lifecycle_boundary.nako_manages_packages,
      message: response.lifecycle_boundary.message,
    },
  };
}

function mapAddonInstallGuideSnippet(
  snippet: AdminAddonInstallGuideResponse["docker_compose"],
) {
  return {
    title: snippet.title,
    filename: snippet.filename,
    content: snippet.content,
    notes: snippet.notes,
  };
}

function mapAddonInstallGuideStep(step: AdminAddonInstallGuideResponse["health_check_steps"][number]) {
  return {
    title: step.title,
    command: step.command,
    expectedResult: step.expected_result,
  };
}

function mapAddonHealth(response: AdminAddonHealthCheckResponse): AddonHealthSummary {
  return {
    addonId: response.addon_id,
    status: response.status,
    latencyMs: response.latency_ms,
    protocolVersion: response.protocol_version ?? null,
    addonVersion: response.addon_version ?? null,
    resourceCount: response.resource_count ?? null,
    safeErrorCode: response.safe_error_code ?? null,
  };
}

function mapAddonDiagnostic(
  response: AdminAddonResourceCallDiagnosticResponse,
): AddonDiagnosticSummary {
  return {
    addonId: response.addon_id,
    resource: response.resource,
    status: response.status,
    latencyMs: response.latency_ms,
    attempts: response.attempts,
    httpStatus: response.http_status ?? null,
    safeErrorCode: response.safe_error_code ?? null,
  };
}

async function loadSection<T>(loader: () => Promise<T>, fallback: T): Promise<LoadResult<T>> {
  try {
    return {
      value: await loader(),
      source: "live",
    };
  } catch (error: unknown) {
    return {
      value: fallback,
      source: "mock",
      error: error instanceof Error ? error.message : "Admin API request failed",
    };
  }
}

function recordError<T>(errors: AdminErrorMap, section: AdminSectionKey, result: LoadResult<T>) {
  if (result.error) {
    errors[section] = result.error;
  }
}

function combineLoadSources(sources: DataSourceMode[]): DataSourceMode {
  if (sources.length === 0) {
    return "mock";
  }

  if (sources.every((source) => source === "live")) {
    return "live";
  }

  if (sources.some((source) => source === "live")) {
    return "hybrid";
  }

  return "mock";
}

function combineLoadErrors(results: Array<{ error?: string }>) {
  const messages = Array.from(
    new Set(results.map((result) => result.error).filter((message): message is string => Boolean(message))),
  );

  return messages.length > 0 ? messages.join("; ") : undefined;
}

function mapCatalogGovernance(
  response: AdminCatalogGovernanceItemListResponse,
): CatalogGovernanceSummary {
  return {
    items: response.items.map(mapCatalogGovernanceItem),
    page: response.page,
  };
}

function mapCatalogGovernanceItemDetail(
  response: AdminCatalogGovernanceItemDetailResponse,
): CatalogGovernanceItemDetailSummary {
  return {
    item: mapCatalogGovernanceItem(response.item),
    providerMappings: response.provider_mappings.map(mapCatalogGovernanceProviderMapping),
    repairActions: response.repair_actions,
  };
}

function mapCatalogGovernanceItem(
  item: AdminCatalogGovernanceItem,
): CatalogGovernanceItemSummary {
  return {
    id: item.item_id,
    libraryId: item.library_id,
    kind: item.kind,
    parentId: item.parent_id,
    title: item.title,
    releaseDate: item.release_date,
    issues: item.issues,
    sourceCount: item.source_count,
    representativeSourceId: item.representative_source_id,
    representativeFileName: item.representative_file_name,
    providerMappingCount: item.provider_mapping_count,
    acceptedProviderMappingCount: item.accepted_provider_mapping_count,
    duplicateRelationshipCount: item.duplicate_relationship_count,
    localInference: item.local_inference
      ? {
          sourceId: item.local_inference.source_id,
          inferredKind: item.local_inference.inferred_kind,
          inferredTitle: item.local_inference.inferred_title,
          inferredYear: item.local_inference.inferred_year,
          inferredSeason: item.local_inference.inferred_season,
          inferredEpisode: item.local_inference.inferred_episode,
          confidenceMilli: item.local_inference.confidence_milli,
          evidenceSource: item.local_inference.evidence_source,
          hasEvidence: item.local_inference.has_evidence,
          inferenceVersion: item.local_inference.inference_version,
        }
      : null,
  };
}

function mapCatalogGovernanceProviderMapping(
  mapping: AdminCatalogGovernanceProviderMappingSummary,
): CatalogGovernanceProviderMappingSummary {
  return {
    id: mapping.mapping_id,
    itemId: mapping.item_id,
    status: mapping.status,
    confidenceMilli: mapping.confidence_milli,
    source: formatMetadataSource(mapping.source),
    subject: {
      id: mapping.subject.subject_id,
      provider: formatExternalProvider(mapping.subject.provider),
      kind: formatProviderSubjectKind(mapping.subject.subject_kind),
      key: mapping.subject.subject_key,
      title: mapping.subject.title,
      releaseYear: mapping.subject.release_year,
      locale: mapping.subject.locale,
    },
  };
}

function mapCatalogGovernanceProviderMappingReviewPlan(
  response: AdminCatalogGovernanceProviderMappingReviewPlanResponse,
): CatalogGovernanceProviderMappingReviewPlanSummary {
  const { plan } = response;

  return {
    item: mapCatalogGovernanceItem(plan.item),
    mapping: mapCatalogGovernanceProviderMapping(plan.mapping),
    decision: plan.decision,
    currentStatus: plan.current_status,
    targetStatus: plan.target_status,
    status: plan.status,
    readiness: {
      status: plan.readiness.status,
      actionable: plan.readiness.actionable,
      reasons: plan.readiness.reasons,
    },
    boundary: {
      updatesProviderMappingStatus: plan.boundary.updates_provider_mapping_status,
      updatesCanonicalMetadata: plan.boundary.updates_canonical_metadata,
      updatesProviderSubject: plan.boundary.updates_provider_subject,
      updatesLocalInference: plan.boundary.updates_local_inference,
      updatesSourceDuplicates: plan.boundary.updates_source_duplicates,
      updatesHierarchy: plan.boundary.updates_hierarchy,
      writesNfo: plan.boundary.writes_nfo,
      writesLibraryFiles: plan.boundary.writes_library_files,
      updatesArtwork: plan.boundary.updates_artwork,
      updatesPlaybackState: plan.boundary.updates_playback_state,
    },
  };
}

function mapCatalogGovernanceProviderMappingReviewResult(
  response: AdminCatalogGovernanceProviderMappingReviewResponse,
): CatalogGovernanceProviderMappingReviewResultSummary {
  return {
    itemId: response.item_id,
    mappingId: response.mapping_id,
    decision: response.decision,
    previousStatus: response.previous_status,
    currentStatus: response.current_status,
    changed: response.changed,
    idempotentReplay: response.idempotent_replay,
    plan: mapCatalogGovernanceProviderMappingReviewPlan({
      admin_api_version: response.admin_api_version,
      public_api_version: response.public_api_version,
      plan: response.plan,
    }),
  };
}

function formatMetadataSource(source: AdminMetadataSource): string {
  if (typeof source === "string") {
    return source;
  }

  if ("provider" in source) {
    return `provider:${formatExternalProvider(source.provider)}`;
  }

  return `addon:${source.addon}`;
}

function formatExternalProvider(provider: AdminExternalProvider): string {
  return typeof provider === "string" ? provider : provider.other;
}

function formatProviderSubjectKind(
  kind: AdminCatalogGovernanceProviderMappingSummary["subject"]["subject_kind"],
): string {
  return typeof kind === "string" ? kind : kind.other;
}

function mapAcquisitionIntake(
  response: AdminAcquisitionIntakeCandidateListResponse,
): IntakeSummary {
  return {
    candidates: response.candidates.map((candidate) => ({
      id: candidate.id,
      sourceKind: candidate.source_kind,
      sourceScheme: candidate.source_scheme ?? "unknown",
      state: candidate.state,
      sizeBytes: candidate.size_bytes,
      hasDiagnostics: candidate.has_diagnostics,
      linkedArtifactId: candidate.managed_import_artifact_id,
    })),
    page: response.page,
  };
}

function mapGeneratedArtifactProposals(
  response: AdminGeneratedArtifactProposalListResponse,
): GeneratedArtifactProposalSummary {
  return {
    proposals: response.proposals.map((proposal) => ({
      id: proposal.id,
      capability: proposal.capability,
      kind: proposal.kind,
      status: proposal.status,
      targetKind: proposal.target.kind,
      readinessStatus: proposal.readiness.status,
      actionable: proposal.readiness.actionable,
      confidenceMilli: proposal.payload.confidence_milli,
      payloadShape: proposal.payload.shape,
      providerName: proposal.provenance.provider_name,
      promptFingerprint: proposal.provenance.prompt_fingerprint,
      payloadFingerprint: proposal.payload.payload_fingerprint,
    })),
    page: response.page,
  };
}

function mapGeneratedArtifactReviewPlan(
  response: AdminGeneratedArtifactReviewPlanResponse,
): GeneratedArtifactReviewPlanSummary {
  const { plan } = response;

  return {
    artifactId: plan.artifact_id,
    decision: plan.decision,
    status: plan.status,
    action: plan.action,
    reasons: plan.reasons,
    capability: plan.capability,
    kind: plan.kind,
    target: {
      kind: plan.target.kind,
      libraryId: plan.target.library_id,
      itemId: plan.target.item_id,
      sourceId: plan.target.source_id,
    },
    payload: {
      validJson: plan.payload.valid_json,
      shape: plan.payload.shape,
      payloadFingerprint: plan.payload.payload_fingerprint,
      payloadBytes: plan.payload.payload_bytes,
      objectFieldCount: plan.payload.object_field_count,
      arrayItemCount: plan.payload.array_item_count,
      hasTextualValues: plan.payload.has_textual_values,
      hasExplanation: plan.payload.has_explanation,
      confidenceMilli: plan.payload.confidence_milli,
    },
    readiness: {
      status: plan.readiness.status,
      actionable: plan.readiness.actionable,
      reasons: plan.readiness.reasons,
    },
    boundary: {
      acceptedIntoCanonicalMetadata: plan.boundary.accepted_into_canonical_metadata,
      writesSidecar: plan.boundary.writes_sidecar,
      writesLibraryFiles: plan.boundary.writes_library_files,
      appliesImmediately: plan.boundary.applies_immediately,
      requiresMetadataAuthorityApply: plan.boundary.requires_metadata_authority_apply,
    },
  };
}

function mapGeneratedArtifactReviewResult(
  response: AdminGeneratedArtifactReviewResponse,
): GeneratedArtifactReviewResultSummary {
  return {
    artifactId: response.artifact_id,
    decision: response.decision,
    artifactStatus: response.artifact_status,
    acceptedAt: response.accepted_at,
    idempotentReplay: response.idempotent_replay,
    plan: mapGeneratedArtifactReviewPlan({
      admin_api_version: response.admin_api_version,
      public_api_version: response.public_api_version,
      plan: response.plan,
    }),
  };
}

function mapEvents(response: AdminOutboxEventListResponse): EventSummary {
  return {
    events: response.events.map((event) => ({
      id: event.id,
      kind: event.kind,
      status: event.status,
      attempts: event.attempts,
      hasError: event.has_error,
    })),
    page: response.page,
  };
}

function mapJobs(response: AdminJobListResponse): JobRow[] {
  return response.jobs.map((job) => ({
    id: job.id,
    kind: job.kind,
    status: job.status,
    resourceClass: job.resource_class,
    hasError: job.has_error,
  }));
}

function mapPlayback(
  sessions: AdminPlaybackSessionListResponse,
  runtime: AdminPlaybackRuntimeDiagnosticsResponse,
): PlaybackSummary {
  return {
    hardwarePolicy: hardwarePolicyLabel(runtime),
    ffmpegStatus: runtime.ffmpeg.probe_status,
    accelerators: runtime.hardware.capabilities.map((capability) => ({
      name: capability.accelerator,
      available: capability.available,
    })),
    sessions: sessions.sessions.map((session) => ({
      id: session.id,
      kind: session.mode,
      sourceTitle: session.source_id,
      state: session.state,
    })),
  };
}

function mapStorage(response: AdminStorageStagingDiagnosticsResponse): StorageSummary {
  return {
    stagingUsedBytes: response.summary.used_manifest_bytes,
    stagingMaxBytes: response.summary.configured_max_bytes,
    vfsObjectCount: response.summary.vfs_cache.object_count,
    records: response.records.map((record) => ({
      id: record.id,
      sourceScheme: record.source_scheme,
      purpose: record.purpose,
      state: record.state,
      sizeBytes: record.size_bytes,
      hasValidationError: record.has_validation_error,
    })),
  };
}

function mapNetwork(response: AdminServerConfigDiagnosticsResponse["network"]): NetworkSummary {
  return {
    exposureMode: response.exposure_mode,
    readinessStatus: response.readiness.status,
    readinessReason: response.readiness.reason,
    endpointConfigured: response.external_endpoint.configured,
    endpointScheme: response.external_endpoint.scheme,
    trustedProxyHeaders: response.trusted_proxy.headers_enabled,
    trustedProxySourceCount: response.trusted_proxy.source_count,
    allowedOriginCount: response.origins.allowed_origin_count,
    tunnelProviderCount: response.tunnel_providers.length,
  };
}

function mapSettings(response: AdminServerConfigDiagnosticsResponse): SettingRow[] {
  return [
    {
      label: "Admin auth",
      value: response.auth.enabled ? "Auth configured" : "Auth disabled",
    },
    {
      label: "Network readiness",
      value: `${response.network.exposure_mode} · ${response.network.readiness.status}`,
    },
    {
      label: "FFmpeg",
      value: "Runtime diagnostics enabled",
    },
    {
      label: "Transcode policy",
      value: `${response.transcode.gpu_concurrency} GPU slot`,
    },
    {
      label: "Settings edits",
      value: "Planned Admin API",
    },
  ];
}

function hardwarePolicyLabel(runtime: AdminPlaybackRuntimeDiagnosticsResponse) {
  const requested =
    typeof runtime.hardware.policy.requested === "string"
      ? runtime.hardware.policy.requested.toUpperCase()
      : "configured hardware";

  if (runtime.hardware.selection.fallback_used) {
    return `${requested} requested, fallback active`;
  }

  return `${requested} selected`;
}
