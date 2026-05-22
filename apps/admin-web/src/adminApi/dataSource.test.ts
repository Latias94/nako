import { describe, expect, it } from "vitest";

import { createAdminDataSource } from "./dataSource";
import { TARU_ADMIN_ROUTES } from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
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
    expect(data.sources.jobs).toBe("mock");
    expect(data.errors.jobs).toContain("HTTP 503");
    expect(data.jobs[0].id).toBe("job-scan");
    expect(data.sources.playbackRuntime).toBe("live");
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
