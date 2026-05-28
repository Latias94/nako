"use client"

import { useState } from "react"
import {
  Puzzle,
  Search,
  Download,
  CheckCircle2,
  AlertCircle,
  Settings,
  Trash2,
  ExternalLink,
  RefreshCw,
  Star,
  Package,
  Globe,
  Database,
  Tv,
  Film,
  Music,
  ImageIcon,
  Subtitles,
  MoreHorizontal,
  Play,
  Square,
  RotateCcw,
  Terminal,
  Activity,
  Cpu,
  HardDrive,
  Network,
  Clock,
  ChevronRight,
  Plus,
  Copy,
  Eye,
  EyeOff,
  FileCode,
  Container,
  Server,
  Zap,
  Shield,
  AlertTriangle,
  Info,
  X,
  Check,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
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
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion"
import { cn } from "@/lib/utils"

// Sidecar 插件类型
type PluginStatus = "running" | "stopped" | "error" | "starting" | "stopping"
type PluginType = "container" | "binary" | "script"

interface SidecarPlugin {
  id: string
  name: string
  description: string
  version: string
  author: string
  category: string
  type: PluginType
  status: PluginStatus
  enabled: boolean
  official: boolean
  hasUpdate: boolean
  updateVersion?: string
  // Sidecar 特有字段
  endpoint: string
  port: number
  healthCheck: string
  resources: {
    cpu: string
    memory: string
    uptime: string
  }
  capabilities: string[]
  config: Record<string, unknown>
  logs?: string[]
  containerImage?: string
  binaryPath?: string
}

// 已安装的 Sidecar 插件
const installedPlugins: SidecarPlugin[] = [
  {
    id: "tmdb-provider",
    name: "TMDb Provider",
    description: "从 The Movie Database 获取电影和剧集元数据，支持多语言",
    version: "2.1.0",
    author: "Nako Team",
    category: "metadata",
    type: "container",
    status: "running",
    enabled: true,
    official: true,
    hasUpdate: false,
    endpoint: "http://localhost:9001",
    port: 9001,
    healthCheck: "/health",
    resources: { cpu: "2.3%", memory: "128 MB", uptime: "3d 12h" },
    capabilities: ["metadata.movie", "metadata.tv", "artwork"],
    config: { apiKey: "sk-***", language: "zh-CN", includeAdult: false },
    containerImage: "ghcr.io/nako/tmdb-provider:2.1.0",
  },
  {
    id: "subtitle-finder",
    name: "Subtitle Finder",
    description: "自动搜索和下载字幕，支持 OpenSubtitles、SubHD 等多个源",
    version: "1.5.2",
    author: "Nako Team",
    category: "subtitles",
    type: "container",
    status: "running",
    enabled: true,
    official: true,
    hasUpdate: true,
    updateVersion: "1.6.0",
    endpoint: "http://localhost:9002",
    port: 9002,
    healthCheck: "/health",
    resources: { cpu: "0.5%", memory: "64 MB", uptime: "3d 12h" },
    capabilities: ["subtitle.search", "subtitle.download"],
    config: { providers: ["opensubtitles", "subhd"], preferredLanguages: ["zh-CN", "en"] },
    containerImage: "ghcr.io/nako/subtitle-finder:1.5.2",
  },
  {
    id: "transcode-worker",
    name: "Transcode Worker",
    description: "硬件加速转码服务，支持 NVENC/QSV/VAAPI",
    version: "3.0.0",
    author: "Nako Team",
    category: "transcoding",
    type: "container",
    status: "running",
    enabled: true,
    official: true,
    hasUpdate: false,
    endpoint: "http://localhost:9003",
    port: 9003,
    healthCheck: "/health",
    resources: { cpu: "45.2%", memory: "2.1 GB", uptime: "3d 12h" },
    capabilities: ["transcode.video", "transcode.audio", "hardware.nvenc"],
    config: { hwAccel: "nvenc", maxJobs: 2, outputPresets: ["1080p", "720p", "480p"] },
    containerImage: "ghcr.io/nako/transcode-worker:3.0.0",
  },
  {
    id: "anidb-provider",
    name: "AniDB Provider",
    description: "动画元数据提供者，支持动画识别和关联",
    version: "2.0.1",
    author: "Community",
    category: "metadata",
    type: "container",
    status: "stopped",
    enabled: false,
    official: false,
    hasUpdate: false,
    endpoint: "http://localhost:9004",
    port: 9004,
    healthCheck: "/health",
    resources: { cpu: "0%", memory: "0 MB", uptime: "-" },
    capabilities: ["metadata.anime"],
    config: { clientId: "", clientVersion: "1" },
    containerImage: "ghcr.io/nako-community/anidb-provider:2.0.1",
  },
  {
    id: "intro-detector",
    name: "Intro Detector",
    description: "使用音频指纹检测片头片尾，支持自动跳过",
    version: "1.2.0",
    author: "Nako Team",
    category: "analysis",
    type: "binary",
    status: "error",
    enabled: true,
    official: true,
    hasUpdate: false,
    endpoint: "http://localhost:9005",
    port: 9005,
    healthCheck: "/health",
    resources: { cpu: "0%", memory: "0 MB", uptime: "-" },
    capabilities: ["analyze.intro", "analyze.credits"],
    config: { minDuration: 15, maxDuration: 180 },
    binaryPath: "/opt/nako/plugins/intro-detector",
    logs: [
      "[ERROR] Failed to connect to Chromaprint library",
      "[INFO] Retrying in 30 seconds...",
      "[ERROR] Connection refused: localhost:9005",
    ],
  },
  {
    id: "webhook-notify",
    name: "Webhook Notifier",
    description: "发送媒体事件到 Discord、Slack、自定义 Webhook",
    version: "1.1.0",
    author: "Community",
    category: "notification",
    type: "script",
    status: "running",
    enabled: true,
    official: false,
    hasUpdate: false,
    endpoint: "http://localhost:9006",
    port: 9006,
    healthCheck: "/health",
    resources: { cpu: "0.1%", memory: "32 MB", uptime: "2d 8h" },
    capabilities: ["notify.playback", "notify.library", "notify.system"],
    config: { webhooks: [{ name: "Discord", url: "https://discord.com/api/webhooks/..." }] },
    binaryPath: "/opt/nako/plugins/webhook-notify/main.py",
  },
]

// 插件市场
const marketplacePlugins = [
  {
    id: "douban-provider",
    name: "豆瓣 Provider",
    description: "从豆瓣获取中文电影和剧集元数据、评分和短评",
    version: "1.0.0",
    author: "Community",
    category: "metadata",
    type: "container" as PluginType,
    official: false,
    downloads: 12500,
    rating: 4.5,
    capabilities: ["metadata.movie", "metadata.tv", "rating"],
    containerImage: "ghcr.io/nako-community/douban-provider:1.0.0",
    requirements: { minVersion: "1.0.0", resources: "128MB RAM" },
  },
  {
    id: "trakt-sync",
    name: "Trakt Sync",
    description: "双向同步观看历史、收藏和评分到 Trakt.tv",
    version: "2.3.0",
    author: "Nako Team",
    category: "sync",
    type: "container" as PluginType,
    official: true,
    downloads: 8900,
    rating: 4.8,
    capabilities: ["sync.history", "sync.collection", "sync.ratings"],
    containerImage: "ghcr.io/nako/trakt-sync:2.3.0",
    requirements: { minVersion: "1.0.0", resources: "64MB RAM" },
  },
  {
    id: "plex-import",
    name: "Plex Importer",
    description: "从 Plex 导入媒体库、观看历史和用户数据",
    version: "1.0.0",
    author: "Community",
    category: "migration",
    type: "binary" as PluginType,
    official: false,
    downloads: 3200,
    rating: 4.2,
    capabilities: ["import.library", "import.history", "import.users"],
    requirements: { minVersion: "1.0.0", resources: "256MB RAM" },
  },
  {
    id: "ldap-auth",
    name: "LDAP Auth",
    description: "使用 LDAP/Active Directory 进行用户认证",
    version: "1.2.0",
    author: "Nako Team",
    category: "authentication",
    type: "container" as PluginType,
    official: true,
    downloads: 5600,
    rating: 4.6,
    capabilities: ["auth.ldap", "auth.ad", "sync.users"],
    containerImage: "ghcr.io/nako/ldap-auth:1.2.0",
    requirements: { minVersion: "1.0.0", resources: "64MB RAM" },
  },
  {
    id: "ai-tagger",
    name: "AI Tagger",
    description: "使用 AI 自动识别和标记媒体内容、生成描述",
    version: "0.9.0",
    author: "Nako Team",
    category: "analysis",
    type: "container" as PluginType,
    official: true,
    downloads: 2100,
    rating: 4.3,
    capabilities: ["analyze.content", "tag.auto", "describe.auto"],
    containerImage: "ghcr.io/nako/ai-tagger:0.9.0",
    requirements: { minVersion: "1.2.0", resources: "2GB RAM, GPU recommended" },
  },
]

// 插件分类
const categories = [
  { id: "all", name: "全部", icon: Package },
  { id: "metadata", name: "元数据", icon: Database },
  { id: "subtitles", name: "字幕", icon: Subtitles },
  { id: "transcoding", name: "转码", icon: Cpu },
  { id: "analysis", name: "分析", icon: Activity },
  { id: "notification", name: "通知", icon: Globe },
  { id: "sync", name: "同步", icon: RefreshCw },
  { id: "authentication", name: "认证", icon: Shield },
]

const statusColors: Record<PluginStatus, string> = {
  running: "bg-emerald-500",
  stopped: "bg-zinc-400",
  error: "bg-red-500",
  starting: "bg-amber-500",
  stopping: "bg-amber-500",
}

const statusLabels: Record<PluginStatus, string> = {
  running: "运行中",
  stopped: "已停止",
  error: "错误",
  starting: "启动中",
  stopping: "停止中",
}

const typeIcons: Record<PluginType, typeof Container> = {
  container: Container,
  binary: FileCode,
  script: Terminal,
}

const typeLabels: Record<PluginType, string> = {
  container: "容器",
  binary: "二进制",
  script: "脚本",
}

export function AdminPlugins() {
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedCategory, setSelectedCategory] = useState("all")
  const [plugins, setPlugins] = useState(installedPlugins)
  const [selectedPlugin, setSelectedPlugin] = useState<SidecarPlugin | null>(null)
  const [showAddDialog, setShowAddDialog] = useState(false)
  const [showLogsDialog, setShowLogsDialog] = useState(false)
  const [logsPlugin, setLogsPlugin] = useState<SidecarPlugin | null>(null)

  const togglePlugin = (id: string) => {
    setPlugins(prev => prev.map(p => {
      if (p.id === id) {
        const newEnabled = !p.enabled
        return { 
          ...p, 
          enabled: newEnabled,
          status: newEnabled ? "starting" : "stopping"
        }
      }
      return p
    }))
    // 模拟状态变化
    setTimeout(() => {
      setPlugins(prev => prev.map(p => {
        if (p.id === id) {
          return { 
            ...p, 
            status: p.enabled ? "running" : "stopped",
            resources: p.enabled ? p.resources : { cpu: "0%", memory: "0 MB", uptime: "-" }
          }
        }
        return p
      }))
    }, 1500)
  }

  const restartPlugin = (id: string) => {
    setPlugins(prev => prev.map(p => 
      p.id === id ? { ...p, status: "starting" } : p
    ))
    setTimeout(() => {
      setPlugins(prev => prev.map(p => 
        p.id === id ? { ...p, status: "running" } : p
      ))
    }, 2000)
  }

  const filteredInstalled = plugins.filter(p => 
    (selectedCategory === "all" || p.category === selectedCategory) &&
    (p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
     p.description.toLowerCase().includes(searchQuery.toLowerCase()))
  )

  const filteredMarketplace = marketplacePlugins.filter(p => 
    (selectedCategory === "all" || p.category === selectedCategory) &&
    (p.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
     p.description.toLowerCase().includes(searchQuery.toLowerCase()))
  )

  const runningCount = plugins.filter(p => p.status === "running").length
  const errorCount = plugins.filter(p => p.status === "error").length
  const hasUpdates = plugins.filter(p => p.hasUpdate).length

  return (
    <>
      {/* Page Header */}
      <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div>
          <h1 className="text-xl font-semibold text-foreground">插件管理</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            Sidecar 模式插件 - 独立运行，通过 API 通信
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {/* Status Overview */}
          <div className="mr-4 flex items-center gap-4 text-sm">
            <span className="flex items-center gap-1.5">
              <span className="h-2 w-2 rounded-full bg-emerald-500" />
              {runningCount} 运行中
            </span>
            {errorCount > 0 && (
              <span className="flex items-center gap-1.5 text-red-500">
                <span className="h-2 w-2 rounded-full bg-red-500" />
                {errorCount} 错误
              </span>
            )}
          </div>
          {hasUpdates > 0 && (
            <Button variant="outline" size="sm">
              <Download className="mr-2 h-3.5 w-3.5" />
              更新全部 ({hasUpdates})
            </Button>
          )}
          <Button size="sm" onClick={() => setShowAddDialog(true)}>
            <Plus className="mr-2 h-3.5 w-3.5" />
            添加插件
          </Button>
        </div>
      </div>

      {/* Search and Filter */}
      <div className="mb-6 flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
        <div className="relative w-full max-w-sm">
          <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
          <Input 
            placeholder="搜索插件..." 
            className="pl-9"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <div className="flex flex-wrap gap-2">
          {categories.slice(0, 6).map((cat) => (
            <Button
              key={cat.id}
              variant={selectedCategory === cat.id ? "default" : "outline"}
              size="sm"
              onClick={() => setSelectedCategory(cat.id)}
            >
              {cat.name}
            </Button>
          ))}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="outline" size="sm">
                更多
                <ChevronRight className="ml-1 h-3 w-3" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent>
              {categories.slice(6).map((cat) => (
                <DropdownMenuItem 
                  key={cat.id}
                  onClick={() => setSelectedCategory(cat.id)}
                >
                  {cat.name}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      <Tabs defaultValue="installed" className="space-y-6">
        <TabsList>
          <TabsTrigger value="installed">
            已安装
            <Badge variant="secondary" className="ml-2">{plugins.length}</Badge>
          </TabsTrigger>
          <TabsTrigger value="marketplace">
            插件市场
          </TabsTrigger>
          <TabsTrigger value="develop">
            开发
          </TabsTrigger>
        </TabsList>

        {/* 已安装插件 */}
        <TabsContent value="installed" className="space-y-4">
          {filteredInstalled.length === 0 ? (
            <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-12">
              <Puzzle className="h-12 w-12 text-muted-foreground/50" />
              <p className="mt-4 text-sm text-muted-foreground">没有找到匹配的插件</p>
            </div>
          ) : (
            <div className="space-y-3">
              {filteredInstalled.map((plugin) => (
                <SidecarPluginCard 
                  key={plugin.id} 
                  plugin={plugin}
                  onToggle={() => togglePlugin(plugin.id)}
                  onRestart={() => restartPlugin(plugin.id)}
                  onSettings={() => setSelectedPlugin(plugin)}
                  onLogs={() => {
                    setLogsPlugin(plugin)
                    setShowLogsDialog(true)
                  }}
                />
              ))}
            </div>
          )}
        </TabsContent>

        {/* 插件市场 */}
        <TabsContent value="marketplace" className="space-y-4">
          {filteredMarketplace.length === 0 ? (
            <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border py-12">
              <Package className="h-12 w-12 text-muted-foreground/50" />
              <p className="mt-4 text-sm text-muted-foreground">没有找到匹配的插件</p>
            </div>
          ) : (
            <div className="grid gap-4 lg:grid-cols-2">
              {filteredMarketplace.map((plugin) => (
                <MarketplacePluginCard key={plugin.id} plugin={plugin} />
              ))}
            </div>
          )}
        </TabsContent>

        {/* 开发者选项 */}
        <TabsContent value="develop" className="space-y-6">
          <div className="rounded-lg border border-border bg-card p-6">
            <h3 className="text-base font-medium">开发自定义插件</h3>
            <p className="mt-1 text-sm text-muted-foreground">
              Nako 使用 Sidecar 模式，插件作为独立服务运行，通过 gRPC/REST API 与主应用通信。
            </p>
            
            <div className="mt-6 grid gap-4 lg:grid-cols-3">
              <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                <Container className="h-8 w-8 text-primary" />
                <h4 className="mt-3 font-medium">容器插件</h4>
                <p className="mt-1 text-xs text-muted-foreground">
                  打包为 Docker 镜像，支持任何语言和运行时
                </p>
              </div>
              <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                <FileCode className="h-8 w-8 text-primary" />
                <h4 className="mt-3 font-medium">二进制插件</h4>
                <p className="mt-1 text-xs text-muted-foreground">
                  编译后的可执行文件，高性能，低资源占用
                </p>
              </div>
              <div className="rounded-lg border border-border/50 bg-muted/30 p-4">
                <Terminal className="h-8 w-8 text-primary" />
                <h4 className="mt-3 font-medium">脚本插件</h4>
                <p className="mt-1 text-xs text-muted-foreground">
                  Python/Node.js 脚本，快速开发和迭代
                </p>
              </div>
            </div>

            <div className="mt-6 flex flex-wrap gap-3">
              <Button variant="outline" size="sm">
                <FileCode className="mr-2 h-4 w-4" />
                查看 SDK 文档
              </Button>
              <Button variant="outline" size="sm">
                <Package className="mr-2 h-4 w-4" />
                示例项目
              </Button>
              <Button variant="outline" size="sm">
                <Globe className="mr-2 h-4 w-4" />
                API 参考
              </Button>
            </div>
          </div>

          {/* 插件接口说明 */}
          <div className="rounded-lg border border-border bg-card">
            <Accordion type="single" collapsible className="w-full">
              <AccordionItem value="api" className="border-none">
                <AccordionTrigger className="px-6 hover:no-underline">
                  <span className="flex items-center gap-2">
                    <Zap className="h-4 w-4" />
                    插件 API 接口
                  </span>
                </AccordionTrigger>
                <AccordionContent className="px-6 pb-4">
                  <div className="rounded-lg bg-zinc-950 p-4 font-mono text-xs text-zinc-300">
                    <pre>{`// 插件必须实现的接口
interface NakoPlugin {
  // 插件元信息
  manifest: PluginManifest
  
  // 生命周期
  onLoad(): Promise<void>
  onUnload(): Promise<void>
  
  // 健康检查
  healthCheck(): Promise<HealthStatus>
  
  // 能力处理
  handleCapability(
    capability: string, 
    request: CapabilityRequest
  ): Promise<CapabilityResponse>
}`}</pre>
                  </div>
                </AccordionContent>
              </AccordionItem>
              <AccordionItem value="manifest" className="border-none">
                <AccordionTrigger className="px-6 hover:no-underline">
                  <span className="flex items-center gap-2">
                    <FileCode className="h-4 w-4" />
                    插件清单 (manifest.json)
                  </span>
                </AccordionTrigger>
                <AccordionContent className="px-6 pb-4">
                  <div className="rounded-lg bg-zinc-950 p-4 font-mono text-xs text-zinc-300">
                    <pre>{`{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "A custom Nako plugin",
  "author": "Your Name",
  "type": "container",
  "capabilities": [
    "metadata.movie",
    "metadata.tv"
  ],
  "config": {
    "apiKey": {
      "type": "string",
      "required": true,
      "secret": true
    }
  },
  "resources": {
    "memory": "128MB",
    "cpu": "0.5"
  }
}`}</pre>
                  </div>
                </AccordionContent>
              </AccordionItem>
            </Accordion>
          </div>
        </TabsContent>
      </Tabs>

      {/* 插件设置对话框 */}
      {selectedPlugin && (
        <PluginSettingsDialog 
          plugin={selectedPlugin} 
          onClose={() => setSelectedPlugin(null)} 
        />
      )}

      {/* 添加插件对话框 */}
      <AddPluginDialog 
        open={showAddDialog} 
        onOpenChange={setShowAddDialog} 
      />

      {/* 日志对话框 */}
      {logsPlugin && (
        <PluginLogsDialog
          plugin={logsPlugin}
          open={showLogsDialog}
          onOpenChange={setShowLogsDialog}
        />
      )}
    </>
  )
}

// Sidecar 插件卡片
function SidecarPluginCard({ 
  plugin, 
  onToggle,
  onRestart,
  onSettings,
  onLogs,
}: { 
  plugin: SidecarPlugin
  onToggle: () => void
  onRestart: () => void
  onSettings: () => void
  onLogs: () => void
}) {
  const TypeIcon = typeIcons[plugin.type]

  return (
    <div className={cn(
      "rounded-lg border bg-card transition-colors",
      plugin.status === "error" ? "border-red-500/50" : "border-border/50"
    )}>
      {/* 主要信息行 */}
      <div className="flex items-center justify-between gap-4 p-4">
        <div className="flex items-center gap-3">
          {/* 状态指示器 */}
          <div className="relative">
            <div className={cn(
              "flex h-10 w-10 items-center justify-center rounded-lg",
              plugin.enabled ? "bg-primary/10" : "bg-muted"
            )}>
              <TypeIcon className={cn(
                "h-5 w-5",
                plugin.enabled ? "text-primary" : "text-muted-foreground"
              )} />
            </div>
            <span className={cn(
              "absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-card",
              statusColors[plugin.status]
            )} />
          </div>

          {/* 插件信息 */}
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-medium">{plugin.name}</h3>
              {plugin.official && (
                <Badge variant="secondary" className="text-[10px]">官方</Badge>
              )}
              <Badge variant="outline" className="text-[10px]">
                {typeLabels[plugin.type]}
              </Badge>
              {plugin.hasUpdate && (
                <Badge className="text-[10px]">
                  v{plugin.updateVersion}
                </Badge>
              )}
            </div>
            <p className="mt-0.5 text-xs text-muted-foreground line-clamp-1">
              {plugin.description}
            </p>
          </div>
        </div>

        {/* 资源使用和控制 */}
        <div className="flex items-center gap-4">
          {/* 资源指标 */}
          {plugin.status === "running" && (
            <div className="hidden items-center gap-4 text-xs text-muted-foreground lg:flex">
              <span className="flex items-center gap-1">
                <Cpu className="h-3 w-3" />
                {plugin.resources.cpu}
              </span>
              <span className="flex items-center gap-1">
                <HardDrive className="h-3 w-3" />
                {plugin.resources.memory}
              </span>
              <span className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                {plugin.resources.uptime}
              </span>
            </div>
          )}

          {/* 状态标签 */}
          <Badge 
            variant={plugin.status === "error" ? "destructive" : "outline"}
            className="min-w-[60px] justify-center"
          >
            {statusLabels[plugin.status]}
          </Badge>

          {/* 控制按钮 */}
          <div className="flex items-center gap-1">
            {plugin.status === "running" && (
              <Button 
                variant="ghost" 
                size="icon" 
                className="h-8 w-8"
                onClick={onRestart}
              >
                <RotateCcw className="h-4 w-4" />
              </Button>
            )}
            <Switch 
              checked={plugin.enabled} 
              onCheckedChange={onToggle}
              disabled={plugin.status === "starting" || plugin.status === "stopping"}
            />
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <MoreHorizontal className="h-4 w-4" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuItem onClick={onSettings}>
                  <Settings className="mr-2 h-4 w-4" />
                  配置
                </DropdownMenuItem>
                <DropdownMenuItem onClick={onLogs}>
                  <Terminal className="mr-2 h-4 w-4" />
                  查看日志
                </DropdownMenuItem>
                <DropdownMenuItem>
                  <Network className="mr-2 h-4 w-4" />
                  API 端点
                </DropdownMenuItem>
                {plugin.hasUpdate && (
                  <DropdownMenuItem>
                    <Download className="mr-2 h-4 w-4" />
                    更新到 v{plugin.updateVersion}
                  </DropdownMenuItem>
                )}
                <DropdownMenuSeparator />
                <DropdownMenuItem>
                  <ExternalLink className="mr-2 h-4 w-4" />
                  文档
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem className="text-destructive">
                  <Trash2 className="mr-2 h-4 w-4" />
                  卸载
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </div>

      {/* 错误信息 */}
      {plugin.status === "error" && plugin.logs && (
        <div className="border-t border-red-500/30 bg-red-500/5 px-4 py-2">
          <p className="font-mono text-xs text-red-400">
            {plugin.logs[plugin.logs.length - 1]}
          </p>
        </div>
      )}
    </div>
  )
}

// 市场插件卡片
function MarketplacePluginCard({ plugin }: { plugin: typeof marketplacePlugins[0] }) {
  const [installing, setInstalling] = useState(false)
  const TypeIcon = typeIcons[plugin.type]

  const handleInstall = () => {
    setInstalling(true)
    setTimeout(() => setInstalling(false), 3000)
  }

  return (
    <div className="rounded-lg border border-border/50 bg-card p-4">
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-3">
          <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-muted">
            <TypeIcon className="h-5 w-5 text-muted-foreground" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-medium">{plugin.name}</h3>
              {plugin.official && (
                <Badge variant="secondary" className="text-[10px]">官方</Badge>
              )}
            </div>
            <p className="mt-0.5 text-xs text-muted-foreground line-clamp-2">
              {plugin.description}
            </p>
            <div className="mt-2 flex flex-wrap items-center gap-2">
              {plugin.capabilities.slice(0, 3).map((cap) => (
                <Badge key={cap} variant="outline" className="text-[10px]">
                  {cap}
                </Badge>
              ))}
            </div>
            <div className="mt-2 flex items-center gap-3 text-xs text-muted-foreground">
              <span>v{plugin.version}</span>
              <span className="flex items-center gap-1">
                <Star className="h-3 w-3 fill-amber-400 text-amber-400" />
                {plugin.rating}
              </span>
              <span className="flex items-center gap-1">
                <Download className="h-3 w-3" />
                {plugin.downloads.toLocaleString()}
              </span>
            </div>
          </div>
        </div>

        <Button 
          size="sm" 
          onClick={handleInstall}
          disabled={installing}
        >
          {installing ? (
            <>
              <RefreshCw className="mr-2 h-3.5 w-3.5 animate-spin" />
              安装中
            </>
          ) : (
            <>
              <Download className="mr-2 h-3.5 w-3.5" />
              安装
            </>
          )}
        </Button>
      </div>
    </div>
  )
}

// 插件设置对话框
function PluginSettingsDialog({ 
  plugin, 
  onClose 
}: { 
  plugin: SidecarPlugin
  onClose: () => void 
}) {
  const [showSecrets, setShowSecrets] = useState<Record<string, boolean>>({})

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            {plugin.name} 配置
          </DialogTitle>
          <DialogDescription>
            配置插件参数和连接设置
          </DialogDescription>
        </DialogHeader>

        <Tabs defaultValue="general" className="mt-4">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="general">常规</TabsTrigger>
            <TabsTrigger value="connection">连接</TabsTrigger>
            <TabsTrigger value="advanced">高级</TabsTrigger>
          </TabsList>

          <TabsContent value="general" className="space-y-4 pt-4">
            {/* 根据 plugin.config 动态生成配置项 */}
            {Object.entries(plugin.config).map(([key, value]) => (
              <div key={key} className="space-y-2">
                <Label htmlFor={key} className="capitalize">
                  {key.replace(/([A-Z])/g, ' $1').trim()}
                </Label>
                {typeof value === "boolean" ? (
                  <Switch id={key} checked={value} />
                ) : typeof value === "string" && key.toLowerCase().includes("key") ? (
                  <div className="relative">
                    <Input 
                      id={key}
                      type={showSecrets[key] ? "text" : "password"}
                      defaultValue={value as string}
                      className="pr-10"
                    />
                    <Button
                      variant="ghost"
                      size="icon"
                      className="absolute right-1 top-1/2 h-7 w-7 -translate-y-1/2"
                      onClick={() => setShowSecrets(prev => ({ ...prev, [key]: !prev[key] }))}
                    >
                      {showSecrets[key] ? (
                        <EyeOff className="h-4 w-4" />
                      ) : (
                        <Eye className="h-4 w-4" />
                      )}
                    </Button>
                  </div>
                ) : Array.isArray(value) ? (
                  <div className="space-y-2">
                    {(value as string[]).map((item, i) => (
                      <div key={i} className="flex items-center gap-2">
                        <Input defaultValue={typeof item === 'string' ? item : JSON.stringify(item)} />
                        <Button variant="ghost" size="icon" className="h-8 w-8 shrink-0">
                          <X className="h-4 w-4" />
                        </Button>
                      </div>
                    ))}
                    <Button variant="outline" size="sm">
                      <Plus className="mr-2 h-4 w-4" />
                      添加
                    </Button>
                  </div>
                ) : (
                  <Input id={key} defaultValue={String(value)} />
                )}
              </div>
            ))}
          </TabsContent>

          <TabsContent value="connection" className="space-y-4 pt-4">
            <div className="space-y-2">
              <Label>API 端点</Label>
              <div className="flex items-center gap-2">
                <Input value={plugin.endpoint} readOnly className="font-mono text-sm" />
                <Button variant="outline" size="icon">
                  <Copy className="h-4 w-4" />
                </Button>
              </div>
            </div>
            <div className="space-y-2">
              <Label>端口</Label>
              <Input type="number" defaultValue={plugin.port} />
            </div>
            <div className="space-y-2">
              <Label>健康检查路径</Label>
              <Input defaultValue={plugin.healthCheck} className="font-mono" />
            </div>
            {plugin.containerImage && (
              <div className="space-y-2">
                <Label>容器镜像</Label>
                <Input value={plugin.containerImage} readOnly className="font-mono text-sm" />
              </div>
            )}
          </TabsContent>

          <TabsContent value="advanced" className="space-y-4 pt-4">
            <div className="space-y-2">
              <Label>能力 (Capabilities)</Label>
              <div className="flex flex-wrap gap-2">
                {plugin.capabilities.map((cap) => (
                  <Badge key={cap} variant="outline">{cap}</Badge>
                ))}
              </div>
            </div>
            <div className="space-y-2">
              <Label>资源限制</Label>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <Label className="text-xs text-muted-foreground">CPU</Label>
                  <Input defaultValue="0.5" />
                </div>
                <div>
                  <Label className="text-xs text-muted-foreground">内存</Label>
                  <Input defaultValue="256MB" />
                </div>
              </div>
            </div>
            <div className="space-y-2">
              <Label>环境变量</Label>
              <Textarea 
                placeholder="KEY=VALUE&#10;ANOTHER_KEY=VALUE" 
                className="font-mono text-sm"
                rows={4}
              />
            </div>
          </TabsContent>
        </Tabs>

        <DialogFooter className="mt-6">
          <Button variant="outline" onClick={onClose}>取消</Button>
          <Button onClick={onClose}>保存配置</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

// 添加插件对话框
function AddPluginDialog({ 
  open, 
  onOpenChange 
}: { 
  open: boolean
  onOpenChange: (open: boolean) => void 
}) {
  const [addMethod, setAddMethod] = useState<"marketplace" | "url" | "local">("marketplace")

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>添加插件</DialogTitle>
          <DialogDescription>
            从市场安装或手动添加 Sidecar 插件
          </DialogDescription>
        </DialogHeader>

        <Tabs value={addMethod} onValueChange={(v) => setAddMethod(v as typeof addMethod)} className="mt-4">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="marketplace">市场</TabsTrigger>
            <TabsTrigger value="url">URL</TabsTrigger>
            <TabsTrigger value="local">本地</TabsTrigger>
          </TabsList>

          <TabsContent value="marketplace" className="pt-4">
            <p className="text-sm text-muted-foreground">
              前往插件市场标签页浏览和安装插件
            </p>
          </TabsContent>

          <TabsContent value="url" className="space-y-4 pt-4">
            <div className="space-y-2">
              <Label>插件 URL</Label>
              <Input placeholder="https://github.com/user/nako-plugin/releases/latest" />
              <p className="text-xs text-muted-foreground">
                支持 GitHub Release、Docker Hub 或直接 URL
              </p>
            </div>
            <div className="space-y-2">
              <Label>插件类型</Label>
              <Select defaultValue="container">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="container">容器镜像</SelectItem>
                  <SelectItem value="binary">二进制文件</SelectItem>
                  <SelectItem value="script">脚本</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </TabsContent>

          <TabsContent value="local" className="space-y-4 pt-4">
            <div className="space-y-2">
              <Label>本地路径</Label>
              <Input placeholder="/opt/nako/plugins/my-plugin" />
            </div>
            <div className="space-y-2">
              <Label>清单文件</Label>
              <Input placeholder="manifest.json" />
            </div>
            <div className="rounded-lg border border-dashed border-border bg-muted/30 p-6 text-center">
              <Package className="mx-auto h-8 w-8 text-muted-foreground" />
              <p className="mt-2 text-sm text-muted-foreground">
                或拖拽插件包到此处
              </p>
            </div>
          </TabsContent>
        </Tabs>

        <DialogFooter className="mt-6">
          <Button variant="outline" onClick={() => onOpenChange(false)}>取消</Button>
          <Button disabled={addMethod === "marketplace"}>
            <Plus className="mr-2 h-4 w-4" />
            添加
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

// 日志对话框
function PluginLogsDialog({
  plugin,
  open,
  onOpenChange,
}: {
  plugin: SidecarPlugin
  open: boolean
  onOpenChange: (open: boolean) => void
}) {
  const mockLogs = [
    { time: "2024-01-15 10:23:45", level: "INFO", message: "Plugin started successfully" },
    { time: "2024-01-15 10:23:46", level: "INFO", message: `Listening on ${plugin.endpoint}` },
    { time: "2024-01-15 10:24:01", level: "DEBUG", message: "Health check passed" },
    { time: "2024-01-15 10:25:12", level: "INFO", message: "Processing request: metadata.movie" },
    { time: "2024-01-15 10:25:13", level: "DEBUG", message: "Cache hit for movie ID: 12345" },
    { time: "2024-01-15 10:30:00", level: "WARN", message: "Rate limit approaching (80%)" },
    ...(plugin.logs?.map(log => ({ 
      time: new Date().toISOString().replace('T', ' ').slice(0, 19), 
      level: log.startsWith('[ERROR]') ? 'ERROR' : log.startsWith('[WARN]') ? 'WARN' : 'INFO',
      message: log.replace(/^\[(ERROR|WARN|INFO|DEBUG)\]\s*/, '')
    })) || []),
  ]

  const levelColors: Record<string, string> = {
    ERROR: "text-red-400",
    WARN: "text-amber-400",
    INFO: "text-emerald-400",
    DEBUG: "text-zinc-500",
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Terminal className="h-5 w-5" />
            {plugin.name} 日志
          </DialogTitle>
        </DialogHeader>

        <div className="mt-4 flex items-center gap-2">
          <Select defaultValue="all">
            <SelectTrigger className="w-32">
              <SelectValue placeholder="级别" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部</SelectItem>
              <SelectItem value="error">ERROR</SelectItem>
              <SelectItem value="warn">WARN</SelectItem>
              <SelectItem value="info">INFO</SelectItem>
              <SelectItem value="debug">DEBUG</SelectItem>
            </SelectContent>
          </Select>
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
            <Input placeholder="搜索日志..." className="pl-9" />
          </div>
          <Button variant="outline" size="sm">
            <Download className="mr-2 h-4 w-4" />
            导出
          </Button>
        </div>

        <div className="mt-4 max-h-96 overflow-auto rounded-lg bg-zinc-950 p-4 font-mono text-xs">
          {mockLogs.map((log, i) => (
            <div key={i} className="flex gap-3 py-0.5">
              <span className="text-zinc-600">{log.time}</span>
              <span className={cn("w-12", levelColors[log.level])}>[{log.level}]</span>
              <span className="text-zinc-300">{log.message}</span>
            </div>
          ))}
        </div>

        <DialogFooter className="mt-4">
          <Button variant="outline" onClick={() => onOpenChange(false)}>关闭</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
