import { invoke, isTauri } from "@tauri-apps/api/core";

export interface DesktopBootstrap {
  runtime: "tauri_desktop";
  profile: DesktopServerProfile | null;
  nativePlayback: {
    available: boolean;
    reason: "native_playback_core_not_integrated";
  };
}

export interface DesktopServerProfile {
  baseUrl: string;
  source: "environment" | "session";
}

export async function loadDesktopBootstrap(): Promise<DesktopBootstrap | null> {
  if (!canInvokeTauri()) {
    return null;
  }

  return invoke<DesktopBootstrap>("desktop_bootstrap");
}

export async function saveDesktopServerProfile(baseUrl: string): Promise<DesktopBootstrap | null> {
  if (!canInvokeTauri()) {
    return null;
  }

  return invoke<DesktopBootstrap>("save_server_profile", {
    input: {
      baseUrl,
    },
  });
}

export async function clearDesktopServerProfile(): Promise<DesktopBootstrap | null> {
  if (!canInvokeTauri()) {
    return null;
  }

  return invoke<DesktopBootstrap>("clear_server_profile");
}

function canInvokeTauri(): boolean {
  if (isTauri()) {
    return true;
  }

  return (
    typeof window !== "undefined" &&
    typeof (window as WindowWithTauriInternals).__TAURI_INTERNALS__?.invoke === "function"
  );
}

interface WindowWithTauriInternals extends Window {
  __TAURI_INTERNALS__?: {
    invoke?: unknown;
  };
}
