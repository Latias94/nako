import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { TARU_ADMIN_ROUTES } from "./generated/contract";
import {
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
      [TARU_ADMIN_ROUTES.addons, mockAddons],
      [TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab"), mockAddonDetail],
      [TARU_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"), mockAddonHealth],
      [TARU_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"), mockAddonSurfaces],
      [TARU_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"), mockAddonInstallGuide],
      [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`, mockAddonTokens],
      [`${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`, mockAddonGrants],
      [TARU_ADMIN_ROUTES.acquisitionIntakeCandidates, mockAcquisitionIntakeCandidates],
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
    await expect(client.getAddons({ status: "enabled" })).resolves.toEqual(mockAddons);
    await expect(client.getAddonDetail("addon-subtitle-lab")).resolves.toEqual(mockAddonDetail);
    await expect(client.checkAddonHealth("addon-subtitle-lab")).resolves.toEqual(mockAddonHealth);
    await expect(client.getAddonSurfaces("addon-subtitle-lab")).resolves.toEqual(mockAddonSurfaces);
    await expect(client.getAddonInstallGuide("addon-subtitle-lab")).resolves.toEqual(mockAddonInstallGuide);
    await expect(client.getAddonTokens("addon-subtitle-lab")).resolves.toEqual(mockAddonTokens);
    await expect(client.getAddonGrants("addon-subtitle-lab")).resolves.toEqual(mockAddonGrants);
    await expect(
      client.getAcquisitionIntakeCandidates({ library_id: "library-anime", state: "ready" }),
    ).resolves.toEqual(mockAcquisitionIntakeCandidates);
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
      `${TARU_ADMIN_ROUTES.addons}?status=enabled`,
      TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab"),
      TARU_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"),
      TARU_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"),
      TARU_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"),
      `${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`,
      `${TARU_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`,
      `${TARU_ADMIN_ROUTES.acquisitionIntakeCandidates}?library_id=library-anime&state=ready`,
      TARU_ADMIN_ROUTES.events,
      TARU_ADMIN_ROUTES.jobs,
      TARU_ADMIN_ROUTES.playbackSessions,
      TARU_ADMIN_ROUTES.playbackRuntime,
      `${TARU_ADMIN_ROUTES.playbackSupport}?session_id=session-hls`,
      TARU_ADMIN_ROUTES.storageStaging,
      TARU_ADMIN_ROUTES.systemConfig,
    ]);
  });

  it("sends Addon lifecycle and diagnostic mutations through Admin-only routes", async () => {
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");
      if (url.pathname.endsWith("/diagnostics/resource-call")) {
        return Response.json(mockAddonDiagnostic);
      }
      return Response.json(mockAddonDetail);
    });
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(
      client.updateAddonStatus("addon-subtitle-lab", { status: "disabled" }),
    ).resolves.toEqual(mockAddonDetail);
    await expect(client.unregisterAddon("addon-subtitle-lab")).resolves.toEqual(mockAddonDetail);
    await expect(
      client.diagnoseAddonResourceCall("addon-subtitle-lab", {
        resource: "subtitle",
        payload: { safe_probe: true },
      }),
    ).resolves.toEqual(mockAddonDiagnostic);

    expect(fetcher.mock.calls).toMatchObject([
      [
        TARU_ADMIN_ROUTES.addonStatus.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "PATCH",
          body: JSON.stringify({ status: "disabled" }),
        },
      ],
      [
        TARU_ADMIN_ROUTES.addonUnregister.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        TARU_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "POST",
          body: JSON.stringify({ resource: "subtitle", payload: { safe_probe: true } }),
        },
      ],
    ]);
  });

  it("registers an Addon manifest through the Admin-only route as disabled by default", async () => {
    const fetcher = vi.fn(async () => Response.json(mockAddonDetail));
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.registerAddon(mockAddonDetail.addon.manifest)).resolves.toEqual(mockAddonDetail);

    expect(fetcher).toHaveBeenCalledWith(TARU_ADMIN_ROUTES.addons, {
      method: "POST",
      headers: {
        Authorization: "Bearer redacted-test-token",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        id: undefined,
        manifest: mockAddonDetail.addon.manifest,
        granted_scopes: [],
        status: "disabled",
      }),
    });
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
});
