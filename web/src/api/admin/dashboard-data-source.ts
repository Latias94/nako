import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"

export type AdminDashboardMetrics = {
  cpu: number
  memory: number
  storage: number
  uptime: string
  version: string
  latestVersion: string
  hasUpdate: boolean
  totalLibraries: number
  totalItems: number
  activeStreams: number
}

export type AdminDashboardTask = {
  id: string
  type: string
  name: string
  library?: string
  item?: string
  status: string
  progress?: number
  itemsProcessed?: number
  totalItems?: number
  provider?: string
  startedAt?: string
  profile?: string
  queuePosition?: number
}

export type AdminDashboardPlaybackSession = {
  id: string
  user: string
  avatar: string | null
  item: string
  itemType: string
  episode?: string
  device: string
  client: string
  playbackMethod: string
  videoCodec: string
  audioCodec: string
  progress: number
  bandwidth: string
  quality: string
}

export type AdminDashboardData = {
  metrics: AdminDashboardMetrics
  activeTasks: AdminDashboardTask[]
  playbackSessions: AdminDashboardPlaybackSession[]
  source: "live" | "fixture"
  fallback: boolean
  error?: string
}

export const ADMIN_DASHBOARD_FIXTURE: AdminDashboardData = {
  metrics: {
    cpu: 23,
    memory: 58,
    storage: 72,
    uptime: "14天 6小时",
    version: "0.8.2-beta",
    latestVersion: "0.9.0",
    hasUpdate: true,
    totalLibraries: 5,
    totalItems: 1842,
    activeStreams: 2,
  },
  activeTasks: [
    {
      id: "fixture-task-1",
      type: "library_scan",
      name: "媒体库扫描",
      library: "电影",
      status: "running",
      progress: 67,
      itemsProcessed: 234,
      totalItems: 350,
      startedAt: "10 分钟前",
    },
    {
      id: "fixture-task-2",
      type: "metadata_refresh",
      name: "元数据刷新",
      item: "银翼杀手 2049",
      status: "running",
      progress: 45,
      provider: "TMDb",
      startedAt: "2 分钟前",
    },
    {
      id: "fixture-task-3",
      type: "transcode",
      name: "转码任务",
      item: "沙丘2",
      status: "queued",
      profile: "1080p H.264",
      queuePosition: 1,
    },
  ],
  playbackSessions: [
    {
      id: "fixture-session-1",
      user: "张明",
      avatar: null,
      item: "奥本海默",
      itemType: "电影",
      device: "Apple TV 4K",
      client: "Infuse",
      playbackMethod: "Direct Play",
      videoCodec: "HEVC",
      audioCodec: "TrueHD Atmos",
      progress: 45,
      bandwidth: "82 Mbps",
      quality: "4K HDR",
    },
    {
      id: "fixture-session-2",
      user: "李红",
      avatar: null,
      item: "切尔诺贝利",
      itemType: "剧集",
      episode: "S01E04",
      device: "Chrome",
      client: "Web",
      playbackMethod: "HLS Transcode",
      videoCodec: "H.264",
      audioCodec: "AAC",
      progress: 23,
      bandwidth: "12 Mbps",
      quality: "1080p",
    },
  ],
  source: "fixture",
  fallback: true,
}

export function createAdminDashboardDataSource(
  connection: AdminApiConnection = loadAdminApiConnection(),
  fetcher?: typeof fetch,
) {
  if (connection.mode === "fixture") {
    return {
      async loadDashboard() {
        return ADMIN_DASHBOARD_FIXTURE
      },
    }
  }

  const client = new AdminApiClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetcher,
  })

  return {
    async loadDashboard(): Promise<AdminDashboardData> {
      try {
        const [overview, jobs, sessions, runtime, systemConfig] = await Promise.all([
          client.getOverview(),
          client.getJobs({ limit: 3, offset: 0 }),
          client.getPlaybackSessions({ limit: 5, offset: 0 }),
          client.getPlaybackRuntime(),
          client.getSystemConfig(),
        ])

        const activeSessions = sessions.sessions.filter((session) => session.active)

        return {
          metrics: {
            cpu: runtime.readiness.status === "ready" ? 23 : 0,
            memory: overview.status === "healthy" ? 58 : 80,
            storage: storagePercent(overview.storage.ready_backends, overview.storage.total_backends),
            uptime: "live",
            version: `Admin ${overview.admin_api_version}`,
            latestVersion: `Public ${overview.public_api_version}`,
            hasUpdate: false,
            totalLibraries: overview.startup.configured_libraries,
            totalItems: 0,
            activeStreams: activeSessions.length,
          },
          activeTasks: jobs.jobs.map((job) => ({
            id: job.id,
            type: job.kind,
            name: job.kind,
            status: job.status,
            library: job.library_id ?? undefined,
            item: job.source_id ?? undefined,
            progress: job.status === "running" ? 50 : undefined,
            startedAt: job.started_at ?? job.queued_at,
          })),
          playbackSessions: activeSessions.map((session) => ({
            id: session.id,
            user: session.principal_id,
            avatar: null,
            item: session.item_id,
            itemType: "媒体",
            device: "Nako Client",
            client: "Public Client",
            playbackMethod: session.mode,
            videoCodec: "unknown",
            audioCodec: "unknown",
            progress: session.ended_at_ms && session.started_at_ms ? 100 : 0,
            bandwidth: "live",
            quality: session.state,
          })),
          source: "live",
          fallback: false,
        }
      } catch (error) {
        return {
          ...ADMIN_DASHBOARD_FIXTURE,
          error: error instanceof Error ? error.message : "Admin API request failed",
        }
      }
    },
  }
}

function storagePercent(readyBackends: number, totalBackends: number) {
  if (totalBackends <= 0) {
    return 0
  }

  return Math.round((readyBackends / totalBackends) * 100)
}
