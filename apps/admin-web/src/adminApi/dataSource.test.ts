import { describe, expect, it } from "vitest";

import { createAdminDataSource } from "./dataSource";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
  mockAddonDetail,
  mockAddonDiagnostic,
  mockAddonGrants,
  mockAddonHealth,
  mockAddonInstallGuide,
  mockAddons,
  mockAddonsRouteSummary,
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

describe("Admin data source", () => {
  it("composes live Admin API read models into console data", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [NAKO_ADMIN_ROUTES.overview]: mockOverview,
        [NAKO_ADMIN_ROUTES.addons]: mockAddons,
        [NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDetail,
        [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: mockAddonHealth,
        [NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")]: mockAddonSurfaces,
        [NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab")]: mockAddonInstallGuide,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: mockAddonTokens,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: mockAddonGrants,
        [NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDiagnostic,
        [NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates]: mockAcquisitionIntakeCandidates,
        [NAKO_ADMIN_ROUTES.catalogGovernanceItems]: mockCatalogGovernance,
        [NAKO_ADMIN_ROUTES.generatedArtifactProposals]: mockGeneratedArtifactProposals,
        [NAKO_ADMIN_ROUTES.events]: mockEvents,
        [NAKO_ADMIN_ROUTES.jobs]: mockJobs,
        [NAKO_ADMIN_ROUTES.playbackSessions]: mockPlaybackSessions,
        [NAKO_ADMIN_ROUTES.playbackRuntime]: mockPlaybackRuntime,
        [NAKO_ADMIN_ROUTES.storageStaging]: mockStorageStaging,
        [NAKO_ADMIN_ROUTES.systemConfig]: mockSystemConfig,
      }),
    });

    const data = await dataSource.load();

    expect(data.sources.overview).toBe("live");
    expect(data.sources.addons).toBe("live");
    expect(data.sources.addonHealth).toBe("live");
    expect(data.sources.addonSurfaces).toBe("live");
    expect(data.sources.addonInstallGuide).toBe("live");
    expect(data.sources.addonTokens).toBe("live");
    expect(data.sources.addonGrants).toBe("live");
    expect(data.sources.acquisitionIntake).toBe("live");
    expect(data.sources.generatedArtifactProposals).toBe("live");
    expect(data.sources.jobs).toBe("live");
    expect(data.sources.playbackSessions).toBe("live");
    expect(data.sources.playbackRuntime).toBe("live");
    expect(data.sources.storageStaging).toBe("live");
    expect(data.sources.systemConfig).toBe("live");
    expect(data.network).toMatchObject({
      exposureMode: "reverse_proxy",
      readinessStatus: "ready",
      tunnelProviderCount: 1,
    });
    expect(data.addons.addons[0]).toMatchObject({
      name: "Subtitle Lab",
      status: "enabled",
      protocolVersion: "2026-05-15",
    });
    expect(data.addons.selectedAddon).toMatchObject({
      resourceCount: 2,
      authMode: "bearer",
    });
    expect(data.addons.health).toMatchObject({
      status: "reachable",
      latencyMs: 42,
    });
    expect(data.addons.surfaces?.hostedPages[0]).toMatchObject({
      title: "Subtitle diagnostics",
    });
    expect(data.addons.installGuide).toMatchObject({
      addonName: "Subtitle Lab",
      dockerCompose: {
        filename: "compose.dev-nako-subtitle-lab.yml",
      },
      lifecycleBoundary: {
        nakoManagesContainers: false,
        nakoManagesProcesses: false,
      },
    });
    expect(data.addons.installGuide?.secretReferences[0]).toMatchObject({
      envVar: "ADDON_SECRET_SUBTITLE_PROVIDER_KEY",
      placeholder: "secret-reference:subtitle-provider-key",
    });
    expect(data.addons.tokens[0].tokenPrefix).toBe("nako_at_subtitle");
    expect(data.addons.grants[0]).toMatchObject({
      permission: "subtitle_write",
      libraryId: "library-anime",
    });
    expect(data.addons.diagnostic).toMatchObject({
      resource: "subtitle",
      status: "succeeded",
    });
    expect(data.jobs[0]).toMatchObject({
      kind: "library_scan",
      resourceClass: "library",
    });
    expect(data.acquisitionIntake.candidates[0]).toMatchObject({
      sourceKind: "watch_folder",
      sourceScheme: "local",
      state: "ready",
      hasDiagnostics: true,
    });
    expect(data.generatedArtifactProposals.proposals[0]).toMatchObject({
      capability: "metadata_cleanup",
      targetKind: "media_source",
      readinessStatus: "ready",
      confidenceMilli: 810,
    });
    expect(data.playback.accelerators).toContainEqual({
      name: "nvenc",
      available: true,
    });
    expect(data.settings).toContainEqual({
      label: "Admin auth",
      value: "Auth configured",
    });
    expect(data.settings).toContainEqual({
      label: "Network readiness",
      value: "reverse_proxy · ready",
    });
  });

  it("falls back per section when one Admin API read model fails", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [NAKO_ADMIN_ROUTES.overview]: mockOverview,
        [NAKO_ADMIN_ROUTES.addons]: new Response("offline", { status: 503 }),
        [NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDetail,
        [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: mockAddonHealth,
        [NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")]: mockAddonSurfaces,
        [NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab")]: mockAddonInstallGuide,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: mockAddonTokens,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: mockAddonGrants,
        [NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDiagnostic,
        [NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates]: mockAcquisitionIntakeCandidates,
        [NAKO_ADMIN_ROUTES.catalogGovernanceItems]: mockCatalogGovernance,
        [NAKO_ADMIN_ROUTES.generatedArtifactProposals]: mockGeneratedArtifactProposals,
        [NAKO_ADMIN_ROUTES.events]: mockEvents,
        [NAKO_ADMIN_ROUTES.jobs]: new Response("offline", { status: 503 }),
        [NAKO_ADMIN_ROUTES.playbackSessions]: mockPlaybackSessions,
        [NAKO_ADMIN_ROUTES.playbackRuntime]: mockPlaybackRuntime,
        [NAKO_ADMIN_ROUTES.storageStaging]: mockStorageStaging,
        [NAKO_ADMIN_ROUTES.systemConfig]: mockSystemConfig,
      }),
    });

    const data = await dataSource.load();

    expect(data.sources.overview).toBe("live");
    expect(data.sources.addons).toBe("mock");
    expect(data.errors.addons).toContain("HTTP 503");
    expect(data.addons.addons[0].id).toBe("addon-subtitle-lab");
    expect(data.sources.jobs).toBe("mock");
    expect(data.errors.jobs).toContain("HTTP 503");
    expect(data.jobs[0].id).toBe("job-scan");
    expect(data.sources.playbackRuntime).toBe("live");
  });

  it("loads route-local Overview with section fallback", async () => {
    const seenPaths: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenPaths.push(url.pathname);
        return Response.json(mockOverview);
      },
    });

    const liveResult = await liveSource.loadOverview?.();

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockOverview,
    });
    expect(seenPaths).toEqual([NAKO_ADMIN_ROUTES.overview]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadOverview?.();

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockOverview,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Jobs with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockJobs);
      },
    });

    const liveResult = await liveSource.loadJobs?.({
      status: "failed",
      limit: 10,
      offset: 20,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockJobs,
    });
    expect(seenSearchParams).toEqual(["?status=failed&limit=10&offset=20"]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadJobs?.({ status: "failed" });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockJobs,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Media Libraries diagnostics from system config with section fallback", async () => {
    const seenPaths: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenPaths.push(url.pathname);
        return Response.json(mockSystemConfig);
      },
    });

    const liveResult = await liveSource.loadLibraries?.();

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockSystemConfig,
    });
    expect(seenPaths).toEqual([NAKO_ADMIN_ROUTES.systemConfig]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadLibraries?.();

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockSystemConfig,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local System Settings diagnostics from system config with section fallback", async () => {
    const seenPaths: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenPaths.push(url.pathname);
        return Response.json(mockSystemConfig);
      },
    });

    const liveResult = await liveSource.loadSettings?.();

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockSystemConfig,
    });
    expect(seenPaths).toEqual([NAKO_ADMIN_ROUTES.systemConfig]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadSettings?.();

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockSystemConfig,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Acquisition Intake with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockAcquisitionIntakeCandidates);
      },
    });

    const liveResult = await liveSource.loadAcquisitionIntake?.({
      library_id: "library-anime",
      state: "ready",
      source_kind: "watch_folder",
      managed_import_artifact_id: "artifact-managed",
      limit: 10,
      offset: 20,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockAcquisitionIntakeCandidates,
    });
    expect(seenSearchParams).toEqual([
      "?library_id=library-anime&state=ready&source_kind=watch_folder&managed_import_artifact_id=artifact-managed&limit=10&offset=20",
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadAcquisitionIntake?.({
      state: "ready",
    });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockAcquisitionIntakeCandidates,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Generated Artifacts with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockGeneratedArtifactProposals);
      },
    });

    const liveResult = await liveSource.loadGeneratedArtifacts?.({
      limit: 10,
      offset: 20,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockGeneratedArtifactProposals,
    });
    expect(seenSearchParams).toEqual(["?limit=10&offset=20"]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadGeneratedArtifacts?.({
      limit: 10,
    });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockGeneratedArtifactProposals,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Addons with generated query params and safe section fallback", async () => {
    const seenRequests: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(`${url.pathname}${url.search}`);

        if (url.pathname === NAKO_ADMIN_ROUTES.addons) {
          return Response.json(mockAddons);
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")) {
          return Response.json(mockAddonDetail);
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")) {
          return Response.json(mockAddonHealth);
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")) {
          return Response.json(mockAddonSurfaces);
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab")) {
          return Response.json(mockAddonInstallGuide);
        }
        if (url.pathname === `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`) {
          return Response.json(mockAddonTokens);
        }
        if (url.pathname === `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`) {
          return Response.json(mockAddonGrants);
        }

        return new Response("not found", { status: 404 });
      },
    });

    const liveResult = await liveSource.loadAddons?.({ status: "enabled" });

    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        addons: expect.arrayContaining([
          expect.objectContaining({
            name: "Subtitle Lab",
            grantedScopeCount: 2,
          }),
        ]),
        selectedAddon: {
          name: "Subtitle Lab",
          resourceCount: 2,
          authMode: "bearer",
        },
        surfaceSummary: {
          hostedPageCount: 1,
          taskCount: 1,
        },
        installBoundary: {
          secretReferenceCount: 1,
          nakoManagesContainers: false,
        },
      },
    });
    expect(seenRequests[0]).toBe(`${NAKO_ADMIN_ROUTES.addons}?status=enabled`);
    expect(JSON.stringify(liveResult?.value)).not.toContain("http://subtitle-lab:9100");
    expect(JSON.stringify(liveResult?.value)).not.toContain("ADDON_SECRET_SUBTITLE_PROVIDER_KEY");
    expect(JSON.stringify(liveResult?.value)).not.toContain("/pages/diagnostics");
    expect(JSON.stringify(liveResult?.value)).not.toContain("docker_compose");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadAddons?.({ status: "enabled" });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        addons: [mockAddonsRouteSummary.addons[0]],
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Catalog Governance with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockCatalogGovernance);
      },
    });

    const liveResult = await liveSource.loadCatalogGovernance?.({
      library_id: "library-anime",
      max_confidence_milli: 500,
      limit: 10,
      offset: 0,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockCatalogGovernance,
    });
    expect(seenSearchParams).toEqual([
      "?library_id=library-anime&max_confidence_milli=500&limit=10&offset=0",
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadCatalogGovernance?.({
      library_id: "library-anime",
    });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockCatalogGovernance,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Playback Sessions with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockPlaybackSessions);
      },
    });

    const liveResult = await liveSource.loadPlaybackSessions?.({
      source_id: "source-hls",
      kind: "hls_transcode",
      state: "running",
      limit: 10,
      offset: 0,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockPlaybackSessions,
    });
    expect(seenSearchParams).toEqual([
      "?source_id=source-hls&kind=hls_transcode&state=running&limit=10&offset=0",
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadPlaybackSessions?.({
      state: "running",
    });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockPlaybackSessions,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Storage Staging with generated query params and section fallback", async () => {
    const seenSearchParams: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenSearchParams.push(url.search);
        return Response.json(mockStorageStaging);
      },
    });

    const liveResult = await liveSource.loadStorageStaging?.({
      purpose: "ffmpeg_input",
      state: "ready",
      limit: 10,
      offset: 0,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockStorageStaging,
    });
    expect(seenSearchParams).toEqual([
      "?purpose=ffmpeg_input&state=ready&limit=10&offset=0",
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadStorageStaging?.({
      state: "ready",
    });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockStorageStaging,
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("exposes safe Addon action methods through the data-source seam", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [NAKO_ADMIN_ROUTES.addonStatus.replace(":addon_id", "addon-subtitle-lab")]: {
          ...mockAddonDetail,
          addon: {
            ...mockAddonDetail.addon,
            summary: {
              ...mockAddonDetail.addon.summary,
              status: "disabled",
            },
          },
        },
        [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: {
          ...mockAddonHealth,
          status: "degraded",
          safe_error_code: "latency_budget_exceeded",
        },
        [NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: {
          ...mockAddonDiagnostic,
          status: "retryable_http_failure",
          http_status: 503,
          safe_error_code: "upstream_unavailable",
        },
      }),
    });

    await expect(dataSource.setAddonStatus?.("addon-subtitle-lab", "disabled")).resolves.toMatchObject({
      selectedAddon: {
        status: "disabled",
      },
    });
    await expect(dataSource.checkAddonHealth?.("addon-subtitle-lab")).resolves.toMatchObject({
      status: "degraded",
      safeErrorCode: "latency_budget_exceeded",
    });
    await expect(dataSource.diagnoseAddonResource?.("addon-subtitle-lab", "subtitle")).resolves.toMatchObject({
      status: "retryable_http_failure",
      httpStatus: 503,
      safeErrorCode: "upstream_unavailable",
    });
  });

  it("registers pasted Addon manifest JSON as a disabled Addon and returns onboarding handoff state", async () => {
    let postedBody: unknown;
    const dataSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        if (url.pathname === NAKO_ADMIN_ROUTES.addons && init?.method === "POST") {
          postedBody = JSON.parse(String(init.body));
          return Response.json({
            ...mockAddonDetail,
            addon: {
              ...mockAddonDetail.addon,
              summary: {
                ...mockAddonDetail.addon.summary,
                status: "disabled",
                granted_scopes: [],
              },
            },
          });
        }

        return new Response("not found", { status: 404 });
      },
    });

    const result = await dataSource.registerAddonManifestJson?.(
      JSON.stringify(mockAddonDetail.addon.manifest, null, 2),
    );

    expect(postedBody).toMatchObject({
      manifest: mockAddonDetail.addon.manifest,
      granted_scopes: [],
      status: "disabled",
    });
    expect(result).toMatchObject({
      status: "registered",
      addon: {
        name: "Subtitle Lab",
        status: "disabled",
        resourceCount: 2,
      },
      nextSteps: [
        "Open the generated Addon Install Guide",
        "Start the Addon Sidecar outside Nako",
        "Run Addon Health Check before enabling",
      ],
    });
  });

  it("rejects invalid pasted Addon manifest JSON before calling the Admin API", async () => {
    let called = false;
    const dataSource = createAdminDataSource({
      fetcher: async () => {
        called = true;
        return Response.json(mockAddonDetail);
      },
    });

    await expect(dataSource.registerAddonManifestJson?.("{ bad json")).resolves.toMatchObject({
      status: "invalid_json",
      error: "Manifest JSON could not be parsed.",
    });

    expect(called).toBe(false);
  });

  it("exposes Addon credential and grant onboarding actions without putting raw tokens in load data", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: {
          token: mockAddonTokens.tokens[0],
          raw_token: "nako_at_one_time_raw_token",
        },
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens/addon-token-active/rotate`]: {
          rotated: mockAddonTokens.tokens[0],
          token: {
            ...mockAddonTokens.tokens[0],
            id: "addon-token-rotated",
            token_prefix: "nako_at_rotated",
          },
          raw_token: "nako_at_rotated_one_time_raw_token",
        },
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens/addon-token-active/revoke`]: {
          token: {
            ...mockAddonTokens.tokens[0],
            status: "revoked",
            revoked_at: "2026-05-22T03:00:00.000Z",
          },
        },
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: {
          grants: [
            {
              id: "addon-grant-metadata",
              addon_id: "addon-subtitle-lab",
              permission: "metadata_write",
              library_id: null,
              created_at: "2026-05-22T03:00:00.000Z",
            },
          ],
        },
      }),
    });

    await expect(dataSource.issueAddonToken?.("addon-subtitle-lab", "sidecar runtime")).resolves.toMatchObject({
      rawToken: "nako_at_one_time_raw_token",
      token: {
        tokenPrefix: "nako_at_subtitle",
      },
    });
    await expect(dataSource.rotateAddonToken?.("addon-subtitle-lab", "addon-token-active", "replacement")).resolves.toMatchObject({
      rawToken: "nako_at_rotated_one_time_raw_token",
      token: {
        id: "addon-token-rotated",
      },
    });
    await expect(dataSource.revokeAddonToken?.("addon-subtitle-lab", "addon-token-active")).resolves.toMatchObject({
      status: "revoked",
    });
    await expect(
      dataSource.replaceAddonGrants?.("addon-subtitle-lab", [
        { permission: "metadata_write", libraryId: null },
      ]),
    ).resolves.toEqual([
      {
        id: "addon-grant-metadata",
        permission: "metadata_write",
        libraryId: null,
      },
    ]);

    const loaded = await createAdminDataSource({
      fetcher: fetcherFor({
        [NAKO_ADMIN_ROUTES.overview]: mockOverview,
        [NAKO_ADMIN_ROUTES.addons]: mockAddons,
        [NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDetail,
        [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: mockAddonHealth,
        [NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")]: mockAddonSurfaces,
        [NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab")]: mockAddonInstallGuide,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: mockAddonTokens,
        [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: mockAddonGrants,
        [NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDiagnostic,
        [NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates]: mockAcquisitionIntakeCandidates,
        [NAKO_ADMIN_ROUTES.catalogGovernanceItems]: mockCatalogGovernance,
        [NAKO_ADMIN_ROUTES.events]: mockEvents,
        [NAKO_ADMIN_ROUTES.jobs]: mockJobs,
        [NAKO_ADMIN_ROUTES.playbackSessions]: mockPlaybackSessions,
        [NAKO_ADMIN_ROUTES.playbackRuntime]: mockPlaybackRuntime,
        [NAKO_ADMIN_ROUTES.storageStaging]: mockStorageStaging,
        [NAKO_ADMIN_ROUTES.systemConfig]: mockSystemConfig,
      }),
    }).load();

    expect(JSON.stringify(loaded)).not.toContain("raw_token");
    expect(JSON.stringify(loaded)).not.toContain("one_time_raw_token");
  });
});

function fetcherFor(routes: Record<string, unknown | Response>): typeof fetch {
  return async (input: string | URL | Request) => {
    const url = new URL(input.toString(), "http://127.0.0.1");
    const response = routes[url.pathname];

    if (response instanceof Response) {
      return response;
    }

    if (!response) {
      return new Response("not found", { status: 404 });
    }

    return Response.json(response);
  };
}
