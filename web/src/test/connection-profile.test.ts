import { afterEach, describe, expect, it } from "vitest"
import {
  CONNECTION_PROFILE_STORAGE_KEY,
  CONNECTION_SESSION_STORAGE_KEY,
  createBrowserConnectionProfileStore,
  createLiveConnectionProfile,
  detectNakoRuntime,
  loadConnectionState,
  normalizeServerBaseUrl,
  saveApiClientConnection,
  saveConnectionState,
  toApiClientConnection,
} from "@/src/api/connection-profile"
import { loadAdminApiConnection } from "@/src/api/admin/connection"
import { loadPublicClientConnection, savePublicClientConnection } from "@/src/api/public/connection"

function createMemoryStorage() {
  const entries = new Map<string, string>()

  return {
    getItem(key: string) {
      return entries.get(key) ?? null
    },
    setItem(key: string, value: string) {
      entries.set(key, value)
    },
    removeItem(key: string) {
      entries.delete(key)
    },
  }
}

afterEach(() => {
  delete (window as typeof window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__
})

describe("connection profile boundary", () => {
  it("normalizes server URLs without carrying credentials or query state", () => {
    expect(normalizeServerBaseUrl("nako.test/library?debug=1#frag", "8096")).toBe(
      "http://nako.test:8096/library",
    )
    expect(() => normalizeServerBaseUrl("https://user:pass@nako.test")).toThrow(
      /must not include credentials/,
    )
    expect(() => normalizeServerBaseUrl("ftp://nako.test")).toThrow(/http or https/)
  })

  it("stores profile and session state separately so bearer tokens do not enter the profile", () => {
    const profileStorage = createMemoryStorage()
    const sessionStorage = createMemoryStorage()
    const store = createBrowserConnectionProfileStore({
      profileStorage,
      sessionStorage,
      runtime: "browser",
    })

    saveConnectionState(
      {
        profile: createLiveConnectionProfile({
          baseUrl: "nako.test",
          port: "8096",
          runtime: "browser",
          serverName: "Home",
        }),
        session: {
          bearerToken: "secret-token",
          principalId: "admin",
        },
      },
      store,
    )

    expect(profileStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)).not.toContain("secret-token")
    expect(sessionStorage.getItem(CONNECTION_SESSION_STORAGE_KEY)).toContain("secret-token")
    expect(toApiClientConnection(loadConnectionState(store))).toEqual({
      mode: "live",
      baseUrl: "http://nako.test:8096",
      bearerToken: "secret-token",
    })
  })

  it("keeps legacy Public/Admin connection helpers on the shared profile boundary", () => {
    savePublicClientConnection({
      mode: "live",
      baseUrl: "http://nako.test",
      bearerToken: "public-token",
    })

    expect(window.localStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)).not.toContain("public-token")
    expect(loadPublicClientConnection()).toEqual({
      mode: "live",
      baseUrl: "http://nako.test",
      bearerToken: "public-token",
    })
    expect(loadAdminApiConnection()).toEqual(loadPublicClientConnection())
  })

  it("can adapt live API connections without persisting bearer tokens in profile storage", () => {
    const profileStorage = createMemoryStorage()
    const sessionStorage = createMemoryStorage()
    const store = createBrowserConnectionProfileStore({ profileStorage, sessionStorage })

    saveApiClientConnection(
      {
        mode: "live",
        baseUrl: "https://nako.example",
        bearerToken: "api-token",
      },
      store,
    )

    expect(profileStorage.getItem(CONNECTION_PROFILE_STORAGE_KEY)).not.toContain("api-token")
    expect(toApiClientConnection(loadConnectionState(store))).toEqual({
      mode: "live",
      baseUrl: "https://nako.example",
      bearerToken: "api-token",
    })
  })

  it("detects Tauri runtime without requiring Tauri APIs in browser tests", () => {
    expect(detectNakoRuntime()).toBe("browser")

    Object.defineProperty(window, "__TAURI_INTERNALS__", {
      configurable: true,
      value: {},
    })

    expect(detectNakoRuntime()).toBe("tauri")
  })
})
