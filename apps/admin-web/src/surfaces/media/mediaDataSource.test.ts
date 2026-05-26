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
    const library = await dataSource.getLibrary("library-anime");
    const sources = await dataSource.listLibrarySources("library-anime");
    const item = await dataSource.getItem("item-episode-1");

    expect(dataSource.source).toBe("fixture");
    expect(dataSource.label).toBe("Fixture mode");
    expect(libraries.source).toBe("fixture");
    expect(libraries.value.libraries[0].name).toBe("Anime Vault");
    expect(library.value.library.name).toBe("Anime Vault");
    expect(sources.value.sources[0].source.file_name).toBe("Pilot.mkv");
    expect(item.value.item.metadata.title).toBe("Pilot");
  });

  it("uses generated Public Client SDK routes for library and item detail", async () => {
    const fetch = vi.fn(async (input: string | URL | Request, _init?: RequestInit) =>
      jsonResponse({ ok: true, path: String(input) }),
    );
    const dataSource = createPublicClientMediaDataSource(
      {
        baseUrl: "http://nako.test",
        bearerToken: "secret-token",
        mode: "live",
      },
      fetch,
    );

    await dataSource.getLibrary("library anime");
    await dataSource.listLibrarySources("library anime", { limit: 10, offset: 5 });
    await dataSource.getItem("item episode");

    expect(fetch).toHaveBeenNthCalledWith(
      1,
      "http://nako.test/libraries/library%20anime",
      expect.objectContaining({ method: "GET" }),
    );
    expect(fetch).toHaveBeenNthCalledWith(
      2,
      "http://nako.test/libraries/library%20anime/sources?limit=10&offset=5",
      expect.objectContaining({ method: "GET" }),
    );
    expect(fetch).toHaveBeenNthCalledWith(
      3,
      "http://nako.test/items/item%20episode",
      expect.objectContaining({ method: "GET" }),
    );
  });

  it("uses generated Public Client SDK routes for playback decisions and user playback state", async () => {
    const fetch = vi.fn(async (input: string | URL | Request, _init?: RequestInit) =>
      jsonResponse({ ok: true, path: String(input) }),
    );
    const dataSource = createPublicClientMediaDataSource(
      {
        baseUrl: "http://nako.test",
        bearerToken: "secret-token",
        mode: "live",
      },
      fetch,
    );

    await dataSource.getPlaybackDecision("source episode", { direct_play: true });
    await dataSource.getUserPlaybackState("item episode");
    await dataSource.updateUserPlaybackProgress("item episode", {
      duration_ms: 1440000,
      position_ms: 90000,
      source_id: "source episode",
    });
    await dataSource.setUserWatchedState("item episode", {
      duration_ms: 1440000,
      position_ms: 1440000,
      source_id: "source episode",
      watched: true,
    });

    expect(fetch).toHaveBeenNthCalledWith(
      1,
      "http://nako.test/sources/source%20episode/playback/decision?direct_play=true",
      expect.objectContaining({ method: "GET" }),
    );
    expect(fetch).toHaveBeenNthCalledWith(
      2,
      "http://nako.test/users/me/playback-state/items/item%20episode",
      expect.objectContaining({ method: "GET" }),
    );
    expect(fetch).toHaveBeenNthCalledWith(
      3,
      "http://nako.test/users/me/playback-state/items/item%20episode/progress",
      expect.objectContaining({
        body: JSON.stringify({
          duration_ms: 1440000,
          position_ms: 90000,
          source_id: "source episode",
        }),
        method: "PUT",
      }),
    );
    expect(fetch).toHaveBeenNthCalledWith(
      4,
      "http://nako.test/users/me/playback-state/items/item%20episode/watched",
      expect.objectContaining({
        body: JSON.stringify({
          duration_ms: 1440000,
          position_ms: 1440000,
          source_id: "source episode",
          watched: true,
        }),
        method: "PUT",
      }),
    );
  });

  it("uses generated Public Client SDK route for browser playback tickets", async () => {
    const fetch = vi.fn(async (_input: string | URL | Request, _init?: RequestInit) =>
      jsonResponse({
        expires_at: "2026-05-26T12:00:00Z",
        item_id: "item episode",
        mode: "direct",
        source_id: "source episode",
        urls: [
          {
            content_type: "video/mp4",
            kind: "stream",
            supports_range_requests: true,
            url: "/sources/source%20episode/stream?ticket=nako_bpt_secret",
          },
        ],
      }),
    );
    const dataSource = createPublicClientMediaDataSource(
      {
        baseUrl: "http://nako.test/api/",
        bearerToken: "secret-token",
        mode: "live",
      },
      fetch,
    );

    const ticket = await dataSource.createBrowserPlaybackTicket("source episode", {
      capabilities: { direct_play: true },
      mode: "direct",
    });

    expect(fetch).toHaveBeenCalledWith(
      "http://nako.test/api/sources/source%20episode/playback/browser-ticket",
      expect.objectContaining({
        body: JSON.stringify({
          capabilities: { direct_play: true },
          mode: "direct",
        }),
        method: "POST",
      }),
    );
    const headers = fetch.mock.calls[0][1]?.headers as Headers;
    expect(headers.get("Authorization")).toBe("Bearer secret-token");
    expect(ticket.value.urls[0].url).toBe(
      "http://nako.test/sources/source%20episode/stream?ticket=nako_bpt_secret",
    );
    expect(ticket.value.urls[0].url).not.toContain("Bearer");
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
