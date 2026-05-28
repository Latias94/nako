"use client"
import { resolveArtwork } from '@/lib/artwork'

import { useState, useRef, useEffect, useMemo } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Textarea } from "@/components/ui/textarea"
import { Input } from "@/components/ui/input"
import { Switch } from "@/components/ui/switch"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  ArrowLeft,
  Send,
  Mic,
  Paperclip,
  Bot,
  Sparkles,
  Settings,
  Copy,
  RotateCcw,
  RefreshCw,
  Search,
  Download,
  FolderSync,
  Subtitles,
  Film,
  ListPlus,
  CheckCircle2,
  Loader2,
  AlertCircle,
  ChevronDown,
  ChevronRight,
  Plus,
  Zap,
  Clock,
  MessageSquare,
  Trash2,
  Edit3,
  Globe,
  Brain,
  Check,
  X,
  PanelLeftClose,
  PanelLeft,
  Star,
  MoreHorizontal,
  Ellipsis,
  Eye,
  EyeOff,
  TestTube,
  Plug,
  ChevronUp,
  Grip,
  LayoutGrid
} from "lucide-react"
import { cn } from "@/lib/utils"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

interface AgentAssistantProps {
  onBack: () => void
  onNavigate?: (view: string, params?: Record<string, unknown>) => void
}

// LLM 提供商
interface LLMProvider {
  id: string
  name: string
  icon: string
  apiKeySet: boolean
  apiKey?: string
  baseUrl?: string
  models: LLMModel[]
  defaultModel?: string
  isDefault?: boolean
}

interface LLMModel {
  id: string
  name: string
  description?: string
  contextLength?: number
  pricing?: string
  enabled: boolean
}

// Skill 定义
interface Skill {
  id: string
  name: string
  description: string
  icon: React.ReactNode
  category: "resource" | "media" | "library" | "smart"
  enabled: boolean
  source: "builtin" | "plugin"
  pluginName?: string
}

// 消息类型 - 扩展支持多种消息模板
interface ChatMessage {
  id: string
  role: "user" | "assistant"
  content: string
  timestamp: Date
  toolCalls?: ToolCall[]
  thinking?: string
  suggestions?: string[]
  // 扩展消息模板
  template?: MessageTemplate
}

// 消息模板类型
type MessageTemplate =
  | { type: "search_results"; data: SearchResultsData }
  | { type: "download_task"; data: DownloadTaskData }
  | { type: "media_card"; data: MediaCardData }
  | { type: "subscription"; data: SubscriptionData }
  | { type: "task_progress"; data: TaskProgressData }

interface SearchResultsData {
  query: string
  source: string // "local" | "jackett" | "prowlarr" | ...
  results: MediaResult[]
  totalCount: number
}

interface DownloadTaskData {
  id: string
  title: string
  status: "pending" | "downloading" | "completed" | "failed"
  progress: number
  speed?: string
  eta?: string
  size?: string
}

interface MediaCardData {
  id: string
  title: string
  year?: number
  poster?: string
  type: "movie" | "series" | "anime"
  rating?: number
  overview?: string
  action?: "play" | "download" | "subscribe"
}

interface SubscriptionData {
  id: string
  title: string
  type: "series" | "anime"
  status: "active" | "paused" | "completed"
  nextEpisode?: string
  lastUpdate?: Date
}

interface TaskProgressData {
  id: string
  name: string
  type: "scrape" | "organize" | "transcode" | "subtitle"
  status: "running" | "completed" | "failed"
  progress: number
  message?: string
}

interface ToolCall {
  id: string
  name: string
  status: "pending" | "running" | "success" | "error"
  input?: Record<string, unknown>
  output?: string
  duration?: number
}

interface MediaResult {
  id: string
  title: string
  year?: number
  poster?: string
  type: "movie" | "series" | "anime"
  rating?: number
  quality?: string
  size?: string
  source?: string
}

interface Conversation {
  id: string
  title: string
  preview: string
  timestamp: Date
  messageCount: number
}

// 预设提供商
const defaultProviders: LLMProvider[] = [
  {
    id: "openai",
    name: "OpenAI",
    icon: "https://upload.wikimedia.org/wikipedia/commons/0/04/ChatGPT_logo.svg",
    apiKeySet: false,
    models: [
      { id: "gpt-4o", name: "GPT-4o", description: "最新多模态模型", contextLength: 128000, pricing: "$5/$15 per 1M tokens", enabled: true },
      { id: "gpt-4o-mini", name: "GPT-4o Mini", description: "快速经济", contextLength: 128000, pricing: "$0.15/$0.6 per 1M tokens", enabled: true },
      { id: "gpt-4-turbo", name: "GPT-4 Turbo", description: "高性能", contextLength: 128000, pricing: "$10/$30 per 1M tokens", enabled: false },
    ],
    defaultModel: "gpt-4o-mini",
    isDefault: true
  },
  {
    id: "anthropic",
    name: "Anthropic",
    icon: "https://upload.wikimedia.org/wikipedia/commons/7/78/Anthropic_logo.svg",
    apiKeySet: false,
    models: [
      { id: "claude-3-5-sonnet", name: "Claude 3.5 Sonnet", description: "最佳平衡", contextLength: 200000, pricing: "$3/$15 per 1M tokens", enabled: true },
      { id: "claude-3-opus", name: "Claude 3 Opus", description: "最强能力", contextLength: 200000, pricing: "$15/$75 per 1M tokens", enabled: false },
      { id: "claude-3-haiku", name: "Claude 3 Haiku", description: "快速响应", contextLength: 200000, pricing: "$0.25/$1.25 per 1M tokens", enabled: true },
    ],
    defaultModel: "claude-3-5-sonnet"
  },
  {
    id: "google",
    name: "Google",
    icon: "https://www.gstatic.com/lamda/images/gemini_sparkle_v002_d4735304ff6292a690345.svg",
    apiKeySet: false,
    models: [
      { id: "gemini-1.5-pro", name: "Gemini 1.5 Pro", description: "长上下文", contextLength: 1000000, pricing: "$3.5/$10.5 per 1M tokens", enabled: true },
      { id: "gemini-1.5-flash", name: "Gemini 1.5 Flash", description: "快速高效", contextLength: 1000000, pricing: "$0.075/$0.3 per 1M tokens", enabled: true },
    ],
    defaultModel: "gemini-1.5-flash"
  },
  {
    id: "ollama",
    name: "Ollama (本地)",
    icon: "",
    apiKeySet: true,
    baseUrl: "http://localhost:11434",
    models: [
      { id: "llama3.1", name: "Llama 3.1", description: "开源高质量", contextLength: 128000, pricing: "免费", enabled: true },
      { id: "qwen2.5", name: "Qwen 2.5", description: "中文优化", contextLength: 128000, pricing: "免费", enabled: true },
      { id: "mistral", name: "Mistral", description: "高效推理", contextLength: 32000, pricing: "免费", enabled: false },
    ],
    defaultModel: "llama3.1"
  }
]

// 预设 Skills
const defaultSkills: Skill[] = [
  { id: "search_resource", name: "搜索资源", description: "从多个来源搜索影视资源", icon: <Search className="h-4 w-4" />, category: "resource", enabled: true, source: "builtin" },
  { id: "download_media", name: "下载媒体", description: "下载种子、HTTP、网盘资源", icon: <Download className="h-4 w-4" />, category: "resource", enabled: true, source: "builtin" },
  { id: "subscribe_media", name: "订阅追更", description: "自动追踪剧集更新", icon: <ListPlus className="h-4 w-4" />, category: "resource", enabled: true, source: "builtin" },
  { id: "scrape_metadata", name: "刮削元数据", description: "获取元数据和海报", icon: <Film className="h-4 w-4" />, category: "media", enabled: true, source: "builtin" },
  { id: "organize_library", name: "整理媒体库", description: "重命名、分类、去重", icon: <FolderSync className="h-4 w-4" />, category: "library", enabled: true, source: "builtin" },
  { id: "generate_subtitle", name: "生成字幕", description: "使用 Whisper 生成字幕", icon: <Subtitles className="h-4 w-4" />, category: "media", enabled: true, source: "plugin", pluginName: "whisper-sidecar" },
  { id: "translate_subtitle", name: "翻译字幕", description: "翻译字幕语言", icon: <Globe className="h-4 w-4" />, category: "media", enabled: true, source: "plugin", pluginName: "whisper-sidecar" },
  { id: "recommend", name: "智能推荐", description: "基于喜好推荐内容", icon: <Sparkles className="h-4 w-4" />, category: "smart", enabled: true, source: "builtin" },
  { id: "analyze_content", name: "内容分析", description: "分析视频内容", icon: <Brain className="h-4 w-4" />, category: "smart", enabled: false, source: "builtin" },
]

// 历史会话
const mockConversations: Conversation[] = [
  { id: "1", title: "搜索沙丘2资源", preview: "找到了 4K HDR 版本", timestamp: new Date(Date.now() - 10 * 60 * 1000), messageCount: 4 },
  { id: "2", title: "整理电影库", preview: "已整理 23 部电影", timestamp: new Date(Date.now() - 60 * 60 * 1000), messageCount: 6 },
  { id: "3", title: "订阅怪奇物语", preview: "订阅已创建", timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000), messageCount: 3 },
]

// 预设建议
const defaultSuggestions = [
  { text: "搜索《沙丘2》4K 资源", icon: <Search className="h-4 w-4" /> },
  { text: "订阅《怪奇物语》第五季", icon: <ListPlus className="h-4 w-4" /> },
  { text: "整理下载目录", icon: <FolderSync className="h-4 w-4" /> },
  { text: "推荐类似《星际穿越》的电影", icon: <Sparkles className="h-4 w-4" /> },
]

export function AgentAssistant({ onBack }: AgentAssistantProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [input, setInput] = useState("")
  const [isLoading, setIsLoading] = useState(false)
  const [expandedTools, setExpandedTools] = useState<Record<string, boolean>>({})
  const [sidebarOpen, setSidebarOpen] = useState(true)
  const [currentView, setCurrentView] = useState<"chat" | "skills" | "settings">("chat")
  const [skills, setSkills] = useState<Skill[]>(defaultSkills)
  const [providers, setProviders] = useState<LLMProvider[]>(defaultProviders)
  const [conversations, setConversations] = useState<Conversation[]>(mockConversations)
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null)
  const [editingProviderId, setEditingProviderId] = useState<string | null>(null)
  const [showApiKey, setShowApiKey] = useState<Record<string, boolean>>({})
  const [historyCollapsed, setHistoryCollapsed] = useState(false)

  const scrollRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLTextAreaElement>(null)

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [messages])

  const formatTime = (date: Date) => {
    const now = new Date()
    const diff = now.getTime() - date.getTime()
    const minutes = Math.floor(diff / 60000)
    const hours = Math.floor(diff / 3600000)
    const days = Math.floor(diff / 86400000)

    if (minutes < 1) return "刚刚"
    if (minutes < 60) return `${minutes}分钟前`
    if (hours < 24) return `${hours}小时前`
    if (days < 7) return `${days}天前`
    return date.toLocaleDateString()
  }

  const handleNewChat = () => {
    setMessages([])
    setActiveConversationId(null)
    setInput("")
  }

  // 消息发送逻辑 (简化版)
  const handleSend = async () => {
    if (!input.trim() || isLoading) return

    const userMessage: ChatMessage = {
      id: `msg-${Date.now()}`,
      role: "user",
      content: input,
      timestamp: new Date(),
    }

    setMessages(prev => [...prev, userMessage])
    setInput("")
    setIsLoading(true)

    await new Promise(resolve => setTimeout(resolve, 600))

    let response: ChatMessage

    if (input.includes("搜索") || input.includes("找")) {
      const keyword = input.replace(/搜索|找|帮我|《|》/g, "").trim()
      response = {
        id: `msg-${Date.now() + 1}`,
        role: "assistant",
        content: "",
        timestamp: new Date(),
        toolCalls: [{ id: "tool-1", name: "search_resource", status: "running", input: { query: keyword } }],
      }
      setMessages(prev => [...prev, response])

  setTimeout(() => {
  setMessages(prev => prev.map(msg => {
  if (msg.id === response.id) {
  return {
  ...msg,
  content: `找到以下「${keyword}」相关资源：`,
  toolCalls: [{ ...msg.toolCalls![0], status: "success" as const, duration: 2.3 }],
  template: {
    type: "search_results" as const,
    data: {
      query: keyword,
      source: "Jackett",
      totalCount: 4,
      results: [
        { id: "r1", title: keyword, year: 2024, type: "movie" as const, rating: 8.5, quality: "4K HDR", size: "24.5 GB", source: "PT站" },
        { id: "r2", title: keyword, year: 2024, type: "movie" as const, rating: 8.5, quality: "1080p", size: "12.3 GB", source: "Alist" },
        { id: "r3", title: `${keyword} 中字`, year: 2024, type: "movie" as const, rating: 8.5, quality: "4K", size: "28.1 GB", source: "网盘" },
      ]
    }
  },
  suggestions: ["下载 4K 版本", "查看更多", "添加订阅"],
            }
          }
          return msg
        }))
        setIsLoading(false)
      }, 2000)
      return
    }

    response = {
      id: `msg-${Date.now() + 1}`,
      role: "assistant",
      content: "我是 Nako 智能助手，可以帮你搜索资源、下载影片、整理媒体库。你可以直接告诉我想做什么。",
      timestamp: new Date(),
      suggestions: ["搜索最新电影", "整理媒体库", "查看订阅"],
    }

    setMessages(prev => [...prev, response])
    setIsLoading(false)
  }

  const handleSuggestionClick = (text: string) => {
    setInput(text)
    inputRef.current?.focus()
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  const toggleToolExpand = (toolId: string) => {
    setExpandedTools(prev => ({ ...prev, [toolId]: !prev[toolId] }))
  }

  const toggleSkill = (skillId: string) => {
    setSkills(prev => prev.map(s => s.id === skillId ? { ...s, enabled: !s.enabled } : s))
  }

  const setDefaultProvider = (providerId: string) => {
    setProviders(prev => prev.map(p => ({ ...p, isDefault: p.id === providerId })))
  }

  const updateProviderApiKey = (providerId: string, apiKey: string) => {
    setProviders(prev => prev.map(p =>
      p.id === providerId ? { ...p, apiKey, apiKeySet: apiKey.length > 0 } : p
    ))
  }

  const toggleModelEnabled = (providerId: string, modelId: string) => {
    setProviders(prev => prev.map(p =>
      p.id === providerId
        ? { ...p, models: p.models.map(m => m.id === modelId ? { ...m, enabled: !m.enabled } : m) }
        : p
    ))
  }

  const getToolStatusIcon = (status: ToolCall["status"]) => {
    switch (status) {
      case "running": return <Loader2 className="h-3.5 w-3.5 animate-spin text-cyan-400" />
      case "success": return <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
      case "error": return <AlertCircle className="h-3.5 w-3.5 text-red-400" />
      default: return <Clock className="h-3.5 w-3.5 text-muted-foreground" />
    }
  }

  const defaultProvider = useMemo(() => providers.find(p => p.isDefault), [providers])

  // 对话列表分组
  const groupedConversations = useMemo(() => {
    const now = new Date()
    const today: Conversation[] = []
    const week: Conversation[] = []
    const older: Conversation[] = []

    conversations.forEach(conv => {
      const diff = now.getTime() - conv.timestamp.getTime()
      const days = diff / 86400000
      if (days < 1) today.push(conv)
      else if (days < 7) week.push(conv)
      else older.push(conv)
    })

    return { today, week, older }
  }, [conversations])

  // ===== 设置页面 =====
  const SettingsView = () => (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center gap-3 border-b border-border/50 px-6 py-4">
        <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => setCurrentView("chat")}>
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <h1 className="text-lg font-semibold">AI 设置</h1>
      </div>

      <ScrollArea className="flex-1">
        <div className="mx-auto max-w-3xl space-y-8 p-6">
          {/* LLM 提供商 */}
          <section>
            <div className="mb-4 flex items-center justify-between">
              <div>
                <h2 className="text-base font-medium">语言模型提供商</h2>
                <p className="text-sm text-muted-foreground">配置 API Key 并选择默认模型</p>
              </div>
              <Button variant="outline" size="sm" className="gap-2">
                <Plus className="h-3.5 w-3.5" />
                添加提供商
              </Button>
            </div>

            <div className="space-y-3">
              {providers.map(provider => (
                <div
                  key={provider.id}
                  className={cn(
                    "rounded-xl border transition-all",
                    provider.isDefault ? "border-cyan-500/50 bg-cyan-500/5" : "border-border/50 bg-card/50"
                  )}
                >
                  {/* Provider Header */}
                  <div
                    className="flex cursor-pointer items-center justify-between p-4"
                    onClick={() => setEditingProviderId(editingProviderId === provider.id ? null : provider.id)}
                  >
                    <div className="flex items-center gap-3">
                      <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted/50">
                        {provider.icon ? (
                          <img src={resolveArtwork(provider.icon)} alt={provider.name} className="h-5 w-5" />
                        ) : (
                          <Bot className="h-5 w-5 text-muted-foreground" />
                        )}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium">{provider.name}</span>
                          {provider.isDefault && (
                            <Badge variant="secondary" className="bg-cyan-500/10 text-cyan-500 text-[10px]">默认</Badge>
                          )}
                          {provider.apiKeySet ? (
                            <Badge variant="secondary" className="bg-emerald-500/10 text-emerald-500 text-[10px]">已配置</Badge>
                          ) : (
                            <Badge variant="secondary" className="bg-amber-500/10 text-amber-500 text-[10px]">未配置</Badge>
                          )}
                        </div>
                        <p className="text-xs text-muted-foreground">
                          {provider.models.filter(m => m.enabled).length} 个可用模型
                          {provider.defaultModel && ` · 默认: ${provider.models.find(m => m.id === provider.defaultModel)?.name}`}
                        </p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      {!provider.isDefault && provider.apiKeySet && (
                        <Button
                          variant="ghost"
                          size="sm"
                          className="h-7 text-xs"
                          onClick={(e) => { e.stopPropagation(); setDefaultProvider(provider.id); }}
                        >
                          设为默认
                        </Button>
                      )}
                      <ChevronDown className={cn(
                        "h-4 w-4 text-muted-foreground transition-transform",
                        editingProviderId === provider.id && "rotate-180"
                      )} />
                    </div>
                  </div>

                  {/* Provider Details */}
                  {editingProviderId === provider.id && (
                    <div className="border-t border-border/50 p-4 pt-4">
                      {/* API Key */}
                      <div className="mb-4 space-y-2">
                        <Label className="text-xs">API Key</Label>
                        <div className="flex gap-2">
                          <div className="relative flex-1">
                            <Input
                              type={showApiKey[provider.id] ? "text" : "password"}
                              placeholder={provider.id === "ollama" ? "本地无需 API Key" : "输入 API Key..."}
                              value={provider.apiKey || ""}
                              onChange={(e) => updateProviderApiKey(provider.id, e.target.value)}
                              className="pr-10 font-mono text-xs"
                              disabled={provider.id === "ollama"}
                            />
                            <Button
                              variant="ghost"
                              size="icon"
                              className="absolute right-1 top-1/2 h-6 w-6 -translate-y-1/2"
                              onClick={() => setShowApiKey(prev => ({ ...prev, [provider.id]: !prev[provider.id] }))}
                            >
                              {showApiKey[provider.id] ? <EyeOff className="h-3.5 w-3.5" /> : <Eye className="h-3.5 w-3.5" />}
                            </Button>
                          </div>
                          <Button variant="outline" size="sm" className="gap-1.5">
                            <TestTube className="h-3.5 w-3.5" />
                            测试
                          </Button>
                        </div>
                      </div>

                      {/* Base URL for custom providers */}
                      {(provider.id === "ollama" || provider.baseUrl) && (
                        <div className="mb-4 space-y-2">
                          <Label className="text-xs">API 端点</Label>
                          <Input
                            placeholder="http://localhost:11434"
                            value={provider.baseUrl || ""}
                            onChange={(e) => setProviders(prev => prev.map(p =>
                              p.id === provider.id ? { ...p, baseUrl: e.target.value } : p
                            ))}
                            className="font-mono text-xs"
                          />
                        </div>
                      )}

                      {/* Models */}
                      <div className="space-y-2">
                        <div className="flex items-center justify-between">
                          <Label className="text-xs">可用模型</Label>
                          {provider.id === "ollama" && (
                            <Button variant="ghost" size="sm" className="h-6 gap-1 text-xs">
                              <RefreshCw className="h-3 w-3" />
                              刷新模型列表
                            </Button>
                          )}
                        </div>
                        <div className="space-y-1 rounded-lg border border-border/50 p-2">
                          {provider.models.map(model => (
                            <div
                              key={model.id}
                              className={cn(
                                "flex items-center justify-between rounded-md px-3 py-2 transition-colors",
                                model.enabled ? "bg-muted/30" : "opacity-50"
                              )}
                            >
                              <div className="flex items-center gap-3">
                                <Switch
                                  checked={model.enabled}
                                  onCheckedChange={() => toggleModelEnabled(provider.id, model.id)}
                                  className="scale-75"
                                />
                                <div>
                                  <div className="flex items-center gap-2">
                                    <span className="text-sm font-medium">{model.name}</span>
                                    {provider.defaultModel === model.id && (
                                      <Badge variant="outline" className="text-[10px]">默认</Badge>
                                    )}
                                  </div>
                                  <p className="text-xs text-muted-foreground">{model.description}</p>
                                </div>
                              </div>
                              <div className="text-right text-xs text-muted-foreground">
                                <div>{model.contextLength?.toLocaleString()} tokens</div>
                                <div>{model.pricing}</div>
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </section>

          {/* 行为设置 */}
          <section>
            <h2 className="mb-4 text-base font-medium">行为设置</h2>
            <div className="space-y-4 rounded-xl border border-border/50 bg-card/50 p-4">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">自动执行安全操作</p>
                  <p className="text-xs text-muted-foreground">搜索、推荐等不影响数据的操作自动执行</p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">下载前确认</p>
                  <p className="text-xs text-muted-foreground">开始下载前询问确认</p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">删除操作二次确认</p>
                  <p className="text-xs text-muted-foreground">删除文件时需要额外确认</p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-medium">记忆历史对话</p>
                  <p className="text-xs text-muted-foreground">在新对话中参考之前的偏好和上下文</p>
                </div>
                <Switch defaultChecked />
              </div>
            </div>
          </section>
        </div>
      </ScrollArea>
    </div>
  )

  // ===== Skills 页面 =====
  const SkillsView = () => {
    const [skillFilter, setSkillFilter] = useState<"all" | Skill["category"]>("all")
    const filteredSkills = skillFilter === "all" ? skills : skills.filter(s => s.category === skillFilter)

    return (
      <div className="flex h-full flex-col">
        <div className="flex items-center gap-3 border-b border-border/50 px-6 py-4">
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => setCurrentView("chat")}>
            <ArrowLeft className="h-4 w-4" />
          </Button>
          <h1 className="text-lg font-semibold">Skills 管理</h1>
        </div>

        <div className="flex-1 overflow-hidden">
          <div className="mx-auto max-w-3xl p-6">
            <Tabs value={skillFilter} onValueChange={(v) => setSkillFilter(v as typeof skillFilter)}>
              <TabsList className="mb-6 w-full justify-start bg-muted/30">
                <TabsTrigger value="all">全部</TabsTrigger>
                <TabsTrigger value="resource">资源获取</TabsTrigger>
                <TabsTrigger value="media">媒体处理</TabsTrigger>
                <TabsTrigger value="library">库管理</TabsTrigger>
                <TabsTrigger value="smart">智能功能</TabsTrigger>
              </TabsList>

              <div className="grid gap-3">
                {filteredSkills.map(skill => (
                  <div
                    key={skill.id}
                    className={cn(
                      "flex items-center justify-between rounded-xl border p-4 transition-all",
                      skill.enabled ? "border-border/50 bg-card/50" : "border-border/30 bg-muted/20 opacity-60"
                    )}
                  >
                    <div className="flex items-center gap-4">
                      <div className={cn(
                        "flex h-10 w-10 items-center justify-center rounded-xl",
                        skill.enabled ? "bg-cyan-500/10 text-cyan-500" : "bg-muted text-muted-foreground"
                      )}>
                        {skill.icon}
                      </div>
                      <div>
                        <div className="flex items-center gap-2">
                          <span className="font-medium">{skill.name}</span>
                          {skill.source === "plugin" && (
                            <Badge variant="outline" className="text-[10px]">
                              <Plug className="mr-1 h-2.5 w-2.5" />
                              {skill.pluginName}
                            </Badge>
                          )}
                        </div>
                        <p className="text-sm text-muted-foreground">{skill.description}</p>
                      </div>
                    </div>
                    <Switch checked={skill.enabled} onCheckedChange={() => toggleSkill(skill.id)} />
                  </div>
                ))}
              </div>
            </Tabs>
          </div>
        </div>
      </div>
    )
  }

  // ===== 聊天视图 =====
  const ChatView = () => (
    <div className="flex h-full">
      {/* Sidebar */}
      <div className={cn(
        "flex h-full flex-col border-r border-border/50 bg-card/30 transition-all duration-300",
        sidebarOpen ? "w-64" : "w-0 overflow-hidden"
      )}>
        {/* Sidebar Header */}
        <div className="flex items-center justify-between border-b border-border/50 p-3">
          <Button variant="ghost" size="sm" className="h-8 gap-2 px-3" onClick={handleNewChat}>
            <Plus className="h-4 w-4" />
            新对话
          </Button>
          <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => setSidebarOpen(false)}>
            <PanelLeftClose className="h-4 w-4" />
          </Button>
        </div>

        {/* Conversations - 分组折叠 */}
        <ScrollArea className="flex-1 p-2">
          {groupedConversations.today.length > 0 && (
            <ConversationGroup title="今天" conversations={groupedConversations.today} defaultOpen />
          )}
          {groupedConversations.week.length > 0 && (
            <ConversationGroup title="最近 7 天" conversations={groupedConversations.week} defaultOpen />
          )}
          {groupedConversations.older.length > 0 && (
            <ConversationGroup title="更早" conversations={groupedConversations.older} />
          )}
        </ScrollArea>
      </div>

      {/* Main Chat Area */}
      <div className="flex flex-1 flex-col">
        {/* Chat Header */}
        <div className="flex items-center justify-between border-b border-border/50 px-4 py-3">
          <div className="flex items-center gap-3">
            {!sidebarOpen && (
              <Button variant="ghost" size="icon" className="h-8 w-8" onClick={() => setSidebarOpen(true)}>
                <PanelLeft className="h-4 w-4" />
              </Button>
            )}
            <Button variant="ghost" size="icon" className="h-8 w-8" onClick={onBack}>
              <ArrowLeft className="h-4 w-4" />
            </Button>
            <span className="font-medium">
              {activeConversationId
                ? conversations.find(c => c.id === activeConversationId)?.title
                : "新对话"}
            </span>
          </div>
          <div className="flex items-center gap-1">
            {/* Skills 和设置按钮移到这里 */}
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 gap-1.5 px-2.5 text-xs"
                    onClick={() => setCurrentView("skills")}
                  >
                    <Zap className="h-3.5 w-3.5" />
                    <span className="hidden sm:inline">Skills</span>
                  </Button>
                </TooltipTrigger>
                <TooltipContent>管理 AI 技能</TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-8 gap-1.5 px-2.5 text-xs"
                    onClick={() => setCurrentView("settings")}
                  >
                    <Settings className="h-3.5 w-3.5" />
                    <span className="hidden sm:inline">设置</span>
                  </Button>
                </TooltipTrigger>
                <TooltipContent>AI 设置</TooltipContent>
              </Tooltip>
            </TooltipProvider>
            <div className="mx-1 h-4 w-px bg-border/50" />
            {/* 当前模型选择器 */}
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button variant="ghost" size="sm" className="h-8 gap-1.5 text-xs text-muted-foreground">
                  <Bot className="h-3.5 w-3.5" />
                  <span className="hidden md:inline">{defaultProvider?.name} · {defaultProvider?.models.find(m => m.id === defaultProvider.defaultModel)?.name}</span>
                  <span className="md:hidden">{defaultProvider?.models.find(m => m.id === defaultProvider.defaultModel)?.name?.split('-')[0]}</span>
                  <ChevronDown className="ml-1 h-3 w-3" />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-64">
                <div className="px-2 py-1.5 text-xs font-medium text-muted-foreground">快速切换模型</div>
                <DropdownMenuSeparator />
                {providers.filter(p => p.apiKeySet).map(provider => (
                  <div key={provider.id}>
                    <div className="px-2 py-1 text-[10px] font-medium uppercase tracking-wider text-muted-foreground/60">
                      {provider.name}
                    </div>
                    {provider.models.filter(m => m.enabled).map(model => (
                      <DropdownMenuItem
                        key={model.id}
                        className="gap-2"
                        onClick={() => {
                          setProviders(prev => prev.map(p => ({
                            ...p,
                            isDefault: p.id === provider.id,
                            defaultModel: p.id === provider.id ? model.id : p.defaultModel
                          })))
                        }}
                      >
                        <div className="flex flex-1 items-center justify-between">
                          <span className="text-sm">{model.name}</span>
                          {defaultProvider?.id === provider.id && defaultProvider?.defaultModel === model.id && (
                            <Check className="h-3.5 w-3.5 text-cyan-500" />
                          )}
                        </div>
                      </DropdownMenuItem>
                    ))}
                  </div>
                ))}
                <DropdownMenuSeparator />
                <DropdownMenuItem onClick={() => setCurrentView("settings")} className="gap-2">
                  <Settings className="h-3.5 w-3.5" />
                  管理提供商和模型
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>

        {/* Messages */}
        <ScrollArea className="flex-1" ref={scrollRef}>
          {messages.length === 0 ? (
            <WelcomeScreen />
          ) : (
            <div className="mx-auto max-w-3xl space-y-6 p-6">
              {messages.map(message => (
                <MessageBubble key={message.id} message={message} />
              ))}
              {isLoading && messages[messages.length - 1]?.role === "user" && (
                <div className="flex gap-4">
                  <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-cyan-500/20 to-teal-500/20">
                    <Bot className="h-4 w-4 text-cyan-500" />
                  </div>
                  <div className="flex items-center gap-2 text-sm text-muted-foreground">
                    <Loader2 className="h-4 w-4 animate-spin" />
                    思考中...
                  </div>
                </div>
              )}
            </div>
          )}
        </ScrollArea>

        {/* Input */}
        <div className="border-t border-border/50 p-4">
          <div className="mx-auto max-w-3xl">
            <div className="relative rounded-2xl border border-border/50 bg-card/50 focus-within:border-cyan-500/50 focus-within:ring-2 focus-within:ring-cyan-500/10">
              <Textarea
                ref={inputRef}
                value={input}
                onChange={(e) => setInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder="输入消息..."
                className="min-h-[52px] max-h-[200px] resize-none border-0 bg-transparent px-4 py-3.5 pr-28 focus-visible:ring-0"
                rows={1}
              />
              <div className="absolute bottom-2 right-2 flex items-center gap-1">
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-foreground"
                        onClick={() => {
                          const input = document.createElement('input')
                          input.type = 'file'
                          input.accept = 'image/*,.pdf,.txt'
                          input.onchange = (e) => {
                            const file = (e.target as HTMLInputElement).files?.[0]
                            if (file) {
                              // TODO: Handle file upload
                            }
                          }
                          input.click()
                        }}
                      >
                        <Paperclip className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>添加附件</TooltipContent>
                  </Tooltip>
                </TooltipProvider>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-8 w-8 text-muted-foreground hover:text-foreground"
                      >
                        <Mic className="h-4 w-4" />
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent>语音输入</TooltipContent>
                  </Tooltip>
                </TooltipProvider>
                <Button
                  size="icon"
                  className="h-8 w-8 bg-cyan-500 hover:bg-cyan-600"
                  onClick={handleSend}
                  disabled={!input.trim() || isLoading}
                >
                  <Send className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )

  // ===== 消息模板组件 =====

  // 搜索结果模板
  const SearchResultsTemplate = ({ data }: { data: SearchResultsData }) => (
    <div className="w-full space-y-3">
      <div className="flex items-center justify-between text-xs text-muted-foreground">
        <span>找到 {data.totalCount} 个结果</span>
        <Badge variant="outline" className="text-[10px]">{data.source}</Badge>
      </div>
      {data.results.map(media => (
        <div
          key={media.id}
          className="flex items-center gap-4 rounded-xl border border-border/50 bg-card/50 p-3 transition-colors hover:bg-muted/30"
        >
          <div className="relative h-16 w-12 shrink-0 overflow-hidden rounded-lg bg-muted">
            {media.poster ? (
              <img src={resolveArtwork(media.poster)} alt={media.title} className="h-full w-full object-cover" />
            ) : (
              <div className="flex h-full w-full items-center justify-center">
                <Film className="h-5 w-5 text-muted-foreground" />
              </div>
            )}
          </div>
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <span className="font-medium truncate">{media.title}</span>
              {media.year && <span className="text-sm text-muted-foreground shrink-0">{media.year}</span>}
            </div>
            <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              {media.rating && (
                <span className="flex items-center gap-1">
                  <Star className="h-3 w-3 fill-amber-400 text-amber-400" />
                  {media.rating}
                </span>
              )}
              {media.quality && <Badge variant="secondary" className="text-[10px]">{media.quality}</Badge>}
              {media.size && <span>{media.size}</span>}
              {media.source && <span className="text-[10px] opacity-60">{media.source}</span>}
            </div>
          </div>
          <Button size="sm" className="shrink-0 gap-1.5 bg-cyan-500 hover:bg-cyan-600">
            <Download className="h-3.5 w-3.5" />
            下载
          </Button>
        </div>
      ))}
    </div>
  )

  // 下载任务模板
  const DownloadTaskTemplate = ({ data }: { data: DownloadTaskData }) => (
    <div className="w-full rounded-xl border border-border/50 bg-card/50 p-4">
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-3">
          {data.status === "downloading" ? (
            <Loader2 className="h-5 w-5 animate-spin text-cyan-500" />
          ) : data.status === "completed" ? (
            <div className="flex h-5 w-5 items-center justify-center rounded-full bg-green-500/20">
              <Check className="h-3 w-3 text-green-500" />
            </div>
          ) : data.status === "failed" ? (
            <div className="flex h-5 w-5 items-center justify-center rounded-full bg-red-500/20">
              <X className="h-3 w-3 text-red-500" />
            </div>
          ) : (
            <div className="flex h-5 w-5 items-center justify-center rounded-full bg-muted">
              <Clock className="h-3 w-3 text-muted-foreground" />
            </div>
          )}
          <div>
            <p className="font-medium text-sm truncate max-w-[300px]">{data.title}</p>
            <div className="flex items-center gap-2 text-xs text-muted-foreground">
              {data.size && <span>{data.size}</span>}
              {data.speed && <><span>·</span><span>{data.speed}</span></>}
              {data.eta && <><span>·</span><span>剩余 {data.eta}</span></>}
            </div>
          </div>
        </div>
        <Badge variant={
          data.status === "completed" ? "default" :
          data.status === "failed" ? "destructive" :
          "secondary"
        }>
          {data.status === "pending" && "等待中"}
          {data.status === "downloading" && "下载中"}
          {data.status === "completed" && "已完成"}
          {data.status === "failed" && "失败"}
        </Badge>
      </div>
      {data.status === "downloading" && (
        <div className="space-y-1">
          <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
            <div
              className="h-full bg-gradient-to-r from-cyan-500 to-teal-500 transition-all"
              style={{ width: `${data.progress}%` }}
            />
          </div>
          <p className="text-right text-xs text-muted-foreground">{data.progress}%</p>
        </div>
      )}
    </div>
  )

  // 对话分组组件
  const ConversationGroup = ({ title, conversations, defaultOpen = false }: {
    title: string
    conversations: Conversation[]
    defaultOpen?: boolean
  }) => {
    const [open, setOpen] = useState(defaultOpen)

    return (
      <Collapsible open={open} onOpenChange={setOpen} className="mb-2">
        <CollapsibleTrigger className="flex w-full items-center gap-2 px-2 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground">
          <ChevronRight className={cn("h-3 w-3 transition-transform", open && "rotate-90")} />
          {title}
          <span className="ml-auto text-[10px]">{conversations.length}</span>
        </CollapsibleTrigger>
        <CollapsibleContent className="space-y-0.5">
          {conversations.map(conv => (
            <div
              key={conv.id}
              onClick={() => setActiveConversationId(conv.id)}
              className={cn(
                "group flex cursor-pointer items-center gap-2 rounded-lg px-3 py-2 transition-colors",
                activeConversationId === conv.id ? "bg-cyan-500/10" : "hover:bg-muted/50"
              )}
            >
              <MessageSquare className="h-4 w-4 shrink-0 text-muted-foreground" />
              <div className="flex-1 truncate">
                <p className="truncate text-sm">{conv.title}</p>
              </div>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <Button
                    variant="ghost"
                    size="icon"
                    className="h-6 w-6 opacity-0 group-hover:opacity-100"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <MoreHorizontal className="h-3.5 w-3.5" />
                  </Button>
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem><Edit3 className="mr-2 h-3.5 w-3.5" />重命名</DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem className="text-destructive"><Trash2 className="mr-2 h-3.5 w-3.5" />删除</DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          ))}
        </CollapsibleContent>
      </Collapsible>
    )
  }

  // 消息气泡组件
  const MessageBubble = ({ message }: { message: ChatMessage }) => {
    const isUser = message.role === "user"

    return (
      <div className={cn("group flex gap-4", isUser && "flex-row-reverse")}>
        {/* Avatar */}
        <div className={cn(
          "flex h-8 w-8 shrink-0 items-center justify-center rounded-full",
          isUser ? "bg-gradient-to-br from-violet-500/20 to-fuchsia-500/20" : "bg-gradient-to-br from-cyan-500/20 to-teal-500/20"
        )}>
          {isUser ? (
            <span className="text-xs font-medium text-violet-500">U</span>
          ) : (
            <Bot className="h-4 w-4 text-cyan-500" />
          )}
        </div>

        {/* Content */}
        <div className={cn("flex-1 space-y-3", isUser && "flex flex-col items-end")}>
          {/* Text */}
          {message.content && (
            <div className={cn(
              "rounded-2xl px-4 py-3 text-sm",
              isUser
                ? "bg-gradient-to-br from-cyan-500 to-teal-500 text-white"
                : "bg-muted/50"
            )}>
              <div className="whitespace-pre-wrap">{message.content}</div>
            </div>
          )}

          {/* Tool Calls */}
          {message.toolCalls?.map(tool => (
            <div
              key={tool.id}
              className="w-full rounded-xl border border-border/50 bg-card/50 overflow-hidden"
            >
              <button
                onClick={() => toggleToolExpand(tool.id)}
                className="flex w-full items-center justify-between p-3 text-left hover:bg-muted/30"
              >
                <div className="flex items-center gap-3">
                  {getToolStatusIcon(tool.status)}
                  <span className="text-sm font-medium">{skills.find(s => s.id === tool.name)?.name || tool.name}</span>
                  {tool.duration && (
                    <span className="text-xs text-muted-foreground">{tool.duration}s</span>
                  )}
                </div>
                <ChevronDown className={cn(
                  "h-4 w-4 text-muted-foreground transition-transform",
                  expandedTools[tool.id] && "rotate-180"
                )} />
              </button>
              {expandedTools[tool.id] && tool.input && (
                <div className="border-t border-border/50 bg-muted/20 p-3">
                  <pre className="text-xs text-muted-foreground overflow-auto">
                    {JSON.stringify(tool.input, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          ))}

          {/* Media Results - 兼容旧格式 */}
          {message.template?.type === "search_results" && (
            <SearchResultsTemplate data={message.template.data} />
          )}

          {/* Download Task Template */}
          {message.template?.type === "download_task" && (
            <DownloadTaskTemplate data={message.template.data} />
          )}

          {/* Suggestions */}
          {message.suggestions && message.suggestions.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {message.suggestions.map((suggestion, i) => (
                <Button
                  key={i}
                  variant="outline"
                  size="sm"
                  className="h-7 rounded-full text-xs"
                  onClick={() => handleSuggestionClick(suggestion)}
                >
                  {suggestion}
                </Button>
              ))}
            </div>
          )}

          {/* Message Actions */}
          {!isUser && message.content && (
            <div className="flex items-center gap-1 opacity-0 transition-opacity group-hover:opacity-100">
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7"
                      onClick={() => {
                        navigator.clipboard.writeText(message.content || "")
                      }}
                    >
                      <Copy className="h-3.5 w-3.5" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>复制</TooltipContent>
                </Tooltip>
              </TooltipProvider>
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7"
                      onClick={() => {
                        // 重新生成 - 找到上一条用户消息重新发送
                        const userMessages = messages.filter(m => m.role === "user")
                        const lastUserMessage = userMessages[userMessages.length - 1]
                        if (lastUserMessage) {
                          setInput(lastUserMessage.content || "")
                        }
                      }}
                    >
                      <RefreshCw className="h-3.5 w-3.5" />
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>重新生成</TooltipContent>
                </Tooltip>
              </TooltipProvider>
            </div>
          )}
        </div>
      </div>
    )
  }

  // 欢迎界面
  const WelcomeScreen = () => (
    <div className="flex min-h-full w-full flex-col items-center justify-center px-6 py-12">
      <div className="flex flex-col items-center">
        <div className="mb-6 flex h-20 w-20 items-center justify-center rounded-3xl bg-gradient-to-br from-cyan-500/20 via-teal-500/15 to-emerald-500/10 ring-1 ring-cyan-500/20">
          <img src={resolveArtwork("/nako-icon.png")} alt="Nako" className="h-12 w-12 rounded-xl" />
        </div>

        <h1 className="mb-2 text-2xl font-semibold tracking-tight">你好，我是 Nako</h1>
        <p className="mb-10 max-w-md text-center text-muted-foreground">
          我可以帮你搜索资源、下载影片、整理媒体库、智能推荐
        </p>

        <div className="mb-8 grid w-full max-w-xl grid-cols-2 gap-3">
          {defaultSuggestions.map((suggestion, i) => (
            <button
              key={i}
              onClick={() => handleSuggestionClick(suggestion.text)}
              className="group flex items-center gap-3 rounded-2xl border border-border/50 bg-card/30 p-4 text-left transition-all hover:border-cyan-500/30 hover:bg-cyan-500/5"
            >
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-cyan-500/10 text-cyan-500">
                {suggestion.icon}
              </div>
              <span className="text-sm">{suggestion.text}</span>
            </button>
          ))}
        </div>
      </div>
    </div>
  )

  // 根据当前视图渲染
  if (currentView === "settings") return <SettingsView />
  if (currentView === "skills") return <SkillsView />
  return <ChatView />
}
