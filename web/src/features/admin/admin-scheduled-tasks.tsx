"use client"

import { useEffect, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import { 
  Calendar, Clock, Play, Pause, Plus, MoreVertical, Trash2, Edit2,
  RefreshCw, FolderSearch, Database, Download, Shield, HardDrive,
  CheckCircle2, XCircle, AlertCircle, Timer, Settings, Film, Tv,
  Music, Image, Archive, Zap, FileText, Globe, AlertTriangle,
  ChevronLeft, ChevronRight, History
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Switch } from "@/components/ui/switch"
import { Checkbox } from "@/components/ui/checkbox"
import { Textarea } from "@/components/ui/textarea"
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs"
import { Progress } from "@/components/ui/progress"
import { Slider } from "@/components/ui/slider"
import { cn } from "@/lib/utils"
import {
  ADMIN_TASKS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminLibraryKind,
} from "@/src/api/admin/read-models-data-source"

type TaskType = "scan" | "metadata" | "backup" | "cleanup" | "update" | "optimize" | "subtitle" | "thumbnail"
type TaskStatus = "idle" | "running" | "success" | "failed" | "scheduled"
type ScheduleFrequency = "interval" | "daily" | "weekly" | "monthly" | "cron"

interface TaskConfig {
  // 扫描任务
  targetLibraries?: string[]
  scanNewOnly?: boolean
  // 备份任务
  backupPath?: string
  keepBackups?: number
  includeImages?: boolean
  // 清理任务
  cacheMaxAge?: number // days
  minFreeSpace?: number // GB
  cleanTranscodes?: boolean
  cleanImages?: boolean
  cleanLogs?: boolean
  // 元数据任务
  metadataLanguage?: string
  refreshAll?: boolean
  downloadImages?: boolean
  // 字幕任务
  subtitleLanguages?: string[]
  preferSDH?: boolean
  // 缩略图任务
  overwriteExisting?: boolean
  chapters?: boolean
}

interface ScheduledTask {
  id: string
  name: string
  description: string
  type: TaskType
  enabled: boolean
  schedule: {
    frequency: ScheduleFrequency
    time?: string
    days?: number[] // 0-6 for weekly, 1-31 for monthly
    interval?: number // minutes
    cron?: string
  }
  config: TaskConfig
  lastRun?: {
    timestamp: string
    status: "success" | "failed"
    duration: string
    itemsProcessed?: number
    error?: string
  }
  nextRun?: string
  status: TaskStatus
  progress?: number
}

const taskTypeConfig: Record<TaskType, { 
  icon: React.ComponentType<{ className?: string }>
  label: string
  color: string
  description: string
}> = {
  scan: { icon: FolderSearch, label: "媒体库扫描", color: "text-blue-500", description: "扫描媒体库以发现新内容" },
  metadata: { icon: RefreshCw, label: "元数据刷新", color: "text-purple-500", description: "从在线数据库更新媒体信息" },
  backup: { icon: Database, label: "数据库备份", color: "text-green-500", description: "备份配置和数据库" },
  cleanup: { icon: Trash2, label: "缓存清理", color: "text-orange-500", description: "清理临时文件和缓存" },
  update: { icon: Download, label: "自动更新", color: "text-cyan-500", description: "检查并安装服务器更新" },
  optimize: { icon: HardDrive, label: "数据库优化", color: "text-yellow-500", description: "优化数据库性能" },
  subtitle: { icon: FileText, label: "字幕下载", color: "text-pink-500", description: "自动下载缺失字幕" },
  thumbnail: { icon: Image, label: "缩略图生成", color: "text-indigo-500", description: "生成视频预览缩略图" },
}

const libraries = [
  { id: "movies", name: "电影", icon: Film },
  { id: "tv", name: "剧集", icon: Tv },
  { id: "anime", name: "动画", icon: Film },
  { id: "music", name: "音乐", icon: Music },
]

const libraryIconByKind: Record<AdminLibraryKind, typeof Film> = {
  movie: Film,
  tv: Tv,
  anime: Film,
  music: Music,
  photo: Image,
  documentary: Film,
  personal: Archive,
  unknown: Archive,
}

// 执行历史记录
interface TaskHistory {
  id: string
  taskId: string
  taskName: string
  taskType: TaskType
  timestamp: string
  status: "success" | "failed"
  duration: string
  itemsProcessed?: number
  error?: string
}

// 生成模拟历史记录
const generateHistory = (count: number = 100): TaskHistory[] => {
  const taskNames = [
    { name: "每日媒体库扫描", type: "scan" as TaskType },
    { name: "元数据刷新", type: "metadata" as TaskType },
    { name: "数据库备份", type: "backup" as TaskType },
    { name: "转码缓存清理", type: "cleanup" as TaskType },
    { name: "数据库优化", type: "optimize" as TaskType },
    { name: "字幕自动下载", type: "subtitle" as TaskType },
  ]
  
  const history: TaskHistory[] = []
  const now = new Date()
  
  for (let i = 0; i < count; i++) {
    const task = taskNames[Math.floor(Math.random() * taskNames.length)]
    const status = Math.random() > 0.1 ? "success" : "failed"
    const timestamp = new Date(now.getTime() - i * 4 * 60 * 60 * 1000) // 每4小时一条
    
    history.push({
      id: `history-${i}`,
      taskId: `task-${Math.floor(Math.random() * 7) + 1}`,
      taskName: task.name,
      taskType: task.type,
      timestamp: timestamp.toISOString(),
      status,
      duration: `${Math.floor(Math.random() * 30)}分${Math.floor(Math.random() * 60)}秒`,
      itemsProcessed: task.type === "scan" || task.type === "metadata" ? Math.floor(Math.random() * 1000) : undefined,
      error: status === "failed" ? "任务执行失败: 权限不足或网络超时" : undefined,
    })
  }
  
  return history
}

const defaultTasks: ScheduledTask[] = [
  {
    id: "1",
    name: "每日媒体库扫描",
    description: "扫描所有媒体库以发现新内容",
    type: "scan",
    enabled: true,
    schedule: { frequency: "daily", time: "03:00" },
    config: { targetLibraries: ["movies", "tv", "anime"], scanNewOnly: true },
    lastRun: {
      timestamp: "2024-03-15T03:00:00Z",
      status: "success",
      duration: "12分34秒",
      itemsProcessed: 847,
    },
    nextRun: "2024-03-16T03:00:00Z",
    status: "scheduled",
  },
  {
    id: "2",
    name: "元数据刷新",
    description: "更新最近添加项目的元数据",
    type: "metadata",
    enabled: true,
    schedule: { frequency: "weekly", time: "04:00", days: [0] },
    config: { metadataLanguage: "zh-CN", refreshAll: false, downloadImages: true },
    lastRun: {
      timestamp: "2024-03-10T04:00:00Z",
      status: "success",
      duration: "45分12秒",
      itemsProcessed: 234,
    },
    nextRun: "2024-03-17T04:00:00Z",
    status: "scheduled",
  },
  {
    id: "3",
    name: "数据库备份",
    description: "创建数据库和配置的备份",
    type: "backup",
    enabled: true,
    schedule: { frequency: "daily", time: "02:00" },
    config: { backupPath: "/backups/nako", keepBackups: 7, includeImages: false },
    lastRun: {
      timestamp: "2024-03-15T02:00:00Z",
      status: "success",
      duration: "2分15秒",
    },
    nextRun: "2024-03-16T02:00:00Z",
    status: "scheduled",
  },
  {
    id: "4",
    name: "转码缓存清理",
    description: "删除旧的转码缓存文件",
    type: "cleanup",
    enabled: true,
    schedule: { frequency: "weekly", time: "05:00", days: [6] },
    config: { cacheMaxAge: 7, minFreeSpace: 50, cleanTranscodes: true, cleanImages: false, cleanLogs: false },
    lastRun: {
      timestamp: "2024-03-09T05:00:00Z",
      status: "failed",
      duration: "0分45秒",
      error: "权限不足: /var/cache/transcodes",
    },
    nextRun: "2024-03-16T05:00:00Z",
    status: "scheduled",
  },
  {
    id: "5",
    name: "自动更新检查",
    description: "检查服务器更新",
    type: "update",
    enabled: false,
    schedule: { frequency: "daily", time: "06:00" },
    config: {},
    lastRun: {
      timestamp: "2024-03-01T06:00:00Z",
      status: "success",
      duration: "0分12秒",
    },
    status: "idle",
  },
  {
    id: "6",
    name: "数据库优化",
    description: "优化数据库性能",
    type: "optimize",
    enabled: true,
    schedule: { frequency: "monthly", time: "01:00", days: [1] },
    config: {},
    lastRun: {
      timestamp: "2024-03-01T01:00:00Z",
      status: "success",
      duration: "8分45秒",
    },
    nextRun: "2024-04-01T01:00:00Z",
    status: "scheduled",
  },
  {
    id: "7",
    name: "字幕自动下载",
    description: "为缺失字幕的视频下载字幕",
    type: "subtitle",
    enabled: true,
    schedule: { frequency: "daily", time: "04:30" },
    config: { subtitleLanguages: ["zh-CN", "zh-TW", "en"], preferSDH: false },
    lastRun: {
      timestamp: "2024-03-15T04:30:00Z",
      status: "success",
      duration: "15分23秒",
      itemsProcessed: 42,
    },
    nextRun: "2024-03-16T04:30:00Z",
    status: "scheduled",
  },
]

export function AdminScheduledTasks() {
  const { data: tasksData = ADMIN_TASKS_READ_MODEL_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "scheduled-tasks"],
    queryFn: () => createAdminReadModelsDataSource().loadTasks(),
    staleTime: 15 * 1000,
    retry: 0,
  })
  const libraries = tasksData.libraries.map((library) => ({
    ...library,
    icon: libraryIconByKind[library.type] ?? Archive,
  }))
  const [taskList, setTaskList] = useState<ScheduledTask[]>(ADMIN_TASKS_READ_MODEL_FIXTURE.tasks)
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [editingTask, setEditingTask] = useState<ScheduledTask | null>(null)
  const [currentRunningTask, setCurrentRunningTask] = useState<ScheduledTask | null>(
    ADMIN_TASKS_READ_MODEL_FIXTURE.runningTask,
  )
  
  // 执行历史相关状态
  const history = tasksData.history
  const [historyPage, setHistoryPage] = useState(1)
  const [historyFilter, setHistoryFilter] = useState<"all" | "success" | "failed">("all")
  const historyPerPage = 10

  useEffect(() => {
    setTaskList(tasksData.tasks)
    setCurrentRunningTask(tasksData.runningTask)
  }, [tasksData])

  // 新任务表单状态
  const [formData, setFormData] = useState({
    name: "",
    type: "scan" as TaskType,
    frequency: "daily" as ScheduleFrequency,
    time: "03:00",
    days: [0] as number[],
    interval: 60,
    cron: "",
    // 配置
    targetLibraries: ["movies", "tv", "anime"] as string[],
    scanNewOnly: true,
    backupPath: "/backups/nako",
    keepBackups: 7,
    includeImages: false,
    cacheMaxAge: 7,
    minFreeSpace: 50,
    cleanTranscodes: true,
    cleanImages: false,
    cleanLogs: false,
    metadataLanguage: "zh-CN",
    refreshAll: false,
    downloadImages: true,
    subtitleLanguages: ["zh-CN", "zh-TW", "en"] as string[],
    preferSDH: false,
    overwriteExisting: false,
    chapters: true,
  })

  const resetForm = () => {
    setFormData({
      name: "",
      type: "scan",
      frequency: "daily",
      time: "03:00",
      days: [0],
      interval: 60,
      cron: "",
      targetLibraries: ["movies", "tv", "anime"],
      scanNewOnly: true,
      backupPath: "/backups/nako",
      keepBackups: 7,
      includeImages: false,
      cacheMaxAge: 7,
      minFreeSpace: 50,
      cleanTranscodes: true,
      cleanImages: false,
      cleanLogs: false,
      metadataLanguage: "zh-CN",
      refreshAll: false,
      downloadImages: true,
      subtitleLanguages: ["zh-CN", "zh-TW", "en"],
      preferSDH: false,
      overwriteExisting: false,
      chapters: true,
    })
    setEditingTask(null)
  }

  const openEditDialog = (task: ScheduledTask) => {
    setEditingTask(task)
    setFormData({
      name: task.name,
      type: task.type,
      frequency: task.schedule.frequency,
      time: task.schedule.time || "03:00",
      days: task.schedule.days || [0],
      interval: task.schedule.interval || 60,
      cron: task.schedule.cron || "",
      targetLibraries: task.config.targetLibraries || ["movies", "tv", "anime"],
      scanNewOnly: task.config.scanNewOnly ?? true,
      backupPath: task.config.backupPath || "/backups/nako",
      keepBackups: task.config.keepBackups || 7,
      includeImages: task.config.includeImages ?? false,
      cacheMaxAge: task.config.cacheMaxAge || 7,
      minFreeSpace: task.config.minFreeSpace || 50,
      cleanTranscodes: task.config.cleanTranscodes ?? true,
      cleanImages: task.config.cleanImages ?? false,
      cleanLogs: task.config.cleanLogs ?? false,
      metadataLanguage: task.config.metadataLanguage || "zh-CN",
      refreshAll: task.config.refreshAll ?? false,
      downloadImages: task.config.downloadImages ?? true,
      subtitleLanguages: task.config.subtitleLanguages || ["zh-CN", "zh-TW", "en"],
      preferSDH: task.config.preferSDH ?? false,
      overwriteExisting: task.config.overwriteExisting ?? false,
      chapters: task.config.chapters ?? true,
    })
    setIsDialogOpen(true)
  }

  const handleSaveTask = () => {
    const newTask: ScheduledTask = {
      id: editingTask?.id || `task-${Date.now()}`,
      name: formData.name || taskTypeConfig[formData.type].label,
      description: taskTypeConfig[formData.type].description,
      type: formData.type,
      enabled: true,
      schedule: {
        frequency: formData.frequency,
        time: formData.time,
        days: formData.days,
        interval: formData.interval,
        cron: formData.cron,
      },
      config: {
        targetLibraries: formData.targetLibraries,
        scanNewOnly: formData.scanNewOnly,
        backupPath: formData.backupPath,
        keepBackups: formData.keepBackups,
        includeImages: formData.includeImages,
        cacheMaxAge: formData.cacheMaxAge,
        minFreeSpace: formData.minFreeSpace,
        cleanTranscodes: formData.cleanTranscodes,
        cleanImages: formData.cleanImages,
        cleanLogs: formData.cleanLogs,
        metadataLanguage: formData.metadataLanguage,
        refreshAll: formData.refreshAll,
        downloadImages: formData.downloadImages,
        subtitleLanguages: formData.subtitleLanguages,
        preferSDH: formData.preferSDH,
        overwriteExisting: formData.overwriteExisting,
        chapters: formData.chapters,
      },
      status: "scheduled",
      nextRun: new Date(Date.now() + 86400000).toISOString(),
    }

    if (editingTask) {
      setTaskList(prev => prev.map(t => t.id === editingTask.id ? newTask : t))
    } else {
      setTaskList(prev => [...prev, newTask])
    }

    setIsDialogOpen(false)
    resetForm()
  }

  const handleDeleteTask = (taskId: string) => {
    setTaskList(prev => prev.filter(t => t.id !== taskId))
  }

  const formatSchedule = (schedule: ScheduledTask["schedule"]) => {
    const dayNames = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"]
    
    switch (schedule.frequency) {
      case "interval":
        return `每 ${schedule.interval} 分钟`
      case "daily":
        return `每天 ${schedule.time}`
      case "weekly":
        const days = schedule.days?.map(d => dayNames[d]).join(", ")
        return `每周${days} ${schedule.time}`
      case "monthly":
        return `每月 ${schedule.days?.[0]} 日 ${schedule.time}`
      case "cron":
        return schedule.cron || "手动触发"
      default:
        return "未知"
    }
  }

  const formatNextRun = (nextRun?: string) => {
    if (!nextRun) return "未计划"
    const date = new Date(nextRun)
    const now = new Date()
    const diff = date.getTime() - now.getTime()
    const hours = Math.floor(diff / (1000 * 60 * 60))
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60))
    
    if (hours > 24) {
      return date.toLocaleDateString("zh-CN", { weekday: "short", month: "short", day: "numeric" })
    }
    if (hours > 0) {
      return `${hours}小时${minutes}分钟后`
    }
    return `${minutes}分钟后`
  }

  const toggleTaskEnabled = (taskId: string) => {
    setTaskList((prev) =>
      prev.map((task) =>
        task.id === taskId ? { ...task, enabled: !task.enabled } : task
      )
    )
  }

  const handleRunNow = (task: ScheduledTask) => {
    setCurrentRunningTask({
      ...task,
      status: "running",
      progress: 0,
    })
    let progress = 0
    const interval = setInterval(() => {
      progress += Math.random() * 15
      if (progress >= 100) {
        clearInterval(interval)
        setCurrentRunningTask(null)
      } else {
        setCurrentRunningTask((prev) => prev ? { ...prev, progress } : null)
      }
    }, 500)
  }

  const getStatusBadge = (task: ScheduledTask) => {
    if (!task.enabled) {
      return <Badge variant="secondary">已禁用</Badge>
    }
    if (task.status === "running") {
      return <Badge className="gap-1 bg-blue-500">
        <RefreshCw className="h-3 w-3 animate-spin" />
        运行中
      </Badge>
    }
    if (task.lastRun?.status === "failed") {
      return <Badge variant="destructive" className="gap-1">
        <XCircle className="h-3 w-3" />
        失败
      </Badge>
    }
    return <Badge variant="outline" className="gap-1 border-green-500/30 text-green-500">
      <CheckCircle2 className="h-3 w-3" />
      正常
    </Badge>
  }

  const stats = {
    total: taskList.length,
    active: taskList.filter(t => t.enabled).length,
    failed: taskList.filter(t => t.lastRun?.status === "failed").length,
    nextTask: taskList.find(t => t.enabled && t.nextRun),
  }

  return (
    <div className="space-y-6 p-1">
      {/* Header */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-2xl font-bold">计划任务</h1>
          <p className="text-sm text-muted-foreground">
            配置自动化维护和扫描任务
            <span className="ml-2 text-xs">
              {tasksData.source === "live" ? "Live Admin API" : "Fixture fallback"}
              {tasksData.error ? ` · ${tasksData.error}` : ""}
            </span>
          </p>
        </div>
        <Button onClick={() => { resetForm(); setIsDialogOpen(true) }}>
          <Plus className="mr-2 h-4 w-4" />
          新建任务
        </Button>
      </div>

      {/* Running Task */}
      {currentRunningTask && (
        <Card className="border-blue-500/30 bg-blue-500/5">
          <CardHeader className="pb-3">
            <div className="flex items-center justify-between">
              <CardTitle className="flex items-center gap-2 text-base">
                <RefreshCw className="h-4 w-4 animate-spin text-blue-500" />
                正在运行的任务
              </CardTitle>
              <Button variant="outline" size="sm" onClick={() => setCurrentRunningTask(null)}>
                <Pause className="mr-2 h-4 w-4" />
                停止
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-4">
              <div className="flex-1">
                <h3 className="font-medium">{currentRunningTask.name}</h3>
                <p className="text-sm text-muted-foreground">{currentRunningTask.description}</p>
              </div>
              <div className="w-48">
                <div className="mb-1 flex justify-between text-xs">
                  <span>进度</span>
                  <span>{Math.round(currentRunningTask.progress || 0)}%</span>
                </div>
                <Progress value={currentRunningTask.progress} className="h-2" />
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Stats */}
      <div className="grid gap-4 sm:grid-cols-4">
        <Card className="border-border/50 bg-card/50">
          <CardContent className="flex items-center gap-3 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
              <Calendar className="h-5 w-5 text-primary" />
            </div>
            <div>
              <p className="text-2xl font-bold">{stats.total}</p>
              <p className="text-xs text-muted-foreground">总任务数</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50 bg-card/50">
          <CardContent className="flex items-center gap-3 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-green-500/10">
              <CheckCircle2 className="h-5 w-5 text-green-500" />
            </div>
            <div>
              <p className="text-2xl font-bold">{stats.active}</p>
              <p className="text-xs text-muted-foreground">已启用</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50 bg-card/50">
          <CardContent className="flex items-center gap-3 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-destructive/10">
              <XCircle className="h-5 w-5 text-destructive" />
            </div>
            <div>
              <p className="text-2xl font-bold">{stats.failed}</p>
              <p className="text-xs text-muted-foreground">失败</p>
            </div>
          </CardContent>
        </Card>
        <Card className="border-border/50 bg-card/50">
          <CardContent className="flex items-center gap-3 p-4">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
              <Timer className="h-5 w-5 text-muted-foreground" />
            </div>
            <div>
              <p className="text-sm font-medium">下次运行</p>
              <p className="text-xs text-muted-foreground">
                {formatNextRun(stats.nextTask?.nextRun)}
              </p>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Task List */}
      <div className="space-y-4">
        {taskList.map((task) => {
          const TypeIcon = taskTypeConfig[task.type].icon
          const typeColor = taskTypeConfig[task.type].color
          
          return (
            <Card
              key={task.id}
              className={cn(
                "border-border/50 transition-colors",
                !task.enabled && "opacity-60",
                task.lastRun?.status === "failed" && task.enabled && "border-l-2 border-l-destructive"
              )}
            >
              <CardContent className="p-4">
                <div className="flex flex-col gap-4 sm:flex-row sm:items-center">
                  {/* Icon and Info */}
                  <div className="flex flex-1 items-start gap-4">
                    <div className={cn(
                      "flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-lg bg-muted",
                      task.enabled && "bg-primary/10"
                    )}>
                      <TypeIcon className={cn("h-6 w-6", task.enabled ? typeColor : "text-muted-foreground")} />
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold">{task.name}</h3>
                        {getStatusBadge(task)}
                      </div>
                      <p className="mt-0.5 text-sm text-muted-foreground">{task.description}</p>
                      <div className="mt-2 flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <Clock className="h-3.5 w-3.5" />
                          {formatSchedule(task.schedule)}
                        </span>
                        {task.lastRun && (
                          <span className="flex items-center gap-1">
                            <Calendar className="h-3.5 w-3.5" />
                            上次: {new Date(task.lastRun.timestamp).toLocaleDateString("zh-CN")} ({task.lastRun.duration})
                            {task.lastRun.itemsProcessed !== undefined && (
                              <span className="text-muted-foreground/70">
                                ({task.lastRun.itemsProcessed} 项)
                              </span>
                            )}
                          </span>
                        )}
                        {task.nextRun && task.enabled && (
                          <span className="flex items-center gap-1 text-primary">
                            <Timer className="h-3.5 w-3.5" />
                            下次: {formatNextRun(task.nextRun)}
                          </span>
                        )}
                      </div>
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="flex items-center gap-2 self-end sm:self-center">
                    <Switch
                      checked={task.enabled}
                      onCheckedChange={() => toggleTaskEnabled(task.id)}
                    />
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={!task.enabled || currentRunningTask !== null}
                      onClick={() => handleRunNow(task)}
                    >
                      <Play className="mr-2 h-4 w-4" />
                      立即运行
                    </Button>
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" size="icon" className="h-8 w-8">
                          <MoreVertical className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => openEditDialog(task)}>
                          <Edit2 className="mr-2 h-4 w-4" />
                          编辑
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => openEditDialog(task)}>
                          <Settings className="mr-2 h-4 w-4" />
                          配置参数
                        </DropdownMenuItem>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem 
                          className="text-destructive"
                          onClick={() => handleDeleteTask(task.id)}
                        >
                          <Trash2 className="mr-2 h-4 w-4" />
                          删除
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>
                </div>

                {/* Last Run Error */}
                {task.lastRun?.status === "failed" && task.lastRun.error && (
                  <div className="mt-3 flex items-start gap-2 rounded-lg bg-destructive/10 px-3 py-2 text-sm text-destructive">
                    <AlertCircle className="h-4 w-4 flex-shrink-0 mt-0.5" />
                    <div>
                      <span className="font-medium">上次运行失败: </span>
                      <span>{task.lastRun.error}</span>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )
        })}
      </div>

      {/* 执行历史 */}
      <Card className="border-border/50">
        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <History className="h-5 w-5 text-muted-foreground" />
              <CardTitle className="text-base">执行历史</CardTitle>
            </div>
            <div className="flex items-center gap-2">
              <Select value={historyFilter} onValueChange={(v: "all" | "success" | "failed") => { setHistoryFilter(v); setHistoryPage(1) }}>
                <SelectTrigger className="h-8 w-[100px] text-xs">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">全部</SelectItem>
                  <SelectItem value="success">成功</SelectItem>
                  <SelectItem value="failed">失败</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          {(() => {
            const filteredHistory = history.filter(h => 
              historyFilter === "all" || h.status === historyFilter
            )
            const totalPages = Math.ceil(filteredHistory.length / historyPerPage)
            const paginatedHistory = filteredHistory.slice(
              (historyPage - 1) * historyPerPage,
              historyPage * historyPerPage
            )
            
            return (
              <>
                <div className="divide-y divide-border/30">
                  {paginatedHistory.map((item) => {
                    const typeConfig = taskTypeConfig[item.taskType]
                    const TypeIcon = typeConfig.icon
                    
                    return (
                      <div key={item.id} className="flex items-center gap-4 px-4 py-3 hover:bg-muted/30">
                        <div className={cn(
                          "flex h-8 w-8 items-center justify-center rounded-lg",
                          item.status === "success" ? "bg-green-500/10" : "bg-destructive/10"
                        )}>
                          {item.status === "success" ? (
                            <CheckCircle2 className="h-4 w-4 text-green-500" />
                          ) : (
                            <XCircle className="h-4 w-4 text-destructive" />
                          )}
                        </div>
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <TypeIcon className={cn("h-3.5 w-3.5", typeConfig.color)} />
                            <span className="font-medium text-sm truncate">{item.taskName}</span>
                          </div>
                          <div className="flex items-center gap-3 text-xs text-muted-foreground mt-0.5">
                            <span>{new Date(item.timestamp).toLocaleString("zh-CN", { 
                              month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" 
                            })}</span>
                            <span>{item.duration}</span>
                            {item.itemsProcessed !== undefined && (
                              <span>{item.itemsProcessed} 项</span>
                            )}
                          </div>
                          {item.error && (
                            <p className="text-xs text-destructive mt-1 truncate">{item.error}</p>
                          )}
                        </div>
                      </div>
                    )
                  })}
                </div>
                
                {/* 分页控制 */}
                <div className="flex items-center justify-between border-t border-border/30 px-4 py-3">
                  <p className="text-xs text-muted-foreground">
                    共 {filteredHistory.length} 条记录，第 {historyPage}/{totalPages} 页
                  </p>
                  <div className="flex items-center gap-1">
                    <Button
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={historyPage === 1}
                      onClick={() => setHistoryPage(1)}
                    >
                      <ChevronLeft className="h-3 w-3" />
                      <ChevronLeft className="h-3 w-3 -ml-2" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={historyPage === 1}
                      onClick={() => setHistoryPage(p => Math.max(1, p - 1))}
                    >
                      <ChevronLeft className="h-3 w-3" />
                    </Button>
                    <span className="px-2 text-xs text-muted-foreground min-w-[60px] text-center">
                      {historyPage} / {totalPages}
                    </span>
                    <Button
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={historyPage === totalPages}
                      onClick={() => setHistoryPage(p => Math.min(totalPages, p + 1))}
                    >
                      <ChevronRight className="h-3 w-3" />
                    </Button>
                    <Button
                      variant="outline"
                      size="icon"
                      className="h-7 w-7"
                      disabled={historyPage === totalPages}
                      onClick={() => setHistoryPage(totalPages)}
                    >
                      <ChevronRight className="h-3 w-3" />
                      <ChevronRight className="h-3 w-3 -ml-2" />
                    </Button>
                  </div>
                </div>
              </>
            )
          })()}
        </CardContent>
      </Card>

      {/* Create/Edit Dialog */}
      <Dialog open={isDialogOpen} onOpenChange={(open) => { setIsDialogOpen(open); if (!open) resetForm() }}>
        <DialogContent className="max-w-2xl max-h-[85vh] overflow-y-auto scrollbar-none">
          <DialogHeader>
            <DialogTitle>{editingTask ? "编辑任务" : "新建计划任务"}</DialogTitle>
            <DialogDescription>
              {editingTask ? "修改任务的计划和配置参数" : "设置一个新的自动化任务及其执行计划"}
            </DialogDescription>
          </DialogHeader>
          
          <Tabs defaultValue="basic" className="w-full">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="basic">基本设置</TabsTrigger>
              <TabsTrigger value="schedule">执行计划</TabsTrigger>
              <TabsTrigger value="config">任务参数</TabsTrigger>
            </TabsList>
            
            {/* Basic Settings */}
            <TabsContent value="basic" className="space-y-4 mt-4">
              <div>
                <Label htmlFor="task-name">任务名称</Label>
                <Input 
                  id="task-name" 
                  placeholder="例如: 每日媒体库扫描" 
                  className="mt-1.5"
                  value={formData.name}
                  onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
                />
              </div>
              <div>
                <Label>任务类型</Label>
                <div className="grid grid-cols-2 gap-2 mt-1.5">
                  {Object.entries(taskTypeConfig).map(([key, config]) => {
                    const Icon = config.icon
                    const isSelected = formData.type === key
                    return (
                      <button
                        key={key}
                        type="button"
                        onClick={() => setFormData(prev => ({ ...prev, type: key as TaskType }))}
                        className={cn(
                          "flex items-start gap-3 rounded-lg border p-3 text-left transition-colors",
                          isSelected 
                            ? "border-primary bg-primary/5" 
                            : "border-border hover:border-primary/50"
                        )}
                      >
                        <Icon className={cn("h-5 w-5 mt-0.5", config.color)} />
                        <div>
                          <p className="font-medium text-sm">{config.label}</p>
                          <p className="text-xs text-muted-foreground">{config.description}</p>
                        </div>
                      </button>
                    )
                  })}
                </div>
              </div>
            </TabsContent>

            {/* Schedule Settings */}
            <TabsContent value="schedule" className="space-y-4 mt-4">
              <div>
                <Label>执行频率</Label>
                <Select 
                  value={formData.frequency} 
                  onValueChange={(v) => setFormData(prev => ({ ...prev, frequency: v as ScheduleFrequency }))}
                >
                  <SelectTrigger className="mt-1.5">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="interval">固定间隔</SelectItem>
                    <SelectItem value="daily">每天</SelectItem>
                    <SelectItem value="weekly">每周</SelectItem>
                    <SelectItem value="monthly">每月</SelectItem>
                    <SelectItem value="cron">Cron 表达式</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {formData.frequency === "interval" && (
                <div>
                  <Label>间隔时间 (分钟)</Label>
                  <div className="flex items-center gap-4 mt-1.5">
                    <Slider
                      value={[formData.interval]}
                      onValueChange={([v]) => setFormData(prev => ({ ...prev, interval: v }))}
                      min={5}
                      max={1440}
                      step={5}
                      className="flex-1"
                    />
                    <span className="w-20 text-sm text-muted-foreground">{formData.interval} 分钟</span>
                  </div>
                </div>
              )}

              {(formData.frequency === "daily" || formData.frequency === "weekly" || formData.frequency === "monthly") && (
                <div>
                  <Label htmlFor="time">执行时间</Label>
                  <Input 
                    id="time" 
                    type="time" 
                    className="mt-1.5 w-32"
                    value={formData.time}
                    onChange={(e) => setFormData(prev => ({ ...prev, time: e.target.value }))}
                  />
                </div>
              )}

              {formData.frequency === "weekly" && (
                <div>
                  <Label>执行日期</Label>
                  <div className="flex flex-wrap gap-2 mt-1.5">
                    {["周日", "周一", "周二", "周三", "周四", "周五", "周六"].map((day, idx) => (
                      <Button
                        key={idx}
                        type="button"
                        variant={formData.days.includes(idx) ? "default" : "outline"}
                        size="sm"
                        onClick={() => {
                          setFormData(prev => ({
                            ...prev,
                            days: prev.days.includes(idx)
                              ? prev.days.filter(d => d !== idx)
                              : [...prev.days, idx]
                          }))
                        }}
                      >
                        {day}
                      </Button>
                    ))}
                  </div>
                </div>
              )}

              {formData.frequency === "monthly" && (
                <div>
                  <Label>执行日���</Label>
                  <Select 
                    value={String(formData.days[0] || 1)}
                    onValueChange={(v) => setFormData(prev => ({ ...prev, days: [parseInt(v)] }))}
                  >
                    <SelectTrigger className="mt-1.5 w-32">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {Array.from({ length: 28 }, (_, i) => (
                        <SelectItem key={i + 1} value={String(i + 1)}>
                          每月 {i + 1} 日
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {formData.frequency === "cron" && (
                <div>
                  <Label htmlFor="cron">Cron 表达式</Label>
                  <Input 
                    id="cron" 
                    placeholder="0 3 * * *" 
                    className="mt-1.5 font-mono"
                    value={formData.cron}
                    onChange={(e) => setFormData(prev => ({ ...prev, cron: e.target.value }))}
                  />
                  <p className="text-xs text-muted-foreground mt-1">格式: 分 时 日 月 周</p>
                </div>
              )}
            </TabsContent>

            {/* Task-specific Config */}
            <TabsContent value="config" className="space-y-4 mt-4">
              {/* Scan Task Config */}
              {formData.type === "scan" && (
                <>
                  <div>
                    <Label>目标媒体库</Label>
                    <div className="flex flex-wrap gap-2 mt-1.5">
                      {libraries.map(lib => {
                        const Icon = lib.icon
                        const isSelected = formData.targetLibraries.includes(lib.id)
                        return (
                          <Button
                            key={lib.id}
                            type="button"
                            variant={isSelected ? "default" : "outline"}
                            size="sm"
                            className="gap-1.5"
                            onClick={() => {
                              setFormData(prev => ({
                                ...prev,
                                targetLibraries: isSelected
                                  ? prev.targetLibraries.filter(id => id !== lib.id)
                                  : [...prev.targetLibraries, lib.id]
                              }))
                            }}
                          >
                            <Icon className="h-3.5 w-3.5" />
                            {lib.name}
                          </Button>
                        )
                      })}
                    </div>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>仅扫描新文件</Label>
                      <p className="text-xs text-muted-foreground">跳过已索引的文件以加快扫描速度</p>
                    </div>
                    <Switch 
                      checked={formData.scanNewOnly}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, scanNewOnly: v }))}
                    />
                  </div>
                </>
              )}

              {/* Backup Task Config */}
              {formData.type === "backup" && (
                <>
                  <div>
                    <Label htmlFor="backup-path">备份路径</Label>
                    <Input 
                      id="backup-path"
                      className="mt-1.5 font-mono"
                      value={formData.backupPath}
                      onChange={(e) => setFormData(prev => ({ ...prev, backupPath: e.target.value }))}
                    />
                  </div>
                  <div>
                    <Label>保留备份数量</Label>
                    <div className="flex items-center gap-4 mt-1.5">
                      <Slider
                        value={[formData.keepBackups]}
                        onValueChange={([v]) => setFormData(prev => ({ ...prev, keepBackups: v }))}
                        min={1}
                        max={30}
                        step={1}
                        className="flex-1"
                      />
                      <span className="w-16 text-sm text-muted-foreground">{formData.keepBackups} 个</span>
                    </div>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>包含图片缓存</Label>
                      <p className="text-xs text-muted-foreground">备份海报和剧照（会增加备份大小）</p>
                    </div>
                    <Switch 
                      checked={formData.includeImages}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, includeImages: v }))}
                    />
                  </div>
                </>
              )}

              {/* Cleanup Task Config */}
              {formData.type === "cleanup" && (
                <>
                  <div>
                    <Label>缓存保留天数</Label>
                    <div className="flex items-center gap-4 mt-1.5">
                      <Slider
                        value={[formData.cacheMaxAge]}
                        onValueChange={([v]) => setFormData(prev => ({ ...prev, cacheMaxAge: v }))}
                        min={1}
                        max={30}
                        step={1}
                        className="flex-1"
                      />
                      <span className="w-16 text-sm text-muted-foreground">{formData.cacheMaxAge} 天</span>
                    </div>
                  </div>
                  <div>
                    <Label>最小可用空间 (GB)</Label>
                    <div className="flex items-center gap-4 mt-1.5">
                      <Slider
                        value={[formData.minFreeSpace]}
                        onValueChange={([v]) => setFormData(prev => ({ ...prev, minFreeSpace: v }))}
                        min={10}
                        max={200}
                        step={10}
                        className="flex-1"
                      />
                      <span className="w-16 text-sm text-muted-foreground">{formData.minFreeSpace} GB</span>
                    </div>
                    <p className="text-xs text-muted-foreground mt-1">当可用空间低于此值时触发清理</p>
                  </div>
                  <div className="space-y-3">
                    <Label>清理内容</Label>
                    <div className="space-y-2">
                      <div className="flex items-center gap-2">
                        <Checkbox 
                          id="clean-transcodes"
                          checked={formData.cleanTranscodes}
                          onCheckedChange={(v) => setFormData(prev => ({ ...prev, cleanTranscodes: v === true }))}
                        />
                        <label htmlFor="clean-transcodes" className="text-sm">转码缓存</label>
                      </div>
                      <div className="flex items-center gap-2">
                        <Checkbox 
                          id="clean-images"
                          checked={formData.cleanImages}
                          onCheckedChange={(v) => setFormData(prev => ({ ...prev, cleanImages: v === true }))}
                        />
                        <label htmlFor="clean-images" className="text-sm">图片缓存</label>
                      </div>
                      <div className="flex items-center gap-2">
                        <Checkbox 
                          id="clean-logs"
                          checked={formData.cleanLogs}
                          onCheckedChange={(v) => setFormData(prev => ({ ...prev, cleanLogs: v === true }))}
                        />
                        <label htmlFor="clean-logs" className="text-sm">旧日志文件</label>
                      </div>
                    </div>
                  </div>
                </>
              )}

              {/* Metadata Task Config */}
              {formData.type === "metadata" && (
                <>
                  <div>
                    <Label>元数据语言</Label>
                    <Select 
                      value={formData.metadataLanguage}
                      onValueChange={(v) => setFormData(prev => ({ ...prev, metadataLanguage: v }))}
                    >
                      <SelectTrigger className="mt-1.5">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="zh-CN">简体中文</SelectItem>
                        <SelectItem value="zh-TW">繁体中文</SelectItem>
                        <SelectItem value="en">English</SelectItem>
                        <SelectItem value="ja">日本語</SelectItem>
                        <SelectItem value="ko">한국어</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>刷新所有项目</Label>
                      <p className="text-xs text-muted-foreground">包括已有元数据的项目</p>
                    </div>
                    <Switch 
                      checked={formData.refreshAll}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, refreshAll: v }))}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>下载图片</Label>
                      <p className="text-xs text-muted-foreground">海报、剧照、演员照片</p>
                    </div>
                    <Switch 
                      checked={formData.downloadImages}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, downloadImages: v }))}
                    />
                  </div>
                </>
              )}

              {/* Subtitle Task Config */}
              {formData.type === "subtitle" && (
                <>
                  <div>
                    <Label>字幕语言优先级</Label>
                    <div className="flex flex-wrap gap-2 mt-1.5">
                      {[
                        { id: "zh-CN", name: "简体中文" },
                        { id: "zh-TW", name: "繁体中文" },
                        { id: "en", name: "English" },
                        { id: "ja", name: "日本語" },
                      ].map(lang => {
                        const isSelected = formData.subtitleLanguages.includes(lang.id)
                        return (
                          <Button
                            key={lang.id}
                            type="button"
                            variant={isSelected ? "default" : "outline"}
                            size="sm"
                            onClick={() => {
                              setFormData(prev => ({
                                ...prev,
                                subtitleLanguages: isSelected
                                  ? prev.subtitleLanguages.filter(id => id !== lang.id)
                                  : [...prev.subtitleLanguages, lang.id]
                              }))
                            }}
                          >
                            {lang.name}
                          </Button>
                        )
                      })}
                    </div>
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>优先 SDH 字幕</Label>
                      <p className="text-xs text-muted-foreground">包含听障描述的字幕</p>
                    </div>
                    <Switch 
                      checked={formData.preferSDH}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, preferSDH: v }))}
                    />
                  </div>
                </>
              )}

              {/* Thumbnail Task Config */}
              {formData.type === "thumbnail" && (
                <>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>覆盖已有缩略图</Label>
                      <p className="text-xs text-muted-foreground">重新生成所有缩略图</p>
                    </div>
                    <Switch 
                      checked={formData.overwriteExisting}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, overwriteExisting: v }))}
                    />
                  </div>
                  <div className="flex items-center justify-between">
                    <div>
                      <Label>生成章节预览</Label>
                      <p className="text-xs text-muted-foreground">为视频章节生成预览图</p>
                    </div>
                    <Switch 
                      checked={formData.chapters}
                      onCheckedChange={(v) => setFormData(prev => ({ ...prev, chapters: v }))}
                    />
                  </div>
                </>
              )}

              {/* Generic config for update/optimize */}
              {(formData.type === "update" || formData.type === "optimize") && (
                <div className="flex items-center justify-center py-8 text-muted-foreground">
                  <p>此任务类型无需额外配置</p>
                </div>
              )}
            </TabsContent>
          </Tabs>

          <DialogFooter className="mt-6">
            <Button variant="outline" onClick={() => { setIsDialogOpen(false); resetForm() }}>
              取消
            </Button>
            <Button onClick={handleSaveTask}>
              {editingTask ? "保存更改" : "创建任务"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
