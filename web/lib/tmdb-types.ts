// TMDb 数据类型

export interface TMDbMedia {
  id: number
  title?: string // 电影
  name?: string // 剧集
  original_title?: string
  original_name?: string
  overview: string
  poster_path: string | null
  backdrop_path: string | null
  release_date?: string // 电影
  first_air_date?: string // 剧集
  vote_average: number
  vote_count?: number
  genre_ids: number[]
  media_type?: "movie" | "tv"
  popularity?: number
}

export interface TMDbPerson {
  id: number
  name: string
  profile_path: string | null
  character?: string
  job?: string
  known_for_department?: string
}

export interface TMDbCredits {
  cast: TMDbPerson[]
  crew: TMDbPerson[]
}

export interface TMDbDetails extends TMDbMedia {
  runtime?: number // 电影
  episode_run_time?: number[] // 剧集
  number_of_seasons?: number
  number_of_episodes?: number
  status: string
  tagline?: string
  genres: Array<{ id: number; name: string }>
  production_companies: Array<{ id: number; name: string; logo_path: string | null }>
  credits?: TMDbCredits
  images?: {
    backdrops: Array<{ file_path: string; width: number; height: number }>
    posters: Array<{ file_path: string; width: number; height: number }>
  }
}

export interface TMDbResponse {
  results: TMDbMedia[]
  page?: number
  total_pages?: number
  total_results?: number
  fallback?: boolean
  image_base: string
}

// 转换为应用内部格式
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

// TMDb 图片尺寸
export const POSTER_SIZES = {
  small: "w185",
  medium: "w342",
  large: "w500",
  original: "original",
} as const

export const BACKDROP_SIZES = {
  small: "w300",
  medium: "w780",
  large: "w1280",
  original: "original",
} as const

// 辅助函数：构建图片 URL
export function getTMDbImageUrl(
  path: string | null,
  size: string = "w500",
  baseUrl: string = "https://image.tmdb.org/t/p"
): string {
  if (!path) {
    return "/placeholder-poster.jpg"
  }
  return `${baseUrl}/${size}${path}`
}

// 辅助函数：将 TMDb 数据转换为应用内部格式
export function transformTMDbMedia(item: TMDbMedia, imageBase: string): MediaItem {
  const isMovie = item.media_type === "movie" || !!item.title
  const title = item.title || item.name || "Unknown"
  const originalTitle = item.original_title || item.original_name || title
  const releaseDate = item.release_date || item.first_air_date
  const year = releaseDate ? new Date(releaseDate).getFullYear() : 0

  return {
    id: item.id.toString(),
    title,
    originalTitle,
    year,
    rating: Math.round(item.vote_average * 10) / 10,
    poster: getTMDbImageUrl(item.poster_path, "w500", imageBase),
    backdrop: getTMDbImageUrl(item.backdrop_path, "w1280", imageBase),
    overview: item.overview,
    type: isMovie ? "movie" : "series",
    quality: "1080p", // 默认值，实际应该从本地媒体信息获取
  }
}
