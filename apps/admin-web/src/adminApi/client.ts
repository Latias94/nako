import type {
  AddonGrantsResponse,
  AddonEventDeliveryAttemptsResponse,
  AddonEventDispatchResponse,
  AddonEventReplayResponse,
  AddonEventSchedulerWorkResponse,
  AddonTaskRunResponse,
  AddonTaskRunsQuery,
  AddonTaskRunsResponse,
  AddonTokenIssuedResponse,
  AddonTokenResponse,
  AddonTokenRotationResponse,
  AddonTokensResponse,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAcquisitionIntakeCandidatesQuery,
  AcceptManagedArtworkCandidateResponse,
  AdminAddonHealthCheckResponse,
  AdminAddonInstallGuidePreviewRequest,
  AdminAddonInstallGuidePreviewResponse,
  AdminAddonInstallGuideResponse,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticRequest,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAccessSummaryResponse,
  AdminCreateInvitationRequest,
  AdminCreateInvitationResponse,
  AdminInvitationListResponse,
  AdminInvitationResponse,
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
  AdminIncidentBundleResponse,
  AdminOperatorReadinessResponse,
  AdminArtworkKind,
  AdminItemArtworkGalleryQuery,
  AdminManagedArtworkArtifactCleanupQuery,
  AdminManagedArtworkArtifactCleanupResponse,
  AdminManagedArtworkArtifactLifecycleQuery,
  AdminManagedArtworkArtifactLifecycleResponse,
  AdminManagedArtworkArtifactRemediationPlanQuery,
  AdminManagedArtworkArtifactRemediationPlanResponse,
  AdminManagedArtworkArtifactStrayFileCleanupQuery,
  AdminManagedArtworkArtifactStrayFileCleanupResponse,
  AdminManagedArtworkArtifactStorageDriftQuery,
  AdminManagedArtworkArtifactStorageDriftResponse,
  AdminManagedArtworkGalleryResponse,
  AdminSelectItemArtworkRequest,
  AdminJobCancelRequestResponse,
  AdminJobCommandResponse,
  AdminJobListItem,
  AdminLibraryMetadataProfileResponse,
  AdminMetadataRawCacheSettingsResponse,
  AdminMetadataProfile,
  AdminUpdateMetadataRawCacheSettingsRequest,
  AdminWatchFolderDiscoveryRequest,
  AdminWatchFolderDiscoveryResponse,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminPageQuery,
  AdminOutboxEventListResponse,
  AdminOutboxEventsQuery,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackRuntimeSettingsResponse,
  AdminPlaybackSessionsQuery,
  AdminPlaybackSessionListResponse,
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
  AdminServerConfigDiagnosticsResponse,
  AdminSourceDuplicateReconciliationApplyRequest,
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationPlanQuery,
  AdminSourceDuplicateReconciliationPlanResponse,
  AdminStorageStagingQuery,
  AdminStorageStagingDiagnosticsResponse,
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
  RetryAddonTaskRunRequest,
  ReplayAddonEventRequest,
  RequeueManagedArtworkIngestResponse,
  IssueAddonTokenRequest,
  RegisterAddonRequest,
  ReplaceAddonGrantsRequest,
  AdminUpdateLibraryMetadataProfileRequest,
  PublishSelectedArtworkResponse,
  AdminUpdatePlaybackRuntimeSettingsRequest,
  ProcessManagedArtworkIngestResponse,
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

  async getOperatorReadiness(): Promise<AdminOperatorReadinessResponse> {
    return this.getJson<AdminOperatorReadinessResponse>(
      NAKO_ADMIN_ROUTES.operatorReadiness,
    );
  }

  async getIncidentBundle(): Promise<AdminIncidentBundleResponse> {
    return this.getJson<AdminIncidentBundleResponse>(NAKO_ADMIN_ROUTES.incidentBundle);
  }

  async getAccessSummary(): Promise<AdminAccessSummaryResponse> {
    return this.getJson<AdminAccessSummaryResponse>(NAKO_ADMIN_ROUTES.accessSummary);
  }

  async getAccessInvitations(
    query: AdminPageQuery = {},
  ): Promise<AdminInvitationListResponse> {
    return this.getJson<AdminInvitationListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.accessInvitations, query),
    );
  }

  async createAccessInvitation(
    request: AdminCreateInvitationRequest,
  ): Promise<AdminCreateInvitationResponse> {
    return this.postJson<AdminCreateInvitationResponse>(
      NAKO_ADMIN_ROUTES.accessInvitations,
      request,
    );
  }

  async revokeAccessInvitation(invitationId: string): Promise<AdminInvitationResponse> {
    return this.postJson<AdminInvitationResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.accessInvitationRevoke,
        "invitation_id",
        invitationId,
      ),
      {},
    );
  }

  async getAddons(query: AdminAddonsQuery = {}): Promise<AdminAddonRegistrationsResponse> {
    return this.getJson<AdminAddonRegistrationsResponse>(withQuery(NAKO_ADMIN_ROUTES.addons, query));
  }

  async getAddonDetail(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.getJson<AdminAddonRegistrationResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonDetail, "addon_id", addonId),
    );
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
      routeWithParam(NAKO_ADMIN_ROUTES.addonStatus, "addon_id", addonId),
      request,
    );
  }

  async unregisterAddon(addonId: string): Promise<AdminAddonRegistrationResponse> {
    return this.postJson<AdminAddonRegistrationResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonUnregister, "addon_id", addonId),
      {},
    );
  }

  async checkAddonHealth(addonId: string): Promise<AdminAddonHealthCheckResponse> {
    return this.postJson<AdminAddonHealthCheckResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonHealthCheck, "addon_id", addonId),
      {},
    );
  }

  async getAddonSurfaces(addonId: string): Promise<AdminAddonSurfacesResponse> {
    return this.getJson<AdminAddonSurfacesResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonSurfaces, "addon_id", addonId),
    );
  }

  async getAddonInstallGuide(addonId: string): Promise<AdminAddonInstallGuideResponse> {
    return this.getJson<AdminAddonInstallGuideResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonInstallGuide, "addon_id", addonId),
    );
  }

  async previewAddonInstallGuide(
    request: AdminAddonInstallGuidePreviewRequest,
  ): Promise<AdminAddonInstallGuidePreviewResponse> {
    return this.postJson<AdminAddonInstallGuidePreviewResponse>(
      NAKO_ADMIN_ROUTES.addonInstallGuidePreview,
      request,
    );
  }

  async diagnoseAddonResourceCall(
    addonId: string,
    request: AdminAddonResourceCallDiagnosticRequest,
  ): Promise<AdminAddonResourceCallDiagnosticResponse> {
    return this.postJson<AdminAddonResourceCallDiagnosticResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic, "addon_id", addonId),
      request,
    );
  }

  async getAddonTokens(addonId: string): Promise<AddonTokensResponse> {
    return this.getJson<AddonTokensResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonTokens, "addon_id", addonId),
    );
  }

  async issueAddonToken(
    addonId: string,
    request: IssueAddonTokenRequest,
  ): Promise<AddonTokenIssuedResponse> {
    return this.postJson<AddonTokenIssuedResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonTokens, "addon_id", addonId),
      request,
    );
  }

  async rotateAddonToken(
    addonId: string,
    tokenId: string,
    request: IssueAddonTokenRequest,
  ): Promise<AddonTokenRotationResponse> {
    return this.postJson<AddonTokenRotationResponse>(
      routeWithParam(
        routeWithParam(NAKO_ADMIN_ROUTES.addonTokenRotate, "addon_id", addonId),
        "token_id",
        tokenId,
      ),
      request,
    );
  }

  async revokeAddonToken(addonId: string, tokenId: string): Promise<AddonTokenResponse> {
    return this.postJson<AddonTokenResponse>(
      routeWithParam(
        routeWithParam(NAKO_ADMIN_ROUTES.addonTokenRevoke, "addon_id", addonId),
        "token_id",
        tokenId,
      ),
      {},
    );
  }

  async getAddonGrants(addonId: string): Promise<AddonGrantsResponse> {
    return this.getJson<AddonGrantsResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonGrants, "addon_id", addonId),
    );
  }

  async getAddonTaskRuns(
    addonId: string,
    query: AddonTaskRunsQuery = {},
  ): Promise<AddonTaskRunsResponse> {
    return this.getJson<AddonTaskRunsResponse>(
      withQuery(
        routeWithParam(NAKO_ADMIN_ROUTES.addonTaskRuns, "addon_id", addonId),
        query,
      ),
    );
  }

  async getAddonTaskRun(addonId: string, jobId: string): Promise<AddonTaskRunResponse> {
    return this.getJson<AddonTaskRunResponse>(
      addonTaskRunPath(NAKO_ADMIN_ROUTES.addonTaskRun, addonId, jobId),
    );
  }

  async retryAddonTaskRun(
    addonId: string,
    jobId: string,
    request: RetryAddonTaskRunRequest,
  ): Promise<AddonTaskRunResponse> {
    return this.postJson<AddonTaskRunResponse>(
      addonTaskRunPath(NAKO_ADMIN_ROUTES.addonTaskRunRetry, addonId, jobId),
      request,
    );
  }

  async replaceAddonGrants(
    addonId: string,
    request: ReplaceAddonGrantsRequest,
  ): Promise<AddonGrantsResponse> {
    return this.putJson<AddonGrantsResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.addonGrants, "addon_id", addonId),
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

  async getManagedArtworkArtifactLifecycle(
    query: AdminManagedArtworkArtifactLifecycleQuery = {},
  ): Promise<AdminManagedArtworkArtifactLifecycleResponse> {
    return this.getJson<AdminManagedArtworkArtifactLifecycleResponse>(
      withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactLifecycle, query),
    );
  }

  async acceptManagedArtworkCandidate(
    candidateId: string,
  ): Promise<AcceptManagedArtworkCandidateResponse> {
    return this.postJson<AcceptManagedArtworkCandidateResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.managedArtworkCandidateAccept,
        "candidate_id",
        candidateId,
      ),
      {},
    );
  }

  async requeueManagedArtworkIngest(
    ingestId: string,
  ): Promise<RequeueManagedArtworkIngestResponse> {
    return this.postJson<RequeueManagedArtworkIngestResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.managedArtworkIngestRequeue,
        "ingest_id",
        ingestId,
      ),
      {},
    );
  }

  async processNextManagedArtworkIngest(): Promise<ProcessManagedArtworkIngestResponse> {
    return this.postJson<ProcessManagedArtworkIngestResponse>(
      NAKO_ADMIN_ROUTES.managedArtworkIngestProcessNext,
      {},
    );
  }

  async cleanupManagedArtworkArtifacts(
    query: AdminManagedArtworkArtifactCleanupQuery,
  ): Promise<AdminManagedArtworkArtifactCleanupResponse> {
    return this.postJson<AdminManagedArtworkArtifactCleanupResponse>(
      withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactCleanup, query),
      {},
    );
  }

  async getManagedArtworkArtifactStorageDrift(
    query: AdminManagedArtworkArtifactStorageDriftQuery = {},
  ): Promise<AdminManagedArtworkArtifactStorageDriftResponse> {
    return this.getJson<AdminManagedArtworkArtifactStorageDriftResponse>(
      withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactStorageDrift, query),
    );
  }

  async getManagedArtworkArtifactRemediationPlan(
    query: AdminManagedArtworkArtifactRemediationPlanQuery = {},
  ): Promise<AdminManagedArtworkArtifactRemediationPlanResponse> {
    return this.getJson<AdminManagedArtworkArtifactRemediationPlanResponse>(
      withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediationPlan, query),
    );
  }

  async remediateManagedArtworkArtifactStrayFiles(
    query: AdminManagedArtworkArtifactStrayFileCleanupQuery = {},
  ): Promise<AdminManagedArtworkArtifactStrayFileCleanupResponse> {
    return this.postJson<AdminManagedArtworkArtifactStrayFileCleanupResponse>(
      withQuery(NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediateStrayFiles, query),
      {},
    );
  }

  async publishManagedArtworkArtifact(artifactId: string): Promise<PublishSelectedArtworkResponse> {
    return this.postJson<PublishSelectedArtworkResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.managedArtworkArtifactPublish, "artifact_id", artifactId),
      {},
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

  async getEvents(query: AdminOutboxEventsQuery = {}): Promise<AdminOutboxEventListResponse> {
    return this.getJson<AdminOutboxEventListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.events, query),
    );
  }

  async getAddonEventDeliveryAttempts(
    eventId: string,
  ): Promise<AddonEventDeliveryAttemptsResponse> {
    return this.getJson<AddonEventDeliveryAttemptsResponse>(
      eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliveryAttempts, eventId),
    );
  }

  async getAddonEventSchedulerWork(
    eventId: string,
  ): Promise<AddonEventSchedulerWorkResponse> {
    return this.getJson<AddonEventSchedulerWorkResponse>(
      eventPath(NAKO_ADMIN_ROUTES.eventAddonSchedulerWork, eventId),
    );
  }

  async deliverAddonEvents(eventId: string): Promise<AddonEventDispatchResponse> {
    return this.postJson<AddonEventDispatchResponse>(
      eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliver, eventId),
      {},
    );
  }

  async replayAddonEvents(
    eventId: string,
    request: ReplayAddonEventRequest,
  ): Promise<AddonEventReplayResponse> {
    return this.postJson<AddonEventReplayResponse>(
      eventPath(NAKO_ADMIN_ROUTES.eventAddonReplay, eventId),
      request,
    );
  }

  async getJobs(query: AdminJobsQuery = {}): Promise<AdminJobListResponse> {
    return this.getJson<AdminJobListResponse>(withQuery(NAKO_ADMIN_ROUTES.jobs, query));
  }

  async cancelJob(jobId: string): Promise<AdminJobCancelRequestResponse> {
    return this.postJson<AdminJobCancelRequestResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.jobCancel, "job_id", jobId),
      {},
    );
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

  async getPlaybackRuntimeSettings(): Promise<AdminPlaybackRuntimeSettingsResponse> {
    return this.getJson<AdminPlaybackRuntimeSettingsResponse>(
      NAKO_ADMIN_ROUTES.settingsPlaybackRuntime,
    );
  }

  async updatePlaybackRuntimeSettings(
    request: AdminUpdatePlaybackRuntimeSettingsRequest,
  ): Promise<AdminPlaybackRuntimeSettingsResponse> {
    return this.putJson<AdminPlaybackRuntimeSettingsResponse>(
      NAKO_ADMIN_ROUTES.settingsPlaybackRuntime,
      request,
    );
  }

  async getPlaybackSupport(
    query: AdminPlaybackSupportQuery = {},
  ): Promise<AdminPlaybackSupportEvidenceResponse> {
    return this.getJson<AdminPlaybackSupportEvidenceResponse>(
      withQuery(NAKO_ADMIN_ROUTES.playbackSupport, query),
    );
  }

  async getSourceDuplicateReconciliationPlan(
    libraryId: string,
    sourceId: string,
    query: AdminSourceDuplicateReconciliationPlanQuery = {},
  ): Promise<AdminSourceDuplicateReconciliationPlanResponse> {
    return this.getJson<AdminSourceDuplicateReconciliationPlanResponse>(
      withQuery(sourceDuplicateReconciliationPath(
        NAKO_ADMIN_ROUTES.sourceDuplicateReconciliationPlan,
        libraryId,
        sourceId,
      ), query),
    );
  }

  async applySourceDuplicateReconciliation(
    libraryId: string,
    sourceId: string,
    request: AdminSourceDuplicateReconciliationApplyRequest,
  ): Promise<AdminSourceDuplicateReconciliationApplyResponse> {
    return this.postJson<AdminSourceDuplicateReconciliationApplyResponse>(
      sourceDuplicateReconciliationPath(
        NAKO_ADMIN_ROUTES.sourceDuplicateReconciliationApply,
        libraryId,
        sourceId,
      ),
      request,
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

  async getVfsCacheRepairActionPlan(): Promise<AdminVfsCacheRepairActionPlanResponse> {
    return this.getJson<AdminVfsCacheRepairActionPlanResponse>(
      NAKO_ADMIN_ROUTES.storageVfsCacheRepairActionPlan,
    );
  }

  async getVfsCacheRepairRemediationPlan(): Promise<AdminVfsCacheRepairRemediationPlanResponse> {
    return this.getJson<AdminVfsCacheRepairRemediationPlanResponse>(
      NAKO_ADMIN_ROUTES.storageVfsCacheRepairRemediationPlan,
    );
  }

  async planVfsCacheRepairAutomation(
    request: AdminVfsCacheRepairAutomationPolicyRequest,
  ): Promise<AdminVfsCacheRepairAutomationPlanResponse> {
    return this.postJson<AdminVfsCacheRepairAutomationPlanResponse>(
      NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationPlan,
      request,
    );
  }

  async enqueueVfsCacheRepairAutomation(
    request: AdminVfsCacheRepairAutomationEnqueueRequest,
  ): Promise<AdminVfsCacheRepairAutomationEnqueueResponse> {
    return this.postJson<AdminVfsCacheRepairAutomationEnqueueResponse>(
      NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationJobs,
      request,
    );
  }

  async getVfsCacheRepairTargets(
    query: { limit?: number; offset?: number } = {},
  ): Promise<AdminVfsCacheRepairTargetListResponse> {
    return this.getJson<AdminVfsCacheRepairTargetListResponse>(
      withQuery(NAKO_ADMIN_ROUTES.storageVfsCacheRepairTargets, query),
    );
  }

  async getVfsCacheRepairTargetPreview(
    targetRef: string,
  ): Promise<AdminVfsCacheRepairTargetPreviewResponse> {
    return this.getJson<AdminVfsCacheRepairTargetPreviewResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.storageVfsCacheRepairTargetPreview,
        "target_ref",
        targetRef,
      ),
    );
  }

  async refreshLatestVfsCacheRepair(): Promise<AdminVfsCacheRefreshResponse> {
    return this.postJson<AdminVfsCacheRefreshResponse>(
      NAKO_ADMIN_ROUTES.storageVfsCacheRepairRefreshCache,
      {},
    );
  }

  async refreshVfsCacheRepairTarget(
    targetRef: string,
  ): Promise<AdminVfsCacheRefreshResponse> {
    return this.postJson<AdminVfsCacheRefreshResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.storageVfsCacheRepairTargetRefreshCache,
        "target_ref",
        targetRef,
      ),
      {},
    );
  }

  async enqueueVfsCacheRepairTarget(
    targetRef: string,
    request: AdminVfsCacheRepairEnqueueRequest = {},
  ): Promise<AdminVfsCacheRepairEnqueueResponse> {
    return this.postJson<AdminVfsCacheRepairEnqueueResponse>(
      routeWithParam(
        NAKO_ADMIN_ROUTES.storageVfsCacheRepairTargetEnqueue,
        "target_ref",
        targetRef,
      ),
      request,
    );
  }

  async executeVfsCacheRepairJob(jobId: string): Promise<AdminVfsCacheRepairExecuteResponse> {
    return this.postJson<AdminVfsCacheRepairExecuteResponse>(
      routeWithParam(NAKO_ADMIN_ROUTES.storageVfsCacheRepairJobExecute, "job_id", jobId),
      {},
    );
  }

  async retryVfsCacheRepairJob(
    jobId: string,
    request: AdminVfsCacheRepairRetryRequest = {},
  ): Promise<AdminJobListItem> {
    return this.postJson<AdminJobListItem>(
      routeWithParam(NAKO_ADMIN_ROUTES.storageVfsCacheRepairJobRetry, "job_id", jobId),
      request,
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

function sourceDuplicateReconciliationPath(path: string, libraryId: string, sourceId: string) {
  return routeWithParam(
    routeWithParam(path, "library_id", libraryId),
    "source_id",
    sourceId,
  );
}

function addonTaskRunPath(path: string, addonId: string, jobId: string) {
  return routeWithParam(
    routeWithParam(path, "addon_id", addonId),
    "job_id",
    jobId,
  );
}

function eventPath(path: string, eventId: string) {
  return routeWithParam(path, "event_id", eventId);
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
