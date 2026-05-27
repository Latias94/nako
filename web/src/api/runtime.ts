import { NakoClient, type FetchLike as PublicClientFetch } from "@nako/sdk";

import { createAdminApi, type AdminApi, type AdminApiConnection } from "@/api/admin/client";
import {
  clearDesktopServerProfile,
  loadDesktopBootstrap,
  saveDesktopServerProfile,
  type DesktopBootstrap,
} from "@/api/desktop";
import { createMediaApi, type MediaApi, type MediaApiConnection } from "@/api/media/client";
import { normalizeBaseUrl } from "@/api/shared";

const sessionTokenKey = "nako.sessionToken";
const serverBaseUrlKey = "nako.serverBaseUrl";

export interface ServerConnectionResult {
  apiVersion: string;
  baseUrl: string;
  desktop: DesktopBootstrap | null;
  status: string;
}

export const mediaApi: MediaApi = {
  get source() {
    return currentMediaApi().source;
  },
  listLibraries: (page) => currentMediaApi().listLibraries(page),
  getLibrary: (libraryId) => currentMediaApi().getLibrary(libraryId),
  listLibrarySources: (libraryId, page) => currentMediaApi().listLibrarySources(libraryId, page),
  listContinueWatching: (page) => currentMediaApi().listContinueWatching(page),
  searchItems: (query) => currentMediaApi().searchItems(query),
  getItem: (itemId) => currentMediaApi().getItem(itemId),
  managementContextLinks: (query) => currentMediaApi().managementContextLinks(query),
  createBrowserPlaybackTicket: (sourceId, body) =>
    currentMediaApi().createBrowserPlaybackTicket(sourceId, body),
};

export const adminApi: AdminApi = {
  get source() {
    return currentAdminApi().source;
  },
  getOverview: () => currentAdminApi().getOverview(),
  getAccessSummary: () => currentAdminApi().getAccessSummary(),
  getAddons: (query) => currentAdminApi().getAddons(query),
  getJobs: (query) => currentAdminApi().getJobs(query),
};

export function mediaConnectionFromEnvironment(): MediaApiConnection {
  const baseUrl = readConfiguredServerBaseUrl();
  if (!baseUrl) {
    return { mode: "fixture" };
  }

  return {
    mode: "live",
    baseUrl,
    bearerToken: readSessionToken(),
  };
}

export function adminConnectionFromEnvironment(): AdminApiConnection {
  const baseUrl = readConfiguredServerBaseUrl();
  if (!baseUrl) {
    return { mode: "fixture" };
  }

  return {
    mode: "live",
    baseUrl,
    token: readSessionToken(),
  };
}

export function readConfiguredServerBaseUrl(): string | undefined {
  if (typeof window !== "undefined") {
    const stored = window.localStorage.getItem(serverBaseUrlKey)?.trim();
    if (stored) {
      try {
        return normalizeServerBaseUrl(stored);
      } catch {
        window.localStorage.removeItem(serverBaseUrlKey);
      }
    }
  }

  const baseUrl = import.meta.env.VITE_NAKO_BASE_URL?.trim();
  if (!baseUrl) {
    return undefined;
  }

  try {
    return normalizeServerBaseUrl(baseUrl);
  } catch {
    return undefined;
  }
}

export async function bootstrapDesktopConnection(): Promise<DesktopBootstrap | null> {
  const bootstrap = await loadDesktopBootstrap();
  if (bootstrap?.profile) {
    writeConfiguredServerBaseUrl(bootstrap.profile.baseUrl);
  }

  return bootstrap;
}

export async function configureServerConnection(
  rawBaseUrl: string,
  fetcher?: PublicClientFetch,
): Promise<ServerConnectionResult> {
  const baseUrl = normalizeServerBaseUrl(rawBaseUrl);
  const health = await new NakoClient({ baseUrl, fetch: fetcher }).health();
  const desktop = await saveDesktopServerProfile(baseUrl);
  const storedBaseUrl = desktop?.profile?.baseUrl ?? baseUrl;

  writeConfiguredServerBaseUrl(storedBaseUrl);

  return {
    apiVersion: health.version,
    baseUrl: storedBaseUrl,
    desktop,
    status: health.status,
  };
}

export async function clearServerConnection(): Promise<void> {
  await clearDesktopServerProfile();

  if (typeof window !== "undefined") {
    window.localStorage.removeItem(serverBaseUrlKey);
  }
}

export function normalizeServerBaseUrl(rawBaseUrl: string): string {
  const trimmed = rawBaseUrl.trim();
  if (!trimmed) {
    throw new Error("Server URL is required");
  }

  let parsed: URL;
  try {
    parsed = new URL(trimmed);
  } catch {
    throw new Error("Server URL must be a valid absolute URL");
  }

  if (parsed.protocol !== "http:" && parsed.protocol !== "https:") {
    throw new Error("Server URL must use http or https");
  }

  if (parsed.username || parsed.password) {
    throw new Error("Server URL must not include credentials");
  }

  if (parsed.search) {
    throw new Error("Server URL must not include a query string");
  }

  if (parsed.hash) {
    throw new Error("Server URL must not include a fragment");
  }

  return normalizeBaseUrl(parsed.toString());
}

function readSessionToken(): string | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return window.sessionStorage.getItem(sessionTokenKey) ?? undefined;
}

function writeConfiguredServerBaseUrl(baseUrl: string): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.setItem(serverBaseUrlKey, baseUrl);
}

function currentMediaApi(): MediaApi {
  return createMediaApi(mediaConnectionFromEnvironment());
}

function currentAdminApi(): AdminApi {
  return createAdminApi(adminConnectionFromEnvironment());
}
