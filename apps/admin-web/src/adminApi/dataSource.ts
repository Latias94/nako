import { AdminApiClient, type AdminApiClientOptions } from "./client";
import {
  mockAdminConsoleData,
  mockAcquisitionIntakeCandidates,
  mockAddonDetail,
  mockAddonDiagnostic,
  mockAddonGrants,
  mockAddonHealth,
  mockAddonInstallGuide,
  mockAddons,
  mockAddonSurfaces,
  mockAddonTokens,
  mockCatalogGovernance,
  mockEvents,
  mockGeneratedArtifactProposals,
  mockJobs,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackSessions,
  mockStorageStaging,
  mockSystemConfig,
} from "./mockData";
import type {
  AddonGrantsResponse,
  AddonTokensResponse,
  AdminAcquisitionIntakeCandidateListResponse,
  AdminAddonHealthCheckResponse,
  AdminAddonInstallGuideResponse,
  AdminAddonRegistrationResponse,
  AdminAddonRegistrationsResponse,
  AdminAddonResourceCallDiagnosticResponse,
  AdminAddonSurfacesResponse,
  AdminCatalogGovernanceItemListResponse,
  AdminGeneratedArtifactProposalListResponse,
  AdminJobListResponse,
  AdminOutboxEventListResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListResponse,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
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
  AddonTokenActionResult,
  AddonTokenSummaryRow,
  AddonGrantSummaryRow,
  CatalogGovernanceSummary,
  DataSourceMode,
  EventSummary,
  GeneratedArtifactProposalSummary,
  IntakeSummary,
  JobRow,
  NetworkSummary,
  PlaybackSummary,
  SettingRow,
  StorageSummary,
} from "./types";

export type { AdminConsoleData, AdminSourceMap, DataSourceMode };

export type AdminDataSource = {
  load(): Promise<AdminConsoleData>;
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
      "Start the Addon Sidecar outside Taru",
      "Run Addon Health Check before enabling",
    ],
  };
}

function stringField(value: unknown) {
  return typeof value === "string" ? value : "";
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
      taruManagesContainers: response.lifecycle_boundary.taru_manages_containers,
      taruManagesProcesses: response.lifecycle_boundary.taru_manages_processes,
      taruManagesPackages: response.lifecycle_boundary.taru_manages_packages,
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

function mapCatalogGovernance(
  response: AdminCatalogGovernanceItemListResponse,
): CatalogGovernanceSummary {
  return {
    items: response.items.map((item) => ({
      id: item.item_id,
      title: item.title,
      kind: item.kind,
      issues: item.issues,
      sourceCount: item.source_count,
      providerMappingCount: item.provider_mapping_count,
    })),
    page: response.page,
  };
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
      kind: session.kind,
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
