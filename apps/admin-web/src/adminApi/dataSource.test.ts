import { describe, expect, it } from "vitest";

import { createAdminDataSource } from "./dataSource";
import {
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
        "/admin/v1/overview": mockOverview,
        "/admin/v1/catalog/governance/items": mockCatalogGovernance,
        "/admin/v1/events": mockEvents,
        "/admin/v1/jobs": mockJobs,
        "/admin/v1/playback/sessions": mockPlaybackSessions,
        "/admin/v1/playback/runtime": mockPlaybackRuntime,
        "/admin/v1/storage/staging": mockStorageStaging,
        "/admin/v1/system/config": mockSystemConfig,
      }),
    });

    const data = await dataSource.load();

    expect(data.sources.overview).toBe("live");
    expect(data.sources.jobs).toBe("live");
    expect(data.sources.playbackSessions).toBe("live");
    expect(data.sources.playbackRuntime).toBe("live");
    expect(data.sources.storageStaging).toBe("live");
    expect(data.sources.systemConfig).toBe("live");
    expect(data.jobs[0]).toMatchObject({
      kind: "library_scan",
      resourceClass: "library",
    });
    expect(data.playback.accelerators).toContainEqual({
      name: "nvenc",
      available: true,
    });
    expect(data.settings).toContainEqual({
      label: "Admin auth",
      value: "Auth configured",
    });
  });

  it("falls back per section when one Admin API read model fails", async () => {
    const dataSource = createAdminDataSource({
      fetcher: fetcherFor({
        "/admin/v1/overview": mockOverview,
        "/admin/v1/catalog/governance/items": mockCatalogGovernance,
        "/admin/v1/events": mockEvents,
        "/admin/v1/jobs": new Response("offline", { status: 503 }),
        "/admin/v1/playback/sessions": mockPlaybackSessions,
        "/admin/v1/playback/runtime": mockPlaybackRuntime,
        "/admin/v1/storage/staging": mockStorageStaging,
        "/admin/v1/system/config": mockSystemConfig,
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
