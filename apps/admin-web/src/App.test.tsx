import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { App } from "./App";
import type { AdminDataSource } from "./adminApi/dataSource";
import { mockAdminConsoleData } from "./adminApi/mockData";

describe("Admin web console scaffold", () => {
  it("renders live overview data through the Admin API boundary", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return {
          ...mockAdminConsoleData,
          sources: {
            ...mockAdminConsoleData.sources,
            overview: "live",
            addons: "live",
            addonHealth: "live",
            addonSurfaces: "live",
            addonInstallGuide: "live",
            addonTokens: "live",
            addonGrants: "live",
            acquisitionIntake: "live",
            generatedArtifactProposals: "live",
            jobs: "live",
            playbackRuntime: "live",
            storageStaging: "live",
            systemConfig: "live",
          },
          overview: {
            ...mockAdminConsoleData.overview,
            storage: {
              ...mockAdminConsoleData.overview.storage,
              total_backends: 1,
              ready_backends: 1,
              degraded_backends: 0,
              backends: [
                {
                  library_id: "library-live",
                  library_name: "Live Library",
                  backend_kind: "local",
                  status: "ready",
                },
              ],
            },
          },
        };
      },
    };

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Live Library")).toBeInTheDocument();
    expect(screen.getByText("Acquisition Intake")).toBeInTheDocument();
    expect(screen.getByText("Addon Operations")).toBeInTheDocument();
    expect(screen.getAllByText("Subtitle Lab").length).toBeGreaterThan(0);
    expect(screen.getByText("Health: reachable · 42 ms")).toBeInTheDocument();
    expect(screen.getByText("external and untrusted")).toBeInTheDocument();
    expect(screen.getByText("Addon Install Guide")).toBeInTheDocument();
    expect(screen.getByText("compose.dev-taru-subtitle-lab.yml")).toBeInTheDocument();
    expect(screen.getByText("dev-taru-subtitle-lab.service")).toBeInTheDocument();
    expect(screen.getByText(/Taru generates this guide only/)).toBeInTheDocument();
    expect(screen.getByText("subtitle · succeeded")).toBeInTheDocument();
    expect(screen.getByText("Network Access")).toBeInTheDocument();
    expect(screen.getByText("Generated Artifacts")).toBeInTheDocument();
    expect(screen.getByText("reverse_proxy")).toBeInTheDocument();
    expect(screen.getByText("candidate-ready")).toBeInTheDocument();
    expect(screen.getByText("metadata_cleanup")).toBeInTheDocument();
    expect(screen.getAllByText("Live Admin API").length).toBeGreaterThan(0);
    expect(screen.getByText("1/1")).toBeInTheDocument();
  });

  it("falls back to safe mock data when the Admin API is unavailable", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        throw new Error("Admin API is offline");
      },
    };

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("Admin API is offline")).toBeInTheDocument();
    await waitFor(() => {
      expect(screen.getAllByText("Anime Vault").length).toBeGreaterThan(0);
    });
    expect(screen.getAllByText("Mock data").length).toBeGreaterThan(0);
  });

  it("shows a section fallback summary when individual Admin API read models use mock data", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return {
          ...mockAdminConsoleData,
          errors: {
            jobs: "Admin API request failed with HTTP 503",
            playbackRuntime: "Admin API request failed with HTTP 503",
          },
        };
      },
    };

    render(<App dataSource={dataSource} />);

    expect(await screen.findByText("2 Admin API read models are using safe mock data.")).toBeInTheDocument();
  });

  it("does not render unsafe locator, local path, or token value fields", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return {
          ...mockAdminConsoleData,
          sources: {
            ...mockAdminConsoleData.sources,
            overview: "live",
            addons: "live",
            addonHealth: "live",
            addonSurfaces: "live",
            addonInstallGuide: "live",
            addonTokens: "live",
            addonGrants: "live",
            acquisitionIntake: "live",
            generatedArtifactProposals: "live",
            jobs: "live",
            playbackRuntime: "live",
            storageStaging: "live",
            systemConfig: "live",
          },
        };
      },
    };

    const { container } = render(<App dataSource={dataSource} />);

    await screen.findByText("Server operations and media governance");
    const renderedText = container.textContent ?? "";

    expect(renderedText).not.toContain("source_uri");
    expect(renderedText).not.toContain("cache_uri");
    expect(renderedText).not.toContain("local_path");
    expect(renderedText).not.toContain("output_path");
    expect(renderedText).not.toContain("prompt_json");
    expect(renderedText).not.toContain("artifact_json");
    expect(renderedText).not.toContain("local:///");
    expect(renderedText).not.toContain("raw_token");
    expect(renderedText).not.toContain("bearer");
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
    expect(renderedText).not.toContain("redacted-test-token");
  });

  it("runs safe Addon operations through data-source actions", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return mockAdminConsoleData;
      },
      async setAddonStatus() {
        return {
          ...mockAdminConsoleData.addons,
          selectedAddon: mockAdminConsoleData.addons.selectedAddon
            ? {
                ...mockAdminConsoleData.addons.selectedAddon,
                status: "disabled",
              }
            : null,
          addons: mockAdminConsoleData.addons.addons.map((addon) =>
            addon.id === "addon-subtitle-lab" ? { ...addon, status: "disabled" } : addon,
          ),
        };
      },
      async checkAddonHealth() {
        return {
          addonId: "addon-subtitle-lab",
          status: "degraded",
          latencyMs: 120,
          protocolVersion: "2026-05-15",
          addonVersion: "0.3.0",
          resourceCount: 2,
          safeErrorCode: "latency_budget_exceeded",
        };
      },
      async diagnoseAddonResource() {
        return {
          addonId: "addon-subtitle-lab",
          resource: "subtitle",
          status: "retryable_http_failure",
          latencyMs: 140,
          attempts: 2,
          httpStatus: 503,
          safeErrorCode: "upstream_unavailable",
        };
      },
    };

    render(<App dataSource={dataSource} />);

    fireEvent.click(await screen.findByText("Disable Addon"));
    expect(await screen.findByText("Subtitle Lab disabled")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Run health check"));
    expect(await screen.findByText("Subtitle Lab health degraded")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Run resource diagnostic"));
    expect(await screen.findByText("Subtitle Lab diagnostic retryable_http_failure")).toBeInTheDocument();
    expect(screen.getByText("subtitle · retryable_http_failure")).toBeInTheDocument();
  });

  it("runs Addon credential and grant onboarding actions with one-time token display", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return mockAdminConsoleData;
      },
      async issueAddonToken() {
        return {
          rawToken: "taru_at_one_time_raw_token",
          token: {
            id: "addon-token-new",
            label: "sidecar runtime",
            tokenPrefix: "taru_at_new",
            status: "active",
            lastUsedAt: null,
          },
        };
      },
      async rotateAddonToken() {
        return {
          rawToken: "taru_at_rotated_one_time_raw_token",
          token: {
            id: "addon-token-rotated",
            label: "rotated runtime",
            tokenPrefix: "taru_at_rotated",
            status: "active",
            lastUsedAt: null,
          },
        };
      },
      async revokeAddonToken() {
        return {
          id: "addon-token-active",
          label: "sidecar runtime",
          tokenPrefix: "taru_at_subtitle",
          status: "revoked",
          lastUsedAt: "2026-05-22T02:44:00.000Z",
        };
      },
      async replaceAddonGrants() {
        return [
          {
            id: "addon-grant-metadata",
            permission: "metadata_write",
            libraryId: null,
          },
        ];
      },
    };

    render(<App dataSource={dataSource} />);

    await screen.findByText("Addon Credentials & Grants");
    fireEvent.change(screen.getByLabelText("Addon token label"), {
      target: { value: "sidecar runtime" },
    });
    fireEvent.click(screen.getByText("Issue token"));
    expect(await screen.findByText("Copy this Addon Token now. It will not be shown again.")).toBeInTheDocument();
    expect(screen.getByText("taru_at_one_time_raw_token")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Rotate first token"));
    expect(await screen.findByText("taru_at_rotated_one_time_raw_token")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Revoke first token"));
    expect(await screen.findByText("Token addon-token-active revoked")).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText("Addon grant permission"), {
      target: { value: "metadata_write" },
    });
    fireEvent.click(screen.getByText("Replace grants"));
    expect(await screen.findByText("metadata_write · global")).toBeInTheDocument();
    expect(screen.getByText("Enable readiness")).toBeInTheDocument();
    expect(screen.getByText("Sidecar lifecycle remains external")).toBeInTheDocument();
  });

  it("registers pasted Addon manifest JSON from the onboarding panel and keeps the sidecar lifecycle external", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return mockAdminConsoleData;
      },
      previewAddonManifestJson(manifestJson) {
        const manifest = JSON.parse(manifestJson);
        return {
          status: "ready",
          manifest,
          summary: {
            manifestId: manifest.id,
            name: manifest.name,
            version: manifest.version,
            protocolVersion: manifest.protocol_version,
            baseUrl: manifest.base_url,
            resourceCount: manifest.resources.length,
            declaredScopes: manifest.scopes,
            secretReferenceCount: manifest.secret_reference_fields?.length ?? 0,
          },
        };
      },
      async registerAddonManifestJson() {
        return {
          status: "registered",
          addon: {
            id: "addon-subtitle-lab",
            manifestId: "dev.taru.subtitle-lab",
            name: "Subtitle Lab",
            version: "0.3.0",
            protocolVersion: "2026-05-15",
            baseUrl: "http://subtitle-lab:9100",
            status: "disabled",
            resourceCount: 2,
            grantedScopes: [],
          },
          nextSteps: [
            "Open the generated Addon Install Guide",
            "Start the Addon Sidecar outside Taru",
            "Run Addon Health Check before enabling",
          ],
        };
      },
    };

    render(<App dataSource={dataSource} />);

    const manifestBox = await screen.findByLabelText("Addon manifest JSON");
    fireEvent.change(manifestBox, {
      target: {
        value: JSON.stringify(
          {
            id: "dev.taru.subtitle-lab",
            name: "Subtitle Lab",
            version: "0.3.0",
            protocol_version: "2026-05-15",
            base_url: "http://subtitle-lab:9100",
            description: "Suggests subtitle sidecars and metadata improvements.",
            resources: [
              {
                kind: "subtitle",
                path: "/resources/subtitles",
                input_schema: null,
                output_schema: null,
                required_scopes: ["subtitle_read"],
                timeout_ms: 5000,
                max_attempts: 2,
              },
              {
                kind: "metadata",
                path: "/resources/metadata",
                input_schema: null,
                output_schema: null,
                required_scopes: ["item_metadata_read"],
                timeout_ms: null,
                max_attempts: null,
              },
            ],
            auth: "bearer",
            default_timeout_ms: 5000,
            default_max_attempts: 2,
            scopes: ["subtitle_read", "item_metadata_read"],
          },
          null,
          2,
        ),
      },
    });

    expect(screen.getByText("dev.taru.subtitle-lab · 2 resources")).toBeInTheDocument();

    fireEvent.click(screen.getByText("Register disabled Addon"));

    expect((await screen.findAllByText("Subtitle Lab registered as disabled")).length).toBeGreaterThan(0);
    expect(screen.getByText("Start the Addon Sidecar outside Taru")).toBeInTheDocument();
    expect(screen.getByText(/Registration does not install or start the/)).toBeInTheDocument();
  });
});
