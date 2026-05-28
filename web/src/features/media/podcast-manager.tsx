"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState, useEffect } from "react"
import {
  ChevronLeft, Play, Pause, SkipBack, SkipForward, Clock, Download, Share2,
  Plus, Search, Rss, Check, MoreHorizontal, ExternalLink, RefreshCw, Settings,
  ChevronRight, Headphones, ListMusic, Calendar, CheckCircle2, Circle, Bookmark, X
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Slider } from "@/components/ui/slider"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Label } from "@/components/ui/label"
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
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

// Types
interface Podcast {
  id: string
  title: string
  author: string
  description: string
  cover: string
  feedUrl: string
  website?: string
  episodeCount: number
  lastUpdated: string
  isSubscribed: boolean
  categories: string[]
}

interface Episode {
  id: string
  podcastId: string
  podcastTitle: string
  podcastCover: string
  title: string
  description: string
  duration: number
  publishedAt: string
  audioUrl: string
  isPlayed: boolean
  isDownloaded: boolean
  playProgress: number
  isBookmarked: boolean
}

interface PodcastManagerProps {
  onBack: () => void
}

// Mock data
const mockPodcasts: Podcast[] = [
  {
    id: "1",
    title: "科技乱谈",
    author: "科技乱谈",
    description: "每周聊聊科技圈的那些事儿，深度分析科技新闻和行业趋势。",
    cover: "https://picsum.photos/seed/tech/300/300",
    feedUrl: "https://example.com/feed1.xml",
    website: "https://example.com",
    episodeCount: 156,
    lastUpdated: "2024-03-15",
    isSubscribed: true,
    categories: ["科技", "新闻"],
  },
  {
    id: "2",
    title: "故事 FM",
    author: "故事 FM",
    description: "用你的声音，讲述你的故事。真实的人，真实的故事。",
    cover: "https://picsum.photos/seed/story/300/300",
    feedUrl: "https://example.com/feed2.xml",
    episodeCount: 320,
    lastUpdated: "2024-03-14",
    isSubscribed: true,
    categories: ["故事", "生活"],
  },
  {
    id: "3",
    title: "商业就是这样",
    author: "商业就是这样",
    description: "理解商业世界的运作逻辑，洞察商业背后的故事。",
    cover: "https://picsum.photos/seed/biz/300/300",
    feedUrl: "https://example.com/feed3.xml",
    episodeCount: 89,
    lastUpdated: "2024-03-13",
    isSubscribed: true,
    categories: ["商业", "财经"],
  },
  {
    id: "4",
    title: "日谈公园",
    author: "日谈公园",
    description: "一档泛文化类播客节目，聊聊生活中有趣的话题。",
    cover: "https://picsum.photos/seed/park/300/300",
    feedUrl: "https://example.com/feed4.xml",
    episodeCount: 450,
    lastUpdated: "2024-03-12",
    isSubscribed: false,
    categories: ["脱口秀", "生活"],
  },
  {
    id: "5",
    title: "硬核历史",
    author: "硬核历史",
    description: "深入浅出地讲述历史故事，揭秘历史真相。",
    cover: "https://picsum.photos/seed/history/300/300",
    feedUrl: "https://example.com/feed5.xml",
    episodeCount: 78,
    lastUpdated: "2024-03-10",
    isSubscribed: false,
    categories: ["历史", "教育"],
  },
]

const generateMockEpisodes = (count: number): Episode[] => {
  return Array.from({ length: count }, (_, i) => {
    const podcast = mockPodcasts[i % mockPodcasts.length]
    return {
      id: `episode-${i}`,
      podcastId: podcast.id,
      podcastTitle: podcast.title,
      podcastCover: podcast.cover,
      title: `第 ${count - i} 期：${["人工智能的未来", "创业故事分享", "数字游民生活", "投资理财入门", "职场生存指南"][i % 5]}`,
      description: "本期节目我们聊聊最近发生的一些有趣的事情，邀请了特别嘉宾来分享他们的观点和经验。",
      duration: 1800 + Math.floor(Math.random() * 3600),
      publishedAt: new Date(Date.now() - i * 24 * 60 * 60 * 1000).toISOString(),
      audioUrl: "",
      isPlayed: Math.random() > 0.7,
      isDownloaded: Math.random() > 0.8,
      playProgress: Math.random() > 0.7 ? Math.floor(Math.random() * 100) : 0,
      isBookmarked: Math.random() > 0.9,
    }
  })
}

type ViewMode = "subscriptions" | "discover" | "downloads" | "history" | "podcast-detail"

export function PodcastManager({ onBack }: PodcastManagerProps) {
  const [viewMode, setViewMode] = useState<ViewMode>("subscriptions")
  const [episodes] = useState(() => generateMockEpisodes(50))
  const [selectedPodcast, setSelectedPodcast] = useState<Podcast | null>(null)
  const [currentEpisode, setCurrentEpisode] = useState<Episode | null>(null)
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentTime, setCurrentTime] = useState(0)
  const [playbackSpeed, setPlaybackSpeed] = useState(1)
  const [searchQuery, setSearchQuery] = useState("")
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [feedUrl, setFeedUrl] = useState("")
  const [isLoading, setIsLoading] = useState(true)
  const [activeTab, setActiveTab] = useState<"all" | "unplayed">("all")

  useEffect(() => {
    const timer = setTimeout(() => setIsLoading(false), 500)
    return () => clearTimeout(timer)
  }, [])

  // Simulate playback progress
  useEffect(() => {
    if (isPlaying && currentEpisode) {
      const interval = setInterval(() => {
        setCurrentTime(t => {
          if (t >= currentEpisode.duration) return 0
          return t + playbackSpeed
        })
      }, 1000)
      return () => clearInterval(interval)
    }
  }, [isPlaying, currentEpisode, playbackSpeed])

  const formatTime = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    const s = Math.floor(seconds % 60)
    if (h > 0) return `${h}:${m.toString().padStart(2, "0")}:${s.toString().padStart(2, "0")}`
    return `${m}:${s.toString().padStart(2, "0")}`
  }

  const formatDuration = (seconds: number) => {
    const h = Math.floor(seconds / 3600)
    const m = Math.floor((seconds % 3600) / 60)
    if (h > 0) return `${h} 小时 ${m} 分钟`
    return `${m} 分钟`
  }

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr)
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const days = Math.floor(diff / (24 * 60 * 60 * 1000))

    if (days === 0) return "今天"
    if (days === 1) return "昨天"
    if (days < 7) return `${days} 天前`
    return date.toLocaleDateString("zh-CN", { month: "short", day: "numeric" })
  }

  const playEpisode = (episode: Episode) => {
    setCurrentEpisode(episode)
    setCurrentTime(episode.playProgress > 0 ? (episode.playProgress / 100) * episode.duration : 0)
    setIsPlaying(true)
  }

  const subscribedPodcasts = mockPodcasts.filter(p => p.isSubscribed)
  const discoverPodcasts = mockPodcasts.filter(p => !p.isSubscribed)
  const downloadedEpisodes = episodes.filter(e => e.isDownloaded)
  const playedEpisodes = episodes.filter(e => e.isPlayed)
  const filteredEpisodes = activeTab === "unplayed" ? episodes.filter(e => !e.isPlayed) : episodes

  // Podcast card
  const PodcastCard = ({ podcast }: { podcast: Podcast }) => (
    <div
      className="group cursor-pointer"
      onClick={() => { setSelectedPodcast(podcast); setViewMode("podcast-detail") }}
    >
      <div className="relative mb-2 overflow-hidden rounded-lg">
        <img
          src={resolveArtwork(podcast.cover)}
          alt={podcast.title}
          className="aspect-square w-full object-cover transition-transform group-hover:scale-105"
        />
        {podcast.isSubscribed && (
          <Badge className="absolute right-2 top-2 bg-primary">已订阅</Badge>
        )}
      </div>
      <h3 className="truncate font-medium">{podcast.title}</h3>
      <p className="text-sm text-muted-foreground">{podcast.episodeCount} 集</p>
    </div>
  )

  // Episode item
  const EpisodeItem = ({ episode, showPodcast = true }: { episode: Episode; showPodcast?: boolean }) => (
    <div className={cn(
      "group flex gap-3 rounded-lg p-3 hover:bg-muted/50",
      currentEpisode?.id === episode.id && "bg-muted"
    )}>
      {showPodcast && (
        <img
          src={resolveArtwork(episode.podcastCover)}
          alt={episode.podcastTitle}
          className="h-16 w-16 flex-shrink-0 rounded-lg object-cover"
        />
      )}
      <div className="min-w-0 flex-1">
        <div className="mb-1 flex items-start justify-between gap-2">
          <div className="min-w-0">
            {showPodcast && (
              <p className="truncate text-xs text-muted-foreground">{episode.podcastTitle}</p>
            )}
            <h4 className={cn(
              "line-clamp-2 font-medium leading-snug",
              episode.isPlayed && "text-muted-foreground"
            )}>
              {episode.title}
            </h4>
          </div>
          <div className="flex items-center gap-1">
            {episode.isBookmarked && <Bookmark className="h-4 w-4 fill-primary text-primary" />}
            {episode.isDownloaded && <Download className="h-4 w-4 text-muted-foreground" />}
          </div>
        </div>
        <p className="mb-2 line-clamp-2 text-sm text-muted-foreground">{episode.description}</p>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3 text-xs text-muted-foreground">
            <span>{formatDate(episode.publishedAt)}</span>
            <span className="flex items-center gap-1">
              <Clock className="h-3 w-3" />
              {formatDuration(episode.duration)}
            </span>
            {episode.playProgress > 0 && episode.playProgress < 100 && (
              <span className="text-primary">{episode.playProgress}% 已播放</span>
            )}
            {episode.isPlayed && episode.playProgress === 0 && (
              <span className="flex items-center gap-1 text-green-500">
                <CheckCircle2 className="h-3 w-3" />
                已播放
              </span>
            )}
          </div>
          <div className="flex items-center gap-1">
            <Button
              size="sm"
              variant={currentEpisode?.id === episode.id && isPlaying ? "secondary" : "default"}
              className="h-8"
              onClick={(e) => { e.stopPropagation(); playEpisode(episode) }}
            >
              {currentEpisode?.id === episode.id && isPlaying ? (
                <><Pause className="mr-1 h-3 w-3" /> 暂停</>
              ) : (
                <><Play className="mr-1 h-3 w-3" /> 播放</>
              )}
            </Button>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem>
                  <Bookmark className="mr-2 h-4 w-4" />
                  {episode.isBookmarked ? "取消收藏" : "收藏"}
                </DropdownMenuItem>
                <DropdownMenuItem>
                  <Download className="mr-2 h-4 w-4" />
                  {episode.isDownloaded ? "删除下载" : "下载"}
                </DropdownMenuItem>
                <DropdownMenuItem>
                  {episode.isPlayed ? (
                    <><Circle className="mr-2 h-4 w-4" /> 标记为未播放</>
                  ) : (
                    <><CheckCircle2 className="mr-2 h-4 w-4" /> 标记为已播放</>
                  )}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem>
                  <Share2 className="mr-2 h-4 w-4" />
                  分享
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
        {/* Progress bar */}
        {episode.playProgress > 0 && episode.playProgress < 100 && (
          <div className="mt-2 h-1 overflow-hidden rounded-full bg-muted">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${episode.playProgress}%` }}
            />
          </div>
        )}
      </div>
    </div>
  )

  return (
    <div className="flex h-screen flex-col bg-background">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ChevronLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-center gap-2">
            <img src={resolveArtwork("/nako-icon.png")} alt="Nako" className="h-8 w-8 rounded-lg" />
            <h1 className="text-lg font-semibold">播客</h1>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="搜索播客或单集..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-64 pl-9"
            />
          </div>

          <Button onClick={() => setShowAddDialog(true)}>
            <Plus className="mr-2 h-4 w-4" />
            添加播客
          </Button>

          <Button variant="outline" size="icon">
            <RefreshCw className="h-4 w-4" />
          </Button>
        </div>
      </header>

      {/* Navigation tabs */}
      <div className="border-b border-border px-4 py-2">
        <Tabs value={viewMode} onValueChange={(v) => setViewMode(v as ViewMode)}>
          <TabsList>
            <TabsTrigger value="subscriptions" className="gap-2">
              <Rss className="h-4 w-4" />
              订阅
            </TabsTrigger>
            <TabsTrigger value="discover" className="gap-2">
              <Headphones className="h-4 w-4" />
              发现
            </TabsTrigger>
            <TabsTrigger value="downloads" className="gap-2">
              <Download className="h-4 w-4" />
              下载
            </TabsTrigger>
            <TabsTrigger value="history" className="gap-2">
              <Clock className="h-4 w-4" />
              历史
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* Content */}
      <ScrollArea className="flex-1">
        <div className="p-4 pb-32">
          {/* Subscriptions View */}
          {viewMode === "subscriptions" && (
            <div className="space-y-8">
              {/* My podcasts */}
              <section>
                <h2 className="mb-4 text-lg font-semibold">我的播客 ({subscribedPodcasts.length})</h2>
                <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {subscribedPodcasts.map(podcast => (
                    <PodcastCard key={podcast.id} podcast={podcast} />
                  ))}
                </div>
              </section>

              {/* Latest episodes */}
              <section>
                <div className="mb-4 flex items-center justify-between">
                  <h2 className="text-lg font-semibold">最新单集</h2>
                  <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as "all" | "unplayed")}>
                    <TabsList className="h-8">
                      <TabsTrigger value="all" className="h-6 text-xs">全部</TabsTrigger>
                      <TabsTrigger value="unplayed" className="h-6 text-xs">未播放</TabsTrigger>
                    </TabsList>
                  </Tabs>
                </div>
                <div className="space-y-2">
                  {isLoading ? (
                    Array.from({ length: 5 }).map((_, i) => (
                      <div key={i} className="flex gap-3 p-3">
                        <Skeleton className="h-16 w-16 rounded-lg" />
                        <div className="flex-1 space-y-2">
                          <Skeleton className="h-4 w-1/4" />
                          <Skeleton className="h-4 w-3/4" />
                          <Skeleton className="h-3 w-1/2" />
                        </div>
                      </div>
                    ))
                  ) : (
                    filteredEpisodes.slice(0, 20).map(episode => (
                      <EpisodeItem key={episode.id} episode={episode} />
                    ))
                  )}
                </div>
              </section>
            </div>
          )}

          {/* Discover View */}
          {viewMode === "discover" && (
            <div className="space-y-8">
              <section>
                <h2 className="mb-4 text-lg font-semibold">发现新播客</h2>
                <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
                  {discoverPodcasts.map(podcast => (
                    <PodcastCard key={podcast.id} podcast={podcast} />
                  ))}
                </div>
              </section>

              <section>
                <h2 className="mb-4 text-lg font-semibold">热门分类</h2>
                <div className="flex flex-wrap gap-2">
                  {["科技", "商业", "历史", "故事", "喜剧", "教育", "新闻", "生活", "音乐", "体育"].map(cat => (
                    <Badge key={cat} variant="secondary" className="cursor-pointer px-4 py-2 text-sm hover:bg-secondary/80">
                      {cat}
                    </Badge>
                  ))}
                </div>
              </section>
            </div>
          )}

          {/* Downloads View */}
          {viewMode === "downloads" && (
            <div>
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold">已下载 ({downloadedEpisodes.length})</h2>
                <Button variant="outline" size="sm">
                  <Settings className="mr-2 h-4 w-4" />
                  下载设置
                </Button>
              </div>
              {downloadedEpisodes.length === 0 ? (
                <div className="py-12 text-center">
                  <Download className="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
                  <p className="text-muted-foreground">暂无下载的单集</p>
                  <p className="text-sm text-muted-foreground">下载单集以便离线收听</p>
                </div>
              ) : (
                <div className="space-y-2">
                  {downloadedEpisodes.map(episode => (
                    <EpisodeItem key={episode.id} episode={episode} />
                  ))}
                </div>
              )}
            </div>
          )}

          {/* History View */}
          {viewMode === "history" && (
            <div>
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold">播放历史</h2>
                <Button variant="outline" size="sm">
                  清除历史
                </Button>
              </div>
              <div className="space-y-2">
                {playedEpisodes.map(episode => (
                  <EpisodeItem key={episode.id} episode={episode} />
                ))}
              </div>
            </div>
          )}

          {/* Podcast Detail View */}
          {viewMode === "podcast-detail" && selectedPodcast && (
            <div>
              <Button variant="ghost" className="mb-4" onClick={() => setViewMode("subscriptions")}>
                <ChevronLeft className="mr-2 h-4 w-4" />
                返回
              </Button>

              <div className="mb-6 flex gap-6">
                <img
                  src={resolveArtwork(selectedPodcast.cover)}
                  alt={selectedPodcast.title}
                  className="h-48 w-48 rounded-xl object-cover shadow-lg"
                />
                <div className="flex flex-col justify-between py-2">
                  <div>
                    <h1 className="mb-2 text-2xl font-bold">{selectedPodcast.title}</h1>
                    <p className="mb-2 text-muted-foreground">{selectedPodcast.author}</p>
                    <p className="mb-4 line-clamp-3 text-sm text-muted-foreground">{selectedPodcast.description}</p>
                    <div className="flex flex-wrap gap-2">
                      {selectedPodcast.categories.map(cat => (
                        <Badge key={cat} variant="secondary">{cat}</Badge>
                      ))}
                    </div>
                  </div>
                  <div className="flex items-center gap-2">
                    <Button>
                      {selectedPodcast.isSubscribed ? (
                        <><Check className="mr-2 h-4 w-4" /> 已订阅</>
                      ) : (
                        <><Plus className="mr-2 h-4 w-4" /> 订阅</>
                      )}
                    </Button>
                    {selectedPodcast.website && (
                      <Button variant="outline">
                        <ExternalLink className="mr-2 h-4 w-4" />
                        访问网站
                      </Button>
                    )}
                    <Button variant="ghost" size="icon">
                      <Share2 className="h-5 w-5" />
                    </Button>
                  </div>
                </div>
              </div>

              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-lg font-semibold">全部单集 ({selectedPodcast.episodeCount})</h2>
                <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as "all" | "unplayed")}>
                  <TabsList className="h-8">
                    <TabsTrigger value="all" className="h-6 text-xs">全部</TabsTrigger>
                    <TabsTrigger value="unplayed" className="h-6 text-xs">未播放</TabsTrigger>
                  </TabsList>
                </Tabs>
              </div>
              <div className="space-y-2">
                {episodes
                  .filter(e => e.podcastId === selectedPodcast.id)
                  .filter(e => activeTab === "all" || !e.isPlayed)
                  .map(episode => (
                    <EpisodeItem key={episode.id} episode={episode} showPodcast={false} />
                  ))}
              </div>
            </div>
          )}
        </div>
      </ScrollArea>

      {/* Now playing bar */}
      {currentEpisode && (
        <div className="flex h-20 items-center justify-between border-t border-border bg-card px-4">
          {/* Episode info */}
          <div className="flex items-center gap-3 w-1/4 min-w-0">
            <img src={resolveArtwork(currentEpisode.podcastCover)} alt="" className="h-14 w-14 rounded-lg object-cover" />
            <div className="min-w-0">
              <p className="truncate font-medium">{currentEpisode.title}</p>
              <p className="truncate text-sm text-muted-foreground">{currentEpisode.podcastTitle}</p>
            </div>
          </div>

          {/* Playback controls */}
          <div className="flex flex-col items-center gap-1 w-2/4">
            <div className="flex items-center gap-2">
              <Button variant="ghost" size="icon" onClick={() => setCurrentTime(t => Math.max(0, t - 15))}>
                <SkipBack className="h-5 w-5" />
              </Button>
              <Button size="icon" className="h-10 w-10 rounded-full" onClick={() => setIsPlaying(!isPlaying)}>
                {isPlaying ? <Pause className="h-5 w-5" /> : <Play className="h-5 w-5 ml-0.5" />}
              </Button>
              <Button variant="ghost" size="icon" onClick={() => setCurrentTime(t => Math.min(currentEpisode.duration, t + 30))}>
                <SkipForward className="h-5 w-5" />
              </Button>
            </div>

            <div className="flex w-full max-w-md items-center gap-2">
              <span className="w-12 text-right text-xs text-muted-foreground">{formatTime(currentTime)}</span>
              <Slider
                value={[currentTime]}
                max={currentEpisode.duration}
                step={1}
                onValueChange={([v]) => setCurrentTime(v)}
                className="flex-1"
              />
              <span className="w-12 text-xs text-muted-foreground">{formatTime(currentEpisode.duration)}</span>
            </div>
          </div>

          {/* Speed & extras */}
          <div className="flex items-center justify-end gap-2 w-1/4">
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm">
                  {playbackSpeed}x
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                {[0.5, 0.75, 1, 1.25, 1.5, 1.75, 2].map(speed => (
                  <DropdownMenuItem
                    key={speed}
                    onClick={() => setPlaybackSpeed(speed)}
                    className={cn(playbackSpeed === speed && "bg-muted")}
                  >
                    {speed}x
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
            <Button variant="ghost" size="icon">
              <ListMusic className="h-5 w-5" />
            </Button>
          </div>
        </div>
      )}

      {/* Add podcast dialog */}
      <Dialog open={showAddDialog} onOpenChange={setShowAddDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>添加播客</DialogTitle>
            <DialogDescription>输入播客的 RSS 订阅地址来添加新播客</DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>RSS 订阅地址</Label>
              <Input
                placeholder="https://example.com/feed.xml"
                value={feedUrl}
                onChange={(e) => setFeedUrl(e.target.value)}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowAddDialog(false)}>取消</Button>
            <Button onClick={() => setShowAddDialog(false)}>添加</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
