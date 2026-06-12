export const enLibrariesMessages = {
  "libraries.title": "Media Libraries",
  "libraries.kicker": "Library operations",
  "libraries.description":
    "Configured Media Library boundaries from redacted Admin system diagnostics.",
  "libraries.refresh": "Refresh",
  "libraries.configured.title": "Configured libraries",
  "libraries.configured.description":
    "{count} configured from Admin system config diagnostics",
  "libraries.redactionHint": "Root references stay redacted",
  "libraries.loading": "Loading Media Libraries",
  "libraries.empty": "No Media Libraries are configured.",
  "libraries.fallback": "{error}. Showing deterministic mock fallback data.",
  "libraries.dataSourceUnavailable":
    "Media Libraries route data source is unavailable",
  "libraries.column.mediaLibrary": "Media Library",
  "libraries.column.preset": "Preset",
  "libraries.column.backend": "Backend",
  "libraries.column.rootScheme": "Root Scheme",
  "libraries.column.secretReference": "Secret Reference",
  "libraries.column.runtimePolicy": "Runtime Policy",
  "libraries.manage": "Manage",
  "libraries.manageAria": "Manage {name}",
  "libraries.secret.notRequired": "Not required",
  "libraries.secret.configured": "Secret Reference configured",
  "libraries.secret.missing": "Secret Reference missing",
  "libraries.runtime.localPolicy": "local policy",
  "libraries.runtime.defaultTimeout": "default timeout",
  "libraries.runtime.defaultAttempts": "default attempts",
} as const;

export type LibrariesMessageId = keyof typeof enLibrariesMessages;

export const zhHansLibrariesMessages = {
  "libraries.title": "媒体库",
  "libraries.kicker": "媒体库运维",
  "libraries.description": "来自已脱敏 Admin 系统诊断的媒体库边界。",
  "libraries.refresh": "刷新",
  "libraries.configured.title": "已配置媒体库",
  "libraries.configured.description": "{count} 个来自 Admin 系统配置诊断",
  "libraries.redactionHint": "根引用保持脱敏",
  "libraries.loading": "正在加载媒体库",
  "libraries.empty": "尚未配置媒体库。",
  "libraries.fallback": "{error}。正在显示确定性 mock 回退数据。",
  "libraries.dataSourceUnavailable": "媒体库路由数据源不可用",
  "libraries.column.mediaLibrary": "媒体库",
  "libraries.column.preset": "预设",
  "libraries.column.backend": "后端",
  "libraries.column.rootScheme": "根 Scheme",
  "libraries.column.secretReference": "Secret Reference",
  "libraries.column.runtimePolicy": "运行策略",
  "libraries.manage": "管理",
  "libraries.manageAria": "管理 {name}",
  "libraries.secret.notRequired": "不需要",
  "libraries.secret.configured": "Secret Reference 已配置",
  "libraries.secret.missing": "Secret Reference 缺失",
  "libraries.runtime.localPolicy": "本地策略",
  "libraries.runtime.defaultTimeout": "默认超时",
  "libraries.runtime.defaultAttempts": "默认重试",
} satisfies Record<LibrariesMessageId, string>;

export const librariesMessageCatalogs = {
  "en-US": enLibrariesMessages,
  "zh-Hans": zhHansLibrariesMessages,
} as const;
