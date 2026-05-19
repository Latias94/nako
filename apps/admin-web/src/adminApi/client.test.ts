import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { TARU_ADMIN_ROUTES } from "./generated/contract";
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

describe("AdminApiClient", () => {
  it("loads the overview through /admin/v1/overview with optional bearer auth", async () => {
    const fetcher = vi.fn(async () =>
      Response.json({
        ...mockOverview,
        status: "degraded",
      }),
    );
    const client = new AdminApiClient({
      baseUrl: "http://127.0.0.1:3000/",
      token: "redacted-test-token",
      fetcher,
    });

    const overview = await client.getOverview();

    expect(overview.status).toBe("degraded");
    expect(fetcher).toHaveBeenCalledWith(`http://127.0.0.1:3000${TARU_ADMIN_ROUTES.overview}`, {
      headers: {
        Authorization: "Bearer redacted-test-token",
      },
    });
  });

  it("reports non-successful Admin API responses", async () => {
    const fetcher = vi.fn(async () => new Response("not found", { status: 404 }));
    const client = new AdminApiClient({ fetcher });

    await expect(client.getOverview()).rejects.toThrow("HTTP 404");
  });

  it("loads existing Admin API read models through typed route methods", async () => {
    const responses = new Map<string, unknown>([
      [TARU_ADMIN_ROUTES.catalogGovernanceItems, mockCatalogGovernance],
      [TARU_ADMIN_ROUTES.events, mockEvents],
      [TARU_ADMIN_ROUTES.jobs, mockJobs],
      [TARU_ADMIN_ROUTES.playbackSessions, mockPlaybackSessions],
      [TARU_ADMIN_ROUTES.playbackRuntime, mockPlaybackRuntime],
      [TARU_ADMIN_ROUTES.storageStaging, mockStorageStaging],
      [TARU_ADMIN_ROUTES.systemConfig, mockSystemConfig],
    ]);
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");
      const response = responses.get(url.pathname);

      if (!response) {
        return new Response("not found", { status: 404 });
      }

      return Response.json(response);
    });
    const client = new AdminApiClient({ fetcher });

    await expect(client.getCatalogGovernanceItems()).resolves.toEqual(mockCatalogGovernance);
    await expect(client.getEvents()).resolves.toEqual(mockEvents);
    await expect(client.getJobs()).resolves.toEqual(mockJobs);
    await expect(client.getPlaybackSessions()).resolves.toEqual(mockPlaybackSessions);
    await expect(client.getPlaybackRuntime()).resolves.toEqual(mockPlaybackRuntime);
    await expect(client.getStorageStaging()).resolves.toEqual(mockStorageStaging);
    await expect(client.getSystemConfig()).resolves.toEqual(mockSystemConfig);

    expect(fetcher.mock.calls.map(([input]) => input.toString())).toEqual([
      TARU_ADMIN_ROUTES.catalogGovernanceItems,
      TARU_ADMIN_ROUTES.events,
      TARU_ADMIN_ROUTES.jobs,
      TARU_ADMIN_ROUTES.playbackSessions,
      TARU_ADMIN_ROUTES.playbackRuntime,
      TARU_ADMIN_ROUTES.storageStaging,
      TARU_ADMIN_ROUTES.systemConfig,
    ]);
  });
});
