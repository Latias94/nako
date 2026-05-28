"use client"

import { useState, useEffect, useRef, useMemo, useCallback } from "react"
import { useVirtualizer } from "@tanstack/react-virtual"
import { 
  Search, X, Clock, Film, Tv, User, Tag, Play, Star, Globe, HardDrive, 
  Settings, Loader2, Download, ExternalLink, Plug, ChevronDown, AlertCircle,
  Filter, SlidersHorizontal, ArrowUpDown, Magnet, Link2, FolderDown, 
  CheckCircle2, XCircle, Trash2, RotateCcw, Music, Image, BookOpen,
  Calendar, Gauge, Server, Zap, MoreHorizontal, Plus, RefreshCw, Check,
  ArrowUp, FileVideo, Clapperboard
} from "lucide-react"
import { Input } from "@/components/ui/input"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger, DropdownMenuSeparator, DropdownMenuCheckboxItem, DropdownMenuLabel } from "@/components/ui/dropdown-menu"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter } from "@/components/ui/dialog"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { cn } from "@/lib/utils"

// ===== 类型定义 =====

// 搜索源类型 - 支持多种来源
type SearchSourceType = "local" | "indexer" | "scraper" | "direct" | "webdav" | "alist"

interface SearchSource {
  id: string
  name: string
  type: SearchSourceType
  icon?: string
  enabled: boolean
  url?: string
  apiKey?: string
  priority: number
  categories: MediaCategory[]
  status: "connected" | "error" | "unconfigured" | "testing"
  lastSync?: Date
  resultCount?: number
  // 下载能力
  downloadCapability?: DownloadCapability
}

type MediaCategory = "movie" | "series" | "anime" | "music" | "book" | "photo" | "other"

// 下载能力类型
type DownloadCapability = "torrent" | "magnet" | "direct" | "webdav" | "alist" | "none"

// 搜索结果 - 统一格式
interface SearchResult {
  id: string
  title: string
  originalTitle?: string
  year?: number
  type: MediaCategory
  // 本地资源属性
  isLocal?: boolean
  localPath?: string
  quality?: string
  // 远程资源属性
  source?: string
  sourceId?: string
  size?: string
  seeders?: number
  leechers?: number
  downloadUrl?: string
  magnetUrl?: string
  infoUrl?: string
  // 通用属性
  poster?: string
  rating?: number
  description?: string
  // 元数据
  resolution?: string
  codec?: string
  audio?: string
  uploadTime?: Date
}

// 筛选器配置
interface SearchFilters {
  types: MediaCategory[]
  sources: string[]
  quality: string[]
  yearRange: [number, number] | null
  minSeeders: number
  sortBy: "relevance" | "seeders" | "size" | "date"
  sortOrder: "asc" | "desc"
}

// 下载器配置
interface Downloader {
  id: string
  name: string
  type: "qbittorrent" | "transmission" | "aria2" | "builtin"
  enabled: boolean
  isDefault: boolean
  supports: DownloadCapability[]
  status: "connected" | "error" | "unconfigured"
}

// ===== 默认配置 =====

const defaultSearchSources: SearchSource[] = [
  { 
    id: "local", 
    name: "本地媒体库", 
    type: "local", 
    enabled: true, 
    priority: 1, 
    categories: ["movie", "series", "anime", "music", "photo"], 
    status: "connected",
    downloadCapability: "none"
  },
  { 
    id: "jackett", 
    name: "Jackett", 
    type: "indexer", 
    enabled: false, 
    priority: 2, 
    categories: ["movie", "series", "anime"], 
    status: "unconfigured",
    downloadCapability: "torrent"
  },
  { 
    id: "prowlarr", 
    name: "Prowlarr", 
    type: "indexer", 
    enabled: false, 
    priority: 3, 
    categories: ["movie", "series", "anime", "music"], 
    status: "unconfigured",
    downloadCapability: "torrent"
  },
  { 
    id: "alist", 
    name: "Alist", 
    type: "alist", 
    enabled: false, 
    priority: 4, 
    categories: ["movie", "series", "anime"], 
    status: "unconfigured",
    downloadCapability: "alist"
  },
]

const defaultDownloaders: Downloader[] = [
  { id: "qbit", name: "qBittorrent", type: "qbittorrent", enabled: true, isDefault: true, supports: ["torrent", "magnet"], status: "connected" },
  { id: "aria2", name: "Aria2", type: "aria2", enabled: false, isDefault: false, supports: ["direct", "magnet"], status: "unconfigured" },
  { id: "builtin", name: "内置下载器", type: "builtin", enabled: true, isDefault: false, supports: ["direct", "webdav", "alist"], status: "connected" },
]

const defaultFilters: SearchFilters = {
  types: [],
  sources: [],
  quality: [],
  yearRange: null,
  minSeeders: 0,
  sortBy: "relevance",
  sortOrder: "desc"
}

const qualityOptions = ["4K", "4K HDR", "4K DV", "1080p", "720p", "其他"]
const categoryIcons: Record<MediaCategory, React.ReactNode> = {
  movie: <Film className="h-4 w-4" />,
  series: <Tv className="h-4 w-4" />,
  anime: <span className="text-sm">🎨</span>,
  music: <Music className="h-4 w-4" />,
  book: <BookOpen className="h-4 w-4" />,
  photo: <Image className="h-4 w-4" />,
  other: <Tag className="h-4 w-4" />,
}
const categoryNames: Record<MediaCategory, string> = {
  movie: "电影",
  series: "剧集",
  anime: "动画",
  music: "音乐",
  book: "书籍",
  photo: "图片",
  other: "其他",
}

// ===== 模拟数据 =====

const mockLocalResults: SearchResult[] = [
  { id: "l1", title: "沙丘2", originalTitle: "Dune: Part Two", year: 2024, type: "movie", isLocal: true, quality: "4K HDR", rating: 8.6, localPath: "/movies/dune2", poster: "https://image.tmdb.org/t/p/w300/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg" },
  { id: "l2", title: "沙丘", originalTitle: "Dune", year: 2021, type: "movie", isLocal: true, quality: "4K", rating: 8.0, localPath: "/movies/dune", poster: "https://image.tmdb.org/t/p/w300/d5NXSklXo0qyIYkgV94XAgMIckC.jpg" },
  { id: "l3", title: "真探", originalTitle: "True Detective", year: 2014, type: "series", isLocal: true, quality: "1080p", rating: 8.9, localPath: "/series/true-detective", poster: "https://image.tmdb.org/t/p/w300/7wUxRFB7JVNX5fGobvV6kDaAHE0.jpg" },
  ]

const mockRemoteResults: SearchResult[] = [
  { id: "r1", title: "Dune.Part.Two.2024.2160p.UHD.BluRay.x265.HDR.DTS-HD.MA.7.1", year: 2024, type: "movie", source: "Jackett", sourceId: "jackett", quality: "4K HDR", size: "45.2 GB", seeders: 1250, leechers: 89, magnetUrl: "magnet:?xt=..." },
  { id: "r2", title: "Dune.Part.Two.2024.1080p.BluRay.x264.DTS-HD.MA.7.1", year: 2024, type: "movie", source: "Jackett", sourceId: "jackett", quality: "1080p", size: "18.6 GB", seeders: 890, leechers: 45, magnetUrl: "magnet:?xt=..." },
  { id: "r3", title: "沙丘2 Dune.Part.Two.2024.2160p.WEB-DL", year: 2024, type: "movie", source: "Alist", sourceId: "alist", quality: "4K", size: "22.1 GB", downloadUrl: "https://..." },
  { id: "r4", title: "沙丘2.Dune.Part.Two.2024.CHINESE.2160p.UHD.BluRay", year: 2024, type: "movie", source: "Prowlarr", sourceId: "prowlarr", quality: "4K 中字", size: "48.5 GB", seeders: 445, leechers: 28, magnetUrl: "magnet:?xt=..." },
]

// ===== 组件 =====

interface SearchPageProps {
  onBack: () => void
  initialQuery?: string
}

export function SearchPage({ onBack, initialQuery = "" }: SearchPageProps) {
  // 状态
  const [query, setQuery] = useState(initialQuery)
  const [isSearching, setIsSearching] = useState(false)
  const [hasSearched, setHasSearched] = useState(false)
  const [recentSearches, setRecentSearches] = useState<string[]>(["沙丘", "真探", "诺兰", "星际穿越"])
  
  // 搜索源和结果
  const [sources, setSources] = useState<SearchSource[]>(defaultSearchSources)
  const [localResults, setLocalResults] = useState<SearchResult[]>([])
  const [remoteResults, setRemoteResults] = useState<SearchResult[]>([])
  
  // 筛选和排序
  const [filters, setFilters] = useState<SearchFilters>(defaultFilters)
  const [activeSourceTab, setActiveSourceTab] = useState<"all" | "local" | "remote">("all")
  
  // 下载器
  const [downloaders, setDownloaders] = useState<Downloader[]>(defaultDownloaders)
  const [downloadDialog, setDownloadDialog] = useState<{ open: boolean; result: SearchResult | null }>({ open: false, result: null })
  
  // 配置对话框
  const [sourceConfigDialog, setSourceConfigDialog] = useState(false)
  const [editingSource, setEditingSource] = useState<SearchSource | null>(null)
  
  const inputRef = useRef<HTMLInputElement>(null)
  
  // 聚焦输入框
  useEffect(() => {
    inputRef.current?.focus()
  }, [])

  // 活跃的搜索源
  const activeSources = useMemo(() => sources.filter(s => s.enabled && s.status === "connected"), [sources])
  const hasRemoteSources = useMemo(() => sources.some(s => s.type !== "local" && s.enabled && s.status === "connected"), [sources])

  // 筛选后的结果
  const filteredResults = useMemo(() => {
    let results: SearchResult[] = []
    
    if (activeSourceTab === "all" || activeSourceTab === "local") {
      results = [...results, ...localResults]
    }
    if (activeSourceTab === "all" || activeSourceTab === "remote") {
      results = [...results, ...remoteResults]
    }
    
    // 应用筛选器
    if (filters.types.length > 0) {
      results = results.filter(r => filters.types.includes(r.type))
    }
    if (filters.sources.length > 0) {
      results = results.filter(r => r.isLocal ? filters.sources.includes("local") : filters.sources.includes(r.sourceId || ""))
    }
    if (filters.quality.length > 0) {
      results = results.filter(r => r.quality && filters.quality.some(q => r.quality!.includes(q)))
    }
    if (filters.minSeeders > 0) {
      results = results.filter(r => !r.seeders || r.seeders >= filters.minSeeders)
    }
    
    // 排序
    results.sort((a, b) => {
      let comparison = 0
      switch (filters.sortBy) {
        case "seeders":
          comparison = (b.seeders || 0) - (a.seeders || 0)
          break
        case "size":
          const sizeA = parseFloat(a.size?.replace(/[^0-9.]/g, "") || "0")
          const sizeB = parseFloat(b.size?.replace(/[^0-9.]/g, "") || "0")
          comparison = sizeB - sizeA
          break
        case "date":
          comparison = (b.uploadTime?.getTime() || 0) - (a.uploadTime?.getTime() || 0)
          break
        default:
          // relevance - 本地优先
          comparison = (b.isLocal ? 1 : 0) - (a.isLocal ? 1 : 0)
      }
      return filters.sortOrder === "asc" ? -comparison : comparison
    })
    
    return results
  }, [localResults, remoteResults, activeSourceTab, filters])

  // 按来源分组远程结果
  const groupedRemoteResults = useMemo(() => {
    const groups: Record<string, SearchResult[]> = {}
    remoteResults.forEach(result => {
      const sourceId = result.sourceId || "unknown"
      if (!groups[sourceId]) groups[sourceId] = []
      groups[sourceId].push(result)
    })
    return groups
  }, [remoteResults])

  // 搜索处理
  const handleSearch = async () => {
    if (!query.trim()) return
    performSearch(query)
  }

  // 执行搜索 - 支持传入搜索词
  const performSearch = async (searchQuery: string) => {
    if (!searchQuery.trim()) return
    
    setIsSearching(true)
    setHasSearched(true)
    
    // 添加到最近搜索
    setRecentSearches(prev => {
      const filtered = prev.filter(s => s !== searchQuery)
      return [searchQuery, ...filtered].slice(0, 10)
    })
    
    // 模拟搜索延迟
    await new Promise(resolve => setTimeout(resolve, 800))
    
    // 本地搜索
    setLocalResults(mockLocalResults.filter(r => 
      r.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      r.originalTitle?.toLowerCase().includes(searchQuery.toLowerCase())
    ))
    
    // 远程搜索（仅当有活跃的远程源时）
    if (hasRemoteSources) {
      await new Promise(resolve => setTimeout(resolve, 500))
      setRemoteResults(mockRemoteResults.filter(r => 
        r.title.toLowerCase().includes(searchQuery.toLowerCase())
      ))
    } else {
      setRemoteResults([])
    }
    
    setIsSearching(false)
  }

  // 键盘事件
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSearch()
    }
    if (e.key === "Escape") {
      if (query) {
        setQuery("")
      } else {
        onBack()
      }
    }
  }

  // 下载处理
  const handleDownload = (result: SearchResult) => {
    // 本地资源直接播放
    if (result.isLocal) {
      console.log("Play local:", result.localPath)
      return
    }
    // 远程资源显示下载对话框
    setDownloadDialog({ open: true, result })
  }

  // 确认下载
  const confirmDownload = (downloaderId: string) => {
    const result = downloadDialog.result
    if (!result) return
    
    console.log("Download:", result.title, "with", downloaderId)
    setDownloadDialog({ open: false, result: null })
    // TODO: 调用实际下载 API
  }

  // 清除筛选
  const clearFilters = () => {
    setFilters(defaultFilters)
  }

  // 活跃筛选数量
  const activeFilterCount = useMemo(() => {
    let count = 0
    if (filters.types.length > 0) count++
    if (filters.sources.length > 0) count++
    if (filters.quality.length > 0) count++
    if (filters.minSeeders > 0) count++
    return count
  }, [filters])

  // 本地媒体卡片 - 海报风格
  const LocalMediaCard = ({ result }: { result: SearchResult }) => (
    <div 
      className="group relative cursor-pointer overflow-hidden rounded-xl bg-card/50 transition-all hover:ring-2 hover:ring-cyan-500/50"
      onClick={() => console.log("Play:", result.localPath)}
    >
      {/* 海报 */}
      <div className="relative aspect-[2/3] bg-muted">
        {result.poster ? (
          <img 
            src={result.poster} 
            alt={result.title} 
            className="h-full w-full object-cover transition-transform group-hover:scale-105"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center bg-gradient-to-br from-muted to-muted/50">
            <Film className="h-12 w-12 text-muted-foreground/30" />
          </div>
        )}
        
        {/* 悬浮播放按钮 */}
        <div className="absolute inset-0 flex items-center justify-center bg-black/0 opacity-0 transition-all group-hover:bg-black/40 group-hover:opacity-100">
          <div className="flex h-14 w-14 items-center justify-center rounded-full bg-cyan-500 shadow-lg">
            <Play className="h-6 w-6 fill-white text-white" />
          </div>
        </div>
        
        {/* 画质标签 */}
        {result.quality && (
          <Badge className="absolute right-2 top-2 bg-black/70 text-[10px] font-medium text-white backdrop-blur">
            {result.quality}
          </Badge>
        )}
        
        {/* 评分 */}
        {result.rating && (
          <div className="absolute bottom-2 left-2 flex items-center gap-1 rounded bg-black/70 px-1.5 py-0.5 backdrop-blur">
            <Star className="h-3 w-3 fill-amber-400 text-amber-400" />
            <span className="text-xs font-medium text-white">{result.rating}</span>
          </div>
        )}
      </div>
      
      {/* 标题信息 */}
      <div className="p-3">
        <h4 className="truncate font-medium text-sm">{result.title}</h4>
        <div className="mt-1 flex items-center gap-2 text-xs text-muted-foreground">
          {result.year && <span>{result.year}</span>}
          {result.type && (
            <Badge variant="outline" className="h-4 border-0 bg-muted px-1.5 text-[10px]">
              {result.type === "movie" ? "电影" : result.type === "series" ? "剧集" : "动画"}
            </Badge>
          )}
        </div>
      </div>
    </div>
  )

  // 远程资源卡片 - 列表风格
  const RemoteResourceCard = ({ result, onDownload }: { result: SearchResult; onDownload: () => void }) => (
    <div className="group flex items-center gap-4 rounded-xl border border-border/50 bg-card/30 p-3 transition-colors hover:bg-muted/30">
      {/* 类型图标 */}
      <div className={cn(
        "flex h-12 w-12 shrink-0 items-center justify-center rounded-lg",
        result.type === "movie" ? "bg-purple-500/10 text-purple-500" :
        result.type === "series" ? "bg-blue-500/10 text-blue-500" :
        result.type === "anime" ? "bg-pink-500/10 text-pink-500" :
        "bg-muted text-muted-foreground"
      )}>
        {result.type === "movie" ? <Film className="h-5 w-5" /> :
         result.type === "series" ? <Tv className="h-5 w-5" /> :
         result.type === "anime" ? <Clapperboard className="h-5 w-5" /> :
         <FileVideo className="h-5 w-5" />}
      </div>
      
      {/* 信息 */}
      <div className="flex-1 min-w-0">
        <h4 className="truncate font-medium text-sm">{result.title}</h4>
        <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
          {result.quality && (
            <Badge variant="secondary" className="h-4 px-1.5 text-[10px]">{result.quality}</Badge>
          )}
          {result.size && <span>{result.size}</span>}
          {result.seeders !== undefined && (
            <span className={cn(
              "flex items-center gap-1",
              result.seeders > 10 ? "text-green-500" : result.seeders > 0 ? "text-amber-500" : "text-red-500"
            )}>
              <ArrowUp className="h-3 w-3" />
              {result.seeders}
            </span>
          )}
          {result.uploadTime && (
            <span>{result.uploadTime.toLocaleDateString()}</span>
          )}
        </div>
      </div>
      
      {/* 下载按钮 */}
      <Button 
        size="sm" 
        className="shrink-0 gap-1.5 bg-cyan-500 opacity-0 transition-opacity group-hover:opacity-100 hover:bg-cyan-600"
        onClick={(e) => {
          e.stopPropagation()
          onDownload()
        }}
      >
        <Download className="h-3.5 w-3.5" />
        下载
      </Button>
    </div>
  )

  // 虚拟化的源结果组件 - 当结果超过阈值时使用虚拟列表
  const VirtualizedSourceResults = ({ 
    source, 
    results, 
    onDownload 
  }: { 
    source: SearchSource
    results: SearchResult[]
    onDownload: (result: SearchResult) => void 
  }) => {
    const [isExpanded, setIsExpanded] = useState(false)
    const parentRef = useRef<HTMLDivElement>(null)
    
    const displayResults = isExpanded ? results : results.slice(0, 5)
    const useVirtualization = isExpanded && results.length > 20
    
    const virtualizer = useVirtualizer({
      count: useVirtualization ? results.length : 0,
      getScrollElement: () => parentRef.current,
      estimateSize: () => 68, // 每个项目的估计高度
      overscan: 5,
    })
    
    return (
      <div className="space-y-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div className={cn(
              "flex h-7 w-7 items-center justify-center rounded-lg",
              source.type === "indexer" ? "bg-orange-500/10" : "bg-blue-500/10"
            )}>
              {source.type === "indexer" ? (
                <Plug className="h-4 w-4 text-orange-500" />
              ) : (
                <FolderDown className="h-4 w-4 text-blue-500" />
              )}
            </div>
            <h3 className="font-medium">{source.name}</h3>
            <Badge variant="secondary" className="text-xs">{results.length}</Badge>
          </div>
          {results.length > 5 && (
            <Button 
              variant="ghost" 
              size="sm" 
              className="text-xs text-muted-foreground"
              onClick={() => setIsExpanded(!isExpanded)}
            >
              {isExpanded ? "收起" : `展开全部 (${results.length})`}
            </Button>
          )}
        </div>
        
        {/* 资源列表 - 根据数量决定是否使用虚拟化 */}
        {useVirtualization ? (
          <div 
            ref={parentRef}
            className="max-h-[400px] overflow-auto rounded-lg"
            style={{ scrollbarWidth: "thin" }}
          >
            <div
              style={{
                height: `${virtualizer.getTotalSize()}px`,
                width: "100%",
                position: "relative",
              }}
            >
              {virtualizer.getVirtualItems().map((virtualRow) => (
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
                    <RemoteResourceCard 
                      result={results[virtualRow.index]} 
                      onDownload={() => onDownload(results[virtualRow.index])}
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-2">
            {displayResults.map(result => (
              <RemoteResourceCard 
                key={result.id} 
                result={result} 
                onDownload={() => onDownload(result)}
              />
            ))}
            {!isExpanded && results.length > 5 && (
              <Button 
                variant="ghost" 
                className="w-full text-xs text-muted-foreground"
                onClick={() => setIsExpanded(true)}
              >
                显示更多 ({results.length - 5})
              </Button>
            )}
          </div>
        )}
      </div>
    )
  }

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Header - 搜索栏 */}
      <div className="sticky top-0 z-10 border-b border-border/50 bg-background/95 px-4 py-4 backdrop-blur supports-[backdrop-filter]:bg-background/80">
        <div className="mx-auto max-w-5xl">
          {/* 搜索输入框 */}
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="icon" className="shrink-0" onClick={onBack}>
              <X className="h-5 w-5" />
            </Button>
            
            <div className="relative flex-1">
              <Search className="absolute left-4 top-1/2 h-5 w-5 -translate-y-1/2 text-muted-foreground" />
              <Input
                ref={inputRef}
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="搜索电影、剧集、演员..."
                className="h-12 rounded-full border-border/50 bg-muted/30 pl-12 pr-4 text-base focus-visible:ring-cyan-500/50"
              />
              {query && (
                <Button
                  variant="ghost"
                  size="icon"
                  className="absolute right-2 top-1/2 h-8 w-8 -translate-y-1/2"
                  onClick={() => setQuery("")}
                >
                  <X className="h-4 w-4" />
                </Button>
              )}
            </div>

            <Button 
              onClick={handleSearch} 
              disabled={!query.trim() || isSearching}
              className="shrink-0 rounded-full bg-cyan-500 px-6 hover:bg-cyan-600"
            >
              {isSearching ? <Loader2 className="h-4 w-4 animate-spin" /> : "搜索"}
            </Button>
          </div>

          {/* 搜索源和筛选 - 有搜索结果时显示 */}
          {hasSearched && (
            <div className="mt-4 flex flex-wrap items-center gap-2">
              {/* 来源切换 */}
              <Tabs value={activeSourceTab} onValueChange={(v) => setActiveSourceTab(v as typeof activeSourceTab)} className="shrink-0">
                <TabsList className="h-8 bg-muted/50">
                  <TabsTrigger value="all" className="h-6 px-3 text-xs">
                    全部
                    {(localResults.length + remoteResults.length) > 0 && (
                      <Badge variant="secondary" className="ml-1.5 h-4 px-1 text-[10px]">
                        {localResults.length + remoteResults.length}
                      </Badge>
                    )}
                  </TabsTrigger>
                  <TabsTrigger value="local" className="h-6 gap-1.5 px-3 text-xs">
                    <HardDrive className="h-3 w-3" />
                    本地
                    {localResults.length > 0 && (
                      <Badge variant="secondary" className="ml-1 h-4 px-1 text-[10px]">{localResults.length}</Badge>
                    )}
                  </TabsTrigger>
                  <TabsTrigger value="remote" className="h-6 gap-1.5 px-3 text-xs" disabled={!hasRemoteSources}>
                    <Globe className="h-3 w-3" />
                    在线
                    {remoteResults.length > 0 && (
                      <Badge variant="secondary" className="ml-1 h-4 px-1 text-[10px]">{remoteResults.length}</Badge>
                    )}
                  </TabsTrigger>
                </TabsList>
              </Tabs>

              <div className="h-4 w-px bg-border/50" />

              {/* 类型筛选 */}
              <Popover>
                <PopoverTrigger asChild>
                  <Button variant="outline" size="sm" className={cn("h-8 gap-1.5 text-xs", filters.types.length > 0 && "border-cyan-500/50 bg-cyan-500/10")}>
                    <Film className="h-3.5 w-3.5" />
                    类型
                    {filters.types.length > 0 && (
                      <Badge className="h-4 px-1 text-[10px]">{filters.types.length}</Badge>
                    )}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-48 p-2" align="start">
                  {(Object.keys(categoryNames) as MediaCategory[]).map(cat => (
                    <label key={cat} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 hover:bg-muted">
                      <input
                        type="checkbox"
                        checked={filters.types.includes(cat)}
                        onChange={(e) => {
                          setFilters(prev => ({
                            ...prev,
                            types: e.target.checked 
                              ? [...prev.types, cat]
                              : prev.types.filter(t => t !== cat)
                          }))
                        }}
                        className="rounded border-muted-foreground"
                      />
                      <span className="flex items-center gap-2 text-sm">
                        {categoryIcons[cat]}
                        {categoryNames[cat]}
                      </span>
                    </label>
                  ))}
                </PopoverContent>
              </Popover>

              {/* 画质筛选 */}
              <Popover>
                <PopoverTrigger asChild>
                  <Button variant="outline" size="sm" className={cn("h-8 gap-1.5 text-xs", filters.quality.length > 0 && "border-cyan-500/50 bg-cyan-500/10")}>
                    <Gauge className="h-3.5 w-3.5" />
                    画质
                    {filters.quality.length > 0 && (
                      <Badge className="h-4 px-1 text-[10px]">{filters.quality.length}</Badge>
                    )}
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-40 p-2" align="start">
                  {qualityOptions.map(q => (
                    <label key={q} className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 hover:bg-muted">
                      <input
                        type="checkbox"
                        checked={filters.quality.includes(q)}
                        onChange={(e) => {
                          setFilters(prev => ({
                            ...prev,
                            quality: e.target.checked 
                              ? [...prev.quality, q]
                              : prev.quality.filter(x => x !== q)
                          }))
                        }}
                        className="rounded border-muted-foreground"
                      />
                      <span className="text-sm">{q}</span>
                    </label>
                  ))}
                </PopoverContent>
              </Popover>

              {/* 排序 */}
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="outline" size="sm" className="h-8 gap-1.5 text-xs">
                    <ArrowUpDown className="h-3.5 w-3.5" />
                    {filters.sortBy === "relevance" && "相关性"}
                    {filters.sortBy === "seeders" && "做种数"}
                    {filters.sortBy === "size" && "文件大小"}
                    {filters.sortBy === "date" && "上传时间"}
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="start">
                  <DropdownMenuItem onClick={() => setFilters(prev => ({ ...prev, sortBy: "relevance" }))}>
                    <Check className={cn("mr-2 h-4 w-4", filters.sortBy !== "relevance" && "opacity-0")} />
                    相关性
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setFilters(prev => ({ ...prev, sortBy: "seeders" }))}>
                    <Check className={cn("mr-2 h-4 w-4", filters.sortBy !== "seeders" && "opacity-0")} />
                    做种数
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setFilters(prev => ({ ...prev, sortBy: "size" }))}>
                    <Check className={cn("mr-2 h-4 w-4", filters.sortBy !== "size" && "opacity-0")} />
                    文件大小
                  </DropdownMenuItem>
                  <DropdownMenuItem onClick={() => setFilters(prev => ({ ...prev, sortBy: "date" }))}>
                    <Check className={cn("mr-2 h-4 w-4", filters.sortBy !== "date" && "opacity-0")} />
                    上传时间
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>

              {/* 清除筛选 */}
              {activeFilterCount > 0 && (
                <Button variant="ghost" size="sm" className="h-8 gap-1.5 text-xs text-muted-foreground" onClick={clearFilters}>
                  <X className="h-3.5 w-3.5" />
                  清除筛选
                </Button>
              )}

              <div className="flex-1" />

              {/* 搜���源配置 */}
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button 
                      variant="outline" 
                      size="sm" 
                      className="h-8 gap-1.5 text-xs"
                      onClick={() => setSourceConfigDialog(true)}
                    >
                      <Server className="h-3.5 w-3.5" />
                      搜索源
                      <Badge variant="secondary" className="h-4 px-1 text-[10px]">
                        {activeSources.length}
                      </Badge>
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>管理搜索源</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
          )}
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="mx-auto max-w-5xl px-4 py-6">
          {/* 未搜索状态 - 显示最近搜索 */}
          {!hasSearched && (
            <div className="space-y-8">
              {/* 最近搜索 */}
              {recentSearches.length > 0 && (
                <div>
                  <div className="mb-4 flex items-center justify-between">
                    <h3 className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
                      <Clock className="h-4 w-4" />
                      最近搜索
                    </h3>
                    <Button 
                      variant="ghost" 
                      size="sm" 
                      className="h-7 text-xs text-muted-foreground"
                      onClick={() => setRecentSearches([])}
                    >
                      清除
                    </Button>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {recentSearches.map((term, i) => (
                      <Button
                        key={i}
                        variant="outline"
                        size="sm"
                        className="h-8 gap-2 rounded-full text-sm"
                        onClick={() => {
                          setQuery(term)
                          // 直接执行搜索而不是依赖 state 更新
                          performSearch(term)
                        }}
                      >
                        {term}
                      </Button>
                    ))}
                  </div>
                </div>
              )}

              {/* 搜索源状态 */}
              <div>
                <h3 className="mb-4 flex items-center gap-2 text-sm font-medium text-muted-foreground">
                  <Server className="h-4 w-4" />
                  搜索源
                </h3>
                <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                  {sources.map(source => (
                    <div 
                      key={source.id}
                      className={cn(
                        "flex items-center gap-3 rounded-xl border p-4 transition-colors",
                        source.enabled && source.status === "connected" 
                          ? "border-green-500/30 bg-green-500/5"
                          : source.enabled && source.status === "error"
                          ? "border-red-500/30 bg-red-500/5"
                          : "border-border/50 bg-card/30"
                      )}
                    >
                      <div className={cn(
                        "flex h-10 w-10 items-center justify-center rounded-lg",
                        source.type === "local" ? "bg-cyan-500/10 text-cyan-500" :
                        source.type === "indexer" ? "bg-orange-500/10 text-orange-500" :
                        source.type === "alist" ? "bg-blue-500/10 text-blue-500" :
                        "bg-muted text-muted-foreground"
                      )}>
                        {source.type === "local" ? <HardDrive className="h-5 w-5" /> :
                         source.type === "indexer" ? <Plug className="h-5 w-5" /> :
                         source.type === "alist" ? <FolderDown className="h-5 w-5" /> :
                         <Globe className="h-5 w-5" />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="font-medium text-sm truncate">{source.name}</span>
                          {source.status === "connected" && <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />}
                          {source.status === "error" && <XCircle className="h-3.5 w-3.5 text-red-500" />}
                          {source.status === "unconfigured" && <AlertCircle className="h-3.5 w-3.5 text-muted-foreground" />}
                        </div>
                        <p className="text-xs text-muted-foreground truncate">
                          {source.status === "connected" && "已连接"}
                          {source.status === "error" && "连接失败"}
                          {source.status === "unconfigured" && "未配置"}
                        </p>
                      </div>
                      <Switch 
                        checked={source.enabled && source.status === "connected"}
                        disabled={source.status === "unconfigured"}
                        onCheckedChange={(checked) => {
                          setSources(prev => prev.map(s => 
                            s.id === source.id ? { ...s, enabled: checked } : s
                          ))
                        }}
                      />
                    </div>
                  ))}
                  
                  {/* 添加搜索源 */}
                  <button 
                    className="flex items-center justify-center gap-2 rounded-xl border border-dashed border-border/50 p-4 text-muted-foreground transition-colors hover:border-cyan-500/50 hover:bg-cyan-500/5 hover:text-cyan-500"
                    onClick={() => setSourceConfigDialog(true)}
                  >
                    <Plus className="h-5 w-5" />
                    <span className="text-sm">配置搜索源</span>
                  </button>
                </div>
              </div>

              {/* 快捷提示 */}
              <div className="rounded-xl border border-border/50 bg-card/30 p-4">
                <h4 className="mb-2 font-medium text-sm">搜索技巧</h4>
                <ul className="space-y-1.5 text-xs text-muted-foreground">
                  <li className="flex items-center gap-2">
                    <kbd className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">Enter</kbd>
                    <span>开始搜索</span>
                  </li>
                  <li className="flex items-center gap-2">
                    <kbd className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">Esc</kbd>
                    <span>清除搜索词或返回</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">year:2024</span>
                    <span>按年份筛选</span>
                  </li>
                  <li className="flex items-start gap-2">
                    <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 font-mono text-[10px]">4k</span>
                    <span>搜索 4K 资源</span>
                  </li>
                </ul>
              </div>
            </div>
          )}

          {/* 搜索中 */}
          {isSearching && (
            <div className="flex flex-col items-center justify-center py-20">
              <Loader2 className="h-8 w-8 animate-spin text-cyan-500" />
              <p className="mt-4 text-muted-foreground">搜索中...</p>
              <div className="mt-2 flex items-center gap-2 text-xs text-muted-foreground">
                {activeSources.map(s => (
                  <Badge key={s.id} variant="outline" className="text-[10px]">{s.name}</Badge>
                ))}
              </div>
            </div>
          )}

          {/* 搜索结果 */}
          {hasSearched && !isSearching && (
            <div className="space-y-6">
              {filteredResults.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-20">
                  <Search className="h-12 w-12 text-muted-foreground/30" />
                  <p className="mt-4 text-muted-foreground">未找到相关结果</p>
                  <p className="mt-1 text-xs text-muted-foreground">尝试更换关键词或调整筛选条件</p>
                  {!hasRemoteSources && (
                    <Button 
                      variant="outline" 
                      size="sm" 
                      className="mt-4 gap-2"
                      onClick={() => setSourceConfigDialog(true)}
                    >
                      <Plug className="h-4 w-4" />
                      配置在线搜索源
                    </Button>
                  )}
                </div>
              ) : (
                <>
                  {/* 本地结果 - 海报卡片网格 */}
                  {localResults.length > 0 && (activeSourceTab === "all" || activeSourceTab === "local") && (
                    <div className="space-y-4">
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-cyan-500/10">
                            <HardDrive className="h-4 w-4 text-cyan-500" />
                          </div>
                          <h3 className="font-medium">本地媒体库</h3>
                          <Badge variant="secondary" className="text-xs">{localResults.length}</Badge>
                        </div>
                        <Button variant="ghost" size="sm" className="text-xs text-muted-foreground hover:bg-transparent hover:text-foreground">
                          查看全部
                        </Button>
                      </div>
                      
                      {/* 海报卡片网格 */}
                      <div className="grid gap-4 grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                        {localResults.map(result => (
                          <LocalMediaCard key={result.id} result={result} />
                        ))}
                      </div>
                    </div>
                  )}

                  {/* 在线结果 - 按来源分组 */}
                  {remoteResults.length > 0 && (activeSourceTab === "all" || activeSourceTab === "remote") && (
                    <div className="space-y-6">
                      {/* 按来源分组 */}
                      {Object.entries(groupedRemoteResults).map(([sourceId, results]) => {
                        const source = sources.find(s => s.id === sourceId)
                        if (!source || results.length === 0) return null
                        
                        return (
                          <VirtualizedSourceResults
                            key={sourceId}
                            source={source}
                            results={results}
                            onDownload={handleDownload}
                          />
                        )
                      })}
                    </div>
                  )}
                </>
              )}
            </div>
          )}
        </div>
      </ScrollArea>

      {/* 下载对话框 */}
      <Dialog open={downloadDialog.open} onOpenChange={(open) => setDownloadDialog({ open, result: open ? downloadDialog.result : null })}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>选择下载方式</DialogTitle>
            <DialogDescription className="truncate">
              {downloadDialog.result?.title}
            </DialogDescription>
          </DialogHeader>
          
          <div className="space-y-3 py-4">
            {downloadDialog.result?.magnetUrl && (
              <div className="rounded-lg border border-border/50 p-3">
                <div className="mb-2 flex items-center gap-2 text-sm font-medium">
                  <Magnet className="h-4 w-4 text-orange-500" />
                  磁力链接
                </div>
                <div className="space-y-2">
                  {downloaders.filter(d => d.supports.includes("magnet") && d.status === "connected").map(d => (
                    <Button 
                      key={d.id}
                      variant="outline" 
                      className="w-full justify-start gap-2"
                      onClick={() => confirmDownload(d.id)}
                    >
                      {d.name}
                      {d.isDefault && <Badge variant="secondary" className="ml-auto text-[10px]">默认</Badge>}
                    </Button>
                  ))}
                </div>
              </div>
            )}
            
            {downloadDialog.result?.downloadUrl && (
              <div className="rounded-lg border border-border/50 p-3">
                <div className="mb-2 flex items-center gap-2 text-sm font-medium">
                  <Link2 className="h-4 w-4 text-blue-500" />
                  直接下载
                </div>
                <div className="space-y-2">
                  {downloaders.filter(d => d.supports.includes("direct") && d.status === "connected").map(d => (
                    <Button 
                      key={d.id}
                      variant="outline" 
                      className="w-full justify-start gap-2"
                      onClick={() => confirmDownload(d.id)}
                    >
                      {d.name}
                      {d.isDefault && <Badge variant="secondary" className="ml-auto text-[10px]">默认</Badge>}
                    </Button>
                  ))}
                </div>
              </div>
            )}
          </div>

          <DialogFooter>
            <Button variant="ghost" onClick={() => setDownloadDialog({ open: false, result: null })}>
              取消
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 搜索源配置对话框 */}
      <Dialog open={sourceConfigDialog} onOpenChange={setSourceConfigDialog}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>搜索源配置</DialogTitle>
            <DialogDescription>
              配置本地和在线搜索源，支持 Jackett、Prowlarr、Alist 等
            </DialogDescription>
          </DialogHeader>
          
          <ScrollArea className="max-h-[60vh]">
            <div className="space-y-4 py-4">
              {sources.map(source => (
                <div key={source.id} className="rounded-lg border border-border/50 p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-3">
                      <div className={cn(
                        "flex h-10 w-10 items-center justify-center rounded-lg",
                        source.type === "local" ? "bg-cyan-500/10 text-cyan-500" :
                        source.type === "indexer" ? "bg-orange-500/10 text-orange-500" :
                        "bg-blue-500/10 text-blue-500"
                      )}>
                        {source.type === "local" ? <HardDrive className="h-5 w-5" /> :
                         source.type === "indexer" ? <Plug className="h-5 w-5" /> :
                         <FolderDown className="h-5 w-5" />}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium">{source.name}</span>
                          {source.status === "connected" && (
                            <Badge variant="outline" className="border-green-500/50 text-green-500 text-[10px]">已连接</Badge>
                          )}
                          {source.status === "error" && (
                            <Badge variant="outline" className="border-red-500/50 text-red-500 text-[10px]">错误</Badge>
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground">
                          {source.type === "local" && "本地媒体库搜索"}
                          {source.type === "indexer" && "种子索引聚合器"}
                          {source.type === "alist" && "网盘聚合工具"}
                        </p>
                      </div>
                    </div>
                    <Switch 
                      checked={source.enabled}
                      onCheckedChange={(checked) => {
                        setSources(prev => prev.map(s => 
                          s.id === source.id ? { ...s, enabled: checked } : s
                        ))
                      }}
                    />
                  </div>
                  
                  {/* 配置表单 - 非本地源 */}
                  {source.type !== "local" && source.enabled && (
                    <div className="mt-4 space-y-3 border-t border-border/50 pt-4">
                      <div className="space-y-2">
                        <Label className="text-xs">API 地址</Label>
                        <Input 
                          placeholder={`例如: http://localhost:${source.type === "indexer" ? "9117" : "5244"}`}
                          value={source.url || ""}
                          onChange={(e) => {
                            setSources(prev => prev.map(s => 
                              s.id === source.id ? { ...s, url: e.target.value } : s
                            ))
                          }}
                          className="h-9 text-sm"
                        />
                      </div>
                      {source.type === "indexer" && (
                        <div className="space-y-2">
                          <Label className="text-xs">API Key</Label>
                          <Input 
                            type="password"
                            placeholder="输入 API Key"
                            value={source.apiKey || ""}
                            onChange={(e) => {
                              setSources(prev => prev.map(s => 
                                s.id === source.id ? { ...s, apiKey: e.target.value } : s
                              ))
                            }}
                            className="h-9 text-sm"
                          />
                        </div>
                      )}
                      <Button 
                        variant="outline" 
                        size="sm" 
                        className="gap-2"
                        onClick={() => {
                          // 模拟测试连接
                          setSources(prev => prev.map(s => 
                            s.id === source.id ? { ...s, status: "testing" as const } : s
                          ))
                          setTimeout(() => {
                            setSources(prev => prev.map(s => 
                              s.id === source.id ? { ...s, status: source.url ? "connected" : "error" } : s
                            ))
                          }, 1000)
                        }}
                      >
                        <RefreshCw className="h-3.5 w-3.5" />
                        测试连接
                      </Button>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </ScrollArea>

          <DialogFooter>
            <Button variant="outline" onClick={() => setSourceConfigDialog(false)}>关闭</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}

// 结果卡片组件
function ResultCard({ result, onDownload }: { result: SearchResult; onDownload: () => void }) {
  const isLocal = result.isLocal

  return (
    <div className={cn(
      "group flex items-center gap-4 rounded-xl border p-4 transition-all",
      isLocal 
        ? "border-cyan-500/20 bg-cyan-500/5 hover:border-cyan-500/40" 
        : "border-border/50 bg-card/30 hover:border-border"
    )}>
      {/* 海报/图标 */}
      <div className="relative h-16 w-12 shrink-0 overflow-hidden rounded-lg bg-muted">
        {result.poster ? (
          <img src={result.poster} alt={result.title} className="h-full w-full object-cover" />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            {categoryIcons[result.type]}
          </div>
        )}
        {isLocal && (
          <div className="absolute -right-1 -top-1 flex h-5 w-5 items-center justify-center rounded-full bg-cyan-500">
            <HardDrive className="h-3 w-3 text-white" />
          </div>
        )}
      </div>

      {/* 信息 */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <h4 className="font-medium truncate">{result.title}</h4>
          {result.year && <span className="shrink-0 text-sm text-muted-foreground">{result.year}</span>}
        </div>
        
        <div className="mt-1 flex flex-wrap items-center gap-2 text-xs">
          {result.rating && (
            <span className="flex items-center gap-1 text-amber-500">
              <Star className="h-3 w-3 fill-current" />
              {result.rating}
            </span>
          )}
          {result.quality && (
            <Badge variant="secondary" className="text-[10px]">{result.quality}</Badge>
          )}
          {result.size && (
            <span className="text-muted-foreground">{result.size}</span>
          )}
          {result.seeders !== undefined && (
            <span className="flex items-center gap-1 text-green-500">
              <span className="h-1.5 w-1.5 rounded-full bg-current" />
              {result.seeders}
            </span>
          )}
          {result.source && (
            <Badge variant="outline" className="text-[10px]">{result.source}</Badge>
          )}
        </div>
      </div>

      {/* 操作 */}
      <div className="flex shrink-0 items-center gap-2">
        {isLocal ? (
          <Button size="sm" className="gap-1.5 bg-cyan-500 hover:bg-cyan-600" onClick={onDownload}>
            <Play className="h-3.5 w-3.5" />
            播放
          </Button>
        ) : (
          <Button size="sm" className="gap-1.5" onClick={onDownload}>
            <Download className="h-3.5 w-3.5" />
            下载
          </Button>
        )}
      </div>
    </div>
  )
}
