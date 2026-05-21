import { render, screen, waitFor } from "@testing-library/react";
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
            acquisitionIntake: "live",
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
    expect(screen.getByText("candidate-ready")).toBeInTheDocument();
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
            acquisitionIntake: "live",
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
    expect(renderedText).not.toContain("C:\\");
    expect(renderedText).not.toContain("F:\\");
    expect(renderedText).not.toContain("/Users/");
    expect(renderedText).not.toContain("redacted-test-token");
  });
});
