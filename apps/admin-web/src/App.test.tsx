import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { App } from "./App";
import type { AdminConsoleData, AdminDataSource } from "./adminApi/dataSource";
import type { AdminJobListResponse, AdminJobsQuery } from "./adminApi/types";
import { mockAdminConsoleData, mockJobs } from "./adminApi/mockData";

afterEach(() => {
  window.history.pushState(null, "", "/");
});

describe("Admin Web V2 route shell", () => {
  it("redirects to the route-owned Jobs proof by default", async () => {
    render(<App dataSource={jobsDataSource()} />);

    expect(await screen.findByRole("heading", { name: "Jobs" })).toBeInTheDocument();
    expect(screen.getByText("Admin Web V2")).toBeInTheDocument();
    expect(screen.getByText("Legacy Console")).toBeInTheDocument();
    expect(await screen.findByText("Live Admin API")).toBeInTheDocument();
    expect(screen.getByText("library_scan")).toBeInTheDocument();
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
      "/jobs?status=failed&kind=metadata_refresh&resource_class=metadata&library_id=library-films&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadJobs }} />);

    await waitFor(() => {
      expect(loadJobs).toHaveBeenCalledWith({
        status: "failed",
        kind: "metadata_refresh",
        resource_class: "metadata",
        library_id: "library-films",
        source_id: undefined,
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

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("job-scan")).toBeInTheDocument();
  });

  it("keeps unsafe fields out of the Jobs route rendering", async () => {
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
