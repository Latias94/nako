export const enArtworkMaintenanceMessages = {
  "artworkMaintenance.refresh": "Refresh",
  "artworkMaintenance.description":
    "Read-only Managed Artwork lifecycle, storage drift, and remediation diagnostics without cleanup commands.",
  "artworkMaintenance.kicker": "Managed Artwork",
  "artworkMaintenance.title": "Artwork Maintenance",
  "artworkMaintenance.fallback":
    "{error}. Showing deterministic maintenance fallback data.",
  "artworkMaintenance.filters": "Artwork maintenance filters",
  "artworkMaintenance.filter.limit": "Limit",
  "artworkMaintenance.filter.limitAria":
    "Managed Artwork maintenance page limit",
  "artworkMaintenance.filter.offset": "Offset",
  "artworkMaintenance.filter.offsetAria":
    "Managed Artwork maintenance page offset",
  "artworkMaintenance.filter.cleanupOnly": "Cleanup candidates",
  "artworkMaintenance.filter.cleanupOnlyAria": "Show cleanup candidates only",
  "artworkMaintenance.filter.cleanupOnlyValue": "Cleanup candidates only",
  "artworkMaintenance.filter.fileScanLimit": "File scan limit",
  "artworkMaintenance.filter.fileScanLimitAria":
    "Managed Artwork maintenance file scan limit",
  "artworkMaintenance.filter.active": "{count} filters",
  "artworkMaintenance.clear": "Clear",
  "artworkMaintenance.loading":
    "Loading Managed Artwork maintenance diagnostics",
  "artworkMaintenance.dataSourceUnavailable":
    "Managed Artwork maintenance data source is unavailable",
  "artworkMaintenance.summary.totalArtifacts": "Total artifacts",
  "artworkMaintenance.summary.cleanupCandidates": "Cleanup candidates",
  "artworkMaintenance.summary.missingArtifacts": "Missing artifacts",
  "artworkMaintenance.summary.strayFiles": "Cleanable stray files",
  "artworkMaintenance.dryRun": "Dry-run read",
  "artworkMaintenance.liveRead": "Live read",
  "artworkMaintenance.scanTruncated": "Scan truncated",
  "artworkMaintenance.scanComplete": "Scan complete",
  "artworkMaintenance.lifecycle.title": "Artifact lifecycle",
  "artworkMaintenance.lifecycle.description":
    "{returned} artifacts, offset {offset}, limit {limit}",
  "artworkMaintenance.lifecycle.empty":
    "No Managed Artwork artifacts match the current page.",
  "artworkMaintenance.lifecycle.ingest": "ingest {ingestId}",
  "artworkMaintenance.storage.title": "Storage drift",
  "artworkMaintenance.storage.description":
    "Scanned {scanned} files with file scan limit {limit}",
  "artworkMaintenance.storage.missingTitle": "Missing DB-backed artifacts",
  "artworkMaintenance.storage.noMissing":
    "No missing DB-backed artifacts are visible.",
  "artworkMaintenance.storage.strayTitle": "Stray artifact files",
  "artworkMaintenance.storage.noStray": "No stray artifact files are visible.",
  "artworkMaintenance.remediation.title": "Remediation plan",
  "artworkMaintenance.remediation.description":
    "{missing} missing artifacts and {stray} cleanable stray files",
  "artworkMaintenance.remediation.missingTitle": "Missing artifact actions",
  "artworkMaintenance.remediation.noMissing":
    "No missing artifact actions are visible.",
  "artworkMaintenance.remediation.strayTitle": "Stray file actions",
  "artworkMaintenance.remediation.noStray":
    "No stray file actions are visible.",
  "artworkMaintenance.column.artifact": "Artifact",
  "artworkMaintenance.column.scope": "Scope",
  "artworkMaintenance.column.media": "Media",
  "artworkMaintenance.column.state": "State",
  "artworkMaintenance.column.updated": "Updated",
  "artworkMaintenance.unknownMediaType": "unknown media type",
  "artworkMaintenance.cleanupCandidate": "cleanup candidate",
  "artworkMaintenance.protected": "protected",
  "artworkMaintenance.hashPresent": "hash present",
  "artworkMaintenance.hashAbsent": "hash absent",
  "artworkMaintenance.selectedCount": "{count} selected",
  "artworkMaintenance.dimensionsUnavailable": "dimensions unavailable",
  "artworkMaintenance.sizeUnavailable": "size unavailable",
  "artworkMaintenance.stray.unrecognizedArtifact": "unrecognized artifact",
  "artworkMaintenance.stray.unknownExtension": "unknown extension",
  "artworkMaintenance.stray.inspectOnly": "inspect only",
} as const;

export type ArtworkMaintenanceMessageId =
  keyof typeof enArtworkMaintenanceMessages;

export const zhHansArtworkMaintenanceMessages = {
  "artworkMaintenance.refresh": "刷新",
  "artworkMaintenance.description":
    "只读 Managed Artwork 生命周期、storage drift 和 remediation 诊断，不开放 cleanup 命令。",
  "artworkMaintenance.kicker": "Managed Artwork",
  "artworkMaintenance.title": "Artwork 维护",
  "artworkMaintenance.fallback": "{error}。正在显示确定性维护回退数据。",
  "artworkMaintenance.filters": "Artwork 维护过滤器",
  "artworkMaintenance.filter.limit": "Limit",
  "artworkMaintenance.filter.limitAria": "Managed Artwork 维护页面 limit",
  "artworkMaintenance.filter.offset": "Offset",
  "artworkMaintenance.filter.offsetAria": "Managed Artwork 维护页面 offset",
  "artworkMaintenance.filter.cleanupOnly": "Cleanup 候选",
  "artworkMaintenance.filter.cleanupOnlyAria": "只显示 cleanup 候选",
  "artworkMaintenance.filter.cleanupOnlyValue": "只显示 cleanup 候选",
  "artworkMaintenance.filter.fileScanLimit": "File scan limit",
  "artworkMaintenance.filter.fileScanLimitAria":
    "Managed Artwork 维护 file scan limit",
  "artworkMaintenance.filter.active": "{count} 个过滤器",
  "artworkMaintenance.clear": "清除",
  "artworkMaintenance.loading": "正在加载 Managed Artwork 维护诊断",
  "artworkMaintenance.dataSourceUnavailable":
    "Managed Artwork 维护数据源不可用",
  "artworkMaintenance.summary.totalArtifacts": "Artifact 总数",
  "artworkMaintenance.summary.cleanupCandidates": "Cleanup 候选",
  "artworkMaintenance.summary.missingArtifacts": "缺失 artifact",
  "artworkMaintenance.summary.strayFiles": "可清理 stray file",
  "artworkMaintenance.dryRun": "Dry-run 读取",
  "artworkMaintenance.liveRead": "实时读取",
  "artworkMaintenance.scanTruncated": "扫描已截断",
  "artworkMaintenance.scanComplete": "扫描完成",
  "artworkMaintenance.lifecycle.title": "Artifact 生命周期",
  "artworkMaintenance.lifecycle.description":
    "返回 {returned} 个 artifact，offset {offset}，limit {limit}",
  "artworkMaintenance.lifecycle.empty":
    "当前页没有匹配的 Managed Artwork artifact。",
  "artworkMaintenance.lifecycle.ingest": "ingest {ingestId}",
  "artworkMaintenance.storage.title": "Storage drift",
  "artworkMaintenance.storage.description":
    "已扫描 {scanned} 个文件，file scan limit {limit}",
  "artworkMaintenance.storage.missingTitle": "缺失的 DB-backed artifact",
  "artworkMaintenance.storage.noMissing": "没有可见的缺失 DB-backed artifact。",
  "artworkMaintenance.storage.strayTitle": "Stray artifact file",
  "artworkMaintenance.storage.noStray": "没有可见的 stray artifact file。",
  "artworkMaintenance.remediation.title": "Remediation 计划",
  "artworkMaintenance.remediation.description":
    "{missing} 个缺失 artifact，{stray} 个可清理 stray file",
  "artworkMaintenance.remediation.missingTitle": "缺失 artifact 动作",
  "artworkMaintenance.remediation.noMissing": "没有可见的缺失 artifact 动作。",
  "artworkMaintenance.remediation.strayTitle": "Stray file 动作",
  "artworkMaintenance.remediation.noStray": "没有可见的 stray file 动作。",
  "artworkMaintenance.column.artifact": "Artifact",
  "artworkMaintenance.column.scope": "Scope",
  "artworkMaintenance.column.media": "媒体",
  "artworkMaintenance.column.state": "状态",
  "artworkMaintenance.column.updated": "更新时间",
  "artworkMaintenance.unknownMediaType": "未知媒体类型",
  "artworkMaintenance.cleanupCandidate": "cleanup 候选",
  "artworkMaintenance.protected": "受保护",
  "artworkMaintenance.hashPresent": "hash 存在",
  "artworkMaintenance.hashAbsent": "hash 缺失",
  "artworkMaintenance.selectedCount": "{count} 个已选择",
  "artworkMaintenance.dimensionsUnavailable": "尺寸不可用",
  "artworkMaintenance.sizeUnavailable": "大小不可用",
  "artworkMaintenance.stray.unrecognizedArtifact": "未识别 artifact",
  "artworkMaintenance.stray.unknownExtension": "未知扩展名",
  "artworkMaintenance.stray.inspectOnly": "仅检查",
} satisfies Record<ArtworkMaintenanceMessageId, string>;

export const artworkMaintenanceMessageCatalogs = {
  "en-US": enArtworkMaintenanceMessages,
  "zh-Hans": zhHansArtworkMaintenanceMessages,
} as const;
