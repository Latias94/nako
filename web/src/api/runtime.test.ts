import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, describe, expect, it, vi } from "vitest";

import {
  bootstrapDesktopConnection,
  configureServerConnection,
  normalizeServerBaseUrl,
  readConfiguredServerBaseUrl,
} from "@/api/runtime";

describe("runtime server connection", () => {
  afterEach(() => {
    window.localStorage.clear();
    window.sessionStorage.clear();
    clearMocks();
    vi.restoreAllMocks();
  });

  it("normalizes and validates server base URLs before storing them", async () => {
    expect(normalizeServerBaseUrl(" http://127.0.0.1:7833/ ")).toBe("http://127.0.0.1:7833");
    expect(normalizeServerBaseUrl("https://nako.example/base/")).toBe(
      "https://nako.example/base",
    );
    expect(() => normalizeServerBaseUrl("file:///library")).toThrow("http or https");
    expect(() => normalizeServerBaseUrl("https://user:secret@nako.example")).toThrow(
      "must not include credentials",
    );
  });

  it("checks public health before storing a browser server profile", async () => {
    const requests: string[] = [];
    const fetcher = vi.fn(async (input: RequestInfo | URL) => {
      requests.push(requestUrl(input));
      return jsonResponse({ status: "ok", version: "v1" });
    });

    const result = await configureServerConnection(" http://127.0.0.1:7833/ ", fetcher);

    expect(result).toMatchObject({
      apiVersion: "v1",
      baseUrl: "http://127.0.0.1:7833",
      status: "ok",
    });
    expect(requests).toEqual(["http://127.0.0.1:7833/health"]);
    expect(readConfiguredServerBaseUrl()).toBe("http://127.0.0.1:7833");
  });

  it("applies Tauri desktop bootstrap profiles without storing credentials", async () => {
    mockIPC((cmd) => {
      if (cmd !== "desktop_bootstrap") {
        throw new Error(`unexpected command ${cmd}`);
      }

      return {
        runtime: "tauri_desktop",
        profile: {
          baseUrl: "https://nako.example",
          source: "environment",
        },
        nativePlayback: {
          available: false,
          reason: "native_playback_core_not_integrated",
        },
      };
    });

    const bootstrap = await bootstrapDesktopConnection();

    expect(bootstrap?.runtime).toBe("tauri_desktop");
    expect(readConfiguredServerBaseUrl()).toBe("https://nako.example");
    expect(JSON.stringify(bootstrap)).not.toContain("secret");
  });

  it("saves verified profiles through the Tauri shell when available", async () => {
    const commands: Array<{ cmd: string; payload: unknown }> = [];
    mockIPC((cmd, payload) => {
      commands.push({ cmd, payload });

      if (cmd === "save_server_profile") {
        return {
          runtime: "tauri_desktop",
          profile: {
            baseUrl: "https://nako.example",
            source: "session",
          },
          nativePlayback: {
            available: false,
            reason: "native_playback_core_not_integrated",
          },
        };
      }

      throw new Error(`unexpected command ${cmd}`);
    });

    const result = await configureServerConnection("https://nako.example/", async () =>
      jsonResponse({ status: "ok", version: "v1" }),
    );

    expect(result.desktop?.profile?.source).toBe("session");
    expect(readConfiguredServerBaseUrl()).toBe("https://nako.example");
    expect(commands).toEqual([
      {
        cmd: "save_server_profile",
        payload: {
          input: {
            baseUrl: "https://nako.example",
          },
        },
      },
    ]);
    expect(JSON.stringify(commands)).not.toContain("Bearer");
  });
});

function jsonResponse(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "Content-Type": "application/json",
      "x-nako-api-version": "v1",
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
