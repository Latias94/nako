import { describe, expect, it } from "vitest";

import { createAdminDataSource } from "./dataSource";
import { TARU_ADMIN_ROUTES } from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
  mockAddonDetail,
  mockAddonDiagnostic,
  mockAddonGrants,
  mockAddonHealth,
  mockAddons,
  mockAddonSurfaces,
  mockAddonTokens,
  mockCatalogGovernance,
  mockEvents,
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
        [TARU_ADMIN_ROUTES.overview]: mockOverview,
        [TARU_ADMIN_ROUTES.addons]: mockAddons,
        [TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDetail,
        [TARU_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: mockAddonHealth,
        [TARU_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")]: mockAddonSurfaces,
        [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: mockAddonTokens,
        [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: mockAddonGrants,
        [TARU_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDiagnostic,
        [TARU_ADMIN_ROUTES.acquisitionIntakeCandidates]: mockAcquisitionIntakeCandidates,
        [TARU_ADMIN_ROUTES.catalogGovernanceItems]: mockCatalogGovernance,
        [TARU_ADMIN_ROUTES.events]: mockEvents,
        [TARU_ADMIN_ROUTES.jobs]: mockJobs,
        [TARU_ADMIN_ROUTES.playbackSessions]: mockPlaybackSessions,
        [TARU_ADMIN_ROUTES.playbackRuntime]: mockPlaybackRuntime,
        [TARU_ADMIN_ROUTES.storageStaging]: mockStorageStaging,
        [TARU_ADMIN_ROUTES.systemConfig]: mockSystemConfig,
      }),
    });

    const data = await dataSource.load();

    expect(data.sources.overview).toBe("live");
    expect(data.sources.addons).toBe("live");
    expect(data.sources.addonHealth).toBe("live");
    expect(data.sources.addonSurfaces).toBe("live");
    expect(data.sources.addonTokens).toBe("live");
    expect(data.sources.addonGrants).toBe("live");
    expect(data.sources.acquisitionIntake).toBe("live");
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
    expect(data.addons.tokens[0].tokenPrefix).toBe("taru_at_subtitle");
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
        [TARU_ADMIN_ROUTES.overview]: mockOverview,
        [TARU_ADMIN_ROUTES.addons]: new Response("offline", { status: 503 }),
        [TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDetail,
        [TARU_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: mockAddonHealth,
        [TARU_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab")]: mockAddonSurfaces,
        [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`]: mockAddonTokens,
        [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`]: mockAddonGrants,
        [TARU_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: mockAddonDiagnostic,
        [TARU_ADMIN_ROUTES.acquisitionIntakeCandidates]: mockAcquisitionIntakeCandidates,
        [TARU_ADMIN_ROUTES.catalogGovernanceItems]: mockCatalogGovernance,
        [TARU_ADMIN_ROUTES.events]: mockEvents,
        [TARU_ADMIN_ROUTES.jobs]: new Response("offline", { status: 503 }),
        [TARU_ADMIN_ROUTES.playbackSessions]: mockPlaybackSessions,
        [TARU_ADMIN_ROUTES.playbackRuntime]: mockPlaybackRuntime,
        [TARU_ADMIN_ROUTES.storageStaging]: mockStorageStaging,
        [TARU_ADMIN_ROUTES.systemConfig]: mockSystemConfig,
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

  it("exposes safe Addon action methods through the data-source seam", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        [TARU_ADMIN_ROUTES.addonStatus.replace(":addon_id", "addon-subtitle-lab")]: {
          ...mockAddonDetail,
          addon: {
            ...mockAddonDetail.addon,
            summary: {
              ...mockAddonDetail.addon.summary,
              status: "disabled",
            },
          },
        },
        [TARU_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab")]: {
          ...mockAddonHealth,
          status: "degraded",
          safe_error_code: "latency_budget_exceeded",
        },
        [TARU_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab")]: {
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
