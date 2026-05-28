export type AdminApiConnection =
  | {
      mode: "fixture"
    }
  | {
      mode: "live"
      baseUrl: string
      bearerToken?: string
    }

const ADMIN_API_CONNECTION_KEY = "nako.adminApi.connection"

export function loadAdminApiConnection(): AdminApiConnection {
  if (typeof window === "undefined") {
    return { mode: "fixture" }
  }

  const raw = window.localStorage.getItem(ADMIN_API_CONNECTION_KEY)
  if (!raw) {
    return { mode: "fixture" }
  }

  try {
    const parsed = JSON.parse(raw) as Partial<Extract<AdminApiConnection, { mode: "live" }>>
    if (parsed.mode !== "live" || !parsed.baseUrl || typeof parsed.baseUrl !== "string") {
      return { mode: "fixture" }
    }

    return {
      mode: "live",
      baseUrl: parsed.baseUrl,
      bearerToken: typeof parsed.bearerToken === "string" ? parsed.bearerToken : undefined,
    }
  } catch {
    return { mode: "fixture" }
  }
}

export function saveAdminApiConnection(connection: AdminApiConnection) {
  if (typeof window === "undefined") {
    return
  }

  if (connection.mode === "fixture") {
    window.localStorage.removeItem(ADMIN_API_CONNECTION_KEY)
    return
  }

  window.localStorage.setItem(ADMIN_API_CONNECTION_KEY, JSON.stringify(connection))
}
