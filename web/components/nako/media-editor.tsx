"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState } from "react"
import {
  ChevronLeft, Save, RotateCcw, Upload, Search, Plus, X, Trash2,
  Film, Tv, Calendar, Clock, Star, Tag, Users, Globe, FileText,
  Image as ImageIcon, RefreshCw, Check, AlertCircle, Loader2
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { ScrollArea } from "@/components/ui/scroll-area"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog"
import { cn } from "@/lib/utils"

interface MediaData {
  id: string
  type: "movie" | "series"
  title: string
  originalTitle: string
  sortTitle: string
  year: number
  releaseDate: string
  overview: string
  tagline: string
  rating: number
  runtime: number
  status: string
  genres: string[]
  studios: string[]
  cast: { name: string; role: string; photo: string }[]
  directors: string[]
  writers: string[]
  poster: string
  backdrop: string
  logos: string[]
  language: string
  country: string
  certification: string
  tmdbId: string
  imdbId: string
  tags: string[]
  isVisible: boolean
  dateAdded: string
}

// 模拟数据
const mockMedia: MediaData = {
  id: "1",
  type: "movie",
  title: "沙丘2",
  originalTitle: "Dune: Part Two",
  sortTitle: "Dune Part Two",
  year: 2024,
  releaseDate: "2024-02-27",
  overview: "保罗·厄崔迪与弗里曼人联手，向杀害其家族的阴谋者复仇。他必须做出选择：是选择他一生挚爱的女子，还是选择宇宙的命运。他还要努力阻止只有他才能预见的可怕未来。",
  tagline: "Long live the fighters",
  rating: 8.6,
  runtime: 166,
  status: "已发布",
  genres: ["科幻", "冒险", "动作"],
  studios: ["Legendary Pictures", "Warner Bros."],
  cast: [
    { name: "提摩西·查拉梅", role: "Paul Atreides", photo: "/avatars/avatar-1.jpg" },
    { name: "赞达亚", role: "Chani", photo: "/avatars/avatar-2.jpg" },
    { name: "丽贝卡·弗格森", role: "Lady Jessica", photo: "/avatars/avatar-3.jpg" },
    { name: "奥斯汀·巴特勒", role: "Feyd-Rautha", photo: "/avatars/avatar-4.jpg" },
  ],
  directors: ["丹尼斯·维伦纽瓦"],
  writers: ["丹尼斯·维伦纽瓦", "乔·斯派茨"],
  poster: "/posters/dune2.jpg",
  backdrop: "/backdrops/dune2-backdrop.jpg",
  logos: [],
  language: "en",
  country: "US",
  certification: "PG-13",
  tmdbId: "693134",
  imdbId: "tt15239678",
  tags: ["科幻史诗", "改编作品", "沙漠"],
  isVisible: true,
  dateAdded: "2024-03-01",
}

const genreOptions = [
  "动作", "冒险", "动画", "喜剧", "犯罪", "纪录片", "剧情", "家庭",
  "奇幻", "历史", "恐怖", "音乐", "悬疑", "爱情", "科幻", "惊悚", "战争", "西部"
]

const languageOptions = [
  { value: "zh", label: "中文" },
  { value: "en", label: "英语" },
  { value: "ja", label: "日语" },
  { value: "ko", label: "韩语" },
  { value: "fr", label: "法语" },
  { value: "de", label: "德语" },
  { value: "es", label: "西班牙语" },
]

const countryOptions = [
  { value: "CN", label: "中国" },
  { value: "US", label: "美国" },
  { value: "JP", label: "日本" },
  { value: "KR", label: "韩国" },
  { value: "GB", label: "英国" },
  { value: "FR", label: "法国" },
  { value: "DE", label: "德国" },
]

interface MediaEditorProps {
  mediaId?: string
  mediaType?: "movie" | "series"
  onBack?: () => void
  onSave?: (data: MediaData) => void
}

export function MediaEditor({ mediaId, mediaType = "movie", onBack, onSave }: MediaEditorProps) {
  const [data, setData] = useState<MediaData>(mockMedia)
  const [activeTab, setActiveTab] = useState("basic")
  const [isSaving, setIsSaving] = useState(false)
  const [hasChanges, setHasChanges] = useState(false)
  const [showMetadataSearch, setShowMetadataSearch] = useState(false)
  const [showImagePicker, setShowImagePicker] = useState<"poster" | "backdrop" | null>(null)
  const [newTag, setNewTag] = useState("")
  const [newGenre, setNewGenre] = useState("")

  const updateField = <K extends keyof MediaData>(field: K, value: MediaData[K]) => {
    setData(prev => ({ ...prev, [field]: value }))
    setHasChanges(true)
  }

  const handleSave = async () => {
    setIsSaving(true)
    // 模拟保存
    await new Promise(resolve => setTimeout(resolve, 1000))
    setIsSaving(false)
    setHasChanges(false)
    onSave?.(data)
  }

  const handleReset = () => {
    setData(mockMedia)
    setHasChanges(false)
  }

  const addTag = () => {
    if (newTag.trim() && !data.tags.includes(newTag.trim())) {
      updateField("tags", [...data.tags, newTag.trim()])
      setNewTag("")
    }
  }

  const removeTag = (tag: string) => {
    updateField("tags", data.tags.filter(t => t !== tag))
  }

  const addGenre = () => {
    if (newGenre && !data.genres.includes(newGenre)) {
      updateField("genres", [...data.genres, newGenre])
      setNewGenre("")
    }
  }

  const removeGenre = (genre: string) => {
    updateField("genres", data.genres.filter(g => g !== genre))
  }

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-4 lg:px-6">
        <div className="flex items-center gap-4">
          {onBack && (
            <Button variant="ghost" size="icon" onClick={onBack}>
              <ChevronLeft className="h-5 w-5" />
            </Button>
          )}
          <div>
            <h1 className="text-xl font-semibold">编辑媒体信息</h1>
            <p className="text-sm text-muted-foreground">
              {data.type === "movie" ? "电影" : "剧集"} · {data.title}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {hasChanges && (
            <Badge variant="outline" className="text-yellow-600 border-yellow-600">
              未保存更改
            </Badge>
          )}
          <Button variant="outline" onClick={() => setShowMetadataSearch(true)}>
            <RefreshCw className="mr-2 h-4 w-4" />
            刷新元数据
          </Button>
          <Button variant="outline" onClick={handleReset} disabled={!hasChanges}>
            <RotateCcw className="mr-2 h-4 w-4" />
            重置
          </Button>
          <Button onClick={handleSave} disabled={!hasChanges || isSaving}>
            {isSaving ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Save className="mr-2 h-4 w-4" />
            )}
            保存
          </Button>
        </div>
      </div>

      {/* Content */}
      <div className="flex flex-1 overflow-hidden">
        {/* 左侧预览 */}
        <div className="hidden w-80 flex-shrink-0 border-r border-border p-6 lg:block">
          <div className="sticky top-0 space-y-4">
            {/* 海报 */}
            <div
              className="group relative aspect-[2/3] cursor-pointer overflow-hidden rounded-lg bg-muted"
              onClick={() => setShowImagePicker("poster")}
            >
              <img src={resolveArtwork(data.poster)} alt={data.title} className="h-full w-full object-cover" />
              <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100">
                <Button variant="secondary" size="sm">
                  <ImageIcon className="mr-2 h-4 w-4" />
                  更换海报
                </Button>
              </div>
            </div>

            {/* 背景图 */}
            <div
              className="group relative aspect-video cursor-pointer overflow-hidden rounded-lg bg-muted"
              onClick={() => setShowImagePicker("backdrop")}
            >
              <img src={resolveArtwork(data.backdrop)} alt={data.title} className="h-full w-full object-cover" />
              <div className="absolute inset-0 flex items-center justify-center bg-black/50 opacity-0 transition-opacity group-hover:opacity-100">
                <Button variant="secondary" size="sm">
                  <ImageIcon className="mr-2 h-4 w-4" />
                  更换背景
                </Button>
              </div>
            </div>

            {/* 快捷信息 */}
            <div className="space-y-2 text-sm">
              <div className="flex items-center gap-2 text-muted-foreground">
                <Calendar className="h-4 w-4" />
                <span>添加于 {data.dateAdded}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <FileText className="h-4 w-4" />
                <span>TMDB: {data.tmdbId}</span>
              </div>
              <div className="flex items-center gap-2 text-muted-foreground">
                <FileText className="h-4 w-4" />
                <span>IMDB: {data.imdbId}</span>
              </div>
            </div>
          </div>
        </div>

        {/* 右侧编辑区 */}
        <div className="flex-1 overflow-hidden">
          <Tabs value={activeTab} onValueChange={setActiveTab} className="flex h-full flex-col">
            <div className="border-b border-border px-4 lg:px-6">
              <TabsList className="h-12 w-full justify-start gap-1 bg-transparent p-0">
                <TabsTrigger
                  value="basic"
                  className="h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
                >
                  基本信息
                </TabsTrigger>
                <TabsTrigger
                  value="details"
                  className="h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
                >
                  详细信息
                </TabsTrigger>
                <TabsTrigger
                  value="cast"
                  className="h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
                >
                  演职员
                </TabsTrigger>
                <TabsTrigger
                  value="media"
                  className="h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
                >
                  图片管理
                </TabsTrigger>
                <TabsTrigger
                  value="advanced"
                  className="h-12 rounded-none border-b-2 border-transparent px-4 data-[state=active]:border-primary data-[state=active]:bg-transparent"
                >
                  高级选项
                </TabsTrigger>
              </TabsList>
            </div>

            <ScrollArea className="flex-1">
              <div className="p-4 lg:p-6">
                {/* 基本信息 */}
                <TabsContent value="basic" className="mt-0 space-y-6">
                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label htmlFor="title">标题</Label>
                      <Input
                        id="title"
                        value={data.title}
                        onChange={e => updateField("title", e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="originalTitle">原始标题</Label>
                      <Input
                        id="originalTitle"
                        value={data.originalTitle}
                        onChange={e => updateField("originalTitle", e.target.value)}
                      />
                    </div>
                  </div>

                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label htmlFor="sortTitle">排序标题</Label>
                      <Input
                        id="sortTitle"
                        value={data.sortTitle}
                        onChange={e => updateField("sortTitle", e.target.value)}
                        placeholder="用于排序的标题"
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="tagline">标语</Label>
                      <Input
                        id="tagline"
                        value={data.tagline}
                        onChange={e => updateField("tagline", e.target.value)}
                      />
                    </div>
                  </div>

                  <div className="space-y-2">
                    <Label htmlFor="overview">简介</Label>
                    <Textarea
                      id="overview"
                      value={data.overview}
                      onChange={e => updateField("overview", e.target.value)}
                      rows={5}
                    />
                  </div>

                  <div className="grid gap-6 lg:grid-cols-3">
                    <div className="space-y-2">
                      <Label htmlFor="year">年份</Label>
                      <Input
                        id="year"
                        type="number"
                        value={data.year}
                        onChange={e => updateField("year", parseInt(e.target.value))}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="releaseDate">发布日期</Label>
                      <Input
                        id="releaseDate"
                        type="date"
                        value={data.releaseDate}
                        onChange={e => updateField("releaseDate", e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="runtime">时长 (分钟)</Label>
                      <Input
                        id="runtime"
                        type="number"
                        value={data.runtime}
                        onChange={e => updateField("runtime", parseInt(e.target.value))}
                      />
                    </div>
                  </div>

                  {/* 类型 */}
                  <div className="space-y-2">
                    <Label>类型</Label>
                    <div className="flex flex-wrap gap-2">
                      {data.genres.map(genre => (
                        <Badge key={genre} variant="secondary" className="gap-1">
                          {genre}
                          <button onClick={() => removeGenre(genre)}>
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                      <Select value={newGenre} onValueChange={val => { setNewGenre(val); if (val) addGenre(); }}>
                        <SelectTrigger className="w-32">
                          <SelectValue placeholder="添加类型" />
                        </SelectTrigger>
                        <SelectContent>
                          {genreOptions.filter(g => !data.genres.includes(g)).map(genre => (
                            <SelectItem key={genre} value={genre}>{genre}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  {/* 标签 */}
                  <div className="space-y-2">
                    <Label>自定义标签</Label>
                    <div className="flex flex-wrap gap-2">
                      {data.tags.map(tag => (
                        <Badge key={tag} variant="outline" className="gap-1">
                          <Tag className="h-3 w-3" />
                          {tag}
                          <button onClick={() => removeTag(tag)}>
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                      <div className="flex gap-1">
                        <Input
                          value={newTag}
                          onChange={e => setNewTag(e.target.value)}
                          onKeyDown={e => e.key === "Enter" && addTag()}
                          placeholder="添加标签"
                          className="h-8 w-32"
                        />
                        <Button size="sm" variant="outline" onClick={addTag}>
                          <Plus className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                {/* 详细信息 */}
                <TabsContent value="details" className="mt-0 space-y-6">
                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label>评分</Label>
                      <div className="flex items-center gap-2">
                        <Input
                          type="number"
                          step="0.1"
                          min="0"
                          max="10"
                          value={data.rating}
                          onChange={e => updateField("rating", parseFloat(e.target.value))}
                          className="w-24"
                        />
                        <Star className="h-5 w-5 fill-yellow-500 text-yellow-500" />
                        <span className="text-muted-foreground">/ 10</span>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label>状态</Label>
                      <Select value={data.status} onValueChange={val => updateField("status", val)}>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="已发布">已发布</SelectItem>
                          <SelectItem value="即将上映">即将上映</SelectItem>
                          <SelectItem value="制作中">制作中</SelectItem>
                          <SelectItem value="已取消">已取消</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label>语言</Label>
                      <Select value={data.language} onValueChange={val => updateField("language", val)}>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {languageOptions.map(opt => (
                            <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>国家/地区</Label>
                      <Select value={data.country} onValueChange={val => updateField("country", val)}>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          {countryOptions.map(opt => (
                            <SelectItem key={opt.value} value={opt.value}>{opt.label}</SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label>分级</Label>
                      <Select value={data.certification} onValueChange={val => updateField("certification", val)}>
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="G">G - 普遍级</SelectItem>
                          <SelectItem value="PG">PG - 建议家长指导</SelectItem>
                          <SelectItem value="PG-13">PG-13 - 13岁以下需家长指导</SelectItem>
                          <SelectItem value="R">R - 限制级</SelectItem>
                          <SelectItem value="NC-17">NC-17 - 仅成人</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label>制片公司</Label>
                      <Input
                        value={data.studios.join(", ")}
                        onChange={e => updateField("studios", e.target.value.split(",").map(s => s.trim()))}
                        placeholder="多个公司用逗号分隔"
                      />
                    </div>
                  </div>

                  <div className="grid gap-6 lg:grid-cols-2">
                    <div className="space-y-2">
                      <Label>TMDB ID</Label>
                      <Input
                        value={data.tmdbId}
                        onChange={e => updateField("tmdbId", e.target.value)}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label>IMDB ID</Label>
                      <Input
                        value={data.imdbId}
                        onChange={e => updateField("imdbId", e.target.value)}
                      />
                    </div>
                  </div>
                </TabsContent>

                {/* 演职员 */}
                <TabsContent value="cast" className="mt-0 space-y-6">
                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <Label>导演</Label>
                      <Button variant="outline" size="sm">
                        <Plus className="mr-2 h-4 w-4" />
                        添加导演
                      </Button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {data.directors.map(director => (
                        <Badge key={director} variant="secondary" className="gap-1 py-1.5">
                          <Users className="h-3 w-3" />
                          {director}
                          <button className="ml-1 hover:text-destructive">
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <Label>编剧</Label>
                      <Button variant="outline" size="sm">
                        <Plus className="mr-2 h-4 w-4" />
                        添加编剧
                      </Button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {data.writers.map(writer => (
                        <Badge key={writer} variant="secondary" className="gap-1 py-1.5">
                          <Users className="h-3 w-3" />
                          {writer}
                          <button className="ml-1 hover:text-destructive">
                            <X className="h-3 w-3" />
                          </button>
                        </Badge>
                      ))}
                    </div>
                  </div>

                  <div className="space-y-4">
                    <div className="flex items-center justify-between">
                      <Label>演员</Label>
                      <Button variant="outline" size="sm">
                        <Plus className="mr-2 h-4 w-4" />
                        添加演员
                      </Button>
                    </div>
                    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                      {data.cast.map((person, index) => (
                        <div key={index} className="flex items-center gap-3 rounded-lg border border-border bg-card p-3">
                          <img
                            src={resolveArtwork(person.photo)}
                            alt={person.name}
                            className="h-12 w-12 rounded-full object-cover"
                          />
                          <div className="flex-1 min-w-0">
                            <p className="font-medium truncate">{person.name}</p>
                            <p className="text-sm text-muted-foreground truncate">{person.role}</p>
                          </div>
                          <Button variant="ghost" size="icon" className="h-8 w-8 flex-shrink-0">
                            <X className="h-4 w-4" />
                          </Button>
                        </div>
                      ))}
                    </div>
                  </div>
                </TabsContent>

                {/* 图片管理 */}
                <TabsContent value="media" className="mt-0 space-y-6">
                  <div className="space-y-4">
                    <Label>海报</Label>
                    <div className="grid gap-4 sm:grid-cols-3 lg:grid-cols-4">
                      <div
                        className={cn(
                          "relative aspect-[2/3] overflow-hidden rounded-lg border-2 cursor-pointer",
                          "border-primary ring-2 ring-primary/20"
                        )}
                      >
                        <img src={resolveArtwork(data.poster)} alt="当前海报" className="h-full w-full object-cover" />
                        <div className="absolute top-2 left-2">
                          <Badge className="bg-primary">当前</Badge>
                        </div>
                      </div>
                      <div
                        className="flex aspect-[2/3] cursor-pointer items-center justify-center rounded-lg border-2 border-dashed border-border hover:border-primary/50 hover:bg-muted/50 transition-colors"
                        onClick={() => setShowImagePicker("poster")}
                      >
                        <div className="text-center">
                          <Upload className="mx-auto h-8 w-8 text-muted-foreground" />
                          <p className="mt-2 text-sm text-muted-foreground">上传或搜索</p>
                        </div>
                      </div>
                    </div>
                  </div>

                  <div className="space-y-4">
                    <Label>背景图</Label>
                    <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                      <div
                        className={cn(
                          "relative aspect-video overflow-hidden rounded-lg border-2 cursor-pointer",
                          "border-primary ring-2 ring-primary/20"
                        )}
                      >
                        <img src={resolveArtwork(data.backdrop)} alt="当前背景" className="h-full w-full object-cover" />
                        <div className="absolute top-2 left-2">
                          <Badge className="bg-primary">当前</Badge>
                        </div>
                      </div>
                      <div
                        className="flex aspect-video cursor-pointer items-center justify-center rounded-lg border-2 border-dashed border-border hover:border-primary/50 hover:bg-muted/50 transition-colors"
                        onClick={() => setShowImagePicker("backdrop")}
                      >
                        <div className="text-center">
                          <Upload className="mx-auto h-8 w-8 text-muted-foreground" />
                          <p className="mt-2 text-sm text-muted-foreground">上传或搜索</p>
                        </div>
                      </div>
                    </div>
                  </div>
                </TabsContent>

                {/* 高级选项 */}
                <TabsContent value="advanced" className="mt-0 space-y-6">
                  <div className="rounded-lg border border-border bg-card p-4 space-y-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium">在媒体库中显示</p>
                        <p className="text-sm text-muted-foreground">隐藏后不会在用户界面显示</p>
                      </div>
                      <Switch
                        checked={data.isVisible}
                        onCheckedChange={val => updateField("isVisible", val)}
                      />
                    </div>
                  </div>

                  <div className="rounded-lg border border-destructive/50 bg-destructive/5 p-4 space-y-4">
                    <h3 className="font-medium text-destructive">危险操作</h3>
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium">删除媒体</p>
                        <p className="text-sm text-muted-foreground">从媒体库中删除此项目（不删除文件）</p>
                      </div>
                      <Button variant="destructive" size="sm">
                        <Trash2 className="mr-2 h-4 w-4" />
                        删除
                      </Button>
                    </div>
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="font-medium">删除媒体和文件</p>
                        <p className="text-sm text-muted-foreground">同时删除关联的媒体文件</p>
                      </div>
                      <Button variant="destructive" size="sm">
                        <Trash2 className="mr-2 h-4 w-4" />
                        删除全部
                      </Button>
                    </div>
                  </div>
                </TabsContent>
              </div>
            </ScrollArea>
          </Tabs>
        </div>
      </div>

      {/* 元数据搜索弹窗 */}
      <Dialog open={showMetadataSearch} onOpenChange={setShowMetadataSearch}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>搜索元数据</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div className="flex gap-2">
              <Input placeholder="搜索电影或剧集..." defaultValue={data.title} />
              <Button>
                <Search className="h-4 w-4" />
              </Button>
            </div>
            <div className="text-center py-8 text-muted-foreground">
              输入标题搜索 TMDb 元数据
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* 图片选择弹窗 */}
      <Dialog open={!!showImagePicker} onOpenChange={() => setShowImagePicker(null)}>
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>
              选择{showImagePicker === "poster" ? "海报" : "背景图"}
            </DialogTitle>
          </DialogHeader>
          <Tabs defaultValue="search">
            <TabsList>
              <TabsTrigger value="search">从 TMDb 搜索</TabsTrigger>
              <TabsTrigger value="upload">上传图片</TabsTrigger>
              <TabsTrigger value="url">图片 URL</TabsTrigger>
            </TabsList>
            <TabsContent value="search" className="space-y-4">
              <div className="flex gap-2">
                <Input placeholder="搜索图片..." />
                <Button>搜索</Button>
              </div>
              <div className="text-center py-8 text-muted-foreground">
                搜索 TMDb 获取图片
              </div>
            </TabsContent>
            <TabsContent value="upload" className="space-y-4">
              <div className="flex aspect-video items-center justify-center rounded-lg border-2 border-dashed border-border">
                <div className="text-center">
                  <Upload className="mx-auto h-12 w-12 text-muted-foreground" />
                  <p className="mt-2 text-muted-foreground">点击或拖拽上传图片</p>
                </div>
              </div>
            </TabsContent>
            <TabsContent value="url" className="space-y-4">
              <Input placeholder="输入图片 URL..." />
              <Button className="w-full">确认</Button>
            </TabsContent>
          </Tabs>
        </DialogContent>
      </Dialog>
    </div>
  )
}
