"use client"

import { useState } from "react"
import { 
  Play, Trash2, Clock, Calendar, Search, Filter, ChevronLeft,
  MoreHorizontal, Film, Tv, Music, Image as ImageIcon, X, CheckCheck
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Progress } from "@/components/ui/progress"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { cn } from "@/lib/utils"

// 活动类型
type ActivityType = "watch" | "finish" | "add_list" | "remove_list" | "favorite" | "rate"

// 媒体类型
type MediaType = "movie" | "series" | "anime" | "music" | "photo"

// 历史记录项
interface HistoryItem {
  id: string
  mediaId: string
  mediaTitle: string
  mediaType: MediaType
  poster: string
  episodeInfo?: {
    season?: number
    episode?: number
    title?: string
  }
  activityType: ActivityType
  progress?: number // 0-100
  duration?: number // 总时长（分钟）
  watchedDuration?: number // 已观看时长（分钟）
  timestamp: Date
  device?: string
}

// 分组后的历史记录
interface GroupedHistory {
  date: string
  label: string
  items: HistoryItem[]
}

// Mock 数据生成
const generateMockHistory = (): HistoryItem[] => {
  const titles = [
    { title: "沙丘2", type: "movie" as MediaType },
    { title: "葬送的芙莉莲", type: "anime" as MediaType, episode: { season: 1, episode: 12, title: "勇者的剑" } },
    { title: "真探 第一季", type: "series" as MediaType, episode: { season: 1, episode: 5, title: "秘密的终结" } },
    { title: "奥本海默", type: "movie" as MediaType },
    { title: "咒术回战", type: "anime" as MediaType, episode: { season: 2, episode: 8, title: "�的谷事变" } },
    { title: "银翼杀手2049", type: "movie" as MediaType },
    { title: "星际穿越", type: "movie" as MediaType },
  ]
  
  const activities: ActivityType[] = ["watch", "finish", "watch", "watch", "add_list", "watch", "favorite"]
  
  const now = new Date()
  
  return Array.from({ length: 30 }, (_, i) => {
    const titleData = titles[i % titles.length]
    const hoursAgo = i * 3 + Math.floor(Math.random() * 3)
    const timestamp = new Date(now.getTime() - hoursAgo * 60 * 60 * 1000)
    const activity = activities[i % activities.length]
    const duration = 90 + Math.floor(Math.random() * 90)
    const watchedDuration = activity === "finish" ? duration : Math.floor(Math.random() * duration)
    
    return {
      id: `history-${i}`,
      mediaId: `media-${i % titles.length}`,
      mediaTitle: titleData.title,
      mediaType: titleData.type,
      poster: `https://image.tmdb.org/t/p/w200/8b8R8l88Qje9dn9OE8PY05Nxl1X.jpg`,
      episodeInfo: titleData.episode,
      activityType: activity,
      progress: activity === "finish" ? 100 : Math.floor((watchedDuration / duration) * 100),
      duration,
      watchedDuration,
      timestamp,
      device: ["网页", "iOS", "Android", "Apple TV"][i % 4],
    }
  })
}

// 按日期分组
function groupByDate(items: HistoryItem[]): GroupedHistory[] {
  const groups: Map<string, HistoryItem[]> = new Map()
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const yesterday = new Date(today.getTime() - 24 * 60 * 60 * 1000)
  
  for (const item of items) {
    const itemDate = new Date(item.timestamp)
    const itemDateOnly = new Date(itemDate.getFullYear(), itemDate.getMonth(), itemDate.getDate())
    
    let key: string
    let label: string
    
    if (itemDateOnly.getTime() === today.getTime()) {
      key = "today"
      label = "今天"
    } else if (itemDateOnly.getTime() === yesterday.getTime()) {
      key = "yesterday"
      label = "昨天"
    } else if (itemDateOnly.getTime() > today.getTime() - 7 * 24 * 60 * 60 * 1000) {
      const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"]
      key = `weekday-${itemDateOnly.getDay()}`
      label = weekdays[itemDateOnly.getDay()]
    } else {
      key = itemDateOnly.toISOString().split("T")[0]
      label = `${itemDateOnly.getMonth() + 1}月${itemDateOnly.getDate()}日`
    }
    
    if (!groups.has(key)) {
      groups.set(key, [])
    }
    groups.get(key)!.push(item)
  }
  
  return Array.from(groups.entries()).map(([date, items]) => ({
    date,
    label: items.length > 0 
      ? date === "today" ? "今天"
      : date === "yesterday" ? "昨天"
      : date.startsWith("weekday-") ? ["周日", "周一", "周二", "周三", "周四", "周五", "周六"][parseInt(date.split("-")[1])]
      : `${new Date(date).getMonth() + 1}月${new Date(date).getDate()}日`
      : date,
    items,
  }))
}

// 格式化时间
function formatTime(date: Date): string {
  return date.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" })
}

function formatDuration(minutes: number): string {
  if (minutes < 60) return `${minutes}分钟`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  return mins > 0 ? `${hours}小时${mins}分钟` : `${hours}小时`
}

// 活动类型图标和标签
const activityConfig: Record<ActivityType, { icon: typeof Play; label: string; color: string }> = {
  watch: { icon: Play, label: "观看", color: "text-blue-500" },
  finish: { icon: CheckCheck, label: "看完", color: "text-green-500" },
  add_list: { icon: Clock, label: "添加到列表", color: "text-purple-500" },
  remove_list: { icon: X, label: "从列表移除", color: "text-orange-500" },
  favorite: { icon: Clock, label: "收藏", color: "text-pink-500" },
  rate: { icon: Clock, label: "评分", color: "text-yellow-500" },
}

// 媒体类型图标
const mediaTypeIcons: Record<MediaType, typeof Film> = {
  movie: Film,
  series: Tv,
  anime: Tv,
  music: Music,
  photo: ImageIcon,
}

interface ActivityHistoryProps {
  onBack?: () => void
  onSelectMedia?: (mediaId: string, mediaType: MediaType) => void
  onContinueWatch?: (item: HistoryItem) => void
}

export function ActivityHistory({ onBack, onSelectMedia, onContinueWatch }: ActivityHistoryProps) {
  const [historyItems, setHistoryItems] = useState<HistoryItem[]>(() => generateMockHistory())
  const [searchQuery, setSearchQuery] = useState("")
  const [filterType, setFilterType] = useState<MediaType | "all">("all")
  const [isLoading, setIsLoading] = useState(false)
  const [clearDialogOpen, setClearDialogOpen] = useState(false)
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set())
  const [isSelectionMode, setIsSelectionMode] = useState(false)

  // 过滤历史记录
  const filteredItems = historyItems.filter((item) => {
    const matchesSearch = item.mediaTitle.toLowerCase().includes(searchQuery.toLowerCase())
    const matchesType = filterType === "all" || item.mediaType === filterType
    return matchesSearch && matchesType
  })

  // 分组
  const groupedHistory = groupByDate(filteredItems)

  // 删除单个记录
  const handleDelete = (id: string) => {
    setHistoryItems((prev) => prev.filter((item) => item.id !== id))
  }

  // 删除选中的记录
  const handleDeleteSelected = () => {
    setHistoryItems((prev) => prev.filter((item) => !selectedItems.has(item.id)))
    setSelectedItems(new Set())
    setIsSelectionMode(false)
  }

  // 清空所有历史
  const handleClearAll = () => {
    setHistoryItems([])
    setClearDialogOpen(false)
  }

  // 切换选中状态
  const toggleSelection = (id: string) => {
    setSelectedItems((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-lg font-semibold">播放历史</h1>
            <p className="text-xs text-muted-foreground">{historyItems.length} 条记录</p>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          {isSelectionMode ? (
            <>
              <span className="text-sm text-muted-foreground">
                已选择 {selectedItems.size} 项
              </span>
              <Button
                variant="destructive"
                size="sm"
                disabled={selectedItems.size === 0}
                onClick={handleDeleteSelected}
              >
                删除选中
              </Button>
              <Button variant="outline" size="sm" onClick={() => {
                setIsSelectionMode(false)
                setSelectedItems(new Set())
              }}>
                取消
              </Button>
            </>
          ) : (
            <>
              <Button variant="outline" size="sm" onClick={() => setIsSelectionMode(true)}>
                选择
              </Button>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button variant="ghost" size="icon">
                    <MoreHorizontal className="h-5 w-5" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem onClick={() => setIsSelectionMode(true)}>
                    选择多个
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    className="text-destructive"
                    onClick={() => setClearDialogOpen(true)}
                  >
                    清空所有历史
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </>
          )}
        </div>
      </header>

      {/* Search and Filter */}
      <div className="flex items-center gap-3 border-b border-border px-4 py-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            placeholder="搜索历史记录..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-9"
          />
        </div>
        
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm" className="gap-2">
              <Filter className="h-4 w-4" />
              {filterType === "all" ? "全部" : filterType === "movie" ? "电影" : filterType === "series" ? "剧集" : "动画"}
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end">
            <DropdownMenuItem onClick={() => setFilterType("all")}>
              全部类型
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setFilterType("movie")}>
              <Film className="mr-2 h-4 w-4" />
              电影
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setFilterType("series")}>
              <Tv className="mr-2 h-4 w-4" />
              剧集
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => setFilterType("anime")}>
              <Tv className="mr-2 h-4 w-4" />
              动画
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>

      {/* History List */}
      <ScrollArea className="flex-1">
        {isLoading ? (
          <div className="space-y-4 p-4">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="flex gap-3">
                <Skeleton className="h-20 w-14 rounded" />
                <div className="flex-1 space-y-2">
                  <Skeleton className="h-4 w-2/3" />
                  <Skeleton className="h-3 w-1/2" />
                  <Skeleton className="h-3 w-1/4" />
                </div>
              </div>
            ))}
          </div>
        ) : groupedHistory.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-20">
            <Clock className="mb-4 h-12 w-12 text-muted-foreground/30" />
            <p className="text-muted-foreground">暂无播放历史</p>
          </div>
        ) : (
          <div className="divide-y divide-border">
            {groupedHistory.map((group) => (
              <div key={group.date}>
                {/* Date Header */}
                <div className="sticky top-0 z-10 bg-muted/50 px-4 py-2 backdrop-blur-sm">
                  <div className="flex items-center gap-2">
                    <Calendar className="h-4 w-4 text-muted-foreground" />
                    <span className="text-sm font-medium">{group.label}</span>
                    <Badge variant="secondary" className="text-xs">
                      {group.items.length}
                    </Badge>
                  </div>
                </div>

                {/* Items */}
                <div className="divide-y divide-border/50">
                  {group.items.map((item) => (
                    <HistoryItemCard
                      key={item.id}
                      item={item}
                      isSelected={selectedItems.has(item.id)}
                      isSelectionMode={isSelectionMode}
                      onSelect={() => toggleSelection(item.id)}
                      onClick={() => onSelectMedia?.(item.mediaId, item.mediaType)}
                      onContinue={() => onContinueWatch?.(item)}
                      onDelete={() => handleDelete(item.id)}
                    />
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}
      </ScrollArea>

      {/* Clear All Dialog */}
      <AlertDialog open={clearDialogOpen} onOpenChange={setClearDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>清空播放历史</AlertDialogTitle>
            <AlertDialogDescription>
              确定要清空所有播放历史吗？此操作无法撤销。
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
              onClick={handleClearAll}
            >
              清空
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}

// 历史记录卡片
function HistoryItemCard({
  item,
  isSelected,
  isSelectionMode,
  onSelect,
  onClick,
  onContinue,
  onDelete,
}: {
  item: HistoryItem
  isSelected: boolean
  isSelectionMode: boolean
  onSelect: () => void
  onClick: () => void
  onContinue: () => void
  onDelete: () => void
}) {
  const MediaIcon = mediaTypeIcons[item.mediaType]
  const activityInfo = activityConfig[item.activityType]

  return (
    <div
      className={cn(
        "group flex gap-3 px-4 py-3 transition-colors",
        isSelectionMode && "cursor-pointer",
        isSelected && "bg-primary/5"
      )}
      onClick={isSelectionMode ? onSelect : undefined}
    >
      {/* Selection Checkbox */}
      {isSelectionMode && (
        <div className="flex items-center">
          <div
            className={cn(
              "flex h-5 w-5 items-center justify-center rounded border-2",
              isSelected ? "border-primary bg-primary text-primary-foreground" : "border-muted-foreground"
            )}
          >
            {isSelected && <CheckCheck className="h-3 w-3" />}
          </div>
        </div>
      )}

      {/* Poster */}
      <button
        className="relative h-20 w-14 flex-shrink-0 overflow-hidden rounded bg-muted"
        onClick={!isSelectionMode ? onClick : undefined}
        disabled={isSelectionMode}
      >
        <img
          src={item.poster}
          alt={item.mediaTitle}
          className="h-full w-full object-cover"
        />
        {item.progress !== undefined && item.progress < 100 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
            <div
              className="h-full bg-primary"
              style={{ width: `${item.progress}%` }}
            />
          </div>
        )}
        {/* Play overlay on hover */}
        {!isSelectionMode && (
          <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
            <Play className="h-6 w-6 text-white" fill="white" />
          </div>
        )}
      </button>

      {/* Info */}
      <div className="min-w-0 flex-1">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <h4 className="truncate font-medium">{item.mediaTitle}</h4>
            {item.episodeInfo && (
              <p className="text-sm text-muted-foreground">
                {item.episodeInfo.season && `S${item.episodeInfo.season} `}
                E{item.episodeInfo.episode}
                {item.episodeInfo.title && ` · ${item.episodeInfo.title}`}
              </p>
            )}
          </div>
          {!isSelectionMode && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon"
                  className="h-8 w-8 opacity-0 group-hover:opacity-100"
                >
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {item.progress !== undefined && item.progress < 100 && (
                  <DropdownMenuItem onClick={onContinue}>
                    <Play className="mr-2 h-4 w-4" />
                    继续播放
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem onClick={onClick}>
                  <MediaIcon className="mr-2 h-4 w-4" />
                  查看详情
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={onDelete} className="text-destructive">
                  <Trash2 className="mr-2 h-4 w-4" />
                  删除记录
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>

        <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <Badge variant="outline" className="h-5 gap-1 px-1.5">
            <MediaIcon className="h-3 w-3" />
            {item.mediaType === "movie" ? "电影" : item.mediaType === "anime" ? "动画" : "剧集"}
          </Badge>
          <span>{formatTime(item.timestamp)}</span>
          {item.watchedDuration !== undefined && item.duration && (
            <span>
              {formatDuration(item.watchedDuration)} / {formatDuration(item.duration)}
            </span>
          )}
          {item.device && <span>· {item.device}</span>}
        </div>

        {/* Progress bar for watching items */}
        {item.progress !== undefined && item.progress > 0 && item.progress < 100 && (
          <div className="mt-2">
            <Progress value={item.progress} className="h-1" />
          </div>
        )}
      </div>
    </div>
  )
}
