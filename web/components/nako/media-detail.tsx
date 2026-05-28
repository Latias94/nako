"use client"

import { useState } from "react"
import { 
  Play, 
  Download, 
  Heart, 
  Share2, 
  ChevronLeft, 
  Star, 
  Clock, 
  Film,
  Tv,
  Volume2,
  Subtitles,
  Info,
  MoreHorizontal,
  Check,
  ChevronDown,
  ChevronRight,
  SkipBack,
  SkipForward,
  Image as ImageIcon,
  User,
  Settings,
  Plus,
  Shuffle,
  HardDrive,
  Cpu,
  FileVideo
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"

// 导航回调类型
interface MediaDetailProps {
  onBack: () => void
  onNavigate?: (type: "person" | "genre" | "tag" | "collection" | "studio", value: string, id?: string) => void
  onPlay?: (mediaId: string, sourceId?: string, episodeId?: string) => void
  onViewImages?: (images: string[]) => void
  mediaType?: "movie" | "series"
}

// 电影数据
const movieData = {
  id: "1",
  type: "movie" as const,
  title: "沙丘2",
  originalTitle: "Dune: Part Two",
  year: 2024,
  rating: 8.6,
  runtime: 166,
  certification: "PG-13",
  tagline: "传奇延续",
  overview: "保罗·厄崔迪与弗雷曼人联合起来，向那些摧毁他家族的阴谋者展开复仇。他必须在爱人与宇宙命运之间做出抉择，同时努力阻止只有他能预见的可怕未来。",
  poster: "/posters/dune2.jpg",
  backdrop: "/backdrops/dune2-backdrop.jpg",
  genres: [
    { id: "sci-fi", name: "科幻" },
    { id: "adventure", name: "冒险" },
    { id: "drama", name: "剧情" }
  ],
  tags: [
    { id: "epic", name: "史诗" },
    { id: "space", name: "太空" },
    { id: "desert", name: "沙漠" },
    { id: "prophecy", name: "预言" }
  ],
  directors: [
    { id: "d1", name: "丹尼斯·维伦纽瓦", photo: null }
  ],
  writers: [
    { id: "w1", name: "丹尼斯·维伦纽瓦", role: "编剧" },
    { id: "w2", name: "乔·斯派茨", role: "编剧" }
  ],
  cast: [
    { id: "a1", name: "提莫西·查拉梅", character: "保罗·厄崔迪", photo: null },
    { id: "a2", name: "赞达亚", character: "契尼", photo: null },
    { id: "a3", name: "丽贝卡·弗格森", character: "杰西卡夫人", photo: null },
    { id: "a4", name: "乔什·布洛林", character: "葛尼·哈莱克", photo: null },
    { id: "a5", name: "奥斯汀·巴特勒", character: "菲德-罗萨·哈克南", photo: null },
    { id: "a6", name: "弗洛伦丝·皮尤", character: "伊勒琅公主", photo: null },
    { id: "a7", name: "戴夫·巴蒂斯塔", character: "格罗苏·拉班", photo: null },
    { id: "a8", name: "克里斯托弗·沃肯", character: "沙丹四世皇帝", photo: null },
  ],
  studio: { id: "legendary", name: "Legendary Pictures" },
  collection: { id: "dune", name: "沙丘系列" },
  collectionItems: [
    { id: "0", title: "沙丘", year: 2021, available: true, poster: "/posters/dune2.jpg" },
    { id: "1", title: "沙丘2", year: 2024, available: true, current: true, poster: "/posters/dune2.jpg" },
    { id: "2", title: "沙丘: 预言 (筹备中)", year: 2026, available: false, poster: null },
  ],
  sources: [
    {
      id: "1",
      quality: "4K",
      hdr: "Dolby Vision",
      resolution: "3840×2160",
      codec: "HEVC",
      bitrate: "80 Mbps",
      audio: [
        { language: "英语", codec: "TrueHD Atmos", channels: "7.1" },
        { language: "普通话", codec: "AAC", channels: "5.1" },
      ],
      subtitles: ["简体中文", "繁體中文", "English", "日本語"],
      fileSize: "65.2 GB",
      container: "MKV"
    },
    {
      id: "2", 
      quality: "1080p",
      hdr: null,
      resolution: "1920×1080",
      codec: "H.264",
      bitrate: "15 Mbps",
      audio: [
        { language: "英语", codec: "AC3", channels: "5.1" },
      ],
      subtitles: ["简体中文", "English"],
      fileSize: "12.8 GB",
      container: "MKV"
    }
  ],
  watched: false,
  favorite: false,
  watchProgress: 0,
}

// 剧集数据
const seriesData = {
  id: "2",
  type: "series" as const,
  title: "真探",
  originalTitle: "True Detective",
  year: 2014,
  endYear: null,
  rating: 8.9,
  certification: "TV-MA",
  tagline: "时间是一个扁平的圆",
  overview: "这部美剧以独立单元剧的形式，每季讲述一个独立的犯罪故事。每季都有不同的角色、设定和故事线，但都围绕着美国的凶杀案展开，探讨人性、存在主义和道德的灰色地带。",
  poster: "/posters/true-detective.jpg",
  backdrop: "/backdrops/true-detective-backdrop.jpg",
  genres: [
    { id: "crime", name: "犯罪" },
    { id: "drama", name: "剧情" },
    { id: "mystery", name: "悬疑" }
  ],
  tags: [
    { id: "anthology", name: "单元剧" },
    { id: "detective", name: "侦探" },
    { id: "dark", name: "黑暗" },
  ],
  creators: [
    { id: "c1", name: "尼克·皮佐拉托", role: "创作者" }
  ],
  cast: [
    { id: "a1", name: "马修·麦康纳", character: "拉斯特·科尔", photo: null, seasons: [1] },
    { id: "a2", name: "伍迪·哈里森", character: "马蒂·哈特", photo: null, seasons: [1] },
    { id: "a3", name: "科林·法瑞尔", character: "雷·维尔科罗", photo: null, seasons: [2] },
    { id: "a4", name: "朱迪·福斯特", character: "利兹·丹弗斯", photo: null, seasons: [4] },
  ],
  studio: { id: "hbo", name: "HBO" },
  network: "HBO",
  seasons: [
    {
      id: "s1",
      number: 1,
      title: "第一季",
      year: 2014,
      episodeCount: 8,
      overview: "1995年，路易斯安那州侦探拉斯特·科尔和马蒂·哈特被指派调查一起仪式化谋杀案。17年后，此案再次浮出水面...",
      episodes: [
        { id: "e1", number: 1, title: "漫漫长夜", runtime: 58, overview: "2012年，侦探们被要求回顾1995年的一起谋杀案...", watched: true, progress: 100 },
        { id: "e2", number: 2, title: "目睹", runtime: 59, overview: "拉斯特和马蒂追踪一个线索到一个偏远的教堂...", watched: true, progress: 100 },
        { id: "e3", number: 3, title: "被锁之门", runtime: 55, overview: "调查陷入僵局，拉斯特开始独自行动...", watched: true, progress: 100 },
        { id: "e4", number: 4, title: "什么是谁", runtime: 57, overview: "一次卧底行动揭露了更大的阴谋...", watched: false, progress: 45 },
        { id: "e5", number: 5, title: "秘密命运", runtime: 56, watched: false, progress: 0 },
        { id: "e6", number: 6, title: "困扰", runtime: 58, watched: false, progress: 0 },
        { id: "e7", number: 7, title: "重新审视", runtime: 55, watched: false, progress: 0 },
        { id: "e8", number: 8, title: "形式与虚空", runtime: 60, watched: false, progress: 0 },
      ]
    },
    {
      id: "s2",
      number: 2,
      title: "第二季",
      year: 2015,
      episodeCount: 8,
      overview: "三名执法人员和一名罪犯必须在一起令人费解的谋杀案中互相合作...",
      episodes: Array.from({ length: 8 }, (_, i) => ({
        id: `s2e${i + 1}`,
        number: i + 1,
        title: `第${i + 1}集`,
        runtime: 55,
        watched: false,
        progress: 0
      }))
    },
  ],
  totalEpisodes: 30,
  watchedEpisodes: 3,
  favorite: false,
}

export function MediaDetail({ 
  onBack, 
  onNavigate, 
  onPlay,
  onViewImages,
  mediaType = "movie" 
}: MediaDetailProps) {
  const isMovie = mediaType === "movie"
  const data = isMovie ? movieData : seriesData
  
  const [selectedSource, setSelectedSource] = useState(isMovie ? movieData.sources[0] : null)
  const [isFavorite, setIsFavorite] = useState(data.favorite)
  const [isInList, setIsInList] = useState(false)
  const [selectedSeason, setSelectedSeason] = useState(!isMovie ? seriesData.seasons[0] : null)
  const [expandedEpisode, setExpandedEpisode] = useState<string | null>(null)

  // 处理标签点击
  const handleTagClick = (type: "person" | "genre" | "tag" | "collection" | "studio", value: string, id?: string) => {
    if (onNavigate) {
      onNavigate(type, value, id)
    }
  }

  // 继续观看的集数
  const nextEpisode = !isMovie && selectedSeason 
    ? selectedSeason.episodes.find(ep => !ep.watched || (ep.progress > 0 && ep.progress < 100))
    : null

  // 格式化时长
  const formatRuntime = (minutes: number) => {
    const hours = Math.floor(minutes / 60)
    const mins = minutes % 60
    return hours > 0 ? `${hours}小时${mins}分钟` : `${mins}分钟`
  }

  return (
    <div className="min-h-screen bg-background">
      {/* Hero Section - 全屏沉浸式背景 */}
      <div className="relative">
        {/* 背景图片 - 全宽 */}
        <div className="absolute inset-0 h-[70vh] min-h-[500px] lg:h-[75vh]">
          <img 
            src={data.backdrop} 
            alt=""
            className="h-full w-full object-cover object-top"
          />
          {/* 多层渐变遮罩 - Netflix 风格 */}
          <div className="absolute inset-0 bg-gradient-to-t from-background via-background/60 to-transparent" />
          <div className="absolute inset-0 bg-gradient-to-r from-background/95 via-background/50 to-transparent lg:from-background/80" />
          <div className="absolute inset-x-0 bottom-0 h-32 bg-gradient-to-t from-background to-transparent" />
        </div>

        {/* 返回按钮 - 固定位置 */}
        <div className="absolute left-4 top-4 z-20 lg:left-8 lg:top-6">
          <Button
            variant="ghost"
            size="icon"
            onClick={onBack}
            className="h-10 w-10 rounded-full bg-black/40 text-white backdrop-blur-sm hover:bg-black/60 hover:text-white"
          >
            <ChevronLeft className="h-5 w-5" />
          </Button>
        </div>

        {/* 主内容区 */}
        <div className="relative z-10 flex min-h-[70vh] items-end pb-8 lg:min-h-[75vh] lg:pb-12">
          <div className="w-full px-4 lg:px-12">
            <div className="mx-auto max-w-7xl">
              <div className="flex flex-col gap-6 lg:flex-row lg:items-end lg:gap-10">
                
                {/* 海报 - 左侧 */}
                <div className="hidden flex-shrink-0 lg:block">
                  <div className="relative w-52 overflow-hidden rounded-lg shadow-2xl ring-1 ring-white/10 xl:w-60">
                    <div className="aspect-[2/3]">
                      <img 
                        src={data.poster} 
                        alt={data.title}
                        className="h-full w-full object-cover"
                      />
                    </div>
                    {/* 品质标签 */}
                    {isMovie && selectedSource && (
                      <div className="absolute bottom-0 inset-x-0 bg-gradient-to-t from-black/80 to-transparent p-3">
                        <div className="flex items-center gap-2">
                          <Badge className="bg-primary text-[10px] font-bold">{selectedSource.quality}</Badge>
                          {selectedSource.hdr && (
                            <Badge variant="outline" className="border-white/30 text-[10px] text-white">{selectedSource.hdr}</Badge>
                          )}
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {/* 信息区 - 右侧 */}
                <div className="flex-1 space-y-4 lg:space-y-5">
                  {/* 标签行 */}
                  <div className="flex flex-wrap items-center gap-2">
                    <Badge className="bg-primary/90 text-xs">
                      {isMovie ? "电影" : "剧集"}
                    </Badge>
                    {!isMovie && (
                      <Badge variant="outline" className="border-white/20 text-xs text-white/80">
                        {seriesData.network}
                      </Badge>
                    )}
                    {isMovie && movieData.collection && (
                      <Badge 
                        variant="outline" 
                        className="cursor-pointer border-white/20 text-xs text-white/80 transition-colors hover:bg-white/10"
                        onClick={() => handleTagClick("collection", movieData.collection.name, movieData.collection.id)}
                      >
                        {movieData.collection.name}
                      </Badge>
                    )}
                  </div>

                  {/* 标题 */}
                  <div>
                    <h1 className="text-balance text-3xl font-bold text-white drop-shadow-lg lg:text-5xl xl:text-6xl">
                      {data.title}
                    </h1>
                    {data.originalTitle !== data.title && (
                      <p className="mt-2 text-base text-white/60 lg:text-lg">{data.originalTitle}</p>
                    )}
                  </div>

                  {/* 元信息 */}
                  <div className="flex flex-wrap items-center gap-x-4 gap-y-2 text-sm text-white/80">
                    <div className="flex items-center gap-1.5">
                      <Star className="h-4 w-4 fill-warning text-warning" />
                      <span className="font-semibold text-white">{data.rating}</span>
                    </div>
                    <span className="text-white/40">|</span>
                    <span>{data.year}{!isMovie && (seriesData.endYear ? ` - ${seriesData.endYear}` : " - 至今")}</span>
                    <span className="text-white/40">|</span>
                    {isMovie ? (
                      <span>{formatRuntime(movieData.runtime)}</span>
                    ) : (
                      <span>{seriesData.seasons.length} 季</span>
                    )}
                    <span className="text-white/40">|</span>
                    <Badge variant="outline" className="border-white/30 text-[11px] text-white/80">
                      {data.certification}
                    </Badge>
                  </div>

                  {/* 类型标签 */}
                  <div className="flex flex-wrap gap-2">
                    {data.genres.map((genre) => (
                      <button 
                        key={genre.id}
                        className="rounded-full bg-white/10 px-3 py-1 text-sm text-white/90 backdrop-blur-sm transition-colors hover:bg-white/20"
                        onClick={() => handleTagClick("genre", genre.name, genre.id)}
                      >
                        {genre.name}
                      </button>
                    ))}
                  </div>

                  {/* 简介 - 直接显示 */}
                  <p className="max-w-2xl text-sm leading-relaxed text-white/70 lg:text-base">
                    {data.overview}
                  </p>

                  {/* 主要演员 - 内联显示 */}
                  <div className="text-sm text-white/60">
                    <span className="text-white/40">主演：</span>
                    {(isMovie ? movieData.cast : seriesData.cast).slice(0, 4).map((person, i) => (
                      <span key={person.id}>
                        <button 
                          className="text-white/80 underline-offset-2 hover:text-white hover:underline"
                          onClick={() => handleTagClick("person", person.name, person.id)}
                        >
                          {person.name}
                        </button>
                        {i < 3 && <span className="text-white/40">、</span>}
                      </span>
                    ))}
                    {(isMovie ? movieData.cast : seriesData.cast).length > 4 && (
                      <span className="text-white/40"> 等</span>
                    )}
                  </div>

                  {/* 操作按钮 */}
                  <div className="flex flex-wrap items-center gap-3 pt-2">
                    <Button 
                      size="lg" 
                      className="h-12 gap-2 px-8 text-base font-semibold"
                      onClick={() => onPlay?.(data.id, selectedSource?.id, nextEpisode?.id)}
                    >
                      <Play className="h-5 w-5 fill-current" />
                      {!isMovie && nextEpisode ? (
                        nextEpisode.progress > 0 ? "继续观看" : `播放 S${selectedSeason?.number}E${nextEpisode.number}`
                      ) : (
                        movieData.watchProgress > 0 ? "继续播放" : "播放"
                      )}
                    </Button>
                    
                    <Button 
                      variant="outline" 
                      size="lg"
                      className={cn(
                        "h-12 gap-2 border-white/20 bg-white/10 text-white backdrop-blur-sm hover:bg-white/20 hover:text-white",
                        isInList && "border-primary/50 bg-primary/20 text-primary-foreground"
                      )}
                      onClick={() => setIsInList(!isInList)}
                    >
                      {isInList ? (
                        <>
                          <Check className="h-5 w-5" />
                          已添加
                        </>
                      ) : (
                        <>
                          <Plus className="h-5 w-5" />
                          我的列表
                        </>
                      )}
                    </Button>

                    <Button 
                      variant="ghost" 
                      size="icon"
                      className={cn(
                        "h-12 w-12 rounded-full border border-white/20 text-white hover:bg-white/20 hover:text-white",
                        isFavorite && "border-red-500/50 bg-red-500/20 text-red-400"
                      )}
                      onClick={() => setIsFavorite(!isFavorite)}
                    >
                      <Heart className={cn("h-5 w-5", isFavorite && "fill-current")} />
                    </Button>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button 
                          variant="ghost" 
                          size="icon"
                          className="h-12 w-12 rounded-full border border-white/20 text-white hover:bg-white/20 hover:text-white"
                        >
                          <MoreHorizontal className="h-5 w-5" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="start" className="w-48">
                        <DropdownMenuItem>
                          <Download className="mr-2 h-4 w-4" />
                          下载
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Share2 className="mr-2 h-4 w-4" />
                          分享
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Shuffle className="mr-2 h-4 w-4" />
                          随机播放
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem>
                          <Check className="mr-2 h-4 w-4" />
                          {isMovie ? "标记为已看" : "全部标记为已看"}
                        </DropdownMenuItem>
                        <DropdownMenuItem>
                          <Settings className="mr-2 h-4 w-4" />
                          编辑元数据
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 下方内容区 */}
      <div className="relative z-10 bg-background">
        <div className="mx-auto max-w-7xl px-4 py-8 lg:px-12">
          
          {/* 剧集选择器 - 仅剧集显示 */}
          {!isMovie && (
            <div className="mb-8">
              <div className="mb-4 flex items-center justify-between">
                <h2 className="text-xl font-semibold">剧集</h2>
                <Select 
                  value={selectedSeason?.id} 
                  onValueChange={(value) => setSelectedSeason(seriesData.seasons.find(s => s.id === value) || null)}
                >
                  <SelectTrigger className="w-40">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {seriesData.seasons.map((season) => (
                      <SelectItem key={season.id} value={season.id}>
                        {season.title}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* 剧集网格 - Netflix 风格 */}
              <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                {selectedSeason?.episodes.map((episode) => (
                  <div 
                    key={episode.id}
                    className="group cursor-pointer overflow-hidden rounded-lg bg-card transition-all hover:bg-card/80 hover:ring-1 hover:ring-primary/50"
                    onClick={() => onPlay?.(data.id, undefined, episode.id)}
                  >
                    {/* 缩略图 */}
                    <div className="relative aspect-video bg-muted">
                      <div className="absolute inset-0 flex items-center justify-center">
                        <Tv className="h-8 w-8 text-muted-foreground/50" />
                      </div>
                      {/* 播放进度 */}
                      {episode.progress > 0 && episode.progress < 100 && (
                        <div className="absolute inset-x-0 bottom-0 h-1 bg-white/20">
                          <div className="h-full bg-primary" style={{ width: `${episode.progress}%` }} />
                        </div>
                      )}
                      {/* 已看标记 */}
                      {episode.watched && (
                        <div className="absolute right-2 top-2 rounded-full bg-primary p-1">
                          <Check className="h-3 w-3" />
                        </div>
                      )}
                      {/* 播放悬浮 */}
                      <div className="absolute inset-0 flex items-center justify-center bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                        <div className="rounded-full bg-white p-3">
                          <Play className="h-6 w-6 fill-black text-black" />
                        </div>
                      </div>
                      {/* 时长 */}
                      <div className="absolute bottom-2 right-2 rounded bg-black/70 px-1.5 py-0.5 text-xs text-white">
                        {episode.runtime}分钟
                      </div>
                    </div>
                    {/* 信息 */}
                    <div className="p-3">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-medium text-muted-foreground">{episode.number}.</span>
                        <h4 className="truncate text-sm font-medium">{episode.title}</h4>
                      </div>
                      {episode.overview && (
                        <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{episode.overview}</p>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* 详细信息区块 */}
          <div className="grid gap-8 lg:grid-cols-3">
            {/* 左侧：演员和制作人员 */}
            <div className="lg:col-span-2 space-y-8">
              {/* 演员 */}
              <section>
                <h3 className="mb-4 text-lg font-semibold">演员阵容</h3>
                <div className="grid grid-cols-2 gap-3 sm:grid-cols-3 md:grid-cols-4">
                  {(isMovie ? movieData.cast : seriesData.cast).map((person) => (
                    <button
                      key={person.id}
                      className="flex items-center gap-3 rounded-lg p-2 text-left transition-colors hover:bg-secondary"
                      onClick={() => handleTagClick("person", person.name, person.id)}
                    >
                      <div className="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-full bg-muted">
                        <User className="h-5 w-5 text-muted-foreground" />
                      </div>
                      <div className="min-w-0">
                        <p className="truncate text-sm font-medium">{person.name}</p>
                        <p className="truncate text-xs text-muted-foreground">{person.character}</p>
                      </div>
                    </button>
                  ))}
                </div>
              </section>

              {/* 更多详情 */}
              <section>
                <h3 className="mb-4 text-lg font-semibold">更多信息</h3>
                <div className="grid gap-4 sm:grid-cols-2">
                  <div>
                    <h4 className="text-sm text-muted-foreground">导演</h4>
                    <div className="mt-1 flex flex-wrap gap-2">
                      {(isMovie ? movieData.directors : seriesData.creators).map((person) => (
                        <button
                          key={person.id}
                          className="text-sm underline-offset-2 hover:underline"
                          onClick={() => handleTagClick("person", person.name, person.id)}
                        >
                          {person.name}
                        </button>
                      ))}
                    </div>
                  </div>
                  <div>
                    <h4 className="text-sm text-muted-foreground">制片公司</h4>
                    <button
                      className="mt-1 text-sm underline-offset-2 hover:underline"
                      onClick={() => handleTagClick("studio", data.studio.name, data.studio.id)}
                    >
                      {data.studio.name}
                    </button>
                  </div>
                  <div>
                    <h4 className="text-sm text-muted-foreground">标签</h4>
                    <div className="mt-1 flex flex-wrap gap-1">
                      {data.tags.map((tag) => (
                        <Badge
                          key={tag.id}
                          variant="outline"
                          className="cursor-pointer text-xs hover:bg-secondary"
                          onClick={() => handleTagClick("tag", tag.name, tag.id)}
                        >
                          {tag.name}
                        </Badge>
                      ))}
                    </div>
                  </div>
                </div>
              </section>
            </div>

            {/* 右侧：媒体源信息 - 自部署特色 */}
            {isMovie && (
              <div className="space-y-6">
                <section>
                  <h3 className="mb-4 text-lg font-semibold">媒体源</h3>
                  <div className="space-y-3">
                    {movieData.sources.map((source) => (
                      <div
                        key={source.id}
                        className={cn(
                          "cursor-pointer rounded-lg border p-4 transition-all",
                          selectedSource?.id === source.id 
                            ? "border-primary bg-primary/5" 
                            : "border-border hover:border-primary/50"
                        )}
                        onClick={() => setSelectedSource(source)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <Badge className={cn(
                              "font-bold",
                              source.quality === "4K" ? "bg-primary" : "bg-secondary text-secondary-foreground"
                            )}>
                              {source.quality}
                            </Badge>
                            {source.hdr && (
                              <Badge variant="outline" className="text-[10px]">{source.hdr}</Badge>
                            )}
                          </div>
                          <span className="text-sm text-muted-foreground">{source.fileSize}</span>
                        </div>
                        <div className="mt-3 grid grid-cols-2 gap-2 text-xs text-muted-foreground">
                          <div className="flex items-center gap-1.5">
                            <Cpu className="h-3.5 w-3.5" />
                            <span>{source.codec}</span>
                          </div>
                          <div className="flex items-center gap-1.5">
                            <FileVideo className="h-3.5 w-3.5" />
                            <span>{source.container}</span>
                          </div>
                          <div className="flex items-center gap-1.5">
                            <Volume2 className="h-3.5 w-3.5" />
                            <span>{source.audio.length} 音轨</span>
                          </div>
                          <div className="flex items-center gap-1.5">
                            <Subtitles className="h-3.5 w-3.5" />
                            <span>{source.subtitles.length} 字幕</span>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </section>

                {/* 系列合集 */}
                {movieData.collection && (
                  <section>
                    <h3 className="mb-4 text-lg font-semibold">{movieData.collection.name}</h3>
                    <div className="space-y-2">
                      {movieData.collectionItems.map((item) => (
                        <div
                          key={item.id}
                          className={cn(
                            "flex items-center gap-3 rounded-lg p-2 transition-colors",
                            item.current ? "bg-primary/10" : item.available ? "hover:bg-secondary cursor-pointer" : "opacity-50"
                          )}
                        >
                          <div className="h-16 w-11 flex-shrink-0 overflow-hidden rounded bg-muted">
                            {item.poster ? (
                              <img src={item.poster} alt={item.title} className="h-full w-full object-cover" />
                            ) : (
                              <div className="flex h-full w-full items-center justify-center">
                                <Film className="h-4 w-4 text-muted-foreground" />
                              </div>
                            )}
                          </div>
                          <div className="min-w-0 flex-1">
                            <p className={cn("truncate text-sm font-medium", item.current && "text-primary")}>
                              {item.title}
                            </p>
                            <p className="text-xs text-muted-foreground">{item.year}</p>
                          </div>
                          {item.current && (
                            <Badge variant="secondary" className="text-[10px]">当前</Badge>
                          )}
                        </div>
                      ))}
                    </div>
                  </section>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  )
}
