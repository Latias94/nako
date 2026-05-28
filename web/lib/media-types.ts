import type { ClientMediaKind, MediaItemDto } from "@nako/sdk"

export interface MediaItem {
  id: string
  title: string
  originalTitle: string
  year: number
  rating: number
  poster: string
  backdrop: string
  overview: string
  type: "movie" | "series"
  duration?: string
  episodes?: number
  quality?: string
}

export function mapPublicMediaItem(item: MediaItemDto): MediaItem {
  const metadata = item.metadata

  return {
    id: item.id,
    title: metadata.title,
    originalTitle: metadata.original_title ?? metadata.title,
    year: releaseYear(metadata.release_date),
    rating: numericRating(metadata.ratings),
    poster: "/placeholder.jpg",
    backdrop: "/placeholder.jpg",
    overview: metadata.overview ?? "",
    type: mediaTypeForKind(item.kind),
    duration: formatRuntime(metadata.runtime_minutes),
    quality: undefined,
  }
}

function mediaTypeForKind(kind: ClientMediaKind): MediaItem["type"] {
  return kind === "movie" ? "movie" : "series"
}

function releaseYear(releaseDate: string | null) {
  if (!releaseDate) {
    return 0
  }

  const timestamp = Date.parse(releaseDate)
  if (Number.isNaN(timestamp)) {
    return 0
  }

  return new Date(timestamp).getUTCFullYear()
}

function numericRating(ratings: MediaItemDto["metadata"]["ratings"]) {
  for (const rating of ratings) {
    const value = Number.parseFloat(rating.value)
    if (!Number.isNaN(value)) {
      return Math.round(value * 10) / 10
    }
  }

  return 0
}

function formatRuntime(runtimeMinutes: number | null) {
  if (!runtimeMinutes || runtimeMinutes <= 0) {
    return undefined
  }

  const hours = Math.floor(runtimeMinutes / 60)
  const minutes = runtimeMinutes % 60

  if (hours <= 0) {
    return `${minutes}m`
  }

  if (minutes === 0) {
    return `${hours}h`
  }

  return `${hours}h ${minutes}m`
}
