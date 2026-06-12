export const enOverviewMessages = {
  "overview.title": "Overview",
  "overview.kicker": "Operations",
  "overview.description":
    "Server health, runtime counters, Media Library storage, and provider availability from the Admin overview read model.",
  "overview.refresh": "Refresh",
  "overview.fallback": "{error}. Showing deterministic mock fallback data.",
  "overview.loading": "Loading Overview",
  "overview.dataSourceUnavailable": "Overview route data source is unavailable",
  "overview.column.mediaLibrary": "Media Library",
  "overview.column.backend": "Backend",
  "overview.column.status": "Status",
  "overview.column.provider": "Provider",
  "overview.metric.serverStatus.label": "Server status",
  "overview.metric.serverStatus.healthy": "Healthy",
  "overview.metric.serverStatus.degraded": "Degraded",
  "overview.metric.storage.label": "Storage backends",
  "overview.metric.storage.value": "{ready}/{total} ready",
  "overview.metric.storage.ready": "Ready",
  "overview.metric.storage.degraded": "Degraded",
  "overview.metric.storage.unavailable": "Unavailable",
  "overview.metric.activeTasks.label": "Active tasks",
  "overview.metric.activeTasks.badge": "Running",
  "overview.metric.failedJobs.label": "Failed jobs",
  "overview.metric.failedJobs.attention": "Attention",
  "overview.metric.failedJobs.clear": "Clear",
  "overview.metric.configuredLibraries.label": "Configured libraries",
  "overview.metric.configuredLibraries.badge": "Configured",
  "overview.metric.recoveredJobs.label": "Recovered jobs",
  "overview.metric.recoveredJobs.badge": "Recovered",
  "overview.operatorReadiness.title": "Product-Operator readiness",
  "overview.operatorReadiness.description": "Overall readiness is {status}",
  "overview.operatorReadiness.sourceReason": "Reason code {reason}",
  "overview.operatorReadiness.sourceReason.redacted": "redacted",
  "overview.operatorReadiness.action": "Action route {route}",
  "overview.operatorReadiness.area.setup": "Setup",
  "overview.operatorReadiness.area.mediaLibraryScan": "Media Library scan",
  "overview.operatorReadiness.area.playback": "Playback",
  "overview.operatorReadiness.area.storage": "Storage",
  "overview.operatorReadiness.area.network": "Network",
  "overview.operatorReadiness.area.backup": "Backup",
  "overview.operatorReadiness.status.ready": "Ready",
  "overview.operatorReadiness.status.degraded": "Degraded",
  "overview.operatorReadiness.status.unavailable": "Unavailable",
  "overview.operatorReadiness.reason.authConfigured": "Auth is configured",
  "overview.operatorReadiness.reason.authTokenReferenceMissing":
    "Auth token reference is missing",
  "overview.operatorReadiness.reason.authDisabledLocalOnly":
    "Auth is disabled for local-only access",
  "overview.operatorReadiness.reason.authDisabledRemoteExposure":
    "Auth is disabled for remote exposure",
  "overview.operatorReadiness.reason.mediaLibraryConfigured":
    "Media Library configuration is present",
  "overview.operatorReadiness.reason.noMediaLibraryConfigured":
    "No Media Library is configured",
  "overview.operatorReadiness.reason.scanWorkPending":
    "{count} scan or fingerprint jobs are pending",
  "overview.operatorReadiness.reason.scanRepairPressure":
    "{count} scan or job failures need review",
  "overview.operatorReadiness.reason.watchFolderRuntimeCoverageGap":
    "{count} watch-folder runtime needs configuration",
  "overview.operatorReadiness.reason.playbackReady":
    "Playback runtime is ready",
  "overview.operatorReadiness.reason.playbackDegraded":
    "Playback runtime is degraded",
  "overview.operatorReadiness.reason.playbackUnavailable":
    "Playback runtime is unavailable",
  "overview.operatorReadiness.reason.storageReady":
    "Storage backends are ready",
  "overview.operatorReadiness.reason.storageDegraded":
    "{count} storage backends need review",
  "overview.operatorReadiness.reason.storageUnavailable":
    "{count} storage backends are unavailable",
  "overview.operatorReadiness.reason.vfsCacheRepairPressure":
    "{count} VFS cache repair target needs review",
  "overview.operatorReadiness.reason.networkReady": "Network posture is ready",
  "overview.operatorReadiness.reason.networkDegraded":
    "{count} network checks need review",
  "overview.operatorReadiness.reason.networkUnavailable":
    "{count} network checks are unavailable",
  "overview.operatorReadiness.reason.backupRunbookAvailable":
    "Backup and restore runbook is available",
  "overview.operatorReadiness.reason.backupNeedsDurableDatabase":
    "Database is ephemeral; use a durable database for backup",
  "overview.operatorReadiness.action.jobs": "Admin Jobs",
  "overview.operatorReadiness.action.playbackRuntime": "Playback runtime",
  "overview.operatorReadiness.action.storageRepair": "Storage repair targets",
  "overview.operatorReadiness.action.systemConfig": "System config",
  "overview.storage.description":
    "{ready} ready, {degraded} degraded, {unavailable} unavailable",
  "overview.metadata.title": "Metadata providers",
  "overview.metadata.description":
    "{available} available, {disabled} disabled, {unavailable} unavailable",
  "overview.sourceFingerprint.title": "Source fingerprint hash",
  "overview.sourceFingerprint.description":
    "{fingerprinted}/{total} sources fingerprinted, {contentHash} with content-hash evidence",
  "overview.sourceFingerprint.coverage.label": "Fingerprint coverage",
  "overview.sourceFingerprint.coverage.value": "{fingerprinted}/{total}",
  "overview.sourceFingerprint.coverage.detail":
    "{contentHash} content-hash evidence",
  "overview.sourceFingerprint.queue.label": "Queued hash jobs",
  "overview.sourceFingerprint.queue.detail":
    "{claimable} claimable, {delayed} delayed",
  "overview.sourceFingerprint.failures.label": "Failed hash jobs",
  "overview.sourceFingerprint.failures.detail":
    "{running} running, {succeeded} succeeded",
  "overview.sourceFingerprint.retry.label": "Next retry",
  "overview.sourceFingerprint.retry.detail": "Oldest queued {oldest}",
  "overview.sourceFingerprint.timestamp.none": "None",
} as const;

export type OverviewMessageId = keyof typeof enOverviewMessages;

export const zhHansOverviewMessages = {
  "overview.title": "总览",
  "overview.kicker": "运维",
  "overview.description":
    "来自 Admin 总览读模型的服务器健康、运行时计数、媒体库存储和 Provider 可用性。",
  "overview.refresh": "刷新",
  "overview.fallback": "{error}。正在显示确定性 mock 回退数据。",
  "overview.loading": "正在加载总览",
  "overview.dataSourceUnavailable": "总览路由数据源不可用",
  "overview.column.mediaLibrary": "媒体库",
  "overview.column.backend": "后端",
  "overview.column.status": "状态",
  "overview.column.provider": "Provider",
  "overview.metric.serverStatus.label": "服务器状态",
  "overview.metric.serverStatus.healthy": "健康",
  "overview.metric.serverStatus.degraded": "降级",
  "overview.metric.storage.label": "存储后端",
  "overview.metric.storage.value": "{ready}/{total} 就绪",
  "overview.metric.storage.ready": "就绪",
  "overview.metric.storage.degraded": "降级",
  "overview.metric.storage.unavailable": "不可用",
  "overview.metric.activeTasks.label": "活动任务",
  "overview.metric.activeTasks.badge": "运行中",
  "overview.metric.failedJobs.label": "失败任务",
  "overview.metric.failedJobs.attention": "需关注",
  "overview.metric.failedJobs.clear": "正常",
  "overview.metric.configuredLibraries.label": "已配置媒体库",
  "overview.metric.configuredLibraries.badge": "已配置",
  "overview.metric.recoveredJobs.label": "已恢复任务",
  "overview.metric.recoveredJobs.badge": "已恢复",
  "overview.operatorReadiness.title": "Product-Operator readiness",
  "overview.operatorReadiness.description": "整体 readiness 为 {status}",
  "overview.operatorReadiness.sourceReason": "原因代码 {reason}",
  "overview.operatorReadiness.sourceReason.redacted": "已隐藏",
  "overview.operatorReadiness.action": "Action route {route}",
  "overview.operatorReadiness.area.setup": "初始化",
  "overview.operatorReadiness.area.mediaLibraryScan": "媒体库扫描",
  "overview.operatorReadiness.area.playback": "播放",
  "overview.operatorReadiness.area.storage": "存储",
  "overview.operatorReadiness.area.network": "网络",
  "overview.operatorReadiness.area.backup": "备份",
  "overview.operatorReadiness.status.ready": "就绪",
  "overview.operatorReadiness.status.degraded": "降级",
  "overview.operatorReadiness.status.unavailable": "不可用",
  "overview.operatorReadiness.reason.authConfigured": "Auth 已配置",
  "overview.operatorReadiness.reason.authTokenReferenceMissing":
    "缺少 Auth token 引用",
  "overview.operatorReadiness.reason.authDisabledLocalOnly":
    "本地访问模式下 Auth 已关闭",
  "overview.operatorReadiness.reason.authDisabledRemoteExposure":
    "远程暴露模式下 Auth 已关闭",
  "overview.operatorReadiness.reason.mediaLibraryConfigured":
    "媒体库配置已存在",
  "overview.operatorReadiness.reason.noMediaLibraryConfigured":
    "尚未配置媒体库",
  "overview.operatorReadiness.reason.scanWorkPending":
    "{count} 个扫描或 fingerprint 任务待处理",
  "overview.operatorReadiness.reason.scanRepairPressure":
    "{count} 个扫描或任务失败需要处理",
  "overview.operatorReadiness.reason.watchFolderRuntimeCoverageGap":
    "{count} 个 watch-folder 运行时需要配置",
  "overview.operatorReadiness.reason.playbackReady": "播放运行时已就绪",
  "overview.operatorReadiness.reason.playbackDegraded": "播放运行时降级",
  "overview.operatorReadiness.reason.playbackUnavailable": "播放运行时不可用",
  "overview.operatorReadiness.reason.storageReady": "存储后端已就绪",
  "overview.operatorReadiness.reason.storageDegraded":
    "{count} 个存储后端需要检查",
  "overview.operatorReadiness.reason.storageUnavailable":
    "{count} 个存储后端不可用",
  "overview.operatorReadiness.reason.vfsCacheRepairPressure":
    "{count} 个 VFS cache repair 目标需要处理",
  "overview.operatorReadiness.reason.networkReady": "网络状态已就绪",
  "overview.operatorReadiness.reason.networkDegraded":
    "{count} 个网络检查需要处理",
  "overview.operatorReadiness.reason.networkUnavailable":
    "{count} 个网络检查不可用",
  "overview.operatorReadiness.reason.backupRunbookAvailable":
    "备份和恢复 runbook 可用",
  "overview.operatorReadiness.reason.backupNeedsDurableDatabase":
    "数据库是临时的；请使用可持久化数据库以支持备份",
  "overview.operatorReadiness.action.jobs": "Admin Jobs",
  "overview.operatorReadiness.action.playbackRuntime": "Playback runtime",
  "overview.operatorReadiness.action.storageRepair": "Storage repair targets",
  "overview.operatorReadiness.action.systemConfig": "System config",
  "overview.storage.description":
    "{ready} 就绪，{degraded} 降级，{unavailable} 不可用",
  "overview.metadata.title": "Metadata Provider",
  "overview.metadata.description":
    "{available} 可用，{disabled} 禁用，{unavailable} 不可用",
  "overview.sourceFingerprint.title": "Source fingerprint hash",
  "overview.sourceFingerprint.description":
    "{fingerprinted}/{total} 个 Source 已有 fingerprint，{contentHash} 个包含 content-hash evidence",
  "overview.sourceFingerprint.coverage.label": "Fingerprint 覆盖",
  "overview.sourceFingerprint.coverage.value": "{fingerprinted}/{total}",
  "overview.sourceFingerprint.coverage.detail":
    "{contentHash} 个 content-hash evidence",
  "overview.sourceFingerprint.queue.label": "排队 Hash 任务",
  "overview.sourceFingerprint.queue.detail":
    "{claimable} 个可领取，{delayed} 个延迟",
  "overview.sourceFingerprint.failures.label": "失败 Hash 任务",
  "overview.sourceFingerprint.failures.detail":
    "{running} 个运行中，{succeeded} 个已成功",
  "overview.sourceFingerprint.retry.label": "下次重试",
  "overview.sourceFingerprint.retry.detail": "最早排队 {oldest}",
  "overview.sourceFingerprint.timestamp.none": "无",
} satisfies Record<OverviewMessageId, string>;

export const overviewMessageCatalogs = {
  "en-US": enOverviewMessages,
  "zh-Hans": zhHansOverviewMessages,
} as const;
