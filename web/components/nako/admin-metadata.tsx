"use client"

import { useState } from "react"
import { 
  Database, 
  Search, 
  RefreshCw,
  ExternalLink,
  CheckCircle2,
  AlertCircle,
  Clock,
  Image,
  FileText,
  Film,
  Tv,
  MoreHorizontal,
  Link,
  Unlink,
  Pencil,
  Download,
  Trash2,
  Globe,
  Loader2
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
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
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Progress } from "@/components/ui/progress"

// 元数据提供者
const metadataProviders = [
  {
    id: "tmdb",
    name: "TheMovieDB",
    icon: "🎬",
    enabled: true,
    priority: 1,
    types: ["movie", "tv"],
    lastSync: "2024-03-15 14:30",
    status: "connected",
    languages: ["zh-CN", "en", "ja"],
    apiKey: "sk-****-****-1234"
  },
  {
    id: "tvdb",
    name: "TheTVDB",
    icon: "📺",
    enabled: true,
    priority: 2,
    types: ["tv", "anime"],
    lastSync: "2024-03-15 14:30",
    status: "connected",
    languages: ["en", "zh-CN"],
    apiKey: "****-5678"
  },
  {
    id: "anidb",
    name: "AniDB",
    icon: "🎌",
    enabled: true,
    priority: 1,
    types: ["anime"],
    lastSync: "2024-03-15 12:00",
    status: "connected",
    languages: ["ja", "en"],
    apiKey: null
  },
  {
    id: "douban",
    name: "豆瓣",
    icon: "🥜",
    enabled: false,
    priority: 3,
    types: ["movie", "tv"],
    lastSync: null,
    status: "disconnected",
    languages: ["zh-CN"],
    apiKey: null
  },
]

// 待匹配的媒体
const unmatchedMedia = [
  {
    id: "1",
    filename: "The.Matrix.1999.2160p.UHD.BluRay.x265.mkv",
    path: "/media/movies/The Matrix (1999)",
    type: "movie",
    size: "45.2 GB",
    addedAt: "2024-03-15 10:00",
    suggestions: [
      { id: "tmdb-603", title: "黑客帝国", year: 1999, source: "TMDB", match: 95 },
      { id: "tmdb-604", title: "黑客帝国2：重装上阵", year: 2003, source: "TMDB", match: 45 },
    ]
  },
  {
    id: "2",
    filename: "[SubGroup] Anime Title - 01 [1080p].mkv",
    path: "/media/anime/Unknown Anime",
    type: "anime",
    size: "1.4 GB",
    addedAt: "2024-03-14 22:30",
    suggestions: []
  },
  {
    id: "3",
    filename: "Documentary.About.Nature.2024.WEB-DL.mkv",
    path: "/media/documentary/Nature Doc",
    type: "movie",
    size: "8.7 GB",
    addedAt: "2024-03-14 18:00",
    suggestions: [
      { id: "tmdb-12345", title: "大自然的奇迹", year: 2024, source: "TMDB", match: 72 },
    ]
  },
]

// 最近匹配的媒体
const recentMatches = [
  { id: "1", title: "沙丘2", originalTitle: "Dune: Part Two", year: 2024, source: "TMDB", matchedAt: "2024-03-15 14:30", status: "success" },
  { id: "2", title: "周处除三害", originalTitle: "The Pig, the Snake and the Pigeon", year: 2023, source: "TMDB", matchedAt: "2024-03-15 14:25", status: "success" },
  { id: "3", title: "葬送的芙莉莲", originalTitle: "Frieren", year: 2023, source: "AniDB", matchedAt: "2024-03-15 14:20", status: "success" },
  { id: "4", title: "Unknown File", originalTitle: null, year: null, source: null, matchedAt: "2024-03-15 14:15", status: "failed" },
]

export function AdminMetadata() {
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedMedia, setSelectedMedia] = useState<typeof unmatchedMedia[0] | null>(null)
  const [isMatching, setIsMatching] = useState(false)

  const handleMatch = (mediaId: string, matchId: string) => {
    setIsMatching(true)
    // 模拟匹配过程
    setTimeout(() => {
      setIsMatching(false)
      setSelectedMedia(null)
    }, 1500)
  }

  return (
    <div className="space-y-6">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">元数据管理</h1>
          <p className="text-muted-foreground">配置元数据提供者和手动匹配媒体</p>
        </div>
        <Button variant="outline" className="gap-2">
          <RefreshCw className="h-4 w-4" />
          刷新全部元数据
        </Button>
      </div>

      <Tabs defaultValue="providers" className="space-y-6">
        <TabsList>
          <TabsTrigger value="providers">元数据提供者</TabsTrigger>
          <TabsTrigger value="unmatched">
            待匹配
            {unmatchedMedia.length > 0 && (
              <Badge variant="secondary" className="ml-2 bg-warning/10 text-warning">
                {unmatchedMedia.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="recent">最近匹配</TabsTrigger>
        </TabsList>

        {/* 元数据提供者 */}
        <TabsContent value="providers" className="space-y-4">
          <div className="grid gap-4">
            {metadataProviders.map((provider) => (
              <Card key={provider.id}>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-4">
                      <div className="text-3xl">{provider.icon}</div>
                      <div>
                        <div className="flex items-center gap-2">
                          <h3 className="font-semibold">{provider.name}</h3>
                          {provider.status === "connected" ? (
                            <Badge variant="secondary" className="bg-success/10 text-success gap-1">
                              <CheckCircle2 className="h-3 w-3" />
                              已连接
                            </Badge>
                          ) : (
                            <Badge variant="secondary" className="bg-muted text-muted-foreground gap-1">
                              <AlertCircle className="h-3 w-3" />
                              未连接
                            </Badge>
                          )}
                        </div>
                        <div className="flex items-center gap-4 mt-1 text-sm text-muted-foreground">
                          <span>支持: {provider.types.map(t => t === "movie" ? "电影" : t === "tv" ? "剧集" : "动画").join(", ")}</span>
                          <span>优先级: {provider.priority}</span>
                          {provider.lastSync && (
                            <span className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              {provider.lastSync}
                            </span>
                          )}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center gap-3">
                      <Switch checked={provider.enabled} />
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="outline" size="sm">
                            <MoreHorizontal className="h-4 w-4" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem>
                            <Pencil className="h-4 w-4 mr-2" />
                            配置
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <RefreshCw className="h-4 w-4 mr-2" />
                            测试连接
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <ExternalLink className="h-4 w-4 mr-2" />
                            访问网站
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>

          {/* 提供者优先级说明 */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">匹配优先级</CardTitle>
              <CardDescription>
                系统会按照优先级顺序尝试从各提供者获取元数据
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2 text-sm">
                <div className="flex items-center gap-2">
                  <Badge variant="outline">电影</Badge>
                  <span className="text-muted-foreground">→</span>
                  <span>TMDB → TheTVDB → 豆瓣</span>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">剧集</Badge>
                  <span className="text-muted-foreground">→</span>
                  <span>TMDB → TheTVDB</span>
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant="outline">动画</Badge>
                  <span className="text-muted-foreground">→</span>
                  <span>AniDB → TheTVDB → TMDB</span>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* 待匹配 */}
        <TabsContent value="unmatched" className="space-y-4">
          <div className="flex items-center gap-4">
            <div className="relative flex-1 max-w-sm">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input 
                placeholder="搜索文件..." 
                className="pl-9"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
            </div>
            <Select defaultValue="all">
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部类型</SelectItem>
                <SelectItem value="movie">电影</SelectItem>
                <SelectItem value="tv">剧集</SelectItem>
                <SelectItem value="anime">动画</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className="space-y-3">
            {unmatchedMedia.map((media) => (
              <Card key={media.id} className="overflow-hidden">
                <CardContent className="p-4">
                  <div className="flex items-start justify-between gap-4">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        {media.type === "movie" ? (
                          <Film className="h-4 w-4 text-muted-foreground" />
                        ) : media.type === "anime" ? (
                          <Film className="h-4 w-4 text-muted-foreground" />
                        ) : (
                          <Tv className="h-4 w-4 text-muted-foreground" />
                        )}
                        <span className="font-medium truncate">{media.filename}</span>
                      </div>
                      <p className="text-sm text-muted-foreground truncate font-mono">{media.path}</p>
                      <div className="flex items-center gap-4 mt-2 text-xs text-muted-foreground">
                        <span>{media.size}</span>
                        <span>添加于 {media.addedAt}</span>
                      </div>

                      {/* 匹配建议 */}
                      {media.suggestions.length > 0 && (
                        <div className="mt-3 space-y-2">
                          <p className="text-xs text-muted-foreground">匹配建议:</p>
                          {media.suggestions.slice(0, 2).map((suggestion) => (
                            <div 
                              key={suggestion.id}
                              className="flex items-center justify-between p-2 rounded bg-secondary/50 border border-border"
                            >
                              <div className="flex items-center gap-2">
                                <span className="font-medium text-sm">{suggestion.title}</span>
                                <span className="text-xs text-muted-foreground">({suggestion.year})</span>
                                <Badge variant="outline" className="text-xs">{suggestion.source}</Badge>
                              </div>
                              <div className="flex items-center gap-2">
                                <Badge 
                                  variant="secondary" 
                                  className={suggestion.match >= 90 ? "bg-success/10 text-success" : suggestion.match >= 70 ? "bg-warning/10 text-warning" : "bg-muted"}
                                >
                                  {suggestion.match}% 匹配
                                </Badge>
                                <Button 
                                  size="sm" 
                                  variant="outline"
                                  onClick={() => handleMatch(media.id, suggestion.id)}
                                >
                                  <Link className="h-3 w-3 mr-1" />
                                  确认
                                </Button>
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </div>
                    <Button 
                      variant="outline"
                      onClick={() => setSelectedMedia(media)}
                    >
                      手动匹配
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        </TabsContent>

        {/* 最近匹配 */}
        <TabsContent value="recent" className="space-y-4">
          <Card>
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>标题</TableHead>
                  <TableHead>原始标题</TableHead>
                  <TableHead>年份</TableHead>
                  <TableHead>来源</TableHead>
                  <TableHead>匹配时间</TableHead>
                  <TableHead>状态</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {recentMatches.map((match) => (
                  <TableRow key={match.id}>
                    <TableCell className="font-medium">{match.title}</TableCell>
                    <TableCell className="text-muted-foreground">{match.originalTitle || "-"}</TableCell>
                    <TableCell>{match.year || "-"}</TableCell>
                    <TableCell>
                      {match.source ? (
                        <Badge variant="outline">{match.source}</Badge>
                      ) : "-"}
                    </TableCell>
                    <TableCell className="text-muted-foreground">{match.matchedAt}</TableCell>
                    <TableCell>
                      {match.status === "success" ? (
                        <Badge variant="secondary" className="bg-success/10 text-success gap-1">
                          <CheckCircle2 className="h-3 w-3" />
                          成功
                        </Badge>
                      ) : (
                        <Badge variant="secondary" className="bg-destructive/10 text-destructive gap-1">
                          <AlertCircle className="h-3 w-3" />
                          失败
                        </Badge>
                      )}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </Card>
        </TabsContent>
      </Tabs>

      {/* 手动匹配对话框 */}
      <Dialog open={!!selectedMedia} onOpenChange={() => setSelectedMedia(null)}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>手动匹配</DialogTitle>
            <DialogDescription>
              搜索并选择正确的元数据匹配
            </DialogDescription>
          </DialogHeader>
          
          {selectedMedia && (
            <div className="space-y-4 py-4">
              <div className="p-3 rounded bg-secondary/50 border border-border">
                <p className="text-sm font-mono truncate">{selectedMedia.filename}</p>
                <p className="text-xs text-muted-foreground mt-1">{selectedMedia.path}</p>
              </div>

              <div className="space-y-2">
                <Label>搜索标题</Label>
                <div className="flex gap-2">
                  <Input placeholder="输入电影/剧集名称..." className="flex-1" />
                  <Button variant="outline">
                    <Search className="h-4 w-4" />
                  </Button>
                </div>
              </div>

              <div className="space-y-2">
                <Label>选择提供者</Label>
                <Select defaultValue="all">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">所有提供者</SelectItem>
                    <SelectItem value="tmdb">TheMovieDB</SelectItem>
                    <SelectItem value="tvdb">TheTVDB</SelectItem>
                    <SelectItem value="anidb">AniDB</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* 搜索结果区域 */}
              <div className="border border-border rounded-lg p-4 min-h-[200px] flex items-center justify-center text-muted-foreground">
                <div className="text-center">
                  <Globe className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-sm">输入标题开始搜索</p>
                </div>
              </div>
            </div>
          )}
          
          <DialogFooter>
            <Button variant="outline" onClick={() => setSelectedMedia(null)}>
              取消
            </Button>
            <Button disabled={isMatching}>
              {isMatching && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              确认匹配
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
