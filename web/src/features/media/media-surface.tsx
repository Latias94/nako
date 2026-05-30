"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useEffect, useState, useRef, forwardRef, useImperativeHandle, lazy } from "react"
import { Play, Clock, ChevronRight, ChevronLeft, Star, Calendar, Info, Film, Tv, User, Tag, Clapperboard, Building2, Menu, Search, X, Heart, Settings, Download, ListMusic, Bell, History, Image, Music, Mic, Bot, Workflow, LayoutGrid, Sparkles, MoreHorizontal, Pin, RefreshCw, FolderEdit, Trash2, Eye, EyeOff } from "lucide-react"
import { Button } from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible"
import { Skeleton } from "@/components/ui/skeleton"
import { cn } from "@/lib/utils"
import {
  heartbeatPublicPlaybackSession,
  useTrendingMedia,
  useCategoryMedia,
  useContinueWatchingMedia,
  useMediaDetails,
  usePlaybackPlan,
} from "@/lib/use-media"
import type { MediaItem } from "@/lib/media-types"
import type { LibraryBrowserRouteState } from "./library-browser"
import type { MyListRouteState } from "./my-list-page"
import { AddToPlaylistButton } from "./add-to-playlist-button"
import { ManagementContextLinks } from "./management-context-links"

const MediaDetail = lazy(() => import("./media-detail").then((module) => ({ default: module.MediaDetail })))
const VideoPlayer = lazy(() => import("./video-player").then((module) => ({ default: module.VideoPlayer })))
const ImageViewer = lazy(() => import("./image-viewer").then((module) => ({ default: module.ImageViewer })))
const SearchPage = lazy(() => import("./search-page").then((module) => ({ default: module.SearchPage })))
const UserSelectPage = lazy(() => import("@/src/features/account").then((module) => ({ default: module.UserSelectPage })))
const MyListPage = lazy(() => import("./my-list-page").then((module) => ({ default: module.MyListPage })))
const SettingsPage = lazy(() => import("@/src/features/settings").then((module) => ({ default: module.SettingsPage })))
const LibraryBrowser = lazy(() => import("./library-browser").then((module) => ({ default: module.LibraryBrowser })))
const ActivityHistory = lazy(() => import("./activity-history").then((module) => ({ default: module.ActivityHistory })))
const PersonDetail = lazy(() => import("./person-detail").then((module) => ({ default: module.PersonDetail })))
const FilterPage = lazy(() => import("./filter-page").then((module) => ({ default: module.FilterPage })))
const NotificationCenter = lazy(() =>
  import("@/src/features/notifications").then((module) => ({ default: module.NotificationCenter })),
)

// Ref 类型
export interface MediaSurfaceRef {
  openSearch: () => void
}

// 导航状态类型
type ViewState =
  | { type: "browse" }
  | { type: "detail"; mediaId: string; mediaType: "movie" | "series" }
  | { type: "person"; name: string; id?: string }
  | { type: "genre"; name: string; id?: string }
  | { type: "tag"; name: string; id?: string }
  | { type: "collection"; name: string; id?: string }
  | { type: "studio"; name: string; id?: string }
  | { type: "player"; mediaId: string; mediaType: "movie" | "series"; sourceId?: string }
  | { type: "images"; mediaTitle: string }
  | { type: "search"; query?: string }
  | { type: "user-select" }
  | ({ type: "my-list" } & MyListRouteState)
  | { type: "settings" }
  | { type: "library"; libraryId: string; state?: LibraryBrowserRouteState }
  | { type: "history" }
  | { type: "filter"; libraryId?: string }
  | { type: "downloads" }
  | { type: "notifications" }
  | { type: "photos" }
  | { type: "music" }
  | { type: "podcasts" }
  | { type: "agent" }
  | { type: "automations" }

type PlaybackMediaType = "movie" | "series" | "anime" | "music" | "photo"
type DeferredMediaFeature = "downloads" | "photos" | "music" | "podcasts" | "agent" | "automations"

const DEFERRED_MEDIA_FEATURES = {
  downloads: {
    title: "下载管理",
    description: "下载器集成会等传输任务、订阅和权限边界稳定后再回到 live product。",
    icon: Download,
  },
  photos: {
    title: "照片库",
    description: "照片不是 Nako 当前影视库核心域，后续应作为独立媒体类型重新设计。",
    icon: Image,
  },
  music: {
    title: "音乐库",
    description: "音乐需要独立的专辑、艺人、曲目和播放队列模型，不混入当前影视体验。",
    icon: Music,
  },
  podcasts: {
    title: "播客",
    description: "播客订阅、单集下载和收听进度属于后续产品域，当前不进入运行时包。",
    icon: Mic,
  },
  agent: {
    title: "AI 助手",
    description: "AI 助手需要插件权限、工具调用审计和模型配置后再接入主产品。",
    icon: Bot,
  },
  automations: {
    title: "自动化",
    description: "自动化编排会在 webhook、任务和插件能力稳定后作为独立管理面恢复。",
    icon: Workflow,
  },
} satisfies Record<
  DeferredMediaFeature,
  {
    title: string
    description: string
    icon: React.ComponentType<{ className?: string }>
  }
>

const DEFERRED_MEDIA_FEATURE_KEYS = new Set<ViewState["type"]>(Object.keys(DEFERRED_MEDIA_FEATURES) as DeferredMediaFeature[])

export type MediaSurfaceRouteView =
  | { type: "browse" }
  | { type: "detail"; mediaId: string; mediaType: "movie" | "series" }
  | { type: "search"; query?: string }
  | ({ type: "my-list" } & MyListRouteState)
  | { type: "library"; libraryId: string; state?: LibraryBrowserRouteState }

export interface MediaSurfaceProps {
  initialView?: MediaSurfaceRouteView
  routeKey?: string
  onRouteNavigate?: (view: MediaSurfaceRouteView) => void
}

const DEFAULT_MEDIA_VIEW: MediaSurfaceRouteView = { type: "browse" }

// 模拟相关作品数据
const relatedWorksData = {
  persons: {
    "丹尼斯·维伦纽瓦": {
      role: "导演",
      works: [
        { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const },
        { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie" as const },
        { id: "3", title: "银翼杀手 2049", year: 2017, rating: 8.0, type: "movie" as const },
        { id: "4", title: "降临", year: 2016, rating: 7.9, type: "movie" as const },
        { id: "5", title: "边境杀手", year: 2015, rating: 7.6, type: "movie" as const },
        { id: "6", title: "囚徒", year: 2013, rating: 8.1, type: "movie" as const },
      ]
    },
    "提莫西·查拉梅": {
      role: "演员",
      works: [
        { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const, character: "保罗·厄崔迪" },
        { id: "7", title: "旺卡", year: 2023, rating: 7.1, type: "movie" as const, character: "威利·旺卡" },
        { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie" as const, character: "保罗·厄崔迪" },
        { id: "8", title: "小妇人", year: 2019, rating: 7.8, type: "movie" as const, character: "劳里" },
        { id: "9", title: "请以你的名字呼唤我", year: 2017, rating: 7.9, type: "movie" as const, character: "艾力欧" },
      ]
    },
    "马修·麦康纳": {
      role: "演员",
      works: [
        { id: "10", title: "真探 第一季", year: 2014, rating: 9.0, type: "series" as const, character: "拉斯特·科尔" },
        { id: "11", title: "星际穿越", year: 2014, rating: 8.7, type: "movie" as const, character: "库珀" },
        { id: "12", title: "达拉斯买家俱乐部", year: 2013, rating: 8.0, type: "movie" as const, character: "罗恩·伍德鲁夫" },
      ]
    }
  },
  genres: {
    "科幻": [
      { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const },
      { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie" as const },
      { id: "3", title: "银翼杀手 2049", year: 2017, rating: 8.0, type: "movie" as const },
      { id: "11", title: "星际穿越", year: 2014, rating: 8.7, type: "movie" as const },
      { id: "4", title: "降临", year: 2016, rating: 7.9, type: "movie" as const },
      { id: "13", title: "黑客帝国", year: 1999, rating: 8.7, type: "movie" as const },
    ],
    "犯罪": [
      { id: "10", title: "真探", year: 2014, rating: 8.9, type: "series" as const },
      { id: "14", title: "绝命毒师", year: 2008, rating: 9.5, type: "series" as const },
      { id: "15", title: "黑道家族", year: 1999, rating: 9.2, type: "series" as const },
    ],
    "剧情": [
      { id: "16", title: "奥本海默", year: 2023, rating: 8.4, type: "movie" as const },
      { id: "17", title: "坠落的审判", year: 2023, rating: 7.8, type: "movie" as const },
    ]
  },
  tags: {
    "史诗": [
      { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const },
      { id: "18", title: "指环王三部曲", year: 2001, rating: 8.9, type: "movie" as const },
      { id: "19", title: "角斗士", year: 2000, rating: 8.5, type: "movie" as const },
    ],
    "单元剧": [
      { id: "10", title: "真探", year: 2014, rating: 8.9, type: "series" as const },
      { id: "20", title: "美国恐怖故事", year: 2011, rating: 8.0, type: "series" as const },
      { id: "21", title: "宿敌", year: 2017, rating: 8.4, type: "series" as const },
    ]
  },
  collections: {
    "沙丘系列": [
      { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie" as const },
      { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const },
    ]
  },
  studios: {
    "HBO": [
      { id: "10", title: "真探", year: 2014, rating: 8.9, type: "series" as const },
      { id: "22", title: "权力的游戏", year: 2011, rating: 9.2, type: "series" as const },
      { id: "23", title: "继承之战", year: 2018, rating: 8.9, type: "series" as const },
    ],
    "Legendary Pictures": [
      { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie" as const },
      { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie" as const },
      { id: "24", title: "哥斯拉大战金刚", year: 2021, rating: 6.0, type: "movie" as const },
    ]
  }
}

// LibraryItem component with dropdown state management
function LibraryItem({
  lib,
  isSelected,
  onSelect
}: {
  lib: { id: string; name: string; count: number; icon: React.ComponentType<{ className?: string }> }
  isSelected: boolean
  onSelect: () => void
}) {
  const [isDropdownOpen, setIsDropdownOpen] = useState(false)
  const IconComponent = lib.icon

  return (
    <div
      className={cn(
        "group flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm transition-colors",
        isSelected
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
      )}
    >
      <button
        onClick={onSelect}
        className="flex flex-1 items-center gap-2"
      >
        <IconComponent className="h-4 w-4" />
        <span>{lib.name}</span>
      </button>
      {/* 数字和设置按钮共享同一位置，悬停/打开菜单时切换 */}
      <div className="relative w-8 h-5 flex items-center justify-end">
        <span className={cn(
          "text-[10px] text-muted-foreground absolute right-0 transition-opacity",
          (isDropdownOpen) ? "opacity-0" : "group-hover:opacity-0"
        )}>
          {lib.count}
        </span>
        <DropdownMenu open={isDropdownOpen} onOpenChange={setIsDropdownOpen}>
          <DropdownMenuTrigger asChild>
            <button className={cn(
              "absolute right-0 rounded p-0.5 hover:bg-sidebar-accent transition-opacity",
              isDropdownOpen ? "opacity-100" : "opacity-0 group-hover:opacity-100"
            )}>
              <MoreHorizontal className="h-3.5 w-3.5 text-muted-foreground" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="right" align="start" sideOffset={8} className="w-40">
            <DropdownMenuItem className="gap-2 text-xs">
              <Pin className="h-3.5 w-3.5" />
              置顶媒体库
            </DropdownMenuItem>
            <DropdownMenuItem className="gap-2 text-xs">
              <RefreshCw className="h-3.5 w-3.5" />
              扫描媒体库
            </DropdownMenuItem>
            <DropdownMenuItem className="gap-2 text-xs">
              <Eye className="h-3.5 w-3.5" />
              显示/隐藏
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem className="gap-2 text-xs">
              <FolderEdit className="h-3.5 w-3.5" />
              编辑媒体库
            </DropdownMenuItem>
            <DropdownMenuItem className="gap-2 text-xs text-destructive">
              <Trash2 className="h-3.5 w-3.5" />
              删除媒体库
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  )
}

// Mock data for realistic media content
const libraries = [
  { id: "movies", name: "电影", count: 847, icon: Film },
  { id: "tvshows", name: "剧集", count: 156, icon: Tv },
  { id: "anime", name: "动画", count: 234, icon: Star },
  { id: "documentary", name: "纪录片", count: 89, icon: Film },
]

export const MediaSurface = forwardRef<MediaSurfaceRef, MediaSurfaceProps>(function MediaSurface(
  { initialView = DEFAULT_MEDIA_VIEW, routeKey = "browse", onRouteNavigate },
  ref,
) {
  const [selectedLibrary, setSelectedLibrary] = useState("全部")
  const [viewState, setViewState] = useState<ViewState>(initialView)
  const [navHistory, setNavHistory] = useState<ViewState[]>([]) // 导航历史堆栈
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false)
  const [currentMediaId, setCurrentMediaId] = useState<string>("1")
  const [currentMediaType, setCurrentMediaType] = useState<"movie" | "series" | "anime" | "music" | "photo">("movie")

  // 使用 TanStack Query 获取媒体数据
  const { data: trendingData, isLoading, error } = useTrendingMedia()
  const { categories, fallback } = useCategoryMedia()
  const continueWatching = useContinueWatchingMedia()

  useEffect(() => {
    setViewState(initialView)
    setNavHistory([])

    if (initialView.type === "detail") {
      setCurrentMediaId(initialView.mediaId)
      setCurrentMediaType(initialView.mediaType)
    }

    if (initialView.type === "library") {
      setSelectedLibrary(initialView.libraryId)
    }
  }, [routeKey])

  // 导航到新页面（推入历史）
  const navigateTo = (newState: ViewState) => {
    const routeTarget = mediaRouteTarget(newState)
    if (routeTarget && onRouteNavigate) {
      onRouteNavigate(routeTarget)
      return
    }

    setNavHistory(prev => [...prev, viewState])
    setViewState(newState)
  }

  // 暴露给父组件的方法
  useImperativeHandle(ref, () => ({
    openSearch: () => navigateTo({ type: "search" })
  }))

  // 返回上一页（弹出历史）
  const goBack = () => {
    if (navHistory.length > 0) {
      const prevState = navHistory[navHistory.length - 1]
      setNavHistory(prev => prev.slice(0, -1))
      setViewState(prevState)
    } else {
      if (onRouteNavigate && viewState.type !== "browse") {
        onRouteNavigate({ type: "browse" })
        return
      }

      setViewState({ type: "browse" })
    }
  }

  // 处理导航（从详情页点击标签/演员等）
  const handleNavigate = (type: "person" | "genre" | "tag" | "collection" | "studio", value: string, id?: string) => {
    navigateTo({ type, name: value, id })
  }

  // 处理播放
  const handlePlay = (mediaId = currentMediaId, sourceId?: string) => {
    setCurrentMediaId(mediaId)
    navigateTo({
      type: "player",
      mediaId,
      mediaType: currentMediaType === "series" || currentMediaType === "anime" ? "series" : "movie",
      sourceId,
    })
  }

  // 处理查看图片
  const handleViewImages = (mediaTitle: string) => {
    navigateTo({ type: "images", mediaTitle })
  }

  const openHistoryItem = (mediaId: string, mediaType: PlaybackMediaType) => {
    setCurrentMediaId(mediaId)
    setCurrentMediaType(mediaType)

    if (mediaType === "music") {
      navigateTo({ type: "music" })
      return
    }

    if (mediaType === "photo") {
      navigateTo({ type: "photos" })
      return
    }

    navigateTo({
      type: "detail",
      mediaId,
      mediaType: mediaType === "series" || mediaType === "anime" ? "series" : "movie",
    })
  }

  // 视频播放器
  if (viewState.type === "player") {
    return (
      <MediaPlayerRoute
        viewState={viewState}
        onBack={goBack}
      />
    )
  }

  // 图片查看器
  if (viewState.type === "images") {
    return (
      <ImageViewer
        images={[
          { id: "1", url: null, type: "backdrop", resolution: "3840x2160" },
          { id: "2", url: null, type: "backdrop", resolution: "1920x1080" },
          { id: "3", url: null, type: "poster", resolution: "2000x3000" },
          { id: "4", url: null, type: "still", resolution: "1920x1080" },
          { id: "5", url: null, type: "still", resolution: "1920x1080" },
        ]}
        onClose={goBack}
        mediaTitle={viewState.mediaTitle}
      />
    )
  }

  // 用户选择页面
  if (viewState.type === "user-select") {
    return (
      <UserSelectPage
        onSelectUser={() => setViewState({ type: "browse" })}
      />
    )
  }

  // 搜索页面
  if (viewState.type === "search") {
    return (
      <SearchPage
        onBack={goBack}
        initialQuery={viewState.query}
        onQueryCommit={(query) => {
          onRouteNavigate?.({ type: "search", query })
        }}
      />
    )
  }

  // 我的列表页面
  if (viewState.type === "my-list") {
    return (
      <MyListPage
        onBack={goBack}
        playlistId={viewState.playlistId}
        viewMode={viewState.viewMode}
        onRouteStateChange={(state) => {
          if (onRouteNavigate) {
            onRouteNavigate({ type: "my-list", ...state })
            return
          }

          setViewState({ type: "my-list", ...state })
        }}
        onSelectMedia={(id, type) => {
          setCurrentMediaId(id)
          setCurrentMediaType(type)
          navigateTo({ type: "detail", mediaId: id, mediaType: type })
        }}
      />
    )
  }

  // 设置页面
  if (viewState.type === "settings") {
  return (
  <SettingsPage onBack={goBack} />
  )
  }

  // 媒体库浏览页面
  if (viewState.type === "library") {
  return (
  <LibraryBrowser
  libraryId={viewState.libraryId}
  onBack={goBack}
  onSelectMedia={(mediaId) => {
  setCurrentMediaId(mediaId)
  navigateTo({ type: "detail", mediaId, mediaType: "movie" })
  }}
  onEditMedia={(mediaId) => {
  // 可以打开编辑器
  }}
  onSearch={() => navigateTo({ type: "search" })}
  routeState={viewState.state}
  onRouteStateChange={(state) => {
  onRouteNavigate?.({ type: "library", libraryId: viewState.libraryId, state })
  }}
  isAdmin={true}
  />
  )
  }

  // 播放历史页面
  if (viewState.type === "history") {
    return (
      <ActivityHistory
        onBack={goBack}
        onSelectMedia={(mediaId, mediaType) => {
          openHistoryItem(mediaId, mediaType)
        }}
        onContinueWatch={(item) => {
          setCurrentMediaId(item.mediaId)
          setCurrentMediaType(item.mediaType)
          navigateTo({
            type: "player",
            mediaId: item.mediaId,
            mediaType: item.mediaType === "series" || item.mediaType === "anime" ? "series" : "movie",
          })
        }}
      />
    )
  }

  // 筛选页面
  if (viewState.type === "filter") {
    return (
      <FilterPage
        onBack={goBack}
        onApplyFilters={(filters) => {
          goBack()
        }}
      />
    )
  }

  // 通知中心
  if (viewState.type === "notifications") {
    return (
      <NotificationCenter
        onBack={goBack}
      />
    )
  }

  // 人物详情页面
  if (viewState.type === "person") {
    return (
      <PersonDetail
        personName={viewState.name}
        personId={viewState.id ?? ""}
        onBack={goBack}
        onSelectMedia={(mediaId, mediaType) => {
          setCurrentMediaId(mediaId)
          setCurrentMediaType(mediaType)
          navigateTo({ type: "detail", mediaId, mediaType })
        }}
      />
    )
  }

  if (isDeferredMediaFeature(viewState.type)) {
    return <DeferredMediaFeaturePanel feature={viewState.type} onBack={goBack} />
  }

  // 显示详情页
  if (viewState.type === "detail") {
    return (
      <MediaDetailRoute
        viewState={viewState}
        onBack={goBack}
        onNavigate={handleNavigate}
        onPlay={handlePlay}
        onViewImages={handleViewImages}
      />
    )
  }

  // 显示相关作品页（按标签等筛选）- person 现在由 PersonDetail 处理
  if (viewState.type === "genre" || viewState.type === "tag" || viewState.type === "collection" || viewState.type === "studio") {
    return (
      <RelatedWorksView
        viewState={viewState}
        onBack={goBack}
        onSelectWork={(id, type) => {
          setCurrentMediaId(id)
          setCurrentMediaType(type)
          navigateTo({ type: "detail", mediaId: id, mediaType: type })
        }}
        onNavigate={handleNavigate}
      />
    )
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)]">
      {/* Mobile Header */}
      <div className="fixed inset-x-0 top-14 z-30 flex h-12 items-center justify-between border-b border-border/50 bg-background/95 px-4 backdrop-blur supports-[backdrop-filter]:bg-background/60 lg:hidden">
        <Sheet open={mobileMenuOpen} onOpenChange={setMobileMenuOpen}>
          <SheetTrigger asChild>
            <Button variant="ghost" size="icon" className="h-9 w-9">
              <Menu className="h-5 w-5" />
            </Button>
          </SheetTrigger>
          <SheetContent side="left" className="w-64 bg-sidebar p-4">
            <nav className="space-y-1">
              <h3 className="mb-3 px-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                媒体库
              </h3>
              <button
                onClick={() => {
                  setSelectedLibrary("全部")
                  setMobileMenuOpen(false)
                }}
                className={cn(
                  "flex w-full items-center justify-between rounded-md px-2 py-2.5 text-sm transition-colors",
                  selectedLibrary === "全部"
                    ? "bg-sidebar-accent text-sidebar-accent-foreground"
                    : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
                )}
              >
                <span>全部媒体</span>
                <span className="text-xs text-muted-foreground">1,371</span>
              </button>
{libraries.map((lib) => {
  const IconComponent = lib.icon
  return (
  <button
  key={lib.id}
  onClick={() => {
  setSelectedLibrary(lib.name)
  setMobileMenuOpen(false)
  // 根据类型跳转到不同页面
  if (lib.id === "photos") navigateTo({ type: "photos" })
  else if (lib.id === "music") navigateTo({ type: "music" })
  else if (lib.id === "podcasts") navigateTo({ type: "podcasts" })
  else navigateTo({ type: "library", libraryId: lib.id })
  }}
  className={cn(
  "flex w-full items-center justify-between rounded-md px-2 py-2.5 text-sm transition-colors",
  selectedLibrary === lib.name
  ? "bg-sidebar-accent text-sidebar-accent-foreground"
  : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
  )}
  >
  <span className="flex items-center gap-2">
  <IconComponent className="h-4 w-4" />
  <span>{lib.name}</span>
  </span>
  <span className="text-xs text-muted-foreground">{lib.count}</span>
  </button>
  )
  })}
              <div className="mt-6">
                <h3 className="mb-3 px-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                  快捷方式
                </h3>
                <button className="flex w-full items-center gap-2 rounded-md px-2 py-2.5 text-sm text-sidebar-foreground/80 hover:bg-sidebar-accent/50">
                  <Clock className="h-4 w-4" />
                  <span>继续观看</span>
                </button>
                <button className="flex w-full items-center gap-2 rounded-md px-2 py-2.5 text-sm text-sidebar-foreground/80 hover:bg-sidebar-accent/50">
                  <Star className="h-4 w-4" />
                  <span>我的收藏</span>
                </button>
              </div>
            </nav>
          </SheetContent>
        </Sheet>

        <span className="text-sm font-medium">{selectedLibrary}</span>

        <Button
          variant="ghost"
          size="icon"
          className="h-9 w-9"
            onClick={() => navigateTo({ type: "search" })}
          >
            <Search className="h-5 w-5" />
          </Button>
      </div>

      {/* Desktop Sidebar - Optimized */}
      <aside className="hidden w-52 flex-shrink-0 border-r border-border/50 bg-sidebar lg:flex lg:flex-col">
        {/* Scrollable Content */}
        <ScrollArea className="flex-1 px-2 py-3">
          {/* 媒体库 - 核心导航 */}
          <nav className="space-y-0.5">
            <button
              onClick={() => { setSelectedLibrary("全部"); navigateTo({ type: "browse" }); }}
              className={cn(
                "flex w-full items-center justify-between rounded-lg px-3 py-2 text-sm transition-colors",
                viewState.type === "browse" && selectedLibrary === "全部"
                  ? "bg-sidebar-accent text-sidebar-accent-foreground"
                  : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
              )}
            >
              <span className="flex items-center gap-2">
                <LayoutGrid className="h-4 w-4" />
                <span>全部媒体</span>
              </span>
            </button>
            {libraries.slice(0, 4).map((lib) => (
              <LibraryItem
                key={lib.id}
                lib={lib}
                isSelected={selectedLibrary === lib.name}
                onSelect={() => {
                  setSelectedLibrary(lib.name)
                  if (lib.id === "photos") navigateTo({ type: "photos" })
                  else if (lib.id === "music") navigateTo({ type: "music" })
                  else if (lib.id === "podcasts") navigateTo({ type: "podcasts" })
                  else navigateTo({ type: "library", libraryId: lib.id })
                }}
              />
            ))}
          </nav>

          {/* 更多媒体类型 - 可折叠 */}
          {libraries.length > 4 && (
            <Collapsible className="mt-1">
              <CollapsibleTrigger className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-xs text-muted-foreground hover:bg-sidebar-accent/30">
                <ChevronRight className="h-3 w-3 transition-transform [[data-state=open]_&]:rotate-90" />
                更多媒体
              </CollapsibleTrigger>
              <CollapsibleContent className="space-y-0.5">
                {libraries.slice(4).map((lib) => (
                  <LibraryItem
                    key={lib.id}
                    lib={lib}
                    isSelected={selectedLibrary === lib.name}
                    onSelect={() => {
                      setSelectedLibrary(lib.name)
                      if (lib.id === "photos") navigateTo({ type: "photos" })
                      else if (lib.id === "music") navigateTo({ type: "music" })
                      else if (lib.id === "podcasts") navigateTo({ type: "podcasts" })
                      else navigateTo({ type: "library", libraryId: lib.id })
                    }}
                  />
                ))}
              </CollapsibleContent>
            </Collapsible>
          )}

          {/* 分隔线 */}
          <div className="my-3 border-t border-border/30" />

          {/* 快捷方式 - 更紧凑 */}
          <nav className="space-y-0.5">
            <button
              onClick={() => navigateTo({ type: "history" })}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
            >
              <History className="h-4 w-4" />
              <span>播放历史</span>
            </button>
            <button
              onClick={() => navigateTo({ type: "my-list" })}
              className="flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
            >
              <ListMusic className="h-4 w-4" />
              <span>我的列表</span>
            </button>
          </nav>
        </ScrollArea>

        {/* 底部固定 */}
        <div className="border-t border-border/50 p-2">
          <div className="flex gap-1">
            <Button
              variant="ghost"
              size="sm"
              className="flex-1 justify-start gap-2 px-3 text-xs"
              onClick={() => navigateTo({ type: "settings" })}
            >
              <Settings className="h-3.5 w-3.5" />
              设置
            </Button>
            <Button
              variant="ghost"
              size="sm"
              className="flex-1 justify-start gap-2 px-3 text-xs"
              onClick={() => setViewState({ type: "user-select" })}
            >
              <User className="h-3.5 w-3.5" />
              用户
            </Button>
          </div>
        </div>
      </aside>

      {/* Main Content */}
          <main className="flex-1 overflow-y-auto pt-12 scrollbar-none lg:pt-0">
        <div className="p-4 lg:p-6 xl:p-8">
          {/* Loading State - Skeleton */}
          {isLoading && (
            <div className="space-y-8">
              {/* Continue Watching Skeleton */}
              <section>
                <div className="mb-4 flex items-center justify-between">
                  <Skeleton className="h-6 w-24" />
                  <Skeleton className="h-8 w-20" />
                </div>
                <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
                  {[1, 2, 3].map((i) => (
                    <div key={i} className="space-y-3">
                      <Skeleton className="aspect-video w-full rounded-lg" />
                      <div className="space-y-2">
                        <Skeleton className="h-4 w-3/4" />
                        <Skeleton className="h-3 w-1/2" />
                      </div>
                    </div>
                  ))}
                </div>
              </section>

              {/* Recently Added Skeleton */}
              <section>
                <div className="mb-4 flex items-center justify-between">
                  <Skeleton className="h-6 w-28" />
                  <Skeleton className="h-8 w-20" />
                </div>
                <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
                  {[1, 2, 3, 4, 5, 6, 7, 8].map((i) => (
                    <div key={i} className="space-y-2">
                      <Skeleton className="aspect-[2/3] w-full rounded-lg" />
                      <Skeleton className="h-3 w-full" />
                      <Skeleton className="h-3 w-2/3" />
                    </div>
                  ))}
                </div>
              </section>

              {/* Recommended Skeleton */}
              <section>
                <div className="mb-4 flex items-center justify-between">
                  <Skeleton className="h-6 w-32" />
                  <Skeleton className="h-8 w-20" />
                </div>
                <div className="grid grid-cols-3 gap-3 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 xl:grid-cols-8">
                  {[1, 2, 3, 4, 5, 6, 7, 8].map((i) => (
                    <div key={i} className="space-y-2">
                      <Skeleton className="aspect-[2/3] w-full rounded-lg" />
                      <Skeleton className="h-3 w-full" />
                      <Skeleton className="h-3 w-1/2" />
                    </div>
                  ))}
                </div>
              </section>
            </div>
          )}

          {/* Error State */}
          {error && !trendingData && (
            <div className="flex flex-col items-center justify-center py-12 text-center">
              <p className="text-muted-foreground">加载失败，请稍后重试</p>
            </div>
          )}

          {/* Continue Watching Section */}
          {continueWatching.data && continueWatching.data.items.length > 0 && (
            <section className="mb-8 lg:mb-10">
  <div className="mb-3 flex items-center justify-between lg:mb-4">
  <h2 className="text-base font-semibold text-foreground lg:text-lg">继续观看</h2>
  {continueWatching.data.fallback && (
  <Badge variant="secondary" className="mr-2 text-[10px]">演示数据</Badge>
  )}
  <Button variant="ghost" size="sm" className="h-8 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground lg:h-9 lg:text-sm" onClick={() => navigateTo({ type: "library", libraryId: "movies" })}>
  查看全部 <ChevronRight className="ml-1 h-4 w-4" />
  </Button>
  </div>
              <div className="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 lg:gap-4">
                {continueWatching.data.items.slice(0, 3).map(({ item, state }) => (
                  <ContinueWatchingCard
                    key={item.id}
                    item={{
                      id: item.id,
                      title: item.title,
                      originalTitle: item.originalTitle,
                      year: item.year,
                      progress: Math.round(state.progressPercent ?? 0),
                      duration: formatDurationMs(state.durationMs) ?? item.duration ?? "未知时长",
                      thumbnail: item.backdrop || item.poster,
                      type: item.type === "series" ? "剧集" : "电影",
                      episode: item.type === "series" ? "S01E03" : undefined,
                    }}
                    onClick={() => {
                      setCurrentMediaId(item.id)
                      setCurrentMediaType(item.type === "series" ? "series" : "movie")
                      navigateTo({ type: "detail", mediaId: item.id, mediaType: item.type === "series" ? "series" : "movie" })
                    }}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Recently Added Section - 使用 TMDb 数据 */}
          {trendingData && trendingData.items.length > 0 && (
            <section className="mb-8 lg:mb-10">
              <div className="mb-3 flex items-center justify-between lg:mb-4">
                <h2 className="text-base font-semibold text-foreground lg:text-lg">最近添加</h2>
                {fallback && (
                  <Badge variant="secondary" className="mr-2 text-[10px]">演示数据</Badge>
                )}
                <Button variant="ghost" size="sm" className="h-8 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground lg:h-9 lg:text-sm" onClick={() => navigateTo({ type: "library", libraryId: "movies" })}>
                  查看全部 <ChevronRight className="ml-1 h-4 w-4" />
                </Button>
              </div>
              <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 lg:gap-4 xl:grid-cols-6">
                {trendingData.items.slice(0, 6).map((item) => (
                  <MediaCard
                    key={item.id}
                    item={{
                      id: item.id,
                      title: item.title,
                      originalTitle: item.originalTitle,
                      year: item.year,
                      rating: item.rating,
                      poster: item.poster,
                      type: item.type === "series" ? "剧集" : "电影",
                      quality: "1080p",
                    }}
                    onClick={() => {
                      setCurrentMediaId(item.id)
                      setCurrentMediaType(item.type === "series" ? "series" : "movie")
                      navigateTo({ type: "detail", mediaId: item.id, mediaType: item.type === "series" ? "series" : "movie" })
                    }}
                  />
                ))}
              </div>
            </section>
          )}

          {/* Netflix 风格横向滚动推荐列表 - 使用动态分类数据 */}
          {categories.map((row) => (
            <HorizontalScrollRow
              key={row.title}
              title={row.title}
              items={row.items}
              onSelectItem={(id, type) => {
                setCurrentMediaId(id)
                setCurrentMediaType(type === "series" ? "series" : "movie")
                navigateTo({
                  type: "detail",
                  mediaId: id,
                  mediaType: type === "series" ? "series" : "movie"
                })
              }}
            />
          ))}

          {/* Library Access Notice */}
          <section className="rounded-lg border border-border/50 bg-card p-3 lg:p-4">
            <div className="flex items-start gap-3">
              <div className="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full bg-info/10">
                <Info className="h-4 w-4 text-info" />
              </div>
              <div>
                <h3 className="text-sm font-medium text-foreground">Library Access</h3>
                <p className="mt-1 text-xs text-muted-foreground lg:text-sm">
                  你当前拥有对 5 个媒体库的完整访问权限。播放来源将根据你的设备能力自动选择最佳版本。
                </p>
              </div>
            </div>
          </section>
        </div>
      </main>
    </div>
  )
})

function mediaRouteTarget(view: ViewState): MediaSurfaceRouteView | null {
  switch (view.type) {
    case "browse":
    case "search":
    case "detail":
    case "my-list":
    case "library":
      return view
    default:
      return null
  }
}

function MediaDetailRoute({
  viewState,
  onBack,
  onNavigate,
  onPlay,
  onViewImages,
}: {
  viewState: Extract<ViewState, { type: "detail" }>
  onBack: () => void
  onNavigate: (type: "person" | "genre" | "tag" | "collection" | "studio", value: string, id?: string) => void
  onPlay: (mediaId?: string, sourceId?: string) => void
  onViewImages: (mediaTitle: string) => void
}) {
  const detail = useMediaDetails(viewState.mediaId, viewState.mediaType)
  const media = detail.data?.item ?? null

  return (
    <MediaDetail
      onBack={onBack}
      onNavigate={onNavigate}
      onPlay={(mediaId, sourceId) => onPlay(mediaId, sourceId)}
      onViewImages={() => onViewImages(media?.title ?? (viewState.mediaType === "movie" ? "沙丘2" : "真探"))}
      mediaId={viewState.mediaId}
      mediaType={viewState.mediaType}
      media={media}
      sources={detail.data?.sources ?? []}
      readiness={detail.data?.readiness ?? []}
      fallback={detail.data?.fallback ?? false}
      isLoading={detail.isLoading}
      error={detail.error instanceof Error ? detail.error.message : undefined}
    />
  )
}

function formatDurationMs(durationMs: number | null) {
  if (!durationMs || durationMs <= 0) {
    return undefined
  }

  const totalMinutes = Math.round(durationMs / 60000)
  const hours = Math.floor(totalMinutes / 60)
  const minutes = totalMinutes % 60

  if (hours <= 0) {
    return `${minutes}m`
  }

  return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`
}

function MediaPlayerRoute({
  viewState,
  onBack,
}: {
  viewState: Extract<ViewState, { type: "player" }>
  onBack: () => void
}) {
  const playbackPlan = usePlaybackPlan(viewState.mediaId, viewState.mediaType, viewState.sourceId)
  const shouldUsePlaybackPlan =
    playbackPlan.data !== undefined && (!playbackPlan.data.fallback || playbackPlan.data.error)
  const liveSources =
    shouldUsePlaybackPlan
      ? playbackPlan.data.mediaUrl
        ? [
            {
              quality: playbackPlan.data.mode?.toUpperCase() ?? "Auto",
              url: playbackPlan.data.mediaUrl,
              contentType: playbackPlan.data.mediaContentType,
            },
          ]
        : []
      : undefined
  const liveSubtitles =
    shouldUsePlaybackPlan
      ? playbackPlan.data.subtitles.map((subtitle) => ({
          id: subtitle.id,
          language: subtitle.language,
          url: subtitle.url,
          srcLang: subtitle.srcLang,
          default: subtitle.default,
          forced: subtitle.forced,
          contentType: subtitle.contentType,
        }))
      : undefined
  const playbackIssue =
    shouldUsePlaybackPlan &&
    (Boolean(playbackPlan.data.error) || !playbackPlan.data.mediaUrl)
  const diagnosticContext = {
    itemId: viewState.mediaId,
    mediaType: viewState.mediaType,
    sourceId: playbackPlan.data?.sourceId ?? viewState.sourceId,
    playbackSessionId: playbackPlan.data?.playbackSessionId,
  }

  return (
    <VideoPlayer
      onBack={onBack}
      mediaTitle={viewState.mediaType === "movie" ? "沙丘2" : "真探"}
      episodeInfo={viewState.mediaType === "series" ? "S01E04" : undefined}
      episodeTitle={viewState.mediaType === "series" ? "什么是谁" : undefined}
      hasNext={viewState.mediaType === "series"}
      hasPrevious={viewState.mediaType === "series"}
      sources={liveSources}
      subtitles={liveSubtitles}
      playbackSessionId={shouldUsePlaybackPlan ? playbackPlan.data.playbackSessionId : undefined}
      onPlaybackHeartbeat={heartbeatPublicPlaybackSession}
      diagnosticActions={
        playbackIssue ? (
          <ManagementContextLinks
            context={diagnosticContext}
            routeNames={["playback.support", "playback.runtime", "jobs.filtered"]}
            tone="hero"
          />
        ) : undefined
      }
    />
  )
}

function isDeferredMediaFeature(value: ViewState["type"]): value is DeferredMediaFeature {
  return DEFERRED_MEDIA_FEATURE_KEYS.has(value)
}

function DeferredMediaFeaturePanel({ feature, onBack }: { feature: DeferredMediaFeature; onBack: () => void }) {
  const metadata = DEFERRED_MEDIA_FEATURES[feature]
  const Icon = metadata.icon

  return (
    <div className="grid min-h-[calc(100vh-3.5rem)] place-items-center bg-background p-6">
      <div className="w-full max-w-xl rounded-lg border border-border bg-card p-6 shadow-sm">
        <div className="flex items-start gap-4">
          <div className="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-lg bg-muted text-muted-foreground">
            <Icon className="h-5 w-5" />
          </div>
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-2">
              <h1 className="text-lg font-semibold text-foreground">{metadata.title}</h1>
              <Badge variant="secondary">Deferred</Badge>
            </div>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">{metadata.description}</p>
            <Button variant="outline" size="sm" className="mt-5 gap-2" onClick={onBack}>
              <ChevronLeft className="h-4 w-4" />
              返回媒体库
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}

function ContinueWatchingCard({
  item,
  onClick,
}: {
  item: {
    id: string | number
    title: string
    originalTitle: string
    year: number
    progress: number
    duration: string
    thumbnail: string
    type: string
    episode?: string
  }
  onClick?: () => void
}) {
  return (
    <div
      className="group relative cursor-pointer overflow-hidden rounded-lg border border-border/50 bg-card transition-all hover:border-border hover:shadow-lg"
      onClick={onClick}
    >
      {/* Thumbnail */}
      <div className="relative aspect-video overflow-hidden">
        <img
          src={resolveArtwork(item.thumbnail)}
          alt={item.title}
          className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
        />
        <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent" />
        <div
          className="absolute left-1/2 top-1/2 flex h-10 w-10 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full bg-primary/90 text-primary-foreground opacity-0 shadow-lg transition-opacity group-hover:opacity-100 lg:h-12 lg:w-12"
        >
          <Play className="h-4 w-4 lg:h-5 lg:w-5" />
        </div>
        {/* Progress bar */}
        <div className="absolute inset-x-0 bottom-0">
          <div className="h-1 w-full bg-white/20">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${item.progress}%` }}
            />
          </div>
        </div>
      </div>
      {/* Info */}
      <div className="p-2.5 lg:p-3">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <h3 className="truncate text-sm font-medium text-foreground">{item.title}</h3>
            <p className="truncate text-xs text-muted-foreground">{item.originalTitle}</p>
          </div>
          <Badge variant="secondary" className="flex-shrink-0 text-[10px]">
            {item.type}
          </Badge>
        </div>
        <div className="mt-1.5 flex flex-wrap items-center gap-1.5 text-xs text-muted-foreground lg:mt-2 lg:gap-2">
          <span>{item.year}</span>
          <span>·</span>
          {"episode" in item && <span>{item.episode}</span>}
          {"episode" in item && <span>·</span>}
          <span>{item.duration}</span>
          <span>·</span>
          <span>{item.progress}% 已观看</span>
        </div>
      </div>
    </div>
  )
}

function MediaCard({
  item,
  onClick,
}: {
  item: {
    id: string | number
    title: string
    originalTitle: string
    year: number
    rating: number
    poster: string
    type: string
    quality: string
    episodes?: string | number
  }
  onClick?: () => void
}) {
  return (
    <div className="group/card relative">
      <button
        type="button"
        className="block w-full cursor-pointer text-left outline-none focus-visible:ring-2 focus-visible:ring-primary"
        onClick={onClick}
      >
        {/* Poster */}
        <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-transform group-hover/card:scale-[1.02]">
          <img
            src={resolveArtwork(item.poster)}
            alt={item.title}
            className="h-full w-full object-cover"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-black/60 via-transparent to-transparent opacity-0 transition-opacity group-hover/card:opacity-100" />
          <div
            className="absolute left-1/2 top-1/2 flex h-9 w-9 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full bg-primary/90 text-primary-foreground opacity-0 shadow-lg transition-opacity group-hover/card:opacity-100 lg:h-10 lg:w-10"
          >
            <Play className="h-4 w-4" />
          </div>
          {/* Quality Badge */}
          <Badge className="absolute right-1.5 top-1.5 bg-black/70 text-[9px] text-white backdrop-blur-sm lg:right-2 lg:top-2 lg:text-[10px]">
            {item.quality}
          </Badge>
          {/* Type Badge for series */}
          {item.type === "剧集" && "episodes" in item && (
            <Badge variant="secondary" className="absolute bottom-1.5 left-1.5 bg-black/70 text-[9px] text-white lg:bottom-2 lg:left-2 lg:text-[10px]">
              {item.episodes} 集
            </Badge>
          )}
        </div>
        {/* Info */}
        <div className="mt-1.5 lg:mt-2">
          <h3 className="truncate text-xs font-medium text-foreground lg:text-sm">{item.title}</h3>
          <div className="mt-0.5 flex items-center gap-1 text-[10px] text-muted-foreground lg:text-xs">
            <span>{item.year}</span>
            <span>·</span>
            <Star className="h-2.5 w-2.5 fill-accent text-accent lg:h-3 lg:w-3" />
            <span>{item.rating}</span>
          </div>
        </div>
      </button>
      <AddToPlaylistButton
        itemId={String(item.id)}
        itemTitle={item.title}
        variant="icon"
        className="absolute left-1.5 top-1.5 z-10 lg:left-2 lg:top-2"
        triggerClassName="border border-white/20"
      />
    </div>
  )
}

// Netflix 风格横向滚动列表
function HorizontalScrollRow({
  title,
  items,
  onSelectItem
}: {
  title: string
  items: MediaItem[]
  onSelectItem: (id: string, type: string) => void
}) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const [showLeftArrow, setShowLeftArrow] = useState(false)
  const [showRightArrow, setShowRightArrow] = useState(true)

  const checkScrollPosition = () => {
    if (scrollRef.current) {
      const { scrollLeft, scrollWidth, clientWidth } = scrollRef.current
      setShowLeftArrow(scrollLeft > 0)
      setShowRightArrow(scrollLeft < scrollWidth - clientWidth - 10)
    }
  }

  const scroll = (direction: "left" | "right") => {
    if (scrollRef.current) {
      const scrollAmount = scrollRef.current.clientWidth * 0.8
      scrollRef.current.scrollBy({
        left: direction === "left" ? -scrollAmount : scrollAmount,
        behavior: "smooth"
      })
    }
  }

  return (
    <section className="group/row relative mb-6 lg:mb-8">
      <div className="mb-2 flex items-center justify-between lg:mb-3">
        <h2 className="text-base font-semibold text-foreground lg:text-lg">{title}</h2>
                <Button variant="ghost" size="sm" className="h-8 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground">
                  查看全部 <ChevronRight className="ml-1 h-4 w-4" />
                </Button>
      </div>

      <div className="relative -mx-4 lg:-mx-6">
        {/* 左箭头 */}
        {showLeftArrow && (
          <button
            onClick={() => scroll("left")}
            className="absolute -left-2 top-0 z-20 hidden h-[calc(100%-2rem)] items-center bg-gradient-to-r from-background via-background/80 to-transparent px-4 opacity-0 transition-opacity group-hover/row:opacity-100 lg:flex"
          >
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary/80 shadow-lg backdrop-blur transition-transform hover:scale-110">
              <ChevronLeft className="h-5 w-5" />
            </div>
          </button>
        )}

        {/* 滚动容器 */}
        <div
          ref={scrollRef}
          onScroll={checkScrollPosition}
              className="flex gap-3 overflow-x-auto px-4 pb-2 pt-1 scrollbar-none lg:gap-4 lg:px-6"
          style={{ scrollbarWidth: "none", msOverflowStyle: "none" }}
        >
          {items.map((item) => (
            <button
              key={item.id}
              onClick={() => onSelectItem(item.id, item.type)}
              className="group flex-shrink-0 text-left"
              style={{ width: "calc((100% - 2 * 0.75rem) / 3)", minWidth: "140px", maxWidth: "180px" }}
            >
              <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted shadow-md transition-all duration-200 group-hover:-translate-y-1 group-hover:shadow-xl group-hover:ring-2 group-hover:ring-primary">
                <img src={resolveArtwork(item.poster)} alt={item.title} className="h-full w-full object-cover" />
                <div className="absolute inset-0 bg-black/50 opacity-0 transition-opacity group-hover:opacity-100" />
                <div className="absolute inset-0 flex items-center justify-center opacity-0 transition-opacity group-hover:opacity-100">
                  <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg">
                    <Play className="h-4 w-4" />
                  </div>
                </div>
              </div>
              <h3 className="mt-1.5 truncate text-xs font-medium lg:text-sm">{item.title}</h3>
              <div className="flex items-center gap-1 text-[10px] text-muted-foreground lg:text-xs">
                <span>{item.year}</span>
                <span>·</span>
                <Star className="h-2.5 w-2.5 fill-accent text-accent" />
                <span>{item.rating}</span>
              </div>
            </button>
          ))}
        </div>

        {/* 右箭头 */}
        {showRightArrow && (
          <button
            onClick={() => scroll("right")}
            className="absolute -right-2 top-0 z-20 hidden h-[calc(100%-2rem)] items-center bg-gradient-to-l from-background via-background/80 to-transparent px-4 opacity-0 transition-opacity group-hover/row:opacity-100 lg:flex"
          >
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary/80 shadow-lg backdrop-blur transition-transform hover:scale-110">
              <ChevronRight className="h-5 w-5" />
            </div>
          </button>
        )}
      </div>
    </section>
  )
}

// 相关作品浏览视图
function RelatedWorksView({
  viewState,
  onBack,
  onSelectWork,
  onNavigate
}: {
  viewState: Exclude<ViewState, { type: "browse" | "detail" }>
  onBack: () => void
  onSelectWork: (id: string, type: "movie" | "series") => void
  onNavigate: (type: "person" | "genre" | "tag" | "collection" | "studio", value: string, id?: string) => void
}) {
  // 获取标题和图标
  const getHeaderInfo = () => {
    switch (viewState.type) {
      case "person":
        const personData = relatedWorksData.persons[viewState.name as keyof typeof relatedWorksData.persons]
        return {
          title: viewState.name,
          subtitle: personData?.role || "演员/导演",
          icon: <User className="h-5 w-5" />
        }
      case "genre":
        return {
          title: viewState.name,
          subtitle: "类型",
          icon: <Clapperboard className="h-5 w-5" />
        }
      case "tag":
        return {
          title: viewState.name,
          subtitle: "标签",
          icon: <Tag className="h-5 w-5" />
        }
      case "collection":
        return {
          title: viewState.name,
          subtitle: "系列",
          icon: <Film className="h-5 w-5" />
        }
      case "studio":
        return {
          title: viewState.name,
          subtitle: "制片公司/电视网",
          icon: <Building2 className="h-5 w-5" />
        }
      default:
        return { title: "相关作品", subtitle: "", icon: null }
    }
  }

  // 获取作品列表
  const getWorks = () => {
    switch (viewState.type) {
      case "person":
        return relatedWorksData.persons[viewState.name as keyof typeof relatedWorksData.persons]?.works || []
      case "genre":
        return relatedWorksData.genres[viewState.name as keyof typeof relatedWorksData.genres] || []
      case "tag":
        return relatedWorksData.tags[viewState.name as keyof typeof relatedWorksData.tags] || []
      case "collection":
        return relatedWorksData.collections[viewState.name as keyof typeof relatedWorksData.collections] || []
      case "studio":
        return relatedWorksData.studios[viewState.name as keyof typeof relatedWorksData.studios] || []
      default:
        return []
    }
  }

  const { title, subtitle, icon } = getHeaderInfo()
  const works = getWorks()

  return (
    <div className="min-h-[calc(100vh-3.5rem)] bg-background">
      {/* 头部 */}
      <div className="border-b border-border/50">
        <div className="mx-auto max-w-6xl px-6 py-6 lg:px-8">
          <div className="flex items-center gap-4">
            <Button
              variant="ghost"
              size="sm"
              onClick={onBack}
              className="gap-2"
            >
              <ChevronLeft className="h-4 w-4" />
              返回
            </Button>
            <div className="flex items-center gap-3">
              {icon && (
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
                  {icon}
                </div>
              )}
              <div>
                <p className="text-xs text-muted-foreground">{subtitle}</p>
                <h1 className="text-xl font-bold lg:text-2xl">{title}</h1>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 作品列表 */}
      <div className="mx-auto max-w-6xl px-6 py-8 lg:px-8">
        {works.length > 0 ? (
          <>
            <p className="mb-6 text-sm text-muted-foreground">{works.length} 部作品</p>
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
              {works.map((work) => (
                <div
                  key={work.id}
                  className="group cursor-pointer"
                  onClick={() => onSelectWork(work.id, work.type)}
                >
                  <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-transform group-hover:scale-[1.02]">
                    <div className="absolute inset-0 flex items-center justify-center text-muted-foreground">
                      {work.type === "series" ? <Tv className="h-10 w-10" /> : <Film className="h-10 w-10" />}
                    </div>
                    <div className="absolute inset-0 bg-gradient-to-t from-background/60 via-transparent to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
                    <Button
                      size="icon"
                      className="absolute left-1/2 top-1/2 h-10 w-10 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/90 text-primary-foreground opacity-0 shadow-lg transition-opacity group-hover:opacity-100"
                    >
                      <Play className="h-4 w-4" />
                    </Button>
                    {/* 类型标签 */}
                    <Badge
                      variant="secondary"
                      className="absolute right-2 top-2 text-[10px]"
                    >
                      {work.type === "series" ? "剧集" : "电影"}
                    </Badge>
                  </div>
                  <div className="mt-2">
                    <h3 className="truncate text-sm font-medium text-foreground group-hover:text-primary transition-colors">
                      {work.title}
                    </h3>
                    <div className="mt-0.5 flex items-center gap-1 text-xs text-muted-foreground">
                      <span>{work.year}</span>
                      <span>·</span>
                      <Star className="h-3 w-3 fill-warning text-warning" />
                      <span>{work.rating}</span>
                    </div>
                    {"character" in work && work.character && (
                      <p className="mt-1 truncate text-xs text-muted-foreground">饰 {work.character}</p>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </>
        ) : (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <Film className="mb-4 h-12 w-12 text-muted-foreground" />
            <p className="text-muted-foreground">暂无相关作品</p>
            <Button variant="outline" className="mt-4" onClick={onBack}>
              返回浏览
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
