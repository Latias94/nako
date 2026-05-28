import { useQuery } from "@tanstack/react-query"
import type { TMDbResponse, TMDbDetails, MediaItem } from "./tmdb-types"
import { transformTMDbMedia } from "./tmdb-types"

// 获取热门/推荐媒体
export function useTrendingMedia() {
  return useQuery({
    queryKey: ["tmdb", "trending"],
    queryFn: async (): Promise<{ items: MediaItem[]; fallback: boolean }> => {
      try {
        const response = await fetch("/api/tmdb?endpoint=trending")
        if (!response.ok) throw new Error(`API error: ${response.status}`)
        const data: TMDbResponse = await response.json()
        
        return {
          items: data.results.map((item) => transformTMDbMedia(item, data.image_base)),
          fallback: data.fallback || false,
        }
      } catch (error) {
        console.error("useTrendingMedia error:", error)
        throw error
      }
    },
    staleTime: 5 * 60 * 1000,
    retry: 2,
  })
}

// 获取分类推荐数据（简化版 - 一次获取）
export function useCategoryMedia() {
  const { data, isLoading, error } = useTrendingMedia()

  // 从 trending 数据中构建分类
  const items = data?.items || []
  const movies = items.filter((item) => item.type === "movie")
  const series = items.filter((item) => item.type === "series")

  const categories = [
    {
      title: "为你推荐",
      items: items.slice(0, 8),
    },
    {
      title: "热门电影",
      items: movies.slice(0, 8),
    },
    {
      title: "热门剧集",
      items: series.slice(0, 8),
    },
  ].filter((c) => c.items.length > 0)

  return {
    categories,
    isLoading,
    error,
    fallback: data?.fallback || false,
  }
}

// 搜索媒体
export function useSearchMedia(query: string) {
  return useQuery({
    queryKey: ["tmdb", "search", query],
    queryFn: async (): Promise<{ items: MediaItem[]; fallback: boolean }> => {
      if (!query.trim()) {
        return { items: [], fallback: false }
      }
      
      const response = await fetch(`/api/tmdb?endpoint=search&query=${encodeURIComponent(query)}`)
      const data: TMDbResponse = await response.json()
      
      return {
        items: data.results.map((item) => transformTMDbMedia(item, data.image_base)),
        fallback: data.fallback || false,
      }
    },
    enabled: query.length > 0,
  })
}

// 获取媒体详情
export function useMediaDetails(id: string, mediaType: "movie" | "series") {
  const tmdbType = mediaType === "series" ? "tv" : "movie"
  
  return useQuery({
    queryKey: ["tmdb", "details", tmdbType, id],
    queryFn: async (): Promise<TMDbDetails | null> => {
      const response = await fetch(`/api/tmdb?endpoint=details&id=${id}&media_type=${tmdbType}`)
      if (!response.ok) return null
      return response.json()
    },
    enabled: !!id,
  })
}
