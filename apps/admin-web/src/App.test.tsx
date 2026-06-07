import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";
import type { AdminConsoleData, AdminDataSource } from "./adminApi/dataSource";
import type {
  AdminAcquisitionIntakeCandidatesQuery,
  AdminAddonsQuery,
  AdminCatalogGovernanceItemsQuery,
  AdminGeneratedArtifactProposalsQuery,
  AdminJobListResponse,
  AdminJobsQuery,
  AdminPlaybackSessionsQuery,
  AdminSourceDuplicateReconciliationPlanResponse,
  AdminStorageStagingQuery,
  AddonsRouteSummary,
  AdminMetadataProfile,
  CatalogBrowseQuery,
  CatalogBrowseSummary,
  CatalogGovernanceItemDetailSummary,
  CatalogGovernanceProviderMappingReviewPlanSummary,
  CatalogGovernanceProviderMappingReviewResultSummary,
  GeneratedArtifactReviewPlanSummary,
  GeneratedArtifactReviewResultSummary,
  ItemArtworkGallerySummary,
  ItemArtworkMutationResultSummary,
  ItemDetailSummary,
  LibraryCommandAction,
  LibraryManagementDetail,
  AdminAccessSummaryResponse,
  AdminJobListItem,
} from "./adminApi/types";
import {
  mockAdminConsoleData,
  mockAcquisitionIntakeCandidates,
  mockAccessSummary,
  mockAddonsRouteSummary,
  mockCatalogBrowse,
  mockCatalogGovernance,
  mockGeneratedArtifactProposals,
  mockItemArtworkGallerySummary,
  mockItemDetailSummary,
  mockJobCancelRequestResponse,
  mockJobs,
  mockLibraryMetadataProfile,
  mockMetadataRawCacheSettings,
  mockOverview,
  mockPlaybackRuntimeSettings,
  mockPlaybackSessions,
  mockSourceDuplicateReconciliationApply,
  mockSourceDuplicateReconciliationPlan,
  mockStorageStaging,
  mockSystemConfig,
  mockVfsCacheRefreshResponse,
  mockVfsCacheRepairAutomationEnqueueResponse,
  mockVfsCacheRepairAutomationPlan,
  mockVfsCacheRepairActionPlan,
  mockVfsCacheRepairEnqueueResponse,
  mockVfsCacheRepairExecuteResponse,
  mockVfsCacheRepairRemediationPlan,
  mockVfsCacheRepairRetryJob,
  mockVfsCacheRepairTargets,
} from "./adminApi/mockData";

afterEach(() => {
  window.history.pushState(null, "", "/");
  window.localStorage.clear();
});

describe("Admin Web V2 route shell", () => {
  it("redirects to the route-owned Overview page by default", async () => {
    const loadOverview = vi.fn(async () => ({
      value: mockOverview,
      source: "live" as const,
    }));

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadOverview }} />);

    expect(await screen.findByRole("heading", { name: "Overview" })).toBeInTheDocument();
    await waitFor(() => {
      expect(window.location.pathname).toBe("/overview");
    });
    expect(screen.getByText("Admin Web V2")).toBeInTheDocument();
    expect(screen.getByText("Legacy Console")).toBeInTheDocument();
    expect((await screen.findAllByText("Live Admin API")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("Storage backends").length).toBeGreaterThan(0);
    expect(screen.getByText("2/3 ready")).toBeInTheDocument();
    expect(screen.getByText("Source fingerprint hash")).toBeInTheDocument();
    expect(screen.getByText("109/128")).toBeInTheDocument();
    expect(screen.getByText("Metadata providers")).toBeInTheDocument();
    expect(screen.getByText("Anime Vault")).toBeInTheDocument();
    expect(loadOverview).toHaveBeenCalledTimes(1);
  });

  it("keeps the legacy console route available while workflows migrate", async () => {
    window.history.pushState(null, "", "/legacy");

    render(<App dataSource={{ load: async () => mockAdminConsoleData }} />);

    expect(
      await screen.findByRole("heading", {
        name: "Server operations and media governance",
      }),
    ).toBeInTheDocument();
    expect(screen.getByText("Addon Operations")).toBeInTheDocument();
    expect(screen.getByText("Addon Credentials & Grants")).toBeInTheDocument();
    expect(screen.getAllByText("Subtitle Lab").length).toBeGreaterThan(0);
  });

  it("maps URL search params into generated AdminJobsQuery fields", async () => {
    const loadJobs = vi.fn(async (query?: AdminJobsQuery) => ({
      value: mockJobs,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/jobs?status=failed&kind=metadata_refresh&resource_class=metadata&library_id=library-films&source_id=source-hash-1&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadJobs }} />);

    await waitFor(() => {
      expect(loadJobs).toHaveBeenCalledWith({
        status: "failed",
        kind: "metadata_refresh",
        resource_class: "metadata",
        library_id: "library-films",
        source_id: "source-hash-1",
        limit: 10,
        offset: 20,
      });
    });
  });

  it("updates Jobs search params from shadcn-style filter controls", async () => {
    const loadJobs = vi.fn(async (query?: AdminJobsQuery) => ({
      value: mockJobs,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/jobs");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadJobs }} />);

    fireEvent.change(await screen.findByLabelText("Job status filter"), {
      target: { value: "failed" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("status=failed");
      expect(loadJobs).toHaveBeenLastCalledWith(
        expect.objectContaining({ status: "failed", limit: 20, offset: 0 }),
      );
    });

    fireEvent.change(screen.getByLabelText("Job media source filter"), {
      target: { value: "source-hash-1" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("source_id=source-hash-1");
      expect(loadJobs).toHaveBeenLastCalledWith(
        expect.objectContaining({ source_id: "source-hash-1", limit: 20, offset: 0 }),
      );
    });

    fireEvent.click(screen.getByRole("button", { name: /Source hash jobs/ }));

    await waitFor(() => {
      expect(window.location.search).toContain("kind=source_fingerprint_hash");
      expect(window.location.search).toContain(
        "resource_class=disk.scan.source_fingerprint_hash",
      );
      expect(loadJobs).toHaveBeenLastCalledWith(
        expect.objectContaining({
          kind: "source_fingerprint_hash",
          resource_class: "disk.scan.source_fingerprint_hash",
          source_id: "source-hash-1",
          limit: 20,
          offset: 0,
        }),
      );
    });

    const vfsCacheRepairButton = screen.getByRole("button", {
      name: /VFS cache repair jobs/,
    });
    expect(vfsCacheRepairButton).toHaveAttribute("aria-pressed", "false");

    fireEvent.click(vfsCacheRepairButton);

    await waitFor(() => {
      expect(window.location.search).toContain("kind=vfs_cache_repair");
      expect(window.location.search).toContain("resource_class=storage.vfs.cache_repair");
      expect(window.location.search).not.toContain("source_id=source-hash-1");
      expect(
        screen.getByRole("button", { name: /VFS cache repair jobs/ }),
      ).toHaveAttribute("aria-pressed", "true");
      expect(loadJobs).toHaveBeenLastCalledWith(
        expect.objectContaining({
          kind: "vfs_cache_repair",
          resource_class: "storage.vfs.cache_repair",
          source_id: undefined,
          limit: 20,
          offset: 0,
        }),
      );
    });
  });

  it("renders localized Jobs route copy", async () => {
    window.history.pushState(null, "", "/jobs");

    render(<App dataSource={jobsDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "任务" })).toBeInTheDocument();
    expect(await screen.findByText("任务队列")).toBeInTheDocument();
    expect(screen.getByLabelText("任务状态过滤器")).toBeInTheDocument();
    expect(screen.getByLabelText("任务媒体源过滤器")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Source Hash 任务/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "VFS cache repair 任务" })).toBeInTheDocument();
    expect(screen.getByText("URL 过滤条件具有权威性")).toBeInTheDocument();
    expect(screen.getByText("队列压力")).toBeInTheDocument();
    expect(screen.getByText("4 组，2 个排队任务")).toBeInTheDocument();
    expect(screen.getByText("2 个可领取")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
    expect(screen.getByText("生命周期")).toBeInTheDocument();
    expect(screen.getAllByText("优先级 normal").length).toBeGreaterThan(0);
    expect(screen.getByText("操作")).toBeInTheDocument();
  });

  it("shows deterministic mock fallback when the Jobs read model is unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadJobs() {
        return {
          value: mockJobs,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };

    window.history.pushState(null, "", "/jobs");
    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("job-scan")).toBeInTheDocument();
    expect(screen.getByText("Lifecycle")).toBeInTheDocument();
    expect(screen.getByText("Queue pressure")).toBeInTheDocument();
    expect(screen.getByText("4 groups, 2 queued jobs")).toBeInTheDocument();
    expect(screen.getByText("2 claimable")).toBeInTheDocument();
    expect(screen.getAllByText("storage.vfs.cache_repair").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Priority normal").length).toBeGreaterThan(0);
    expect(screen.getByText("Attempt 2/3")).toBeInTheDocument();
    expect(screen.getByText("Retry of job-source-hash-original")).toBeInTheDocument();
    expect(screen.getByText("Job actions require live Admin API data.")).toBeInTheDocument();
    expect(
      screen.getByRole("button", {
        name: "Execute VFS cache repair job job-vfs-cache-repair",
      }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", {
        name: "Cancel job job-vfs-cache-repair",
      }),
    ).toBeDisabled();
  });

  it("runs live VFS cache repair job execute and retry actions from the Jobs route", async () => {
    const queuedRepairJob = {
      ...mockVfsCacheRepairRetryJob,
      id: "job-vfs-cache-repair-queued-action",
      status: "queued",
      has_error: false,
      completed_at: null,
    } satisfies AdminJobListItem;
    const failedRepairJob = {
      ...mockVfsCacheRepairRetryJob,
      id: "job-vfs-cache-repair-failed-action",
      status: "failed",
      has_error: true,
      completed_at: "2026-06-05T00:08:00.000Z",
    } satisfies AdminJobListItem;
    const jobs: AdminJobListResponse = {
      ...mockJobs,
      jobs: [queuedRepairJob, failedRepairJob, mockJobs.jobs[2]],
      page: { limit: 20, offset: 0, returned: 3 },
    };
    const loadJobs = vi.fn(async () => ({
      value: jobs,
      source: "live" as const,
    }));
    const executeVfsCacheRepairJob = vi.fn(async () => ({
      ...mockVfsCacheRepairExecuteResponse,
      job: {
        ...mockVfsCacheRepairExecuteResponse.job,
        id: queuedRepairJob.id,
      },
    }));
    const retryVfsCacheRepairJob = vi.fn(async () => mockVfsCacheRepairRetryJob);
    const cancelJob = vi.fn(async () => ({
      ...mockJobCancelRequestResponse,
      job: {
        ...mockJobCancelRequestResponse.job,
        id: queuedRepairJob.id,
        status: "cancelled" as const,
      },
      terminal: true,
    }));
    window.history.pushState(null, "", "/jobs?kind=vfs_cache_repair&resource_class=storage.vfs.cache_repair");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadJobs,
          cancelJob,
          executeVfsCacheRepairJob,
          retryVfsCacheRepairJob,
        }}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", {
        name: "Execute VFS cache repair job job-vfs-cache-repair-queued-action",
      }),
    );

    await waitFor(() => {
      expect(executeVfsCacheRepairJob).toHaveBeenCalledWith(
        "job-vfs-cache-repair-queued-action",
      );
    });
    expect(
      await screen.findByText(
        "Executed VFS cache repair job job-vfs-cache-repair-queued-action, status succeeded.",
      ),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: "Cancel job job-vfs-cache-repair-queued-action",
      }),
    );

    await waitFor(() => {
      expect(cancelJob).toHaveBeenCalledWith("job-vfs-cache-repair-queued-action");
    });
    expect(
      await screen.findByText(
        "Cancellation requested for job job-vfs-cache-repair-queued-action, status cancelled, terminal true.",
      ),
    ).toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", {
        name: "Retry VFS cache repair job job-vfs-cache-repair-failed-action",
      }),
    );

    await waitFor(() => {
      expect(retryVfsCacheRepairJob).toHaveBeenCalledWith(
        "job-vfs-cache-repair-failed-action",
      );
    });
    expect(
      await screen.findByText(
        "Queued VFS cache repair retry job job-vfs-cache-repair-retry, status queued.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByText("No action for this status")).toBeInTheDocument();
  });

  it("shows deterministic mock fallback when the Overview read model is unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadOverview() {
        return {
          value: mockOverview,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };

    window.history.pushState(null, "", "/overview");
    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("Anime Vault")).toBeInTheDocument();
  });

  it("renders localized Overview route copy", async () => {
    const loadOverview = vi.fn(async () => ({
      value: mockOverview,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/overview");

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadOverview }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "总览" })).toBeInTheDocument();
    expect(await screen.findByText("服务器状态")).toBeInTheDocument();
    expect(screen.getAllByText("存储后端").length).toBeGreaterThan(0);
    expect(await screen.findByText("2/3 就绪")).toBeInTheDocument();
    expect(await screen.findByText("Metadata Provider")).toBeInTheDocument();
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
  });

  it("keeps unsafe fields out of the Overview route rendering", async () => {
    const unsafeOverview = {
      ...mockOverview,
      raw_token: "nako_at_one_time_raw_token",
      root_ref: "local://unsafe-root",
      source_uri: "file:///Users/frank/media",
      storage: {
        ...mockOverview.storage,
        cache_uri: "file:///F:/nako/cache",
        backends: mockOverview.storage.backends.map((backend) => ({
          ...backend,
          local_path: "F:\\media\\library",
          token_env: "TMDB_API_KEY",
        })),
      },
      metadata: {
        ...mockOverview.metadata,
        providers: mockOverview.metadata.providers.map((provider) => ({
          ...provider,
          provider_secret: "secret-provider-token",
        })),
      },
      source_fingerprint_hash: {
        ...mockOverview.source_fingerprint_hash,
        raw_fingerprint: "source:v1:content_hash:sha256:secret-content",
        raw_locator: "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
      },
    } as unknown as typeof mockOverview;
    window.history.pushState(null, "", "/overview");
    const { container } = render(<App dataSource={overviewDataSource(unsafeOverview)} />);

    await screen.findByRole("heading", { name: "Overview" });
    await screen.findByText("Anime Vault");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("one_time_raw_token");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("cache_uri");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("TMDB_API_KEY");
    expect(renderedText).not.toContain("provider_secret");
    expect(renderedText).not.toContain("secret-provider-token");
    expect(renderedText).not.toContain("raw_fingerprint");
    expect(renderedText).not.toContain("source:v1:content_hash:sha256:secret-content");
    expect(renderedText).not.toContain("raw_locator");
    expect(renderedText).not.toContain("Hidden Movie.mkv");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders Media Libraries as a route-owned V2 page", async () => {
    const loadLibraries = vi.fn(async () => ({
      value: mockSystemConfig,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/libraries");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadLibraries }} />);

    expect(await screen.findByRole("heading", { name: "Media Libraries" })).toBeInTheDocument();
    expect(screen.getByText("Configured libraries")).toBeInTheDocument();
    expect(await screen.findByText("Anime Vault")).toBeInTheDocument();
    expect(screen.getByText("Films")).toBeInTheDocument();
    expect(screen.getByText("Secret Reference configured")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadLibraries).toHaveBeenCalledTimes(1);
  });

  it("shows deterministic mock fallback when Media Libraries diagnostics are unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadLibraries() {
        return {
          value: mockSystemConfig,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/libraries");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("Anime Vault")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Media Libraries route rendering", async () => {
    window.history.pushState(null, "", "/libraries");
    const { container } = render(<App dataSource={librariesDataSource()} />);

    await screen.findByRole("heading", { name: "Media Libraries" });
    await screen.findByText("Anime Vault");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("webdav_password");
    expect(renderedText).not.toContain("TMDB_API_KEY");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders Media Library detail as a route-owned management entry", async () => {
    const loadLibraryDetail = vi.fn(async (libraryId: string) => ({
      value: libraryManagementDetail(libraryId),
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/libraries/library-anime");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadLibraryDetail }} />);

    expect(await screen.findByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
    expect(await screen.findByText("Library facts")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Libraries" })).toBeInTheDocument();
    expect(await screen.findByText("Library facts")).toBeInTheDocument();
    expect(screen.getByText("Metadata profile")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Source inventory" })).toBeInTheDocument();
    expect(screen.getByText("2 returned by bridge")).toBeInTheDocument();
    expect(screen.getByText("Episode 01.mkv")).toBeInTheDocument();
    expect(screen.getAllByText("NFO import").length).toBeGreaterThan(0);
    expect(screen.getByText("GET / PUT")).toBeInTheDocument();
    expect((await screen.findAllByText("Live Admin API")).length).toBeGreaterThan(0);
    expect(loadLibraryDetail).toHaveBeenCalledWith("library-anime");
  });

  it("edits Metadata Profile by full replacement and confirms library commands", async () => {
    const loadLibraryDetail = vi.fn(async (libraryId: string) => ({
      value: libraryManagementDetail(libraryId),
      source: "live" as const,
    }));
    const updateLibraryMetadataProfile = vi.fn(
      async (libraryId: string, profile: AdminMetadataProfile) => ({
      ...mockLibraryMetadataProfile(libraryId),
      profile,
    }));
    const runLibraryCommand = vi.fn(async (_libraryId: string, action: LibraryCommandAction) => ({
      action,
      job: {
        id: "job-scan-request",
        kind: "library_scan",
        status: "queued",
        resourceClass: "disk.scan",
        queuedAt: "2026-05-19T10:10:00Z",
        completedAt: null,
        hasError: false,
      },
    }));
    window.history.pushState(null, "", "/libraries/library-anime");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadLibraryDetail,
          updateLibraryMetadataProfile,
          runLibraryCommand,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
    fireEvent.click(await screen.findByRole("button", { name: "Edit" }));
    fireEvent.change(screen.getByLabelText("Refresh mode"), {
      target: { value: "full_refresh" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Save replacement" }));

    await waitFor(() => {
      expect(updateLibraryMetadataProfile).toHaveBeenCalledWith(
        "library-anime",
        expect.objectContaining({ refresh_mode: "full_refresh" }),
      );
    });
    expect(runLibraryCommand).not.toHaveBeenCalled();

    fireEvent.click(screen.getAllByRole("button", { name: "Queue" })[0]);
    expect(runLibraryCommand).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => {
      expect(runLibraryCommand).toHaveBeenCalledWith("library-anime", "scan");
    });
    expect(await screen.findByText(/Queued job job-scan-request/)).toBeInTheDocument();
  });

  it("shows deterministic mock fallback when Media Library detail is unavailable", async () => {
    window.history.pushState(null, "", "/libraries/library-anime");

    render(<App dataSource={{ load: async () => emptyConsoleData() }} />);

    expect(
      await screen.findByText(/Media Library detail route data source is unavailable/),
    ).toBeInTheDocument();
    expect(screen.getAllByText("Mock fallback").length).toBeGreaterThan(0);
    expect(screen.getByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Media Library detail route rendering", async () => {
    const detail = unsafeLibraryManagementDetail();
    window.history.pushState(null, "", "/libraries/library-anime");
    const { container } = render(<App dataSource={libraryDetailDataSource(detail)} />);

    expect(await screen.findByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("webdav_password");
    expect(renderedText).not.toContain("TMDB_API_KEY");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("secret-provider-token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders localized shell and library management copy", async () => {
    const loadLibraryDetail = vi.fn(async (libraryId: string) => ({
      value: libraryManagementDetail(libraryId),
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/libraries/library-anime");

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadLibraryDetail }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Anime Vault" })).toBeInTheDocument();
    expect(screen.getByRole("navigation", { name: "主导航" })).toBeInTheDocument();
    expect(screen.getByLabelText("语言")).toHaveValue("zh-Hans");
    expect(await screen.findByText("媒体库事实")).toBeInTheDocument();
    expect(screen.getByText("Metadata Profile")).toBeInTheDocument();
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
    expect(screen.getByText("刷新")).toBeInTheDocument();
  });

  it("renders System Settings as a route-owned V2 page", async () => {
    const loadSettings = vi.fn(async () => ({
      value: mockSystemConfig,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadSettings }} />);

    expect(await screen.findByRole("heading", { name: "System Settings" })).toBeInTheDocument();
    expect(await screen.findByText("Network readiness")).toBeInTheDocument();
    expect(await screen.findByText("reverse_proxy")).toBeInTheDocument();
    expect(screen.getAllByText("ready").length).toBeGreaterThan(0);
    expect(screen.getByText("Database")).toBeInTheDocument();
    expect(screen.getAllByText("sqlite").length).toBeGreaterThan(0);
    expect(screen.getByText("Metadata policy")).toBeInTheDocument();
    expect(await screen.findByText("Live Admin API")).toBeInTheDocument();
    expect(loadSettings).toHaveBeenCalledTimes(1);
  });

  it("shows deterministic mock fallback when System Settings diagnostics are unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("Network readiness")).toBeInTheDocument();
  });

  it("renders localized System Settings route copy", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "live",
        };
      },
      async loadMetadataRawCacheSettings() {
        return {
          value: mockMetadataRawCacheSettings,
          source: "live",
        };
      },
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "系统设置" })).toBeInTheDocument();
    expect(await screen.findByText("网络就绪度")).toBeInTheDocument();
    expect(screen.getByText("Admin 认证")).toBeInTheDocument();
    expect(screen.getByText("数据库")).toBeInTheDocument();
    expect(screen.getByText("Metadata 策略")).toBeInTheDocument();
    expect(screen.getByText("Metadata raw cache 设置")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "编辑覆盖值" })).toBeInTheDocument();
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
  });

  it("renders Users & Access as a route-owned V2 page", async () => {
    const loadAccessSummary = vi.fn(async () => ({
      value: mockAccessSummary,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/access");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAccessSummary }} />);

    expect(await screen.findByRole("heading", { name: "Users & Access" })).toBeInTheDocument();
    expect((await screen.findAllByText("Single-Admin Mode")).length).toBeGreaterThan(0);
    expect(screen.getByText("local-admin")).toBeInTheDocument();
    expect(screen.getByText("Effective Library Access")).toBeInTheDocument();
    expect(screen.getByText("Anime Vault")).toBeInTheDocument();
    expect(screen.getByText("Backend edit contracts are available.")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadAccessSummary).toHaveBeenCalledTimes(1);
  });

  it("shows deterministic mock fallback when Users & Access is unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadAccessSummary() {
        return {
          value: mockAccessSummary,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/access");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("Effective Library Access")).toBeInTheDocument();
  });

  it("renders localized Users & Access route copy", async () => {
    const loadAccessSummary = vi.fn(async () => ({
      value: mockAccessSummary,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/access");

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadAccessSummary }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "用户与访问" })).toBeInTheDocument();
    expect(await screen.findByText("当前主体")).toBeInTheDocument();
    expect(await screen.findByText("有效 Library Access")).toBeInTheDocument();
    expect(await screen.findByText("后端编辑契约已经可用。")).toBeInTheDocument();
    expect(await screen.findByText("本地管理员")).toBeInTheDocument();
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
  });

  it("keeps unsafe fields out of the Users & Access route rendering", async () => {
    const unsafeAccessSummary = {
      ...mockAccessSummary,
      raw_token: "nako_secret_token",
      token_env: "NAKO_ADMIN_TOKEN",
      local_path: "F:\\nako\\access.toml",
      source_uri: "file:///Users/frank/private",
      url: "https://user:secret@example.test/access",
      auth: {
        ...mockAccessSummary.auth,
        token_env: "NAKO_ADMIN_TOKEN",
        raw_token: "nako_secret_token",
      },
      library_access: {
        ...mockAccessSummary.library_access,
        libraries: mockAccessSummary.library_access.libraries.map((library) => ({
          ...library,
          root_ref: "local://unsafe-root",
          source_uri: "file:///Users/frank/private",
          url: "https://user:secret@example.test/access",
        })),
      },
    } as unknown as AdminAccessSummaryResponse;
    window.history.pushState(null, "", "/access");
    const { container } = render(<App dataSource={accessDataSource(unsafeAccessSummary)} />);

    await screen.findByRole("heading", { name: "Users & Access" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("NAKO_ADMIN_TOKEN");
    expect(renderedText).not.toContain("nako_secret_token");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("unsafe-root");
    expect(renderedText).not.toContain("file://");
    expect(renderedText).not.toContain("https://user:secret@example.test");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("saves metadata raw cache settings only after confirmation on live Settings data", async () => {
    const updateMetadataRawCacheSettings = vi.fn(async (request) => ({
      admin_api_version: "v1",
      retention_ms: request.retention_ms,
      cleanup_on_startup: request.cleanup_on_startup,
      source: "admin" as const,
      effect: "requires_restart" as const,
      updated_at_ms: 1779700000000,
    }));
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "live",
        };
      },
      async loadMetadataRawCacheSettings() {
        return {
          value: {
            admin_api_version: "v1",
            retention_ms: 604800000,
            cleanup_on_startup: true,
            source: "configured",
            effect: "active",
            updated_at_ms: null,
          },
          source: "live",
        };
      },
      updateMetadataRawCacheSettings,
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Metadata raw cache settings")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Edit override/i }));
    fireEvent.change(screen.getByLabelText("Retention milliseconds"), {
      target: { value: "3600000" },
    });
    fireEvent.click(screen.getByRole("checkbox", { name: "Cleanup on startup" }));
    fireEvent.click(screen.getByRole("button", { name: /Prepare save/i }));
    expect(updateMetadataRawCacheSettings).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: /Confirm save/i }));

    await waitFor(() => {
      expect(updateMetadataRawCacheSettings).toHaveBeenCalledWith({
        retention_ms: 3600000,
        cleanup_on_startup: false,
      });
    });
    expect(await screen.findByText(/requires_restart/)).toBeInTheDocument();
  });

  it("shows a visible error when metadata raw cache settings save fails", async () => {
    const updateMetadataRawCacheSettings = vi.fn(async () => {
      throw new Error("Admin API request failed with HTTP 503");
    });
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "live",
        };
      },
      async loadMetadataRawCacheSettings() {
        return {
          value: {
            admin_api_version: "v1",
            retention_ms: 604800000,
            cleanup_on_startup: true,
            source: "configured",
            effect: "active",
            updated_at_ms: null,
          },
          source: "live",
        };
      },
      updateMetadataRawCacheSettings,
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Metadata raw cache settings")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Edit override/i }));
    fireEvent.change(screen.getByLabelText("Retention milliseconds"), {
      target: { value: "3600000" },
    });
    fireEvent.click(screen.getByRole("button", { name: /Prepare save/i }));
    fireEvent.click(screen.getByRole("button", { name: /Confirm save/i }));

    await waitFor(() => {
      expect(updateMetadataRawCacheSettings).toHaveBeenCalledWith({
        retention_ms: 3600000,
        cleanup_on_startup: true,
      });
    });
    expect(await screen.findByText("Admin API request failed with HTTP 503")).toBeInTheDocument();
  });

  it("does not expose a fake save action for metadata raw cache mock fallback", async () => {
    const updateMetadataRawCacheSettings = vi.fn();
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
      async loadMetadataRawCacheSettings() {
        return {
          value: {
            admin_api_version: "v1",
            retention_ms: 604800000,
            cleanup_on_startup: true,
            source: "configured",
            effect: "active",
            updated_at_ms: null,
          },
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
      updateMetadataRawCacheSettings,
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Metadata raw cache settings")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Edit override/i })).toBeDisabled();
    expect(screen.getByText(/Save is disabled/)).toBeInTheDocument();
    expect(updateMetadataRawCacheSettings).not.toHaveBeenCalled();
  });

  it("saves playback runtime settings only after confirmation on live Settings data", async () => {
    const updatePlaybackRuntimeSettings = vi.fn(async (request) => ({
      ...mockPlaybackRuntimeSettings,
      settings: request.settings,
      source: "admin" as const,
      effect: "requires_restart" as const,
      updated_at_ms: 1779700000000,
    }));
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "live",
        };
      },
      async loadMetadataRawCacheSettings() {
        return {
          value: mockMetadataRawCacheSettings,
          source: "live",
        };
      },
      async loadPlaybackRuntimeSettings() {
        return {
          value: mockPlaybackRuntimeSettings,
          source: "live",
        };
      },
      updatePlaybackRuntimeSettings,
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Playback runtime settings")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: /Edit runtime/i }));
    fireEvent.change(screen.getByLabelText("CPU transcode workers"), {
      target: { value: "3" },
    });
    fireEvent.click(screen.getByRole("checkbox", { name: /Staging cleanup on startup/i }));
    fireEvent.click(screen.getByRole("button", { name: /Prepare save/i }));
    expect(updatePlaybackRuntimeSettings).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: /Confirm save/i }));

    await waitFor(() => {
      expect(updatePlaybackRuntimeSettings).toHaveBeenCalledWith({
        settings: {
          ...mockPlaybackRuntimeSettings.settings,
          cpu_concurrency: 3,
          staging_cleanup_on_startup: false,
        },
      });
    });
    expect(await screen.findByText(/Playback runtime settings override saved/)).toBeInTheDocument();
    expect(await screen.findByText(/requires_restart/)).toBeInTheDocument();
  });

  it("does not expose a fake save action for playback runtime settings mock fallback", async () => {
    const updatePlaybackRuntimeSettings = vi.fn();
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadSettings() {
        return {
          value: mockSystemConfig,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
      async loadPlaybackRuntimeSettings() {
        return {
          value: mockPlaybackRuntimeSettings,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
      updatePlaybackRuntimeSettings,
    };
    window.history.pushState(null, "", "/settings");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Playback runtime settings")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Edit runtime/i })).toBeDisabled();
    expect(screen.getByText(/Playback runtime changes are disabled/)).toBeInTheDocument();
    expect(updatePlaybackRuntimeSettings).not.toHaveBeenCalled();
  });

  it("keeps unsafe fields out of the System Settings route rendering", async () => {
    const unsafeSystemConfig = {
      ...mockSystemConfig,
      root_ref: "local://unsafe-root",
      source_uri: "file:///Users/frank/nako/config.toml",
      local_path: "F:\\nako\\config.toml",
      auth: {
        ...mockSystemConfig.auth,
        token_env: "NAKO_ADMIN_TOKEN",
      },
      network: {
        ...mockSystemConfig.network,
        external_endpoint: {
          ...mockSystemConfig.network.external_endpoint,
          host_fingerprint: "external-host-fingerprint",
        },
        tunnel_providers: mockSystemConfig.network.tunnel_providers.map((provider) => ({
          ...provider,
          endpoint_host_fingerprint: "tunnel-host-fingerprint",
          token_env: "CLOUDFLARE_TUNNEL_TOKEN",
        })),
      },
      runtime: {
        ...mockSystemConfig.runtime,
        listen_addr: "127.0.0.1:3000",
      },
      metadata: {
        ...mockSystemConfig.metadata,
        providers: mockSystemConfig.metadata.providers.map((provider) => ({
          ...provider,
          api_key_env: "TMDB_API_KEY",
          token_env: "TMDB_TOKEN",
        })),
        runtime: {
          ...mockSystemConfig.metadata.runtime,
          user_agent: "private-metadata-agent",
        },
      },
      artwork: {
        ...mockSystemConfig.artwork,
        artifact_root: "F:\\nako\\artwork",
        fetch_user_agent: "private-artwork-agent",
      },
    } as unknown as typeof mockSystemConfig;
    const unsafePlaybackRuntimeSettings = {
      ...mockPlaybackRuntimeSettings,
      settings: {
        ...mockPlaybackRuntimeSettings.settings,
        hardware_acceleration: "file:///Users/frank/render-device",
        hardware_fallback: "F:\\gpu\\device",
      },
    };
    const dataSource = settingsDataSource(unsafeSystemConfig);
    dataSource.loadPlaybackRuntimeSettings = async () => ({
      value: unsafePlaybackRuntimeSettings,
      source: "live",
    });
    window.history.pushState(null, "", "/settings");
    const { container } = render(<App dataSource={dataSource} />);

    await screen.findByRole("heading", { name: "System Settings" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("NAKO_ADMIN_TOKEN");
    expect(renderedText).not.toContain("CLOUDFLARE_TUNNEL_TOKEN");
    expect(renderedText).not.toContain("TMDB_API_KEY");
    expect(renderedText).not.toContain("TMDB_TOKEN");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("api_key_env");
    expect(renderedText).not.toContain("host_fingerprint");
    expect(renderedText).not.toContain("external-host-fingerprint");
    expect(renderedText).not.toContain("tunnel-host-fingerprint");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("127.0.0.1:3000");
    expect(renderedText).not.toContain("private-metadata-agent");
    expect(renderedText).not.toContain("private-artwork-agent");
    expect(renderedText).not.toContain("render-device");
    expect(renderedText).not.toContain("gpu\\device");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("maps Acquisition Intake URL search params into generated query fields", async () => {
    const loadAcquisitionIntake = vi.fn(async (query?: AdminAcquisitionIntakeCandidatesQuery) => ({
      value: mockAcquisitionIntakeCandidates,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/acquisition/intake?library_id=library-anime&state=ready&source_kind=watch_folder&managed_import_artifact_id=artifact-managed&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAcquisitionIntake }} />);

    await waitFor(() => {
      expect(loadAcquisitionIntake).toHaveBeenCalledWith({
        library_id: "library-anime",
        state: "ready",
        source_kind: "watch_folder",
        managed_import_artifact_id: "artifact-managed",
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Acquisition Intake as a route-owned V2 page", async () => {
    const loadAcquisitionIntake = vi.fn(async () => ({
      value: mockAcquisitionIntakeCandidates,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/acquisition/intake");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAcquisitionIntake }} />);

    expect(await screen.findByRole("heading", { name: "Acquisition Intake" })).toBeInTheDocument();
    expect(await screen.findByText("candidate-ready")).toBeInTheDocument();
    expect(screen.getAllByText("watch_folder").length).toBeGreaterThan(0);
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadAcquisitionIntake).toHaveBeenCalledWith({
      library_id: undefined,
      state: undefined,
      source_kind: undefined,
      managed_import_artifact_id: undefined,
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Acquisition Intake route copy", async () => {
    window.history.pushState(null, "", "/acquisition/intake");

    render(<App dataSource={acquisitionIntakeDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Acquisition Intake" })).toBeInTheDocument();
    expect(await screen.findByText("Intake 候选")).toBeInTheDocument();
    expect(screen.getByLabelText("Intake 状态过滤器")).toBeInTheDocument();
    expect(screen.getByText("URL 过滤条件具有权威性")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Acquisition Intake search params from filter controls", async () => {
    const loadAcquisitionIntake = vi.fn(async (query?: AdminAcquisitionIntakeCandidatesQuery) => ({
      value: mockAcquisitionIntakeCandidates,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/acquisition/intake");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAcquisitionIntake }} />);

    fireEvent.change(await screen.findByLabelText("Intake state filter"), {
      target: { value: "blocked" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("state=blocked");
      expect(loadAcquisitionIntake).toHaveBeenLastCalledWith(
        expect.objectContaining({ state: "blocked", limit: 20, offset: 0 }),
      );
    });
  });

  it("shows deterministic mock fallback when Acquisition Intake diagnostics are unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadAcquisitionIntake() {
        return {
          value: mockAcquisitionIntakeCandidates,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/acquisition/intake");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("candidate-ready")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Acquisition Intake route rendering", async () => {
    window.history.pushState(null, "", "/acquisition/intake");
    const { container } = render(<App dataSource={acquisitionIntakeDataSource(unsafeAcquisitionIntake())} />);

    await screen.findByRole("heading", { name: "Acquisition Intake" });
    await screen.findByText("candidate-ready");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_ref_redacted");
    expect(renderedText).not.toContain("local://unsafe-root");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/drop");
    expect(renderedText).not.toContain("raw_locator");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("maps Generated Artifacts URL search params into generated query fields", async () => {
    const loadGeneratedArtifacts = vi.fn(async (query?: AdminGeneratedArtifactProposalsQuery) => ({
      value: mockGeneratedArtifactProposals,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/automation/generated-artifacts?limit=10&offset=20");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifacts }} />);

    await waitFor(() => {
      expect(loadGeneratedArtifacts).toHaveBeenCalledWith({
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Generated Artifacts as a route-owned V2 page", async () => {
    const loadGeneratedArtifacts = vi.fn(async () => ({
      value: mockGeneratedArtifactProposals,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/automation/generated-artifacts");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifacts }} />);

    expect(await screen.findByRole("heading", { name: "Generated Artifacts" })).toBeInTheDocument();
    expect(await screen.findByText("artifact-metadata-cleanup")).toBeInTheDocument();
    expect(screen.getByText("metadata_cleanup")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadGeneratedArtifacts).toHaveBeenCalledWith({
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Generated Artifacts route copy", async () => {
    window.history.pushState(null, "", "/automation/generated-artifacts");

    render(<App dataSource={generatedArtifactsDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Generated Artifacts" })).toBeInTheDocument();
    expect(await screen.findByText("Proposal 队列")).toBeInTheDocument();
    expect(screen.getByLabelText("Generated artifacts 页面 limit")).toBeInTheDocument();
    expect(screen.getByText("URL 分页具有权威性")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Generated Artifacts search params from pagination controls", async () => {
    const loadGeneratedArtifacts = vi.fn(async (query?: AdminGeneratedArtifactProposalsQuery) => ({
      value: mockGeneratedArtifactProposals,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/automation/generated-artifacts");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifacts }} />);

    fireEvent.change(await screen.findByLabelText("Generated artifacts page limit"), {
      target: { value: "10" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("limit=10");
      expect(loadGeneratedArtifacts).toHaveBeenLastCalledWith(
        expect.objectContaining({ limit: 10, offset: 0 }),
      );
    });
  });

  it("shows deterministic mock fallback when Generated Artifacts proposals are unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadGeneratedArtifacts() {
        return {
          value: mockGeneratedArtifactProposals,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/automation/generated-artifacts");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("artifact-metadata-cleanup")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Generated Artifacts route rendering", async () => {
    window.history.pushState(null, "", "/automation/generated-artifacts");
    const { container } = render(<App dataSource={generatedArtifactsDataSource(unsafeGeneratedArtifacts())} />);

    await screen.findByRole("heading", { name: "Generated Artifacts" });
    await screen.findByText("artifact-metadata-cleanup");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("prompt_text");
    expect(renderedText).not.toContain("secret prompt body");
    expect(renderedText).not.toContain("payload_body");
    expect(renderedText).not.toContain("secret payload body");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/generated");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders a Generated Artifact review-plan route for the selected decision", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=reject",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifactReviewPlan }} />);

    expect(await screen.findByRole("heading", { name: "Generated Artifact Review" })).toBeInTheDocument();
    expect(await screen.findByText("artifact-metadata-cleanup")).toBeInTheDocument();
    expect((await screen.findAllByText("reject")).length).toBeGreaterThan(0);
    expect(await screen.findByText("mark_rejected")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadGeneratedArtifactReviewPlan).toHaveBeenCalledWith(
      "artifact-metadata-cleanup",
      "reject",
    );
  });

  it("renders localized Generated Artifact review route copy", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=reject",
    );

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifactReviewPlan }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Generated Artifact 审查" })).toBeInTheDocument();
    expect(await screen.findByText("Review 计划")).toBeInTheDocument();
    expect(screen.getByText("Review 边界")).toBeInTheDocument();
    expect(screen.getByText("已确认操作")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "拒绝" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /准备 reject 审查/ })).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("keeps Generated Artifact review decision state in the URL search", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=accept",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifactReviewPlan }} />);

    fireEvent.click(await screen.findByRole("button", { name: /Reject/ }));

    await waitFor(() => {
      expect(window.location.search).toContain("decision=reject");
      expect(loadGeneratedArtifactReviewPlan).toHaveBeenLastCalledWith(
        "artifact-metadata-cleanup",
        "reject",
      );
    });
  });

  it("shows deterministic mock fallback for Generated Artifact review-plan preview", async () => {
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=accept",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData() }} />);

    expect(await screen.findByText(/review-plan data source is unavailable/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("stage_metadata_authority_review")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Generated Artifact review-plan rendering", async () => {
    const unsafePlan = {
      ...generatedArtifactReviewPlanSummary({ decision: "accept" }),
      prompt_text: "secret prompt body",
      payload_body: "secret payload body",
      raw_provider_response: "provider raw body",
      artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
      payload: {
        ...generatedArtifactReviewPlanSummary({ decision: "accept" }).payload,
        raw_json: '{"secret":"secret payload body"}',
      },
      target: {
        ...generatedArtifactReviewPlanSummary({ decision: "accept" }).target,
        source_uri: "file:///Users/frank/generated",
        local_path: "F:\\generated\\artifact.json",
      },
    } as GeneratedArtifactReviewPlanSummary;
    const loadGeneratedArtifactReviewPlan = vi.fn(async () => ({
      value: unsafePlan,
      source: "live" as const,
    }));
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=accept",
    );

    const { container } = render(
      <App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifactReviewPlan }} />,
    );

    await screen.findByRole("heading", { name: "Generated Artifact Review" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("prompt_text");
    expect(renderedText).not.toContain("secret prompt body");
    expect(renderedText).not.toContain("payload_body");
    expect(renderedText).not.toContain("secret payload body");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("artifact_storage_handle");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/generated");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("requires explicit confirmation before posting a Generated Artifact review mutation", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    const reviewGeneratedArtifact = vi.fn(async (artifactId: string, decision: "accept" | "reject") =>
      generatedArtifactReviewResultSummary({ artifactId, decision }),
    );
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=accept",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadGeneratedArtifactReviewPlan,
          reviewGeneratedArtifact,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Generated Artifact Review" })).toBeInTheDocument();
    expect(reviewGeneratedArtifact).not.toHaveBeenCalled();

    fireEvent.click(await screen.findByRole("button", { name: /Prepare accept review/ }));
    expect(reviewGeneratedArtifact).not.toHaveBeenCalled();

    fireEvent.click(await screen.findByRole("button", { name: /Confirm accept/ }));

    await waitFor(() => {
      expect(reviewGeneratedArtifact).toHaveBeenCalledWith("artifact-metadata-cleanup", "accept");
    });
    expect(await screen.findByText("Review result")).toBeInTheDocument();
    expect(screen.getByText("accepted")).toBeInTheDocument();
    expect(screen.getByText("new result")).toBeInTheDocument();
  });

  it("shows a visible error when Generated Artifact review mutation is unavailable", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=reject",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadGeneratedArtifactReviewPlan }} />);

    fireEvent.click(await screen.findByRole("button", { name: /Prepare reject review/ }));
    fireEvent.click(await screen.findByRole("button", { name: /Confirm reject/ }));

    expect(await screen.findByText("Generated Artifact review action is unavailable")).toBeInTheDocument();
    expect(screen.queryByText("rejected")).not.toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Generated Artifact review result rendering", async () => {
    const loadGeneratedArtifactReviewPlan = vi.fn(async (artifactId: string, decision: "accept" | "reject") => ({
      value: generatedArtifactReviewPlanSummary({ artifactId, decision }),
      source: "live" as const,
    }));
    const unsafeResult = {
      ...generatedArtifactReviewResultSummary({ decision: "accept" }),
      prompt_text: "secret prompt body",
      payload_body: "secret payload body",
      raw_provider_response: "provider raw body",
      artifact_storage_handle: "F:\\nako\\artifact-cache\\metadata.json",
      plan: {
        ...generatedArtifactReviewResultSummary({ decision: "accept" }).plan,
        target: {
          ...generatedArtifactReviewResultSummary({ decision: "accept" }).plan.target,
          source_uri: "file:///Users/frank/generated",
          local_path: "F:\\generated\\artifact.json",
        },
        payload: {
          ...generatedArtifactReviewResultSummary({ decision: "accept" }).plan.payload,
          raw_json: '{"secret":"secret payload body"}',
        },
      },
    } as GeneratedArtifactReviewResultSummary;
    const reviewGeneratedArtifact = vi.fn(async () => unsafeResult);
    window.history.pushState(
      null,
      "",
      "/automation/generated-artifacts/artifact-metadata-cleanup/review?decision=accept",
    );

    const { container } = render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadGeneratedArtifactReviewPlan,
          reviewGeneratedArtifact,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: /Prepare accept review/ }));
    fireEvent.click(await screen.findByRole("button", { name: /Confirm accept/ }));
    await screen.findByText("Review result");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("prompt_text");
    expect(renderedText).not.toContain("secret prompt body");
    expect(renderedText).not.toContain("payload_body");
    expect(renderedText).not.toContain("secret payload body");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("artifact_storage_handle");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/generated");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("maps Addons URL search params into generated AdminAddonsQuery fields", async () => {
    const loadAddons = vi.fn(async (query?: AdminAddonsQuery) => ({
      value: mockAddonsRouteSummary,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/addons?status=disabled");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAddons }} />);

    await waitFor(() => {
      expect(loadAddons).toHaveBeenCalledWith({
        status: "disabled",
      });
    });
  });

  it("renders Addons as a route-owned V2 page", async () => {
    const loadAddons = vi.fn(async () => ({
      value: mockAddonsRouteSummary,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/addons");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAddons }} />);

    expect(await screen.findByRole("heading", { name: "Addons" })).toBeInTheDocument();
    expect(await screen.findByText("Addon registry")).toBeInTheDocument();
    expect(screen.getAllByText("Subtitle Lab").length).toBeGreaterThan(0);
    expect(screen.getByText("Surface declarations")).toBeInTheDocument();
    expect(screen.getByText("Install boundary")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadAddons).toHaveBeenCalledWith({
      status: undefined,
    });
  });

  it("renders localized Addons route copy", async () => {
    const loadAddons = vi.fn(async () => ({
      value: mockAddonsRouteSummary,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/addons");

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadAddons }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Addons" })).toBeInTheDocument();
    expect(await screen.findByText("Addon 注册表")).toBeInTheDocument();
    expect(screen.getByLabelText("Addon 状态过滤器")).toBeInTheDocument();
    expect(screen.getByText("Surface 声明")).toBeInTheDocument();
    expect(screen.getByText("安装边界")).toBeInTheDocument();
    expect(screen.getByText("URL 过滤条件具有权威性")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Addons search params from filter controls", async () => {
    const loadAddons = vi.fn(async (query?: AdminAddonsQuery) => ({
      value: mockAddonsRouteSummary,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/addons");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadAddons }} />);

    fireEvent.change(await screen.findByLabelText("Addon status filter"), {
      target: { value: "disabled" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("status=disabled");
      expect(loadAddons).toHaveBeenLastCalledWith(
        expect.objectContaining({ status: "disabled" }),
      );
    });
  });

  it("shows deterministic mock fallback when Addons diagnostics are unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return emptyConsoleData();
      },
      async loadAddons() {
        return {
          value: mockAddonsRouteSummary,
          source: "mock",
          error: "Admin API request failed with HTTP 503",
        };
      },
    };
    window.history.pushState(null, "", "/addons");

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getAllByText("Subtitle Lab").length).toBeGreaterThan(0);
  });

  it("keeps unsafe fields out of the Addons route rendering", async () => {
    window.history.pushState(null, "", "/addons");
    const { container } = render(<App dataSource={addonsDataSource(unsafeAddonsRouteSummary())} />);

    await screen.findByRole("heading", { name: "Addons" });
    await screen.findAllByText("Subtitle Lab");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("one_time_raw_token");
    expect(renderedText).not.toContain("token_env");
    expect(renderedText).not.toContain("ADDON_SECRET_SUBTITLE_PROVIDER_KEY");
    expect(renderedText).not.toContain("secret-reference:subtitle-provider-key");
    expect(renderedText).not.toContain("http://subtitle-lab:9100");
    expect(renderedText).not.toContain("/pages/diagnostics");
    expect(renderedText).not.toContain("/resources/subtitles");
    expect(renderedText).not.toContain("docker_compose");
    expect(renderedText).not.toContain("curl -fsS");
    expect(renderedText).not.toContain("manifest-secret-payload");
    expect(renderedText).not.toContain("unsafe lifecycle message");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("maps Media Catalog URL search params into public bridge query fields", async () => {
    const loadCatalog = vi.fn(async (query?: CatalogBrowseQuery) => ({
      value: mockCatalogBrowse,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/catalog?q=ova&facet=kind%3Amovie&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalog }} />);

    await waitFor(() => {
      expect(loadCatalog).toHaveBeenCalledWith({
        q: "ova",
        facet: "kind:movie",
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Media Catalog as a route-owned governance browse page", async () => {
    const loadCatalog = vi.fn(async () => ({
      value: mockCatalogBrowse,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/catalog");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalog }} />);

    expect(await screen.findByRole("heading", { name: "Media Catalog" })).toBeInTheDocument();
    expect(await screen.findByText("Unmatched OVA Special")).toBeInTheDocument();
    expect(screen.getByText("1 genres / 2 tags")).toBeInTheDocument();
    expect(screen.getAllByText("sources: detail route").length).toBeGreaterThan(0);
    expect(screen.getByRole("link", { name: "Inspect Unmatched OVA Special" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1",
    );
    expect(screen.getByRole("link", { name: /Governance queue/ }).getAttribute("href")).toContain(
      "/catalog/governance",
    );
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadCatalog).toHaveBeenCalledWith({
      q: undefined,
      facet: undefined,
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Media Catalog route copy", async () => {
    window.history.pushState(null, "", "/catalog");

    render(<App dataSource={catalogBrowseDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "媒体目录" })).toBeInTheDocument();
    expect(await screen.findByText("Media Items")).toBeInTheDocument();
    expect(screen.getByLabelText("Catalog 搜索查询")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /治理队列/ })).toHaveAttribute(
      "href",
      "/catalog/governance?limit=20&offset=0",
    );
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Media Catalog search params from filter controls", async () => {
    const loadCatalog = vi.fn(async (query?: CatalogBrowseQuery) => ({
      value: {
        ...mockCatalogBrowse,
        mode: query?.q || query?.facet ? "search" : "browse",
      } satisfies CatalogBrowseSummary,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/catalog");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalog }} />);

    fireEvent.change(await screen.findByLabelText("Catalog search query"), {
      target: { value: "ova" },
    });
    fireEvent.change(await screen.findByLabelText("Catalog facet filter"), {
      target: { value: "kind:movie" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("q=ova");
      expect(window.location.search).toContain("facet=kind%3Amovie");
      expect(loadCatalog).toHaveBeenLastCalledWith(
        expect.objectContaining({ q: "ova", facet: "kind:movie", limit: 20, offset: 0 }),
      );
    });
  });

  it("shows deterministic mock fallback when Media Catalog browse is unavailable", async () => {
    window.history.pushState(null, "", "/catalog");

    render(<App dataSource={{ load: async () => emptyConsoleData() }} />);

    expect(
      await screen.findByText(/Media Catalog route data source is unavailable/),
    ).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("Unmatched OVA Special")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Media Catalog route rendering", async () => {
    window.history.pushState(null, "", "/catalog");
    const { container } = render(<App dataSource={catalogBrowseDataSource(unsafeCatalogBrowse())} />);

    await screen.findByRole("heading", { name: "Media Catalog" });
    await screen.findByText("Unmatched OVA Special");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/media");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("artifact_storage_handle");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders Media Item detail as a route-owned governance inspection page", async () => {
    const detail = mockItemDetailSummary("item-unknown-1");
    const loadItemDetail = vi.fn(async (itemId: string) => ({
      value: detail,
      source: "live" as const,
      itemId,
    }));
    window.history.pushState(null, "", "/items/item-unknown-1");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadItemDetail }} />);

    expect(await screen.findByRole("heading", { name: "Unmatched OVA Special" })).toBeInTheDocument();
    expect(await screen.findByText("Item facts")).toBeInTheDocument();
    expect(screen.getByText("Canonical Metadata")).toBeInTheDocument();
    expect(screen.getByText("Media Sources")).toBeInTheDocument();
    expect(screen.getByText("Artwork and readiness")).toBeInTheDocument();
    expect(screen.getByText("Unmatched OVA Special.mkv")).toBeInTheDocument();
    expect(screen.getByText("matroska / 48m / 3 streams")).toBeInTheDocument();
    expect(screen.getByText("Provider Mapping")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Open Artwork Gallery" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1/artwork?limit=20&offset=0",
    );
    expect(
      screen.getByRole("link", { name: "Review duplicates for source-unknown-1" }),
    ).toHaveAttribute(
      "href",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime&limit=20&offset=0",
    );
    expect(screen.getByRole("link", { name: "Back to Catalog" })).toHaveAttribute(
      "href",
      "/catalog?limit=20&offset=0",
    );
    expect(screen.getAllByRole("link", { name: /Open/ }).length).toBeGreaterThan(0);
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadItemDetail).toHaveBeenCalledWith("item-unknown-1");
  });

  it("renders localized Media Item detail route copy", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1");

    render(<App dataSource={itemDetailDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Unmatched OVA Special" })).toBeInTheDocument();
    expect(await screen.findByText("条目事实")).toBeInTheDocument();
    expect(screen.getByText("Artwork 与就绪度")).toBeInTheDocument();
    expect(screen.getByText("支持链接")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "打开 Artwork Gallery" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1/artwork?limit=20&offset=0",
    );
    expect(screen.getByRole("link", { name: "审查 source-unknown-1 的重复来源" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime&limit=20&offset=0",
    );
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("shows deterministic mock fallback when Media Item detail is unavailable", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1");

    render(<App dataSource={{ load: async () => emptyConsoleData() }} />);

    expect(
      await screen.findByText(/Media Item detail route data source is unavailable/),
    ).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Unmatched OVA Special" })).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Media Item detail route rendering", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1");
    const { container } = render(<App dataSource={itemDetailDataSource(unsafeItemDetail())} />);

    await screen.findByRole("heading", { name: "Unmatched OVA Special" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/media");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("artifact_storage_handle");
    expect(renderedText).not.toContain("playback_output_path");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders source duplicate reconciliation and maps URL search into plan calls", async () => {
    const loadSourceDuplicateReconciliationPlan = vi.fn(
      async (
        libraryId: string,
        sourceId: string,
        query?: { limit?: number; offset?: number },
      ) => ({
        value: mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
        source: "live" as const,
        query,
      }),
    );
    window.history.pushState(
      null,
      "",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime&limit=5&offset=10",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadSourceDuplicateReconciliationPlan,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Source duplicate reconciliation" })).toBeInTheDocument();
    expect(await screen.findByText("Plan summary")).toBeInTheDocument();
    expect(screen.getByText("source-unknown-2")).toBeInTheDocument();
    expect(screen.getByText("source-extra-3")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Item" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1",
    );
    expect(loadSourceDuplicateReconciliationPlan).toHaveBeenCalledWith(
      "library-anime",
      "source-unknown-1",
      { limit: 5, offset: 10 },
    );
  });

  it("applies source duplicate suggestions only after explicit confirmation", async () => {
    const applySourceDuplicateReconciliation = vi.fn(
      async (libraryId: string, sourceId: string, duplicateSourceId: string) =>
        mockSourceDuplicateReconciliationApply(libraryId, sourceId, duplicateSourceId),
    );
    window.history.pushState(
      null,
      "",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadSourceDuplicateReconciliationPlan: async (libraryId, sourceId, query) => ({
            value: mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
            source: "live",
            query,
          }),
          applySourceDuplicateReconciliation,
        }}
      />,
    );

    expect(await screen.findByText("source-unknown-2")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Prepare suggestion" }));
    expect(applySourceDuplicateReconciliation).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Confirm suggestion" }));

    await waitFor(() => {
      expect(applySourceDuplicateReconciliation).toHaveBeenCalledWith(
        "library-anime",
        "source-unknown-1",
        "source-unknown-2",
      );
    });
    expect(await screen.findByText("Suggested relationship")).toBeInTheDocument();
    expect(screen.getByText("source-dup-suggested")).toBeInTheDocument();
  });

  it("keeps non-suggestion source duplicate candidates read-only", async () => {
    const plan = {
      ...mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1"),
      candidates: [
        {
          ...mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1").candidates[0],
          duplicate_source_id: "source-refresh-required",
          recommended_action: "refresh_source_fingerprint",
        },
      ],
      page: { limit: 20, offset: 0, returned: 1 },
    } satisfies AdminSourceDuplicateReconciliationPlanResponse;
    const applySourceDuplicateReconciliation = vi.fn();
    window.history.pushState(
      null,
      "",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadSourceDuplicateReconciliationPlan: async () => ({
            value: plan,
            source: "live",
          }),
          applySourceDuplicateReconciliation,
        }}
      />,
    );

    expect(await screen.findByText("source-refresh-required")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "No mutation" })).toBeDisabled();
    expect(applySourceDuplicateReconciliation).not.toHaveBeenCalled();
  });

  it("renders localized source duplicate reconciliation copy", async () => {
    window.history.pushState(
      null,
      "",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadSourceDuplicateReconciliationPlan: async (libraryId, sourceId) => ({
            value: mockSourceDuplicateReconciliationPlan(libraryId, sourceId),
            source: "live",
          }),
        }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Source duplicate 调和" })).toBeInTheDocument();
    expect(await screen.findByText("计划摘要")).toBeInTheDocument();
    expect(screen.getByText("重复候选")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "准备建议" })).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the source duplicate reconciliation rendering", async () => {
    const unsafePlan = {
      ...mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1"),
      raw_fingerprint: "source:v1:content_hash:sha256:secret-content",
      raw_locator: "local:///Users/Frankorz/Secret Path/Hidden Movie.mkv?token=secret",
      token_env: "TMDB_API_KEY",
      candidates: mockSourceDuplicateReconciliationPlan("library-anime", "source-unknown-1").candidates.map(
        (candidate) => ({
          ...candidate,
          source_uri: "file:///Users/frank/media/private.mkv",
          local_path: "F:\\media\\private.mkv",
          raw_token: "nako_at_one_time_raw_token",
        }),
      ),
    } as unknown as AdminSourceDuplicateReconciliationPlanResponse;
    window.history.pushState(
      null,
      "",
      "/items/item-unknown-1/sources/source-unknown-1/duplicates?library_id=library-anime",
    );

    const { container } = render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadSourceDuplicateReconciliationPlan: async () => ({
            value: unsafePlan,
            source: "live",
          }),
        }}
      />,
    );

    await screen.findByRole("heading", { name: "Source duplicate reconciliation" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("raw_fingerprint");
    expect(renderedText).not.toContain("source:v1:content_hash:sha256:secret-content");
    expect(renderedText).not.toContain("raw_locator");
    expect(renderedText).not.toContain("Hidden Movie.mkv");
    expect(renderedText).not.toContain("TMDB_API_KEY");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/media/private.mkv");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("one_time_raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders item artwork gallery as a guarded route-owned page", async () => {
    const gallery = mockItemArtworkGallerySummary("item-unknown-1");
    const loadItemArtworkGallery = vi.fn(
      async (itemId: string, query?: { limit?: number; offset?: number }) => ({
        value: gallery,
        source: "live" as const,
        itemId,
        query,
      }),
    );
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=5&offset=10");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadItemArtworkGallery }} />);

    expect(await screen.findByRole("heading", { name: "Managed Artwork" })).toBeInTheDocument();
    expect((await screen.findAllByText("Selected Artwork")).length).toBeGreaterThan(0);
    expect(screen.getByText("candidate-poster-1")).toBeInTheDocument();
    expect(screen.getAllByText("artifact-poster-1").length).toBeGreaterThan(0);
    expect(screen.getByText("/images/image-poster-1")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Prepare select artifact-backdrop-1" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Prepare unpublish poster" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Back to Item" })).toHaveAttribute(
      "href",
      "/items/item-unknown-1",
    );
    expect(loadItemArtworkGallery).toHaveBeenCalledWith("item-unknown-1", {
      limit: 5,
      offset: 10,
    });
  });

  it("renders localized item artwork gallery route copy", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=5&offset=10");

    render(<App dataSource={itemArtworkGalleryDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Managed Artwork" })).toBeInTheDocument();
    expect(await screen.findByText("Gallery 摘要")).toBeInTheDocument();
    expect(screen.getAllByText("Selected Artwork").length).toBeGreaterThan(0);
    expect(screen.getByText("已确认操作结果")).toBeInTheDocument();
    expect(screen.getByLabelText("Artwork gallery 页面 limit")).toHaveValue(5);
    expect(screen.getByRole("button", { name: "准备选择 artifact-backdrop-1" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "准备取消发布 poster" })).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("selects item artwork only after explicit confirmation", async () => {
    const gallery = mockItemArtworkGallerySummary("item-unknown-1");
    const selectItemArtwork = vi.fn(async (itemId: string, kind: string, artifactId: string) => ({
      action: "select" as const,
      itemId,
      kind,
      changed: true,
      selectedArtworkId: "selected-backdrop-1",
      artifactId,
      imageId: "image-backdrop-1",
      routePath: "/images/image-backdrop-1",
      width: 1920,
      height: 1080,
      language: null,
      mediaType: "image/webp",
    }));
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=20&offset=0");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadItemArtworkGallery: async () => ({
            value: gallery,
            source: "live",
          }),
          selectItemArtwork,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Prepare select artifact-backdrop-1" }));

    expect(selectItemArtwork).not.toHaveBeenCalled();
    expect(await screen.findByText("Confirm select")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Confirm select artifact-backdrop-1" }));

    await waitFor(() => {
      expect(selectItemArtwork).toHaveBeenCalledWith(
        "item-unknown-1",
        "backdrop",
        "artifact-backdrop-1",
      );
    });
    expect(await screen.findByText("Selection updated")).toBeInTheDocument();
    expect(screen.getByText("/images/image-backdrop-1")).toBeInTheDocument();
  });

  it("unpublishes item artwork only after explicit confirmation", async () => {
    const gallery = mockItemArtworkGallerySummary("item-unknown-1");
    const unpublishItemArtwork = vi.fn(async (itemId: string, kind: string) => ({
      action: "unpublish" as const,
      itemId,
      kind,
      changed: true,
      selectedArtworkId: "selected-poster-1",
      artifactId: "artifact-poster-1",
      imageId: "image-poster-1",
      routePath: "/images/image-poster-1",
      width: 1000,
      height: 1500,
      language: "ja",
      mediaType: "image/jpeg",
    }));
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=20&offset=0");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadItemArtworkGallery: async () => ({
            value: gallery,
            source: "live",
          }),
          unpublishItemArtwork,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Prepare unpublish poster" }));

    expect(unpublishItemArtwork).not.toHaveBeenCalled();
    expect(await screen.findByText("Confirm unpublish")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Confirm unpublish poster" }));

    await waitFor(() => {
      expect(unpublishItemArtwork).toHaveBeenCalledWith("item-unknown-1", "poster");
    });
    expect(await screen.findByText("Selection unpublished")).toBeInTheDocument();
    expect(screen.getAllByText("/images/image-poster-1").length).toBeGreaterThan(0);
  });

  it("shows a visible error when an item artwork mutation is unavailable", async () => {
    const gallery = mockItemArtworkGallerySummary("item-unknown-1");
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=20&offset=0");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadItemArtworkGallery: async () => ({
            value: gallery,
            source: "live",
          }),
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Prepare select artifact-backdrop-1" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm select artifact-backdrop-1" }));

    expect(await screen.findByText("Item artwork select action is unavailable")).toBeInTheDocument();
    expect(screen.queryByText("Selection updated")).not.toBeInTheDocument();
  });

  it("keeps unsafe fields out of the item artwork mutation result rendering", async () => {
    const gallery = mockItemArtworkGallerySummary("item-unknown-1");
    const unsafeResult = {
      action: "select",
      itemId: "item-unknown-1",
      kind: "backdrop",
      changed: true,
      selectedArtworkId: "selected-backdrop-1",
      artifactId: "artifact-backdrop-1",
      imageId: "image-backdrop-1",
      routePath: "https://provider.example/backdrop.webp?token=secret",
      width: 1920,
      height: 1080,
      language: null,
      mediaType: "image/webp",
      source_uri: "file:///Users/frank/generated",
      storage_uri: "managed-artwork://library-anime/private/backdrop.webp",
      raw_token: "secret-token",
    } satisfies ItemArtworkMutationResultSummary & Record<string, unknown>;
    const selectItemArtwork = vi.fn(async () => unsafeResult);
    window.history.pushState(null, "", "/items/item-unknown-1/artwork?limit=20&offset=0");

    const { container } = render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadItemArtworkGallery: async () => ({
            value: gallery,
            source: "live",
          }),
          selectItemArtwork,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: "Prepare select artifact-backdrop-1" }));
    fireEvent.click(screen.getByRole("button", { name: "Confirm select artifact-backdrop-1" }));

    expect(await screen.findByText("Selection updated")).toBeInTheDocument();
    const renderedText = container.textContent ?? "";
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("storage_uri");
    expect(renderedText).not.toContain("managed-artwork://");
    expect(renderedText).not.toContain("provider.example");
    expect(renderedText).not.toContain("token=secret");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("secret-token");
    expect(renderedText).not.toContain("/Users/");
  });

  it("shows deterministic mock fallback when item artwork gallery is unavailable", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1/artwork");

    render(<App dataSource={{ load: async () => emptyConsoleData() }} />);

    expect(
      await screen.findByText(/Item artwork gallery route data source is unavailable/),
    ).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("candidate-poster-1")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the item artwork gallery route rendering", async () => {
    window.history.pushState(null, "", "/items/item-unknown-1/artwork");
    const { container } = render(
      <App dataSource={itemArtworkGalleryDataSource(unsafeItemArtworkGallery())} />,
    );

    await screen.findByText("candidate-poster-1");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("storage_uri");
    expect(renderedText).not.toContain("cache_uri");
    expect(renderedText).not.toContain("managed-artwork://");
    expect(renderedText).not.toContain("provider.example");
    expect(renderedText).not.toContain("token=secret");
    expect(renderedText).not.toContain("content_hash");
    expect(renderedText).not.toContain("sha256:");
    expect(renderedText).not.toContain("F:\\");
  });

  it("maps Catalog Governance URL search params into generated query fields", async () => {
    const loadCatalogGovernance = vi.fn(async (query?: AdminCatalogGovernanceItemsQuery) => ({
      value: mockCatalogGovernance,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/catalog/governance?library_id=library-anime&max_confidence_milli=500&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalogGovernance }} />);

    await waitFor(() => {
      expect(loadCatalogGovernance).toHaveBeenCalledWith({
        library_id: "library-anime",
        max_confidence_milli: 500,
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Catalog Governance as a route-owned V2 page", async () => {
    const loadCatalogGovernance = vi.fn(async () => ({
      value: mockCatalogGovernance,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/catalog/governance");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalogGovernance }} />);

    expect(await screen.findByRole("heading", { name: "Catalog Governance" })).toBeInTheDocument();
    expect(await screen.findByText("Unmatched OVA Special")).toBeInTheDocument();
    expect(screen.getByText("low_local_inference_confidence")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadCatalogGovernance).toHaveBeenCalledWith({
      library_id: undefined,
      max_confidence_milli: undefined,
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Catalog Governance route copy", async () => {
    const loadCatalogGovernance = vi.fn(async () => ({
      value: mockCatalogGovernance,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/catalog/governance");

    render(
      <App
        dataSource={{ load: async () => emptyConsoleData(), loadCatalogGovernance }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Catalog Governance" })).toBeInTheDocument();
    expect(await screen.findByText("治理队列")).toBeInTheDocument();
    expect(screen.getByLabelText("Catalog library 过滤器")).toBeInTheDocument();
    expect(screen.getByText("本地推断")).toBeInTheDocument();
    expect(screen.getAllByText("审查").length).toBeGreaterThan(0);
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
  });

  it("updates Catalog Governance search params from filter controls", async () => {
    const loadCatalogGovernance = vi.fn(async (query?: AdminCatalogGovernanceItemsQuery) => ({
      value: mockCatalogGovernance,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/catalog/governance");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadCatalogGovernance }} />);

    fireEvent.change(await screen.findByLabelText("Catalog library filter"), {
      target: { value: "library-films" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("library_id=library-films");
      expect(loadCatalogGovernance).toHaveBeenLastCalledWith(
        expect.objectContaining({ library_id: "library-films", limit: 20, offset: 0 }),
      );
    });
  });

  it("keeps unsafe fields out of the Catalog Governance route rendering", async () => {
    window.history.pushState(null, "", "/catalog/governance");
    const { container } = render(<App dataSource={catalogGovernanceDataSource()} />);

    await screen.findByText("Unmatched OVA Special");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("renders Catalog Governance repair context for the selected Provider Mapping decision", async () => {
    const loadCatalogGovernanceItemDetail = vi.fn(async (itemId: string) => ({
      value: catalogGovernanceItemDetailSummary(itemId),
      source: "live" as const,
    }));
    const loadCatalogGovernanceProviderMappingReviewPlan = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") => ({
        value: catalogGovernanceProviderMappingReviewPlanSummary({
          itemId,
          mappingId,
          decision,
        }),
        source: "live" as const,
      }),
    );
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=reject",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail,
          loadCatalogGovernanceProviderMappingReviewPlan,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Catalog Governance Repair" })).toBeInTheDocument();
    expect(await screen.findByText("Film Needs Mapping")).toBeInTheDocument();
    expect(screen.getByLabelText("Provider Mapping selector")).toHaveValue("mapping-tmdb-603");
    expect((await screen.findAllByText("reject")).length).toBeGreaterThan(0);
    expect(await screen.findByText("rejected")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadCatalogGovernanceItemDetail).toHaveBeenCalledWith("item-low-confidence");
    expect(loadCatalogGovernanceProviderMappingReviewPlan).toHaveBeenCalledWith(
      "item-low-confidence",
      "mapping-tmdb-603",
      "reject",
    );
  });

  it("renders localized Catalog Governance repair route copy", async () => {
    const loadCatalogGovernanceItemDetail = vi.fn(async (itemId: string) => ({
      value: catalogGovernanceItemDetailSummary(itemId),
      source: "live" as const,
    }));
    const loadCatalogGovernanceProviderMappingReviewPlan = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") => ({
        value: catalogGovernanceProviderMappingReviewPlanSummary({
          itemId,
          mappingId,
          decision,
        }),
        source: "live" as const,
      }),
    );
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=reject",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail,
          loadCatalogGovernanceProviderMappingReviewPlan,
        }}
        initialLocale="zh-Hans"
      />,
    );

    expect(await screen.findByRole("heading", { name: "Catalog Governance 修复" })).toBeInTheDocument();
    expect(await screen.findByText("Media Item 上下文")).toBeInTheDocument();
    expect(screen.getByLabelText("Provider Mapping 选择器")).toHaveValue("mapping-tmdb-603");
    expect(screen.getByText("修复边界")).toBeInTheDocument();
    expect(screen.getByText("已确认操作")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "拒绝" })).toBeInTheDocument();
    expect((await screen.findAllByText("实时 Admin API")).length).toBeGreaterThan(0);
  });

  it("keeps Catalog Governance repair mapping and decision state in URL search", async () => {
    const loadCatalogGovernanceItemDetail = vi.fn(async (itemId: string) => ({
      value: catalogGovernanceItemDetailSummary(itemId),
      source: "live" as const,
    }));
    const loadCatalogGovernanceProviderMappingReviewPlan = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") => ({
        value: catalogGovernanceProviderMappingReviewPlanSummary({
          itemId,
          mappingId,
          decision,
        }),
        source: "live" as const,
      }),
    );
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=accept",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail,
          loadCatalogGovernanceProviderMappingReviewPlan,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: /Reject/ }));

    await waitFor(() => {
      expect(window.location.search).toContain("decision=reject");
      expect(loadCatalogGovernanceProviderMappingReviewPlan).toHaveBeenLastCalledWith(
        "item-low-confidence",
        "mapping-tmdb-603",
        "reject",
      );
    });
  });

  it("requires explicit confirmation before posting a Catalog Governance Provider Mapping review mutation", async () => {
    const loadCatalogGovernanceItemDetail = vi.fn(async (itemId: string) => ({
      value: catalogGovernanceItemDetailSummary(itemId),
      source: "live" as const,
    }));
    const loadCatalogGovernanceProviderMappingReviewPlan = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") => ({
        value: catalogGovernanceProviderMappingReviewPlanSummary({
          itemId,
          mappingId,
          decision,
        }),
        source: "live" as const,
      }),
    );
    const reviewCatalogGovernanceProviderMapping = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") =>
        catalogGovernanceProviderMappingReviewResultSummary({
          itemId,
          mappingId,
          decision,
        }),
    );
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=accept",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail,
          loadCatalogGovernanceProviderMappingReviewPlan,
          reviewCatalogGovernanceProviderMapping,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Catalog Governance Repair" })).toBeInTheDocument();
    expect(reviewCatalogGovernanceProviderMapping).not.toHaveBeenCalled();

    fireEvent.click(await screen.findByRole("button", { name: /Prepare accept mapping review/ }));
    expect(reviewCatalogGovernanceProviderMapping).not.toHaveBeenCalled();

    fireEvent.click(await screen.findByRole("button", { name: /Confirm accept/ }));

    await waitFor(() => {
      expect(reviewCatalogGovernanceProviderMapping).toHaveBeenCalledWith(
        "item-low-confidence",
        "mapping-tmdb-603",
        "accept",
      );
    });
    expect(await screen.findByText("Review result")).toBeInTheDocument();
    expect(screen.getAllByText("accepted").length).toBeGreaterThan(0);
    expect(screen.getByText("new result")).toBeInTheDocument();
  });

  it("shows a visible error when Catalog Governance Provider Mapping review mutation is unavailable", async () => {
    const loadCatalogGovernanceItemDetail = vi.fn(async (itemId: string) => ({
      value: catalogGovernanceItemDetailSummary(itemId),
      source: "live" as const,
    }));
    const loadCatalogGovernanceProviderMappingReviewPlan = vi.fn(
      async (itemId: string, mappingId: string, decision: "accept" | "reject") => ({
        value: catalogGovernanceProviderMappingReviewPlanSummary({
          itemId,
          mappingId,
          decision,
        }),
        source: "live" as const,
      }),
    );
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=reject",
    );

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail,
          loadCatalogGovernanceProviderMappingReviewPlan,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: /Prepare reject mapping review/ }));
    fireEvent.click(await screen.findByRole("button", { name: /Confirm reject/ }));

    expect(
      await screen.findByText("Catalog Governance Provider Mapping review action is unavailable"),
    ).toBeInTheDocument();
    expect(screen.queryByText("Review result")).not.toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Catalog Governance repair rendering and result", async () => {
    const unsafeDetail = {
      ...catalogGovernanceItemDetailSummary("item-low-confidence"),
      source_uri: "file:///Users/frank/media/private.mkv",
      local_path: "F:\\media\\private.mkv",
      evidence_value: "raw-evidence-token",
      provider_raw_body: "provider raw body",
      nfo_xml: "<movie><title>secret</title></movie>",
    } as CatalogGovernanceItemDetailSummary;
    const unsafePlan = {
      ...catalogGovernanceProviderMappingReviewPlanSummary({ decision: "accept" }),
      source_locator: "local:///library/private/Film Needs Mapping.mkv",
      local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
      raw_provider_response: "provider raw body",
      provider_request_url: "https://provider.example/api?token=secret",
    } as CatalogGovernanceProviderMappingReviewPlanSummary;
    const unsafeResult = {
      ...catalogGovernanceProviderMappingReviewResultSummary({ decision: "accept" }),
      evidence_value: "mapping-raw-evidence",
      source_locator: "local:///library/private/Film Needs Mapping.mkv",
      local_path: "F:\\library\\private\\Film Needs Mapping.mkv",
      raw_provider_response: "provider raw body",
      provider_request_url: "https://provider.example/api?token=secret",
      nfo_xml: "<movie><title>secret</title></movie>",
    } as CatalogGovernanceProviderMappingReviewResultSummary;
    const reviewCatalogGovernanceProviderMapping = vi.fn(async () => unsafeResult);
    window.history.pushState(
      null,
      "",
      "/catalog/governance/item-low-confidence?mapping_id=mapping-tmdb-603&decision=accept",
    );

    const { container } = render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadCatalogGovernanceItemDetail: async () => ({
            value: unsafeDetail,
            source: "live",
          }),
          loadCatalogGovernanceProviderMappingReviewPlan: async () => ({
            value: unsafePlan,
            source: "live",
          }),
          reviewCatalogGovernanceProviderMapping,
        }}
      />,
    );

    fireEvent.click(await screen.findByRole("button", { name: /Prepare accept mapping review/ }));
    fireEvent.click(await screen.findByRole("button", { name: /Confirm accept/ }));
    await screen.findByText("Review result");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("file:///Users/frank/media/private.mkv");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("evidence_value");
    expect(renderedText).not.toContain("raw-evidence-token");
    expect(renderedText).not.toContain("source_locator");
    expect(renderedText).not.toContain("local:///");
    expect(renderedText).not.toContain("raw_provider_response");
    expect(renderedText).not.toContain("provider raw body");
    expect(renderedText).not.toContain("provider_request_url");
    expect(renderedText).not.toContain("token=secret");
    expect(renderedText).not.toContain("nfo_xml");
    expect(renderedText).not.toContain("<movie>");
  });

  it("maps Playback Sessions URL search params into generated query fields", async () => {
    const loadPlaybackSessions = vi.fn(async (query?: AdminPlaybackSessionsQuery) => ({
      value: mockPlaybackSessions,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/playback/sessions?source_id=source-hls&state=running&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadPlaybackSessions }} />);

    await waitFor(() => {
      expect(loadPlaybackSessions).toHaveBeenCalledWith({
        source_id: "source-hls",
        state: "running",
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Playback Sessions as a route-owned V2 page", async () => {
    const loadPlaybackSessions = vi.fn(async () => ({
      value: mockPlaybackSessions,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/playback/sessions");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadPlaybackSessions }} />);

    expect(await screen.findByRole("heading", { name: "Playback Sessions" })).toBeInTheDocument();
    expect(await screen.findByText("session-hls")).toBeInTheDocument();
    expect(screen.getByText("transcode")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadPlaybackSessions).toHaveBeenCalledWith({
      source_id: undefined,
      state: undefined,
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Playback Sessions route copy", async () => {
    window.history.pushState(null, "", "/playback/sessions");

    render(<App dataSource={playbackSessionsDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Playback Sessions" })).toBeInTheDocument();
    expect(await screen.findByText("Session 队列")).toBeInTheDocument();
    expect(screen.getByLabelText("Playback 状态过滤器")).toBeInTheDocument();
    expect(screen.getByText("URL 过滤条件具有权威性")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Playback Sessions search params from filter controls", async () => {
    const loadPlaybackSessions = vi.fn(async (query?: AdminPlaybackSessionsQuery) => ({
      value: mockPlaybackSessions,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/playback/sessions");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadPlaybackSessions }} />);

    fireEvent.change(await screen.findByLabelText("Playback state filter"), {
      target: { value: "running" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("state=running");
      expect(loadPlaybackSessions).toHaveBeenLastCalledWith(
        expect.objectContaining({ state: "running", limit: 20, offset: 0 }),
      );
    });
  });

  it("keeps unsafe fields out of the Playback Sessions route rendering", async () => {
    window.history.pushState(null, "", "/playback/sessions");
    const { container } = render(<App dataSource={playbackSessionsDataSource()} />);

    await screen.findByText("session-hls");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("request_key");
    expect(renderedText).not.toContain("profile:v1");
    expect(renderedText).not.toContain("ffmpeg");
    expect(renderedText).not.toContain("stderr");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("maps Storage Staging URL search params into generated query fields", async () => {
    const loadStorageStaging = vi.fn(async (query?: AdminStorageStagingQuery) => ({
      value: mockStorageStaging,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/storage/staging?purpose=ffmpeg_input&state=ready&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadStorageStaging }} />);

    await waitFor(() => {
      expect(loadStorageStaging).toHaveBeenCalledWith({
        purpose: "ffmpeg_input",
        state: "ready",
        limit: 10,
        offset: 20,
      });
    });
  });

  it("renders Storage Staging as a route-owned V2 page", async () => {
    const loadStorageStaging = vi.fn(async () => ({
      value: mockStorageStaging,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/storage/staging");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadStorageStaging }} />);

    expect(await screen.findByRole("heading", { name: "Storage Staging" })).toBeInTheDocument();
    expect(await screen.findByText("staging-hls")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Purpose/state summary" })).toBeInTheDocument();
    expect(
      screen.getByText("2 purpose/state groups from the staging manifest."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "Cleanup candidate purpose/state summary" }),
    ).toBeInTheDocument();
    expect(
      screen.getByText("2 purpose/state groups from cleanup candidates."),
    ).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Manifest bytes" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Cleanup candidate bytes" })).toBeInTheDocument();
    expect(screen.getAllByRole("columnheader", { name: "Unknown size records" }).length).toBeGreaterThan(0);
    expect(screen.getAllByText("ffmpeg_input").length).toBeGreaterThan(0);
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadStorageStaging).toHaveBeenCalledWith({
      purpose: undefined,
      state: undefined,
      limit: 20,
      offset: 0,
    });
  });

  it("renders localized Storage Staging route copy", async () => {
    window.history.pushState(null, "", "/storage/staging");

    render(<App dataSource={storageStagingDataSource()} initialLocale="zh-Hans" />);

    expect(await screen.findByRole("heading", { name: "Storage Staging" })).toBeInTheDocument();
    expect(await screen.findByText("Staging 记录")).toBeInTheDocument();
    expect(screen.getByText("用途/状态汇总")).toBeInTheDocument();
    expect(screen.getByText("清理候选用途/状态汇总")).toBeInTheDocument();
    expect(screen.getByLabelText("Storage 状态过滤器")).toBeInTheDocument();
    expect(screen.getByText("URL 过滤条件具有权威性")).toBeInTheDocument();
    expect(screen.getByText("实时 Admin API")).toBeInTheDocument();
  });

  it("updates Storage Staging search params from filter controls", async () => {
    const loadStorageStaging = vi.fn(async (query?: AdminStorageStagingQuery) => ({
      value: mockStorageStaging,
      source: "live" as const,
      query,
    }));
    window.history.pushState(null, "", "/storage/staging");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadStorageStaging }} />);

    fireEvent.change(await screen.findByLabelText("Storage state filter"), {
      target: { value: "ready" },
    });

    await waitFor(() => {
      expect(window.location.search).toContain("state=ready");
      expect(loadStorageStaging).toHaveBeenLastCalledWith(
        expect.objectContaining({ state: "ready", limit: 20, offset: 0 }),
      );
    });
  });

  it("renders VFS cache repair context on the Storage Staging route", async () => {
    const loadStorageStaging = vi.fn(async (query?: AdminStorageStagingQuery) => ({
      value: mockStorageStaging,
      source: "live" as const,
      query,
    }));
    const loadVfsCacheRepairActionPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairActionPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairRemediationPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairRemediationPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairAutomationPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairAutomationPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairTargets = vi.fn(async () => ({
      value: mockVfsCacheRepairTargets,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/storage/staging");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadStorageStaging,
          loadVfsCacheRepairActionPlan,
          loadVfsCacheRepairRemediationPlan,
          loadVfsCacheRepairAutomationPlan,
          loadVfsCacheRepairTargets,
        }}
      />,
    );

    expect(await screen.findByRole("heading", { name: "Storage Staging" })).toBeInTheDocument();
    expect(await screen.findByText("VFS cache repair")).toBeInTheDocument();
    expect(await screen.findByText("Plan status")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Automation dry-run plan" })).toBeInTheDocument();
    expect(screen.getByText("Automation policy")).toBeInTheDocument();
    expect(screen.getAllByText("Eligible targets").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Blocked targets").length).toBeGreaterThan(0);
    expect(screen.getAllByText("backend_configuration_required").length).toBeGreaterThan(0);
    expect(screen.getByText("Reads repair targets")).toBeInTheDocument();
    expect(screen.getByText("May start durable jobs")).toBeInTheDocument();
    expect(screen.getAllByText("Readiness").length).toBeGreaterThan(0);
    expect(screen.getByText("Unresolved targets")).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Action group" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Recommended action" })).toBeInTheDocument();
    expect(screen.getAllByText("refresh_cache").length).toBeGreaterThan(0);
    expect(screen.getAllByText("repairable_stale_fallback").length).toBeGreaterThan(0);
    expect(screen.getAllByText("webdav-list-stale-anime").length).toBeGreaterThan(0);
    expect(
      screen.getByText("Cached listing is serving a stale fallback after a backend read failure."),
    ).toBeInTheDocument();
    expect(loadVfsCacheRepairActionPlan).toHaveBeenCalledTimes(1);
    expect(loadVfsCacheRepairRemediationPlan).toHaveBeenCalledTimes(1);
    expect(loadVfsCacheRepairAutomationPlan).toHaveBeenCalledWith({ enabled: true });
    expect(loadVfsCacheRepairTargets).toHaveBeenCalledWith({ limit: 20, offset: 0 });
  });

  it("shows deterministic mock VFS cache repair fallback on the Storage Staging route", async () => {
    const loadStorageStaging = vi.fn(async () => ({
      value: mockStorageStaging,
      source: "live" as const,
    }));
    window.history.pushState(null, "", "/storage/staging");

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadStorageStaging }} />);

    expect(
      await screen.findByText(/VFS cache repair action-plan data source is unavailable/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/VFS cache repair remediation-plan data source is unavailable/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/VFS cache repair automation dry-run data source is unavailable/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/VFS cache repair targets data source is unavailable/),
    ).toBeInTheDocument();
    expect(
      screen.getByText(
        "Repair actions are disabled. Refresh requires live repair action-plan data. Enqueue requires live repair targets data. Automation enqueue requires live automation dry-run data.",
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Refresh latest cache" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Enqueue first repair target" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Enqueue automation batch" })).toBeDisabled();
    expect(screen.getAllByText("webdav-list-stale-anime").length).toBeGreaterThan(0);
  });

  it("runs Storage Staging VFS cache repair refresh and enqueue actions", async () => {
    const loadStorageStaging = vi.fn(async () => ({
      value: mockStorageStaging,
      source: "live" as const,
    }));
    const loadVfsCacheRepairActionPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairActionPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairRemediationPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairRemediationPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairAutomationPlan = vi.fn(async () => ({
      value: mockVfsCacheRepairAutomationPlan,
      source: "live" as const,
    }));
    const loadVfsCacheRepairTargets = vi.fn(async () => ({
      value: mockVfsCacheRepairTargets,
      source: "live" as const,
    }));
    const refreshLatestVfsCacheRepair = vi.fn(async () => mockVfsCacheRefreshResponse);
    const enqueueVfsCacheRepairTarget = vi.fn(async () => mockVfsCacheRepairEnqueueResponse);
    const enqueueVfsCacheRepairAutomation = vi.fn(
      async () => mockVfsCacheRepairAutomationEnqueueResponse,
    );
    window.history.pushState(null, "", "/storage/staging");

    render(
      <App
        dataSource={{
          load: async () => emptyConsoleData(),
          loadStorageStaging,
          loadVfsCacheRepairActionPlan,
          loadVfsCacheRepairRemediationPlan,
          loadVfsCacheRepairAutomationPlan,
          loadVfsCacheRepairTargets,
          refreshLatestVfsCacheRepair,
          enqueueVfsCacheRepairTarget,
          enqueueVfsCacheRepairAutomation,
        }}
      />,
    );

    const refreshButton = await screen.findByRole("button", { name: "Refresh latest cache" });
    await waitFor(() => {
      expect(refreshButton).not.toBeDisabled();
    });
    fireEvent.click(refreshButton);

    await waitFor(() => {
      expect(refreshLatestVfsCacheRepair).toHaveBeenCalledTimes(1);
    });
    expect(
      await screen.findByText("Latest VFS cache refresh completed. Refreshed: yes."),
    ).toBeInTheDocument();

    const enqueueButton = screen.getByRole("button", { name: "Enqueue first repair target" });
    await waitFor(() => {
      expect(enqueueButton).not.toBeDisabled();
    });
    fireEvent.click(enqueueButton);

    await waitFor(() => {
      expect(enqueueVfsCacheRepairTarget).toHaveBeenCalledWith(
        mockVfsCacheRepairTargets.targets[0].target_ref,
        { priority: "normal" },
      );
    });
    expect(
      await screen.findByText(
        "Queued VFS cache repair job job-vfs-cache-repair-queued, status queued.",
      ),
    ).toBeInTheDocument();

    const automationButton = screen.getByRole("button", { name: "Enqueue automation batch" });
    await waitFor(() => {
      expect(automationButton).not.toBeDisabled();
    });
    fireEvent.click(automationButton);

    await waitFor(() => {
      expect(enqueueVfsCacheRepairAutomation).toHaveBeenCalledWith({
        enabled: true,
        priority: "normal",
      });
    });
    expect(
      await screen.findByText("Queued 1 VFS cache repair automation jobs; 0 already queued."),
    ).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Storage Staging route rendering", async () => {
    window.history.pushState(null, "", "/storage/staging");
    const { container } = render(<App dataSource={storageStagingDataSource()} />);

    await screen.findByText("staging-hls");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_ref");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("root_ref");
    expect(renderedText).not.toContain("cache_uri");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });

  it("keeps unsafe fields out of the Jobs route rendering", async () => {
    window.history.pushState(null, "", "/jobs");
    const { container } = render(<App dataSource={jobsDataSource()} />);

    await screen.findByRole("heading", { name: "Jobs" });
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("cache_uri");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("output_path");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
  });
});

function jobsDataSource(): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadJobs() {
      return {
        value: mockJobs,
        source: "live",
      };
    },
  };
}

function overviewDataSource(overview = mockOverview): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadOverview() {
      return {
        value: overview,
        source: "live",
      };
    },
  };
}

function librariesDataSource(): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadLibraries() {
      return {
        value: mockSystemConfig,
        source: "live",
      };
    },
  };
}

function accessDataSource(summary = mockAccessSummary): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadAccessSummary() {
      return {
        value: summary,
        source: "live",
      };
    },
  };
}

function libraryDetailDataSource(detail = libraryManagementDetail()): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadLibraryDetail() {
      return {
        value: detail,
        source: "live",
      };
    },
  };
}

function libraryManagementDetail(libraryId = "library-anime"): LibraryManagementDetail {
  return {
    configuredLibraryCount: mockSystemConfig.libraries.length,
    library: mockSystemConfig.libraries.find((library) => library.id === libraryId) ?? null,
    metadataProfile: mockLibraryMetadataProfile(libraryId),
    sourceInventory: {
      source: "live",
      sourceCount: 2,
      linkedItemCount: 1,
      probedSourceCount: 1,
      returnedSourceCount: 2,
      totalSizeBytes: 1468006400,
      latestScanJob: {
        id: "job-scan",
        kind: "library_scan",
        status: "running",
        resourceClass: "disk.scan",
        queuedAt: "2026-05-19T09:58:00Z",
        completedAt: null,
        hasError: false,
      },
      failedJobCount: 0,
      page: {
        limit: 50,
        offset: 0,
        returned: 2,
      },
      samples: [
        {
          id: "source-anime-1",
          fileName: "Episode 01.mkv",
          itemTitle: "Pilot",
          sizeBytes: 1468006400,
          hasProbe: true,
        },
      ],
    },
  };
}

function unsafeLibraryManagementDetail(): LibraryManagementDetail {
  const detail = libraryManagementDetail("library-anime");
  const library = detail.library ?? mockSystemConfig.libraries[0];

  return {
    ...detail,
    library: {
      ...library,
      root_ref: "local://unsafe-root",
      source_uri: "file:///Users/frank/media",
      local_path: "F:\\media\\library",
      token_env: "TMDB_API_KEY",
      webdav_password: "secret-provider-token",
    } as unknown as LibraryManagementDetail["library"],
    metadataProfile: {
      ...detail.metadataProfile,
      raw_provider_response: "secret-provider-token",
      provider_payload: {
        source_uri: "file:///Users/frank/provider",
      },
    } as unknown as LibraryManagementDetail["metadataProfile"],
  };
}

function settingsDataSource(systemConfig = mockSystemConfig): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadSettings() {
      return {
        value: systemConfig,
        source: "live",
      };
    },
  };
}

function acquisitionIntakeDataSource(
  acquisitionIntake = mockAcquisitionIntakeCandidates,
): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadAcquisitionIntake() {
      return {
        value: acquisitionIntake,
        source: "live",
      };
    },
  };
}

function unsafeAcquisitionIntake(): typeof mockAcquisitionIntakeCandidates {
  return {
    ...mockAcquisitionIntakeCandidates,
    candidates: mockAcquisitionIntakeCandidates.candidates.map((candidate) => ({
      ...candidate,
      source_ref_redacted: "local://unsafe-root",
      source_uri: "file:///Users/frank/drop",
      local_path: "F:\\media\\drop",
      raw_locator: "C:\\media\\drop\\movie.mkv",
      raw_token: "nako_at_intake_raw_token",
    })) as typeof mockAcquisitionIntakeCandidates.candidates,
  };
}

function generatedArtifactsDataSource(
  generatedArtifacts = mockGeneratedArtifactProposals,
): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadGeneratedArtifacts() {
      return {
        value: generatedArtifacts,
        source: "live",
      };
    },
  };
}

function generatedArtifactReviewPlanSummary({
  artifactId = "artifact-metadata-cleanup",
  decision = "accept",
}: {
  artifactId?: string;
  decision?: "accept" | "reject";
} = {}): GeneratedArtifactReviewPlanSummary {
  return {
    artifactId,
    decision,
    status: "ready",
    action: decision === "accept" ? "stage_metadata_authority_review" : "mark_rejected",
    reasons: ["ready"],
    capability: "metadata_cleanup",
    kind: "metadata_suggestion",
    target: {
      kind: "media_source",
      libraryId: "library-anime",
      itemId: "item-unknown-1",
      sourceId: "source-unknown-1",
    },
    payload: {
      validJson: true,
      shape: "object",
      payloadFingerprint: "sha256:cccccccccccccccccccccccccccccccc",
      payloadBytes: 512,
      objectFieldCount: 3,
      arrayItemCount: null,
      hasTextualValues: true,
      hasExplanation: true,
      confidenceMilli: 810,
    },
    readiness: {
      status: "ready",
      actionable: true,
      reasons: ["ready"],
    },
    boundary: {
      acceptedIntoCanonicalMetadata: false,
      writesSidecar: false,
      writesLibraryFiles: false,
      appliesImmediately: false,
      requiresMetadataAuthorityApply: decision === "accept",
    },
  };
}

function generatedArtifactReviewResultSummary({
  artifactId = "artifact-metadata-cleanup",
  decision = "accept",
}: {
  artifactId?: string;
  decision?: "accept" | "reject";
} = {}): GeneratedArtifactReviewResultSummary {
  return {
    artifactId,
    decision,
    artifactStatus: decision === "accept" ? "accepted" : "rejected",
    acceptedAt: decision === "accept" ? "2026-05-25T11:00:00Z" : null,
    idempotentReplay: false,
    plan: generatedArtifactReviewPlanSummary({ artifactId, decision }),
  };
}

function unsafeGeneratedArtifacts(): typeof mockGeneratedArtifactProposals {
  return {
    ...mockGeneratedArtifactProposals,
    proposals: mockGeneratedArtifactProposals.proposals.map((proposal) => ({
      ...proposal,
      prompt_text: "secret prompt body",
      payload_body: "secret payload body",
      raw_provider_response: "provider raw body",
      raw_token: "nako_at_generated_raw_token",
      target: {
        ...proposal.target,
        source_uri: "file:///Users/frank/generated",
        local_path: "F:\\generated\\artifact.json",
      },
      payload: {
        ...proposal.payload,
        raw_json: '{"secret":"secret payload body"}',
      },
      provenance: {
        ...proposal.provenance,
        raw_provider_response: "provider raw body",
      },
    })) as typeof mockGeneratedArtifactProposals.proposals,
  };
}

function addonsDataSource(addons = mockAddonsRouteSummary): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadAddons() {
      return {
        value: addons,
        source: "live",
      };
    },
  };
}

function unsafeAddonsRouteSummary(): AddonsRouteSummary {
  return {
    ...mockAddonsRouteSummary,
    addons: mockAddonsRouteSummary.addons.map((addon) => ({
      ...addon,
      baseUrl: "http://subtitle-lab:9100",
      raw_token: "nako_at_one_time_raw_token",
      token_env: "ADDON_SECRET_SUBTITLE_PROVIDER_KEY",
    })) as AddonsRouteSummary["addons"],
    selectedAddon: mockAddonsRouteSummary.selectedAddon
      ? ({
          ...mockAddonsRouteSummary.selectedAddon,
          baseUrl: "http://subtitle-lab:9100",
          rawManifestJson: "manifest-secret-payload",
          resourcePaths: ["/resources/subtitles", "F:\\addons\\subtitle-lab"],
        } as AddonsRouteSummary["selectedAddon"])
      : null,
    surfaceSummary: mockAddonsRouteSummary.surfaceSummary
      ? ({
          ...mockAddonsRouteSummary.surfaceSummary,
          hostedPageUrl: "http://subtitle-lab:9100/pages/diagnostics",
          hostedPagePath: "/pages/diagnostics",
        } as AddonsRouteSummary["surfaceSummary"])
      : null,
    installBoundary: mockAddonsRouteSummary.installBoundary
      ? ({
          ...mockAddonsRouteSummary.installBoundary,
          docker_compose: "docker_compose: http://subtitle-lab:9100",
          command: "curl -fsS http://subtitle-lab:9100/health",
          envVar: "ADDON_SECRET_SUBTITLE_PROVIDER_KEY",
          message: "unsafe lifecycle message http://subtitle-lab:9100 F:\\addons",
          placeholder: "secret-reference:subtitle-provider-key",
        } as AddonsRouteSummary["installBoundary"])
      : null,
  };
}

function catalogGovernanceDataSource(): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadCatalogGovernance() {
      return {
        value: mockCatalogGovernance,
        source: "live",
      };
    },
  };
}

function catalogGovernanceItemDetailSummary(
  itemId = "item-low-confidence",
): CatalogGovernanceItemDetailSummary {
  return {
    item: {
      id: itemId,
      libraryId: "library-films",
      kind: "movie",
      parentId: null,
      title: "Film Needs Mapping",
      releaseDate: "1999-01-01",
      issues: ["missing_accepted_provider_mapping"],
      sourceCount: 1,
      representativeSourceId: "source-low-confidence",
      representativeFileName: "Film Needs Mapping.mkv",
      providerMappingCount: 1,
      acceptedProviderMappingCount: 0,
      duplicateRelationshipCount: 0,
      localInference: null,
    },
    providerMappings: [
      {
        id: "mapping-tmdb-603",
        itemId,
        status: "candidate",
        confidenceMilli: 820,
        source: "provider:tmdb",
        subject: {
          id: "subject-tmdb-603",
          provider: "tmdb",
          kind: "movie",
          key: "603",
          title: "The Candidate",
          releaseYear: 2026,
          locale: "en-US",
        },
      },
    ],
    repairActions: ["provider_mapping_review"],
  };
}

function catalogGovernanceProviderMappingReviewPlanSummary({
  itemId = "item-low-confidence",
  mappingId = "mapping-tmdb-603",
  decision = "accept",
}: {
  itemId?: string;
  mappingId?: string;
  decision?: "accept" | "reject";
} = {}): CatalogGovernanceProviderMappingReviewPlanSummary {
  const detail = catalogGovernanceItemDetailSummary(itemId);
  const mapping =
    detail.providerMappings.find((candidate) => candidate.id === mappingId) ??
    detail.providerMappings[0];

  return {
    item: detail.item,
    mapping: {
      ...mapping,
      id: mappingId,
    },
    decision,
    currentStatus: "candidate",
    targetStatus: decision === "accept" ? "accepted" : "rejected",
    status: "ready",
    readiness: {
      status: "ready",
      actionable: true,
      reasons: ["provider_mapping_status_change"],
    },
    boundary: {
      updatesProviderMappingStatus: true,
      updatesCanonicalMetadata: false,
      updatesProviderSubject: false,
      updatesLocalInference: false,
      updatesSourceDuplicates: false,
      updatesHierarchy: false,
      writesNfo: false,
      writesLibraryFiles: false,
      updatesArtwork: false,
      updatesPlaybackState: false,
    },
  };
}

function catalogGovernanceProviderMappingReviewResultSummary({
  itemId = "item-low-confidence",
  mappingId = "mapping-tmdb-603",
  decision = "accept",
}: {
  itemId?: string;
  mappingId?: string;
  decision?: "accept" | "reject";
} = {}): CatalogGovernanceProviderMappingReviewResultSummary {
  return {
    itemId,
    mappingId,
    decision,
    previousStatus: "candidate",
    currentStatus: decision === "accept" ? "accepted" : "rejected",
    changed: true,
    idempotentReplay: false,
    plan: catalogGovernanceProviderMappingReviewPlanSummary({
      itemId,
      mappingId,
      decision,
    }),
  };
}

function catalogBrowseDataSource(summary = mockCatalogBrowse): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadCatalog() {
      return {
        value: summary,
        source: "live",
      };
    },
  };
}

function unsafeCatalogBrowse(): CatalogBrowseSummary {
  return {
    ...mockCatalogBrowse,
    items: mockCatalogBrowse.items.map((item) => ({
      ...item,
      source_ref: "local://unsafe-root",
      source_uri: "file:///Users/frank/media/private.mkv",
      raw_provider_response: "provider raw body",
      artifact_storage_handle: "F:\\nako\\artifact-cache\\poster.jpg",
      raw_token: "nako_at_one_time_raw_token",
    }) as CatalogBrowseSummary["items"][number]),
  };
}

function itemDetailDataSource(summary = mockItemDetailSummary("item-unknown-1")): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadItemDetail() {
      return {
        value: summary,
        source: "live",
      };
    },
  };
}

function itemArtworkGalleryDataSource(
  summary = mockItemArtworkGallerySummary("item-unknown-1"),
): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadItemArtworkGallery() {
      return {
        value: summary,
        source: "live",
      };
    },
  };
}

function unsafeItemArtworkGallery(): ItemArtworkGallerySummary {
  return {
    ...mockItemArtworkGallerySummary("item-unknown-1"),
    candidates: mockItemArtworkGallerySummary("item-unknown-1").candidates.map((candidate) => ({
      ...candidate,
      source_uri: "https://provider.example/poster.jpg?token=secret",
      storage_uri: "managed-artwork://library-anime/private/poster.jpg",
    }) as ItemArtworkGallerySummary["candidates"][number]),
    artifacts: mockItemArtworkGallerySummary("item-unknown-1").artifacts.map((artifact) => ({
      ...artifact,
      content_hash: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      local_path: "F:\\nako\\artwork\\poster.jpg",
    }) as ItemArtworkGallerySummary["artifacts"][number]),
    selected: mockItemArtworkGallerySummary("item-unknown-1").selected.map((selection) => ({
      ...selection,
      cache_uri: "managed-artwork://cache/private/poster.jpg",
      routePath: "https://provider.example/poster.jpg?token=secret",
    }) as ItemArtworkGallerySummary["selected"][number]),
  };
}

function unsafeItemDetail(): ItemDetailSummary {
  return {
    ...mockItemDetailSummary("item-unknown-1"),
    item: {
      ...mockItemDetailSummary("item-unknown-1").item,
      source_ref: "local://unsafe-root",
      raw_provider_response: "provider raw body",
      raw_token: "nako_at_one_time_raw_token",
    } as ItemDetailSummary["item"],
    sources: mockItemDetailSummary("item-unknown-1").sources.map((source) => ({
      ...source,
      source_uri: "file:///Users/frank/media/private.mkv",
      playback_output_path: "F:\\nako\\transcode\\playlist.m3u8",
    }) as ItemDetailSummary["sources"][number]),
    images: mockItemDetailSummary("item-unknown-1").images.map((image) => ({
      ...image,
      artifact_storage_handle: "F:\\nako\\artifacts\\poster.jpg",
      routePath: null,
    }) as ItemDetailSummary["images"][number]),
  };
}

function playbackSessionsDataSource(): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadPlaybackSessions() {
      return {
        value: mockPlaybackSessions,
        source: "live",
      };
    },
  };
}

function storageStagingDataSource(): AdminDataSource {
  return {
    async load() {
      return emptyConsoleData();
    },
    async loadStorageStaging() {
      return {
        value: mockStorageStaging,
        source: "live",
      };
    },
  };
}

function emptyConsoleData(): AdminConsoleData {
  return {
    sources: {
      overview: "mock" as const,
      addons: "mock" as const,
      addonHealth: "mock" as const,
      addonSurfaces: "mock" as const,
      addonInstallGuide: "mock" as const,
      addonTokens: "mock" as const,
      addonGrants: "mock" as const,
      acquisitionIntake: "mock" as const,
      generatedArtifactProposals: "mock" as const,
      catalogGovernance: "mock" as const,
      events: "mock" as const,
      jobs: "mock" as const,
      playbackSessions: "mock" as const,
      playbackRuntime: "mock" as const,
      storageStaging: "mock" as const,
      systemConfig: "mock" as const,
    },
    errors: {},
    overview: {} as never,
    addons: {} as never,
    libraries: [],
    catalog: { items: [], page: emptyPage() },
    acquisitionIntake: { candidates: [], page: emptyPage() },
    generatedArtifactProposals: { proposals: [], page: emptyPage() },
    events: { events: [], page: emptyPage() },
    jobs: [],
    playback: { hardwarePolicy: "", ffmpegStatus: "", accelerators: [], sessions: [] },
    storage: { stagingUsedBytes: 0, stagingMaxBytes: 0, vfsObjectCount: 0, records: [] },
    network: {} as never,
    settings: [],
  };
}

function emptyPage(): AdminJobListResponse["page"] {
  return {
    limit: 0,
    offset: 0,
    returned: 0,
  };
}
