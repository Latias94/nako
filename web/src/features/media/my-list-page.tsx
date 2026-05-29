"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState } from "react"
import { ChevronLeft, Play, Star, Heart, Clock, Trash2, Filter, Grid3X3, List, MoreVertical } from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"

// 我的列表数据
const myListData = {
  favorites: [
    { id: "1", title: "沙丘2", year: 2024, rating: 8.6, poster: "/posters/dune2.jpg", type: "电影", addedDate: "2024-03-15" },
    { id: "2", title: "星际穿越", year: 2014, rating: 8.7, poster: "/posters/interstellar.jpg", type: "电影", addedDate: "2024-03-10" },
    { id: "3", title: "真探", year: 2014, rating: 8.9, poster: "/posters/true-detective.jpg", type: "剧集", addedDate: "2024-03-05" },
    { id: "4", title: "银翼杀手 2049", year: 2017, rating: 8.0, poster: "/posters/blade-runner.jpg", type: "电影", addedDate: "2024-02-28" },
  ],
  watchLater: [
    { id: "5", title: "奥本海默", year: 2023, rating: 8.4, poster: "/posters/oppenheimer.jpg", type: "电影", addedDate: "2024-03-12" },
    { id: "6", title: "绝命毒师", year: 2008, rating: 9.5, poster: "/posters/breaking-bad.jpg", type: "剧集", addedDate: "2024-03-08" },
  ],
  history: [
    { id: "1", title: "沙丘2", year: 2024, rating: 8.6, poster: "/posters/dune2.jpg", type: "电影", watchedDate: "2024-03-16", progress: 100 },
    { id: "7", title: "降临", year: 2016, rating: 7.9, poster: "/posters/arrival.jpg", type: "电影", watchedDate: "2024-03-14", progress: 100 },
    { id: "3", title: "真探 S01E03", year: 2014, rating: 8.9, poster: "/posters/true-detective.jpg", type: "剧集", watchedDate: "2024-03-13", progress: 45 },
  ]
}

interface MyListPageProps {
  onBack: () => void
  onSelectMedia: (id: string, type: "movie" | "series") => void
}

export function MyListPage({ onBack, onSelectMedia }: MyListPageProps) {
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid")
  const [activeTab, setActiveTab] = useState("favorites")

  return (
    <div className="min-h-screen bg-background">
      {/* 顶部导航 */}
      <div className="sticky top-0 z-10 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto flex max-w-6xl items-center justify-between p-4">
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="icon" onClick={onBack}>
              <ChevronLeft className="h-5 w-5" />
            </Button>
            <h1 className="text-xl font-semibold">我的列表</h1>
          </div>

          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setViewMode(viewMode === "grid" ? "list" : "grid")}
              className="h-9 w-9"
            >
              {viewMode === "grid" ? <List className="h-4 w-4" /> : <Grid3X3 className="h-4 w-4" />}
            </Button>
            <Button variant="ghost" size="icon" className="h-9 w-9">
              <Filter className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>

      {/* 内容区域 */}
      <div className="mx-auto max-w-6xl p-4">
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="mb-6 grid w-full grid-cols-3 lg:w-auto lg:grid-cols-none lg:inline-flex">
            <TabsTrigger value="favorites" className="gap-2">
              <Heart className="h-4 w-4" />
              <span>收藏</span>
              <Badge variant="secondary" className="ml-1">{myListData.favorites.length}</Badge>
            </TabsTrigger>
            <TabsTrigger value="watchLater" className="gap-2">
              <Clock className="h-4 w-4" />
              <span>稍后观看</span>
              <Badge variant="secondary" className="ml-1">{myListData.watchLater.length}</Badge>
            </TabsTrigger>
            <TabsTrigger value="history" className="gap-2">
              <Play className="h-4 w-4" />
              <span>观看历史</span>
            </TabsTrigger>
          </TabsList>

          {/* 收藏列表 */}
          <TabsContent value="favorites">
            {myListData.favorites.length === 0 ? (
              <EmptyState
                icon={<Heart className="h-12 w-12" />}
                title="暂无收藏"
                description="浏览媒体库，将喜欢的内容添加到收藏"
              />
            ) : (
              <MediaList
                items={myListData.favorites}
                viewMode={viewMode}
                onSelect={onSelectMedia}
                showDate
                dateLabel="添加于"
              />
            )}
          </TabsContent>

          {/* 稍后观看 */}
          <TabsContent value="watchLater">
            {myListData.watchLater.length === 0 ? (
              <EmptyState
                icon={<Clock className="h-12 w-12" />}
                title="稍后观看列表为空"
                description="将想看的内容添加到稍后观看列表"
              />
            ) : (
              <MediaList
                items={myListData.watchLater}
                viewMode={viewMode}
                onSelect={onSelectMedia}
                showDate
                dateLabel="添加于"
              />
            )}
          </TabsContent>

          {/* 观看历史 */}
          <TabsContent value="history">
            <div className="mb-4 flex items-center justify-between">
              <p className="text-sm text-muted-foreground">
                显示最近 30 天的观看记录
              </p>
              <Button variant="ghost" size="sm" className="text-destructive hover:text-destructive">
                <Trash2 className="mr-2 h-4 w-4" />
                清除历史
              </Button>
            </div>
            {myListData.history.length === 0 ? (
              <EmptyState
                icon={<Play className="h-12 w-12" />}
                title="暂无观看记录"
                description="开始观看内容后，历史记录将显示在这里"
              />
            ) : (
              <MediaList
                items={myListData.history}
                viewMode={viewMode}
                onSelect={onSelectMedia}
                showDate
                dateLabel="观看于"
                showProgress
              />
            )}
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}

// 空状态组件
function EmptyState({ icon, title, description }: { icon: React.ReactNode; title: string; description: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center">
      <div className="mb-4 text-muted-foreground/50">{icon}</div>
      <h3 className="mb-2 text-lg font-medium">{title}</h3>
      <p className="text-sm text-muted-foreground">{description}</p>
    </div>
  )
}

// 媒体列表组件
function MediaList({
  items,
  viewMode,
  onSelect,
  showDate,
  dateLabel,
  showProgress
}: {
  items: Array<{
    id: string
    title: string
    year: number
    rating: number
    poster: string
    type: string
    addedDate?: string
    watchedDate?: string
    progress?: number
  }>
  viewMode: "grid" | "list"
  onSelect: (id: string, type: "movie" | "series") => void
  showDate?: boolean
  dateLabel?: string
  showProgress?: boolean
}) {
  if (viewMode === "list") {
    return (
      <div className="space-y-2">
        {items.map((item) => (
          <div
            key={item.id}
            className="group flex items-center gap-4 rounded-lg border border-border/50 bg-card p-3 transition-colors hover:bg-secondary/50"
          >
            <button
              onClick={() => onSelect(item.id, item.type === "剧集" ? "series" : "movie")}
              className="relative h-20 w-14 flex-shrink-0 overflow-hidden rounded-md"
            >
              <img src={resolveArtwork(item.poster)} alt={item.title} className="h-full w-full object-cover" />
              <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100">
                <Play className="h-6 w-6 text-white" />
              </div>
              {showProgress && item.progress !== undefined && item.progress < 100 && (
                <div className="absolute inset-x-0 bottom-0 h-1 bg-white/30">
                  <div className="h-full bg-primary" style={{ width: `${item.progress}%` }} />
                </div>
              )}
            </button>

            <div className="flex-1">
              <button
                onClick={() => onSelect(item.id, item.type === "剧集" ? "series" : "movie")}
                className="text-left"
              >
                <h3 className="font-medium hover:text-primary">{item.title}</h3>
              </button>
              <div className="mt-1 flex items-center gap-2 text-sm text-muted-foreground">
                <span>{item.year}</span>
                <span>·</span>
                <div className="flex items-center gap-1">
                  <Star className="h-3 w-3 fill-accent text-accent" />
                  <span>{item.rating}</span>
                </div>
                <span>·</span>
                <Badge variant="outline" className="text-[10px]">{item.type}</Badge>
              </div>
              {showDate && (
                <p className="mt-1 text-xs text-muted-foreground">
                  {dateLabel} {item.addedDate || item.watchedDate}
                </p>
              )}
            </div>

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8 opacity-0 group-hover:opacity-100">
                  <MoreVertical className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem>
                  <Play className="mr-2 h-4 w-4" />
                  播放
                </DropdownMenuItem>
                <DropdownMenuItem>
                  <Heart className="mr-2 h-4 w-4" />
                  移除收藏
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        ))}
      </div>
    )
  }

  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
      {items.map((item) => (
        <div key={item.id} className="group relative">
          <button
            onClick={() => onSelect(item.id, item.type === "剧集" ? "series" : "movie")}
            className="w-full text-left"
          >
            <div className="relative aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-transform group-hover:scale-[1.02]">
              <img src={resolveArtwork(item.poster)} alt={item.title} className="h-full w-full object-cover" />
              <div className="absolute inset-0 bg-black/50 opacity-0 transition-opacity group-hover:opacity-100" />
              <Button
                size="icon"
                className="absolute left-1/2 top-1/2 h-10 w-10 -translate-x-1/2 -translate-y-1/2 rounded-full opacity-0 transition-opacity group-hover:opacity-100"
              >
                <Play className="h-4 w-4" />
              </Button>
              {showProgress && item.progress !== undefined && item.progress < 100 && (
                <div className="absolute inset-x-0 bottom-0 h-1 bg-white/30">
                  <div className="h-full bg-primary" style={{ width: `${item.progress}%` }} />
                </div>
              )}
            </div>
            <h3 className="mt-2 truncate text-sm font-medium">{item.title}</h3>
            <div className="flex items-center gap-1 text-xs text-muted-foreground">
              <span>{item.year}</span>
              <span>·</span>
              <Star className="h-3 w-3 fill-accent text-accent" />
              <span>{item.rating}</span>
            </div>
          </button>

          {/* 快捷操作 */}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                className="absolute right-1 top-1 h-7 w-7 bg-black/50 text-white opacity-0 backdrop-blur transition-opacity hover:bg-black/70 group-hover:opacity-100"
              >
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem>
                <Play className="mr-2 h-4 w-4" />
                播放
              </DropdownMenuItem>
              <DropdownMenuItem>
                <Heart className="mr-2 h-4 w-4" />
                移除收藏
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      ))}
    </div>
  )
}
