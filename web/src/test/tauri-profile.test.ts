import { describe, expect, it } from "vitest"
import { createLiveConnectionProfile } from "@/src/api/connection-profile"
import {
  clearTauriConnectionProfile,
  loadTauriConnectionState,
  saveTauriConnectionProfile,
  tauriBootstrapToConnectionState,
  type TauriInvoke,
} from "@/src/api/tauri-profile"

function bootstrap(baseUrl: string | null) {
  return {
    runtime: "tauri_desktop",
    profile: baseUrl
      ? {
          baseUrl,
          source: "local_profile" as const,
        }
      : null,
    nativePlayback: {
      available: false,
      reason: "native_playback_core_not_integrated",
    },
  }
}

describe("Tauri profile adapter", () => {
  it("maps desktop bootstrap profile into tauri connection state", () => {
    expect(tauriBootstrapToConnectionState(bootstrap("http://127.0.0.1:8096"))).toMatchObject({
      profile: {
        mode: "live",
        runtime: "tauri",
        baseUrl: "http://127.0.0.1:8096",
      },
      session: null,
    })

    expect(tauriBootstrapToConnectionState(bootstrap(null))).toMatchObject({
      profile: {
        mode: "fixture",
        runtime: "tauri",
      },
      session: null,
    })
  })

  it("uses Tauri invoke commands without passing bearer tokens or session state", async () => {
    const calls: Array<[string, Record<string, unknown> | undefined]> = []
    const invoke: TauriInvoke = async <T>(command: string, args?: Record<string, unknown>) => {
      calls.push([command, args])

      if (command === "desktop_bootstrap") {
        return bootstrap("http://nako.test") as T
      }
      if (command === "save_server_profile") {
        return bootstrap((args?.input as { baseUrl: string }).baseUrl) as T
      }
      if (command === "clear_server_profile") {
        return bootstrap(null) as T
      }
      throw new Error(`unexpected command ${command}`)
    }

    await expect(loadTauriConnectionState(invoke)).resolves.toMatchObject({
      profile: {
        mode: "live",
        baseUrl: "http://nako.test",
      },
    })
    await saveTauriConnectionProfile(
      createLiveConnectionProfile({
        baseUrl: "http://nako.test",
        runtime: "tauri",
      }),
      invoke,
    )
    await clearTauriConnectionProfile(invoke)

    expect(calls.map(([command]) => command)).toEqual([
      "desktop_bootstrap",
      "save_server_profile",
      "clear_server_profile",
    ])
    expect(JSON.stringify(calls)).not.toContain("Bearer")
    expect(JSON.stringify(calls)).not.toContain("token")
  })
})
