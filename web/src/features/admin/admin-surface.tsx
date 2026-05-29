"use client"

import { useEffect, useState } from "react"
import type { ComponentType } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  Server,
  HardDrive,
  Users,
  Activity,
  Database,
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  Clock,
  Folder,
  Settings,
  Puzzle,
  Shield,
  Cpu,
  Play,
  ChevronRight,
  FileWarning,
  Wrench,
  Network,
  Bell,
  Palette,
  Globe,
  Download,
  Trash2,
  RotateCcw,
  ArrowUpRight,
  Info,
  ExternalLink,
  Monitor,
  FileSearch,
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Switch } from "@/components/ui/switch"
import { Separator } from "@/components/ui/separator"
import { Slider } from "@/components/ui/slider"
import { Textarea } from "@/components/ui/textarea"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { cn } from "@/lib/utils"
import { AdminLibraries } from "./admin-libraries"
import { AdminUsers } from "./admin-users"
import { AdminPlugins } from "./admin-plugins"
import { AdminLogs } from "./admin-logs"
import type { AdminLogsRouteState } from "./admin-logs"
import { AdminScheduledTasks } from "./admin-scheduled-tasks"
import { AdminSettings } from "./admin-settings"
import { AdminAcquisitionIntake, type AdminAcquisitionIntakeRouteState } from "./admin-acquisition-intake"
import {
  ADMIN_DASHBOARD_FIXTURE,
  createAdminDashboardDataSource,
  type AdminDashboardData,
  type AdminDashboardMetrics,
  type AdminDashboardPlaybackSession,
  type AdminDashboardTask,
} from "@/src/api/admin/dashboard-data-source"

export type AdminSurfaceSection =
  | "dashboard"
  | "activity"
  | "scheduled-tasks"
  | "acquisition-intake"
  | "libraries"
  | "users"
  | "dlna"
  | "remote-access"
  | "transcoding"
  | "network"
  | "plugins"
  | "notifications"
  | "backup"
  | "advanced"
  | "about"

export interface AdminSurfaceProps {
  activeSection?: AdminSurfaceSection
  onSectionNavigate?: (section: AdminSurfaceSection) => void
  adminLogsState?: AdminLogsRouteState
  onAdminLogsStateChange?: (state: AdminLogsRouteState) => void
  acquisitionIntakeState?: AdminAcquisitionIntakeRouteState
  onAcquisitionIntakeStateChange?: (state: AdminAcquisitionIntakeRouteState) => void
}

interface AdminNavItem {
  name: string
  icon: ComponentType<{ className?: string }>
  component: AdminSurfaceSection
}

interface AdminNavGroup {
  title: string
  items: AdminNavItem[]
}

// 自托管媒体服务器管理面板导航结构
const adminNavGroups: AdminNavGroup[] = [
  {
    title: "服务器",
    items: [
      { name: "仪表盘", icon: Server, component: "dashboard" },
      { name: "活动日志", icon: Activity, component: "activity" },
      { name: "计划任务", icon: Clock, component: "scheduled-tasks" },
    ]
  },
  {
    title: "内容",
    items: [
      { name: "媒体库", icon: Folder, component: "libraries" },
      { name: "采集入口", icon: FileSearch, component: "acquisition-intake" },
    ]
  },
  {
    title: "访问",
    items: [
      { name: "用户管理", icon: Users, component: "users" },
      { name: "DLNA/UPnP", icon: Globe, component: "dlna" },
      { name: "远程访问", icon: Network, component: "remote-access" },
    ]
  },
  {
    title: "系统",
    items: [
      { name: "转码设置", icon: RotateCcw, component: "transcoding" },
      { name: "网络设置", icon: Wrench, component: "network" },
      { name: "插件", icon: Puzzle, component: "plugins" },
      { name: "通知", icon: Bell, component: "notifications" },
    ]
  },
  {
    title: "维护",
    items: [
      { name: "备份与恢复", icon: Download, component: "backup" },
      { name: "高级设置", icon: Settings, component: "advanced" },
      { name: "关于", icon: Info, component: "about" },
    ]
  }
]

// Recent activity log
const activityLog = [
  { time: "刚刚", event: "媒体库扫描完成", detail: "动画 · 新增 12 项", type: "success", icon: CheckCircle2 },
  { time: "5 分钟前", event: "用户登录", detail: "张明 · Apple TV 4K", type: "info", icon: Users },
  { time: "12 分钟前", event: "元数据更新", detail: "沙丘2 · TMDb Provider", type: "info", icon: Database },
  { time: "1 小时前", event: "NFO 导入", detail: "电影库 · 3 个文件", type: "success", icon: Download },
  { time: "2 小时前", event: "自动扫描触发", detail: "检测到文件系统变更", type: "info", icon: RefreshCw },
  { time: "3 小时前", event: "插件更新", detail: "TMDb 插件更新至 v2.1.0", type: "success", icon: Puzzle },
]

// Scheduled tasks
const scheduledTasks = [
  { name: "媒体库扫描", schedule: "每日 03:00", lastRun: "今天 03:00", nextRun: "明天 03:00", status: "idle" },
  { name: "元数据刷新", schedule: "每周日 04:00", lastRun: "上周日", nextRun: "本周日 04:00", status: "idle" },
  { name: "图片优化", schedule: "每日 05:00", lastRun: "今天 05:00", nextRun: "明天 05:00", status: "idle" },
  { name: "日志清理", schedule: "每月 1 号", lastRun: "2024-01-01", nextRun: "2024-02-01", status: "idle" },
  { name: "缓存清理", schedule: "每周一 02:00", lastRun: "本周一", nextRun: "下周一 02:00", status: "idle" },
]

export function AdminSurface({
  activeSection = "dashboard",
  onSectionNavigate,
  adminLogsState,
  onAdminLogsStateChange,
  acquisitionIntakeState,
  onAcquisitionIntakeStateChange,
}: AdminSurfaceProps = {}) {
  const [activeComponent, setActiveComponent] = useState<AdminSurfaceSection>(activeSection)
  const { data: dashboardData = ADMIN_DASHBOARD_FIXTURE } = useQuery({
    queryKey: ["nako", "admin", "dashboard"],
    queryFn: () => createAdminDashboardDataSource().loadDashboard(),
    staleTime: 30 * 1000,
    retry: 0,
  })
  const serverMetrics = dashboardData.metrics

  useEffect(() => {
    setActiveComponent(activeSection)
  }, [activeSection])

  const navigateToSection = (section: AdminSurfaceSection) => {
    if (onSectionNavigate) {
      onSectionNavigate(section)
      return
    }

    setActiveComponent(section)
  }
  // Render the appropriate component based on navigation
  const renderContent = () => {
    switch (activeComponent) {
      case "libraries":
        return <AdminLibraries />
      case "users":
        return <AdminUsers />
      case "plugins":
        return <AdminPlugins />
      case "activity":
        return <AdminLogs routeState={adminLogsState} onRouteStateChange={onAdminLogsStateChange} />
      case "acquisition-intake":
        return (
          <AdminAcquisitionIntake
            routeState={acquisitionIntakeState}
            onRouteStateChange={onAcquisitionIntakeStateChange}
          />
        )
      case "scheduled-tasks":
        return <AdminScheduledTasks />
      case "dlna":
        return <DLNASettingsPage />
      case "remote-access":
        return <RemoteAccessPage />
      case "transcoding":
        return <TranscodingSettingsPage />
      case "network":
        return <NetworkSettingsPage />
      case "notifications":
        return <NotificationsPage />
      case "backup":
        return <BackupPage />
      case "advanced":
        return <AdminSettings />
      case "about":
        return <AboutPage metrics={serverMetrics} />
      default:
        return <AdminDashboard data={dashboardData} />
    }
  }

  return (
    <div className="flex h-[calc(100vh-3.5rem)]">
      {/* Admin Sidebar - Fixed with scrollable nav */}
      <aside className="hidden w-56 flex-shrink-0 border-r border-border/50 bg-sidebar lg:flex lg:flex-col">
        {/* Navigation Area - only shows scrollbar when needed */}
        <nav className="flex-1 space-y-4 overflow-y-auto scrollbar-none p-4">
          {adminNavGroups.map((group, groupIndex) => (
            <div key={group.title}>
              <h3 className="mb-1.5 px-2 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
                {group.title}
              </h3>
              <div className="space-y-0.5">
                {group.items.map((item) => (
                  <button
                    key={item.name}
                    onClick={() => navigateToSection(item.component)}
                    className={cn(
                      "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
                      activeComponent === item.component
                        ? "bg-sidebar-accent text-sidebar-accent-foreground"
                        : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50"
                    )}
                  >
                    <item.icon className="h-4 w-4" />
                    <span>{item.name}</span>
                  </button>
                ))}
              </div>
            </div>
          ))}
        </nav>

        {/* Fixed Server Status - Always visible at bottom */}
        <div className="flex-shrink-0 border-t border-border/50 p-3">
          <div className="flex items-center gap-2">
            <div className="flex h-5 w-5 items-center justify-center rounded-full bg-success/10">
              <CheckCircle2 className="h-3 w-3 text-success" />
            </div>
            <span className="text-xs font-medium text-foreground">服务器正常</span>
          </div>
          <div className="mt-1.5 space-y-0.5 text-[11px] text-muted-foreground">
            <div className="flex justify-between">
              <span>运行时间</span>
              <span className="text-foreground">{serverMetrics.uptime}</span>
            </div>
            <div className="flex justify-between">
              <span>版本</span>
              <span className="font-mono text-foreground">{serverMetrics.version}</span>
            </div>
          </div>
        </div>
      </aside>

      {/* Main Content - Scrollable */}
      <main className="flex-1 overflow-y-auto scrollbar-none">
        <div className="p-6 lg:p-8">
          {renderContent()}
        </div>
      </main>
    </div>
  )
}

// Admin Dashboard Component
function AdminDashboard({ data }: { data: AdminDashboardData }) {
  const { activeTasks, metrics: serverMetrics, playbackSessions } = data

  return (
    <>
      {/* Page Header */}
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-foreground">仪表盘</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          服务器状态总览
        </p>
      </div>

      {/* Version Update Notice */}
      {serverMetrics.hasUpdate && (
        <div className="mb-6 rounded-xl border border-info/30 bg-info/5 p-4">
          <div className="flex items-start justify-between gap-4">
            <div className="flex items-start gap-3">
              <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-info/10">
                <ArrowUpRight className="h-4 w-4 text-info" />
              </div>
              <div>
                <h3 className="font-medium text-foreground">新版本可用</h3>
                <p className="mt-0.5 text-sm text-muted-foreground">
                  Nako <span className="font-mono text-info">{serverMetrics.latestVersion}</span> 已发布，
                  当前版本 <span className="font-mono">{serverMetrics.version}</span>
                </p>
                <div className="mt-2 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
                  <span>主要更新: 性能优化、新增 AI 助手功能、修复已知问题</span>
                </div>
              </div>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <Button variant="ghost" size="sm" className="gap-1.5 text-xs">
                <ExternalLink className="h-3.5 w-3.5" />
                更新日志
              </Button>
              <Button variant="outline" size="sm" className="gap-1.5 text-xs">
                <Info className="h-3.5 w-3.5" />
                查看更新指南
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Server Health Grid */}
      <div className="mb-8 grid grid-cols-2 gap-4 lg:grid-cols-4">
        <MetricCard
              icon={Cpu}
              label="CPU 使用率"
          value={`${serverMetrics.cpu}%`}
          progress={serverMetrics.cpu}
          status="healthy"
        />
        <MetricCard
          icon={Activity}
          label="内存使用"
          value={`${serverMetrics.memory}%`}
          progress={serverMetrics.memory}
          status="healthy"
        />
        <MetricCard
          icon={HardDrive}
          label="存储空间"
          value={`${serverMetrics.storage}%`}
          progress={serverMetrics.storage}
          status="warning"
          detail="4.2 TB / 6 TB"
        />
        <MetricCard
          icon={Play}
          label="活跃播放"
          value={serverMetrics.activeStreams.toString()}
          detail={`${serverMetrics.totalItems} 媒体项`}
        />
      </div>

      {/* Active Streams */}
      {playbackSessions.length > 0 && (
        <section className="mb-6 rounded-lg border border-border/50 bg-card">
          <div className="flex items-center justify-between border-b border-border/50 p-4">
            <h2 className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Play className="h-4 w-4 text-primary" />
              当前播放
              <Badge variant="default" className="ml-1 text-[10px]">
                {playbackSessions.length}
              </Badge>
            </h2>
          </div>
          <div className="divide-y divide-border/50">
            {playbackSessions.map((session) => (
              <PlaybackSessionItem key={session.id} session={session} />
            ))}
          </div>
        </section>
      )}

      {/* Two Column Layout */}
      <div className="grid gap-6 lg:grid-cols-2">
        {/* Active Tasks */}
        <section className="rounded-lg border border-border/50 bg-card">
          <div className="flex items-center justify-between border-b border-border/50 p-4">
            <h2 className="flex items-center gap-2 text-sm font-medium text-foreground">
              <RefreshCw className="h-4 w-4 text-muted-foreground" />
              任务队列
            </h2>
            <Button variant="ghost" size="sm" className="h-7 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground">
              查看全部 <ChevronRight className="ml-1 h-3 w-3" />
            </Button>
          </div>
          <div className="divide-y divide-border/50">
            {activeTasks.slice(0, 3).map((task) => (
              <TaskItem key={task.id} task={task} />
            ))}
          </div>
        </section>

        {/* Recent Activity */}
        <section className="rounded-lg border border-border/50 bg-card">
          <div className="flex items-center justify-between border-b border-border/50 p-4">
            <h2 className="flex items-center gap-2 text-sm font-medium text-foreground">
              <Activity className="h-4 w-4 text-muted-foreground" />
              最近活动
            </h2>
            <Button variant="ghost" size="sm" className="h-7 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground">
              查看全部 <ChevronRight className="ml-1 h-3 w-3" />
            </Button>
          </div>
          <div className="divide-y divide-border/50">
            {activityLog.slice(0, 5).map((log, i) => (
              <div key={i} className="flex items-center gap-3 px-4 py-3">
                <div className={cn(
                  "flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full",
                  log.type === "success" ? "bg-success/10" : "bg-muted"
                )}>
                  <log.icon className={cn(
                    "h-3.5 w-3.5",
                    log.type === "success" ? "text-success" : "text-muted-foreground"
                  )} />
                </div>
                <div className="min-w-0 flex-1">
                  <div className="text-sm text-foreground">{log.event}</div>
                  <div className="text-xs text-muted-foreground">{log.detail}</div>
                </div>
                <span className="flex-shrink-0 text-xs text-muted-foreground">{log.time}</span>
              </div>
            ))}
          </div>
        </section>
      </div>

      {/* Quick Actions */}
      <section className="mt-6">
        <h2 className="mb-4 text-sm font-medium text-foreground">快捷操作</h2>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="mr-2 h-3.5 w-3.5" />
            扫描所有媒体库
          </Button>
          <Button variant="outline" size="sm">
            <Database className="mr-2 h-3.5 w-3.5" />
            刷新元数据
          </Button>
          <Button variant="outline" size="sm">
            <Trash2 className="mr-2 h-3.5 w-3.5" />
            清理缓存
          </Button>
          <Button variant="outline" size="sm">
            <RotateCcw className="mr-2 h-3.5 w-3.5" />
            重启服务器
          </Button>
        </div>
      </section>
    </>
  )
}

// Activity Log Page
function ActivityLogPage() {
  return (
    <>
      <div className="mb-6">
        <h1 className="text-xl font-semibold text-foreground">活动日志</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          查看所有服务器活动记录
        </p>
      </div>

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex items-center justify-between border-b border-border/50 p-4">
          <div className="flex items-center gap-2">
            <Button variant="outline" size="sm">全部</Button>
            <Button variant="ghost" size="sm">用户</Button>
            <Button variant="ghost" size="sm">系统</Button>
            <Button variant="ghost" size="sm">播放</Button>
          </div>
          <Button variant="ghost" size="sm" className="text-xs text-muted-foreground hover:bg-transparent hover:text-foreground">
            清除日志
          </Button>
        </div>
        <div className="divide-y divide-border/50">
          {activityLog.map((log, i) => (
            <div key={i} className="flex items-center gap-4 px-4 py-3">
              <div className={cn(
                "flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-full",
                log.type === "success" ? "bg-success/10" : "bg-muted"
              )}>
                <log.icon className={cn(
                  "h-4 w-4",
                  log.type === "success" ? "text-success" : "text-muted-foreground"
                )} />
              </div>
              <div className="min-w-0 flex-1">
                <div className="text-sm font-medium text-foreground">{log.event}</div>
                <div className="text-xs text-muted-foreground">{log.detail}</div>
              </div>
              <span className="flex-shrink-0 text-xs text-muted-foreground">{log.time}</span>
            </div>
          ))}
        </div>
      </section>
    </>
  )
}

// Scheduled Tasks Page
function ScheduledTasksPage() {
  return (
    <>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold text-foreground">计划任务</h1>
          <p className="mt-1 text-sm text-muted-foreground">
            管理自动执行的后台任务
          </p>
        </div>
        <Button size="sm">
          <Clock className="mr-2 h-3.5 w-3.5" />
          添加任务
        </Button>
      </div>

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="divide-y divide-border/50">
          {scheduledTasks.map((task, i) => (
            <div key={i} className="flex items-center justify-between p-4">
              <div className="flex items-center gap-4">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
                  <Clock className="h-5 w-5 text-muted-foreground" />
                </div>
                <div>
                  <div className="text-sm font-medium text-foreground">{task.name}</div>
                  <div className="text-xs text-muted-foreground">{task.schedule}</div>
                </div>
              </div>
              <div className="flex items-center gap-6">
                <div className="text-right text-xs">
                  <div className="text-muted-foreground">上次运行</div>
                  <div className="text-foreground">{task.lastRun}</div>
                </div>
                <div className="text-right text-xs">
                  <div className="text-muted-foreground">下次运行</div>
                  <div className="text-foreground">{task.nextRun}</div>
                </div>
                <Button variant="outline" size="sm">立即运行</Button>
              </div>
            </div>
          ))}
        </div>
      </section>
    </>
  )
}

function MetricCard({
  icon: Icon,
  label,
  value,
  progress,
  status,
  detail,
}: {
  icon: typeof Server
  label: string
  value: string
  progress?: number
  status?: "healthy" | "warning" | "critical"
  detail?: string
}) {
  return (
    <div className="rounded-lg border border-border/50 bg-card p-4">
      <div className="flex items-center justify-between">
        <Icon className="h-4 w-4 text-muted-foreground" />
        {status && (
          <div
            className={cn(
              "h-2 w-2 rounded-full",
              status === "healthy" && "bg-success",
              status === "warning" && "bg-warning",
              status === "critical" && "bg-destructive"
            )}
          />
        )}
      </div>
      <div className="mt-3">
        <div className="text-2xl font-semibold text-foreground">{value}</div>
        <div className="text-xs text-muted-foreground">{label}</div>
      </div>
      {progress !== undefined && (
        <Progress value={progress} className="mt-3 h-1" />
      )}
      {detail && (
        <div className="mt-2 text-xs text-muted-foreground">{detail}</div>
      )}
    </div>
  )
}

function TaskItem({ task }: { task: AdminDashboardTask }) {
  return (
    <div className="p-4">
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-foreground">{task.name}</span>
            <Badge
              variant={task.status === "running" ? "default" : "secondary"}
              className="text-[10px]"
            >
              {task.status === "running" ? "运行中" : "排队中"}
            </Badge>
          </div>
          <div className="mt-1 text-xs text-muted-foreground">
            {"library" in task && <span>{task.library} 库</span>}
            {"item" in task && <span>{task.item}</span>}
            {"provider" in task && <span> · {task.provider}</span>}
            {"profile" in task && <span> · {task.profile}</span>}
          </div>
        </div>
        {"progress" in task && task.progress !== undefined && (
          <span className="flex-shrink-0 text-sm font-medium text-primary">{task.progress}%</span>
        )}
      </div>
      {"progress" in task && task.progress !== undefined && (
        <Progress value={task.progress} className="mt-3 h-1" />
      )}
    </div>
  )
}

function PlaybackSessionItem({ session }: { session: AdminDashboardPlaybackSession }) {
  return (
    <div className="p-4">
      <div className="flex items-center gap-4">
        <div className="flex h-12 w-12 flex-shrink-0 items-center justify-center rounded-full bg-primary/10 text-lg font-medium text-primary">
          {session.user.charAt(0)}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="text-sm font-medium text-foreground">{session.user}</span>
            <span className="text-xs text-muted-foreground">正在观看</span>
          </div>
          <div className="mt-0.5 text-sm text-foreground">
            {session.item}
            {"episode" in session && <span className="text-muted-foreground"> · {session.episode}</span>}
          </div>
          <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
            <span>{session.device}</span>
            <span>·</span>
            <span>{session.client}</span>
            <span>·</span>
            <Badge
              variant={session.playbackMethod === "Direct Play" ? "secondary" : "outline"}
              className="text-[10px]"
            >
              {session.playbackMethod}
            </Badge>
            <span>·</span>
            <span>{session.quality}</span>
          </div>
        </div>
        <div className="flex flex-col items-end gap-1">
          <span className="text-sm font-medium text-foreground">{session.progress}%</span>
          <span className="text-xs text-muted-foreground">{session.bandwidth}</span>
        </div>
      </div>
      <Progress value={session.progress} className="mt-3 h-1" />
    </div>
  )
}

// DLNA/UPnP 设置页面
function DLNASettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">DLNA/UPnP</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置本地网络设备发现和媒体共享</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            DLNA 服务器
          </CardTitle>
          <CardDescription>允许支持 DLNA 的设备（如智能电视）发现并播放媒体</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>启用 DLNA 服务器</Label>
              <p className="text-xs text-muted-foreground">在本地网络广播媒体服务</p>
            </div>
            <Switch defaultChecked />
          </div>
          <Separator />
          <div className="space-y-2">
            <Label>服务器名称</Label>
            <Input defaultValue="Nako Media Server" />
          </div>
          <div className="space-y-2">
            <Label>共享的媒体库</Label>
            <div className="space-y-2">
              {["电影", "剧集", "音乐", "照片"].map(lib => (
                <div key={lib} className="flex items-center justify-between rounded-lg border border-border/50 p-3">
                  <span className="text-sm">{lib}</span>
                  <Switch defaultChecked={lib !== "照片"} />
                </div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>已发现的设备</CardTitle>
          <CardDescription>当前网络中支持 DLNA 的设备</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {[
              { name: "客厅电视", type: "Samsung Smart TV", ip: "192.168.1.101" },
              { name: "卧室电视", type: "LG webOS TV", ip: "192.168.1.102" },
            ].map(device => (
              <div key={device.ip} className="flex items-center justify-between rounded-lg border border-border/50 p-3">
                <div>
                  <p className="text-sm font-medium">{device.name}</p>
                  <p className="text-xs text-muted-foreground">{device.type} · {device.ip}</p>
                </div>
                <Badge variant="outline" className="text-success border-success/30">在线</Badge>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 远程访问页面
function RemoteAccessPage() {
  const [httpsEnabled, setHttpsEnabled] = useState(false)
  const [ddnsEnabled, setDdnsEnabled] = useState(false)
  const [ddnsProvider, setDdnsProvider] = useState("cloudflare")
  
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">远程访问</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置外部网络访问和安全设置</p>
      </div>

      {/* 连接状态 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="h-5 w-5" />
            连接状态
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 sm:grid-cols-3">
            <div className="flex items-center gap-3 rounded-lg border border-green-500/30 bg-green-500/5 p-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
              </div>
              <div>
                <p className="text-sm font-medium">本地访问</p>
                <p className="text-xs text-muted-foreground">192.168.1.100:8096</p>
              </div>
            </div>
            <div className="flex items-center gap-3 rounded-lg border border-green-500/30 bg-green-500/5 p-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10">
                <CheckCircle2 className="h-5 w-5 text-green-500" />
              </div>
              <div>
                <p className="text-sm font-medium">外部访问</p>
                <p className="text-xs text-muted-foreground">media.example.com</p>
              </div>
            </div>
            <div className="flex items-center gap-3 rounded-lg border border-yellow-500/30 bg-yellow-500/5 p-3">
              <div className="flex h-10 w-10 items-center justify-center rounded-full bg-yellow-500/10">
                <AlertTriangle className="h-5 w-5 text-yellow-500" />
              </div>
              <div>
                <p className="text-sm font-medium">UPnP/NAT-PMP</p>
                <p className="text-xs text-muted-foreground">未检测到</p>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
      
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Network className="h-5 w-5" />
            外部访问
          </CardTitle>
          <CardDescription>配置从外部网络访问服务器的方式</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>允许远程连接</Label>
              <p className="text-xs text-muted-foreground">允许从外部网络访问服务器</p>
            </div>
            <Switch defaultChecked />
          </div>
          <Separator />
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label>内部 HTTP 端口</Label>
              <Input defaultValue="8096" />
            </div>
            <div className="space-y-2">
              <Label>外部 HTTP 端口</Label>
              <Input defaultValue="8096" />
              <p className="text-xs text-muted-foreground">路由器端口映射的外部端口</p>
            </div>
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label>内部 HTTPS 端口</Label>
              <Input defaultValue="8920" />
            </div>
            <div className="space-y-2">
              <Label>外部 HTTPS 端口</Label>
              <Input defaultValue="8920" />
            </div>
          </div>
          <Separator />
          <div className="space-y-2">
            <Label>外部域名</Label>
            <Input placeholder="media.example.com" defaultValue="media.example.com" />
            <p className="text-xs text-muted-foreground">用于生成外部分享链接和访问地址</p>
          </div>
          <div className="space-y-2">
            <Label>已知代理</Label>
            <Input placeholder="10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16" />
            <p className="text-xs text-muted-foreground">反向代理服务器的 IP 地址或 CIDR，用于正确识别客户端 IP</p>
          </div>
        </CardContent>
      </Card>

      {/* DDNS 设置 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Globe className="h-5 w-5" />
                动态 DNS (DDNS)
              </CardTitle>
              <CardDescription>自动更新动态 IP 地址到域名</CardDescription>
            </div>
            <Switch checked={ddnsEnabled} onCheckedChange={setDdnsEnabled} />
          </div>
        </CardHeader>
        {ddnsEnabled && (
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>DDNS 服务商</Label>
              <Select value={ddnsProvider} onValueChange={setDdnsProvider}>
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="cloudflare">Cloudflare</SelectItem>
                  <SelectItem value="aliyun">阿里云 DNS</SelectItem>
                  <SelectItem value="dnspod">DNSPod (腾讯云)</SelectItem>
                  <SelectItem value="duckdns">DuckDNS</SelectItem>
                  <SelectItem value="noip">No-IP</SelectItem>
                  <SelectItem value="custom">自定义</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            {ddnsProvider === "cloudflare" && (
              <>
                <div className="space-y-2">
                  <Label>Zone ID</Label>
                  <Input placeholder="输入 Cloudflare Zone ID" />
                </div>
                <div className="space-y-2">
                  <Label>API Token</Label>
                  <Input type="password" placeholder="输入 API Token" />
                </div>
                <div className="space-y-2">
                  <Label>记录名称</Label>
                  <Input placeholder="media.example.com" />
                </div>
              </>
            )}
            
            {ddnsProvider === "aliyun" && (
              <>
                <div className="space-y-2">
                  <Label>AccessKey ID</Label>
                  <Input placeholder="输入 AccessKey ID" />
                </div>
                <div className="space-y-2">
                  <Label>AccessKey Secret</Label>
                  <Input type="password" placeholder="输入 AccessKey Secret" />
                </div>
                <div className="space-y-2">
                  <Label>域名</Label>
                  <Input placeholder="example.com" />
                </div>
                <div className="space-y-2">
                  <Label>主机记录</Label>
                  <Input placeholder="media" />
                </div>
              </>
            )}

            {ddnsProvider === "dnspod" && (
              <>
                <div className="space-y-2">
                  <Label>SecretId</Label>
                  <Input placeholder="输入 SecretId" />
                </div>
                <div className="space-y-2">
                  <Label>SecretKey</Label>
                  <Input type="password" placeholder="输入 SecretKey" />
                </div>
                <div className="space-y-2">
                  <Label>域名</Label>
                  <Input placeholder="example.com" />
                </div>
                <div className="space-y-2">
                  <Label>子域名</Label>
                  <Input placeholder="media" />
                </div>
              </>
            )}

            {ddnsProvider === "custom" && (
              <>
                <div className="space-y-2">
                  <Label>更新 URL</Label>
                  <Input placeholder="https://ddns.example.com/update?ip={ip}" />
                  <p className="text-xs text-muted-foreground">{"使用 {ip} 作为 IP 地址占位符"}</p>
                </div>
                <div className="space-y-2">
                  <Label>HTTP 方法</Label>
                  <Select defaultValue="GET">
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="GET">GET</SelectItem>
                      <SelectItem value="POST">POST</SelectItem>
                      <SelectItem value="PUT">PUT</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </>
            )}
            
            <div className="space-y-2">
              <Label>更新间隔</Label>
              <Select defaultValue="5">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="1">1 分钟</SelectItem>
                  <SelectItem value="5">5 分钟</SelectItem>
                  <SelectItem value="10">10 分钟</SelectItem>
                  <SelectItem value="30">30 分钟</SelectItem>
                  <SelectItem value="60">1 小时</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="flex items-center gap-2 rounded-lg border border-green-500/30 bg-green-500/5 p-3">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <span className="text-sm">上次更新: 2 分钟前 (123.45.67.89)</span>
            </div>
          </CardContent>
        )}
      </Card>

      {/* SSL/TLS 证书 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="flex items-center gap-2">
                <Shield className="h-5 w-5" />
                SSL/TLS 证书
              </CardTitle>
              <CardDescription>配置 HTTPS 加密连接</CardDescription>
            </div>
            <Switch checked={httpsEnabled} onCheckedChange={setHttpsEnabled} />
          </div>
        </CardHeader>
        {httpsEnabled && (
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>证书来源</Label>
              <Select defaultValue="acme">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="acme">自动申请 (Let&apos;s Encrypt)</SelectItem>
                  <SelectItem value="custom">自定义证书</SelectItem>
                  <SelectItem value="selfsigned">自签名证书</SelectItem>
                </SelectContent>
              </Select>
            </div>
            
            <div className="rounded-lg border border-border/50 bg-muted/30 p-4 space-y-3">
              <div className="flex items-center justify-between">
                <span className="text-sm">证书状态</span>
                <Badge variant="default" className="bg-green-500">有效</Badge>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">域名</span>
                <span>media.example.com</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">颁发者</span>
                <span>Let&apos;s Encrypt</span>
              </div>
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">到期时间</span>
                <span>2024-06-15 (剩余 87 天)</span>
              </div>
            </div>
            
            <div className="flex items-center justify-between">
              <div>
                <Label>强制 HTTPS</Label>
                <p className="text-xs text-muted-foreground">将 HTTP 请求重定向到 HTTPS</p>
              </div>
              <Switch defaultChecked />
            </div>
            
            <div className="flex items-center justify-between">
              <div>
                <Label>HSTS</Label>
                <p className="text-xs text-muted-foreground">启用 HTTP 严格传输安全</p>
              </div>
              <Switch />
            </div>
          </CardContent>
        )}
      </Card>

      {/* 访问控制 */}
      <Card>
        <CardHeader>
          <CardTitle>访问控制</CardTitle>
          <CardDescription>限制可访问服务器的 IP 地址</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>强制登录</Label>
              <p className="text-xs text-muted-foreground">所有访问都需要用户认证</p>
            </div>
            <Switch defaultChecked />
          </div>
          
          <Separator />
          
          <div className="space-y-2">
            <Label>IP 白名单</Label>
            <Textarea 
              placeholder="每行一个 IP 或 CIDR，例如：&#10;192.168.1.0/24&#10;10.0.0.1"
              className="min-h-[80px] font-mono text-sm"
            />
            <p className="text-xs text-muted-foreground">留空表示允许所有 IP 访问</p>
          </div>
          
          <div className="space-y-2">
            <Label>IP 黑名单</Label>
            <Textarea 
              placeholder="每行一个 IP 或 CIDR"
              className="min-h-[80px] font-mono text-sm"
            />
          </div>
          
          <div className="space-y-2">
            <Label>登录失败锁定</Label>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">失败次数</Label>
                <Select defaultValue="5">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="3">3 次</SelectItem>
                    <SelectItem value="5">5 次</SelectItem>
                    <SelectItem value="10">10 次</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label className="text-xs text-muted-foreground">锁定时长</Label>
                <Select defaultValue="30">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="5">5 分钟</SelectItem>
                    <SelectItem value="15">15 分钟</SelectItem>
                    <SelectItem value="30">30 分钟</SelectItem>
                    <SelectItem value="60">1 小时</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 转码设置页面
function TranscodingSettingsPage() {
  const [hwAccelType, setHwAccelType] = useState("vaapi")
  
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">转码设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置视频转码和硬件加速</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RotateCcw className="h-5 w-5" />
            硬件加速
          </CardTitle>
          <CardDescription>使用 GPU 加速视频转码以减少 CPU 负载</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>硬件加速类型</Label>
            <Select value={hwAccelType} onValueChange={setHwAccelType}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">无 (仅 CPU)</SelectItem>
                <SelectItem value="vaapi">VA-API (Linux)</SelectItem>
                <SelectItem value="nvenc">NVENC (NVIDIA)</SelectItem>
                <SelectItem value="qsv">QuickSync (Intel)</SelectItem>
                <SelectItem value="videotoolbox">VideoToolbox (macOS)</SelectItem>
                <SelectItem value="amf">AMF (AMD)</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* 针对不同硬件类型的配置 */}
          {hwAccelType === "none" && (
            <div className="rounded-lg border border-border/50 bg-muted/30 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Info className="h-5 w-5 text-muted-foreground mt-0.5" />
                <div>
                  <p className="text-sm font-medium">软件转码模式</p>
                  <p className="text-xs text-muted-foreground">
                    使用 CPU 进行转码，兼容性最好但性能较低
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>编码线程数</Label>
                <Select defaultValue="auto">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">自动</SelectItem>
                    <SelectItem value="2">2 线程</SelectItem>
                    <SelectItem value="4">4 线程</SelectItem>
                    <SelectItem value="8">8 线程</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>编码预设</Label>
                <Select defaultValue="medium">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="ultrafast">极快</SelectItem>
                    <SelectItem value="veryfast">很快</SelectItem>
                    <SelectItem value="medium">中等</SelectItem>
                    <SelectItem value="slow">慢速</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}

          {hwAccelType === "vaapi" && (
            <div className="rounded-lg border border-orange-500/20 bg-orange-500/5 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Monitor className="h-5 w-5 text-orange-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">VA-API (Video Acceleration API)</p>
                  <p className="text-xs text-muted-foreground">
                    Linux 通用硬件加速接口，支持 Intel/AMD 显卡
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>渲染设备</Label>
                <Select defaultValue="/dev/dri/renderD128">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="/dev/dri/renderD128">/dev/dri/renderD128</SelectItem>
                    <SelectItem value="/dev/dri/renderD129">/dev/dri/renderD129</SelectItem>
                    <SelectItem value="/dev/dri/card0">/dev/dri/card0</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>VA-API 驱动</Label>
                <Select defaultValue="auto">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">自动检测</SelectItem>
                    <SelectItem value="iHD">iHD (Intel 推荐)</SelectItem>
                    <SelectItem value="i965">i965 (旧版 Intel)</SelectItem>
                    <SelectItem value="radeonsi">RadeonSI (AMD)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}

          {hwAccelType === "nvenc" && (
            <div className="rounded-lg border border-green-500/20 bg-green-500/5 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Monitor className="h-5 w-5 text-green-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">NVIDIA NVENC</p>
                  <p className="text-xs text-muted-foreground">
                    适用于 NVIDIA GTX 10 系列及以上显卡
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>CUDA 设备</Label>
                <Select defaultValue="0">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="0">GPU 0 (默认)</SelectItem>
                    <SelectItem value="1">GPU 1</SelectItem>
                    <SelectItem value="2">GPU 2</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>NVENC 预设</Label>
                <Select defaultValue="p4">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="p1">P1 (最快)</SelectItem>
                    <SelectItem value="p4">P4 (平衡)</SelectItem>
                    <SelectItem value="p7">P7 (最高质量)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>码率控制</Label>
                <Select defaultValue="vbr">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="cqp">CQP (恒定 QP)</SelectItem>
                    <SelectItem value="vbr">VBR (可变码率)</SelectItem>
                    <SelectItem value="cbr">CBR (恒定码率)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <Label>B 帧支持</Label>
                  <p className="text-xs text-muted-foreground">提高压缩效率</p>
                </div>
                <Switch defaultChecked />
              </div>
            </div>
          )}

          {hwAccelType === "qsv" && (
            <div className="rounded-lg border border-blue-500/20 bg-blue-500/5 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Monitor className="h-5 w-5 text-blue-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">Intel Quick Sync Video</p>
                  <p className="text-xs text-muted-foreground">
                    适用于 Intel 第6代及以上处理器的核显
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>QSV 设备</Label>
                <Select defaultValue="auto">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="auto">自动检测</SelectItem>
                    <SelectItem value="/dev/dri/renderD128">/dev/dri/renderD128</SelectItem>
                    <SelectItem value="/dev/dri/renderD129">/dev/dri/renderD129</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>编码模式</Label>
                <Select defaultValue="la_icq">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="icq">ICQ (恒定质量)</SelectItem>
                    <SelectItem value="la_icq">LA-ICQ (前瞻恒定质量)</SelectItem>
                    <SelectItem value="vbr">VBR (可变码率)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <Label>低功耗模式</Label>
                  <p className="text-xs text-muted-foreground">降低 GPU 占用</p>
                </div>
                <Switch />
              </div>
            </div>
          )}

          {hwAccelType === "videotoolbox" && (
            <div className="rounded-lg border border-purple-500/20 bg-purple-500/5 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Monitor className="h-5 w-5 text-purple-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">Apple VideoToolbox</p>
                  <p className="text-xs text-muted-foreground">
                    macOS 原生硬件加速，支持 Apple Silicon 和 Intel Mac
                  </p>
                </div>
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <Label>实时编码</Label>
                  <p className="text-xs text-muted-foreground">优化延迟</p>
                </div>
                <Switch />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <Label>允许软件回退</Label>
                  <p className="text-xs text-muted-foreground">硬件不支持时使用软件编码</p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="space-y-2">
                <Label>配置文件</Label>
                <Select defaultValue="main">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="baseline">Baseline (兼容性最好)</SelectItem>
                    <SelectItem value="main">Main (推荐)</SelectItem>
                    <SelectItem value="high">High (质量最高)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}

          {hwAccelType === "amf" && (
            <div className="rounded-lg border border-red-500/20 bg-red-500/5 p-4 space-y-4">
              <div className="flex items-start gap-3">
                <Monitor className="h-5 w-5 text-red-500 mt-0.5" />
                <div>
                  <p className="text-sm font-medium">AMD Advanced Media Framework</p>
                  <p className="text-xs text-muted-foreground">
                    适用于 AMD RX 400 系列及以上显卡
                  </p>
                </div>
              </div>
              <div className="space-y-2">
                <Label>编码质量</Label>
                <Select defaultValue="balanced">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="speed">速度优先</SelectItem>
                    <SelectItem value="balanced">平衡</SelectItem>
                    <SelectItem value="quality">质量优先</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              <div className="space-y-2">
                <Label>码率控制</Label>
                <Select defaultValue="vbr_latency">
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="cqp">CQP (恒定质量)</SelectItem>
                    <SelectItem value="cbr">CBR (恒定码率)</SelectItem>
                    <SelectItem value="vbr_latency">VBR Latency (低延迟)</SelectItem>
                  </SelectContent>
                </Select>
              </div>
            </div>
          )}

          {/* 通用选项 */}
          {hwAccelType !== "none" && (
            <>
              <Separator />
              <div className="flex items-center justify-between">
                <div>
                  <Label>启用硬件解码</Label>
                  <p className="text-xs text-muted-foreground">使用 GPU 进行视频解码</p>
                </div>
                <Switch defaultChecked />
              </div>
              <div className="flex items-center justify-between">
                <div>
                  <Label>启用硬件编码</Label>
                  <p className="text-xs text-muted-foreground">使用 GPU 进行视频编码</p>
                </div>
                <Switch defaultChecked />
              </div>
            </>
          )}

          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <Label>HDR 色调映射</Label>
              <p className="text-xs text-muted-foreground">转码 HDR 内容时转换为 SDR</p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>转码配置</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>最大并发转码数</Label>
            <Select defaultValue="2">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="1">1</SelectItem>
                <SelectItem value="2">2</SelectItem>
                <SelectItem value="3">3</SelectItem>
                <SelectItem value="4">4</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>默认视频编码器</Label>
            <Select defaultValue="h264">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="h264">H.264 (AVC)</SelectItem>
                <SelectItem value="h265">H.265 (HEVC)</SelectItem>
                <SelectItem value="av1">AV1</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>默认音频编码器</Label>
            <Select defaultValue="aac">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="aac">AAC</SelectItem>
                <SelectItem value="ac3">AC3</SelectItem>
                <SelectItem value="opus">Opus</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>临时文件目录</Label>
            <Input defaultValue="/var/cache/nako/transcode" />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>转码质量</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>默认最大码率</Label>
              <span className="text-sm text-muted-foreground">20 Mbps</span>
            </div>
            <Slider defaultValue={[20]} max={50} step={1} />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>2 Mbps</span>
              <span>50 Mbps</span>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 网络设置页面
function NetworkSettingsPage() {
  const [proxyEnabled, setProxyEnabled] = useState(false)
  const [ipv6Enabled, setIpv6Enabled] = useState(false)
  
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">网络设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置服务器网络、代理和带宽</p>
      </div>
      
      {/* 绑定设置 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Server className="h-5 w-5" />
            服务器绑定
          </CardTitle>
          <CardDescription>配置服务器监听的地址和端口</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label>HTTP 端口</Label>
              <Input defaultValue="8096" />
            </div>
            <div className="space-y-2">
              <Label>HTTPS 端口</Label>
              <Input defaultValue="8920" />
            </div>
          </div>
          <div className="space-y-2">
            <Label>绑定地址 (IPv4)</Label>
            <Select defaultValue="0.0.0.0">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0.0.0.0">所有接口 (0.0.0.0)</SelectItem>
                <SelectItem value="127.0.0.1">仅本地 (127.0.0.1)</SelectItem>
                <SelectItem value="192.168.1.100">192.168.1.100 (eth0)</SelectItem>
                <SelectItem value="custom">自定义...</SelectItem>
              </SelectContent>
            </Select>
          </div>
          
          <Separator />
          
          <div className="flex items-center justify-between">
            <div>
              <Label>启用 IPv6</Label>
              <p className="text-xs text-muted-foreground">监听 IPv6 地址</p>
            </div>
            <Switch checked={ipv6Enabled} onCheckedChange={setIpv6Enabled} />
          </div>
          
          {ipv6Enabled && (
            <div className="space-y-2">
              <Label>绑定地址 (IPv6)</Label>
              <Select defaultValue="::">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="::">所有接口 (::)</SelectItem>
                  <SelectItem value="::1">仅本地 (::1)</SelectItem>
                  <SelectItem value="custom">自定义...</SelectItem>
                </SelectContent>
              </Select>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 本地网络 */}
      <Card>
        <CardHeader>
          <CardTitle>本地网络识别</CardTitle>
          <CardDescription>定义哪些 IP 范围被视为本地网络，本地网络不受带宽限制</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>本地网络地址</Label>
            <Textarea 
              defaultValue="192.168.0.0/16&#10;10.0.0.0/8&#10;172.16.0.0/12"
              placeholder="每行一个 CIDR 格式的网络地址"
              className="min-h-[100px] font-mono text-sm"
            />
            <p className="text-xs text-muted-foreground">来自这些网络的连接将被视为本地连接</p>
          </div>
          
          <div className="flex items-center justify-between">
            <div>
              <Label>本地连接跳过认证</Label>
              <p className="text-xs text-muted-foreground">本地网络的某些请求可跳过登录验证</p>
            </div>
            <Switch />
          </div>
        </CardContent>
      </Card>

      {/* 带宽限制 */}
      <Card>
        <CardHeader>
          <CardTitle>带宽限制</CardTitle>
          <CardDescription>限制远程访问的带宽使用</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>启用带宽限制</Label>
              <p className="text-xs text-muted-foreground">限制远程流媒体的最大比特率</p>
            </div>
            <Switch defaultChecked />
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>远程用户最大比特率</Label>
              <span className="text-sm text-muted-foreground">20 Mbps</span>
            </div>
            <Slider defaultValue={[20]} max={100} step={1} />
            <div className="flex justify-between text-xs text-muted-foreground">
              <span>1 Mbps</span>
              <span>无限制</span>
            </div>
          </div>
          
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <Label>同步下载限速</Label>
              <span className="text-sm text-muted-foreground">50 Mbps</span>
            </div>
            <Slider defaultValue={[50]} max={100} step={1} />
          </div>
          
          <div className="space-y-2">
            <Label>最大并发流数</Label>
            <Select defaultValue="0">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">无限制</SelectItem>
                <SelectItem value="5">5 个</SelectItem>
                <SelectItem value="10">10 个</SelectItem>
                <SelectItem value="20">20 个</SelectItem>
                <SelectItem value="50">50 个</SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">限制同时播放的流数量，0 表示不限制</p>
          </div>
        </CardContent>
      </Card>

      {/* 代理设置 */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>代理设置</CardTitle>
              <CardDescription>用于元数据刮削、字幕下载等外部请求</CardDescription>
            </div>
            <Switch checked={proxyEnabled} onCheckedChange={setProxyEnabled} />
          </div>
        </CardHeader>
        {proxyEnabled && (
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <Label>代理类型</Label>
              <Select defaultValue="http">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="http">HTTP/HTTPS</SelectItem>
                  <SelectItem value="socks5">SOCKS5</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid gap-4 sm:grid-cols-3">
              <div className="space-y-2 sm:col-span-2">
                <Label>代理地址</Label>
                <Input placeholder="127.0.0.1" />
              </div>
              <div className="space-y-2">
                <Label>端口</Label>
                <Input placeholder="7890" />
              </div>
            </div>
            <div className="grid gap-4 sm:grid-cols-2">
              <div className="space-y-2">
                <Label>用户名 (可选)</Label>
                <Input placeholder="username" />
              </div>
              <div className="space-y-2">
                <Label>密码 (可选)</Label>
                <Input type="password" placeholder="password" />
              </div>
            </div>
            <div className="space-y-2">
              <Label>不使用代理的地址</Label>
              <Input placeholder="localhost, 127.0.0.1, *.local" />
              <p className="text-xs text-muted-foreground">逗号分隔，支持通配符</p>
            </div>
            <Button variant="outline" size="sm">测试连接</Button>
          </CardContent>
        )}
      </Card>

      {/* DNS 设置 */}
      <Card>
        <CardHeader>
          <CardTitle>DNS 设置</CardTitle>
          <CardDescription>自定义 DNS 服务器用于域名解析</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>使用自定义 DNS</Label>
              <p className="text-xs text-muted-foreground">覆盖系统默认 DNS</p>
            </div>
            <Switch />
          </div>
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label>首选 DNS</Label>
              <Input placeholder="8.8.8.8" />
            </div>
            <div className="space-y-2">
              <Label>备用 DNS</Label>
              <Input placeholder="8.8.4.4" />
            </div>
          </div>
          <div className="rounded-lg border border-border/50 bg-muted/30 p-3">
            <p className="text-xs text-muted-foreground">
              常用 DNS：Google (8.8.8.8)、Cloudflare (1.1.1.1)、阿里 (223.5.5.5)、腾讯 (119.29.29.29)
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 通知设置页面
function NotificationsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">通知设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置系统通知和推送服务</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="h-5 w-5" />
            通知渠道
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4 max-h-[280px] overflow-y-auto scrollbar-none">
            {[
              { name: "Telegram", desc: "通过 Telegram Bot 发送通知", enabled: true },
              { name: "Bark", desc: "iOS 推送通知服务", enabled: false },
              { name: "邮件", desc: "通过 SMTP 发送邮件通知", enabled: false },
              { name: "Webhook", desc: "发送 HTTP 请求到自定义 URL", enabled: false },
              { name: "Discord", desc: "通过 Discord Webhook 发送通知", enabled: false },
              { name: "PushDeer", desc: "开源推送服务", enabled: false },
            ].map(channel => (
            <div key={channel.name} className="flex items-center justify-between rounded-lg border border-border/50 p-4">
              <div>
                <p className="font-medium">{channel.name}</p>
                <p className="text-xs text-muted-foreground">{channel.desc}</p>
              </div>
              <div className="flex items-center gap-2">
                <Button variant="ghost" size="sm">配置</Button>
                <Switch defaultChecked={channel.enabled} />
              </div>
            </div>
          ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 备份与恢复页面
function BackupPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">备份与恢复</h1>
        <p className="mt-1 text-sm text-muted-foreground">管理服务器配置和数据库备份</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Download className="h-5 w-5" />
            创建备份
          </CardTitle>
          <CardDescription>备份包含配置、用户数据和元��据数据库</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2 max-h-[160px] overflow-y-auto scrollbar-none">
            {["配置文件", "用户数据", "元数据数据库", "播放记录", "观看历史", "用户偏好"].map(item => (
              <div key={item} className="flex items-center justify-between">
                <span className="text-sm">{item}</span>
                <Switch defaultChecked />
              </div>
            ))}
          </div>
          <Button className="w-full gap-2">
            <Download className="h-4 w-4" />
            创建备份
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>备份历史</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {[
              { date: "2024-01-15 10:30", size: "128 MB", type: "完整备份" },
              { date: "2024-01-08 10:30", size: "125 MB", type: "完整备份" },
            ].map((backup, i) => (
              <div key={i} className="flex items-center justify-between rounded-lg border border-border/50 p-3">
                <div>
                  <p className="text-sm font-medium">{backup.date}</p>
                  <p className="text-xs text-muted-foreground">{backup.type} · {backup.size}</p>
                </div>
                <div className="flex gap-2">
                  <Button variant="ghost" size="sm">下载</Button>
                  <Button variant="ghost" size="sm" className="text-destructive">删除</Button>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  )
  }
  
// 高级设置页面
function AdvancedSettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">高级设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">系统高级配置选项，请谨慎修改</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle>日志设置</CardTitle>
          <CardDescription>配置系统日志级别和存储</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>日志级别</Label>
            <Select defaultValue="info">
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="debug">Debug (调试)</SelectItem>
                <SelectItem value="info">Info (信息)</SelectItem>
                <SelectItem value="warning">Warning (警告)</SelectItem>
                <SelectItem value="error">Error (错误)</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>日志保留天数</Label>
            <Input type="number" defaultValue="30" />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <Label>启用访问日志</Label>
              <p className="text-xs text-muted-foreground">记录所有 API 请求</p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>缓存设置</CardTitle>
          <CardDescription>配置图片和元数据缓存</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>缓存目录</Label>
            <Input defaultValue="/var/cache/nako" />
          </div>
          <div className="space-y-2">
            <Label>最大缓存大小 (GB)</Label>
            <Input type="number" defaultValue="10" />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <Label>自动清理过期缓存</Label>
              <p className="text-xs text-muted-foreground">超过 30 天未访问的缓存将被清理</p>
            </div>
            <Switch defaultChecked />
          </div>
          <Button variant="outline" className="w-full gap-2">
            <Trash2 className="h-4 w-4" />
            立即清理缓存
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>FFmpeg 设置</CardTitle>
          <CardDescription>配置 FFmpeg 路径和参数</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label>FFmpeg 路径</Label>
            <Input defaultValue="/usr/bin/ffmpeg" />
          </div>
          <div className="space-y-2">
            <Label>FFprobe 路径</Label>
            <Input defaultValue="/usr/bin/ffprobe" />
          </div>
          <div className="space-y-2">
            <Label>自定义 FFmpeg 参数</Label>
            <Input placeholder="-threads 4" />
            <p className="text-xs text-muted-foreground">高级用户可添加自定义参数</p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>实验性功能</CardTitle>
          <CardDescription>这些功能仍在开发中，可能不稳定</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <Label>启用 AI 推荐</Label>
              <p className="text-xs text-muted-foreground">基于观看历史的智能推荐</p>
            </div>
            <Switch />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <Label>启用实时字幕</Label>
              <p className="text-xs text-muted-foreground">使用 AI 生成实时��幕</p>
            </div>
            <Switch />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <Label>启用智能跳过</Label>
              <p className="text-xs text-muted-foreground">自动跳过片头片尾</p>
            </div>
            <Switch defaultChecked />
          </div>
        </CardContent>
      </Card>

      <Card className="border-destructive/50">
        <CardHeader>
          <CardTitle className="text-destructive">危险操作</CardTitle>
          <CardDescription>这些操作不可逆，请谨慎执行</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">重置所有设置</p>
              <p className="text-xs text-muted-foreground">将所有设置恢复为默认值</p>
            </div>
            <Button variant="outline" size="sm" className="text-destructive border-destructive/50 hover:bg-destructive/10">
              重置
            </Button>
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">清空元数据数据库</p>
              <p className="text-xs text-muted-foreground">删除所有刮削的元数据，媒体文件不受影响</p>
            </div>
            <Button variant="outline" size="sm" className="text-destructive border-destructive/50 hover:bg-destructive/10">
              清空
            </Button>
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">重建媒体数据库</p>
              <p className="text-xs text-muted-foreground">完全重建媒体索引，耗时较长</p>
            </div>
            <Button variant="outline" size="sm" className="text-destructive border-destructive/50 hover:bg-destructive/10">
              重建
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

// 关于页面
function AboutPage({ metrics: serverMetrics }: { metrics: AdminDashboardMetrics }) {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">关于 Nako</h1>
        <p className="mt-1 text-sm text-muted-foreground">版本信息和系统状态</p>
      </div>
      
      <Card>
        <CardHeader>
          <CardTitle>版本信息</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">当前版本</span>
            <span className="font-mono text-sm">{serverMetrics.version}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">最新版本</span>
            <div className="flex items-center gap-2">
              <span className="font-mono text-sm text-info">{serverMetrics.latestVersion}</span>
              <Badge variant="secondary" className="text-[10px]">有更新</Badge>
            </div>
          </div>
          <Separator />
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">运行时间</span>
            <span className="text-sm">{serverMetrics.uptime}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">操作系统</span>
            <span className="text-sm">Linux (Docker)</span>
          </div>
          <div className="flex items-center justify-between">
            <span className="text-sm text-muted-foreground">数据库</span>
            <span className="text-sm">SQLite 3.42.0</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>链接</CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {[
            { name: "项目主页", url: "https://github.com/nako-media" },
            { name: "文档", url: "https://docs.nako.media" },
            { name: "更新日志", url: "https://github.com/nako-media/releases" },
            { name: "问题反馈", url: "https://github.com/nako-media/issues" },
          ].map(link => (
            <a 
              key={link.name}
              href={link.url}
              target="_blank"
              rel="noopener noreferrer"
              className="flex items-center justify-between rounded-lg border border-border/50 p-3 transition-colors hover:bg-muted/50"
            >
              <span className="text-sm">{link.name}</span>
              <ExternalLink className="h-4 w-4 text-muted-foreground" />
            </a>
          ))}
        </CardContent>
      </Card>
    </div>
  )
}
