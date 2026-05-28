import { useQuery } from "@tanstack/react-query"
import type { MediaItem } from "./tmdb-types"

const LOCAL_MEDIA_ITEMS: MediaItem[] = [
  {
    id: "1",
    title: "沙丘2",
    originalTitle: "Dune: Part Two",
    year: 2024,
    rating: 8.6,
    poster: "/posters/dune2.jpg",
    backdrop: "/backdrops/dune2-backdrop.jpg",
    overview: "保罗·厄崔迪与弗雷曼人联手，向摧毁家族的势力复仇。",
    type: "movie",
    duration: "2h 46m",
    quality: "4K HDR",
  },
  {
    id: "2",
    title: "奥本海默",
    originalTitle: "Oppenheimer",
    year: 2023,
    rating: 8.4,
    poster: "/posters/oppenheimer.jpg",
    backdrop: "/placeholder.jpg",
    overview: "一位物理学家在战争年代推动了足以改变世界的计划。",
    type: "movie",
    duration: "3h 00m",
    quality: "4K",
  },
  {
    id: "3",
    title: "星际穿越",
    originalTitle: "Interstellar",
    year: 2014,
    rating: 8.7,
    poster: "/posters/interstellar.jpg",
    backdrop: "/placeholder.jpg",
    overview: "一支探索队穿越虫洞，寻找人类新的家园。",
    type: "movie",
    duration: "2h 49m",
    quality: "4K",
  },
  {
    id: "4",
    title: "银翼杀手 2049",
    originalTitle: "Blade Runner 2049",
    year: 2017,
    rating: 8.0,
    poster: "/posters/blade-runner.jpg",
    backdrop: "/placeholder.jpg",
    overview: "一名新型复制人警探追查一桩足以颠覆秩序的秘密。",
    type: "movie",
    duration: "2h 44m",
    quality: "4K",
  },
  {
    id: "5",
    title: "真探",
    originalTitle: "True Detective",
    year: 2014,
    rating: 8.9,
    poster: "/posters/true-detective.jpg",
    backdrop: "/backdrops/true-detective-backdrop.jpg",
    overview: "一对侦探追查横跨多年、跨越信仰与暴力的连环案件。",
    type: "series",
    episodes: 8,
    quality: "1080p",
  },
  {
    id: "6",
    title: "绝命毒师",
    originalTitle: "Breaking Bad",
    year: 2008,
    rating: 9.5,
    poster: "/posters/breaking-bad.jpg",
    backdrop: "/placeholder.jpg",
    overview: "一位化学老师在绝境中走向犯罪世界。",
    type: "series",
    episodes: 62,
    quality: "1080p",
  },
  {
    id: "7",
    title: "继承之战",
    originalTitle: "Succession",
    year: 2018,
    rating: 8.9,
    poster: "/posters/succession.jpg",
    backdrop: "/placeholder.jpg",
    overview: "一个媒体帝国的权力斗争与家族分裂。",
    type: "series",
    episodes: 39,
    quality: "1080p",
  },
  {
    id: "8",
    title: "降临",
    originalTitle: "Arrival",
    year: 2016,
    rating: 7.9,
    poster: "/posters/arrival.jpg",
    backdrop: "/placeholder.jpg",
    overview: "语言学家尝试与突然降临的外星生命建立交流。",
    type: "movie",
    duration: "1h 56m",
    quality: "4K",
  },
]

function matchesQuery(item: MediaItem, query: string) {
  const normalized = query.trim().toLowerCase()
  if (!normalized) {
    return true
  }

  return [item.title, item.originalTitle]
    .filter(Boolean)
    .some((value) => value.toLowerCase().includes(normalized))
}

export function useTrendingMedia() {
  return useQuery({
    queryKey: ["nako", "media", "trending"],
    queryFn: async (): Promise<{ items: MediaItem[]; fallback: boolean }> => ({
      items: LOCAL_MEDIA_ITEMS,
      fallback: true,
    }),
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useCategoryMedia() {
  const { data, isLoading, error } = useTrendingMedia()

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
  ].filter((category) => category.items.length > 0)

  return {
    categories,
    isLoading,
    error,
    fallback: data?.fallback || false,
  }
}

export function useSearchMedia(query: string) {
  return useQuery({
    queryKey: ["nako", "media", "search", query],
    queryFn: async (): Promise<{ items: MediaItem[]; fallback: boolean }> => {
      if (!query.trim()) {
        return { items: [], fallback: true }
      }

      return {
        items: LOCAL_MEDIA_ITEMS.filter((item) => matchesQuery(item, query)),
        fallback: true,
      }
    },
    enabled: query.trim().length > 0,
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}

export function useMediaDetails(id: string, mediaType: "movie" | "series") {
  return useQuery({
    queryKey: ["nako", "media", "details", mediaType, id],
    queryFn: async (): Promise<MediaItem | null> => {
      const item = LOCAL_MEDIA_ITEMS.find((entry) => entry.id === id && entry.type === mediaType)
      return item ?? null
    },
    enabled: !!id,
    staleTime: 5 * 60 * 1000,
    retry: 0,
  })
}
