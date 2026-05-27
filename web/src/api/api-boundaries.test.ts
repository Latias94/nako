import { describe, expect, it } from "vitest";

import { createAdminApi } from "@/api/admin/client";
import { fixtureAdminOverview } from "@/api/admin/fixtures";
import { createMediaApi } from "@/api/media/client";

describe("web API boundaries", () => {
  it("requests browser playback tickets through the Public Client API boundary", async () => {
    const requests: Array<{ auth: string | null; body: string | null; url: string }> = [];
    const api = createMediaApi(
      {
        mode: "live",
        baseUrl: "https://nako.example/api/",
        bearerToken: "session_secret",
      },
      async (input, init) => {
        requests.push({
          auth: new Headers(init?.headers).get("Authorization"),
          body: typeof init?.body === "string" ? init.body : null,
          url: requestUrl(input),
        });

        return jsonResponse({
          expires_at: "2026-05-28T02:00:00Z",
          item_id: "item-1",
          mode: "direct",
          source_id: "source-1",
          urls: [
            {
              content_type: "video/mp4",
              kind: "stream",
              supports_range_requests: true,
              url: "/sources/source-1/stream?ticket=opaque",
            },
          ],
        });
      },
    );

    const result = await api.createBrowserPlaybackTicket("source-1", { mode: "direct" });

    expect(result.source).toBe("live");
    expect(requests).toEqual([
      {
        auth: "Bearer session_secret",
        body: JSON.stringify({ mode: "direct" }),
        url: "https://nako.example/api/sources/source-1/playback/browser-ticket",
      },
    ]);
    expect(result.value.urls[0]?.url).toBe(
      "https://nako.example/sources/source-1/stream?ticket=opaque",
    );
    expect(JSON.stringify(result)).not.toContain("session_secret");
  });

  it("keeps Admin API calls on the generated admin namespace and falls back per section", async () => {
    const requests: Array<{ auth: string | null; url: string }> = [];
    const api = createAdminApi(
      {
        mode: "live",
        baseUrl: "http://127.0.0.1:7833/",
        token: "admin_secret",
      },
      async (input, init) => {
        requests.push({
          auth: new Headers(init?.headers).get("Authorization"),
          url: requestUrl(input),
        });

        return jsonResponse({ code: "unavailable", message: "server offline" }, 503);
      },
    );

    const result = await api.getOverview();

    expect(requests).toEqual([
      {
        auth: "Bearer admin_secret",
        url: "http://127.0.0.1:7833/admin/v1/overview",
      },
    ]);
    expect(result.source).toBe("fixture");
    expect(result.value).toEqual(fixtureAdminOverview);
    expect(result.error).toContain("HTTP 503");
    expect(JSON.stringify(result)).not.toContain("admin_secret");
  });
});

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
    },
  });
}

function requestUrl(input: RequestInfo | URL): string {
  if (typeof input === "string") {
    return input;
  }

  if (input instanceof URL) {
    return input.toString();
  }

  return input.url;
}
