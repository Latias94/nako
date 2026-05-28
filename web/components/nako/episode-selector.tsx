"use client"

import { useState } from "react"
import { Play, Check, ChevronDown, ChevronRight, Clock, Star, Download, MoreHorizontal, List, Grid3X3 } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Progress } from "@/components/ui/progress"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import { cn } from "@/lib/utils"

// 内容类型
export type ContentType = "tv" | "anime" | "multi-part" | "documentary-series" | "custom"

// 分组类型
export type GroupType = "season" | "part" | "arc" | "cour" | "volume" | "chapter" | "custom"

// 单集信息
export interface Episode {
  id: string
  number: number
  title: string
  overview?: string
  thumbnail?: string
  duration: number // 分钟
  airDate?: string
  rating?: number
  watched: boolean
  progress?: number // 0-100
  subtitles?: string[]
  audioTracks?: string[]
}

// 分组（季/篇章/Part等）
export interface EpisodeGroup {
  id: string
  type: GroupType
  number: number
  title: string
  subtitle?: string // 如 "黄金乡篇" "Part 1: 石之海"
  poster?: string
  episodes: Episode[]
  totalEpisodes: number
  watchedCount: number
  year?: number
}

// 组件属性
interface EpisodeSelectorProps {
  contentType: ContentType
  groups: EpisodeGroup[]
  currentEpisodeId?: string
  onSelectEpisode: (episodeId: string, groupId: string) => void
  onMarkWatched?: (episodeId: string, watched: boolean) => void
  onDownload?: (episodeId: string) => void
  viewMode?: "list" | "grid"
  className?: string
}

// 分组类型的显示名称
const groupTypeLabels: Record<GroupType, Record<string, string>> = {
  season: { "zh-CN": "第{n}季", en: "Season {n}", ja: "シーズン{n}" },
  part: { "zh-CN": "Part {n}", en: "Part {n}", ja: "パート{n}" },
  arc: { "zh-CN": "第{n}篇", en: "Arc {n}", ja: "第{n}編" },
  cour: { "zh-CN": "第{n}期", en: "Cour {n}", ja: "第{n}クール" },
  volume: { "zh-CN": "第{n}卷", en: "Volume {n}", ja: "第{n}巻" },
  chapter: { "zh-CN": "第{n}章", en: "Chapter {n}", ja: "第{n}章" },
  custom: { "zh-CN": "", en: "", ja: "" },
}

function formatGroupTitle(type: GroupType, number: number, customTitle?: string, locale = "zh-CN"): string {
  if (type === "custom" && customTitle) return customTitle
  const template = groupTypeLabels[type]?.[locale] || groupTypeLabels[type]?.["zh-CN"] || ""
  return template.replace("{n}", String(number))
}

function formatDuration(minutes: number): string {
  if (minutes < 60) return `${minutes}分钟`
  const hours = Math.floor(minutes / 60)
  const mins = minutes % 60
  return mins > 0 ? `${hours}小时${mins}分钟` : `${hours}小时`
}

export function EpisodeSelector({
  contentType,
  groups,
  currentEpisodeId,
  onSelectEpisode,
  onMarkWatched,
  onDownload,
  viewMode: initialViewMode = "list",
  className,
}: EpisodeSelectorProps) {
  const [expandedGroups, setExpandedGroups] = useState<Set<string>>(
    new Set(groups.length === 1 ? [groups[0].id] : [])
  )
  const [viewMode, setViewMode] = useState<"list" | "grid">(initialViewMode)
  const [selectedGroup, setSelectedGroup] = useState<string | null>(
    groups.length > 0 ? groups[0].id : null
  )

  const toggleGroup = (groupId: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev)
      if (next.has(groupId)) {
        next.delete(groupId)
      } else {
        next.add(groupId)
      }
      return next
    })
  }

  // 单组时使用平铺视图
  const isSingleGroup = groups.length === 1
  const currentGroup = groups.find((g) => g.id === selectedGroup) || groups[0]

  // 计算总集数和已观看数
  const totalEpisodes = groups.reduce((sum, g) => sum + g.totalEpisodes, 0)
  const totalWatched = groups.reduce((sum, g) => sum + g.watchedCount, 0)

  return (
    <div className={cn("flex flex-col", className)}>
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-medium">
            {contentType === "anime" ? "剧集列表" : contentType === "multi-part" ? "分集" : "剧集"}
          </h3>
          <Badge variant="secondary" className="text-xs">
            {totalWatched}/{totalEpisodes}
          </Badge>
        </div>
        
        <div className="flex items-center gap-2">
          {/* 分组选择器（多组时显示） */}
          {!isSingleGroup && (
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="sm" className="h-8 gap-1">
                  {currentGroup && (
                    <>
                      {formatGroupTitle(currentGroup.type, currentGroup.number, currentGroup.subtitle)}
                      <ChevronDown className="h-3 w-3" />
                    </>
                  )}
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {groups.map((group) => (
                  <DropdownMenuItem
                    key={group.id}
                    onClick={() => setSelectedGroup(group.id)}
                    className="flex items-center justify-between gap-4"
                  >
                    <span>
                      {formatGroupTitle(group.type, group.number, group.subtitle)}
                      {group.title && group.type !== "custom" && (
                        <span className="ml-2 text-muted-foreground">{group.title}</span>
                      )}
                    </span>
                    <Badge variant="outline" className="text-xs">
                      {group.watchedCount}/{group.totalEpisodes}
                    </Badge>
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
          
          {/* 视图切换 */}
          <div className="flex items-center rounded-md border border-border">
            <Button
              variant={viewMode === "list" ? "secondary" : "ghost"}
              size="icon"
              className="h-7 w-7 rounded-r-none"
              onClick={() => setViewMode("list")}
            >
              <List className="h-3.5 w-3.5" />
            </Button>
            <Button
              variant={viewMode === "grid" ? "secondary" : "ghost"}
              size="icon"
              className="h-7 w-7 rounded-l-none"
              onClick={() => setViewMode("grid")}
            >
              <Grid3X3 className="h-3.5 w-3.5" />
            </Button>
          </div>
        </div>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        {isSingleGroup ? (
          // 单组：直接显示集列表
          <div className="p-4">
            {viewMode === "list" ? (
              <div className="space-y-1">
                {currentGroup?.episodes.map((episode) => (
                  <EpisodeListItem
                    key={episode.id}
                    episode={episode}
                    groupId={currentGroup.id}
                    isActive={episode.id === currentEpisodeId}
                    onSelect={onSelectEpisode}
                    onMarkWatched={onMarkWatched}
                    onDownload={onDownload}
                  />
                ))}
              </div>
            ) : (
              <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
                {currentGroup?.episodes.map((episode) => (
                  <EpisodeGridItem
                    key={episode.id}
                    episode={episode}
                    groupId={currentGroup.id}
                    isActive={episode.id === currentEpisodeId}
                    onSelect={onSelectEpisode}
                    onMarkWatched={onMarkWatched}
                  />
                ))}
              </div>
            )}
          </div>
        ) : (
          // 多组：折叠面板
          <div className="divide-y divide-border">
            {groups.map((group) => (
              <Collapsible
                key={group.id}
                open={expandedGroups.has(group.id)}
                onOpenChange={() => toggleGroup(group.id)}
              >
                <CollapsibleTrigger asChild>
                  <button className="flex w-full items-center gap-3 px-4 py-3 text-left transition-colors hover:bg-muted/50">
                    <ChevronRight
                      className={cn(
                        "h-4 w-4 text-muted-foreground transition-transform",
                        expandedGroups.has(group.id) && "rotate-90"
                      )}
                    />
                    {group.poster && (
                      <img
                        src={group.poster}
                        alt={group.title}
                        className="h-12 w-8 rounded object-cover"
                      />
                    )}
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium">
                          {formatGroupTitle(group.type, group.number, group.subtitle)}
                        </span>
                        {group.title && group.type !== "custom" && (
                          <span className="text-sm text-muted-foreground">{group.title}</span>
                        )}
                      </div>
                      <div className="flex items-center gap-3 text-xs text-muted-foreground">
                        <span>{group.totalEpisodes}集</span>
                        {group.year && <span>{group.year}</span>}
                        <span>{group.watchedCount}/{group.totalEpisodes} 已观看</span>
                      </div>
                    </div>
                    <Progress
                      value={(group.watchedCount / group.totalEpisodes) * 100}
                      className="h-1.5 w-16"
                    />
                  </button>
                </CollapsibleTrigger>
                <CollapsibleContent>
                  <div className="border-t border-border/50 bg-muted/20 p-4">
                    {viewMode === "list" ? (
                      <div className="space-y-1">
                        {group.episodes.map((episode) => (
                          <EpisodeListItem
                            key={episode.id}
                            episode={episode}
                            groupId={group.id}
                            isActive={episode.id === currentEpisodeId}
                            onSelect={onSelectEpisode}
                            onMarkWatched={onMarkWatched}
                            onDownload={onDownload}
                          />
                        ))}
                      </div>
                    ) : (
                      <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
                        {group.episodes.map((episode) => (
                          <EpisodeGridItem
                            key={episode.id}
                            episode={episode}
                            groupId={group.id}
                            isActive={episode.id === currentEpisodeId}
                            onSelect={onSelectEpisode}
                            onMarkWatched={onMarkWatched}
                          />
                        ))}
                      </div>
                    )}
                  </div>
                </CollapsibleContent>
              </Collapsible>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  )
}

// 列表项
function EpisodeListItem({
  episode,
  groupId,
  isActive,
  onSelect,
  onMarkWatched,
  onDownload,
}: {
  episode: Episode
  groupId: string
  isActive: boolean
  onSelect: (episodeId: string, groupId: string) => void
  onMarkWatched?: (episodeId: string, watched: boolean) => void
  onDownload?: (episodeId: string) => void
}) {
  return (
    <div
      className={cn(
        "group flex items-center gap-3 rounded-lg px-3 py-2 transition-colors",
        isActive ? "bg-primary/10 text-primary" : "hover:bg-muted/50",
        episode.watched && !isActive && "opacity-60"
      )}
    >
      {/* 缩略图 */}
      <button
        className="relative h-14 w-24 flex-shrink-0 overflow-hidden rounded bg-muted"
        onClick={() => onSelect(episode.id, groupId)}
      >
        {episode.thumbnail ? (
          <img
            src={episode.thumbnail}
            alt={episode.title}
            className="h-full w-full object-cover"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center text-2xl font-bold text-muted-foreground/30">
            {episode.number}
          </div>
        )}
        {/* 播放进度 */}
        {episode.progress !== undefined && episode.progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
            <div
              className="h-full bg-primary"
              style={{ width: `${episode.progress}%` }}
            />
          </div>
        )}
        {/* 播放按钮 */}
        <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
          <Play className="h-6 w-6 text-white" fill="white" />
        </div>
      </button>

      {/* 信息 */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">第{episode.number}集</span>
          {episode.watched && <Check className="h-3.5 w-3.5 text-primary" />}
        </div>
        <p className="truncate text-sm text-muted-foreground">{episode.title}</p>
        <div className="mt-0.5 flex items-center gap-2 text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            <Clock className="h-3 w-3" />
            {formatDuration(episode.duration)}
          </span>
          {episode.rating && (
            <span className="flex items-center gap-1">
              <Star className="h-3 w-3 fill-yellow-500 text-yellow-500" />
              {episode.rating.toFixed(1)}
            </span>
          )}
          {episode.airDate && <span>{episode.airDate}</span>}
        </div>
      </div>

      {/* 操作 */}
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
          <DropdownMenuItem onClick={() => onSelect(episode.id, groupId)}>
            <Play className="mr-2 h-4 w-4" />
            播放
          </DropdownMenuItem>
          <DropdownMenuItem onClick={() => onMarkWatched?.(episode.id, !episode.watched)}>
            <Check className="mr-2 h-4 w-4" />
            {episode.watched ? "标记为未观看" : "标记为已观看"}
          </DropdownMenuItem>
          {onDownload && (
            <DropdownMenuItem onClick={() => onDownload(episode.id)}>
              <Download className="mr-2 h-4 w-4" />
              下载
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}

// 网格项
function EpisodeGridItem({
  episode,
  groupId,
  isActive,
  onSelect,
  onMarkWatched,
}: {
  episode: Episode
  groupId: string
  isActive: boolean
  onSelect: (episodeId: string, groupId: string) => void
  onMarkWatched?: (episodeId: string, watched: boolean) => void
}) {
  return (
    <button
      className={cn(
        "group relative overflow-hidden rounded-lg text-left transition-all",
        isActive && "ring-2 ring-primary",
        episode.watched && !isActive && "opacity-60"
      )}
      onClick={() => onSelect(episode.id, groupId)}
    >
      {/* 缩略图 */}
      <div className="relative aspect-video overflow-hidden rounded-lg bg-muted">
        {episode.thumbnail ? (
          <img
            src={episode.thumbnail}
            alt={episode.title}
            className="h-full w-full object-cover transition-transform group-hover:scale-105"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center text-3xl font-bold text-muted-foreground/30">
            {episode.number}
          </div>
        )}
        
        {/* 悬浮播放按钮 */}
        <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
          <Play className="h-10 w-10 text-white" fill="white" />
        </div>
        
        {/* 进度条 */}
        {episode.progress !== undefined && episode.progress > 0 && (
          <div className="absolute bottom-0 left-0 right-0 h-1 bg-black/50">
            <div
              className="h-full bg-primary"
              style={{ width: `${episode.progress}%` }}
            />
          </div>
        )}
        
        {/* 已观看标记 */}
        {episode.watched && (
          <div className="absolute right-1 top-1 rounded-full bg-primary p-0.5">
            <Check className="h-3 w-3 text-primary-foreground" />
          </div>
        )}
        
        {/* 时长 */}
        <div className="absolute bottom-1 right-1 rounded bg-black/70 px-1.5 py-0.5 text-[10px] text-white">
          {formatDuration(episode.duration)}
        </div>
      </div>
      
      {/* 信息 */}
      <div className="mt-2 space-y-0.5">
        <p className="text-xs font-medium">第{episode.number}集</p>
        <p className="truncate text-xs text-muted-foreground">{episode.title}</p>
      </div>
    </button>
  )
}
