import { describe, expect, it } from "vitest";

import { createAdminDataSource } from "./dataSource";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
  mockAccessSummary,
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
  mockCatalogGovernanceItemDetail,
  mockCatalogGovernanceProviderMappingReviewPlan,
  mockCatalogGovernanceProviderMappingReviewResponse,
  mockEvents,
  mockGeneratedArtifactProposals,
  mockGeneratedArtifactReviewPlan,
  mockGeneratedArtifactReviewResponse,
  mockJobs,
  mockLibraryMetadataProfile,
  mockMetadataRawCacheSettings,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackSessions,
  mockPublicCatalogItems,
  mockPublicCatalogSearch,
  mockPublicItemDetail,
  mockPublicSourceProbe,
  mockSourceDuplicateReconciliationApply,
  mockSourceDuplicateReconciliationPlan,
  mockStorageStaging,
  mockSystemConfig,
} from "./mockData";

describe("Admin data source", () => {
  it("composes live Admin API read models into console data", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [NAKO_ADMIN_ROUTES.overview]: mockOverview,
        [NAKO_ADMIN_ROUTES.accessSummary]: mockAccessSummary,
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
        [NAKO_ADMIN_ROUTES.settingsMetadataRawCache]: mockMetadataRawCacheSettings,
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

  it("loads route-local Access Summary with section fallback", async () => {
    const seenPaths: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenPaths.push(url.pathname);
        return Response.json(mockAccessSummary);
      },
    });

    const liveResult = await liveSource.loadAccessSummary?.();

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockAccessSummary,
    });
    expect(seenPaths).toEqual([NAKO_ADMIN_ROUTES.accessSummary]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadAccessSummary?.();

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockAccessSummary,
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

  it("loads route-local Media Library detail from system config and metadata profile routes", async () => {
    const seenPaths: string[] = [];
    const metadataProfileRoute = NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace(
      "{library_id}",
      "library-anime",
    );
    const sourceInventoryRoute = "/libraries/library-anime/sources";
    const sourceInventoryResponse = {
      library: { id: "library-anime", name: "Anime Vault" },
      sources: [
        {
          source: {
            id: "source-anime-1",
            library_id: "library-anime",
            item_id: "item-anime-1",
            file_name: "Episode 01.mkv",
            size_bytes: 1468006400,
            fingerprint: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          },
          item: {
            id: "item-anime-1",
            title: "Pilot",
            kind: "episode",
          },
          probe: {
            duration_ms: 1440000,
            container: "matroska",
            streams: [],
          },
        },
      ],
      page: { limit: 50, offset: 0, returned: 1 },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenPaths.push(url.pathname);

        if (url.pathname === NAKO_ADMIN_ROUTES.systemConfig) {
          return Response.json(mockSystemConfig);
        }
        if (url.pathname === metadataProfileRoute) {
          return Response.json(mockLibraryMetadataProfile("library-anime"));
        }
        if (url.pathname === sourceInventoryRoute) {
          return Response.json(sourceInventoryResponse);
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.jobs) {
          return Response.json(mockJobs);
        }

        return new Response("not found", { status: 404 });
      },
    });

    const liveResult = await liveSource.loadLibraryDetail?.("library-anime");

    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        configuredLibraryCount: mockSystemConfig.libraries.length,
        library: {
          id: "library-anime",
          name: "Anime Vault",
        },
        metadataProfile: {
          library_id: "library-anime",
          profile: {
            refresh_mode: "missing_only",
          },
        },
        sourceInventory: {
          source: "live",
          sourceCount: 1,
          linkedItemCount: 1,
          probedSourceCount: 1,
          latestScanJob: {
            kind: "library_scan",
          },
          failedJobCount: 0,
          samples: [
            {
              fileName: "Episode 01.mkv",
              itemTitle: "Pilot",
            },
          ],
        },
      },
    });
    expect(seenPaths).toEqual([
      NAKO_ADMIN_ROUTES.systemConfig,
      metadataProfileRoute,
      sourceInventoryRoute,
      NAKO_ADMIN_ROUTES.jobs,
    ]);

    const hybridSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        if (url.pathname === NAKO_ADMIN_ROUTES.systemConfig) {
          return Response.json(mockSystemConfig);
        }

        return new Response("offline", { status: 503 });
      },
    });

    const hybridResult = await hybridSource.loadLibraryDetail?.("library-anime");

    expect(hybridResult).toMatchObject({
      source: "hybrid",
      value: {
        library: {
          id: "library-anime",
        },
        metadataProfile: {
          library_id: "library-anime",
        },
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("exposes library profile replacement and command actions through Admin API", async () => {
    const postedBodies: unknown[] = [];
    const profile = {
      ...mockLibraryMetadataProfile("library-anime").profile,
      refresh_mode: "full_refresh" as const,
    };
    const dataSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        if (init?.body) {
          postedBodies.push(JSON.parse(String(init.body)));
        }

        if (url.pathname === NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace("{library_id}", "library-anime")) {
          return Response.json({ ...mockLibraryMetadataProfile("library-anime"), profile });
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.libraryScan.replace("{library_id}", "library-anime")) {
          return Response.json({
            ...mockJobs.jobs[0],
            kind: "library_scan",
            resource_class: "disk.scan",
            library_id: "library-anime",
          });
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.libraryNfoImport.replace("{library_id}", "library-anime")) {
          return Response.json({
            ...mockJobs.jobs[0],
            id: "job-nfo-import",
            kind: "nfo_import",
            resource_class: "metadata.nfo.import",
            library_id: "library-anime",
          });
        }
        if (url.pathname === NAKO_ADMIN_ROUTES.libraryNfoExport.replace("{library_id}", "library-anime")) {
          return Response.json({
            ...mockJobs.jobs[0],
            id: "job-nfo-export",
            kind: "nfo_export",
            resource_class: "metadata.nfo.export",
            library_id: "library-anime",
          });
        }

        return new Response("not found", { status: 404 });
      },
    });

    await expect(dataSource.updateLibraryMetadataProfile?.("library-anime", profile)).resolves.toMatchObject({
      profile: {
        refresh_mode: "full_refresh",
      },
    });
    await expect(dataSource.runLibraryCommand?.("library-anime", "scan")).resolves.toMatchObject({
      action: "scan",
      job: {
        kind: "library_scan",
        resourceClass: "disk.scan",
      },
    });
    await expect(dataSource.runLibraryCommand?.("library-anime", "nfoImport")).resolves.toMatchObject({
      action: "nfoImport",
      job: {
        kind: "nfo_import",
        resourceClass: "metadata.nfo.import",
      },
    });
    await expect(dataSource.runLibraryCommand?.("library-anime", "nfoExport")).resolves.toMatchObject({
      action: "nfoExport",
      job: {
        kind: "nfo_export",
        resourceClass: "metadata.nfo.export",
      },
    });

    expect(postedBodies).toEqual([{ profile }, {}, {}, {}]);
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

  it("loads and updates metadata raw cache settings without mock mutation fallback", async () => {
    const seenRequests: Array<{ path: string; method: string; body: unknown }> = [];
    const dataSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          path: url.pathname,
          method: init?.method ?? "GET",
          body: init?.body ? JSON.parse(String(init.body)) : null,
        });

        if (url.pathname === NAKO_ADMIN_ROUTES.settingsMetadataRawCache && init?.method === "PUT") {
          return Response.json({
            ...mockMetadataRawCacheSettings,
            retention_ms: 3_600_000,
            cleanup_on_startup: false,
            source: "admin",
            effect: "requires_restart",
            updated_at_ms: 1779700000000,
          });
        }

        if (url.pathname === NAKO_ADMIN_ROUTES.settingsMetadataRawCache) {
          return Response.json(mockMetadataRawCacheSettings);
        }

        return new Response("not found", { status: 404 });
      },
    });

    await expect(dataSource.loadMetadataRawCacheSettings?.()).resolves.toMatchObject({
      source: "live",
      value: mockMetadataRawCacheSettings,
    });
    await expect(
      dataSource.updateMetadataRawCacheSettings?.({
        retention_ms: 3_600_000,
        cleanup_on_startup: false,
      }),
    ).resolves.toMatchObject({
      retention_ms: 3_600_000,
      cleanup_on_startup: false,
      source: "admin",
      effect: "requires_restart",
    });

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
        method: "GET",
        body: null,
      },
      {
        path: NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
        method: "PUT",
        body: {
          retention_ms: 3_600_000,
          cleanup_on_startup: false,
        },
      },
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    await expect(fallbackSource.loadMetadataRawCacheSettings?.()).resolves.toMatchObject({
      source: "mock",
      value: mockMetadataRawCacheSettings,
      error: expect.stringContaining("HTTP 503"),
    });
    await expect(
      fallbackSource.updateMetadataRawCacheSettings?.({
        retention_ms: 3_600_000,
        cleanup_on_startup: false,
      }),
    ).rejects.toThrow("HTTP 503");
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

  it("loads Generated Artifact review plans with safe projection and deterministic fallback", async () => {
    const seenRequests: Array<{ body: unknown; path: string }> = [];
    const basePlan = mockGeneratedArtifactReviewPlan("artifact-metadata-cleanup", "reject");
    const unsafePlan = {
      ...basePlan,
      plan: {
        ...basePlan.plan,
        prompt_text: "secret prompt body",
        raw_provider_response: "provider raw body",
        target: {
          ...basePlan.plan.target,
          source_uri: "file:///Users/frank/generated",
          local_path: "F:\\generated\\artifact.json",
        },
        payload: {
          ...basePlan.plan.payload,
          raw_json: '{"secret":"secret payload body"}',
        },
      },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          path: url.pathname,
          body: JSON.parse(init?.body?.toString() ?? "{}"),
        });
        return Response.json(unsafePlan);
      },
    });

    const liveResult = await liveSource.loadGeneratedArtifactReviewPlan?.(
      "artifact-metadata-cleanup",
      "reject",
    );

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.generatedArtifactReviewPlan.replace(
          "{artifact_id}",
          "artifact-metadata-cleanup",
        ),
        body: { decision: "reject" },
      },
    ]);
    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        artifactId: "artifact-metadata-cleanup",
        decision: "reject",
        action: "mark_rejected",
        target: {
          libraryId: "library-anime",
          itemId: "item-unknown-1",
          sourceId: "source-unknown-1",
        },
      },
    });
    const projectedText = JSON.stringify(liveResult?.value);
    expect(projectedText).not.toContain("prompt_text");
    expect(projectedText).not.toContain("secret prompt body");
    expect(projectedText).not.toContain("raw_json");
    expect(projectedText).not.toContain("secret payload body");
    expect(projectedText).not.toContain("raw_provider_response");
    expect(projectedText).not.toContain("provider raw body");
    expect(projectedText).not.toContain("source_uri");
    expect(projectedText).not.toContain("file:///Users/frank/generated");
    expect(projectedText).not.toContain("F:\\");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadGeneratedArtifactReviewPlan?.(
      "artifact-stale-title-match",
      "accept",
    );

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        artifactId: "artifact-stale-title-match",
        decision: "accept",
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("posts Generated Artifact reviews without fake mutation fallback", async () => {
    const seenRequests: Array<{ body: unknown; path: string }> = [];
    const baseResult = mockGeneratedArtifactReviewResponse("artifact-metadata-cleanup", "accept");
    const unsafeResult = {
      ...baseResult,
      prompt_text: "secret prompt body",
      payload_body: "secret payload body",
      raw_provider_response: "provider raw body",
      artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
      plan: {
        ...baseResult.plan,
        raw_provider_response: "provider raw body",
        target: {
          ...baseResult.plan.target,
          source_uri: "file:///Users/frank/generated",
          local_path: "F:\\generated\\artifact.json",
        },
        payload: {
          ...baseResult.plan.payload,
          raw_json: '{"secret":"secret payload body"}',
        },
      },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          path: url.pathname,
          body: JSON.parse(init?.body?.toString() ?? "{}"),
        });
        return Response.json(unsafeResult);
      },
    });

    const liveResult = await liveSource.reviewGeneratedArtifact?.(
      "artifact-metadata-cleanup",
      "accept",
    );

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.generatedArtifactReview.replace(
          "{artifact_id}",
          "artifact-metadata-cleanup",
        ),
        body: { decision: "accept" },
      },
    ]);
    expect(liveResult).toMatchObject({
      artifactId: "artifact-metadata-cleanup",
      decision: "accept",
      artifactStatus: "accepted",
      idempotentReplay: false,
      plan: {
        action: "stage_metadata_authority_review",
      },
    });
    const projectedText = JSON.stringify(liveResult);
    expect(projectedText).not.toContain("prompt_text");
    expect(projectedText).not.toContain("secret prompt body");
    expect(projectedText).not.toContain("payload_body");
    expect(projectedText).not.toContain("secret payload body");
    expect(projectedText).not.toContain("raw_provider_response");
    expect(projectedText).not.toContain("provider raw body");
    expect(projectedText).not.toContain("artifact_storage_handle");
    expect(projectedText).not.toContain("source_uri");
    expect(projectedText).not.toContain("file:///Users/frank/generated");
    expect(projectedText).not.toContain("F:\\");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    await expect(
      fallbackSource.reviewGeneratedArtifact?.("artifact-metadata-cleanup", "reject"),
    ).rejects.toThrow("HTTP 503");
  });

  it("loads route-local Media Catalog through public read bridges and safe summaries", async () => {
    const seenRequests: string[] = [];
    const unsafeBrowseResponse = {
      ...mockPublicCatalogItems,
      items: mockPublicCatalogItems.items.map((item) => ({
        ...item,
        source_uri: "file:///Users/frank/media/private.mkv",
        raw_provider_response: "provider raw body",
        metadata: {
          ...item.metadata,
          provider_secret: "secret-provider-token",
        },
      })),
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(`${url.pathname}${url.search}`);

        if (url.pathname === "/items") {
          return Response.json(unsafeBrowseResponse);
        }

        if (url.pathname === "/search") {
          return Response.json(mockPublicCatalogSearch);
        }

        return new Response("not found", { status: 404 });
      },
    });

    const browseResult = await liveSource.loadCatalog?.({ limit: 10, offset: 20 });
    const searchResult = await liveSource.loadCatalog?.({
      q: "ova",
      facet: "kind:movie",
      limit: 5,
      offset: 0,
    });

    expect(browseResult).toMatchObject({
      source: "live",
      value: {
        mode: "browse",
        items: expect.arrayContaining([
          expect.objectContaining({
            id: "item-unknown-1",
            title: "Unmatched OVA Special",
            genreCount: 1,
            tagCount: 2,
            sourceCount: null,
            imageCount: null,
          }),
        ]),
      },
    });
    expect(searchResult).toMatchObject({
      source: "live",
      value: {
        mode: "search",
        items: [
          expect.objectContaining({
            id: "item-unknown-1",
            score: 0.91,
          }),
        ],
      },
    });
    expect(seenRequests).toEqual([
      "/items?limit=10&offset=20",
      "/search?q=ova&facet=kind%3Amovie&limit=5&offset=0",
    ]);
    expect(JSON.stringify(browseResult?.value)).not.toContain("source_uri");
    expect(JSON.stringify(browseResult?.value)).not.toContain("provider raw body");
    expect(JSON.stringify(browseResult?.value)).not.toContain("secret-provider-token");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadCatalog?.({ q: "ova" });

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        mode: "search",
        items: [
          {
            id: "item-unknown-1",
          },
        ],
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads route-local Media Item detail through public read bridges and safe summaries", async () => {
    const seenRequests: string[] = [];
    const detail = mockPublicItemDetail("item-unknown-1");
    const unsafeDetail = {
      ...detail,
      source_uri: "file:///Users/frank/media/private.mkv",
      raw_provider_response: "provider raw body",
      images: [
        ...detail.images,
        {
          ...detail.images[0],
          id: "image-unsafe",
          url: "file:///Users/frank/artwork/poster.jpg",
          owner: {
            raw_path: "F:\\nako\\artwork\\poster.jpg",
          },
        },
      ],
      sources: [
        ...detail.sources,
        {
          id: "source-extra-3",
          library_id: "library-anime",
          item_id: "item-unknown-1",
          file_name: "Extra 03.mkv",
          size_bytes: null,
          fingerprint: null,
        },
        {
          id: "source-extra-4",
          library_id: "library-anime",
          item_id: "item-unknown-1",
          file_name: "Extra 04.mkv",
          size_bytes: null,
          fingerprint: null,
        },
      ],
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(`${url.pathname}${url.search}`);

        if (url.pathname === "/items/item-unknown-1") {
          return Response.json(unsafeDetail);
        }

        if (
          url.pathname === "/sources/source-unknown-1/probe" ||
          url.pathname === "/sources/source-unknown-2/probe" ||
          url.pathname === "/sources/source-extra-3/probe"
        ) {
          return Response.json(mockPublicSourceProbe(url.pathname.split("/")[2]));
        }

        return new Response("not found", { status: 404 });
      },
    });

    const result = await liveSource.loadItemDetail?.("item-unknown-1");

    expect(result).toMatchObject({
      source: "live",
      value: {
        item: {
          id: "item-unknown-1",
          title: "Unmatched OVA Special",
          sourceCount: 4,
          imageCount: 2,
        },
        sources: expect.arrayContaining([
          expect.objectContaining({
            id: "source-unknown-1",
            fileName: "Unmatched OVA Special.mkv",
            probe: expect.objectContaining({
              streamCount: 3,
              videoStreamCount: 1,
              audioStreamCount: 1,
              subtitleStreamCount: 1,
            }),
          }),
          expect.objectContaining({
            id: "source-extra-4",
            probe: null,
          }),
        ]),
        images: expect.arrayContaining([
          expect.objectContaining({
            id: "image-unsafe",
            routePath: null,
          }),
        ]),
      },
    });
    expect(seenRequests).toEqual([
      "/items/item-unknown-1",
      "/sources/source-unknown-1/probe",
      "/sources/source-unknown-2/probe",
      "/sources/source-extra-3/probe",
    ]);
    expect(JSON.stringify(result?.value)).not.toContain("source_uri");
    expect(JSON.stringify(result?.value)).not.toContain("provider raw body");
    expect(JSON.stringify(result?.value)).not.toContain("file:///Users/frank");
    expect(JSON.stringify(result?.value)).not.toContain("F:\\");

    const fallbackRequests: string[] = [];
    const fallbackSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        fallbackRequests.push(`${url.pathname}${url.search}`);

        return new Response("offline", { status: 503 });
      },
    });

    const fallbackResult = await fallbackSource.loadItemDetail?.("item-unknown-1");

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        item: {
          id: "item-unknown-1",
        },
      },
      error: expect.stringContaining("HTTP 503"),
    });
    expect(fallbackRequests).toEqual(["/items/item-unknown-1"]);
  });

  it("loads item artwork gallery with generated query params and safe summaries", async () => {
    const seenRequests: string[] = [];
    const gallery = {
      item_id: "item-unknown-1",
      summary: {
        candidates: 2,
        artifacts: 2,
        selected: 1,
      },
      candidates: [
        {
          id: "candidate-poster-1",
          addon_id: "addon-artwork-curator",
          side_effect_id: "side-effect-poster-1",
          library_id: "library-anime",
          item_id: "item-unknown-1",
          kind: "poster",
          source_kind: "provider",
          status: "accepted",
          width: 1000,
          height: 1500,
          language: "ja",
          ingest: {
            id: "ingest-poster-1",
            candidate_id: "candidate-poster-1",
            job_id: "job-artwork-1",
            library_id: "library-anime",
            item_id: "item-unknown-1",
            kind: "poster",
            status: "completed",
            has_artifact: true,
            has_failure: false,
            failure_code: null,
            created_at: "2026-05-25T01:00:00Z",
            updated_at: "2026-05-25T01:10:00Z",
          },
          artifact_id: "artifact-poster-1",
          has_stored_artifact: true,
          selected_artwork_count: 1,
          selected: true,
          created_at: "2026-05-25T00:59:00Z",
          updated_at: "2026-05-25T01:11:00Z",
          source_uri: "https://provider.example/poster.jpg?token=secret",
          storage_uri: "managed-artwork://library-anime/private/poster.jpg",
        },
      ],
      artifacts: [
        {
          id: "artifact-poster-1",
          ingest_id: "ingest-poster-1",
          candidate_id: "candidate-poster-1",
          library_id: "library-anime",
          item_id: "item-unknown-1",
          kind: "poster",
          selected_artwork_count: 1,
          selected: true,
          width: 1000,
          height: 1500,
          byte_len: 542000,
          media_type: "image/jpeg",
          has_content_hash: true,
          created_at: "2026-05-25T01:10:00Z",
          updated_at: "2026-05-25T01:11:00Z",
          content_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
          local_path: "F:\\nako\\artwork\\poster.jpg",
        },
      ],
      selected: [
        {
          selected_artwork: {
            id: "selected-poster-1",
            library_id: "library-anime",
            item_id: "item-unknown-1",
            kind: "poster",
            artifact_id: "artifact-poster-1",
            created_at: "2026-05-25T01:12:00Z",
            updated_at: "2026-05-25T01:12:00Z",
          },
          artifact: {
            id: "artifact-poster-1",
            ingest_id: "ingest-poster-1",
            candidate_id: "candidate-poster-1",
            library_id: "library-anime",
            item_id: "item-unknown-1",
            kind: "poster",
            selected_artwork_count: 1,
            selected: true,
            width: 1000,
            height: 1500,
            byte_len: 542000,
            media_type: "image/jpeg",
            has_content_hash: true,
            created_at: "2026-05-25T01:10:00Z",
            updated_at: "2026-05-25T01:11:00Z",
          },
          image: {
            id: "image-poster-1",
            owner: {
              kind: "item",
              item_id: "item-unknown-1",
              raw_path: "F:\\nako\\artwork\\poster.jpg",
            },
            kind: "poster",
            url: "/images/image-poster-1",
            width: 1000,
            height: 1500,
            language: "ja",
            media_type: "image/jpeg",
            etag: "poster-etag",
            cache_uri: "managed-artwork://cache/private/poster.jpg",
          },
        },
      ],
      page: {
        limit: 5,
        offset: 10,
        returned: 1,
      },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(`${url.pathname}${url.search}`);

        if (url.pathname === "/admin/v1/items/item-unknown-1/artwork") {
          return Response.json(gallery);
        }

        return new Response("not found", { status: 404 });
      },
    });

    const result = await liveSource.loadItemArtworkGallery?.("item-unknown-1", {
      limit: 5,
      offset: 10,
    });

    expect(result).toMatchObject({
      source: "live",
      value: {
        itemId: "item-unknown-1",
        totals: {
          candidateCount: 2,
          artifactCount: 2,
          selectedCount: 1,
        },
        candidates: [
          expect.objectContaining({
            id: "candidate-poster-1",
            sourceKind: "provider",
            selected: true,
            ingestStatus: "completed",
          }),
        ],
        artifacts: [
          expect.objectContaining({
            id: "artifact-poster-1",
            hasContentHash: true,
            selected: true,
          }),
        ],
        selected: [
          expect.objectContaining({
            selectedArtworkId: "selected-poster-1",
            artifactId: "artifact-poster-1",
            imageId: "image-poster-1",
            routePath: "/images/image-poster-1",
          }),
        ],
      },
    });
    expect(seenRequests).toEqual(["/admin/v1/items/item-unknown-1/artwork?limit=5&offset=10"]);
    expect(JSON.stringify(result?.value)).not.toContain("source_uri");
    expect(JSON.stringify(result?.value)).not.toContain("storage_uri");
    expect(JSON.stringify(result?.value)).not.toContain("managed-artwork://");
    expect(JSON.stringify(result?.value)).not.toContain("content_hash");
    expect(JSON.stringify(result?.value)).not.toContain("provider.example");
    expect(JSON.stringify(result?.value)).not.toContain("F:\\");

    const fallbackRequests: string[] = [];
    const fallbackSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        fallbackRequests.push(`${url.pathname}${url.search}`);

        return new Response("offline", { status: 503 });
      },
    });

    const fallbackResult = await fallbackSource.loadItemArtworkGallery?.("item-unknown-1");

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        itemId: "item-unknown-1",
        totals: {
          candidateCount: expect.any(Number),
          artifactCount: expect.any(Number),
          selectedCount: expect.any(Number),
        },
      },
      error: expect.stringContaining("HTTP 503"),
    });
    expect(fallbackRequests).toEqual(["/admin/v1/items/item-unknown-1/artwork"]);
  });

  it("maps item artwork select and unpublish mutations without mock success fallback", async () => {
    const seenRequests: Array<{ body: string | null; method: string; path: string }> = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          body: typeof init?.body === "string" ? init.body : null,
          method: init?.method ?? "GET",
          path: url.pathname,
        });

        if (url.pathname === "/admin/v1/items/item%2Funsafe%20id/artwork/backdrop/select") {
          return Response.json({
            selected_artwork: {
              id: "selected-backdrop-1",
              library_id: "library-anime",
              item_id: "item/unsafe id",
              kind: "backdrop",
              artifact_id: "artifact/unsafe id",
              created_at: "2026-05-25T02:00:00Z",
              updated_at: "2026-05-25T02:00:00Z",
            },
            image: {
              id: "image-backdrop-1",
              owner: {
                kind: "item",
                item_id: "item/unsafe id",
                raw_path: "F:\\nako\\private\\backdrop.webp",
              },
              kind: "backdrop",
              url: "https://provider.example/backdrop.webp?token=secret",
              width: 1920,
              height: 1080,
              language: null,
              media_type: "image/webp",
              etag: "private-etag",
            },
            changed: true,
          });
        }

        if (url.pathname === "/admin/v1/items/item%2Funsafe%20id/artwork/poster/selection") {
          return Response.json({
            item_id: "item/unsafe id",
            kind: "poster",
            changed: true,
            unpublished: {
              selected_artwork: {
                id: "selected-poster-1",
                library_id: "library-anime",
                item_id: "item/unsafe id",
                kind: "poster",
                artifact_id: "artifact-poster-1",
                created_at: "2026-05-25T01:12:00Z",
                updated_at: "2026-05-25T01:12:00Z",
              },
              previous_image: {
                id: "image-poster-1",
                owner: {
                  kind: "item",
                  item_id: "item/unsafe id",
                  raw_path: "F:\\nako\\private\\poster.jpg",
                },
                kind: "poster",
                url: "/images/image-poster-1",
                width: 1000,
                height: 1500,
                language: "ja",
                media_type: "image/jpeg",
                etag: "poster-etag",
                cache_uri: "managed-artwork://cache/private/poster.jpg",
              },
            },
          });
        }

        return new Response("not found", { status: 404 });
      },
    });

    const selectResult = await liveSource.selectItemArtwork?.(
      "item/unsafe id",
      "backdrop",
      "artifact/unsafe id",
    );
    const unpublishResult = await liveSource.unpublishItemArtwork?.("item/unsafe id", "poster");

    expect(seenRequests).toEqual([
      {
        body: JSON.stringify({ artifact_id: "artifact/unsafe id" }),
        method: "POST",
        path: "/admin/v1/items/item%2Funsafe%20id/artwork/backdrop/select",
      },
      {
        body: null,
        method: "DELETE",
        path: "/admin/v1/items/item%2Funsafe%20id/artwork/poster/selection",
      },
    ]);
    expect(selectResult).toMatchObject({
      action: "select",
      itemId: "item/unsafe id",
      kind: "backdrop",
      changed: true,
      selectedArtworkId: "selected-backdrop-1",
      artifactId: "artifact/unsafe id",
      imageId: "image-backdrop-1",
      routePath: null,
    });
    expect(unpublishResult).toMatchObject({
      action: "unpublish",
      itemId: "item/unsafe id",
      kind: "poster",
      changed: true,
      selectedArtworkId: "selected-poster-1",
      artifactId: "artifact-poster-1",
      imageId: "image-poster-1",
      routePath: "/images/image-poster-1",
    });

    const projectedText = JSON.stringify({ selectResult, unpublishResult });
    expect(projectedText).not.toContain("provider.example");
    expect(projectedText).not.toContain("token=secret");
    expect(projectedText).not.toContain("raw_path");
    expect(projectedText).not.toContain("cache_uri");
    expect(projectedText).not.toContain("managed-artwork://");
    expect(projectedText).not.toContain("F:\\");

    const failingSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    await expect(
      failingSource.selectItemArtwork?.("item-unknown-1", "poster", "artifact-poster-1"),
    ).rejects.toThrow("HTTP 503");
    await expect(
      failingSource.unpublishItemArtwork?.("item-unknown-1", "poster"),
    ).rejects.toThrow("HTTP 503");
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

  it("loads Catalog Governance item detail through a safe projection and deterministic fallback", async () => {
    const seenRequests: string[] = [];
    const baseDetail = mockCatalogGovernanceItemDetail("item-low-confidence");
    const unsafeDetail = {
      ...baseDetail,
      item: {
        ...baseDetail.item,
        local_inference: {
          source_id: "source-low-confidence",
          inferred_kind: "movie",
          inferred_title: "Film Needs Mapping",
          inferred_year: 1999,
          inferred_season: null,
          inferred_episode: null,
          confidence_milli: 420,
          evidence_source: "path",
          has_evidence: true,
          inference_version: "nako-naming:1",
          evidence_value: "raw-evidence-token",
          source_locator: "local:///library/private/Film Needs Mapping.mkv",
          local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
        },
      },
      provider_mappings: baseDetail.provider_mappings.map((mapping) => ({
        ...mapping,
        provider_raw_body: "provider raw body",
        provider_request_url: "https://provider.example/api?token=secret",
        evidence_value: "mapping-raw-evidence",
        subject: {
          ...mapping.subject,
          raw_payload: "subject raw payload",
        },
      })),
      source_locator: "local:///library/private/Film Needs Mapping.mkv",
      nfo_xml: "<movie><title>secret</title></movie>",
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(url.pathname);
        return Response.json(unsafeDetail);
      },
    });

    const liveResult = await liveSource.loadCatalogGovernanceItemDetail?.(
      "item-low-confidence",
    );

    expect(seenRequests).toEqual([
      NAKO_ADMIN_ROUTES.catalogGovernanceItemDetail.replace(
        "{item_id}",
        "item-low-confidence",
      ),
    ]);
    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        item: {
          id: "item-low-confidence",
          title: "Film Needs Mapping",
          localInference: {
            sourceId: "source-low-confidence",
            hasEvidence: true,
          },
        },
        providerMappings: [
          {
            id: "mapping-tmdb-603",
            source: "provider:tmdb",
            subject: {
              provider: "tmdb",
              key: "603",
            },
          },
        ],
        repairActions: ["provider_mapping_review"],
      },
    });
    const projectedText = JSON.stringify(liveResult?.value);
    expect(projectedText).not.toContain("evidence_value");
    expect(projectedText).not.toContain("raw-evidence-token");
    expect(projectedText).not.toContain("source_locator");
    expect(projectedText).not.toContain("local:///");
    expect(projectedText).not.toContain("local_path");
    expect(projectedText).not.toContain("F:\\");
    expect(projectedText).not.toContain("provider_raw_body");
    expect(projectedText).not.toContain("provider raw body");
    expect(projectedText).not.toContain("provider_request_url");
    expect(projectedText).not.toContain("token=secret");
    expect(projectedText).not.toContain("nfo_xml");
    expect(projectedText).not.toContain("<movie>");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadCatalogGovernanceItemDetail?.(
      "item-fallback",
    );

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        item: {
          id: "item-fallback",
        },
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("loads Catalog Governance Provider Mapping review plans with safe projection and fallback", async () => {
    const seenRequests: Array<{ body: unknown; path: string }> = [];
    const basePlan = mockCatalogGovernanceProviderMappingReviewPlan(
      "item-low-confidence",
      "mapping-tmdb-603",
      "reject",
    );
    const unsafePlan = {
      ...basePlan,
      plan: {
        ...basePlan.plan,
        raw_provider_response: "provider raw body",
        item: {
          ...basePlan.plan.item,
          source_locator: "local:///library/private/Film Needs Mapping.mkv",
          local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
        },
        mapping: {
          ...basePlan.plan.mapping,
          evidence_value: "mapping-raw-evidence",
          provider_request_url: "https://provider.example/api?token=secret",
          subject: {
            ...basePlan.plan.mapping.subject,
            raw_payload: "subject raw payload",
          },
        },
      },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          path: url.pathname,
          body: JSON.parse(init?.body?.toString() ?? "{}"),
        });
        return Response.json(unsafePlan);
      },
    });

    const liveResult = await liveSource.loadCatalogGovernanceProviderMappingReviewPlan?.(
      "item-low-confidence",
      "mapping-tmdb-603",
      "reject",
    );

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReviewPlan
          .replace("{item_id}", "item-low-confidence")
          .replace("{mapping_id}", "mapping-tmdb-603"),
        body: { decision: "reject" },
      },
    ]);
    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        decision: "reject",
        currentStatus: "candidate",
        targetStatus: "rejected",
        mapping: {
          id: "mapping-tmdb-603",
          source: "provider:tmdb",
        },
        boundary: {
          updatesProviderMappingStatus: true,
          writesLibraryFiles: false,
        },
      },
    });
    const projectedText = JSON.stringify(liveResult?.value);
    expect(projectedText).not.toContain("raw_provider_response");
    expect(projectedText).not.toContain("provider raw body");
    expect(projectedText).not.toContain("evidence_value");
    expect(projectedText).not.toContain("mapping-raw-evidence");
    expect(projectedText).not.toContain("provider_request_url");
    expect(projectedText).not.toContain("token=secret");
    expect(projectedText).not.toContain("source_locator");
    expect(projectedText).not.toContain("local:///");
    expect(projectedText).not.toContain("local_path");
    expect(projectedText).not.toContain("F:\\");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadCatalogGovernanceProviderMappingReviewPlan?.(
      "item-fallback",
      "mapping-fallback",
      "accept",
    );

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: {
        decision: "accept",
        mapping: {
          id: "mapping-fallback",
        },
      },
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("posts Catalog Governance Provider Mapping reviews without fake mutation fallback", async () => {
    const seenRequests: Array<{ body: unknown; path: string }> = [];
    const baseResult = mockCatalogGovernanceProviderMappingReviewResponse(
      "item-low-confidence",
      "mapping-tmdb-603",
      "accept",
    );
    const unsafeResult = {
      ...baseResult,
      evidence_value: "mapping-raw-evidence",
      source_locator: "local:///library/private/Film Needs Mapping.mkv",
      local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
      plan: {
        ...baseResult.plan,
        raw_provider_response: "provider raw body",
        item: {
          ...baseResult.plan.item,
          local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
        },
        mapping: {
          ...baseResult.plan.mapping,
          provider_request_url: "https://provider.example/api?token=secret",
          subject: {
            ...baseResult.plan.mapping.subject,
            raw_payload: "subject raw payload",
          },
        },
      },
    };
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          path: url.pathname,
          body: JSON.parse(init?.body?.toString() ?? "{}"),
        });
        return Response.json(unsafeResult);
      },
    });

    const liveResult = await liveSource.reviewCatalogGovernanceProviderMapping?.(
      "item-low-confidence",
      "mapping-tmdb-603",
      "accept",
    );

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReview
          .replace("{item_id}", "item-low-confidence")
          .replace("{mapping_id}", "mapping-tmdb-603"),
        body: { decision: "accept" },
      },
    ]);
    expect(liveResult).toMatchObject({
      itemId: "item-low-confidence",
      mappingId: "mapping-tmdb-603",
      decision: "accept",
      previousStatus: "candidate",
      currentStatus: "accepted",
      changed: true,
      idempotentReplay: false,
      plan: {
        boundary: {
          updatesProviderMappingStatus: true,
          writesLibraryFiles: false,
        },
      },
    });
    const projectedText = JSON.stringify(liveResult);
    expect(projectedText).not.toContain("raw_provider_response");
    expect(projectedText).not.toContain("provider raw body");
    expect(projectedText).not.toContain("evidence_value");
    expect(projectedText).not.toContain("mapping-raw-evidence");
    expect(projectedText).not.toContain("provider_request_url");
    expect(projectedText).not.toContain("token=secret");
    expect(projectedText).not.toContain("source_locator");
    expect(projectedText).not.toContain("local:///");
    expect(projectedText).not.toContain("local_path");
    expect(projectedText).not.toContain("F:\\");

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    await expect(
      fallbackSource.reviewCatalogGovernanceProviderMapping?.(
        "item-low-confidence",
        "mapping-tmdb-603",
        "reject",
      ),
    ).rejects.toThrow("HTTP 503");
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
      state: "running",
      limit: 10,
      offset: 0,
    });

    expect(liveResult).toMatchObject({
      source: "live",
      value: mockPlaybackSessions,
    });
    expect(seenSearchParams).toEqual([
      "?source_id=source-hls&state=running&limit=10&offset=0",
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

  it("loads source duplicate reconciliation plans with generated query params and section fallback", async () => {
    const seenRequests: string[] = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push(`${url.pathname}${url.search}`);
        return Response.json(
          mockSourceDuplicateReconciliationPlan("library/unsafe id", "source/unsafe id"),
        );
      },
    });

    const liveResult = await liveSource.loadSourceDuplicateReconciliationPlan?.(
      "library/unsafe id",
      "source/unsafe id",
      { limit: 5, offset: 10 },
    );

    expect(liveResult).toMatchObject({
      source: "live",
      value: {
        library_id: "library/unsafe id",
        source_id: "source/unsafe id",
        candidates: expect.arrayContaining([
          expect.objectContaining({
            duplicate_source_id: "source-unknown-2",
            recommended_action: "suggest_relationship",
          }),
        ]),
      },
    });
    expect(seenRequests).toEqual([
      "/admin/v1/libraries/library%2Funsafe%20id/sources/source%2Funsafe%20id/duplicate-reconciliation-plan?limit=5&offset=10",
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    const fallbackResult = await fallbackSource.loadSourceDuplicateReconciliationPlan?.(
      "library-anime",
      "source-unknown-1",
    );

    expect(fallbackResult).toMatchObject({
      source: "mock",
      value: mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1"),
      error: expect.stringContaining("HTTP 503"),
    });
  });

  it("applies source duplicate reconciliation without mock success fallback", async () => {
    const seenRequests: Array<{ body: unknown; method: string; path: string }> = [];
    const liveSource = createAdminDataSource({
      fetcher: async (input: string | URL | Request, init?: RequestInit) => {
        const url = new URL(input.toString(), "http://127.0.0.1");
        seenRequests.push({
          body: init?.body ? JSON.parse(String(init.body)) : null,
          method: init?.method ?? "GET",
          path: url.pathname,
        });
        return Response.json(
          mockSourceDuplicateReconciliationApply(
            "library/unsafe id",
            "source/unsafe id",
            "duplicate/source id",
          ),
        );
      },
    });

    await expect(
      liveSource.applySourceDuplicateReconciliation?.(
        "library/unsafe id",
        "source/unsafe id",
        "duplicate/source id",
      ),
    ).resolves.toMatchObject({
      library_id: "library/unsafe id",
      source_id: "source/unsafe id",
      duplicate_source_id: "duplicate/source id",
      relationship_status: "suggested",
      applied_action: "suggest_relationship",
    });

    expect(seenRequests).toEqual([
      {
        path: "/admin/v1/libraries/library%2Funsafe%20id/sources/source%2Funsafe%20id/duplicate-reconciliation-apply",
        method: "POST",
        body: {
          duplicate_source_id: "duplicate/source id",
          expected_action: "suggest_relationship",
        },
      },
    ]);

    const fallbackSource = createAdminDataSource({
      fetcher: async () => new Response("offline", { status: 503 }),
    });

    await expect(
      fallbackSource.applySourceDuplicateReconciliation?.(
        "library-anime",
        "source-unknown-1",
        "source-unknown-2",
      ),
    ).rejects.toThrow("HTTP 503");
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
