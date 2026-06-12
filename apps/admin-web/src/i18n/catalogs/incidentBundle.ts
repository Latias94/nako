export const enIncidentBundleMessages = {
  "incidentBundle.title": "Incident Bundle",
  "incidentBundle.kicker": "Diagnostics",
  "incidentBundle.description":
    "Read-only JSON support artifact assembled from redacted Admin diagnostics.",
  "incidentBundle.actions": "Incident Bundle actions",
  "incidentBundle.refresh": "Refresh",
  "incidentBundle.copy": "Copy JSON",
  "incidentBundle.download": "Download JSON",
  "incidentBundle.copySuccess": "Redacted JSON copied to clipboard.",
  "incidentBundle.copyFailure":
    "Clipboard is unavailable. Download the JSON instead.",
  "incidentBundle.downloadReady": "Redacted JSON download prepared.",
  "incidentBundle.fallback":
    "{error}. Showing deterministic incident bundle fallback data.",
  "incidentBundle.loading": "Loading Incident Bundle",
  "incidentBundle.dataSourceUnavailable":
    "Incident Bundle data source is unavailable",
  "incidentBundle.boolean.yes": "Yes",
  "incidentBundle.boolean.no": "No",
  "incidentBundle.none": "none",
  "incidentBundle.summary.title": "Section status",
  "incidentBundle.summary.description":
    "Fast triage view for bundle sections, readiness, queue pressure, and redaction coverage.",
  "incidentBundle.summary.artifactDetail":
    "JSON-only support artifact; no archive, upload, or unbounded logs.",
  "incidentBundle.summary.overviewDetail":
    "Readiness {readiness}; {failed} failed runtime job(s).",
  "incidentBundle.summary.systemDetail":
    "{libraries} configured library/libraries, {providers} metadata provider(s).",
  "incidentBundle.summary.networkDetail":
    "{exposure} exposure; {origins} allowed origin(s), {tunnels} tunnel provider(s).",
  "incidentBundle.summary.playbackDetail":
    "FFmpeg {ffmpeg}; {available}/{total} GPU capability available.",
  "incidentBundle.summary.storageDetail":
    "{records} staging record(s); repair action {action}.",
  "incidentBundle.summary.jobsDetail":
    "{groups} queue pressure group(s); {failed} failed group(s).",
  "incidentBundle.summary.jobsStatus": "{count} group(s)",
  "incidentBundle.summary.redactionDetail":
    "{families}/10 sensitive field families redacted.",
  "incidentBundle.artifact.title": "Artifact summary",
  "incidentBundle.artifact.description":
    "JSON-only bundle metadata. Archive generation, upload transport, and unbounded logs stay out of this slice.",
  "incidentBundle.artifact.generatedAt": "Generated at",
  "incidentBundle.artifact.zip": "Zip archive included",
  "incidentBundle.artifact.upload": "Upload transport included",
  "incidentBundle.artifact.logs": "Unbounded logs included",
  "incidentBundle.overview.title": "Overview",
  "incidentBundle.overview.description":
    "Server readiness and operator pressure from the Overview read model.",
  "incidentBundle.overview.server": "Server status",
  "incidentBundle.overview.readiness": "Operator readiness",
  "incidentBundle.overview.storage": "Storage backends",
  "incidentBundle.overview.storageValue": "{ready}/{total} ready",
  "incidentBundle.overview.failedJobs": "Failed jobs",
  "incidentBundle.overview.sourceHash": "Source hash coverage",
  "incidentBundle.overview.sourceHashValue": "{fingerprinted}/{total}",
  "incidentBundle.system.title": "System",
  "incidentBundle.system.description":
    "Redacted configuration posture without raw roots, URLs, or secret values.",
  "incidentBundle.system.authEnabled": "Auth enabled",
  "incidentBundle.system.authDisabled": "Auth disabled",
  "incidentBundle.system.database": "Database",
  "incidentBundle.system.libraries": "Configured libraries",
  "incidentBundle.system.runtime": "Runtime concurrency",
  "incidentBundle.system.runtimeValue":
    "scan {scan}, probe {probe}, metadata {metadata}",
  "incidentBundle.system.providers": "Metadata providers",
  "incidentBundle.network.title": "Network",
  "incidentBundle.network.description":
    "Endpoint posture with host fingerprints and counts only.",
  "incidentBundle.network.exposure": "Exposure mode",
  "incidentBundle.network.endpoint": "External endpoint",
  "incidentBundle.network.endpointConfigured": "{scheme} endpoint configured",
  "incidentBundle.network.endpointMissing": "not configured",
  "incidentBundle.network.trustedProxy": "Trusted proxy",
  "incidentBundle.network.trustedProxyValue": "{count} source(s)",
  "incidentBundle.network.origins": "Allowed origins",
  "incidentBundle.network.originsValue": "{count} origin(s)",
  "incidentBundle.network.tunnels": "Tunnel providers",
  "incidentBundle.playback.title": "Playback",
  "incidentBundle.playback.description":
    "Runtime readiness and support evidence without command lines or source locators.",
  "incidentBundle.playback.ffmpeg": "FFmpeg probe",
  "incidentBundle.playback.hardware": "Hardware capabilities",
  "incidentBundle.playback.hardwareValue":
    "{available}/{total} GPU capability available",
  "incidentBundle.playback.supportSubject": "Support subject",
  "incidentBundle.playback.failure": "Failure category",
  "incidentBundle.playback.redaction": "Support evidence redaction",
  "incidentBundle.storage.title": "Storage",
  "incidentBundle.storage.description":
    "Staging pressure and VFS repair action plan without raw backend identity.",
  "incidentBundle.storage.stagingBytes": "Staging bytes",
  "incidentBundle.storage.stagingBytesValue": "{used}/{max}",
  "incidentBundle.storage.records": "Staging records",
  "incidentBundle.storage.vfsObjects": "VFS cache objects",
  "incidentBundle.storage.repairAction": "Repair action",
  "incidentBundle.storage.repairMessage": "Safe repair message",
  "incidentBundle.jobs.title": "Jobs",
  "incidentBundle.jobs.description":
    "Durable queue pressure summaries without raw input, summary, or error payloads.",
  "incidentBundle.jobs.pressureCount": "{count} pressure group(s)",
  "incidentBundle.jobs.queuePressureDetail":
    "{count} {status} in {resource}; {claimable} claimable, {delayed} delayed retries",
  "incidentBundle.jobs.empty": "No queue pressure groups are visible.",
  "incidentBundle.redaction.title": "Redaction summary",
  "incidentBundle.redaction.description":
    "Sensitive field families excluded from the rendered and serialized artifact.",
  "incidentBundle.redaction.rawPaths": "Raw paths",
  "incidentBundle.redaction.locators": "Locators",
  "incidentBundle.redaction.tokens": "Tokens",
  "incidentBundle.redaction.credentials": "Credentials",
  "incidentBundle.redaction.ffmpeg": "FFmpeg command lines",
  "incidentBundle.redaction.providerPayloads": "Provider payloads",
  "incidentBundle.redaction.backendUrls": "Backend URLs",
  "incidentBundle.redaction.queryStrings": "Query strings",
  "incidentBundle.redaction.rawJobPayloads": "Raw job payloads",
  "incidentBundle.redaction.unboundedLogs": "Unbounded logs",
} as const;

export type IncidentBundleMessageId = keyof typeof enIncidentBundleMessages;

export const zhHansIncidentBundleMessages = {
  "incidentBundle.title": "Incident Bundle",
  "incidentBundle.kicker": "诊断",
  "incidentBundle.description": "从已脱敏 Admin 诊断汇总的只读 JSON 支持工件。",
  "incidentBundle.actions": "Incident Bundle 操作",
  "incidentBundle.refresh": "刷新",
  "incidentBundle.copy": "复制 JSON",
  "incidentBundle.download": "下载 JSON",
  "incidentBundle.copySuccess": "已复制脱敏 JSON 到剪贴板。",
  "incidentBundle.copyFailure": "剪贴板不可用。请改用 JSON 下载。",
  "incidentBundle.downloadReady": "已准备脱敏 JSON 下载。",
  "incidentBundle.fallback":
    "{error}。正在显示确定性 incident bundle 回退数据。",
  "incidentBundle.loading": "正在加载 Incident Bundle",
  "incidentBundle.dataSourceUnavailable": "Incident Bundle 数据源不可用",
  "incidentBundle.boolean.yes": "是",
  "incidentBundle.boolean.no": "否",
  "incidentBundle.none": "无",
  "incidentBundle.summary.title": "区块状态",
  "incidentBundle.summary.description":
    "用于快速排障的 bundle 区块、readiness、队列压力和脱敏覆盖摘要。",
  "incidentBundle.summary.artifactDetail":
    "JSON-only 支持工件；不包含压缩包、上传传输或无界日志。",
  "incidentBundle.summary.overviewDetail":
    "Readiness {readiness}；{failed} 个失败运行时任务。",
  "incidentBundle.summary.systemDetail":
    "{libraries} 个已配置媒体库，{providers} 个 metadata provider。",
  "incidentBundle.summary.networkDetail":
    "{exposure} 暴露模式；{origins} 个 allowed origin，{tunnels} 个 tunnel provider。",
  "incidentBundle.summary.playbackDetail":
    "FFmpeg {ffmpeg}；{available}/{total} 个 GPU 能力可用。",
  "incidentBundle.summary.storageDetail":
    "{records} 个 staging 记录；repair action {action}。",
  "incidentBundle.summary.jobsDetail":
    "{groups} 个队列压力分组；{failed} 个失败分组。",
  "incidentBundle.summary.jobsStatus": "{count} 个分组",
  "incidentBundle.summary.redactionDetail":
    "10 类敏感字段族已脱敏 {families} 类。",
  "incidentBundle.artifact.title": "工件摘要",
  "incidentBundle.artifact.description":
    "JSON-only bundle 元数据。压缩包生成、上传传输和无界日志不属于此切片。",
  "incidentBundle.artifact.generatedAt": "生成时间",
  "incidentBundle.artifact.zip": "包含 zip archive",
  "incidentBundle.artifact.upload": "包含上传传输",
  "incidentBundle.artifact.logs": "包含无界日志",
  "incidentBundle.overview.title": "总览",
  "incidentBundle.overview.description":
    "来自 Overview 读模型的服务器 readiness 和 operator 压力。",
  "incidentBundle.overview.server": "服务器状态",
  "incidentBundle.overview.readiness": "Operator readiness",
  "incidentBundle.overview.storage": "存储后端",
  "incidentBundle.overview.storageValue": "{ready}/{total} 就绪",
  "incidentBundle.overview.failedJobs": "失败任务",
  "incidentBundle.overview.sourceHash": "Source hash 覆盖率",
  "incidentBundle.overview.sourceHashValue": "{fingerprinted}/{total}",
  "incidentBundle.system.title": "系统",
  "incidentBundle.system.description":
    "已脱敏配置姿态，不包含原始根路径、URL 或 secret 值。",
  "incidentBundle.system.authEnabled": "Auth 已启用",
  "incidentBundle.system.authDisabled": "Auth 已禁用",
  "incidentBundle.system.database": "数据库",
  "incidentBundle.system.libraries": "已配置媒体库",
  "incidentBundle.system.runtime": "运行时并发",
  "incidentBundle.system.runtimeValue":
    "scan {scan}, probe {probe}, metadata {metadata}",
  "incidentBundle.system.providers": "Metadata Provider",
  "incidentBundle.network.title": "网络",
  "incidentBundle.network.description":
    "Endpoint posture 仅显示 host fingerprint 和计数。",
  "incidentBundle.network.exposure": "暴露模式",
  "incidentBundle.network.endpoint": "外部 endpoint",
  "incidentBundle.network.endpointConfigured": "{scheme} endpoint 已配置",
  "incidentBundle.network.endpointMissing": "未配置",
  "incidentBundle.network.trustedProxy": "Trusted proxy",
  "incidentBundle.network.trustedProxyValue": "{count} 个来源",
  "incidentBundle.network.origins": "Allowed origins",
  "incidentBundle.network.originsValue": "{count} 个 origin",
  "incidentBundle.network.tunnels": "Tunnel providers",
  "incidentBundle.playback.title": "播放",
  "incidentBundle.playback.description":
    "Runtime readiness 和支持证据，不显示命令行或 Source locator。",
  "incidentBundle.playback.ffmpeg": "FFmpeg probe",
  "incidentBundle.playback.hardware": "硬件能力",
  "incidentBundle.playback.hardwareValue":
    "{available}/{total} 个 GPU 能力可用",
  "incidentBundle.playback.supportSubject": "支持主题",
  "incidentBundle.playback.failure": "失败分类",
  "incidentBundle.playback.redaction": "支持证据脱敏",
  "incidentBundle.storage.title": "存储",
  "incidentBundle.storage.description":
    "Staging 压力和 VFS repair action plan，不包含原始后端身份。",
  "incidentBundle.storage.stagingBytes": "Staging bytes",
  "incidentBundle.storage.stagingBytesValue": "{used}/{max}",
  "incidentBundle.storage.records": "Staging 记录",
  "incidentBundle.storage.vfsObjects": "VFS cache 对象",
  "incidentBundle.storage.repairAction": "Repair action",
  "incidentBundle.storage.repairMessage": "安全 repair message",
  "incidentBundle.jobs.title": "任务",
  "incidentBundle.jobs.description":
    "Durable queue pressure 摘要，不包含原始 input、summary 或 error payload。",
  "incidentBundle.jobs.pressureCount": "{count} 个压力分组",
  "incidentBundle.jobs.queuePressureDetail":
    "{resource} 中有 {count} 个 {status}；{claimable} 个可领取，{delayed} 个 delayed retry",
  "incidentBundle.jobs.empty": "没有可见的 queue pressure 分组。",
  "incidentBundle.redaction.title": "脱敏摘要",
  "incidentBundle.redaction.description":
    "从渲染和序列化工件中排除的敏感字段族。",
  "incidentBundle.redaction.rawPaths": "原始路径",
  "incidentBundle.redaction.locators": "Locator",
  "incidentBundle.redaction.tokens": "Token",
  "incidentBundle.redaction.credentials": "凭据",
  "incidentBundle.redaction.ffmpeg": "FFmpeg 命令行",
  "incidentBundle.redaction.providerPayloads": "Provider payload",
  "incidentBundle.redaction.backendUrls": "Backend URL",
  "incidentBundle.redaction.queryStrings": "Query string",
  "incidentBundle.redaction.rawJobPayloads": "原始 job payload",
  "incidentBundle.redaction.unboundedLogs": "无界日志",
} satisfies Record<IncidentBundleMessageId, string>;

export const incidentBundleMessageCatalogs = {
  "en-US": enIncidentBundleMessages,
  "zh-Hans": zhHansIncidentBundleMessages,
} as const;
