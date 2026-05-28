export type PublicClientConnection =
  | {
      mode: "fixture"
    }
  | {
      mode: "live"
      baseUrl: string
      bearerToken?: string
    }

const PUBLIC_CLIENT_CONNECTION_KEY = "nako.publicClient.connection"

export function loadPublicClientConnection(): PublicClientConnection {
  if (typeof window === "undefined") {
    return { mode: "fixture" }
  }

  const raw = window.localStorage.getItem(PUBLIC_CLIENT_CONNECTION_KEY)
  if (!raw) {
    return { mode: "fixture" }
  }

  try {
    const parsed = JSON.parse(raw) as Partial<Extract<PublicClientConnection, { mode: "live" }>>
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

export function savePublicClientConnection(connection: PublicClientConnection) {
  if (typeof window === "undefined") {
    return
  }

  if (connection.mode === "fixture") {
    window.localStorage.removeItem(PUBLIC_CLIENT_CONNECTION_KEY)
    return
  }

  window.localStorage.setItem(PUBLIC_CLIENT_CONNECTION_KEY, JSON.stringify(connection))
}
