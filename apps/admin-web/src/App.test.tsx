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
  AdminStorageStagingQuery,
  AddonsRouteSummary,
} from "./adminApi/types";
import {
  mockAdminConsoleData,
  mockAcquisitionIntakeCandidates,
  mockAddonsRouteSummary,
  mockCatalogGovernance,
  mockGeneratedArtifactProposals,
  mockJobs,
  mockOverview,
  mockPlaybackSessions,
  mockStorageStaging,
  mockSystemConfig,
} from "./adminApi/mockData";

afterEach(() => {
  window.history.pushState(null, "", "/");
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
    expect(await screen.findByText("Live Admin API")).toBeInTheDocument();
    expect(screen.getAllByText("Storage backends").length).toBeGreaterThan(0);
    expect(screen.getByText("2/3 ready")).toBeInTheDocument();
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

    window.history.pushState(null, "", "/jobs");
    render(<App dataSource={dataSource} />);

    expect(await screen.findByText(/HTTP 503/)).toBeInTheDocument();
    expect(screen.getByText("Mock fallback")).toBeInTheDocument();
    expect(screen.getByText("job-scan")).toBeInTheDocument();
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
    window.history.pushState(null, "", "/settings");
    const { container } = render(<App dataSource={settingsDataSource(unsafeSystemConfig)} />);

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

  it("maps Playback Sessions URL search params into generated query fields", async () => {
    const loadPlaybackSessions = vi.fn(async (query?: AdminPlaybackSessionsQuery) => ({
      value: mockPlaybackSessions,
      source: "live" as const,
      query,
    }));
    window.history.pushState(
      null,
      "",
      "/playback/sessions?source_id=source-hls&kind=hls_transcode&state=running&limit=10&offset=20",
    );

    render(<App dataSource={{ load: async () => emptyConsoleData(), loadPlaybackSessions }} />);

    await waitFor(() => {
      expect(loadPlaybackSessions).toHaveBeenCalledWith({
        source_id: "source-hls",
        kind: "hls_transcode",
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
    expect(screen.getByText("hls_transcode")).toBeInTheDocument();
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadPlaybackSessions).toHaveBeenCalledWith({
      source_id: undefined,
      kind: undefined,
      state: undefined,
      limit: 20,
      offset: 0,
    });
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
    expect(screen.getAllByText("ffmpeg_input").length).toBeGreaterThan(0);
    expect(screen.getByText("Live Admin API")).toBeInTheDocument();
    expect(loadStorageStaging).toHaveBeenCalledWith({
      purpose: undefined,
      state: undefined,
      limit: 20,
      offset: 0,
    });
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
