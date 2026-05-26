import { describe, expect, it, vi } from "vitest";

import { createFixtureMediaDataSource, createPublicClientMediaDataSource } from "./mediaDataSource";

describe("Media Web Public Client data source", () => {
  it("uses the generated Public Client SDK with bearer auth for live libraries", async () => {
    const fetch = vi.fn(async (_input: string | URL | Request, _init?: RequestInit) =>
      jsonResponse({
        libraries: [],
        page: { limit: 24, offset: 0, returned: 0 },
      }),
    );

    const dataSource = createPublicClientMediaDataSource(
      {
        baseUrl: "http://nako.test/",
        bearerToken: "secret-token",
        mode: "live",
      },
      fetch,
    );

    await dataSource.listLibraries({ limit: 24, offset: 0 });

    expect(fetch).toHaveBeenCalledWith(
      "http://nako.test/libraries?limit=24&offset=0",
      expect.objectContaining({
        method: "GET",
        headers: expect.any(Headers),
      }),
    );
    const headers = fetch.mock.calls[0][1]?.headers as Headers;
    expect(headers.get("Authorization")).toBe("Bearer secret-token");
  });

  it("keeps fixture mode explicitly separated from live requests", async () => {
    const dataSource = createFixtureMediaDataSource();

    const libraries = await dataSource.listLibraries();

    expect(dataSource.source).toBe("fixture");
    expect(dataSource.label).toBe("Fixture mode");
    expect(libraries.source).toBe("fixture");
    expect(libraries.value.libraries[0].name).toBe("Anime Vault");
  });
});

function jsonResponse(body: unknown) {
  return new Response(JSON.stringify(body), {
    headers: {
      "content-type": "application/json",
      "x-nako-api-version": "v1",
    },
    status: 200,
  });
}
