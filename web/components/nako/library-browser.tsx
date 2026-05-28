"use client"

import { useState, useEffect, useRef, useCallback, useMemo } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"
import { 
  Play, Shuffle, ChevronDown, Check, Search, X, MoreHorizontal,
  ArrowUp, ArrowDown, Pencil, Trash2, Plus, ChevronLeft, 
  Film, Tv, Sparkles, FolderOpen, LayoutGrid, List, Table2,
  FolderSync, Settings, CheckSquare, Square, Tag, SlidersHorizontal
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Skeleton } from "@/components/ui/skeleton"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

// ============ Types ============
interface Library {
  id: string
  name: string
  type: "movie" | "tv" | "music" | "photo" | "mixed"
  icon: typeof Film
  itemCount: number
  path: string
  lastScanned?: string
}

interface MediaItem {
  id: string
  title: string
  originalTitle?: string
  year: number
  poster: string
  rating?: number
  playCount: number
  progress?: number
  duration?: string
  addedAt: string
  genres: string[]
  resolution?: string
  bitrate?: string
  director?: string
  actors?: string[]
  overview?: string
  studio?: string
  country?: string
  contentRating?: string
  type: "movie" | "episode" | "album" | "photo"
}

interface Collection {
  id: string
  name: string
  itemCount: number
  poster: string
}

// ============ Mock Data ============
const mockLibraries: Library[] = [
  { id: "movies", name: "电影", type: "movie", icon: Film, itemCount: 847, path: "/media/movies", lastScanned: "2024-01-15 14:30" },
  { id: "tvshows", name: "剧集", type: "tv", icon: Tv, itemCount: 156, path: "/media/tv", lastScanned: "2024-01-15 14:30" },
  { id: "anime", name: "动画", type: "tv", icon: Sparkles, itemCount: 234, path: "/media/anime", lastScanned: "2024-01-15 12:00" },
  { id: "documentary", name: "纪录片", type: "movie", icon: FolderOpen, itemCount: 89, path: "/media/documentary", lastScanned: "2024-01-14 20:00" },
]

// Generate more mock items for virtualization testing
const generateMockItems = (count: number): MediaItem[] => {
  const titles = ["沙丘2", "奥本海默", "星际穿越", "银翼杀手2049", "降临", "信条", "盗梦空间", "蝙蝠侠", "教父", "肖申克的救赎"]
  const genres = ["科幻", "动作", "剧情", "悬疑", "惊悚", "冒险", "传记", "历史"]
  const resolutions = ["4K", "1080p", "720p", "4K HDR", "4K Dolby Vision"]
  const directors = ["丹尼斯·维伦纽瓦", "克里斯托弗·诺兰", "大卫·芬奇", "马丁·斯科塞斯"]
  const studios = ["华纳兄弟", "传奇影业", "环球影业", "派拉蒙", "迪士尼"]
  const countries = ["美国", "英国", "法国", "日本", "韩国"]
  const overviews = [
    "在遥远的未来，年轻的保罗·厄崔迪必须前往宇宙中最危险的星球，以确保他家族和人民的未来。",
    "讲述美国原子弹之父罗伯特·奥本海默主导制造出原子弹的故事，以及他在二战后面临的道德困境。",
    "一组探险家利用虫洞穿越时空，前往遥远的星系寻找人类的新家园。",
    "一名年轻的银翼杀手发现了一个被掩盖多年的秘密，这将引领他寻找失踪已久的前银翼杀手里克·德卡德。",
    "当神秘的外星飞船降落在地球各处时，一位语言学家被招募来解读外星人的信息。",
    "一名秘密特工必须通过时间逆转来阻止第三次世界大战的爆发。",
    "一个技术高超的盗贼专门在人们做梦时潜入他们的潜意识窃取秘密。",
  ]
  
  return Array.from({ length: count }, (_, i) => ({
  id: `media-${i}`,
  title: titles[i % titles.length] + (i >= titles.length ? ` ${Math.floor(i / titles.length) + 1}` : ""),
  originalTitle: "Original Title",
  year: 2015 + (i % 10),
  poster: `https://image.tmdb.org/t/p/w500/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg`,
  rating: 6.5 + Math.random() * 3.5,
  playCount: Math.floor(Math.random() * 50),
  progress: Math.random() > 0.7 ? Math.floor(Math.random() * 100) : undefined,
  duration: `${Math.floor(90 + Math.random() * 90)}分钟`,
  addedAt: new Date(Date.now() - Math.random() * 365 * 24 * 60 * 60 * 1000).toISOString(),
  genres: [genres[i % genres.length], genres[(i + 3) % genres.length]],
  resolution: resolutions[i % resolutions.length],
  bitrate: `${Math.floor(15 + Math.random() * 25)} Mbps`,
  director: directors[i % directors.length],
  actors: ["演员A", "演员B", "演员C"],
  studio: studios[i % studios.length],
  country: countries[i % countries.length],
  contentRating: ["G", "PG", "PG-13", "R"][i % 4],
  type: "movie",
  overview: overviews[i % overviews.length],
  }))
  }

const mockCollections: Collection[] = [
  { id: "c1", name: "漫威电影宇宙", itemCount: 32, poster: "https://image.tmdb.org/t/p/w500/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg" },
  { id: "c2", name: "星球大战", itemCount: 11, poster: "https://image.tmdb.org/t/p/w500/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg" },
  { id: "c3", name: "诺兰作品集", itemCount: 12, poster: "https://image.tmdb.org/t/p/w500/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg" },
  { id: "c4", name: "哈利波特", itemCount: 8, poster: "https://image.tmdb.org/t/p/w500/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg" },
]

// ============ Filter & Sort Options ============
const quickFilters = [
  { id: "all", label: "全部" },
  { id: "hdr", label: "HDR" },
  { id: "dolby", label: "杜比" },
  { id: "atmos", label: "Atmos" },
  { id: "unwatched", label: "未观看" },
  { id: "watching", label: "观看中" },
  { id: "unmatched", label: "未匹配" },
  { id: "duplicates", label: "副本" },
]

const categoryFilters = [
  { id: "genre", label: "流派", options: ["科幻", "动作", "剧情", "悬疑", "惊悚", "冒险", "喜剧", "恐怖", "爱情"] },
  { id: "year", label: "年份", options: ["2024", "2023", "2022", "2021", "2020", "2019", "2018", "2017", "2016", "2015"] },
  { id: "decade", label: "年代", options: ["2020年代", "2010年代", "2000年代", "1990年代", "1980年代"] },
  { id: "contentRating", label: "内容分级", options: ["G", "PG", "PG-13", "R", "NC-17"] },
  { id: "director", label: "导演", options: ["克里斯托弗·诺兰", "丹尼斯·维伦纽瓦", "大卫·芬奇", "马丁·斯科塞斯"] },
  { id: "actor", label: "演员", options: ["提摩西·查拉梅", "赞达亚", "基里安·墨菲"] },
  { id: "studio", label: "制片公司", options: ["华纳兄弟", "传奇影业", "环球影业", "派拉蒙", "迪士尼"] },
  { id: "country", label: "国家", options: ["美国", "英国", "法国", "日本", "韩国", "中国"] },
  { id: "resolution", label: "分辨率", options: ["4K", "1080p", "720p", "SD"] },
]

const sortOptions = [
  { id: "title", label: "标题" },
  { id: "year", label: "年份" },
  { id: "releaseDate", label: "发布日期" },
  { id: "criticRating", label: "评论家评分" },
  { id: "audienceRating", label: "观众评分" },
  { id: "rating", label: "评分" },
  { id: "contentRating", label: "内容分级" },
  { id: "duration", label: "时长" },
  { id: "progress", label: "播放进度" },
  { id: "playCount", label: "播放" },
  { id: "addedAt", label: "日期已添加" },
  { id: "lastViewed", label: "查看日期" },
  { id: "resolution", label: "分辨率" },
  { id: "bitrate", label: "比特率" },
  { id: "random", label: "随机" },
]

type ViewMode = "grid" | "detail" | "table"

interface LibraryBrowserProps {
  onBack?: () => void
  onSelectMedia?: (mediaId: string) => void
  onEditMedia?: (mediaId: string) => void
  onSearch?: (query: string, libraryId?: string) => void
  isAdmin?: boolean
}

export function LibraryBrowser({ onBack, onSelectMedia, onEditMedia, onSearch, isAdmin = false }: LibraryBrowserProps) {
  // State
  const [currentLibrary, setCurrentLibrary] = useState(mockLibraries[0])
  const [activeTab, setActiveTab] = useState<"recommend" | "library" | "collections" | "categories">("library")
  const [viewMode, setViewMode] = useState<ViewMode>("grid")
  const [quickFilter, setQuickFilter] = useState("all")
  const [categoryFilter, setCategoryFilter] = useState<{ type: string; value: string } | null>(null)
  const [sortBy, setSortBy] = useState("addedAt")
  const [sortOrder, setSortOrder] = useState<"asc" | "desc">("desc")
  const [isScanning, setIsScanning] = useState(false)
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set())
  const [isSelectionMode, setIsSelectionMode] = useState(false)
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false)
  
  // Virtual list state
  const [allItems] = useState(() => generateMockItems(847))
  const [visibleItems, setVisibleItems] = useState<MediaItem[]>([])
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [isInitialLoading, setIsInitialLoading] = useState(true)
  const [page, setPage] = useState(1)
  const itemsPerPage = 50
  const containerRef = useRef<HTMLDivElement>(null)
  const observerRef = useRef<IntersectionObserver | null>(null)
  const loadMoreRef = useRef<HTMLDivElement>(null)
  const detailListRef = useRef<HTMLDivElement>(null)
  const tableContainerRef = useRef<HTMLDivElement>(null)

  // Virtualizer for detail view (only active when detail view mode)
  const detailVirtualizer = useVirtualizer({
    count: viewMode === "detail" ? visibleItems.length : 0,
    getScrollElement: () => detailListRef.current,
    estimateSize: () => 88, // Detail item height
    overscan: 10,
  })

  // Virtualizer for table view
  const tableVirtualizer = useVirtualizer({
    count: viewMode === "table" ? visibleItems.length : 0,
    getScrollElement: () => tableContainerRef.current,
    estimateSize: () => 52, // Table row height
    overscan: 15,
  })

  // Filter and sort items
  const filteredItems = useMemo(() => {
    let items = [...allItems]
    
    // Apply quick filter
    if (quickFilter !== "all") {
      switch (quickFilter) {
        case "hdr":
          items = items.filter(item => item.resolution?.includes("HDR"))
          break
        case "dolby":
          items = items.filter(item => item.resolution?.includes("Dolby"))
          break
        case "unwatched":
          items = items.filter(item => item.playCount === 0)
          break
        case "watching":
          items = items.filter(item => item.progress && item.progress > 0 && item.progress < 100)
          break
      }
    }
    
    // Apply category filter
    if (categoryFilter) {
      items = items.filter(item => {
        switch (categoryFilter.type) {
          case "genre": return item.genres.includes(categoryFilter.value)
          case "year": return item.year.toString() === categoryFilter.value
          case "director": return item.director === categoryFilter.value
          case "studio": return item.studio === categoryFilter.value
          case "country": return item.country === categoryFilter.value
          case "resolution": return item.resolution?.startsWith(categoryFilter.value)
          default: return true
        }
      })
    }
    
    // Apply sort
    items.sort((a, b) => {
      let comparison = 0
      switch (sortBy) {
        case "title": comparison = a.title.localeCompare(b.title); break
        case "year": comparison = a.year - b.year; break
        case "rating": comparison = (a.rating || 0) - (b.rating || 0); break
        case "playCount": comparison = a.playCount - b.playCount; break
        case "addedAt": comparison = new Date(a.addedAt).getTime() - new Date(b.addedAt).getTime(); break
        case "random": comparison = Math.random() - 0.5; break
        default: comparison = 0
      }
      return sortOrder === "asc" ? comparison : -comparison
    })
    
    return items
  }, [allItems, quickFilter, categoryFilter, sortBy, sortOrder])

  // Load items with pagination
  // Load items progressively with initial loading state
  useEffect(() => {
    if (page === 1) {
      setIsInitialLoading(true)
      const timer = setTimeout(() => {
        setVisibleItems(filteredItems.slice(0, itemsPerPage))
        setIsInitialLoading(false)
      }, 400)
      return () => clearTimeout(timer)
    } else {
      setVisibleItems(filteredItems.slice(0, page * itemsPerPage))
    }
  }, [filteredItems, page])

  // Reset page when filters change
  useEffect(() => {
    setPage(1)
    setIsInitialLoading(true)
  }, [quickFilter, categoryFilter, sortBy, sortOrder])

  // Intersection observer for infinite scroll
  useEffect(() => {
    observerRef.current = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && !isLoadingMore && visibleItems.length < filteredItems.length) {
          setIsLoadingMore(true)
          setTimeout(() => {
            setPage(p => p + 1)
            setIsLoadingMore(false)
          }, 300)
        }
      },
      { threshold: 0.1 }
    )

    if (loadMoreRef.current) {
      observerRef.current.observe(loadMoreRef.current)
    }

    return () => observerRef.current?.disconnect()
  }, [isLoadingMore, visibleItems.length, filteredItems.length])

  // Handle scan
  const handleScan = () => {
    setIsScanning(true)
    setTimeout(() => setIsScanning(false), 3000)
  }

  // Handle sort click - toggle order if same field
  const handleSortClick = (field: string) => {
    if (sortBy === field) {
      setSortOrder(prev => prev === "asc" ? "desc" : "asc")
    } else {
      setSortBy(field)
      setSortOrder("desc")
    }
  }

  // Selection handlers
  const toggleItemSelection = (id: string) => {
    const newSelection = new Set(selectedItems)
    if (newSelection.has(id)) {
      newSelection.delete(id)
    } else {
      newSelection.add(id)
    }
    setSelectedItems(newSelection)
  }

  const selectAll = () => {
    setSelectedItems(new Set(visibleItems.map(item => item.id)))
  }

  const clearSelection = () => {
    setSelectedItems(new Set())
    setIsSelectionMode(false)
  }

  // Grid columns - fixed responsive layout
  const getGridColumns = () => {
    return "grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 2xl:grid-cols-7"
  }

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Header */}
      <header className="sticky top-0 z-40 border-b border-border bg-background/95 backdrop-blur-sm">
        <div className="flex h-14 items-center gap-3 px-4">
          {/* Back button */}
          {onBack && (
            <Button variant="ghost" size="icon" onClick={onBack} className="shrink-0">
              <ChevronLeft className="h-5 w-5" />
            </Button>
          )}
          
          {/* Library selector */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" className="gap-2 text-lg font-semibold">
                <currentLibrary.icon className="h-5 w-5" />
                {currentLibrary.name}
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="w-56">
              {mockLibraries.map((lib) => (
                <DropdownMenuItem key={lib.id} onClick={() => setCurrentLibrary(lib)}>
                  <lib.icon className="mr-2 h-4 w-4" />
                  <span className="flex-1">{lib.name}</span>
                  <span className="text-xs text-muted-foreground">{lib.itemCount}</span>
                  {lib.id === currentLibrary.id && <Check className="ml-2 h-4 w-4 text-primary" />}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          {/* Tabs */}
          <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)} className="ml-auto">
            <TabsList className="h-9">
              <TabsTrigger value="recommend" className="text-xs">推荐</TabsTrigger>
              <TabsTrigger value="library" className="text-xs">资料库</TabsTrigger>
              <TabsTrigger value="collections" className="text-xs">合集</TabsTrigger>
              <TabsTrigger value="categories" className="text-xs">分类</TabsTrigger>
            </TabsList>
          </Tabs>

          {/* Admin actions */}
          {isAdmin && (
            <div className="flex items-center gap-1">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button 
                      variant="ghost" 
                      size="icon" 
                      className="h-9 w-9"
                      onClick={() => onSearch?.("", currentLibrary.id)}
                    >
                      <Search className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>在此媒体库中搜索</TooltipContent>
                </Tooltip>
              </TooltipProvider>
              
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-9 w-9" onClick={handleScan} disabled={isScanning}>
                      <FolderSync className={cn("h-4 w-4", isScanning && "animate-spin")} />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>扫描媒体库</TooltipContent>
                </Tooltip>
              </TooltipProvider>
              
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="ghost" size="icon" className="h-9 w-9">
                      <Settings className="h-4 w-4" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>媒体库设置</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
          )}
        </div>

        {/* Filter & Sort Bar */}
        {activeTab === "library" && (
          <div className="flex items-center gap-2 border-t border-border/50 px-4 py-2">
            {/* Quick Filter Dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 gap-1.5 text-sm">
                  {quickFilters.find(f => f.id === quickFilter)?.label || "全部"}
                  <ChevronDown className="h-3.5 w-3.5" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-48">
                {quickFilters.map((filter) => (
                  <DropdownMenuItem key={filter.id} onClick={() => setQuickFilter(filter.id)}>
                    <span className="flex-1">{filter.label}</span>
                    {quickFilter === filter.id && <Check className="h-4 w-4 text-primary" />}
                  </DropdownMenuItem>
                ))}
                <DropdownMenuSeparator />
                {categoryFilters.map((category) => (
                  <DropdownMenu key={category.id}>
                    <DropdownMenuTrigger className="flex w-full items-center px-2 py-1.5 text-sm outline-none hover:bg-accent">
                      <span className="flex-1 text-left">{category.label}</span>
                      <ChevronDown className="h-3.5 w-3.5" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent side="right" className="w-40">
                      {category.options.map((option) => (
                        <DropdownMenuItem 
                          key={option} 
                          onClick={() => setCategoryFilter({ type: category.id, value: option })}
                        >
                          {option}
                          {categoryFilter?.type === category.id && categoryFilter?.value === option && (
                            <Check className="ml-auto h-4 w-4 text-primary" />
                          )}
                        </DropdownMenuItem>
                      ))}
                    </DropdownMenuContent>
                  </DropdownMenu>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Category Filter Dropdown (Type filter like 电影) */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 gap-1.5 text-sm">
                  电影
                  <ChevronDown className="h-3.5 w-3.5" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start">
                <DropdownMenuItem>全部</DropdownMenuItem>
                <DropdownMenuItem>电影</DropdownMenuItem>
                <DropdownMenuItem>剧集</DropdownMenuItem>
                <DropdownMenuItem>预告片</DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Sort Dropdown */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 gap-1.5 text-sm">
                  按 {sortOptions.find(s => s.id === sortBy)?.label} 排序
                  {sortOrder === "asc" ? <ArrowUp className="h-3.5 w-3.5" /> : <ArrowDown className="h-3.5 w-3.5" />}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" className="w-40">
                {sortOptions.map((option) => (
                  <DropdownMenuItem key={option.id} onClick={() => handleSortClick(option.id)}>
                    <span className="flex-1">{option.label}</span>
                    {sortBy === option.id && (
                      sortOrder === "asc" ? <ArrowUp className="h-4 w-4 text-primary" /> : <ArrowDown className="h-4 w-4 text-primary" />
                    )}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {/* Item count */}
            <span className="text-sm text-muted-foreground">{filteredItems.length}</span>

            {/* Active filter badge */}
            {categoryFilter && (
              <Badge variant="secondary" className="gap-1">
                {categoryFilter.value}
                <X className="h-3 w-3 cursor-pointer" onClick={() => setCategoryFilter(null)} />
              </Badge>
            )}

            <div className="flex-1" />

            {/* Action buttons */}
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8">
                    <Play className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>播放全部</TooltipContent>
              </Tooltip>
            </TooltipProvider>

            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8">
                    <Shuffle className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>随机播放</TooltipContent>
              </Tooltip>
            </TooltipProvider>

            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => setIsSelectionMode(!isSelectionMode)}>
                    <SlidersHorizontal className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>批量操作</TooltipContent>
              </Tooltip>
            </TooltipProvider>

            {/* View mode */}
            <div className="flex items-center gap-0.5 rounded-md border border-border p-0.5">
              <Button 
                variant={viewMode === "grid" ? "secondary" : "ghost"} 
                size="icon" 
                className="h-7 w-7"
                onClick={() => setViewMode("grid")}
              >
                <LayoutGrid className="h-3.5 w-3.5" />
              </Button>
              <Button 
                variant={viewMode === "detail" ? "secondary" : "ghost"} 
                size="icon" 
                className="h-7 w-7"
                onClick={() => setViewMode("detail")}
              >
                <List className="h-3.5 w-3.5" />
              </Button>
              <Button 
                variant={viewMode === "table" ? "secondary" : "ghost"} 
                size="icon" 
                className="h-7 w-7"
                onClick={() => setViewMode("table")}
              >
                <Table2 className="h-3.5 w-3.5" />
              </Button>
            </div>

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={() => setIsSelectionMode(true)}>批量选择</DropdownMenuItem>
                <DropdownMenuItem>导出列表</DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem>刷新元数据</DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        )}

          {/* Selection bar */}
          {isSelectionMode && (
          <div className="flex items-center gap-3 border-t border-primary/20 bg-primary/5 px-4 py-2">
            <Button variant="ghost" size="sm" onClick={clearSelection}>
              <X className="mr-1.5 h-4 w-4" />
              取消
            </Button>
            <Button variant="ghost" size="sm" onClick={selectAll}>
              <CheckSquare className="mr-1.5 h-4 w-4" />
              全选
            </Button>
            <span className="text-sm text-muted-foreground">已选择 {selectedItems.size} 项</span>
            <div className="flex-1" />
            <Button variant="ghost" size="sm" disabled={selectedItems.size === 0}>
              <Tag className="mr-1.5 h-4 w-4" />
              编辑标签
            </Button>
            {isAdmin && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="text-destructive hover:text-destructive"
                disabled={selectedItems.size === 0}
                onClick={() => setDeleteDialogOpen(true)}
              >
                <Trash2 className="mr-1.5 h-4 w-4" />
                删除
              </Button>
            )}
          </div>
        )}
      </header>

      {/* Content */}
      <main ref={containerRef} className="flex-1 overflow-y-auto scrollbar-none">
        {activeTab === "library" && (
          <div className="p-4">
            {/* Skeleton Loading */}
            {isInitialLoading && viewMode === "grid" && (
              <div className={cn("grid gap-4", getGridColumns())}>
                {Array.from({ length: 18 }).map((_, i) => (
                  <div key={i} className="space-y-2">
                    <Skeleton className="aspect-[2/3] w-full rounded-lg" />
                    <Skeleton className="h-4 w-3/4" />
                    <Skeleton className="h-3 w-1/2" />
                  </div>
                ))}
              </div>
            )}
            
            {isInitialLoading && viewMode === "detail" && (
              <div className="space-y-2">
                {Array.from({ length: 10 }).map((_, i) => (
                  <div key={i} className="flex gap-4 rounded-lg border border-border p-3">
                    <Skeleton className="h-24 w-16 flex-shrink-0 rounded" />
                    <div className="flex-1 space-y-2">
                      <Skeleton className="h-5 w-1/3" />
                      <Skeleton className="h-4 w-1/4" />
                      <Skeleton className="h-3 w-2/3" />
                    </div>
                  </div>
                ))}
              </div>
            )}
            
            {isInitialLoading && viewMode === "table" && (
              <div className="space-y-2">
                {Array.from({ length: 15 }).map((_, i) => (
                  <div key={i} className="flex gap-4 py-2">
                    <Skeleton className="h-4 w-1/4" />
                    <Skeleton className="h-4 w-16" />
                    <Skeleton className="h-4 w-12" />
                    <Skeleton className="h-4 w-16" />
                    <Skeleton className="h-4 w-20" />
                  </div>
                ))}
              </div>
            )}

            {/* Grid View */}
            {!isInitialLoading && viewMode === "grid" && (
              <div className={cn("grid gap-4", getGridColumns())}>
                {visibleItems.map((item) => (
                  <MediaGridItem
                    key={item.id}
                    item={item}
                    isSelectionMode={isSelectionMode}
                    isSelected={selectedItems.has(item.id)}
                    onSelect={() => toggleItemSelection(item.id)}
                          onClick={() => !isSelectionMode && onSelectMedia?.(item.id)}
                          onEdit={() => onEditMedia?.(item.id)}
                          isAdmin={isAdmin}
                        />
                ))}
              </div>
            )}

            {/* Detail View - Virtualized */}
            {!isInitialLoading && viewMode === "detail" && (
              <div 
                ref={detailListRef}
                className="h-[calc(100vh-200px)] overflow-auto"
                style={{ scrollbarWidth: "thin" }}
              >
                <div
                  style={{
                    height: `${detailVirtualizer.getTotalSize()}px`,
                    width: "100%",
                    position: "relative",
                  }}
                >
                  {detailVirtualizer.getVirtualItems().map((virtualRow) => {
                    const item = visibleItems[virtualRow.index]
                    return (
                      <div
                        key={virtualRow.key}
                        style={{
                          position: "absolute",
                          top: 0,
                          left: 0,
                          width: "100%",
                          height: `${virtualRow.size}px`,
                          transform: `translateY(${virtualRow.start}px)`,
                        }}
                      >
                        <div className="pb-2">
                          <MediaDetailItem
                            item={item}
                            isSelectionMode={isSelectionMode}
                            isSelected={selectedItems.has(item.id)}
                            onSelect={() => toggleItemSelection(item.id)}
                            onClick={() => !isSelectionMode && onSelectMedia?.(item.id)}
                            onEdit={() => onEditMedia?.(item.id)}
                            isAdmin={isAdmin}
                          />
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            )}

            {/* Table View - Virtualized */}
            {!isInitialLoading && viewMode === "table" && (
              <div className="rounded-lg border border-border">
                <table className="w-full text-sm">
                  <thead className="bg-muted/50 sticky top-0 z-10">
                    <tr>
                      {isSelectionMode && <th className="w-10 p-2" />}
                      <th className="p-2 text-left font-medium">标题</th>
                      <th className="p-2 text-left font-medium">年份</th>
                      <th className="p-2 text-left font-medium">评分</th>
                      <th className="p-2 text-left font-medium">时长</th>
                      <th className="p-2 text-left font-medium">分辨率</th>
                      <th className="p-2 text-left font-medium">播放</th>
                      <th className="p-2 text-left font-medium">添加日期</th>
                      {isAdmin && <th className="w-10 p-2" />}
                    </tr>
                  </thead>
                </table>
                <div 
                  ref={tableContainerRef}
                  className="h-[calc(100vh-250px)] overflow-auto"
                  style={{ scrollbarWidth: "thin" }}
                >
                  <div
                    style={{
                      height: `${tableVirtualizer.getTotalSize()}px`,
                      width: "100%",
                      position: "relative",
                    }}
                  >
                    {tableVirtualizer.getVirtualItems().map((virtualRow) => {
                      const item = visibleItems[virtualRow.index]
                      return (
                        <div
                          key={virtualRow.key}
                          style={{
                            position: "absolute",
                            top: 0,
                            left: 0,
                            width: "100%",
                            height: `${virtualRow.size}px`,
                            transform: `translateY(${virtualRow.start}px)`,
                          }}
                          className="flex items-center border-b border-border hover:bg-muted/30 cursor-pointer"
                          onClick={() => !isSelectionMode && onSelectMedia?.(item.id)}
                        >
                          {isSelectionMode && (
                            <div className="w-10 p-2 shrink-0" onClick={(e) => e.stopPropagation()}>
                              <button onClick={() => toggleItemSelection(item.id)}>
                                {selectedItems.has(item.id) ? (
                                  <CheckSquare className="h-4 w-4 text-primary" />
                                ) : (
                                  <Square className="h-4 w-4 text-muted-foreground" />
                                )}
                              </button>
                            </div>
                          )}
                          <div className="flex-1 p-2 min-w-0">
                            <div className="flex items-center gap-2">
                              <img src={item.poster} alt={item.title} className="h-9 w-6 rounded object-cover shrink-0" loading="lazy" />
                              <span className="font-medium truncate">{item.title}</span>
                            </div>
                          </div>
                          <div className="w-16 p-2 text-muted-foreground shrink-0">{item.year}</div>
                          <div className="w-16 p-2 text-muted-foreground shrink-0">{item.rating?.toFixed(1) || "-"}</div>
                          <div className="w-20 p-2 text-muted-foreground shrink-0">{item.duration || "-"}</div>
                          <div className="w-24 p-2 text-muted-foreground shrink-0">{item.resolution || "-"}</div>
                          <div className="w-16 p-2 text-muted-foreground shrink-0">{item.playCount}次</div>
                          <div className="w-24 p-2 text-muted-foreground shrink-0">{new Date(item.addedAt).toLocaleDateString()}</div>
                          {isAdmin && (
                            <div className="w-10 p-2 shrink-0" onClick={(e) => e.stopPropagation()}>
                              <Button variant="ghost" size="icon" className="h-7 w-7" onClick={() => onEditMedia?.(item.id)}>
                                <Pencil className="h-3.5 w-3.5" />
                              </Button>
                            </div>
                          )}
                        </div>
                      )
                    })}
                  </div>
                </div>
              </div>
            )}

            {/* Load more trigger */}
            <div ref={loadMoreRef} className="py-6">
              {isLoadingMore && (
                <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
                  {[1, 2, 3, 4].map((i) => (
                    <div key={i} className="space-y-2">
                      <Skeleton className="aspect-[2/3] w-full rounded-lg" />
                      <Skeleton className="h-3 w-full" />
                    </div>
                  ))}
                </div>
              )}
              {!isLoadingMore && visibleItems.length < filteredItems.length && (
                <p className="text-center text-sm text-muted-foreground">向下滚动加载更多</p>
              )}
              {visibleItems.length >= filteredItems.length && visibleItems.length > 0 && (
                <p className="text-center text-sm text-muted-foreground">已加载全部 {filteredItems.length} 项</p>
              )}
            </div>
          </div>
        )}

        {/* Collections Tab */}
        {activeTab === "collections" && (
          <div className="grid grid-cols-2 gap-4 p-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
            {mockCollections.map((collection) => (
              <div key={collection.id} className="group cursor-pointer">
                <div className="relative aspect-[2/3] overflow-hidden rounded-lg border border-border bg-muted">
                  <img src={collection.poster} alt={collection.name} className="h-full w-full object-cover transition-transform group-hover:scale-105" loading="lazy" />
                  <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent" />
                  <div className="absolute bottom-0 left-0 right-0 p-3">
                    <h3 className="text-sm font-medium text-white">{collection.name}</h3>
                    <p className="text-xs text-white/70">{collection.itemCount} 项</p>
                  </div>
                </div>
              </div>
            ))}
            {isAdmin && (
              <button className="flex aspect-[2/3] items-center justify-center rounded-lg border-2 border-dashed border-border hover:border-primary hover:bg-primary/5">
                <div className="text-center">
                  <Plus className="mx-auto h-8 w-8 text-muted-foreground" />
                  <span className="mt-2 text-sm text-muted-foreground">新建合集</span>
                </div>
              </button>
            )}
          </div>
        )}

        {/* Categories Tab */}
        {activeTab === "categories" && (
          <div className="space-y-8 p-4">
            {categoryFilters.slice(0, 4).map((category) => (
              <div key={category.id}>
                <h3 className="mb-3 text-sm font-semibold text-foreground">{category.label}</h3>
                <div className="flex flex-wrap gap-2">
                  {category.options.map((option) => (
                    <Badge 
                      key={option} 
                      variant="secondary" 
                      className="cursor-pointer hover:bg-primary hover:text-primary-foreground"
                      onClick={() => {
                        setCategoryFilter({ type: category.id, value: option })
                        setActiveTab("library")
                      }}
                    >
                      {option}
                    </Badge>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Recommend Tab */}
        {activeTab === "recommend" && (
          <div className="space-y-8 p-4">
            <section>
              <h3 className="mb-3 text-lg font-semibold">继续观看</h3>
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
                {allItems.filter(i => i.progress && i.progress > 0).slice(0, 6).map((item) => (
                        <MediaGridItem key={item.id} item={item} onClick={() => onSelectMedia?.(item.id)} />
                ))}
              </div>
            </section>
            <section>
              <h3 className="mb-3 text-lg font-semibold">最近添加</h3>
              <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
                {allItems.slice(0, 12).map((item) => (
                        <MediaGridItem key={item.id} item={item} onClick={() => onSelectMedia?.(item.id)} />
                ))}
              </div>
            </section>
          </div>
        )}
      </main>

      {/* Delete Confirmation Dialog */}
      <Dialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>确认删除</DialogTitle>
            <DialogDescription>
              确定要删除选中的 {selectedItems.size} 个项目吗？此操作不可撤销。
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteDialogOpen(false)}>取消</Button>
            <Button variant="destructive" onClick={() => { clearSelection(); setDeleteDialogOpen(false) }}>删除</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// ============ Sub Components ============

interface MediaGridItemProps {
  item: MediaItem
  isSelectionMode?: boolean
  isSelected?: boolean
  onSelect?: () => void
  onClick?: () => void
  onEdit?: () => void
  isAdmin?: boolean
}

function MediaGridItem({ item, isSelectionMode, isSelected, onSelect, onClick, onEdit, isAdmin }: MediaGridItemProps) {

  return (
    <div 
      className={cn(
        "group relative cursor-pointer",
        isSelected && "ring-2 ring-primary ring-offset-2 ring-offset-background rounded-lg"
      )}
      onClick={isSelectionMode ? onSelect : onClick}
    >
      <div className="relative aspect-[2/3] overflow-hidden rounded-lg border border-border bg-muted">
        {/* Lazy loaded image */}
        <img 
          src={item.poster} 
          alt={item.title} 
          className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          loading="lazy"
          decoding="async"
        />
        
        {/* Progress bar */}
        {item.progress && item.progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
            <div className="h-full bg-primary" style={{ width: `${item.progress}%` }} />
          </div>
        )}

        {/* Hover overlay */}
        <div className="absolute inset-0 bg-black/60 opacity-0 transition-opacity group-hover:opacity-100">
          <div className="absolute inset-0 flex items-center justify-center">
            <Button size="icon" className="h-12 w-12 rounded-full">
              <Play className="h-6 w-6" />
            </Button>
          </div>
          {isAdmin && onEdit && (
            <Button 
              variant="ghost" 
              size="icon" 
              className="absolute left-1 top-1 h-7 w-7 text-white hover:bg-white/20"
              onClick={(e) => { e.stopPropagation(); onEdit() }}
            >
              <Pencil className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>

        {/* Selection checkbox */}
        {isSelectionMode && (
          <div className="absolute left-2 top-2">
            {isSelected ? (
              <CheckSquare className="h-5 w-5 text-primary" />
            ) : (
              <Square className="h-5 w-5 text-white/70" />
            )}
          </div>
        )}

        {/* Resolution badge */}
        {item.resolution && (
          <Badge className="absolute right-1 top-1 bg-black/70 text-[10px] text-white">
            {item.resolution}
          </Badge>
        )}
      </div>

      {/* Info */}
      <div className="mt-2 space-y-0.5">
          <h4 className="truncate text-sm font-medium">{item.title}</h4>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span>{item.year}</span>
            {item.rating && (
              <>
                <span>·</span>
                <span>{item.rating.toFixed(1)}</span>
              </>
            )}
          </div>
        <p className="text-xs text-muted-foreground">{item.playCount}次播放</p>
      </div>
    </div>
  )
}

interface MediaDetailItemProps {
  item: MediaItem
  isSelectionMode?: boolean
  isSelected?: boolean
  onSelect?: () => void
  onClick?: () => void
  onEdit?: () => void
  isAdmin?: boolean
}

function MediaDetailItem({ item, isSelectionMode, isSelected, onSelect, onClick, onEdit, isAdmin }: MediaDetailItemProps) {
  return (
    <div 
      className={cn(
        "group flex cursor-pointer gap-4 rounded-lg border border-border p-3 transition-colors hover:bg-muted/50",
        isSelected && "border-primary bg-primary/5"
      )}
      onClick={isSelectionMode ? onSelect : onClick}
    >
      {/* Selection */}
      {isSelectionMode && (
        <div className="flex items-center">
          {isSelected ? (
            <CheckSquare className="h-5 w-5 text-primary" />
          ) : (
            <Square className="h-5 w-5 text-muted-foreground" />
          )}
        </div>
      )}

      {/* Poster */}
      <div className="relative h-24 w-16 shrink-0 overflow-hidden rounded">
        <img src={item.poster} alt={item.title} className="h-full w-full object-cover" loading="lazy" />
        {item.progress && item.progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
            <div className="h-full bg-primary" style={{ width: `${item.progress}%` }} />
          </div>
        )}
      </div>

      {/* Info */}
      <div className="flex flex-1 flex-col justify-center">
        <h4 className="font-medium">{item.title}</h4>
        <p className="text-sm text-muted-foreground">{item.year} · {item.duration}</p>
        {item.overview && (
          <p className="mt-1 line-clamp-2 text-xs text-muted-foreground/80">{item.overview}</p>
        )}
        <div className="mt-1 flex items-center gap-2">
          {item.rating && <Badge variant="secondary">{item.rating.toFixed(1)}</Badge>}
          {item.resolution && <Badge variant="outline" className="text-xs">{item.resolution}</Badge>}
          <span className="text-xs text-muted-foreground">{item.playCount}次播放</span>
        </div>
      </div>

      {/* Actions */}
      <div className="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <Button variant="ghost" size="icon" className="h-8 w-8">
          <Play className="h-4 w-4" />
        </Button>
        {isAdmin && onEdit && (
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={(e) => { e.stopPropagation(); onEdit() }}>
            <Pencil className="h-4 w-4" />
          </Button>
        )}
      </div>
    </div>
  )
}
