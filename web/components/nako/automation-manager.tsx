"use client"

import { useState } from "react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Switch } from "@/components/ui/switch"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Textarea } from "@/components/ui/textarea"
import { 
  ArrowLeft, 
  Plus, 
  Play, 
  Pause, 
  MoreHorizontal,
  Zap,
  Clock,
  FolderInput,
  Download,
  Film,
  Subtitles,
  Bell,
  Trash2,
  Edit,
  Copy,
  ChevronRight,
  CheckCircle2,
  XCircle,
  AlertCircle,
  FileDown,
  FolderSync,
  Search,
  ListPlus,
  Workflow,
  Calendar,
  ArrowRight,
  Settings,
  History
} from "lucide-react"
import { cn } from "@/lib/utils"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"

interface AutomationManagerProps {
  onBack: () => void
}

// 自动化工作流类型
interface Automation {
  id: string
  name: string
  description: string
  trigger: TriggerConfig
  actions: ActionConfig[]
  enabled: boolean
  lastRun?: string
  runCount: number
  status: "idle" | "running" | "success" | "error"
}

interface TriggerConfig {
  type: "file_added" | "schedule" | "manual" | "webhook" | "media_added"
  config: Record<string, unknown>
}

interface ActionConfig {
  id: string
  skill: string
  name: string
  config: Record<string, unknown>
  order: number
}

// 预设自动化
const defaultAutomations: Automation[] = [
  {
    id: "auto-1",
    name: "新媒体入库处理",
    description: "当新文件添加到下载目录时，自动识别、刮削并整理到媒体库",
    trigger: { type: "file_added", config: { path: "/downloads", patterns: ["*.mkv", "*.mp4"] } },
    actions: [
      { id: "a1", skill: "scrape_metadata", name: "刮削元数据", config: {}, order: 1 },
      { id: "a2", skill: "organize_library", name: "整理到媒体库", config: { rule: "default" }, order: 2 },
      { id: "a3", skill: "notify", name: "发送通知", config: { channel: "telegram" }, order: 3 },
    ],
    enabled: true,
    lastRun: "10分钟前",
    runCount: 156,
    status: "success"
  },
  {
    id: "auto-2", 
    name: "剧集自动订阅",
    description: "每天检查订阅的剧集是否有更新，自动下载新集",
    trigger: { type: "schedule", config: { cron: "0 */6 * * *" } },
    actions: [
      { id: "a1", skill: "check_subscriptions", name: "检查订阅更新", config: {}, order: 1 },
      { id: "a2", skill: "search_resource", name: "搜索资源", config: { quality: "1080p" }, order: 2 },
      { id: "a3", skill: "download_media", name: "下载", config: {}, order: 3 },
    ],
    enabled: true,
    lastRun: "2小时前",
    runCount: 89,
    status: "success"
  },
  {
    id: "auto-3",
    name: "自动生成字幕",
    description: "为没有字幕的视频自动生成字幕（需要 Whisper 插件）",
    trigger: { type: "media_added", config: { types: ["movie", "episode"] } },
    actions: [
      { id: "a1", skill: "check_subtitle", name: "检查字幕", config: {}, order: 1 },
      { id: "a2", skill: "generate_subtitle", name: "生成字幕", config: { language: "zh" }, order: 2 },
    ],
    enabled: false,
    lastRun: "从未运行",
    runCount: 0,
    status: "idle"
  },
  {
    id: "auto-4",
    name: "每周媒体库报告",
    description: "每周生成媒体库统计报告并发送到邮箱",
    trigger: { type: "schedule", config: { cron: "0 9 * * 1" } },
    actions: [
      { id: "a1", skill: "generate_report", name: "生成报告", config: {}, order: 1 },
      { id: "a2", skill: "send_email", name: "发送邮件", config: {}, order: 2 },
    ],
    enabled: true,
    lastRun: "3天前",
    runCount: 12,
    status: "success"
  },
]

// 可用的触发器类型
const triggerTypes = [
  { value: "file_added", label: "文件添加", icon: FolderInput, description: "当文件添加到指定目录时触发" },
  { value: "schedule", label: "定时任务", icon: Clock, description: "按照设定的时间周期执行" },
  { value: "media_added", label: "媒体入库", icon: Film, description: "当新媒体添加到库时触发" },
  { value: "manual", label: "手动触发", icon: Play, description: "需要手动点击运行" },
  { value: "webhook", label: "Webhook", icon: Zap, description: "通过 HTTP 请求触发" },
]

// 可用的动作/Skills
const availableActions = [
  { value: "scrape_metadata", label: "刮削元数据", icon: Film, category: "媒体处理" },
  { value: "organize_library", label: "整理媒体库", icon: FolderSync, category: "库管理" },
  { value: "search_resource", label: "搜索资源", icon: Search, category: "资源获取" },
  { value: "download_media", label: "下载媒体", icon: Download, category: "资源获取" },
  { value: "generate_subtitle", label: "生成字幕", icon: Subtitles, category: "媒体处理" },
  { value: "translate_subtitle", label: "翻译字幕", icon: Subtitles, category: "媒体处理" },
  { value: "notify", label: "发送通知", icon: Bell, category: "通知" },
  { value: "send_email", label: "发送邮件", icon: Bell, category: "通知" },
  { value: "check_subscriptions", label: "检查订阅", icon: ListPlus, category: "资源获取" },
  { value: "generate_report", label: "生成报告", icon: FileDown, category: "其他" },
]

// 运行历史
const runHistory = [
  { id: 1, automation: "新媒体入库处理", status: "success", time: "10分钟前", duration: "12秒", details: "处理了 1 个文件" },
  { id: 2, automation: "剧集自动订阅", status: "success", time: "2小时前", duration: "45秒", details: "发现 2 个更新" },
  { id: 3, automation: "新媒体入库处理", status: "success", time: "3小时前", duration: "8秒", details: "处理了 1 个文件" },
  { id: 4, automation: "新媒体入库处理", status: "error", time: "5小时前", duration: "3秒", details: "无法识别文件" },
  { id: 5, automation: "每周媒体库报告", status: "success", time: "3天前", duration: "2分钟", details: "报告已发送" },
]

export function AutomationManager({ onBack }: AutomationManagerProps) {
  const [automations, setAutomations] = useState(defaultAutomations)
  const [activeTab, setActiveTab] = useState<"automations" | "history" | "templates">("automations")
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [editingAutomation, setEditingAutomation] = useState<Automation | null>(null)
  const [newAutomation, setNewAutomation] = useState({
    name: "",
    description: "",
    triggerType: "file_added",
  })

  // 切换自动化启用状态
  const toggleAutomation = (id: string) => {
    setAutomations(prev => prev.map(a => 
      a.id === id ? { ...a, enabled: !a.enabled } : a
    ))
  }

  // 手动运行
  const runAutomation = (id: string) => {
    setAutomations(prev => prev.map(a => 
      a.id === id ? { ...a, status: "running" } : a
    ))
    // 模拟运行
    setTimeout(() => {
      setAutomations(prev => prev.map(a => 
        a.id === id ? { ...a, status: "success", lastRun: "刚刚", runCount: a.runCount + 1 } : a
      ))
    }, 2000)
  }

  // 删除自动化
  const deleteAutomation = (id: string) => {
    setAutomations(prev => prev.filter(a => a.id !== id))
  }

  // 获取状态图标
  const getStatusIcon = (status: Automation["status"]) => {
    switch (status) {
      case "running": return <div className="h-2 w-2 animate-pulse rounded-full bg-blue-500" />
      case "success": return <CheckCircle2 className="h-4 w-4 text-green-500" />
      case "error": return <XCircle className="h-4 w-4 text-destructive" />
      default: return <div className="h-2 w-2 rounded-full bg-muted-foreground/30" />
    }
  }

  // 获取触发器图标
  const getTriggerIcon = (type: string) => {
    const trigger = triggerTypes.find(t => t.value === type)
    return trigger?.icon || Zap
  }

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Header */}
      <header className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="icon" onClick={onBack}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-lg font-semibold">自动化管理</h1>
            <p className="text-xs text-muted-foreground">配置自动化工作流</p>
          </div>
        </div>

        <div className="flex items-center gap-2">
          <Tabs value={activeTab} onValueChange={(v) => setActiveTab(v as typeof activeTab)}>
            <TabsList className="h-8">
              <TabsTrigger value="automations" className="h-7 px-3 text-xs">工作流</TabsTrigger>
              <TabsTrigger value="history" className="h-7 px-3 text-xs">运行历史</TabsTrigger>
              <TabsTrigger value="templates" className="h-7 px-3 text-xs">模板</TabsTrigger>
            </TabsList>
          </Tabs>
          <Button size="sm" className="gap-1" onClick={() => setShowCreateDialog(true)}>
            <Plus className="h-4 w-4" />
            新建
          </Button>
        </div>
      </header>

      {/* Content */}
      <ScrollArea className="flex-1">
        {activeTab === "automations" && (
          <div className="p-4 space-y-4">
            {/* Stats */}
            <div className="grid grid-cols-3 gap-4">
              <Card>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-2xl font-bold">{automations.filter(a => a.enabled).length}</p>
                      <p className="text-xs text-muted-foreground">活跃工作流</p>
                    </div>
                    <Workflow className="h-8 w-8 text-primary/20" />
                  </div>
                </CardContent>
              </Card>
              <Card>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-2xl font-bold">{automations.reduce((acc, a) => acc + a.runCount, 0)}</p>
                      <p className="text-xs text-muted-foreground">总运行次数</p>
                    </div>
                    <Play className="h-8 w-8 text-primary/20" />
                  </div>
                </CardContent>
              </Card>
              <Card>
                <CardContent className="p-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-2xl font-bold">{automations.filter(a => a.status === "success").length}</p>
                      <p className="text-xs text-muted-foreground">最近成功</p>
                    </div>
                    <CheckCircle2 className="h-8 w-8 text-green-500/20" />
                  </div>
                </CardContent>
              </Card>
            </div>

            {/* Automation List */}
            <div className="space-y-3">
              {automations.map((automation) => {
                const TriggerIcon = getTriggerIcon(automation.trigger.type)
                return (
                  <Card key={automation.id} className={cn(!automation.enabled && "opacity-60")}>
                    <CardContent className="p-4">
                      <div className="flex items-start gap-4">
                        {/* Status & Icon */}
                        <div className="flex flex-col items-center gap-2">
                          <div className={cn(
                            "flex h-10 w-10 items-center justify-center rounded-lg",
                            automation.enabled ? "bg-primary/10 text-primary" : "bg-muted text-muted-foreground"
                          )}>
                            <TriggerIcon className="h-5 w-5" />
                          </div>
                          {getStatusIcon(automation.status)}
                        </div>

                        {/* Info */}
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <h3 className="font-medium">{automation.name}</h3>
                            <Badge variant="outline" className="text-[10px]">
                              {triggerTypes.find(t => t.value === automation.trigger.type)?.label}
                            </Badge>
                          </div>
                          <p className="mt-1 text-sm text-muted-foreground line-clamp-1">
                            {automation.description}
                          </p>
                          
                          {/* Actions preview */}
                          <div className="mt-2 flex items-center gap-1 text-xs text-muted-foreground">
                            {automation.actions.slice(0, 3).map((action, i) => (
                              <span key={action.id} className="flex items-center gap-1">
                                {i > 0 && <ArrowRight className="h-3 w-3" />}
                                <span className="rounded bg-muted px-1.5 py-0.5">{action.name}</span>
                              </span>
                            ))}
                            {automation.actions.length > 3 && (
                              <span className="text-muted-foreground">+{automation.actions.length - 3}</span>
                            )}
                          </div>

                          {/* Stats */}
                          <div className="mt-2 flex items-center gap-4 text-xs text-muted-foreground">
                            <span className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              {automation.lastRun}
                            </span>
                            <span className="flex items-center gap-1">
                              <Play className="h-3 w-3" />
                              {automation.runCount} 次运行
                            </span>
                          </div>
                        </div>

                        {/* Actions */}
                        <div className="flex items-center gap-2">
                          <Switch
                            checked={automation.enabled}
                            onCheckedChange={() => toggleAutomation(automation.id)}
                          />
                          <Button 
                            variant="outline" 
                            size="sm"
                            onClick={() => runAutomation(automation.id)}
                            disabled={automation.status === "running"}
                          >
                            {automation.status === "running" ? (
                              <>
                                <div className="mr-1 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                                运行中
                              </>
                            ) : (
                              <>
                                <Play className="mr-1 h-3 w-3" />
                                运行
                              </>
                            )}
                          </Button>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button variant="ghost" size="icon" className="h-8 w-8">
                                <MoreHorizontal className="h-4 w-4" />
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem onClick={() => setEditingAutomation(automation)}>
                                <Edit className="mr-2 h-4 w-4" />
                                编辑
                              </DropdownMenuItem>
                              <DropdownMenuItem>
                                <Copy className="mr-2 h-4 w-4" />
                                复制
                              </DropdownMenuItem>
                              <DropdownMenuItem>
                                <History className="mr-2 h-4 w-4" />
                                查看日志
                              </DropdownMenuItem>
                              <DropdownMenuSeparator />
                              <DropdownMenuItem 
                                className="text-destructive"
                                onClick={() => deleteAutomation(automation.id)}
                              >
                                <Trash2 className="mr-2 h-4 w-4" />
                                删除
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )
              })}
            </div>
          </div>
        )}

        {activeTab === "history" && (
          <div className="p-4 space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium">运行历史</h3>
              <Button variant="outline" size="sm">导出日志</Button>
            </div>

            <div className="space-y-2">
              {runHistory.map((item) => (
                <div 
                  key={item.id}
                  className="flex items-center gap-4 rounded-lg border border-border p-3"
                >
                  {item.status === "success" ? (
                    <CheckCircle2 className="h-5 w-5 text-green-500 shrink-0" />
                  ) : (
                    <XCircle className="h-5 w-5 text-destructive shrink-0" />
                  )}
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium">{item.automation}</p>
                    <p className="text-xs text-muted-foreground">{item.details}</p>
                  </div>
                  <div className="text-right shrink-0">
                    <p className="text-xs text-muted-foreground">{item.time}</p>
                    <p className="text-xs text-muted-foreground">耗时 {item.duration}</p>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}

        {activeTab === "templates" && (
          <div className="p-4 space-y-4">
            <p className="text-sm text-muted-foreground">选择一个模板快速创建工作流</p>
            
            <div className="grid gap-4 sm:grid-cols-2">
              {[
                { name: "新媒体自动处理", desc: "文件入库时自动刮削、整理、通知", icon: FolderInput },
                { name: "剧集追更", desc: "自动检查并下载订阅的新剧集", icon: ListPlus },
                { name: "自动字幕", desc: "为无字幕视频自动生成字幕", icon: Subtitles },
                { name: "定期报告", desc: "每周生成媒体库统计报告", icon: FileDown },
                { name: "存储清理", desc: "定期清理缓存和临时文件", icon: Trash2 },
                { name: "备份任务", desc: "定期备份数据库和配置", icon: Calendar },
              ].map((template) => (
                <Card 
                  key={template.name}
                  className="cursor-pointer transition-colors hover:bg-muted/50"
                  onClick={() => {
                    setNewAutomation({ name: template.name, description: template.desc, triggerType: "file_added" })
                    setShowCreateDialog(true)
                  }}
                >
                  <CardContent className="flex items-center gap-3 p-4">
                    <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                      <template.icon className="h-5 w-5 text-primary" />
                    </div>
                    <div>
                      <h4 className="font-medium">{template.name}</h4>
                      <p className="text-xs text-muted-foreground">{template.desc}</p>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          </div>
        )}
      </ScrollArea>

      {/* Create Dialog */}
      <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>创建自动化工作流</DialogTitle>
            <DialogDescription>
              配置触发条件和执行动作
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label>名称</Label>
              <Input 
                placeholder="例如：新媒体自动处理"
                value={newAutomation.name}
                onChange={(e) => setNewAutomation(prev => ({ ...prev, name: e.target.value }))}
              />
            </div>

            <div className="space-y-2">
              <Label>描述</Label>
              <Textarea 
                placeholder="描述这个工作流的用途..."
                value={newAutomation.description}
                onChange={(e) => setNewAutomation(prev => ({ ...prev, description: e.target.value }))}
              />
            </div>

            <div className="space-y-2">
              <Label>触发方式</Label>
              <Select 
                value={newAutomation.triggerType}
                onValueChange={(v) => setNewAutomation(prev => ({ ...prev, triggerType: v }))}
              >
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {triggerTypes.map((trigger) => (
                    <SelectItem key={trigger.value} value={trigger.value}>
                      <div className="flex items-center gap-2">
                        <trigger.icon className="h-4 w-4" />
                        <span>{trigger.label}</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                {triggerTypes.find(t => t.value === newAutomation.triggerType)?.description}
              </p>
            </div>

            <div className="space-y-2">
              <Label>动作</Label>
              <p className="text-xs text-muted-foreground mb-2">
                选择工作流执行的动作（按顺序执行）
              </p>
              <div className="grid grid-cols-2 gap-2">
                {availableActions.slice(0, 6).map((action) => (
                  <button
                    key={action.value}
                    className="flex items-center gap-2 rounded-lg border border-border p-2 text-left text-sm transition-colors hover:bg-muted/50"
                  >
                    <action.icon className="h-4 w-4 text-muted-foreground" />
                    <span>{action.label}</span>
                  </button>
                ))}
              </div>
            </div>
          </div>

          <DialogFooter>
            <Button variant="outline" onClick={() => setShowCreateDialog(false)}>
              取消
            </Button>
            <Button onClick={() => setShowCreateDialog(false)}>
              创建工作流
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
