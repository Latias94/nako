import {
  createFixtureConnectionProfile,
  createLiveConnectionProfile,
  type NakoConnectionProfile,
  type NakoConnectionState,
} from "@/src/api/connection-profile"

export type TauriInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>

interface TauriServerProfile {
  baseUrl: string
  source: "environment" | "local_profile" | "session"
}

interface TauriDesktopBootstrap {
  runtime: string
  profile: TauriServerProfile | null
  nativePlayback: {
    available: boolean
    reason: string
  }
}

export async function loadTauriConnectionState(
  invoke: TauriInvoke = defaultTauriInvoke,
): Promise<NakoConnectionState> {
  return tauriBootstrapToConnectionState(await invoke<TauriDesktopBootstrap>("desktop_bootstrap"))
}

export async function saveTauriConnectionProfile(
  profile: NakoConnectionProfile,
  invoke: TauriInvoke = defaultTauriInvoke,
): Promise<NakoConnectionState> {
  if (profile.mode === "fixture") {
    return tauriBootstrapToConnectionState(await invoke<TauriDesktopBootstrap>("clear_server_profile"))
  }

  return tauriBootstrapToConnectionState(
    await invoke<TauriDesktopBootstrap>("save_server_profile", {
      input: {
        baseUrl: profile.baseUrl,
      },
    }),
  )
}

export async function clearTauriConnectionProfile(
  invoke: TauriInvoke = defaultTauriInvoke,
): Promise<NakoConnectionState> {
  return tauriBootstrapToConnectionState(await invoke<TauriDesktopBootstrap>("clear_server_profile"))
}

export function tauriBootstrapToConnectionState(bootstrap: TauriDesktopBootstrap): NakoConnectionState {
  if (!bootstrap.profile) {
    return {
      profile: createFixtureConnectionProfile("tauri"),
      session: null,
    }
  }

  return {
    profile: createLiveConnectionProfile({
      baseUrl: bootstrap.profile.baseUrl,
      runtime: "tauri",
    }),
    session: null,
  }
}

function defaultTauriInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (typeof window === "undefined") {
    return Promise.reject(new Error("Tauri invoke is unavailable outside a browser runtime"))
  }

  const tauri = (window as typeof window & {
    __TAURI__?: {
      core?: {
        invoke?: TauriInvoke
      }
    }
  }).__TAURI__

  if (!tauri?.core?.invoke) {
    return Promise.reject(new Error("Tauri invoke is unavailable"))
  }

  return tauri.core.invoke<T>(command, args)
}
