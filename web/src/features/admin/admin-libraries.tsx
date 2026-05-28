"use client"

import { useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { cn } from "@/lib/utils"
import {
  FolderOpen, 
  Plus, 
  Search, 
  MoreHorizontal,
  RefreshCw,
  Trash2,
  Settings,
  Film,
  Tv,
  Music,
  Image,
  HardDrive,
  Clock,
  CheckCircle2,
  AlertCircle,
  AlertTriangle,
  Loader2,
  ChevronRight,
  FolderPlus,
  Eye,
  EyeOff,
  Pencil,
  GripVertical,
  Database,
  Globe,
  Info,
  Download
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Progress } from "@/components/ui/progress"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
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
  ADMIN_LIBRARY_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminLibraryKind,
  type AdminLibraryReadModel,
} from "@/src/api/admin/read-models-data-source"
import { createAdminMutationDataSource } from "@/src/api/admin/mutations-data-source"

// 模拟数据
const libraries = [
  {
    id: "1",
    name: "电影",
    type: "movie",
    icon: Film,
    paths: [
      { path: "/media/movies", available: true },
      { path: "/nas/films", available: true }
    ],
    itemCount: 847,
    totalSize: "4.2 TB",
    lastScanned: "2024-03-15 14:30",
    scanStatus: "idle",
    settings: {
      autoScan: true,
      scanInterval: 6,
      useNfo: true,
      downloadArt: true,
      metadataLanguage: "zh-CN"
    }
  },
  {
    id: "2", 
    name: "剧集",
    type: "tv",
    icon: Tv,
    paths: [
      { path: "/media/tv", available: true },
      { path: "/nas/series", available: false }
    ],
    itemCount: 156,
    totalSize: "8.7 TB",
    lastScanned: "2024-03-15 14:35",
    scanStatus: "scanning",
    scanProgress: 67,
    settings: {
      autoScan: true,
      scanInterval: 3,
      useNfo: true,
      downloadArt: true,
      metadataLanguage: "zh-CN"
    }
  },
  {
    id: "3",
    name: "动画",
    type: "anime",
    icon: Film,
    paths: [
      { path: "/media/anime", available: true }
    ],
    itemCount: 234,
    totalSize: "2.1 TB",
    lastScanned: "2024-03-15 12:00",
    scanStatus: "idle",
    settings: {
      autoScan: true,
      scanInterval: 6,
      useNfo: true,
      downloadArt: true,
      metadataLanguage: "ja"
    }
  },
  {
    id: "4",
    name: "纪录片",
    type: "documentary",
    icon: Film,
    paths: [
      { path: "/media/documentary", available: false }
    ],
    itemCount: 89,
    totalSize: "890 GB",
    lastScanned: "2024-03-14 22:00",
    scanStatus: "error",
    errorMessage: "路径不可访问: /media/documentary",
    settings: {
      autoScan: false,
      scanInterval: 24,
      useNfo: false,
      downloadArt: true,
      metadataLanguage: "zh-CN"
    }
  },
]

const libraryTypes = [
  { value: "movie", label: "电影", icon: Film },
  { value: "tv", label: "剧集", icon: Tv },
  { value: "anime", label: "动画", icon: Film },
  { value: "music", label: "音乐", icon: Music },
  { value: "photo", label: "照片", icon: Image },
]

const libraryIconByKind: Record<AdminLibraryKind, typeof Film> = {
  movie: Film,
  tv: Tv,
  anime: Film,
  music: Music,
  photo: Image,
  documentary: Film,
  personal: FolderOpen,
  unknown: FolderOpen,
}

export function AdminLibraries() {
  const queryClient = useQueryClient()
  const { data: librariesData = ADMIN_LIBRARY_READ_MODEL_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "libraries"],
    queryFn: () => createAdminReadModelsDataSource().loadLibraries(),
    staleTime: 30 * 1000,
    retry: 0,
  })
  const mutationSource = createAdminMutationDataSource()
  const canMutate = librariesData.source === "live" && mutationSource.canMutate
  const libraries = librariesData.libraries
  const [searchQuery, setSearchQuery] = useState("")
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false)
  const [selectedLibrary, setSelectedLibrary] = useState<AdminLibraryReadModel | null>(null)
  const [mutationMessage, setMutationMessage] = useState<string | null>(null)
  const libraryMutation = useMutation({
    mutationFn: async (action: {
      kind: "scan" | "import-nfo" | "export-nfo"
      libraryIds: string[]
    }) => {
      if (!canMutate) {
        throw new Error(mutationSource.unavailableReason ?? "Admin mutation is unavailable")
      }

      const results = await Promise.all(
        action.libraryIds.map((libraryId) => {
          switch (action.kind) {
            case "import-nfo":
              return mutationSource.importLibraryNfo(libraryId)
            case "export-nfo":
              return mutationSource.exportLibraryNfo(libraryId)
            default:
              return mutationSource.scanLibrary(libraryId)
          }
        }),
      )

      return results.length === 1
        ? results[0].message
        : `${results.length} 个媒体库操作已提交`
    },
    onSuccess(message) {
      setMutationMessage(message)
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "libraries"] })
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "scheduled-tasks"] })
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "dashboard"] })
    },
    onError(error) {
      setMutationMessage(error instanceof Error ? error.message : "Admin mutation failed")
    },
  })

  const filteredLibraries = libraries.filter(lib => 
    lib.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  // 统计数据
  const stats = {
    totalLibraries: libraries.length,
    totalItems: libraries.reduce((sum, lib) => sum + lib.itemCount, 0),
    totalSize: "16.0 TB", // 实际应计算
    scanning: libraries.filter(lib => lib.scanStatus === "scanning").length,
    errors: libraries.filter(lib => lib.scanStatus === "error").length,
    unavailablePaths: libraries.reduce((sum, lib) => 
      sum + lib.paths.filter(p => !p.available).length, 0
    )
  }

  const getStatusBadge = (library: AdminLibraryReadModel) => {
    switch (library.scanStatus) {
      case "scanning":
        return (
          <Badge variant="secondary" className="bg-info/10 text-info gap-1">
            <Loader2 className="h-3 w-3 animate-spin" />
            扫描中 {library.scanProgress}%
          </Badge>
        )
      case "error":
        return (
          <Badge variant="secondary" className="bg-destructive/10 text-destructive gap-1">
            <AlertCircle className="h-3 w-3" />
            错误
          </Badge>
        )
      default:
        return (
          <Badge variant="secondary" className="bg-success/10 text-success gap-1">
            <CheckCircle2 className="h-3 w-3" />
            正常
          </Badge>
        )
    }
  }

  const runLibraryMutation = (
    kind: "scan" | "import-nfo" | "export-nfo",
    libraryIds: string[],
    confirmation: string,
  ) => {
    setMutationMessage(null)
    if (!canMutate) {
      setMutationMessage(mutationSource.unavailableReason ?? "连接 live Admin API 后才能执行管理操作")
      return
    }

    if (typeof window !== "undefined" && !window.confirm(confirmation)) {
      return
    }

    libraryMutation.mutate({ kind, libraryIds })
  }

  return (
    <div className="space-y-6 p-1">
      {/* 页面标题 */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">媒体库管理</h1>
          <p className="text-sm text-muted-foreground">
            管理和配置你的媒体库
            <span className="ml-2 text-xs">
              {librariesData.source === "live" ? "Live Admin API" : "Fixture fallback"}
              {librariesData.error ? ` · ${librariesData.error}` : ""}
            </span>
          </p>
        </div>
        <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
          <DialogTrigger asChild>
            <Button className="gap-2">
              <Plus className="h-4 w-4" />
              添加媒体库
            </Button>
          </DialogTrigger>
          <DialogContent className="sm:max-w-lg">
            <DialogHeader>
              <DialogTitle>添加媒体库</DialogTitle>
              <DialogDescription>
                创建新的媒体库来组织你的内容
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="space-y-2">
                <Label>媒体库名称</Label>
                <Input placeholder="例如：家庭电影" />
              </div>
              <div className="space-y-2">
                <Label>媒体类型</Label>
                <Select defaultValue="movie">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {libraryTypes.map((type) => (
                      <SelectItem key={type.value} value={type.value}>
                        <div className="flex items-center gap-2">
                          <type.icon className="h-4 w-4" />
                          {type.label}
                        </div>
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>媒体文件夹</Label>
                <div className="flex gap-2">
                  <Input placeholder="/path/to/media" className="flex-1" />
                  <Button variant="outline" size="icon">
                    <FolderOpen className="h-4 w-4" />
                  </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                  你可以稍后添加更多文件夹
                </p>
              </div>
              <div className="space-y-2">
                <Label>元数据语言</Label>
                <Select defaultValue="zh-CN">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="zh-CN">简体中文</SelectItem>
                    <SelectItem value="zh-TW">繁體中文</SelectItem>
                    <SelectItem value="en">English</SelectItem>
                    <SelectItem value="ja">日本語</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
                取消
              </Button>
              <Button onClick={() => setIsAddDialogOpen(false)}>
                创建媒体库
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* 统计概览 */}
      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <div className="rounded-lg border border-border/50 bg-card/30 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">媒体库</span>
            <FolderOpen className="h-3.5 w-3.5 text-muted-foreground/50" />
          </div>
          <p className="mt-1 text-xl font-semibold">{stats.totalLibraries}</p>
        </div>
        <div className="rounded-lg border border-border/50 bg-card/30 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">总项目</span>
            <Film className="h-3.5 w-3.5 text-muted-foreground/50" />
          </div>
          <p className="mt-1 text-xl font-semibold">{stats.totalItems.toLocaleString()}</p>
        </div>
        <div className="rounded-lg border border-border/50 bg-card/30 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">存储空间</span>
            <HardDrive className="h-3.5 w-3.5 text-muted-foreground/50" />
          </div>
          <p className="mt-1 text-xl font-semibold">{stats.totalSize}</p>
        </div>
        <div className="rounded-lg border border-border/50 bg-card/30 p-3">
          <div className="flex items-center justify-between">
            <span className="text-xs text-muted-foreground">状态</span>
            {stats.errors > 0 || stats.unavailablePaths > 0 ? (
              <AlertTriangle className="h-3.5 w-3.5 text-warning" />
            ) : stats.scanning > 0 ? (
              <Loader2 className="h-3.5 w-3.5 text-info animate-spin" />
            ) : (
              <CheckCircle2 className="h-3.5 w-3.5 text-success" />
            )}
          </div>
          <p className="mt-1 text-xl font-semibold">
            {stats.errors > 0 ? (
              <span className="text-destructive">{stats.errors} 错误</span>
            ) : stats.scanning > 0 ? (
              <span className="text-info">{stats.scanning} 扫描中</span>
            ) : (
              <span className="text-success">正常</span>
            )}
          </p>
          {stats.unavailablePaths > 0 && (
            <p className="text-[10px] text-warning">{stats.unavailablePaths} 路径不可用</p>
          )}
        </div>
      </div>

      {/* 搜索和过滤 */}
      <div className="flex items-center gap-4">
        <div className="relative flex-1 max-w-sm">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input 
            placeholder="搜索媒体库..." 
            className="pl-9"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <Button
          variant="outline"
          className="gap-2"
          disabled={!canMutate || libraryMutation.isPending || libraries.length === 0}
          onClick={() => runLibraryMutation("scan", libraries.map((library) => library.id), "确认扫描全部媒体库？")}
        >
          <RefreshCw className="h-4 w-4" />
          全部扫描
        </Button>
      </div>

      {(mutationMessage || !canMutate) && (
        <div className="rounded-md border border-border/50 bg-muted/30 px-3 py-2 text-xs text-muted-foreground">
          {mutationMessage ?? mutationSource.unavailableReason}
        </div>
      )}

    {/* 媒体库列表 */}
    <div className="grid gap-4 lg:grid-cols-2">
      {filteredLibraries.map((library) => {
        const LibraryIcon = libraryIconByKind[library.type] ?? FolderOpen

        return (
        <Card 
          key={library.id} 
          className={cn(
            "overflow-hidden border-border/50 bg-card/50 backdrop-blur-sm",
            library.scanStatus === "scanning" && "border-l-2 border-l-info",
            library.scanStatus === "error" && "border-l-2 border-l-destructive"
          )}
        >
          <CardContent className="p-0">
            <div className="flex items-stretch">
              {/* 左侧图标区 */}
              <div className={cn(
                "flex w-20 items-center justify-center border-r border-border/30 lg:w-24",
                library.scanStatus === "scanning" ? "bg-info/5" : 
                library.scanStatus === "error" ? "bg-destructive/5" : "bg-secondary/30"
              )}>
                <LibraryIcon className="h-8 w-8 text-muted-foreground/70" />
              </div>
              
              {/* 主内容区 */}
              <div className="flex-1 p-3 lg:p-4">
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0 flex-1">
                      {/* 标题行 */}
                      <div className="flex items-center gap-2 mb-1.5">
                        <h3 className="font-semibold">{library.name}</h3>
                        {getStatusBadge(library)}
                      </div>
                      
                      {library.scanStatus === "error" && (
                        <p className="text-xs text-destructive mb-1.5 truncate">{library.errorMessage}</p>
                      )}
                      
                      {/* 统计信息 - 单行紧凑显示 */}
                      <div className="flex items-center gap-3 text-xs text-muted-foreground mb-2">
                        <span className="flex items-center gap-1 whitespace-nowrap">
                          <Film className="h-3 w-3" />
                          {library.itemCount} 项
                        </span>
                        <span className="flex items-center gap-1 whitespace-nowrap">
                          <HardDrive className="h-3 w-3" />
                          {library.totalSize}
                        </span>
                        <span className="flex items-center gap-1 whitespace-nowrap text-muted-foreground/70">
                          <Clock className="h-3 w-3" />
                          {library.lastScanned.split(" ")[0]}
                        </span>
                      </div>
                      
                      {/* 路径列表 - 带可用性指示 */}
                      <div className="flex flex-wrap items-center gap-1.5">
                        {library.paths.slice(0, 2).map((pathInfo, idx) => (
                          <Badge 
                            key={idx} 
                            variant="outline" 
                            className={cn(
                              "font-mono text-[10px] px-1.5 py-0 gap-1",
                              !pathInfo.available && "border-destructive/50 text-destructive"
                            )}
                          >
                            <span className={cn(
                              "h-1.5 w-1.5 rounded-full",
                              pathInfo.available ? "bg-success" : "bg-destructive"
                            )} />
                            {pathInfo.path}
                          </Badge>
                        ))}
                        {library.paths.length > 2 && (
                          <Badge variant="outline" className="text-[10px] px-1.5 py-0">
                            +{library.paths.length - 2}
                          </Badge>
                        )}
                        <Button variant="ghost" size="sm" className="h-5 text-[10px] px-1.5 gap-0.5">
                          <FolderPlus className="h-2.5 w-2.5" />
                          添加
                        </Button>
                      </div>
                    </div>
                    
                    {/* 操���按钮 */}
                    <div className="flex items-center gap-1 shrink-0">
                      <Button 
                        variant="ghost" 
                        size="icon"
                        className="h-7 w-7"
                        disabled={!canMutate || library.scanStatus === "scanning" || libraryMutation.isPending}
                        onClick={() => runLibraryMutation("scan", [library.id], `确认扫描媒体库「${library.name}」？`)}
                      >
                        <RefreshCw className={`h-3.5 w-3.5 ${library.scanStatus === "scanning" ? "animate-spin" : ""}`} />
                      </Button>
                      <DropdownMenu>
                        <DropdownMenuTrigger asChild>
                          <Button variant="ghost" size="icon" className="h-7 w-7">
                            <MoreHorizontal className="h-3.5 w-3.5" />
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent align="end">
                          <DropdownMenuItem onClick={() => setSelectedLibrary(library)}>
                            <Settings className="h-4 w-4 mr-2" />
                            设置
                          </DropdownMenuItem>
                          <DropdownMenuItem>
                            <Pencil className="h-4 w-4 mr-2" />
                            编辑
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            disabled={!canMutate || libraryMutation.isPending}
                            onClick={() => runLibraryMutation("import-nfo", [library.id], `确认为「${library.name}」导入 NFO？`)}
                          >
                            <Download className="h-4 w-4 mr-2" />
                            导入 NFO
                          </DropdownMenuItem>
                          <DropdownMenuItem
                            disabled={!canMutate || libraryMutation.isPending}
                            onClick={() => runLibraryMutation("export-nfo", [library.id], `确认为「${library.name}」导出 NFO？`)}
                          >
                            <Database className="h-4 w-4 mr-2" />
                            导出 NFO
                          </DropdownMenuItem>
                          <DropdownMenuSeparator />
                          <DropdownMenuItem disabled>
                            <EyeOff className="h-4 w-4 mr-2" />
                            隐藏
                          </DropdownMenuItem>
                          <DropdownMenuItem className="text-destructive" disabled>
                            <Trash2 className="h-4 w-4 mr-2" />
                            删除
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                    </div>
                  </div>
                  
                  {/* 扫描进度 */}
                  {library.scanStatus === "scanning" && (
                    <div className="mt-2">
                      <Progress value={library.scanProgress} className="h-1" />
                    </div>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        )
      })}
      </div>

      {/* 媒体库设置对话框 */}
      <Dialog open={!!selectedLibrary} onOpenChange={() => setSelectedLibrary(null)}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>{selectedLibrary?.name} 设置</DialogTitle>
            <DialogDescription>
              配置媒体库的扫描和元数据选项
            </DialogDescription>
          </DialogHeader>
          
          {selectedLibrary && (
            <Tabs defaultValue="scanning" className="mt-4">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger value="scanning">扫描</TabsTrigger>
                <TabsTrigger value="metadata">元数据</TabsTrigger>
              </TabsList>
              
              <TabsContent value="scanning" className="space-y-4 mt-4">
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>自动扫描</Label>
                    <p className="text-sm text-muted-foreground">
                      定期检查新增和变更的文件
                    </p>
                  </div>
                  <Switch defaultChecked={selectedLibrary.settings.autoScan} />
                </div>
                
                <div className="space-y-2">
                  <Label>扫描间隔</Label>
                  <Select defaultValue={String(selectedLibrary.settings.scanInterval)}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="1">每小时</SelectItem>
                      <SelectItem value="3">每 3 小时</SelectItem>
                      <SelectItem value="6">每 6 小时</SelectItem>
                      <SelectItem value="12">每 12 小时</SelectItem>
                      <SelectItem value="24">每天</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>使用 NFO 文件</Label>
                    <p className="text-sm text-muted-foreground">
                      优先使用本地 NFO 文件中的元数据
                    </p>
                  </div>
                  <Switch defaultChecked={selectedLibrary.settings.useNfo} />
                </div>
              </TabsContent>
              
              <TabsContent value="metadata" className="space-y-4 mt-4">
                <div className="space-y-2">
                  <Label>元数据语言</Label>
                  <Select defaultValue={selectedLibrary.settings.metadataLanguage}>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="zh-CN">简体中文</SelectItem>
                      <SelectItem value="zh-TW">繁體中文</SelectItem>
                      <SelectItem value="en">English</SelectItem>
                      <SelectItem value="ja">日本語</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                
                {/* 元数据提供者选择 */}
                <div className="space-y-3">
                  <div className="flex items-center justify-between">
                    <Label>元数据提供者</Label>
                    <span className="text-xs text-muted-foreground">拖拽调整优先级</span>
                  </div>
                  
                  {/* 可滚动的提供者列表容器 */}
                  <div className="max-h-[240px] overflow-y-auto scrollbar-none space-y-2">
                    {(selectedLibrary.type === "anime" ? [
                      { id: "bangumi", name: "Bangumi", desc: "番组计划 - 动画元数据", enabled: true, installed: true },
                      { id: "anidb", name: "AniDB", desc: "动画数据库", enabled: true, installed: true },
                      { id: "tmdb", name: "TMDb", desc: "The Movie Database", enabled: false, installed: true },
                      { id: "anilist", name: "AniList", desc: "动画追踪与发现", enabled: false, installed: true },
                      { id: "mal", name: "MyAnimeList", desc: "动画列表数据库", enabled: false, installed: false },
                    ] : [
                      { id: "tmdb", name: "TMDb", desc: "The Movie Database", enabled: true, installed: true },
                      { id: "douban", name: "豆瓣", desc: "豆瓣电影 - 中文元数据", enabled: true, installed: true },
                      { id: "imdb", name: "IMDb", desc: "互联网电影数据库", enabled: false, installed: true },
                      { id: "omdb", name: "OMDb", desc: "开放电影数据库", enabled: false, installed: false },
                      { id: "fanart", name: "Fanart.tv", desc: "高清艺术图资源", enabled: false, installed: true },
                      { id: "opensubtitles", name: "OpenSubtitles", desc: "字幕元数据", enabled: false, installed: false },
                    ])
                    .sort((a, b) => {
                      // 已启用的排在前面
                      if (a.enabled !== b.enabled) return a.enabled ? -1 : 1
                      // 已安装的排在未安装的前面
                      if (a.installed !== b.installed) return a.installed ? -1 : 1
                      return 0
                    })
                    .map((provider, index, arr) => {
                      const enabledCount = arr.filter(p => p.enabled).length
                      const displayIndex = provider.enabled ? arr.filter((p, i) => p.enabled && i <= index).length : null
                      
                      return (
                        <div 
                          key={provider.id}
                          className={cn(
                            "flex items-center gap-3 rounded-lg border p-3 transition-colors",
                            provider.enabled 
                              ? "border-border/50 bg-card" 
                              : "border-border/30 bg-muted/30 opacity-70"
                          )}
                        >
                          <GripVertical className={cn(
                            "h-4 w-4 shrink-0",
                            provider.enabled ? "text-muted-foreground/50 cursor-grab" : "text-muted-foreground/30"
                          )} />
                          {displayIndex ? (
                            <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-primary/10 text-xs font-medium text-primary">
                              {displayIndex}
                            </span>
                          ) : (
                            <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded bg-muted text-xs font-medium text-muted-foreground">
                              -
                            </span>
                          )}
                          <div className="flex-1 min-w-0">
                            <div className="flex items-center gap-2">
                              <span className="text-sm font-medium">{provider.name}</span>
                              {!provider.installed && (
                                <Badge variant="outline" className="text-[10px] px-1.5 py-0 text-muted-foreground">
                                  未安装
                                </Badge>
                              )}
                            </div>
                            <p className="text-xs text-muted-foreground truncate">{provider.desc}</p>
                          </div>
                          <Switch 
                            checked={provider.enabled} 
                            disabled={!provider.installed}
                          />
                        </div>
                      )
                    })}
                  </div>
                  
                  <div className="flex items-start gap-2 rounded-lg border border-border/30 bg-muted/20 p-3">
                    <Info className="h-4 w-4 text-muted-foreground mt-0.5 shrink-0" />
                    <p className="text-xs text-muted-foreground">
                      元数据提供者通过插件系统安装。前往 <span className="text-primary font-medium">插件管理</span> 安装更多提供者或配置 API 密钥。
                    </p>
                  </div>
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>下载封面和背景</Label>
                    <p className="text-sm text-muted-foreground">
                      自动下载海报、背景等图片
                    </p>
                  </div>
                  <Switch defaultChecked={selectedLibrary.settings.downloadArt} />
                </div>
                
                <div className="flex items-center justify-between">
                  <div className="space-y-0.5">
                    <Label>保存本地 NFO</Label>
                    <p className="text-sm text-muted-foreground">
                      将刮削的元数据保存为 NFO 文件
                    </p>
                  </div>
                  <Switch />
                </div>
              </TabsContent>
            </Tabs>
          )}
          
          <DialogFooter className="mt-4">
            <Button variant="outline" onClick={() => setSelectedLibrary(null)}>
              取消
            </Button>
            <Button onClick={() => setSelectedLibrary(null)}>
              保存设置
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
