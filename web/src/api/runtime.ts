import { createAdminApi, type AdminApiConnection } from "@/api/admin/client";
import { createMediaApi, type MediaApiConnection } from "@/api/media/client";

const sessionTokenKey = "nako.sessionToken";

export const mediaApi = createMediaApi(mediaConnectionFromEnvironment());
export const adminApi = createAdminApi(adminConnectionFromEnvironment());

export function mediaConnectionFromEnvironment(): MediaApiConnection {
  const baseUrl = import.meta.env.VITE_NAKO_BASE_URL?.trim();
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
  const baseUrl = import.meta.env.VITE_NAKO_BASE_URL?.trim();
  if (!baseUrl) {
    return { mode: "fixture" };
  }

  return {
    mode: "live",
    baseUrl,
    token: readSessionToken(),
  };
}

function readSessionToken(): string | undefined {
  if (typeof window === "undefined") {
    return undefined;
  }

  return window.sessionStorage.getItem(sessionTokenKey) ?? undefined;
}
