export type PublicManagementContext = {
  libraryId?: string
  itemId?: string
  mediaType?: "movie" | "series"
  sourceId?: string
  playbackSessionId?: string
}

const SAFE_IDENTIFIER = /^[A-Za-z0-9][A-Za-z0-9._:-]{0,199}$/

export function normalizePublicManagementContext(
  input: PublicManagementContext = {},
): PublicManagementContext {
  return {
    ...optionalContextField("libraryId", input.libraryId),
    ...optionalContextField("itemId", input.itemId),
    ...optionalMediaType(input.mediaType),
    ...optionalContextField("sourceId", input.sourceId),
    ...optionalContextField("playbackSessionId", input.playbackSessionId),
  }
}

function optionalContextField<K extends keyof PublicManagementContext>(
  key: K,
  value: string | undefined,
): Pick<PublicManagementContext, K> | Record<string, never> {
  const safe = safeIdentifier(value)

  return safe ? { [key]: safe } as Pick<PublicManagementContext, K> : {}
}

function optionalMediaType(
  value: PublicManagementContext["mediaType"] | undefined,
): Pick<PublicManagementContext, "mediaType"> | Record<string, never> {
  return value === "movie" || value === "series" ? { mediaType: value } : {}
}

function safeIdentifier(value: string | undefined) {
  if (typeof value !== "string") {
    return undefined
  }

  const trimmed = value.trim()
  return SAFE_IDENTIFIER.test(trimmed) ? trimmed : undefined
}
