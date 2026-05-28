"use client"

import { useState } from "react"
import { 
  ChevronLeft, 
  Star, 
  Film, 
  Tv,
  Play,
  Grid3X3,
  List,
  SlidersHorizontal,
  User,
  Tag,
  Clapperboard,
  Building2,
  Layers
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  DropdownMenuCheckboxItem,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"

type FilterType = "person" | "genre" | "tag" | "collection" | "studio"

interface FilterPageProps {
  onBack: () => void
  onSelectMedia?: (mediaId: string) => void
  filterType?: FilterType
  filterValue?: string
  filterId?: string
  onApplyFilters?: (filters: unknown) => void
}

// 人物数据
const personData = {
  id: "a1",
  name: "提莫西·查拉梅",
  originalName: "Timothée Chalamet",
  photo: null,
  birthDate: "1995-12-27",
  birthPlace: "纽约市, 美国",
  biography: "提莫西·哈尔·查拉梅是一位美国演员。他开始在多部电视作品中演出，之后在剧情片《星际穿越》中扮演一个配角。他因主演成长电影《请以你的名字呼唤我》而获得广泛认可，并获得奥斯卡最佳男主角提名。",
  roles: ["演员"],
  knownFor: [
    { id: "1", title: "沙丘2", year: 2024, character: "保罗·厄崔迪", rating: 8.6, type: "movie" },
    { id: "2", title: "沙丘", year: 2021, character: "保罗·厄崔迪", rating: 8.0, type: "movie" },
    { id: "3", title: "旺卡", year: 2023, character: "威利·旺卡", rating: 7.1, type: "movie" },
    { id: "4", title: "请以你的名字呼唤我", year: 2017, character: "艾利欧", rating: 7.9, type: "movie" },
    { id: "5", title: "小妇人", year: 2019, character: "劳里", rating: 7.8, type: "movie" },
    { id: "6", title: "星际穿越", year: 2014, character: "汤姆·库珀 (年轻时)", rating: 8.7, type: "movie" },
  ],
  filmography: {
    actor: [
      { id: "1", title: "沙丘2", year: 2024, character: "保罗·厄崔迪", type: "movie" },
      { id: "2", title: "旺卡", year: 2023, character: "威利·旺卡", type: "movie" },
      { id: "3", title: "沙丘", year: 2021, character: "保罗·厄崔迪", type: "movie" },
      { id: "4", title: "法兰西特派", year: 2021, character: "泽弗雷利", type: "movie" },
      { id: "5", title: "小妇人", year: 2019, character: "劳里", type: "movie" },
      { id: "6", title: "漂亮男孩", year: 2018, character: "尼克·谢夫", type: "movie" },
      { id: "7", title: "请以你的名字呼唤我", year: 2017, character: "艾利欧", type: "movie" },
      { id: "8", title: "伯德小姐", year: 2017, character: "凯尔", type: "movie" },
      { id: "9", title: "星际穿越", year: 2014, character: "汤姆 (年轻时)", type: "movie" },
    ]
  }
}

// 分类/标签数据
const genreData = {
  id: "sci-fi",
  name: "科幻",
  description: "科幻类作品通常探索科学与技术对社会和个人的影响，涉及太空探索、时间旅行、人工智能、外星生命等主题。",
  mediaCount: 234,
  media: [
    { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie", quality: "4K Dolby Vision" },
    { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie", quality: "4K HDR" },
    { id: "3", title: "星际穿越", year: 2014, rating: 8.7, type: "movie", quality: "4K" },
    { id: "4", title: "银翼杀手 2049", year: 2017, rating: 8.0, type: "movie", quality: "4K Dolby Vision" },
    { id: "5", title: "降临", year: 2016, rating: 7.9, type: "movie", quality: "4K HDR" },
    { id: "6", title: "头号玩家", year: 2018, rating: 7.4, type: "movie", quality: "4K" },
    { id: "7", title: "黑镜", year: 2011, rating: 8.7, type: "series", quality: "1080p", seasons: 6 },
    { id: "8", title: "爱，死亡和机器人", year: 2019, rating: 8.4, type: "series", quality: "4K", seasons: 3 },
    { id: "9", title: "西部世界", year: 2016, rating: 8.5, type: "series", quality: "4K HDR", seasons: 4 },
    { id: "10", title: "基地", year: 2021, rating: 7.2, type: "series", quality: "4K Dolby Vision", seasons: 2 },
  ]
}

// 系列数据
const collectionData = {
  id: "dune",
  name: "沙丘系列",
  description: "根据弗兰克·赫伯特的同名科幻小说改编的系列电影，讲述了保罗·厄崔迪在沙漠星球厄拉科斯上的史诗故事。",
  overview: "由丹尼斯·维伦纽瓦执导的沙丘系列电影，以其宏大的视觉效果和深刻的主题探讨受到广泛好评。",
  media: [
    { id: "0", title: "沙丘", year: 2021, rating: 8.0, type: "movie", quality: "4K HDR", runtime: 155 },
    { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie", quality: "4K Dolby Vision", runtime: 166 },
  ],
  upcoming: [
    { title: "沙丘: 预言", year: 2026, type: "movie" }
  ]
}

// 制作公司数据
const studioData = {
  id: "legendary",
  name: "Legendary Pictures",
  description: "传奇影业是一家美国电影制作公司，以制作大制作科幻和奇幻电影而闻名。",
  founded: 2000,
  mediaCount: 45,
  media: [
    { id: "1", title: "沙丘2", year: 2024, rating: 8.6, type: "movie", quality: "4K Dolby Vision" },
    { id: "2", title: "沙丘", year: 2021, rating: 8.0, type: "movie", quality: "4K HDR" },
    { id: "3", title: "哥斯拉大战金刚", year: 2021, rating: 6.5, type: "movie", quality: "4K" },
    { id: "4", title: "星际穿越", year: 2014, rating: 8.7, type: "movie", quality: "4K" },
    { id: "5", title: "蝙蝠侠：黑暗骑士崛起", year: 2012, rating: 8.4, type: "movie", quality: "4K" },
  ]
}

const filterTypeIcons: Record<FilterType, typeof User> = {
  person: User,
  genre: Layers,
  tag: Tag,
  collection: Clapperboard,
  studio: Building2,
}

const filterTypeLabels: Record<FilterType, string> = {
  person: "人物",
  genre: "分类",
  tag: "标签",
  collection: "系列",
  studio: "制作公司",
}

export function FilterPage({ 
  onBack, 
  onSelectMedia,
  filterType = "genre",
  filterValue = "科幻",
}: FilterPageProps) {
  const [viewMode, setViewMode] = useState<"grid" | "list">("grid")
  const [sortBy, setSortBy] = useState("year-desc")
  const [typeFilter, setTypeFilter] = useState<"all" | "movie" | "series">("all")

  const Icon = filterTypeIcons[filterType]

  // 根据筛选类型获取数据
  const getData = () => {
    switch (filterType) {
      case "person":
        return { 
          title: personData.name,
          subtitle: personData.originalName,
          description: personData.biography,
          media: personData.filmography.actor
        }
      case "genre":
      case "tag":
        return {
          title: genreData.name,
          subtitle: `${genreData.mediaCount} 个项目`,
          description: genreData.description,
          media: genreData.media
        }
      case "collection":
        return {
          title: collectionData.name,
          subtitle: `${collectionData.media.length} 部作品`,
          description: collectionData.description,
          media: collectionData.media
        }
      case "studio":
        return {
          title: studioData.name,
          subtitle: `${studioData.mediaCount} 个项目`,
          description: studioData.description,
          media: studioData.media
        }
      default:
        return { title: filterValue, subtitle: "", description: "", media: [] }
    }
  }

  const data = getData()

  // 过滤和排序媒体
  const filteredMedia = data.media.filter(item => {
    if (typeFilter === "all") return true
    return item.type === typeFilter
  }).sort((a, b) => {
    switch (sortBy) {
      case "year-desc":
        return b.year - a.year
      case "year-asc":
        return a.year - b.year
      case "rating-desc":
        return ("rating" in b ? b.rating : 0) - ("rating" in a ? a.rating : 0)
      case "title":
        return a.title.localeCompare(b.title)
      default:
        return 0
    }
  })

  return (
    <div className="min-h-screen bg-background">
      {/* 头部区域 */}
      <div className="relative overflow-hidden">
        {/* 背景渐变 */}
        <div className="absolute inset-0 bg-gradient-to-b from-secondary/50 to-background" />
        
        <div className="relative mx-auto max-w-6xl px-4 py-6 lg:px-8 lg:py-8">
          {/* 返回按钮 */}
          <Button 
            variant="ghost" 
            size="sm" 
            onClick={onBack}
            className="mb-4 gap-2"
          >
            <ChevronLeft className="h-4 w-4" />
            返回
          </Button>

          <div className="flex flex-col gap-6 md:flex-row md:items-start">
            {/* 人物头像/图标 */}
            {filterType === "person" ? (
              <div className="hidden h-32 w-32 flex-shrink-0 items-center justify-center overflow-hidden rounded-full bg-muted md:flex lg:h-40 lg:w-40">
                <User className="h-16 w-16 text-muted-foreground lg:h-20 lg:w-20" />
              </div>
            ) : (
              <div className="hidden h-24 w-24 flex-shrink-0 items-center justify-center rounded-xl bg-primary/10 md:flex">
                <Icon className="h-12 w-12 text-primary" />
              </div>
            )}

            {/* 信息 */}
            <div className="flex-1 space-y-3">
              <div className="flex items-center gap-2">
                <Badge variant="secondary" className="text-xs">
                  {filterTypeLabels[filterType]}
                </Badge>
              </div>
              
              <div>
                <h1 className="text-2xl font-bold lg:text-3xl">{data.title}</h1>
                {data.subtitle && (
                  <p className="mt-1 text-muted-foreground">{data.subtitle}</p>
                )}
              </div>

              {filterType === "person" && (
                <div className="flex flex-wrap gap-4 text-sm text-muted-foreground">
                  <span>出生: {personData.birthDate}</span>
                  <span>{personData.birthPlace}</span>
                </div>
              )}

              {data.description && (
                <p className="max-w-2xl text-sm leading-relaxed text-muted-foreground">
                  {data.description}
                </p>
              )}

              {/* 人物角色统计 */}
              {filterType === "person" && (
                <div className="flex gap-4">
                  {personData.roles.map((role) => (
                    <Badge key={role} variant="outline">{role}</Badge>
                  ))}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* 代表作 - 仅人物页面 */}
      {filterType === "person" && (
        <div className="mx-auto max-w-6xl px-4 py-6 lg:px-8">
          <h2 className="mb-4 text-lg font-semibold">代表作品</h2>
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6">
            {personData.knownFor.slice(0, 6).map((item) => (
              <div 
                key={item.id}
                className="group cursor-pointer"
                onClick={() => onSelectMedia?.(item.id)}
              >
                <div className="relative mb-2 aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-transform group-hover:scale-[1.02]">
                  <div className="absolute inset-0 flex items-center justify-center">
                    {item.type === "movie" ? (
                      <Film className="h-8 w-8 text-muted-foreground" />
                    ) : (
                      <Tv className="h-8 w-8 text-muted-foreground" />
                    )}
                  </div>
                  <div className="absolute inset-0 bg-gradient-to-t from-background/60 via-transparent to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
                  <Button
                    size="icon"
                    className="absolute left-1/2 top-1/2 h-10 w-10 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/90 text-primary-foreground opacity-0 shadow-lg transition-opacity group-hover:opacity-100"
                  >
                    <Play className="h-4 w-4" />
                  </Button>
                </div>
                <p className="truncate text-sm font-medium">{item.title}</p>
                <p className="text-xs text-muted-foreground">{item.character}</p>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* 筛选工具栏 */}
      <div className="sticky top-0 z-20 border-b border-border/50 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto flex max-w-6xl items-center justify-between gap-4 px-4 py-3 lg:px-8">
          <div className="flex items-center gap-2">
            <span className="text-sm text-muted-foreground">
              {filteredMedia.length} 个结果
            </span>
          </div>

          <div className="flex items-center gap-2">
            {/* 类型筛选 */}
            <Select value={typeFilter} onValueChange={(v) => setTypeFilter(v as typeof typeFilter)}>
              <SelectTrigger className="w-24 text-xs sm:w-28 sm:text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部类型</SelectItem>
                <SelectItem value="movie">电影</SelectItem>
                <SelectItem value="series">剧集</SelectItem>
              </SelectContent>
            </Select>

            {/* 排序 */}
            <Select value={sortBy} onValueChange={setSortBy}>
              <SelectTrigger className="w-24 text-xs sm:w-32 sm:text-sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="year-desc">年份 (新到旧)</SelectItem>
                <SelectItem value="year-asc">年份 (旧到新)</SelectItem>
                <SelectItem value="rating-desc">评分 (高到低)</SelectItem>
                <SelectItem value="title">标题</SelectItem>
              </SelectContent>
            </Select>

            {/* 更多筛选 */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="outline" size="icon" className="h-9 w-9">
                  <SlidersHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuLabel>筛选选项</DropdownMenuLabel>
                <DropdownMenuSeparator />
                <DropdownMenuCheckboxItem checked>4K 可用</DropdownMenuCheckboxItem>
                <DropdownMenuCheckboxItem checked>HDR</DropdownMenuCheckboxItem>
                <DropdownMenuCheckboxItem>中文字幕</DropdownMenuCheckboxItem>
              </DropdownMenuContent>
            </DropdownMenu>

            {/* 视图切换 */}
            <div className="hidden items-center rounded-md border border-border sm:flex">
              <Button 
                variant={viewMode === "grid" ? "secondary" : "ghost"}
                size="icon"
                className="h-9 w-9 rounded-r-none"
                onClick={() => setViewMode("grid")}
              >
                <Grid3X3 className="h-4 w-4" />
              </Button>
              <Button 
                variant={viewMode === "list" ? "secondary" : "ghost"}
                size="icon"
                className="h-9 w-9 rounded-l-none"
                onClick={() => setViewMode("list")}
              >
                <List className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      </div>

      {/* 媒体网格/列表 */}
      <div className="mx-auto max-w-6xl px-4 py-6 lg:px-8">
        {filterType === "person" && (
          <h2 className="mb-4 text-lg font-semibold">全部作品</h2>
        )}
        
        {viewMode === "grid" ? (
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
            {filteredMedia.map((item) => (
              <MediaGridCard 
                key={item.id} 
                item={item} 
                showCharacter={filterType === "person"}
                onClick={() => onSelectMedia?.(item.id)}
              />
            ))}
          </div>
        ) : (
          <div className="space-y-2">
            {filteredMedia.map((item) => (
              <MediaListCard 
                key={item.id} 
                item={item}
                showCharacter={filterType === "person"}
                onClick={() => onSelectMedia?.(item.id)}
              />
            ))}
          </div>
        )}
      </div>

      {/* 系列预告 - 仅系列页面 */}
      {filterType === "collection" && collectionData.upcoming.length > 0 && (
        <div className="mx-auto max-w-6xl px-4 py-6 lg:px-8">
          <h2 className="mb-4 text-lg font-semibold">即将上映</h2>
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5">
            {collectionData.upcoming.map((item) => (
              <div key={item.title} className="opacity-60">
                <div className="relative mb-2 aspect-[2/3] overflow-hidden rounded-lg bg-muted">
                  <div className="absolute inset-0 flex items-center justify-center">
                    <Film className="h-8 w-8 text-muted-foreground" />
                  </div>
                  <div className="absolute inset-0 flex items-center justify-center bg-background/50">
                    <Badge variant="outline">即将上映</Badge>
                  </div>
                </div>
                <p className="truncate text-sm font-medium">{item.title}</p>
                <p className="text-xs text-muted-foreground">{item.year}</p>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

// 网格卡片
function MediaGridCard({ 
  item, 
  showCharacter,
  onClick 
}: { 
  item: { 
    id: string
    title: string
    year: number
    type: string
    character?: string
    rating?: number
    quality?: string
    seasons?: number
  }
  showCharacter?: boolean
  onClick?: () => void 
}) {
  return (
    <div className="group cursor-pointer" onClick={onClick}>
      <div className="relative mb-2 aspect-[2/3] overflow-hidden rounded-lg bg-muted transition-transform group-hover:scale-[1.02]">
        <div className="absolute inset-0 flex items-center justify-center">
          {item.type === "movie" ? (
            <Film className="h-8 w-8 text-muted-foreground" />
          ) : (
            <Tv className="h-8 w-8 text-muted-foreground" />
          )}
        </div>
        <div className="absolute inset-0 bg-gradient-to-t from-background/60 via-transparent to-transparent opacity-0 transition-opacity group-hover:opacity-100" />
        <Button
          size="icon"
          className="absolute left-1/2 top-1/2 h-10 w-10 -translate-x-1/2 -translate-y-1/2 rounded-full bg-primary/90 text-primary-foreground opacity-0 shadow-lg transition-opacity group-hover:opacity-100"
        >
          <Play className="h-4 w-4" />
        </Button>
        
        {item.quality && (
          <Badge className="absolute right-2 top-2 bg-background/80 text-[10px] text-foreground backdrop-blur-sm">
            {item.quality}
          </Badge>
        )}
        
        {item.type === "series" && item.seasons && (
          <Badge variant="secondary" className="absolute bottom-2 left-2 text-[10px]">
            {item.seasons} 季
          </Badge>
        )}
      </div>
      <p className="truncate text-sm font-medium">{item.title}</p>
      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <span>{item.year}</span>
        {item.rating && (
          <>
            <span>·</span>
            <Star className="h-3 w-3 fill-accent text-accent" />
            <span>{item.rating}</span>
          </>
        )}
      </div>
      {showCharacter && item.character && (
        <p className="truncate text-xs text-muted-foreground">饰 {item.character}</p>
      )}
    </div>
  )
}

// 列表卡片
function MediaListCard({ 
  item,
  showCharacter,
  onClick 
}: { 
  item: { 
    id: string
    title: string
    year: number
    type: string
    character?: string
    rating?: number
    quality?: string
    runtime?: number
  }
  showCharacter?: boolean
  onClick?: () => void 
}) {
  return (
    <div 
      className="group flex cursor-pointer items-center gap-4 rounded-lg border border-border/50 bg-card p-3 transition-colors hover:border-border hover:bg-card/80"
      onClick={onClick}
    >
      {/* 缩略图 */}
      <div className="relative h-16 w-12 flex-shrink-0 overflow-hidden rounded bg-muted sm:h-20 sm:w-14">
        <div className="absolute inset-0 flex items-center justify-center">
          {item.type === "movie" ? (
            <Film className="h-6 w-6 text-muted-foreground" />
          ) : (
            <Tv className="h-6 w-6 text-muted-foreground" />
          )}
        </div>
      </div>

      {/* 信息 */}
      <div className="min-w-0 flex-1">
        <div className="flex items-center gap-2">
          <h3 className="truncate font-medium">{item.title}</h3>
          {item.quality && (
            <Badge variant="secondary" className="hidden text-[10px] sm:inline-flex">
              {item.quality}
            </Badge>
          )}
        </div>
        <div className="mt-1 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <span>{item.year}</span>
          {item.rating && (
            <>
              <span>·</span>
              <span className="flex items-center gap-0.5">
                <Star className="h-3 w-3 fill-accent text-accent" />
                {item.rating}
              </span>
            </>
          )}
          {item.runtime && (
            <>
              <span>·</span>
              <span>{Math.floor(item.runtime / 60)}小时{item.runtime % 60}分钟</span>
            </>
          )}
          <Badge variant="outline" className="text-[10px]">
            {item.type === "movie" ? "电影" : "剧集"}
          </Badge>
        </div>
        {showCharacter && item.character && (
          <p className="mt-1 text-xs text-muted-foreground">饰 {item.character}</p>
        )}
      </div>

      {/* 操作 */}
      <Button 
        size="sm" 
        className="hidden gap-1 opacity-0 transition-opacity group-hover:opacity-100 sm:flex"
      >
        <Play className="h-3 w-3 fill-current" />
        播放
      </Button>
    </div>
  )
}
