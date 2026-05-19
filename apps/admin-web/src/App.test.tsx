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
          overviewSource: "live",
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

  it("does not render unsafe locator, local path, or token value fields", async () => {
    const dataSource: AdminDataSource = {
      async load() {
        return {
          ...mockAdminConsoleData,
          overviewSource: "live",
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
