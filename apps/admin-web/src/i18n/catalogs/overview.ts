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
  "overview.operatorReadiness.area.durableJobs": "Durable jobs",
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
  "overview.operatorReadiness.reason.durableJobsReady":
    "Durable jobs queue is clear",
  "overview.operatorReadiness.reason.durableJobsPressure":
    "{count} durable jobs need review",
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
  "overview.watchFolder.title": "Watch-folder diagnostics",
  "overview.watchFolder.description":
    "{realtime} realtime libraries, {started} started, {skipped} skipped",
  "overview.watchFolder.coverage.label": "Coverage",
  "overview.watchFolder.coverage.value": "{realtime}/{started}",
  "overview.watchFolder.coverage.detail": "{skipped} skipped",
  "overview.watchFolder.tickCoverage.label": "Tick coverage",
  "overview.watchFolder.tickCoverage.value": "{ticked}/{started}",
  "overview.watchFolder.tickCoverage.detail": "{never} never ticked",
  "overview.watchFolder.admission.label": "Scan admission",
  "overview.watchFolder.admission.detail":
    "{reused} reused, {notAdmitted} not admitted",
  "overview.watchFolder.intake.label": "Intake pressure",
  "overview.watchFolder.intake.detail":
    "{observed} observed, {suppressed} suppressed",
  "overview.watchFolder.neverTicked": "Never ticked",
  "overview.watchFolder.none": "None",
  "overview.watchFolder.yes": "Yes",
  "overview.watchFolder.no": "No",
  "overview.watchFolder.root": "{scheme}: {root}",
  "overview.watchFolder.safeReason": "Safe reason {reason}",
  "overview.watchFolder.scanJob": "Scan job {job}",
  "overview.watchFolder.reusedBackoff": "Backoff {backoff}, reused {reused}",
  "overview.watchFolder.counts":
    "{ready} ready, {inspecting} inspecting, {suppressed} suppressed, {failures} failures",
  "overview.watchFolder.column.runtime": "Runtime",
  "overview.watchFolder.column.lastTick": "Last tick",
  "overview.watchFolder.column.admission": "Admission",
  "overview.watchFolder.coverageStatus.disabled": "Disabled",
  "overview.watchFolder.coverageStatus.missingRoot": "Missing root",
  "overview.watchFolder.coverageStatus.started": "Started",
  "overview.watchFolder.coverageStatus.unsupportedRoot": "Unsupported root",
  "overview.watchFolder.admissionStatus.enqueued": "Enqueued",
  "overview.watchFolder.admissionStatus.notAdmitted": "Not admitted",
  "overview.watchFolder.admissionStatus.reusedQueued": "Reused queued",
  "overview.watchFolder.admissionStatus.reusedRunning": "Reused running",
  "overview.watchFolder.enqueueReason.blockedCandidates": "Blocked candidates",
  "overview.watchFolder.enqueueReason.discoveryFailures": "Discovery failures",
  "overview.watchFolder.enqueueReason.newStableCandidates": "New stable candidates",
  "overview.watchFolder.enqueueReason.noNewStableCandidates":
    "No new stable candidates",
  "overview.watchFolder.enqueueReason.suppressedCandidates":
    "Suppressed candidates",
  "overview.watchFolder.enqueueReason.waitingForStability":
    "Waiting for stability",
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
  "overview.operatorReadiness.area.durableJobs": "持久任务",
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
  "overview.operatorReadiness.reason.durableJobsReady": "持久任务队列正常",
  "overview.operatorReadiness.reason.durableJobsPressure":
    "{count} 个持久任务需要处理",
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
  "overview.watchFolder.title": "Watch-folder 诊断",
  "overview.watchFolder.description":
    "{realtime} 个实时媒体库，{started} 个已启动，{skipped} 个已跳过",
  "overview.watchFolder.coverage.label": "覆盖",
  "overview.watchFolder.coverage.value": "{realtime}/{started}",
  "overview.watchFolder.coverage.detail": "{skipped} 个已跳过",
  "overview.watchFolder.tickCoverage.label": "Tick 覆盖",
  "overview.watchFolder.tickCoverage.value": "{ticked}/{started}",
  "overview.watchFolder.tickCoverage.detail": "{never} 个从未 tick",
  "overview.watchFolder.admission.label": "扫描准入",
  "overview.watchFolder.admission.detail":
    "{reused} 个复用，{notAdmitted} 个未准入",
  "overview.watchFolder.intake.label": "Intake 压力",
  "overview.watchFolder.intake.detail":
    "{observed} 个已观测，{suppressed} 个已抑制",
  "overview.watchFolder.neverTicked": "从未 tick",
  "overview.watchFolder.none": "无",
  "overview.watchFolder.yes": "是",
  "overview.watchFolder.no": "否",
  "overview.watchFolder.root": "{scheme}：{root}",
  "overview.watchFolder.safeReason": "安全原因 {reason}",
  "overview.watchFolder.scanJob": "扫描任务 {job}",
  "overview.watchFolder.reusedBackoff": "退避 {backoff}，复用 {reused}",
  "overview.watchFolder.counts":
    "{ready} 个就绪，{inspecting} 个检查中，{suppressed} 个已抑制，{failures} 个失败",
  "overview.watchFolder.column.runtime": "运行时",
  "overview.watchFolder.column.lastTick": "最近 tick",
  "overview.watchFolder.column.admission": "准入",
  "overview.watchFolder.coverageStatus.disabled": "已禁用",
  "overview.watchFolder.coverageStatus.missingRoot": "缺少根路径",
  "overview.watchFolder.coverageStatus.started": "已启动",
  "overview.watchFolder.coverageStatus.unsupportedRoot": "根路径不支持",
  "overview.watchFolder.admissionStatus.enqueued": "已入队",
  "overview.watchFolder.admissionStatus.notAdmitted": "未准入",
  "overview.watchFolder.admissionStatus.reusedQueued": "复用排队中",
  "overview.watchFolder.admissionStatus.reusedRunning": "复用运行中",
  "overview.watchFolder.enqueueReason.blockedCandidates": "被阻塞候选",
  "overview.watchFolder.enqueueReason.discoveryFailures": "发现失败",
  "overview.watchFolder.enqueueReason.newStableCandidates": "新稳定候选",
  "overview.watchFolder.enqueueReason.noNewStableCandidates":
    "无新稳定候选",
  "overview.watchFolder.enqueueReason.suppressedCandidates":
    "已抑制候选",
  "overview.watchFolder.enqueueReason.waitingForStability": "等待稳定",
} satisfies Record<OverviewMessageId, string>;

export const overviewMessageCatalogs = {
  "en-US": enOverviewMessages,
  "zh-Hans": zhHansOverviewMessages,
} as const;
