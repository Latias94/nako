"use client"

import { useState } from "react"
import { 
  Play, Heart, BookmarkPlus, Share2, Star, Calendar, Clock, 
  Users, Building2, Globe, ChevronRight, ExternalLink, Check,
  Tv, Film, Music, Sparkles
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import { ScrollArea, ScrollBar } from "@/components/ui/scroll-area"
import { Progress } from "@/components/ui/progress"
import { cn } from "@/lib/utils"
import { EpisodeSelector, type EpisodeGroup, type ContentType } from "./episode-selector"

// UI 预设类型
export type UIPreset = "default" | "anime" | "movie" | "music" | "documentary" | "minimal"

// UI 预设配置
export const UI_PRESETS: Record<UIPreset, {
  name: string
  description: string
  icon: typeof Film
  accentColor: string
  features: string[]
}> = {
  default: {
    name: "默认",
    description: "适用于大多数媒体内容的通用布局",
    icon: Film,
    accentColor: "primary",
    features: ["标准海报布局", "剧集列表", "演职员信息"],
  },
  anime: {
    name: "动画",
    description: "专为动漫设计的界面，突出季/期、声优信息",
    icon: Sparkles,
    accentColor: "pink",
    features: ["番剧信息卡", "声优列表", "季度/期数导航", "关联作品"],
  },
  movie: {
    name: "电影",
    description: "影院风格布局，突出视觉效果和技术规格",
    icon: Film,
    accentColor: "amber",
    features: ["全屏背景", "技术规格", "系列电影导航"],
  },
  music: {
    name: "音乐",
    description: "专为音乐视频和演唱会设计",
    icon: Music,
    accentColor: "green",
    features: ["专辑封面", "曲目列表", "艺术家信息"],
  },
  documentary: {
    name: "纪录片",
    description: "信息密集型布局，突出主题和集数",
    icon: Tv,
    accentColor: "blue",
    features: ["主题标签", "分集介绍", "相关资源"],
  },
  minimal: {
    name: "简约",
    description: "简洁的播放界面，减少干扰",
    icon: Play,
    accentColor: "neutral",
    features: ["大播放按钮", "基础信息", "快速导航"],
  },
}

// 动漫详情数据
interface AnimeDetailData {
  id: string
  title: string
  originalTitle?: string  // 日文原名
  englishTitle?: string   // 英文名
  poster: string
  backdrop?: string
  bannerImage?: string    // 横幅图
  rating: number
  userRating?: number     // 用户评分
  year: number
  season?: string         // 播出季度 如 "2024年春"
  status: "airing" | "finished" | "upcoming" | "hiatus"
  type: "tv" | "movie" | "ova" | "ona" | "special"
  episodeCount?: number
  episodeDuration?: number // 单集时长
  genres: string[]
  tags?: string[]         // 更详细的标签
  studios: string[]
  source?: string         // 原作类型 如 "漫画" "轻小说" "游戏" "原创"
  overview: string
  
  // 动漫特有
  voiceActors?: Array<{
    id: string
    name: string
    character: string
    characterImage?: string
    photo?: string
  }>
  staff?: Array<{
    id: string
    name: string
    role: string
    photo?: string
  }>
  relations?: Array<{
    id: string
    title: string
    poster: string
    type: string  // "续集" "前传" "外传" "衍生"
    year: number
  }>
  
  // 外部链接
  externalLinks?: Array<{
    site: string
    url: string
    icon?: string
  }>
  
  // 播放数据
  episodes: EpisodeGroup[]
  watchProgress?: {
    watched: number
    total: number
    lastWatchedEpisode?: string
  }
  
  favorite: boolean
  inList: boolean
}

// Mock 动漫数据
const mockAnimeData: AnimeDetailData = {
  id: "anime-1",
  title: "葬送的芙莉莲",
  originalTitle: "葬送のフリーレン",
  englishTitle: "Frieren: Beyond Journey's End",
  poster: "https://image.tmdb.org/t/p/w500/dqZENchTd7lp5zht7BdlqFWjk6H.jpg",
  backdrop: "https://image.tmdb.org/t/p/original/rUunhF0rKaUJLzBj0wvKrczwqhA.jpg",
  rating: 9.1,
  userRating: 9.5,
  year: 2023,
  season: "2023年秋",
  status: "airing",
  type: "tv",
  episodeCount: 28,
  episodeDuration: 24,
  genres: ["奇幻", "冒险", "剧情"],
  tags: ["魔法", "精灵", "勇者", "旅途", "治愈"],
  studios: ["MADHOUSE"],
  source: "漫画",
  overview: "魔王被勇者一行打倒，世界恢复了和平。精灵魔法使芙莉莲与勇者欣梅尔等人分别后，独自踏上了旅途。50年后，芙莉莲与已经年迈的欣梅尔重逢，送别了他的最后一程。在那之后，芙莉莲开始回顾与欣梅尔等人的回忆，踏上了「了解人类」的旅途。",
  voiceActors: [
    { id: "va-1", name: "种崎敦美", character: "芙莉莲", characterImage: "https://via.placeholder.com/100" },
    { id: "va-2", name: "�的井美树", character: "菲伦", characterImage: "https://via.placeholder.com/100" },
    { id: "va-3", name: "小林千晃", character: "修塔尔克", characterImage: "https://via.placeholder.com/100" },
    { id: "va-4", name: "�的浪和也", character: "欣梅尔", characterImage: "https://via.placeholder.com/100" },
  ],
  staff: [
    { id: "st-1", name: "�的�的满", role: "监督" },
    { id: "st-2", name: "�的木昭人", role: "系列构成" },
    { id: "st-3", name: "Evan Call", role: "音乐" },
  ],
  relations: [
    { id: "rel-1", title: "葬送的芙莉莲 第二季", poster: "https://via.placeholder.com/200x300", type: "续集", year: 2025 },
  ],
  externalLinks: [
    { site: "官方网站", url: "https://frieren-anime.jp/" },
    { site: "AniList", url: "https://anilist.co/anime/154587" },
    { site: "MyAnimeList", url: "https://myanimelist.net/anime/52991" },
  ],
  episodes: [
    {
      id: "cour-1",
      type: "cour",
      number: 1,
      title: "第一期",
      subtitle: "勇者派对篇",
      totalEpisodes: 16,
      watchedCount: 12,
      year: 2023,
      episodes: Array.from({ length: 16 }, (_, i) => ({
        id: `ep-${i + 1}`,
        number: i + 1,
        title: `第${i + 1}话`,
        duration: 24,
        watched: i < 12,
        progress: i === 11 ? 65 : undefined,
      })),
    },
    {
      id: "cour-2",
      type: "cour",
      number: 2,
      title: "第二期",
      subtitle: "一级魔法使篇",
      totalEpisodes: 12,
      watchedCount: 0,
      year: 2024,
      episodes: Array.from({ length: 12 }, (_, i) => ({
        id: `ep-${i + 17}`,
        number: i + 17,
        title: `第${i + 17}话`,
        duration: 24,
        watched: false,
      })),
    },
  ],
  watchProgress: {
    watched: 12,
    total: 28,
    lastWatchedEpisode: "ep-12",
  },
  favorite: true,
  inList: true,
}

// 状态标签映射
const statusLabels: Record<string, { label: string; variant: "default" | "secondary" | "destructive" | "outline" }> = {
  airing: { label: "放送中", variant: "default" },
  finished: { label: "已完结", variant: "secondary" },
  upcoming: { label: "即将放送", variant: "outline" },
  hiatus: { label: "暂停放送", variant: "destructive" },
}

const typeLabels: Record<string, string> = {
  tv: "TV动画",
  movie: "剧场版",
  ova: "OVA",
  ona: "ONA",
  special: "特别篇",
}

interface AnimeDetailPageProps {
  data?: AnimeDetailData
  onBack?: () => void
  onPlay?: (episodeId?: string) => void
  onSelectPerson?: (name: string, id: string) => void
  onSelectRelation?: (id: string) => void
}

export function AnimeDetailPage({
  data = mockAnimeData,
  onBack,
  onPlay,
  onSelectPerson,
  onSelectRelation,
}: AnimeDetailPageProps) {
  const [isFavorite, setIsFavorite] = useState(data.favorite)
  const [isInList, setIsInList] = useState(data.inList)
  const [activeTab, setActiveTab] = useState("episodes")

  const statusInfo = statusLabels[data.status] || statusLabels.finished
  const progress = data.watchProgress 
    ? (data.watchProgress.watched / data.watchProgress.total) * 100 
    : 0

  return (
    <div className="min-h-screen bg-background">
      {/* Hero Section with Anime Style */}
      <div className="relative">
        {/* Background */}
        <div className="absolute inset-0 h-[450px] overflow-hidden">
          <img
            src={data.backdrop || data.poster}
            alt=""
            className="h-full w-full object-cover"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-background via-background/80 to-background/30" />
          <div className="absolute inset-0 bg-gradient-to-r from-background via-transparent to-background/50" />
        </div>

        {/* Content */}
        <div className="relative px-4 pt-16 lg:px-8">
          {/* Back Button */}
          <Button
            variant="ghost"
            size="sm"
            className="mb-4 text-white/80 hover:text-white"
            onClick={onBack}
          >
            <ChevronRight className="mr-1 h-4 w-4 rotate-180" />
            返回
          </Button>

          <div className="flex flex-col gap-6 lg:flex-row lg:gap-8">
            {/* Poster */}
            <div className="relative mx-auto flex-shrink-0 lg:mx-0">
              <img
                src={data.poster}
                alt={data.title}
                className="h-[300px] w-[200px] rounded-xl object-cover shadow-2xl ring-1 ring-white/10 lg:h-[360px] lg:w-[240px]"
              />
              {/* Progress Overlay */}
              {progress > 0 && (
                <div className="absolute bottom-0 left-0 right-0 rounded-b-xl bg-black/60 p-2">
                  <Progress value={progress} className="h-1.5" />
                  <p className="mt-1 text-center text-xs text-white/80">
                    {data.watchProgress?.watched}/{data.watchProgress?.total} 已观看
                  </p>
                </div>
              )}
            </div>

            {/* Info */}
            <div className="flex-1 text-center lg:text-left">
              {/* Badges */}
              <div className="mb-3 flex flex-wrap items-center justify-center gap-2 lg:justify-start">
                <Badge variant={statusInfo.variant}>{statusInfo.label}</Badge>
                <Badge variant="outline">{typeLabels[data.type] || data.type}</Badge>
                {data.season && <Badge variant="secondary">{data.season}</Badge>}
              </div>

              {/* Title */}
              <h1 className="text-2xl font-bold text-white lg:text-3xl">{data.title}</h1>
              {data.originalTitle && (
                <p className="mt-1 text-sm text-white/70">{data.originalTitle}</p>
              )}
              {data.englishTitle && data.englishTitle !== data.title && (
                <p className="text-xs text-white/50">{data.englishTitle}</p>
              )}

              {/* Stats Row */}
              <div className="mt-4 flex flex-wrap items-center justify-center gap-4 text-sm text-white/80 lg:justify-start">
                <span className="flex items-center gap-1">
                  <Star className="h-4 w-4 fill-yellow-500 text-yellow-500" />
                  <span className="font-medium">{data.rating.toFixed(1)}</span>
                </span>
                <span className="flex items-center gap-1">
                  <Calendar className="h-4 w-4" />
                  {data.year}
                </span>
                {data.episodeCount && (
                  <span className="flex items-center gap-1">
                    <Tv className="h-4 w-4" />
                    {data.episodeCount}话
                  </span>
                )}
                {data.episodeDuration && (
                  <span className="flex items-center gap-1">
                    <Clock className="h-4 w-4" />
                    {data.episodeDuration}分钟/话
                  </span>
                )}
              </div>

              {/* Genres & Tags */}
              <div className="mt-4 flex flex-wrap items-center justify-center gap-2 lg:justify-start">
                {data.genres.map((genre) => (
                  <Badge key={genre} variant="secondary" className="bg-pink-500/20 text-pink-300">
                    {genre}
                  </Badge>
                ))}
                {data.source && (
                  <Badge variant="outline" className="border-white/30 text-white/70">
                    原作: {data.source}
                  </Badge>
                )}
              </div>

              {/* Studios */}
              <div className="mt-3 flex items-center justify-center gap-2 text-sm text-white/60 lg:justify-start">
                <Building2 className="h-4 w-4" />
                {data.studios.join(" / ")}
              </div>

              {/* Action Buttons */}
              <div className="mt-6 flex flex-wrap items-center justify-center gap-3 lg:justify-start">
                <Button 
                  size="lg" 
                  className="h-12 gap-2 bg-pink-600 hover:bg-pink-700"
                  onClick={() => onPlay?.(data.watchProgress?.lastWatchedEpisode)}
                >
                  <Play className="h-5 w-5" fill="white" />
                  {data.watchProgress?.watched ? "继续观看" : "开始观看"}
                </Button>
                <Button
                  variant="outline"
                  size="lg"
                  className={cn(
                    "h-12 gap-2 border-white/20 bg-white/10 text-white hover:bg-white/20",
                    isInList && "border-pink-500/50 bg-pink-500/20"
                  )}
                  onClick={() => setIsInList(!isInList)}
                >
                  {isInList ? <Check className="h-5 w-5" /> : <BookmarkPlus className="h-5 w-5" />}
                  {isInList ? "已追番" : "追番"}
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  className={cn(
                    "h-12 w-12 text-white/70 hover:text-white",
                    isFavorite && "text-pink-500 hover:text-pink-400"
                  )}
                  onClick={() => setIsFavorite(!isFavorite)}
                >
                  <Heart className={cn("h-5 w-5", isFavorite && "fill-current")} />
                </Button>
                <Button variant="ghost" size="icon" className="h-12 w-12 text-white/70 hover:text-white">
                  <Share2 className="h-5 w-5" />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Tabs Content */}
      <div className="mt-8 px-4 pb-8 lg:px-8">
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="mb-6">
            <TabsTrigger value="episodes">剧集</TabsTrigger>
            <TabsTrigger value="info">详情</TabsTrigger>
            <TabsTrigger value="cast">声优</TabsTrigger>
            <TabsTrigger value="related">关联作品</TabsTrigger>
          </TabsList>

          {/* Episodes Tab */}
          <TabsContent value="episodes" className="mt-0">
            <div className="rounded-xl border border-border bg-card">
              <EpisodeSelector
                contentType="anime"
                groups={data.episodes}
                currentEpisodeId={data.watchProgress?.lastWatchedEpisode}
                onSelectEpisode={(episodeId) => onPlay?.(episodeId)}
                className="max-h-[600px]"
              />
            </div>
          </TabsContent>

          {/* Info Tab */}
          <TabsContent value="info" className="mt-0">
            <div className="grid gap-6 lg:grid-cols-3">
              {/* Overview */}
              <div className="lg:col-span-2">
                <h3 className="mb-3 text-lg font-semibold">简介</h3>
                <p className="leading-relaxed text-muted-foreground">{data.overview}</p>
                
                {/* Tags */}
                {data.tags && data.tags.length > 0 && (
                  <div className="mt-6">
                    <h4 className="mb-2 text-sm font-medium text-muted-foreground">标签</h4>
                    <div className="flex flex-wrap gap-2">
                      {data.tags.map((tag) => (
                        <Badge key={tag} variant="outline" className="cursor-pointer hover:bg-muted">
                          {tag}
                        </Badge>
                      ))}
                    </div>
                  </div>
                )}
              </div>

              {/* Side Info */}
              <div className="space-y-4 rounded-xl border border-border bg-card p-4">
                <h3 className="font-semibold">作品信息</h3>
                <div className="space-y-3 text-sm">
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">类型</span>
                    <span>{typeLabels[data.type]}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">状态</span>
                    <span>{statusInfo.label}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">首播</span>
                    <span>{data.season || data.year}</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">话数</span>
                    <span>{data.episodeCount}话</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">单集时长</span>
                    <span>{data.episodeDuration}分钟</span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-muted-foreground">制作公司</span>
                    <span>{data.studios.join(", ")}</span>
                  </div>
                  {data.source && (
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">原作</span>
                      <span>{data.source}</span>
                    </div>
                  )}
                </div>

                {/* External Links */}
                {data.externalLinks && data.externalLinks.length > 0 && (
                  <div className="border-t border-border pt-4">
                    <h4 className="mb-2 text-sm font-medium">外部链接</h4>
                    <div className="space-y-2">
                      {data.externalLinks.map((link) => (
                        <a
                          key={link.site}
                          href={link.url}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-2 text-sm text-primary hover:underline"
                        >
                          <ExternalLink className="h-3 w-3" />
                          {link.site}
                        </a>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </TabsContent>

          {/* Cast Tab */}
          <TabsContent value="cast" className="mt-0">
            <div className="space-y-8">
              {/* Voice Actors */}
              {data.voiceActors && data.voiceActors.length > 0 && (
                <section>
                  <h3 className="mb-4 text-lg font-semibold">声优</h3>
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {data.voiceActors.map((va) => (
                      <button
                        key={va.id}
                        className="flex items-center gap-3 rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-muted/50"
                        onClick={() => onSelectPerson?.(va.name, va.id)}
                      >
                        <div className="flex h-12 w-12 items-center justify-center rounded-full bg-muted">
                          {va.photo ? (
                            <img src={va.photo} alt={va.name} className="h-full w-full rounded-full object-cover" />
                          ) : (
                            <Users className="h-5 w-5 text-muted-foreground" />
                          )}
                        </div>
                        <div className="flex-1">
                          <p className="font-medium">{va.name}</p>
                          <p className="text-sm text-muted-foreground">CV: {va.character}</p>
                        </div>
                        {va.characterImage && (
                          <img src={va.characterImage} alt={va.character} className="h-10 w-10 rounded-full object-cover ring-2 ring-background" />
                        )}
                      </button>
                    ))}
                  </div>
                </section>
              )}

              {/* Staff */}
              {data.staff && data.staff.length > 0 && (
                <section>
                  <h3 className="mb-4 text-lg font-semibold">制作人员</h3>
                  <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
                    {data.staff.map((person) => (
                      <button
                        key={person.id}
                        className="flex items-center gap-3 rounded-lg border border-border bg-card p-3 text-left transition-colors hover:bg-muted/50"
                        onClick={() => onSelectPerson?.(person.name, person.id)}
                      >
                        <div className="flex h-10 w-10 items-center justify-center rounded-full bg-muted">
                          {person.photo ? (
                            <img src={person.photo} alt={person.name} className="h-full w-full rounded-full object-cover" />
                          ) : (
                            <Users className="h-4 w-4 text-muted-foreground" />
                          )}
                        </div>
                        <div>
                          <p className="text-sm font-medium">{person.name}</p>
                          <p className="text-xs text-muted-foreground">{person.role}</p>
                        </div>
                      </button>
                    ))}
                  </div>
                </section>
              )}
            </div>
          </TabsContent>

          {/* Related Tab */}
          <TabsContent value="related" className="mt-0">
            {data.relations && data.relations.length > 0 ? (
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
                {data.relations.map((rel) => (
                  <button
                    key={rel.id}
                    className="group overflow-hidden rounded-xl border border-border bg-card text-left transition-all hover:border-primary/50 hover:shadow-lg"
                    onClick={() => onSelectRelation?.(rel.id)}
                  >
                    <div className="relative aspect-[2/3] overflow-hidden">
                      <img
                        src={rel.poster}
                        alt={rel.title}
                        className="h-full w-full object-cover transition-transform group-hover:scale-105"
                      />
                      <Badge className="absolute left-2 top-2 bg-black/70">{rel.type}</Badge>
                    </div>
                    <div className="p-3">
                      <p className="font-medium">{rel.title}</p>
                      <p className="text-sm text-muted-foreground">{rel.year}</p>
                    </div>
                  </button>
                ))}
              </div>
            ) : (
              <div className="py-12 text-center text-muted-foreground">
                暂无关联作品
              </div>
            )}
          </TabsContent>
        </Tabs>
      </div>
    </div>
  )
}
