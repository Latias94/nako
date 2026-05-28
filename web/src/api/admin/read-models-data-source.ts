import { AdminApiClient } from "./client"
import { loadAdminApiConnection, type AdminApiConnection } from "./connection"
import type {
  AdminAccessSummaryResponse,
  AdminAccessUserRecord,
  AdminJobListItem,
  AdminMetadataRawCacheSettingsResponse,
  AdminOutboxEventListItem,
  AdminOverviewResponse,
  AdminPlaybackRuntimeDiagnosticsResponse,
  AdminPlaybackSessionListItem,
  AdminServerConfigDiagnosticsResponse,
  AdminStorageStagingDiagnosticsResponse,
  AdminUserRole,
} from "./generated/contract"

export type AdminReadModelSource = "live" | "fixture"

export interface AdminReadModelEnvelope {
  source: AdminReadModelSource
  fallback: boolean
  error?: string
}

export type AdminLibraryKind =
  | "movie"
  | "tv"
  | "anime"
  | "music"
  | "photo"
  | "documentary"
  | "personal"
  | "unknown"

export type AdminLibraryScanStatus = "idle" | "scanning" | "error"

export interface AdminLibraryReadModel {
  id: string
  name: string
  type: AdminLibraryKind
  paths: Array<{
    path: string
    available: boolean
  }>
  itemCount: number
  totalSize: string
  lastScanned: string
  scanStatus: AdminLibraryScanStatus
  scanProgress?: number
  errorMessage?: string
  settings: {
    autoScan: boolean
    scanInterval: number
    useNfo: boolean
    downloadArt: boolean
    metadataLanguage: string
  }
}

export interface AdminLibrariesReadModel extends AdminReadModelEnvelope {
  libraries: AdminLibraryReadModel[]
}

export type AdminUserUiRole = "admin" | "user" | "guest"
export type AdminUserUiStatus = "online" | "offline" | "disabled"

export interface AdminUserReadModel {
  id: string
  name: string
  username: string
  email: string | null
  role: AdminUserUiRole
  avatar: string | null
  status: AdminUserUiStatus
  lastActive: string
  createdAt: string
  libraryAccess: string[]
  settings: {
    canDownload: boolean
    canDelete: boolean
    canManageUsers: boolean
    maxBitrate: number | null
    transcodePolicy: string
    maxStreams: number | null
    remoteAccess: boolean
  }
  stats: {
    totalPlays: number
    totalWatchTime: string
    lastLogin: string
  }
}

export interface AdminActiveSessionReadModel {
  id: string
  userId: string
  userName: string
  device: string
  deviceType: string
  ip: string
  location: string
  lastActivity: string
  startTime: string
  current: boolean
}

export interface AdminLibraryOptionReadModel {
  id: string
  name: string
  type: AdminLibraryKind
}

export interface AdminUsersReadModel extends AdminReadModelEnvelope {
  users: AdminUserReadModel[]
  activeSessions: AdminActiveSessionReadModel[]
  libraries: AdminLibraryOptionReadModel[]
  accessMode: string
  accessCapabilities: {
    userAccounts: string
    roles: string
    libraryAccessPolicy: string
  }
}

export type AdminTaskType =
  | "scan"
  | "metadata"
  | "backup"
  | "cleanup"
  | "update"
  | "optimize"
  | "subtitle"
  | "thumbnail"

export type AdminTaskStatus = "idle" | "running" | "success" | "failed" | "scheduled"
export type AdminScheduleFrequency = "interval" | "daily" | "weekly" | "monthly" | "cron"

export interface AdminTaskConfig {
  targetLibraries?: string[]
  scanNewOnly?: boolean
  backupPath?: string
  keepBackups?: number
  includeImages?: boolean
  cacheMaxAge?: number
  minFreeSpace?: number
  cleanTranscodes?: boolean
  cleanImages?: boolean
  cleanLogs?: boolean
  metadataLanguage?: string
  refreshAll?: boolean
  downloadImages?: boolean
  subtitleLanguages?: string[]
  preferSDH?: boolean
  overwriteExisting?: boolean
  chapters?: boolean
}

export interface AdminScheduledTaskReadModel {
  id: string
  name: string
  description: string
  type: AdminTaskType
  enabled: boolean
  schedule: {
    frequency: AdminScheduleFrequency
    time?: string
    days?: number[]
    interval?: number
    cron?: string
  }
  config: AdminTaskConfig
  lastRun?: {
    timestamp: string
    status: "success" | "failed"
    duration: string
    itemsProcessed?: number
    error?: string
  }
  nextRun?: string
  status: AdminTaskStatus
  progress?: number
}

export interface AdminTaskHistoryReadModel {
  id: string
  taskId: string
  taskName: string
  taskType: AdminTaskType
  timestamp: string
  status: "success" | "failed"
  duration: string
  itemsProcessed?: number
  error?: string
}

export interface AdminTasksReadModel extends AdminReadModelEnvelope {
  tasks: AdminScheduledTaskReadModel[]
  runningTask: AdminScheduledTaskReadModel | null
  history: AdminTaskHistoryReadModel[]
  libraries: AdminLibraryOptionReadModel[]
}

export type AdminLogLevel = "error" | "warn" | "info" | "debug"
export type AdminLogSource = "server" | "auth" | "database" | "api" | "playback" | "scanner"

export interface AdminLogEntryReadModel {
  id: string
  timestamp: string
  level: AdminLogLevel
  source: AdminLogSource
  message: string
  details?: string
  userId?: string
  requestId?: string
}

export interface AdminLogsReadModel extends AdminReadModelEnvelope {
  logs: AdminLogEntryReadModel[]
}

export interface AdminSettingsReadModel extends AdminReadModelEnvelope {
  general: {
    serverName: string
    serverId: string
    listenAddr: string
    authEnabled: boolean
    adminApiVersion: string
    publicApiVersion: string
  }
  network: {
    exposureMode: string
    readinessStatus: string
    readinessReason: string
    externalEndpointConfigured: boolean
    allowedOriginCount: number
  }
  database: {
    configuredBackendKind: string
    activeBackendKind: string
    migratedOnStartup: boolean
    librariesSupported: boolean
  }
  metadata: {
    rawCacheRetentionMs: number
    rawCacheCleanupOnStartup: boolean
    providerCount: number
    enabledProviderCount: number
    language: string
  }
  transcode: {
    hardwareAcceleration: string
    hardwareFallbackUsed: boolean
    cpuConcurrency: number
    gpuConcurrency: number
    remuxConcurrency: number
    remuxTimeoutMs: number
  }
  storage: {
    stagingMaxBytes: number
    stagingUsedBytes: number
    stagingRetentionMs: number
    stagingCleanupOnStartup: boolean
    stagingRecordCount: number
  }
  runtime: {
    scanConcurrency: number
    probeConcurrency: number
    metadataConcurrency: number
    webhookConcurrency: number
  }
}

export const ADMIN_LIBRARY_READ_MODEL_FIXTURE: AdminLibrariesReadModel = {
  source: "fixture",
  fallback: true,
  libraries: [
    fixtureLibrary("1", "电影", "movie", "/media/movies", 847, "4.2 TB", "2024-03-15 14:30"),
    {
      ...fixtureLibrary("2", "剧集", "tv", "/media/tv", 156, "8.7 TB", "2024-03-15 14:35"),
      scanStatus: "scanning",
      scanProgress: 67,
    },
    fixtureLibrary("3", "动画", "anime", "/media/anime", 234, "2.1 TB", "2024-03-15 12:00"),
    {
      ...fixtureLibrary("4", "纪录片", "documentary", "/media/documentary", 89, "890 GB", "2024-03-14 22:00"),
      paths: [{ path: "/media/documentary", available: false }],
      scanStatus: "error",
      errorMessage: "路径不可访问: /media/documentary",
      settings: {
        autoScan: false,
        scanInterval: 24,
        useNfo: false,
        downloadArt: true,
        metadataLanguage: "zh-CN",
      },
    },
  ],
}

export const ADMIN_USERS_READ_MODEL_FIXTURE: AdminUsersReadModel = {
  source: "fixture",
  fallback: true,
  users: [
    fixtureUser("1", "管理员", "admin", "admin", "online", ["all"], "在线"),
    fixtureUser("2", "小明", "xiaoming", "user", "online", ["1", "2", "3"], "3 分钟前"),
    fixtureUser("3", "小红", "xiaohong", "user", "offline", ["1", "3"], "昨天 22:15"),
    fixtureUser("4", "访客", "guest", "guest", "offline", ["1"], "从未登录"),
    fixtureUser("5", "家人", "family", "user", "disabled", ["1", "2"], "账户已禁用"),
  ],
  activeSessions: [
    fixtureSession("s1", "1", "管理员", "Chrome - Windows", "desktop", "正在播放: 沙丘2", true),
    fixtureSession("s2", "2", "小明", "Nako iOS App", "mobile", "浏览媒体库", false),
    fixtureSession("s3", "2", "小明", "Samsung TV", "tv", "正在播放: 奥本海默", false),
  ],
  libraries: [
    { id: "1", name: "电影", type: "movie" },
    { id: "2", name: "剧集", type: "tv" },
    { id: "3", name: "动画", type: "anime" },
    { id: "4", name: "纪录片", type: "documentary" },
    { id: "5", name: "个人收藏", type: "personal" },
  ],
  accessMode: "fixture",
  accessCapabilities: {
    userAccounts: "planned",
    roles: "planned",
    libraryAccessPolicy: "planned",
  },
}

export const ADMIN_TASKS_READ_MODEL_FIXTURE: AdminTasksReadModel = {
  source: "fixture",
  fallback: true,
  tasks: [
    fixtureTask("1", "每日媒体库扫描", "扫描所有媒体库以发现新内容", "scan", "scheduled", true),
    fixtureTask("2", "元数据刷新", "更新最近添加项目的元数据", "metadata", "scheduled", true),
    fixtureTask("3", "数据库备份", "创建数据库和配置的备份", "backup", "scheduled", true),
    {
      ...fixtureTask("4", "转码缓存清理", "删除旧的转码缓存文件", "cleanup", "scheduled", true),
      lastRun: {
        timestamp: "2024-03-09T05:00:00Z",
        status: "failed",
        duration: "0分45秒",
        error: "任务执行失败: 权限不足或网络超时",
      },
    },
    fixtureTask("5", "自动更新检查", "检查服务器更新", "update", "idle", false),
    fixtureTask("6", "数据库优化", "优化数据库性能", "optimize", "scheduled", true),
    fixtureTask("7", "字幕自动下载", "为缺失字幕的视频下载字幕", "subtitle", "scheduled", true),
  ],
  runningTask: {
    ...fixtureTask("running-1", "手动媒体库扫描", "全媒体库扫描（手动触发）", "scan", "running", true),
    progress: 67,
  },
  history: buildFixtureHistory(),
  libraries: ADMIN_USERS_READ_MODEL_FIXTURE.libraries.slice(0, 4),
}

export const ADMIN_LOGS_READ_MODEL_FIXTURE: AdminLogsReadModel = {
  source: "fixture",
  fallback: true,
  logs: buildFixtureLogs(),
}

export const ADMIN_SETTINGS_READ_MODEL_FIXTURE: AdminSettingsReadModel = {
  source: "fixture",
  fallback: true,
  general: {
    serverName: "Nako Server",
    serverId: "nako-fixture",
    listenAddr: "0.0.0.0:8096",
    authEnabled: true,
    adminApiVersion: "fixture",
    publicApiVersion: "fixture",
  },
  network: {
    exposureMode: "private_network",
    readinessStatus: "ready",
    readinessReason: "ready",
    externalEndpointConfigured: true,
    allowedOriginCount: 1,
  },
  database: {
    configuredBackendKind: "sqlite",
    activeBackendKind: "sqlite",
    migratedOnStartup: true,
    librariesSupported: true,
  },
  metadata: {
    rawCacheRetentionMs: 7 * 24 * 60 * 60 * 1000,
    rawCacheCleanupOnStartup: true,
    providerCount: 3,
    enabledProviderCount: 2,
    language: "zh-CN",
  },
  transcode: {
    hardwareAcceleration: "qsv",
    hardwareFallbackUsed: false,
    cpuConcurrency: 2,
    gpuConcurrency: 1,
    remuxConcurrency: 2,
    remuxTimeoutMs: 300_000,
  },
  storage: {
    stagingMaxBytes: 100 * 1024 * 1024 * 1024,
    stagingUsedBytes: 45 * 1024 * 1024 * 1024,
    stagingRetentionMs: 24 * 60 * 60 * 1000,
    stagingCleanupOnStartup: true,
    stagingRecordCount: 3,
  },
  runtime: {
    scanConcurrency: 2,
    probeConcurrency: 4,
    metadataConcurrency: 4,
    webhookConcurrency: 2,
  },
}

export function createAdminReadModelsDataSource(
  connection: AdminApiConnection = loadAdminApiConnection(),
  fetcher?: typeof fetch,
) {
  if (connection.mode === "fixture") {
    return fixtureDataSource()
  }

  const client = new AdminApiClient({
    baseUrl: connection.baseUrl,
    bearerToken: connection.bearerToken,
    fetcher,
  })

  return {
    async loadLibraries(): Promise<AdminLibrariesReadModel> {
      return withFallback(ADMIN_LIBRARY_READ_MODEL_FIXTURE, async () => {
        const [overview, config] = await Promise.all([client.getOverview(), client.getSystemConfig()])
        return mapLibraries(overview, config)
      })
    },

    async loadUsers(): Promise<AdminUsersReadModel> {
      return withFallback(ADMIN_USERS_READ_MODEL_FIXTURE, async () => {
        const [summary, users, sessions] = await Promise.all([
          client.getAccessSummary(),
          client.getAccessUsers({ limit: 100, offset: 0 }),
          client.getPlaybackSessions({ limit: 100, offset: 0 }),
        ])
        return mapUsers(summary, users.users, sessions.sessions)
      })
    },

    async loadTasks(): Promise<AdminTasksReadModel> {
      return withFallback(ADMIN_TASKS_READ_MODEL_FIXTURE, async () => {
        const [jobs, events, summary] = await Promise.all([
          client.getJobs({ limit: 100, offset: 0 }),
          client.getEvents({ limit: 100, offset: 0 }),
          client.getAccessSummary(),
        ])
        return mapTasks(jobs.jobs, events.events, summary)
      })
    },

    async loadLogs(): Promise<AdminLogsReadModel> {
      return withFallback(ADMIN_LOGS_READ_MODEL_FIXTURE, async () => {
        const events = await client.getEvents({ limit: 200, offset: 0 })
        return {
          source: "live",
          fallback: false,
          logs: events.events.map(mapEventToLog),
        }
      })
    },

    async loadSettings(): Promise<AdminSettingsReadModel> {
      return withFallback(ADMIN_SETTINGS_READ_MODEL_FIXTURE, async () => {
        const [config, runtime, staging, rawCache] = await Promise.all([
          client.getSystemConfig(),
          client.getPlaybackRuntime(),
          client.getStorageStaging({ limit: 100, offset: 0 }),
          client.getMetadataRawCacheSettings(),
        ])
        return mapSettings(config, runtime, staging, rawCache)
      })
    },
  }
}

function fixtureDataSource() {
  return {
    async loadLibraries() {
      return ADMIN_LIBRARY_READ_MODEL_FIXTURE
    },
    async loadUsers() {
      return ADMIN_USERS_READ_MODEL_FIXTURE
    },
    async loadTasks() {
      return ADMIN_TASKS_READ_MODEL_FIXTURE
    },
    async loadLogs() {
      return ADMIN_LOGS_READ_MODEL_FIXTURE
    },
    async loadSettings() {
      return ADMIN_SETTINGS_READ_MODEL_FIXTURE
    },
  }
}

async function withFallback<T extends AdminReadModelEnvelope>(
  fixture: T,
  loadLive: () => Promise<T>,
): Promise<T> {
  try {
    return await loadLive()
  } catch (error) {
    return {
      ...fixture,
      source: "fixture",
      fallback: true,
      error: error instanceof Error ? error.message : "Admin API request failed",
    }
  }
}

function mapLibraries(
  overview: AdminOverviewResponse,
  config: AdminServerConfigDiagnosticsResponse,
): AdminLibrariesReadModel {
  const backendStatusByLibrary = new Map(
    overview.storage.backends.map((backend) => [backend.library_id, backend.status]),
  )

  return {
    source: "live",
    fallback: false,
    libraries: config.libraries.map((library) => {
      const backendStatus = backendStatusByLibrary.get(library.id) ?? "ready"
      const available = backendStatus !== "unavailable"

      return {
        id: library.id,
        name: library.name,
        type: normalizeLibraryKind(library.preset),
        paths: [
          {
            path: `${library.root_scheme}://${library.backend_kind}`,
            available,
          },
        ],
        itemCount: 0,
        totalSize: "live",
        lastScanned: "live",
        scanStatus: available ? "idle" : "error",
        errorMessage: available ? undefined : `存储后端不可用: ${backendStatus}`,
        settings: {
          autoScan: true,
          scanInterval: 6,
          useNfo: config.database.capabilities.metadata,
          downloadArt: config.artwork.ingest_worker_enabled,
          metadataLanguage: firstMetadataLanguage(config),
        },
      }
    }),
  }
}

function mapUsers(
  summary: AdminAccessSummaryResponse,
  users: AdminAccessUserRecord[],
  sessions: AdminPlaybackSessionListItem[],
): AdminUsersReadModel {
  const activeSessions = sessions.filter((session) => session.active)
  const activeUserIds = new Set(activeSessions.map((session) => session.principal_id))

  return {
    source: "live",
    fallback: false,
    users: users.map((user) => mapUser(user, activeUserIds)),
    activeSessions: activeSessions.map((session) => ({
      id: session.id,
      userId: session.principal_id,
      userName: session.principal_id,
      device: "Nako Client",
      deviceType: "desktop",
      ip: "redacted",
      location: "Admin API",
      lastActivity: session.active ? `正在播放: ${session.item_id}` : session.state,
      startTime: isoFromMs(session.started_at_ms),
      current: false,
    })),
    libraries: summary.library_access.libraries.map((library) => ({
      id: library.library_id,
      name: library.library_name,
      type: normalizeLibraryKind(library.preset),
    })),
    accessMode: summary.mode,
    accessCapabilities: {
      userAccounts: summary.readiness.user_accounts,
      roles: summary.readiness.roles,
      libraryAccessPolicy: summary.readiness.library_access_policy,
    },
  }
}

function mapTasks(
  jobs: AdminJobListItem[],
  events: AdminOutboxEventListItem[],
  summary: AdminAccessSummaryResponse,
): AdminTasksReadModel {
  const tasks = jobs.map(mapJobToTask)
  const runningTask = tasks.find((task) => task.status === "running") ?? null

  return {
    source: "live",
    fallback: false,
    tasks,
    runningTask,
    history: events.map(mapEventToHistory),
    libraries: summary.library_access.libraries.map((library) => ({
      id: library.library_id,
      name: library.library_name,
      type: normalizeLibraryKind(library.preset),
    })),
  }
}

function mapSettings(
  config: AdminServerConfigDiagnosticsResponse,
  runtime: AdminPlaybackRuntimeDiagnosticsResponse,
  staging: AdminStorageStagingDiagnosticsResponse,
  rawCache: AdminMetadataRawCacheSettingsResponse,
): AdminSettingsReadModel {
  return {
    source: "live",
    fallback: false,
    general: {
      serverName: "Nako Server",
      serverId: `nako-${config.database.active_backend_kind}`,
      listenAddr: config.runtime.listen_addr,
      authEnabled: config.auth.enabled,
      adminApiVersion: config.admin_api_version,
      publicApiVersion: config.public_api_version,
    },
    network: {
      exposureMode: config.network.exposure_mode,
      readinessStatus: config.network.readiness.status,
      readinessReason: config.network.readiness.reason,
      externalEndpointConfigured: config.network.external_endpoint.configured,
      allowedOriginCount: config.network.origins.allowed_origin_count,
    },
    database: {
      configuredBackendKind: config.database.configured_backend_kind,
      activeBackendKind: config.database.active_backend_kind,
      migratedOnStartup: config.database.migrated_on_startup,
      librariesSupported: config.database.capabilities.libraries,
    },
    metadata: {
      rawCacheRetentionMs: rawCache.retention_ms,
      rawCacheCleanupOnStartup: rawCache.cleanup_on_startup,
      providerCount: config.metadata.providers.length,
      enabledProviderCount: config.metadata.providers.filter((provider) => provider.enabled).length,
      language: firstMetadataLanguage(config),
    },
    transcode: {
      hardwareAcceleration: runtime.hardware.selection.acceleration,
      hardwareFallbackUsed: runtime.hardware.selection.fallback_used,
      cpuConcurrency: config.transcode.cpu_concurrency,
      gpuConcurrency: config.transcode.gpu_concurrency,
      remuxConcurrency: runtime.remux.max_concurrent_sessions,
      remuxTimeoutMs: runtime.remux.timeout_ms,
    },
    storage: {
      stagingMaxBytes: staging.summary.configured_max_bytes,
      stagingUsedBytes: staging.summary.used_manifest_bytes,
      stagingRetentionMs: staging.summary.retention_ms,
      stagingCleanupOnStartup: staging.summary.cleanup_on_startup,
      stagingRecordCount: staging.records.length,
    },
    runtime: {
      scanConcurrency: config.runtime.scan_concurrency,
      probeConcurrency: config.runtime.probe_concurrency,
      metadataConcurrency: config.runtime.metadata_concurrency,
      webhookConcurrency: config.runtime.webhook_concurrency,
    },
  }
}

function mapUser(user: AdminAccessUserRecord, activeUserIds: Set<string>): AdminUserReadModel {
  const role = mapUserRole(user.roles)
  const status: AdminUserUiStatus =
    user.status === "disabled" ? "disabled" : activeUserIds.has(user.principal_id) ? "online" : "offline"

  return {
    id: user.user_id,
    name: user.display_name,
    username: user.username,
    email: null,
    role,
    avatar: null,
    status,
    lastActive: status === "online" ? "在线" : status === "disabled" ? "账户已禁用" : "离线",
    createdAt: formatMsDate(user.created_at_ms),
    libraryAccess: role === "admin" ? ["all"] : [],
    settings: {
      canDownload: role !== "guest",
      canDelete: role === "admin",
      canManageUsers: role === "admin",
      maxBitrate: null,
      transcodePolicy: "auto",
      maxStreams: null,
      remoteAccess: true,
    },
    stats: {
      totalPlays: 0,
      totalWatchTime: "0 小时",
      lastLogin: "live",
    },
  }
}

function mapJobToTask(job: AdminJobListItem): AdminScheduledTaskReadModel {
  const taskType = mapJobKind(job.kind)
  const status = mapJobStatus(job.status)

  return {
    id: job.id,
    name: job.kind,
    description: `${job.resource_class} · ${job.library_id ?? job.source_id ?? "server"}`,
    type: taskType,
    enabled: status !== "failed",
    schedule: {
      frequency: "cron",
      cron: "manual",
    },
    config: {
      targetLibraries: job.library_id ? [job.library_id] : undefined,
    },
    lastRun: job.completed_at
      ? {
          timestamp: job.completed_at,
          status: job.status === "failed" ? "failed" : "success",
          duration: "live",
          error: job.has_error ? "任务报告包含错误" : undefined,
        }
      : undefined,
    nextRun: job.status === "queued" ? job.queued_at : undefined,
    status,
    progress: status === "running" ? 50 : undefined,
  }
}

function mapEventToHistory(event: AdminOutboxEventListItem): AdminTaskHistoryReadModel {
  return {
    id: event.id,
    taskId: event.id,
    taskName: event.kind,
    taskType: "cleanup",
    timestamp: event.occurred_at,
    status: event.has_error ? "failed" : "success",
    duration: "live",
    itemsProcessed: event.attempts,
    error: event.has_error ? "事件投递包含错误" : undefined,
  }
}

function mapEventToLog(event: AdminOutboxEventListItem): AdminLogEntryReadModel {
  return {
    id: event.id,
    timestamp: event.occurred_at,
    level: event.has_error ? "error" : event.attempts > 0 ? "warn" : "info",
    source: event.kind.includes("playback")
      ? "playback"
      : event.kind.includes("auth")
        ? "auth"
        : event.kind.includes("scan")
          ? "scanner"
          : "server",
    message: `${event.kind} · ${event.status}`,
    details: event.has_payload ? "payload redacted by Admin API" : undefined,
    userId: undefined,
    requestId: event.id,
  }
}

function mapUserRole(roles: AdminUserRole[]): AdminUserUiRole {
  if (roles.includes("administrator")) {
    return "admin"
  }

  if (roles.includes("viewer")) {
    return "user"
  }

  return roles.length > 0 ? "user" : "guest"
}

function mapJobKind(kind: string): AdminTaskType {
  const normalized = kind.toLowerCase()
  if (normalized.includes("metadata")) return "metadata"
  if (normalized.includes("backup")) return "backup"
  if (normalized.includes("cleanup")) return "cleanup"
  if (normalized.includes("subtitle")) return "subtitle"
  if (normalized.includes("thumbnail") || normalized.includes("image")) return "thumbnail"
  if (normalized.includes("optimize")) return "optimize"
  if (normalized.includes("update")) return "update"
  return "scan"
}

function mapJobStatus(status: string): AdminTaskStatus {
  switch (status) {
    case "running":
      return "running"
    case "failed":
      return "failed"
    case "completed":
    case "succeeded":
      return "success"
    case "queued":
      return "scheduled"
    default:
      return "idle"
  }
}

function normalizeLibraryKind(value: string): AdminLibraryKind {
  const normalized = value.toLowerCase()
  if (normalized.includes("movie") || normalized.includes("film")) return "movie"
  if (normalized.includes("series") || normalized.includes("tv")) return "tv"
  if (normalized.includes("anime")) return "anime"
  if (normalized.includes("music")) return "music"
  if (normalized.includes("photo") || normalized.includes("image")) return "photo"
  if (normalized.includes("documentary")) return "documentary"
  if (normalized.includes("personal")) return "personal"
  return "unknown"
}

function firstMetadataLanguage(config: AdminServerConfigDiagnosticsResponse) {
  return config.metadata.providers.find((provider) => provider.language)?.language ?? "zh-CN"
}

function fixtureLibrary(
  id: string,
  name: string,
  type: AdminLibraryKind,
  path: string,
  itemCount: number,
  totalSize: string,
  lastScanned: string,
): AdminLibraryReadModel {
  return {
    id,
    name,
    type,
    paths: [{ path, available: true }],
    itemCount,
    totalSize,
    lastScanned,
    scanStatus: "idle",
    settings: {
      autoScan: true,
      scanInterval: 6,
      useNfo: true,
      downloadArt: true,
      metadataLanguage: type === "anime" ? "ja" : "zh-CN",
    },
  }
}

function fixtureUser(
  id: string,
  name: string,
  username: string,
  role: AdminUserUiRole,
  status: AdminUserUiStatus,
  libraryAccess: string[],
  lastActive: string,
): AdminUserReadModel {
  return {
    id,
    name,
    username,
    email: username === "guest" ? null : `${username}@example.com`,
    role,
    avatar: null,
    status,
    lastActive,
    createdAt: "2024-01-01",
    libraryAccess,
    settings: {
      canDownload: role !== "guest",
      canDelete: role === "admin",
      canManageUsers: role === "admin",
      maxBitrate: role === "admin" ? null : 20_000,
      transcodePolicy: "auto",
      maxStreams: role === "guest" ? 1 : null,
      remoteAccess: role !== "guest",
    },
    stats: {
      totalPlays: role === "admin" ? 1234 : 0,
      totalWatchTime: role === "admin" ? "156 小时" : "0 小时",
      lastLogin: status === "offline" ? "从未" : "2024-03-15 14:30",
    },
  }
}

function fixtureSession(
  id: string,
  userId: string,
  userName: string,
  device: string,
  deviceType: string,
  lastActivity: string,
  current: boolean,
): AdminActiveSessionReadModel {
  return {
    id,
    userId,
    userName,
    device,
    deviceType,
    ip: "192.168.1.100",
    location: "本地网络",
    lastActivity,
    startTime: "2024-03-15 14:30",
    current,
  }
}

function fixtureTask(
  id: string,
  name: string,
  description: string,
  type: AdminTaskType,
  status: AdminTaskStatus,
  enabled: boolean,
): AdminScheduledTaskReadModel {
  return {
    id,
    name,
    description,
    type,
    enabled,
    schedule: { frequency: "daily", time: "03:00" },
    config: {
      targetLibraries: ["1", "2", "3"],
      scanNewOnly: true,
      metadataLanguage: "zh-CN",
      downloadImages: true,
    },
    lastRun: {
      timestamp: "2024-03-15T03:00:00Z",
      status: "success",
      duration: "12分34秒",
      itemsProcessed: 847,
    },
    nextRun: "2024-03-16T03:00:00Z",
    status,
  }
}

function buildFixtureHistory(): AdminTaskHistoryReadModel[] {
  return Array.from({ length: 50 }, (_, index) => ({
    id: `history-${index}`,
    taskId: `task-${(index % 7) + 1}`,
    taskName: index % 2 === 0 ? "每日媒体库扫描" : "元数据刷新",
    taskType: index % 2 === 0 ? "scan" : "metadata",
    timestamp: new Date(Date.UTC(2024, 2, 15, 3 + (index % 12), 0, 0)).toISOString(),
    status: index % 10 === 0 ? "failed" : "success",
    duration: `${index % 30}分${index % 60}秒`,
    itemsProcessed: 100 + index,
    error: index % 10 === 0 ? "任务执行失败: 权限不足或网络超时" : undefined,
  }))
}

function buildFixtureLogs(): AdminLogEntryReadModel[] {
  const rows: AdminLogEntryReadModel[] = []
  const messages: Array<[AdminLogSource, AdminLogLevel, string]> = [
    ["server", "info", "Server started on port 8096"],
    ["auth", "warn", "Failed login attempt for user 'test'"],
    ["database", "info", "Database connection established"],
    ["api", "info", "GET /api/items - 200 OK (125ms)"],
    ["playback", "info", "Stream started: Movie 'Dune' (1080p)"],
    ["scanner", "error", "Error scanning: Permission denied"],
  ]

  for (let index = 0; index < 120; index += 1) {
    const [source, level, message] = messages[index % messages.length]
    rows.push({
      id: `log-${index}`,
      timestamp: new Date(Date.UTC(2024, 2, 15, 12, index % 60, 0)).toISOString(),
      level,
      source,
      message,
      details: level === "error" ? `Stack trace or additional details for log ${index}` : undefined,
      requestId: `req-${index}`,
    })
  }

  return rows.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime())
}

function formatMsDate(ms: number) {
  return new Date(ms).toISOString().slice(0, 10)
}

function isoFromMs(ms: number) {
  return new Date(ms).toISOString()
}
