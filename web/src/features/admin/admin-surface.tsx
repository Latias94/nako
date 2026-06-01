"use client"

import { lazy, Suspense, useEffect, useState } from "react"
import type { ComponentType } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  Server,
  HardDrive,
  Users,
  Activity,
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  Clock,
  Folder,
  Settings,
  Puzzle,
  Cpu,
  Play,
  Wrench,
  Network,
  Bell,
  Globe,
  Download,
  RotateCcw,
  ArrowUpRight,
  Info,
  ExternalLink,
  Monitor,
  FileSearch,
  Sparkles,
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
import type { AdminGeneratedArtifactsRouteState } from "./admin-generated-artifacts"
import {
  AdminGeneratedArtifactRecovery,
  type AdminGeneratedArtifactRecoveryRouteState,
} from "./admin-generated-artifact-recovery"
import {
  AdminGeneratedArtifactReview,
  type AdminGeneratedArtifactReviewRouteState,
} from "./admin-generated-artifact-review"
import {
  AdminGeneratedArtifactMetadataApply,
  type AdminGeneratedArtifactMetadataApplyRouteState,
} from "./admin-generated-artifact-metadata-apply"
import {
  AdminMetadataCandidateReview,
  type AdminMetadataCandidateReviewRouteState,
} from "./admin-metadata-candidate-review"
import { AdminManagementContextNotice } from "./admin-management-context"
import type { AdminManagementContextRouteState } from "./admin-management-context-state"
import type { AdminGeneratedArtifactReviewDecision } from "@/src/api/admin/read-models-data-source"
import {
  ADMIN_DASHBOARD_FIXTURE,
  createAdminDashboardDataSource,
  type AdminDashboardData,
  type AdminDashboardMetrics,
  type AdminDashboardPlaybackSession,
  type AdminDashboardTask,
} from "@/src/api/admin/dashboard-data-source"

const AdminGeneratedArtifacts = lazy(() =>
  import("./admin-generated-artifacts").then((module) => ({
    default: module.AdminGeneratedArtifacts,
  })),
)

export type AdminSurfaceSection =
  | "dashboard"
  | "activity"
  | "scheduled-tasks"
  | "acquisition-intake"
  | "generated-artifacts"
  | "generated-artifact-recovery"
  | "generated-artifact-review"
  | "generated-artifact-metadata-apply"
  | "metadata-candidate-review"
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
  generatedArtifactsState?: AdminGeneratedArtifactsRouteState
  onGeneratedArtifactsStateChange?: (state: AdminGeneratedArtifactsRouteState) => void
  generatedArtifactRecoveryState?: AdminGeneratedArtifactRecoveryRouteState
  onGeneratedArtifactRecoveryStateChange?: (state: AdminGeneratedArtifactRecoveryRouteState) => void
  generatedArtifactReviewState?: AdminGeneratedArtifactReviewRouteState
  generatedArtifactMetadataApplyState?: AdminGeneratedArtifactMetadataApplyRouteState
  metadataCandidateReviewState?: AdminMetadataCandidateReviewRouteState
  managementContextState?: AdminManagementContextRouteState
  onGeneratedArtifactReviewStateChange?: (state: AdminGeneratedArtifactReviewRouteState) => void
  onMetadataCandidateReviewStateChange?: (state: AdminMetadataCandidateReviewRouteState) => void
  onGeneratedArtifactReviewRequest?: (
    artifactId: string,
    decision: AdminGeneratedArtifactReviewDecision,
  ) => void
  onGeneratedArtifactRecoveryRequest?: () => void
  onGeneratedArtifactReviewBack?: () => void
  onGeneratedArtifactMetadataApplyRequest?: (artifactId: string) => void
  onGeneratedArtifactMetadataApplyBack?: () => void
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
      { name: "生成产物", icon: Sparkles, component: "generated-artifacts" },
      { name: "恢复队列", icon: RotateCcw, component: "generated-artifact-recovery" },
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

export function AdminSurface({
  activeSection = "dashboard",
  onSectionNavigate,
  adminLogsState,
  onAdminLogsStateChange,
  acquisitionIntakeState,
  onAcquisitionIntakeStateChange,
  generatedArtifactsState,
  onGeneratedArtifactsStateChange,
  generatedArtifactRecoveryState,
  onGeneratedArtifactRecoveryStateChange,
  generatedArtifactReviewState,
  generatedArtifactMetadataApplyState,
  metadataCandidateReviewState,
  managementContextState,
  onGeneratedArtifactReviewStateChange,
  onMetadataCandidateReviewStateChange,
  onGeneratedArtifactReviewRequest,
  onGeneratedArtifactRecoveryRequest,
  onGeneratedArtifactReviewBack,
  onGeneratedArtifactMetadataApplyRequest,
  onGeneratedArtifactMetadataApplyBack,
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
        return <AdminLibraries managementContext={managementContextState} />
      case "users":
        return <AdminUsers managementContext={managementContextState} />
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
      case "generated-artifacts":
        return (
          <AdminGeneratedArtifacts
            routeState={generatedArtifactsState}
            onRouteStateChange={onGeneratedArtifactsStateChange}
            onReviewRequest={onGeneratedArtifactReviewRequest}
            onRecoveryRequest={onGeneratedArtifactRecoveryRequest}
          />
        )
      case "generated-artifact-recovery":
        return (
          <AdminGeneratedArtifactRecovery
            routeState={generatedArtifactRecoveryState}
            onRouteStateChange={onGeneratedArtifactRecoveryStateChange}
            onBackToArtifacts={() => navigateToSection("generated-artifacts")}
            onApplyRequest={onGeneratedArtifactMetadataApplyRequest}
          />
        )
      case "generated-artifact-review":
        return (
          <AdminGeneratedArtifactReview
            routeState={generatedArtifactReviewState}
            onRouteStateChange={onGeneratedArtifactReviewStateChange}
            onBackToQueue={onGeneratedArtifactReviewBack}
            onMetadataApplyRequest={onGeneratedArtifactMetadataApplyRequest}
          />
        )
      case "generated-artifact-metadata-apply":
        return (
          <AdminGeneratedArtifactMetadataApply
            routeState={generatedArtifactMetadataApplyState}
            onBackToQueue={onGeneratedArtifactMetadataApplyBack}
          />
        )
      case "metadata-candidate-review":
        return (
          <AdminMetadataCandidateReview
            routeState={metadataCandidateReviewState}
            onRouteStateChange={onMetadataCandidateReviewStateChange}
          />
        )
      case "scheduled-tasks":
        return <AdminScheduledTasks managementContext={managementContextState} />
      case "dlna":
        return <AdminPlaceholderPage title="DLNA/UPnP" description="本地网络发现和媒体共享等待 Admin API 合约接入。" />
      case "remote-access":
        return <AdminPlaceholderPage title="远程访问" description="Remote Access Endpoint 和 Network Tunnel Provider 接入后再开放配置面板。" />
      case "transcoding":
        return <TranscodingRuntimePage managementContext={managementContextState} />
      case "network":
        return <AdminPlaceholderPage title="网络设置" description="网络代理、DNS 和证书设置等待 Admin API 合约接入。" />
      case "notifications":
        return <AdminPlaceholderPage title="通知设置" description="通知渠道会通过 webhook 和事件投递合约接入。" />
      case "backup":
        return <AdminPlaceholderPage title="备份与恢复" description="配置和数据库备份需要 durable job 合约后再开放。" />
      case "advanced":
        return <AdminSettings />
      case "about":
        return <AdminPlaceholderPage title="关于 Nako" description={`当前版本 ${serverMetrics.version}`} />
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
                  <AdminNavButton
                    key={item.name}
                    item={item}
                    activeComponent={activeComponent}
                    onClick={() => navigateToSection(item.component)}
                  />
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
          <Suspense fallback={<AdminContentFallback />}>{renderContent()}</Suspense>
        </div>
      </main>
    </div>
  )
}
function AdminContentFallback() {
  return (
    <div className="grid min-h-[18rem] place-items-center rounded-lg border border-border/50 bg-card">
      <div className="h-7 w-7 animate-spin rounded-full border-2 border-muted border-t-primary" />
    </div>
  )
}

function AdminNavButton({
  item,
  activeComponent,
  onClick,
}: {
  item: AdminNavItem
  activeComponent: AdminSurfaceSection
  onClick: () => void
}) {
  const active =
    activeComponent === item.component ||
    ((activeComponent === "generated-artifact-review" ||
      activeComponent === "generated-artifact-metadata-apply") &&
      item.component === "generated-artifacts")

  return (
    <button
      onClick={onClick}
      className={cn(
        "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-sm transition-colors",
        active
          ? "bg-sidebar-accent text-sidebar-accent-foreground"
          : "text-sidebar-foreground/80 hover:bg-sidebar-accent/50",
      )}
    >
      <item.icon className="h-4 w-4" />
      <span>{item.name}</span>
    </button>
  )
}

function AdminPlaceholderPage({ title, description }: { title: string; description: string }) {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">{title}</h1>
        <p className="mt-1 text-sm text-muted-foreground">{description}</p>
      </div>
      <section className="rounded-lg border border-border/50 bg-card p-4">
        <Badge variant="secondary">planned Admin API</Badge>
      </section>
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
          </div>
          <div className="divide-y divide-border/50">
            <div className="px-4 py-4 text-sm text-muted-foreground">
              活动流待接入。
            </div>
          </div>
        </section>
      </div>

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

function TranscodingRuntimePage({
  managementContext,
}: {
  managementContext?: AdminManagementContextRouteState
}) {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">转码设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          转码策略配置由 Admin Settings 管理；运行时诊断通过 Management Link 进入。
        </p>
      </div>

      <AdminManagementContextNotice
        state={managementContext}
        title={transcodingManagementContextTitle(managementContext)}
        description="播放支持与转码运行时上下文。"
      />

      <section className="rounded-lg border border-border/50 bg-card p-4">
        <div className="flex flex-wrap gap-2">
          <Badge variant="secondary">runtime diagnostics</Badge>
          <Badge variant="outline">planned Admin API</Badge>
        </div>
      </section>
    </div>
  )
}

function transcodingManagementContextTitle(
  managementContext: AdminManagementContextRouteState | undefined,
) {
  switch (managementContext?.panel) {
    case "support":
      return "播放诊断"
    case "runtime":
      return "转码运行时"
    default:
      return "管理上下文"
  }
}

// 转码设置页面
function TranscodingSettingsPage({
  managementContext,
}: {
  managementContext?: AdminManagementContextRouteState
}) {
  const [hwAccelType, setHwAccelType] = useState("vaapi")
  
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold">转码设置</h1>
        <p className="mt-1 text-sm text-muted-foreground">配置视频转码和硬件加速</p>
      </div>

      <AdminManagementContextNotice
        state={managementContext}
        title={transcodingManagementContextTitle(managementContext)}
        description="播放和转码运行时上下文。"
      />
      
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
