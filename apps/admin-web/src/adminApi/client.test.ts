import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";
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
    expect(fetcher).toHaveBeenCalledWith(`http://127.0.0.1:3000${NAKO_ADMIN_ROUTES.overview}`, {
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

  it("reports successful non-JSON responses before parsing", async () => {
    const fetcher = vi.fn(async () =>
      new Response("<!doctype html>", {
        headers: {
          "Content-Type": "text/html",
        },
      }),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(client.getOverview()).rejects.toThrow("non-JSON response");
  });

  it("does not call the browser default fetch as an unbound client method", async () => {
    const originalFetch = globalThis.fetch;
    Object.defineProperty(globalThis, "fetch", {
      configurable: true,
      writable: true,
      value(this: unknown) {
        if (this instanceof AdminApiClient) {
          throw new TypeError("Illegal invocation");
        }

        return Promise.resolve(Response.json(mockOverview));
      },
    });

    try {
      const client = new AdminApiClient();

      await expect(client.getOverview()).resolves.toEqual(mockOverview);
    } finally {
      Object.defineProperty(globalThis, "fetch", {
        configurable: true,
        writable: true,
        value: originalFetch,
      });
    }
  });

  it("loads existing Admin API read models through typed route methods", async () => {
    const responses = new Map<string, unknown>([
      [NAKO_ADMIN_ROUTES.catalogGovernanceItems, mockCatalogGovernance],
      [NAKO_ADMIN_ROUTES.addons, mockAddons],
      [NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab"), mockAddonDetail],
      [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"), mockAddonHealth],
      [NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"), mockAddonSurfaces],
      [NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"), mockAddonInstallGuide],
      [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`, mockAddonTokens],
      [`${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`, mockAddonGrants],
      [NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates, mockAcquisitionIntakeCandidates],
      [NAKO_ADMIN_ROUTES.generatedArtifactProposals, mockGeneratedArtifactProposals],
      [NAKO_ADMIN_ROUTES.events, mockEvents],
      [NAKO_ADMIN_ROUTES.jobs, mockJobs],
      [NAKO_ADMIN_ROUTES.playbackSessions, mockPlaybackSessions],
      [NAKO_ADMIN_ROUTES.playbackRuntime, mockPlaybackRuntime],
      [NAKO_ADMIN_ROUTES.playbackSupport, mockPlaybackSupport],
      [NAKO_ADMIN_ROUTES.storageStaging, mockStorageStaging],
      [NAKO_ADMIN_ROUTES.systemConfig, mockSystemConfig],
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
    await expect(client.getGeneratedArtifactProposals({ limit: 5 })).resolves.toEqual(
      mockGeneratedArtifactProposals,
    );
    await expect(client.getEvents()).resolves.toEqual(mockEvents);
    await expect(client.getJobs({ status: "failed", limit: 5 })).resolves.toEqual(mockJobs);
    await expect(client.getPlaybackSessions()).resolves.toEqual(mockPlaybackSessions);
    await expect(client.getPlaybackRuntime()).resolves.toEqual(mockPlaybackRuntime);
    await expect(client.getPlaybackSupport({ session_id: "session-hls" })).resolves.toEqual(
      mockPlaybackSupport,
    );
    await expect(client.getStorageStaging()).resolves.toEqual(mockStorageStaging);
    await expect(client.getSystemConfig()).resolves.toEqual(mockSystemConfig);

    expect(fetcher.mock.calls.map(([input]) => input.toString())).toEqual([
      NAKO_ADMIN_ROUTES.catalogGovernanceItems,
      `${NAKO_ADMIN_ROUTES.addons}?status=enabled`,
      NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab"),
      NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"),
      NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"),
      NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"),
      `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`,
      `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`,
      `${NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates}?library_id=library-anime&state=ready`,
      `${NAKO_ADMIN_ROUTES.generatedArtifactProposals}?limit=5`,
      NAKO_ADMIN_ROUTES.events,
      `${NAKO_ADMIN_ROUTES.jobs}?status=failed&limit=5`,
      NAKO_ADMIN_ROUTES.playbackSessions,
      NAKO_ADMIN_ROUTES.playbackRuntime,
      `${NAKO_ADMIN_ROUTES.playbackSupport}?session_id=session-hls`,
      NAKO_ADMIN_ROUTES.storageStaging,
      NAKO_ADMIN_ROUTES.systemConfig,
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
        NAKO_ADMIN_ROUTES.addonStatus.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "PATCH",
          body: JSON.stringify({ status: "disabled" }),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.addonUnregister.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.addonResourceCallDiagnostic.replace(":addon_id", "addon-subtitle-lab"),
        {
          method: "POST",
          body: JSON.stringify({ resource: "subtitle", payload: { safe_probe: true } }),
        },
      ],
    ]);
  });

  it("sends Addon token and grant mutations through Admin-only routes", async () => {
    const issuedToken = {
      token: mockAddonTokens.tokens[0],
      raw_token: "nako_at_one_time_raw_token",
    };
    const rotatedToken = {
      rotated: mockAddonTokens.tokens[0],
      token: {
        ...mockAddonTokens.tokens[0],
        id: "addon-token-rotated",
        token_prefix: "nako_at_rotated",
      },
      raw_token: "nako_at_rotated_one_time_raw_token",
    };
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");
      if (url.pathname.endsWith("/tokens")) {
        return Response.json(issuedToken);
      }
      if (url.pathname.endsWith("/rotate")) {
        return Response.json(rotatedToken);
      }
      if (url.pathname.endsWith("/revoke")) {
        return Response.json({ token: { ...mockAddonTokens.tokens[0], status: "revoked" } });
      }
      if (url.pathname.endsWith("/grants")) {
        return Response.json(mockAddonGrants);
      }
      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.issueAddonToken("addon-subtitle-lab", { label: "sidecar runtime" })).resolves.toEqual(
      issuedToken,
    );
    await expect(
      client.rotateAddonToken("addon-subtitle-lab", "addon-token-active", { label: "replacement" }),
    ).resolves.toEqual(rotatedToken);
    await expect(client.revokeAddonToken("addon-subtitle-lab", "addon-token-active")).resolves.toMatchObject({
      token: {
        status: "revoked",
      },
    });
    await expect(
      client.replaceAddonGrants("addon-subtitle-lab", {
        grants: [{ permission: "metadata_write", library_id: null }],
      }),
    ).resolves.toEqual(mockAddonGrants);

    expect(fetcher.mock.calls).toMatchObject([
      [
        `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens`,
        {
          method: "POST",
          body: JSON.stringify({ label: "sidecar runtime" }),
        },
      ],
      [
        `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens/addon-token-active/rotate`,
        {
          method: "POST",
          body: JSON.stringify({ label: "replacement" }),
        },
      ],
      [
        `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/tokens/addon-token-active/revoke`,
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        `${NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", "addon-subtitle-lab")}/grants`,
        {
          method: "PUT",
          body: JSON.stringify({ grants: [{ permission: "metadata_write", library_id: null }] }),
        },
      ],
    ]);
  });

  it("registers an Addon manifest through the Admin-only route as disabled by default", async () => {
    const fetcher = vi.fn(async () => Response.json(mockAddonDetail));
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.registerAddon(mockAddonDetail.addon.manifest)).resolves.toEqual(mockAddonDetail);

    expect(fetcher).toHaveBeenCalledWith(NAKO_ADMIN_ROUTES.addons, {
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
      NAKO_ADMIN_ROUTES.acquisitionIntakeWatchFolderDiscovery,
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
      manifest_id: "nako.metadata",
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

  it("posts addon routing-plan syncs through the Admin-only route", async () => {
    const response = {
      addon_id: "addon/with space",
      manifest_id: "nako.metadata",
      manifest_version: "0.1.0",
      manifest_fingerprint: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      executable: 1,
      deferred: 1,
      plans: [
        {
          declaration_kind: "task",
          declaration_id: "bulk-task",
          status: "executable",
          target: "addon_task_job",
          job_kind: "addon_task",
          required_scope_count: 1,
          filter_configured: false,
          timeout_ms: 30000,
          max_attempts: 2,
        },
      ],
    };
    const fetcher = vi.fn(async () => Response.json(response));
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.getAddonRoutingPlans("addon/with space")).resolves.toEqual(response);

    expect(fetcher).toHaveBeenCalledWith("/admin/v1/addons/addon%2Fwith%20space/routing-plans", {
      method: "POST",
      headers: {
        Authorization: "Bearer redacted-test-token",
        "Content-Type": "application/json",
      },
      body: JSON.stringify({}),
    });
  });
});
