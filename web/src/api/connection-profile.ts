export type NakoRuntimeMode = "browser" | "tauri"

export type NakoConnectionProfile =
  | {
      mode: "fixture"
      runtime: NakoRuntimeMode
    }
  | {
      mode: "live"
      runtime: NakoRuntimeMode
      baseUrl: string
      serverName?: string
    }

export interface NakoSessionState {
  bearerToken?: string
  principalId?: string
  selectedUserId?: string
}

export interface NakoConnectionState {
  profile: NakoConnectionProfile
  session: NakoSessionState | null
}

export type ApiClientConnection =
  | {
      mode: "fixture"
    }
  | {
      mode: "live"
      baseUrl: string
      bearerToken?: string
    }

interface KeyValueStorage {
  getItem(key: string): string | null
  setItem(key: string, value: string): void
  removeItem(key: string): void
}

export interface ConnectionProfileStore {
  loadProfile(): NakoConnectionProfile
  saveProfile(profile: NakoConnectionProfile): void
  loadSession(): NakoSessionState | null
  saveSession(session: NakoSessionState | null): void
  clear(): void
}

export const CONNECTION_PROFILE_STORAGE_KEY = "nako.connection.profile.v1"
export const CONNECTION_SESSION_STORAGE_KEY = "nako.connection.session.v1"

export function detectNakoRuntime(): NakoRuntimeMode {
  if (typeof window === "undefined") {
    return "browser"
  }

  return "__TAURI_INTERNALS__" in window ? "tauri" : "browser"
}

export function normalizeServerBaseUrl(input: string, port?: string): string {
  const trimmedInput = input.trim()
  if (!trimmedInput) {
    throw new Error("Server URL is required")
  }

  const withScheme = /^[a-z][a-z\d+\-.]*:\/\//i.test(trimmedInput)
    ? trimmedInput
    : `http://${trimmedInput}`
  const url = new URL(withScheme)

  if (url.protocol !== "http:" && url.protocol !== "https:") {
    throw new Error("Server URL must use http or https")
  }

  if (url.username || url.password) {
    throw new Error("Server URL must not include credentials")
  }

  const trimmedPort = port?.trim()
  if (trimmedPort) {
    if (!/^\d+$/.test(trimmedPort)) {
      throw new Error("Server port must be numeric")
    }

    url.port = trimmedPort
  }

  url.hash = ""
  url.search = ""

  return url.toString().replace(/\/$/, "")
}

export function createLiveConnectionProfile(
  input: {
    baseUrl: string
    port?: string
    serverName?: string
    runtime?: NakoRuntimeMode
  },
): NakoConnectionProfile {
  return {
    mode: "live",
    runtime: input.runtime ?? detectNakoRuntime(),
    baseUrl: normalizeServerBaseUrl(input.baseUrl, input.port),
    serverName: input.serverName?.trim() || undefined,
  }
}

export function createFixtureConnectionProfile(runtime: NakoRuntimeMode = detectNakoRuntime()): NakoConnectionProfile {
  return {
    mode: "fixture",
    runtime,
  }
}

export function createBrowserConnectionProfileStore(
  options: {
    profileStorage?: KeyValueStorage
    sessionStorage?: KeyValueStorage
    runtime?: NakoRuntimeMode
  } = {},
): ConnectionProfileStore {
  const profileStorage = options.profileStorage ?? browserLocalStorage()
  const sessionStorage = options.sessionStorage ?? browserSessionStorage()
  const runtime = options.runtime ?? detectNakoRuntime()

  return {
    loadProfile() {
      const raw = profileStorage?.getItem(CONNECTION_PROFILE_STORAGE_KEY)
      if (!raw) {
        return createFixtureConnectionProfile(runtime)
      }

      try {
        return parseProfile(raw, runtime)
      } catch {
        return createFixtureConnectionProfile(runtime)
      }
    },
    saveProfile(profile) {
      if (!profileStorage) return

      if (profile.mode === "fixture") {
        profileStorage.removeItem(CONNECTION_PROFILE_STORAGE_KEY)
        return
      }

      profileStorage.setItem(CONNECTION_PROFILE_STORAGE_KEY, JSON.stringify(profile))
    },
    loadSession() {
      const raw = sessionStorage?.getItem(CONNECTION_SESSION_STORAGE_KEY)
      if (!raw) {
        return null
      }

      try {
        return parseSession(raw)
      } catch {
        return null
      }
    },
    saveSession(session) {
      if (!sessionStorage) return

      if (!session || isEmptySession(session)) {
        sessionStorage.removeItem(CONNECTION_SESSION_STORAGE_KEY)
        return
      }

      sessionStorage.setItem(CONNECTION_SESSION_STORAGE_KEY, JSON.stringify(session))
    },
    clear() {
      profileStorage?.removeItem(CONNECTION_PROFILE_STORAGE_KEY)
      sessionStorage?.removeItem(CONNECTION_SESSION_STORAGE_KEY)
    },
  }
}

export function loadConnectionState(store: ConnectionProfileStore = createBrowserConnectionProfileStore()): NakoConnectionState {
  return {
    profile: store.loadProfile(),
    session: store.loadSession(),
  }
}

export function saveConnectionState(
  state: NakoConnectionState,
  store: ConnectionProfileStore = createBrowserConnectionProfileStore(),
) {
  store.saveProfile(state.profile)
  store.saveSession(state.session)
}

export function toApiClientConnection(state: NakoConnectionState): ApiClientConnection {
  if (state.profile.mode === "fixture") {
    return { mode: "fixture" }
  }

  return {
    mode: "live",
    baseUrl: state.profile.baseUrl,
    bearerToken: state.session?.bearerToken,
  }
}

export function saveApiClientConnection(
  connection: ApiClientConnection,
  store: ConnectionProfileStore = createBrowserConnectionProfileStore(),
) {
  if (connection.mode === "fixture") {
    store.clear()
    return
  }

  store.saveProfile(createLiveConnectionProfile({ baseUrl: connection.baseUrl }))
  store.saveSession(connection.bearerToken ? { bearerToken: connection.bearerToken } : null)
}

function parseProfile(raw: string, runtime: NakoRuntimeMode): NakoConnectionProfile {
  const parsed = JSON.parse(raw) as Partial<NakoConnectionProfile>
  if (parsed.mode !== "live" || typeof parsed.baseUrl !== "string") {
    return createFixtureConnectionProfile(runtime)
  }

  return createLiveConnectionProfile({
    baseUrl: parsed.baseUrl,
    serverName: typeof parsed.serverName === "string" ? parsed.serverName : undefined,
    runtime: parsed.runtime === "tauri" ? "tauri" : runtime,
  })
}

function parseSession(raw: string): NakoSessionState | null {
  const parsed = JSON.parse(raw) as Partial<NakoSessionState>
  const session: NakoSessionState = {
    bearerToken: typeof parsed.bearerToken === "string" ? parsed.bearerToken : undefined,
    principalId: typeof parsed.principalId === "string" ? parsed.principalId : undefined,
    selectedUserId: typeof parsed.selectedUserId === "string" ? parsed.selectedUserId : undefined,
  }

  return isEmptySession(session) ? null : session
}

function isEmptySession(session: NakoSessionState) {
  return !session.bearerToken && !session.principalId && !session.selectedUserId
}

function browserLocalStorage() {
  return typeof window === "undefined" ? undefined : window.localStorage
}

function browserSessionStorage() {
  return typeof window === "undefined" ? undefined : window.sessionStorage
}
