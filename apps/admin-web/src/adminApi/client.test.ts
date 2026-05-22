import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { TARU_ADMIN_ROUTES } from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
  mockCatalogGovernance,
  mockEvents,
  mockGeneratedArtifactProposals,
  mockJobs,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackSessions,
  mockPlaybackSupport,
  mockStorageStaging,
  mockSystemConfig,
  mockWatchFolderDiscovery,
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
      [TARU_ADMIN_ROUTES.acquisitionIntakeCandidates, mockAcquisitionIntakeCandidates],
      [TARU_ADMIN_ROUTES.generatedArtifactProposals, mockGeneratedArtifactProposals],
      [TARU_ADMIN_ROUTES.events, mockEvents],
      [TARU_ADMIN_ROUTES.jobs, mockJobs],
      [TARU_ADMIN_ROUTES.playbackSessions, mockPlaybackSessions],
      [TARU_ADMIN_ROUTES.playbackRuntime, mockPlaybackRuntime],
      [TARU_ADMIN_ROUTES.playbackSupport, mockPlaybackSupport],
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
    await expect(
      client.getAcquisitionIntakeCandidates({ library_id: "library-anime", state: "ready" }),
    ).resolves.toEqual(mockAcquisitionIntakeCandidates);
    await expect(client.getGeneratedArtifactProposals({ limit: 5 })).resolves.toEqual(
      mockGeneratedArtifactProposals,
    );
    await expect(client.getEvents()).resolves.toEqual(mockEvents);
    await expect(client.getJobs()).resolves.toEqual(mockJobs);
    await expect(client.getPlaybackSessions()).resolves.toEqual(mockPlaybackSessions);
    await expect(client.getPlaybackRuntime()).resolves.toEqual(mockPlaybackRuntime);
    await expect(client.getPlaybackSupport({ session_id: "session-hls" })).resolves.toEqual(
      mockPlaybackSupport,
    );
    await expect(client.getStorageStaging()).resolves.toEqual(mockStorageStaging);
    await expect(client.getSystemConfig()).resolves.toEqual(mockSystemConfig);

    expect(fetcher.mock.calls.map(([input]) => input.toString())).toEqual([
      TARU_ADMIN_ROUTES.catalogGovernanceItems,
      `${TARU_ADMIN_ROUTES.acquisitionIntakeCandidates}?library_id=library-anime&state=ready`,
      `${TARU_ADMIN_ROUTES.generatedArtifactProposals}?limit=5`,
      TARU_ADMIN_ROUTES.events,
      TARU_ADMIN_ROUTES.jobs,
      TARU_ADMIN_ROUTES.playbackSessions,
      TARU_ADMIN_ROUTES.playbackRuntime,
      `${TARU_ADMIN_ROUTES.playbackSupport}?session_id=session-hls`,
      TARU_ADMIN_ROUTES.storageStaging,
      TARU_ADMIN_ROUTES.systemConfig,
    ]);
  });

  it("posts watch-folder discovery requests through the Admin-only route", async () => {
    const fetcher = vi.fn(async () => Response.json(mockWatchFolderDiscovery));
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(
      client.discoverWatchFolderCandidates({
        target_library_id: "library-anime",
        root_uri: "local:///watch",
        max_depth: 4,
      }),
    ).resolves.toEqual(mockWatchFolderDiscovery);

    expect(fetcher).toHaveBeenCalledWith(
      TARU_ADMIN_ROUTES.acquisitionIntakeWatchFolderDiscovery,
      {
        method: "POST",
        headers: {
          Authorization: "Bearer redacted-test-token",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          target_library_id: "library-anime",
          root_uri: "local:///watch",
          max_depth: 4,
        }),
      },
    );
  });

  it("posts addon runtime readiness diagnostics through the Admin-only route", async () => {
    const response = {
      addon_id: "addon/with space",
      manifest_id: "taru.metadata",
      readiness: {
        status: "degraded",
        reason: "missing_secret_reference",
        checks: [
          {
            name: "secret_references",
            status: "degraded",
            reason: "missing_secret_reference",
            safe_error_code: "missing_secret_reference",
          },
        ],
      },
    };
    const fetcher = vi.fn(async () => Response.json(response));
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.getAddonRuntimeReadiness("addon/with space")).resolves.toEqual(response);

    expect(fetcher).toHaveBeenCalledWith(
      "/admin/v1/addons/addon%2Fwith%20space/runtime-readiness",
      {
        method: "POST",
        headers: {
          Authorization: "Bearer redacted-test-token",
          "Content-Type": "application/json",
        },
        body: JSON.stringify({}),
      },
    );
  });
});
