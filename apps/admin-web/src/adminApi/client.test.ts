import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { NAKO_ADMIN_ROUTES } from "./generated/contract";
import type {
  AdminAddonInstallGuidePreviewRequest,
  AdminAddonInstallGuidePreviewResponse,
} from "./generated/contract";
import {
  mockAcquisitionIntakeCandidates,
  mockAccessSummary,
  mockAccessInvitationCreated,
  mockAccessInvitationRevoked,
  mockAccessInvitations,
  mockAddonDetail,
  mockAddonDiagnostic,
  mockAddonEventDeliveryAttempts,
  mockAddonEventDispatch,
  mockAddonEventReplay,
  mockAddonEventSchedulerWork,
  mockAddonGrants,
  mockAddonHealth,
  mockAddonInstallGuide,
  mockAddons,
  mockAddonTaskRunRetryResponse,
  mockAddonTaskRuns,
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
  mockJobCancelRequestResponse,
  mockJobs,
  mockLibraryMetadataProfile,
  mockManagedArtworkArtifactLifecycle,
  mockManagedArtworkArtifactRemediationPlan,
  mockManagedArtworkArtifactStorageDrift,
  mockMetadataRawCacheSettings,
  mockOverview,
  mockPlaybackRuntime,
  mockPlaybackRuntimeSettings,
  mockPlaybackSessions,
  mockPlaybackSupport,
  mockPublicCatalogItems,
  mockPublicCatalogSearch,
  mockPublicItemDetail,
  mockPublicSourceProbe,
  mockSourceDuplicateReconciliationApply,
  mockSourceDuplicateReconciliationPlan,
  mockStorageStaging,
  mockSystemConfig,
  mockVfsCacheRepairAutomationEnqueueResponse,
  mockVfsCacheRepairAutomationPlan,
  mockWatchFolderDiscovery,
} from "./mockData";

const TEST_ADDON_ID = "addon-subtitle-lab";
const TEST_ADDON_TOKEN_ID = "addon-token-active";

function addonDetailPath(addonId = TEST_ADDON_ID) {
  return NAKO_ADMIN_ROUTES.addonDetail.replace(":addon_id", addonId);
}

function addonTokensPath(addonId = TEST_ADDON_ID) {
  return NAKO_ADMIN_ROUTES.addonTokens.replace("{addon_id}", addonId);
}

function addonTokenRotatePath(addonId = TEST_ADDON_ID, tokenId = TEST_ADDON_TOKEN_ID) {
  return NAKO_ADMIN_ROUTES.addonTokenRotate
    .replace("{addon_id}", addonId)
    .replace("{token_id}", tokenId);
}

function addonTokenRevokePath(addonId = TEST_ADDON_ID, tokenId = TEST_ADDON_TOKEN_ID) {
  return NAKO_ADMIN_ROUTES.addonTokenRevoke
    .replace("{addon_id}", addonId)
    .replace("{token_id}", tokenId);
}

function addonGrantsPath(addonId = TEST_ADDON_ID) {
  return NAKO_ADMIN_ROUTES.addonGrants.replace("{addon_id}", addonId);
}

function addonTaskRunsPath(addonId = TEST_ADDON_ID) {
  return NAKO_ADMIN_ROUTES.addonTaskRuns.replace("{addon_id}", addonId);
}

function addonTaskRunPath(addonId = TEST_ADDON_ID, jobId = "job-addon-task-run-failed") {
  return NAKO_ADMIN_ROUTES.addonTaskRun
    .replace("{addon_id}", addonId)
    .replace("{job_id}", jobId);
}

function addonTaskRunRetryPath(addonId = TEST_ADDON_ID, jobId = "job-addon-task-run-failed") {
  return NAKO_ADMIN_ROUTES.addonTaskRunRetry
    .replace("{addon_id}", addonId)
    .replace("{job_id}", jobId);
}

function eventPath(route: string, eventId = "event-webhook") {
  return route.replace("{event_id}", eventId);
}

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
      [NAKO_ADMIN_ROUTES.accessSummary, mockAccessSummary],
      [
        NAKO_ADMIN_ROUTES.catalogGovernanceItemDetail.replace("{item_id}", "item-candidate"),
        mockCatalogGovernanceItemDetail("item-candidate"),
      ],
      [NAKO_ADMIN_ROUTES.addons, mockAddons],
      [addonDetailPath(), mockAddonDetail],
      [NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"), mockAddonHealth],
      [NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"), mockAddonSurfaces],
      [NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"), mockAddonInstallGuide],
      [addonTokensPath(), mockAddonTokens],
      [addonGrantsPath(), mockAddonGrants],
      [addonTaskRunsPath(), mockAddonTaskRuns],
      [NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates, mockAcquisitionIntakeCandidates],
      [NAKO_ADMIN_ROUTES.generatedArtifactProposals, mockGeneratedArtifactProposals],
      [NAKO_ADMIN_ROUTES.events, mockEvents],
      [NAKO_ADMIN_ROUTES.jobs, mockJobs],
      [
        NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace("{library_id}", "library-anime"),
        mockLibraryMetadataProfile("library-anime"),
      ],
      [NAKO_ADMIN_ROUTES.playbackSessions, mockPlaybackSessions],
      [NAKO_ADMIN_ROUTES.playbackRuntime, mockPlaybackRuntime],
      [NAKO_ADMIN_ROUTES.settingsPlaybackRuntime, mockPlaybackRuntimeSettings],
      [NAKO_ADMIN_ROUTES.playbackSupport, mockPlaybackSupport],
      [NAKO_ADMIN_ROUTES.storageStaging, mockStorageStaging],
      [NAKO_ADMIN_ROUTES.systemConfig, mockSystemConfig],
      [NAKO_ADMIN_ROUTES.settingsMetadataRawCache, mockMetadataRawCacheSettings],
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

    await expect(
      client.getCatalogGovernanceItems({
        library_id: "library-anime",
        max_confidence_milli: 500,
        limit: 5,
        offset: 0,
      }),
    ).resolves.toEqual(mockCatalogGovernance);
    await expect(client.getAccessSummary()).resolves.toEqual(mockAccessSummary);
    await expect(client.getCatalogGovernanceItemDetail("item-candidate")).resolves.toEqual(
      mockCatalogGovernanceItemDetail("item-candidate"),
    );
    await expect(client.getAddons({ status: "enabled" })).resolves.toEqual(mockAddons);
    await expect(client.getAddonDetail("addon-subtitle-lab")).resolves.toEqual(mockAddonDetail);
    await expect(client.checkAddonHealth("addon-subtitle-lab")).resolves.toEqual(mockAddonHealth);
    await expect(client.getAddonSurfaces("addon-subtitle-lab")).resolves.toEqual(mockAddonSurfaces);
    await expect(client.getAddonInstallGuide("addon-subtitle-lab")).resolves.toEqual(mockAddonInstallGuide);
    await expect(client.getAddonTokens("addon-subtitle-lab")).resolves.toEqual(mockAddonTokens);
    await expect(client.getAddonGrants("addon-subtitle-lab")).resolves.toEqual(mockAddonGrants);
    await expect(client.getAddonTaskRuns("addon-subtitle-lab", { limit: 5 })).resolves.toEqual(
      mockAddonTaskRuns,
    );
    await expect(
      client.getAcquisitionIntakeCandidates({ library_id: "library-anime", state: "ready" }),
    ).resolves.toEqual(mockAcquisitionIntakeCandidates);
    await expect(client.getGeneratedArtifactProposals({ limit: 5 })).resolves.toEqual(
      mockGeneratedArtifactProposals,
    );
    await expect(client.getEvents()).resolves.toEqual(mockEvents);
    await expect(client.getJobs({ status: "failed", limit: 5 })).resolves.toEqual(mockJobs);
    await expect(client.getLibraryMetadataProfile("library-anime")).resolves.toEqual(
      mockLibraryMetadataProfile("library-anime"),
    );
    await expect(
      client.getPlaybackSessions({
        source_id: "source-hls",
        state: "running",
        limit: 5,
        offset: 0,
      }),
    ).resolves.toEqual(mockPlaybackSessions);
    await expect(client.getPlaybackRuntime()).resolves.toEqual(mockPlaybackRuntime);
    await expect(client.getPlaybackRuntimeSettings()).resolves.toEqual(
      mockPlaybackRuntimeSettings,
    );
    await expect(client.getPlaybackSupport({ session_id: "session-hls" })).resolves.toEqual(
      mockPlaybackSupport,
    );
    await expect(
      client.getStorageStaging({
        purpose: "ffmpeg_input",
        state: "ready",
        limit: 5,
        offset: 0,
      }),
    ).resolves.toEqual(mockStorageStaging);
    await expect(client.getSystemConfig()).resolves.toEqual(mockSystemConfig);
    await expect(client.getMetadataRawCacheSettings()).resolves.toEqual(
      mockMetadataRawCacheSettings,
    );

    expect(fetcher.mock.calls.map(([input]) => input.toString())).toEqual([
      `${NAKO_ADMIN_ROUTES.catalogGovernanceItems}?library_id=library-anime&max_confidence_milli=500&limit=5&offset=0`,
      NAKO_ADMIN_ROUTES.accessSummary,
      NAKO_ADMIN_ROUTES.catalogGovernanceItemDetail.replace("{item_id}", "item-candidate"),
      `${NAKO_ADMIN_ROUTES.addons}?status=enabled`,
      addonDetailPath(),
      NAKO_ADMIN_ROUTES.addonHealthCheck.replace(":addon_id", "addon-subtitle-lab"),
      NAKO_ADMIN_ROUTES.addonSurfaces.replace(":addon_id", "addon-subtitle-lab"),
      NAKO_ADMIN_ROUTES.addonInstallGuide.replace(":addon_id", "addon-subtitle-lab"),
      addonTokensPath(),
      addonGrantsPath(),
      `${addonTaskRunsPath()}?limit=5`,
      `${NAKO_ADMIN_ROUTES.acquisitionIntakeCandidates}?library_id=library-anime&state=ready`,
      `${NAKO_ADMIN_ROUTES.generatedArtifactProposals}?limit=5`,
      NAKO_ADMIN_ROUTES.events,
      `${NAKO_ADMIN_ROUTES.jobs}?status=failed&limit=5`,
      NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace("{library_id}", "library-anime"),
      `${NAKO_ADMIN_ROUTES.playbackSessions}?source_id=source-hls&state=running&limit=5&offset=0`,
      NAKO_ADMIN_ROUTES.playbackRuntime,
      NAKO_ADMIN_ROUTES.settingsPlaybackRuntime,
      `${NAKO_ADMIN_ROUTES.playbackSupport}?session_id=session-hls`,
      `${NAKO_ADMIN_ROUTES.storageStaging}?purpose=ffmpeg_input&state=ready&limit=5&offset=0`,
      NAKO_ADMIN_ROUTES.systemConfig,
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
    ]);
  });

  it("updates metadata raw cache settings through the Admin settings route", async () => {
    const fetcher = vi.fn(async () =>
      Response.json({
        ...mockMetadataRawCacheSettings,
        retention_ms: 3_600_000,
        cleanup_on_startup: false,
        source: "admin",
        effect: "requires_restart",
        updated_at_ms: 1779700000000,
      }),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.updateMetadataRawCacheSettings({
        retention_ms: 3_600_000,
        cleanup_on_startup: false,
      }),
    ).resolves.toMatchObject({
      retention_ms: 3_600_000,
      cleanup_on_startup: false,
      source: "admin",
      effect: "requires_restart",
    });

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.settingsMetadataRawCache,
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({
          retention_ms: 3_600_000,
          cleanup_on_startup: false,
        }),
      }),
    );
  });

  it("updates playback runtime settings through the generated Admin settings route", async () => {
    const nextSettings = {
      ...mockPlaybackRuntimeSettings.settings,
      cpu_concurrency: 3,
      staging_cleanup_on_startup: false,
    };
    const fetcher = vi.fn(async () =>
      Response.json({
        ...mockPlaybackRuntimeSettings,
        settings: nextSettings,
        source: "admin",
        effect: "requires_restart",
        updated_at_ms: 1779700000000,
      }),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.updatePlaybackRuntimeSettings({ settings: nextSettings }),
    ).resolves.toMatchObject({
      settings: nextSettings,
      source: "admin",
      effect: "requires_restart",
    });

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.settingsPlaybackRuntime,
      expect.objectContaining({
        method: "PUT",
        body: JSON.stringify({ settings: nextSettings }),
      }),
    );
  });

  it("previews an Addon install guide through the generated Admin route", async () => {
    const previewRequest: AdminAddonInstallGuidePreviewRequest = {
      descriptor: {
        manifest: mockAddonDetail.addon.manifest,
        runtime: {
          kind: "http_sidecar",
          image: "ghcr.io/nako/subtitle-lab:0.3.0",
        },
        secret_reference_bindings: [
          {
            field_id: "subtitle-provider-key",
            secret_ref: "secret-reference:subtitle-provider-key",
          },
        ],
        install_notes: ["Use secret references only."],
      },
    };
    const previewResponse: AdminAddonInstallGuidePreviewResponse = {
      guide: {
        manifest_id: "dev.nako.subtitle-lab",
        addon_name: "Subtitle Lab",
        protocol_version: "2026-05-15",
        runtime_kind: "http_sidecar",
        runtime_reference: {
          kind: "image",
          value: "ghcr.io/nako/subtitle-lab:0.3.0",
        },
        base_url_scheme: "http",
        base_url_configured: true,
        declared_resources: ["subtitle", "metadata"],
        declared_scopes: ["subtitle_read", "item_metadata_read"],
        required_secret_fields: [
          {
            id: "subtitle-provider-key",
            label: "Provider API key",
            required: false,
            provided: true,
          },
        ],
        provided_secret_refs: ["secret-reference:subtitle-provider-key"],
        missing_required_secret_fields: [],
        has_configuration_schema: false,
        entry_point_count: 1,
        hosted_page_count: 0,
        task_count: 0,
        event_subscription_count: 0,
        install_steps: [
          {
            kind: "run_sidecar",
            summary: "Run the Addon Sidecar outside Nako.",
          },
        ],
      },
    };
    const fetcher = vi.fn(async () => Response.json(previewResponse));
    const client = new AdminApiClient({ fetcher });

    await expect(client.previewAddonInstallGuide(previewRequest)).resolves.toEqual(
      previewResponse,
    );

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.addonInstallGuidePreview,
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify(previewRequest),
      }),
    );
    const responseText = JSON.stringify(previewResponse);
    expect(responseText).not.toContain("raw-addon-secret");
    expect(responseText).not.toContain("file://");
    expect(responseText).not.toContain("F:\\");
  });

  it("uses generated Access invitation routes for list, create, and revoke", async () => {
    const created = mockAccessInvitationCreated("invitation/unsafe id");
    const revoked = mockAccessInvitationRevoked("invitation/unsafe id");
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (init?.method === "POST" && url.pathname.endsWith("/revoke")) {
        return Response.json(revoked);
      }
      if (init?.method === "POST") {
        return Response.json(created);
      }

      return Response.json(mockAccessInvitations);
    });
    const client = new AdminApiClient({ fetcher });

    await expect(client.getAccessInvitations({ limit: 5, offset: 10 })).resolves.toEqual(
      mockAccessInvitations,
    );
    await expect(
      client.createAccessInvitation({
        email_or_username: "invitee@example.test",
        roles: ["viewer"],
        expires_in_ms: 3_600_000,
      }),
    ).resolves.toEqual(created);
    await expect(client.revokeAccessInvitation("invitation/unsafe id")).resolves.toEqual(
      revoked,
    );

    expect(fetcher.mock.calls).toMatchObject([
      [`${NAKO_ADMIN_ROUTES.accessInvitations}?limit=5&offset=10`, {}],
      [
        NAKO_ADMIN_ROUTES.accessInvitations,
        {
          method: "POST",
          body: JSON.stringify({
            email_or_username: "invitee@example.test",
            roles: ["viewer"],
            expires_in_ms: 3_600_000,
          }),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.accessInvitationRevoke.replace(
          "{invitation_id}",
          encodeURIComponent("invitation/unsafe id"),
        ),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
    ]);
  });

  it("posts Catalog Governance Provider Mapping review-plan decisions through Admin-only routes", async () => {
    const fetcher = vi.fn(async () =>
      Response.json(
        mockCatalogGovernanceProviderMappingReviewPlan(
          "item/unsafe id",
          "mapping/unsafe id",
          "reject",
        ),
      ),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.planCatalogGovernanceProviderMappingReview(
        "item/unsafe id",
        "mapping/unsafe id",
        "reject",
      ),
    ).resolves.toEqual(
      mockCatalogGovernanceProviderMappingReviewPlan(
        "item/unsafe id",
        "mapping/unsafe id",
        "reject",
      ),
    );

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReviewPlan
        .replace("{item_id}", encodeURIComponent("item/unsafe id"))
        .replace("{mapping_id}", encodeURIComponent("mapping/unsafe id")),
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ decision: "reject" }),
      }),
    );
  });

  it("posts Catalog Governance Provider Mapping review mutations through Admin-only routes", async () => {
    const fetcher = vi.fn(async () =>
      Response.json(
        mockCatalogGovernanceProviderMappingReviewResponse(
          "item/unsafe id",
          "mapping/unsafe id",
          "accept",
        ),
      ),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.reviewCatalogGovernanceProviderMapping(
        "item/unsafe id",
        "mapping/unsafe id",
        "accept",
      ),
    ).resolves.toEqual(
      mockCatalogGovernanceProviderMappingReviewResponse(
        "item/unsafe id",
        "mapping/unsafe id",
        "accept",
      ),
    );

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.catalogGovernanceProviderMappingReview
        .replace("{item_id}", encodeURIComponent("item/unsafe id"))
        .replace("{mapping_id}", encodeURIComponent("mapping/unsafe id")),
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ decision: "accept" }),
      }),
    );
  });

  it("posts Generated Artifact review-plan decisions through Admin-only routes", async () => {
    const fetcher = vi.fn(async () => Response.json(mockGeneratedArtifactReviewPlan("artifact/unsafe id", "reject")));
    const client = new AdminApiClient({ fetcher });

    await expect(client.planGeneratedArtifactReview("artifact/unsafe id", "reject")).resolves.toEqual(
      mockGeneratedArtifactReviewPlan("artifact/unsafe id", "reject"),
    );

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.generatedArtifactReviewPlan.replace(
        "{artifact_id}",
        encodeURIComponent("artifact/unsafe id"),
      ),
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ decision: "reject" }),
      }),
    );
  });

  it("posts Generated Artifact review decisions through Admin-only routes", async () => {
    const fetcher = vi.fn(async () => Response.json(mockGeneratedArtifactReviewResponse("artifact/unsafe id", "accept")));
    const client = new AdminApiClient({ fetcher });

    await expect(client.reviewGeneratedArtifact("artifact/unsafe id", "accept")).resolves.toEqual(
      mockGeneratedArtifactReviewResponse("artifact/unsafe id", "accept"),
    );

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.generatedArtifactReview.replace(
        "{artifact_id}",
        encodeURIComponent("artifact/unsafe id"),
      ),
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({ decision: "accept" }),
      }),
    );
  });

  it("uses generated item artwork routes for gallery, select, and unpublish", async () => {
    const gallery = {
      item_id: "item/unsafe id",
      summary: { candidates: 1, artifacts: 1, selected: 1 },
      candidates: [],
      artifacts: [],
      selected: [],
      page: { limit: 5, offset: 10, returned: 0 },
    };
    const selected = {
      selected_artwork: {
        id: "selected-artwork-1",
        library_id: "library-anime",
        item_id: "item/unsafe id",
        kind: "poster",
        artifact_id: "artifact/unsafe id",
        created_at: "2026-05-25T10:00:00Z",
        updated_at: "2026-05-25T10:00:00Z",
      },
      image: {
        id: "selected-artwork-1",
        owner: { item_id: "item/unsafe id" },
        kind: "poster",
        url: "/images/selected-artwork-1",
        width: 800,
        height: 1200,
        language: null,
        media_type: "image/png",
        etag: null,
      },
      changed: true,
    };
    const unpublished = {
      item_id: "item/unsafe id",
      kind: "poster",
      changed: true,
      unpublished: {
        selected_artwork: selected.selected_artwork,
        previous_image: selected.image,
      },
    };
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");
      if (url.pathname.endsWith("/select")) {
        return Response.json(selected);
      }
      if (url.pathname.endsWith("/selection")) {
        return Response.json(unpublished);
      }
      return Response.json(gallery);
    });
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.getItemArtworkGallery("item/unsafe id", { limit: 5, offset: 10 }),
    ).resolves.toEqual(gallery);
    await expect(
      client.selectItemArtwork("item/unsafe id", "poster", "artifact/unsafe id"),
    ).resolves.toEqual(selected);
    await expect(
      client.unpublishItemArtwork("item/unsafe id", "poster"),
    ).resolves.toEqual(unpublished);

    expect(fetcher.mock.calls).toMatchObject([
      [
        `${NAKO_ADMIN_ROUTES.itemArtworkGallery.replace(
          "{item_id}",
          encodeURIComponent("item/unsafe id"),
        )}?limit=5&offset=10`,
        {},
      ],
      [
        NAKO_ADMIN_ROUTES.itemArtworkSelect
          .replace("{item_id}", encodeURIComponent("item/unsafe id"))
          .replace("{kind}", "poster"),
        {
          method: "POST",
          body: JSON.stringify({ artifact_id: "artifact/unsafe id" }),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.itemArtworkSelection
          .replace("{item_id}", encodeURIComponent("item/unsafe id"))
          .replace("{kind}", "poster"),
        {
          method: "DELETE",
        },
      ],
    ]);
  });

  it("uses generated Managed Artwork maintenance read routes with query params", async () => {
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (url.pathname === NAKO_ADMIN_ROUTES.managedArtworkArtifactLifecycle) {
        return Response.json(mockManagedArtworkArtifactLifecycle);
      }
      if (url.pathname === NAKO_ADMIN_ROUTES.managedArtworkArtifactStorageDrift) {
        return Response.json(mockManagedArtworkArtifactStorageDrift);
      }
      if (url.pathname === NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediationPlan) {
        return Response.json(mockManagedArtworkArtifactRemediationPlan);
      }

      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.getManagedArtworkArtifactLifecycle({
        cleanup_candidates_only: true,
        limit: 5,
        offset: 10,
      }),
    ).resolves.toEqual(mockManagedArtworkArtifactLifecycle);
    await expect(
      client.getManagedArtworkArtifactStorageDrift({
        file_scan_limit: 50,
        limit: 5,
        offset: 10,
      }),
    ).resolves.toEqual(mockManagedArtworkArtifactStorageDrift);
    await expect(
      client.getManagedArtworkArtifactRemediationPlan({
        file_scan_limit: 50,
        limit: 5,
        offset: 10,
      }),
    ).resolves.toEqual(mockManagedArtworkArtifactRemediationPlan);

    expect(fetcher.mock.calls).toMatchObject([
      [
        `${NAKO_ADMIN_ROUTES.managedArtworkArtifactLifecycle}?cleanup_candidates_only=true&limit=5&offset=10`,
        {},
      ],
      [
        `${NAKO_ADMIN_ROUTES.managedArtworkArtifactStorageDrift}?file_scan_limit=50&limit=5&offset=10`,
        {},
      ],
      [
        `${NAKO_ADMIN_ROUTES.managedArtworkArtifactRemediationPlan}?file_scan_limit=50&limit=5&offset=10`,
        {},
      ],
    ]);
  });

  it("uses generated source duplicate reconciliation routes for plan and apply", async () => {
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (init?.method === "POST") {
        return Response.json(
          mockSourceDuplicateReconciliationApply(
            "library/unsafe id",
            "source/unsafe id",
            "duplicate/source id",
          ),
        );
      }

      expect(url.search).toBe("?limit=5&offset=10");
      return Response.json(
        mockSourceDuplicateReconciliationPlan("library/unsafe id", "source/unsafe id"),
      );
    });
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.getSourceDuplicateReconciliationPlan(
        "library/unsafe id",
        "source/unsafe id",
        { limit: 5, offset: 10 },
      ),
    ).resolves.toEqual(
      mockSourceDuplicateReconciliationPlan("library/unsafe id", "source/unsafe id"),
    );
    await expect(
      client.applySourceDuplicateReconciliation(
        "library/unsafe id",
        "source/unsafe id",
        {
          duplicate_source_id: "duplicate/source id",
          expected_action: "suggest_relationship",
        },
      ),
    ).resolves.toEqual(
      mockSourceDuplicateReconciliationApply(
        "library/unsafe id",
        "source/unsafe id",
        "duplicate/source id",
      ),
    );

    expect(fetcher.mock.calls).toMatchObject([
      [
        `${NAKO_ADMIN_ROUTES.sourceDuplicateReconciliationPlan
          .replace("{library_id}", encodeURIComponent("library/unsafe id"))
          .replace("{source_id}", encodeURIComponent("source/unsafe id"))}?limit=5&offset=10`,
        {},
      ],
      [
        NAKO_ADMIN_ROUTES.sourceDuplicateReconciliationApply
          .replace("{library_id}", encodeURIComponent("library/unsafe id"))
          .replace("{source_id}", encodeURIComponent("source/unsafe id")),
        {
          method: "POST",
          body: JSON.stringify({
            duplicate_source_id: "duplicate/source id",
            expected_action: "suggest_relationship",
          }),
        },
      ],
    ]);
  });

  it("uses generated VFS cache repair automation routes for dry-run and enqueue", async () => {
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (url.pathname === NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationPlan) {
        return Response.json(mockVfsCacheRepairAutomationPlan);
      }
      if (url.pathname === NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationJobs) {
        return Response.json(mockVfsCacheRepairAutomationEnqueueResponse);
      }

      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.planVfsCacheRepairAutomation({ enabled: true })).resolves.toEqual(
      mockVfsCacheRepairAutomationPlan,
    );
    await expect(
      client.enqueueVfsCacheRepairAutomation({ enabled: true, priority: "high" }),
    ).resolves.toEqual(mockVfsCacheRepairAutomationEnqueueResponse);

    expect(fetcher.mock.calls).toMatchObject([
      [
        NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationPlan,
        {
          method: "POST",
          body: JSON.stringify({ enabled: true }),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.storageVfsCacheRepairAutomationJobs,
        {
          method: "POST",
          body: JSON.stringify({ enabled: true, priority: "high" }),
        },
      ],
    ]);
  });

  it("posts Job cancellation through the generated route with encoded job IDs", async () => {
    const fetcher = vi.fn(async () =>
      Response.json({
        ...mockJobCancelRequestResponse,
        job: {
          ...mockJobCancelRequestResponse.job,
          id: "job/unsafe id",
        },
      }),
    );
    const client = new AdminApiClient({ fetcher });

    await expect(client.cancelJob("job/unsafe id")).resolves.toMatchObject({
      requested: true,
      terminal: true,
      job: {
        id: "job/unsafe id",
        status: "cancelled",
      },
    });

    expect(fetcher).toHaveBeenCalledWith(
      NAKO_ADMIN_ROUTES.jobCancel.replace(
        "{job_id}",
        encodeURIComponent("job/unsafe id"),
      ),
      expect.objectContaining({
        method: "POST",
        body: JSON.stringify({}),
      }),
    );
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
        addonTokensPath(),
        {
          method: "POST",
          body: JSON.stringify({ label: "sidecar runtime" }),
        },
      ],
      [
        addonTokenRotatePath(),
        {
          method: "POST",
          body: JSON.stringify({ label: "replacement" }),
        },
      ],
      [
        addonTokenRevokePath(),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        addonGrantsPath(),
        {
          method: "PUT",
          body: JSON.stringify({ grants: [{ permission: "metadata_write", library_id: null }] }),
        },
      ],
    ]);
  });

  it("uses generated Addon Task Run routes with encoded identifiers", async () => {
    const addonId = "addon/with space";
    const jobId = "job/task run";
    const encodedAddonId = encodeURIComponent(addonId);
    const encodedJobId = encodeURIComponent(jobId);
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (url.pathname.endsWith("/retry")) {
        expect(init?.method).toBe("POST");
        return Response.json({
          ...mockAddonTaskRunRetryResponse,
          run: {
            ...mockAddonTaskRunRetryResponse.run,
            addon_id: addonId,
            job_id: "job-addon-task-run-retry",
            retry_of_job_id: jobId,
          },
        });
      }

      if (url.pathname.endsWith(`/${encodedJobId}`)) {
        return Response.json({
          run: {
            ...mockAddonTaskRuns.runs[0],
            addon_id: addonId,
            job_id: jobId,
          },
          idempotent_replay: false,
        });
      }

      return Response.json(mockAddonTaskRuns);
    });
    const client = new AdminApiClient({ fetcher });

    await expect(client.getAddonTaskRuns(addonId, { limit: 5, offset: 10 })).resolves.toEqual(
      mockAddonTaskRuns,
    );
    await expect(client.getAddonTaskRun(addonId, jobId)).resolves.toMatchObject({
      run: {
        addon_id: addonId,
        job_id: jobId,
      },
    });
    await expect(
      client.retryAddonTaskRun(addonId, jobId, {
        idempotency_key: "retry-task-run-once",
      }),
    ).resolves.toMatchObject({
      run: {
        job_id: "job-addon-task-run-retry",
        retry_of_job_id: jobId,
      },
    });

    expect(fetcher.mock.calls).toMatchObject([
      [
        `${addonTaskRunsPath(encodedAddonId)}?limit=5&offset=10`,
        {},
      ],
      [
        addonTaskRunPath(encodedAddonId, encodedJobId),
        {},
      ],
      [
        addonTaskRunRetryPath(encodedAddonId, encodedJobId),
        {
          method: "POST",
          body: JSON.stringify({ idempotency_key: "retry-task-run-once" }),
        },
      ],
    ]);
  });

  it("uses generated Addon Event delivery routes with encoded identifiers", async () => {
    const eventId = "event/webhook due";
    const encodedEventId = encodeURIComponent(eventId);
    const seenRequests: Array<{ body: unknown; method: string; path: string; search: string }> = [];
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");
      seenRequests.push({
        body: init?.body ? JSON.parse(String(init.body)) : null,
        method: init?.method ?? "GET",
        path: url.pathname,
        search: url.search,
      });

      if (url.pathname === NAKO_ADMIN_ROUTES.events) {
        return Response.json(mockEvents);
      }
      if (
        url.pathname === eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliveryAttempts, encodedEventId)
      ) {
        return Response.json({
          ...mockAddonEventDeliveryAttempts,
          event_id: eventId,
        });
      }
      if (
        url.pathname === eventPath(NAKO_ADMIN_ROUTES.eventAddonSchedulerWork, encodedEventId)
      ) {
        return Response.json({
          ...mockAddonEventSchedulerWork,
          event: {
            ...mockAddonEventSchedulerWork.event,
            id: eventId,
          },
        });
      }
      if (url.pathname === eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliver, encodedEventId)) {
        return Response.json(mockAddonEventDispatch);
      }
      if (url.pathname === eventPath(NAKO_ADMIN_ROUTES.eventAddonReplay, encodedEventId)) {
        return Response.json(mockAddonEventReplay);
      }

      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ fetcher });

    await expect(
      client.getEvents({
        kind: "library_scanned",
        status: "failed",
        library_id: "library-films",
        source_id: "source-a",
        limit: 5,
        offset: 10,
      }),
    ).resolves.toEqual(mockEvents);
    await expect(client.getAddonEventDeliveryAttempts(eventId)).resolves.toMatchObject({
      event_id: eventId,
    });
    await expect(client.getAddonEventSchedulerWork(eventId)).resolves.toMatchObject({
      event: {
        id: eventId,
      },
    });
    await expect(client.deliverAddonEvents(eventId)).resolves.toEqual(mockAddonEventDispatch);
    await expect(
      client.replayAddonEvents(eventId, {
        reason_code: "operator_requested",
      }),
    ).resolves.toEqual(mockAddonEventReplay);

    expect(seenRequests).toEqual([
      {
        path: NAKO_ADMIN_ROUTES.events,
        search:
          "?kind=library_scanned&status=failed&library_id=library-films&source_id=source-a&limit=5&offset=10",
        method: "GET",
        body: null,
      },
      {
        path: eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliveryAttempts, encodedEventId),
        search: "",
        method: "GET",
        body: null,
      },
      {
        path: eventPath(NAKO_ADMIN_ROUTES.eventAddonSchedulerWork, encodedEventId),
        search: "",
        method: "GET",
        body: null,
      },
      {
        path: eventPath(NAKO_ADMIN_ROUTES.eventAddonDeliver, encodedEventId),
        search: "",
        method: "POST",
        body: {},
      },
      {
        path: eventPath(NAKO_ADMIN_ROUTES.eventAddonReplay, encodedEventId),
        search: "",
        method: "POST",
        body: {
          reason_code: "operator_requested",
        },
      },
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

  it("sends library profile replacement and commands through Admin API routes", async () => {
    const profile = {
      ...mockLibraryMetadataProfile("library-anime").profile,
      language: "ja-JP",
      country: "JP",
    };
    const job = {
      ...mockJobs.jobs[0],
      kind: "library_scan",
      resource_class: "disk.scan",
      library_id: "library-anime",
    };
    const fetcher = vi.fn(async (input: string | URL | Request, init?: RequestInit) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (url.pathname === NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace("{library_id}", "library-anime")) {
        return Response.json({ ...mockLibraryMetadataProfile("library-anime"), profile });
      }

      if (
        url.pathname === NAKO_ADMIN_ROUTES.libraryScan.replace("{library_id}", "library-anime") ||
        url.pathname === NAKO_ADMIN_ROUTES.libraryNfoImport.replace("{library_id}", "library-anime") ||
        url.pathname === NAKO_ADMIN_ROUTES.libraryNfoExport.replace("{library_id}", "library-anime")
      ) {
        expect(init?.method).toBe("POST");
        return Response.json(job);
      }

      if (url.pathname === "/libraries/library-anime/sources") {
        return Response.json({
          library: { id: "library-anime", name: "Anime Vault" },
          sources: [],
          page: { limit: 50, offset: 0, returned: 0 },
        });
      }

      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(client.updateLibraryMetadataProfile("library-anime", profile)).resolves.toMatchObject({
      profile: {
        language: "ja-JP",
        country: "JP",
      },
    });
    await expect(client.enqueueLibraryScan("library-anime")).resolves.toMatchObject({
      kind: "library_scan",
      resource_class: "disk.scan",
    });
    await expect(client.enqueueLibraryNfoImport("library-anime")).resolves.toMatchObject({
      library_id: "library-anime",
    });
    await expect(client.enqueueLibraryNfoExport("library-anime")).resolves.toMatchObject({
      library_id: "library-anime",
    });
    await expect(
      client.getPublicLibrarySourceInventoryBridge("library-anime", { limit: 50, offset: 0 }),
    ).resolves.toMatchObject({
      page: {
        returned: 0,
      },
    });

    expect(fetcher.mock.calls).toMatchObject([
      [
        NAKO_ADMIN_ROUTES.libraryMetadataProfile.replace("{library_id}", "library-anime"),
        {
          method: "PUT",
          body: JSON.stringify({ profile }),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.libraryScan.replace("{library_id}", "library-anime"),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.libraryNfoImport.replace("{library_id}", "library-anime"),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        NAKO_ADMIN_ROUTES.libraryNfoExport.replace("{library_id}", "library-anime"),
        {
          method: "POST",
          body: JSON.stringify({}),
        },
      ],
      [
        "/libraries/library-anime/sources?limit=50&offset=0",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
    ]);
  });

  it("bridges public Catalog and item reads through explicit methods", async () => {
    const itemDetail = mockPublicItemDetail("item-unknown-1");
    const sourceProbe = mockPublicSourceProbe("source-unknown-1");
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      const url = new URL(input.toString(), "http://127.0.0.1");

      if (url.pathname === "/items") {
        return Response.json(mockPublicCatalogItems);
      }

      if (url.pathname === "/search") {
        return Response.json(mockPublicCatalogSearch);
      }

      if (url.pathname === "/items/item-unknown-1") {
        return Response.json(itemDetail);
      }

      if (url.pathname === "/items/item-unknown-1/credits") {
        return Response.json({
          item_id: "item-unknown-1",
          credits: itemDetail.credits,
          people: [],
        });
      }

      if (url.pathname === "/items/item-unknown-1/images") {
        return Response.json({
          item_id: "item-unknown-1",
          images: itemDetail.images,
        });
      }

      if (url.pathname === "/sources/source-unknown-1/probe") {
        return Response.json(sourceProbe);
      }

      return new Response("not found", { status: 404 });
    });
    const client = new AdminApiClient({ token: "redacted-test-token", fetcher });

    await expect(
      client.getPublicCatalogItemsBridge({ limit: 10, offset: 20 }),
    ).resolves.toEqual(mockPublicCatalogItems);
    await expect(
      client.getPublicCatalogSearchBridge({
        q: "ova",
        facet: "kind:movie",
        limit: 5,
        offset: 0,
      }),
    ).resolves.toEqual(mockPublicCatalogSearch);
    await expect(client.getPublicItemDetailBridge("item-unknown-1")).resolves.toEqual(itemDetail);
    await expect(client.getPublicItemCreditsBridge("item-unknown-1")).resolves.toMatchObject({
      item_id: "item-unknown-1",
    });
    await expect(client.getPublicItemImagesBridge("item-unknown-1")).resolves.toMatchObject({
      images: itemDetail.images,
    });
    await expect(client.getPublicSourceProbeBridge("source-unknown-1")).resolves.toEqual(sourceProbe);

    expect(fetcher.mock.calls).toMatchObject([
      [
        "/items?limit=10&offset=20",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
      [
        "/search?q=ova&facet=kind%3Amovie&limit=5&offset=0",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
      [
        "/items/item-unknown-1",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
      [
        "/items/item-unknown-1/credits",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
      [
        "/items/item-unknown-1/images",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
      [
        "/sources/source-unknown-1/probe",
        {
          headers: {
            Authorization: "Bearer redacted-test-token",
          },
        },
      ],
    ]);
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
