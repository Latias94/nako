import { describe, expect, it, vi } from "vitest";

import { AdminApiClient } from "./client";
import { mockOverview } from "./mockData";

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
    expect(fetcher).toHaveBeenCalledWith("http://127.0.0.1:3000/admin/v1/overview", {
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
});
