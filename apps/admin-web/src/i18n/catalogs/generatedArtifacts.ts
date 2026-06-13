export const enGeneratedArtifactsMessages = {
  "generatedArtifacts.title": "Generated Artifacts",
  "generatedArtifacts.kicker": "Automation",
  "generatedArtifacts.description":
    "AI-assisted proposals reduced to readiness, payload shape, confidence, and fingerprints. Review mutations stay out of this route.",
  "generatedArtifacts.refresh": "Refresh",
  "generatedArtifacts.fallback":
    "{error}. Showing deterministic mock fallback data.",
  "generatedArtifacts.pagination": "Generated artifact pagination",
  "generatedArtifacts.limit": "Limit",
  "generatedArtifacts.limitAria": "Generated artifacts page limit",
  "generatedArtifacts.offset": "Offset",
  "generatedArtifacts.offsetAria": "Generated artifacts page offset",
  "generatedArtifacts.actionableCount": "{count} actionable",
  "generatedArtifacts.reset": "Reset",
  "generatedArtifacts.queue.title": "Proposal queue",
  "generatedArtifacts.queue.description":
    "{returned} returned, offset {offset}, limit {limit}",
  "generatedArtifacts.queue.urlPagination": "URL pagination is authoritative",
  "generatedArtifacts.loading": "Loading Generated Artifacts proposals",
  "generatedArtifacts.empty":
    "No Generated Artifact proposals match the current page.",
  "generatedArtifacts.dataSourceUnavailable":
    "Generated Artifacts route data source is unavailable",
  "generatedArtifacts.column.proposal": "Proposal",
  "generatedArtifacts.column.status": "Status",
  "generatedArtifacts.column.readiness": "Readiness",
  "generatedArtifacts.column.target": "Target",
  "generatedArtifacts.column.provider": "Provider",
  "generatedArtifacts.column.payload": "Payload",
  "generatedArtifacts.column.confidence": "Confidence",
  "generatedArtifacts.column.fingerprints": "Fingerprints",
  "generatedArtifacts.column.updated": "Updated",
  "generatedArtifacts.unknownProvider": "unknown provider",
  "generatedArtifacts.attempts": "{count} attempts",
  "generatedArtifacts.noTargetId": "no target id",
  "generatedArtifacts.unknown": "unknown",
  "generatedArtifacts.noFingerprint": "no fingerprint",
  "generatedArtifacts.review": "Review",
  "generatedArtifacts.reviewAria": "Review {artifactId}",
} as const;

export type GeneratedArtifactsMessageId =
  keyof typeof enGeneratedArtifactsMessages;

export const zhHansGeneratedArtifactsMessages = {
  "generatedArtifacts.title": "Generated Artifacts",
  "generatedArtifacts.kicker": "自动化",
  "generatedArtifacts.description":
    "AI 辅助 proposal 缩减到就绪度、payload shape、置信度和指纹。Review mutation 不在此路由中。",
  "generatedArtifacts.refresh": "刷新",
  "generatedArtifacts.fallback": "{error}。正在显示确定性 mock 回退数据。",
  "generatedArtifacts.pagination": "Generated artifact 分页",
  "generatedArtifacts.limit": "Limit",
  "generatedArtifacts.limitAria": "Generated artifacts 页面 limit",
  "generatedArtifacts.offset": "Offset",
  "generatedArtifacts.offsetAria": "Generated artifacts 页面 offset",
  "generatedArtifacts.actionableCount": "{count} 个可操作",
  "generatedArtifacts.reset": "重置",
  "generatedArtifacts.queue.title": "Proposal 队列",
  "generatedArtifacts.queue.description":
    "返回 {returned} 条，offset {offset}，limit {limit}",
  "generatedArtifacts.queue.urlPagination": "URL 分页具有权威性",
  "generatedArtifacts.loading": "正在加载 Generated Artifacts proposals",
  "generatedArtifacts.empty": "当前页没有匹配的 Generated Artifact proposal。",
  "generatedArtifacts.dataSourceUnavailable":
    "Generated Artifacts 路由数据源不可用",
  "generatedArtifacts.column.proposal": "Proposal",
  "generatedArtifacts.column.status": "状态",
  "generatedArtifacts.column.readiness": "就绪度",
  "generatedArtifacts.column.target": "目标",
  "generatedArtifacts.column.provider": "Provider",
  "generatedArtifacts.column.payload": "Payload",
  "generatedArtifacts.column.confidence": "置信度",
  "generatedArtifacts.column.fingerprints": "指纹",
  "generatedArtifacts.column.updated": "更新时间",
  "generatedArtifacts.unknownProvider": "未知 provider",
  "generatedArtifacts.attempts": "{count} 次尝试",
  "generatedArtifacts.noTargetId": "无 target id",
  "generatedArtifacts.unknown": "未知",
  "generatedArtifacts.noFingerprint": "无指纹",
  "generatedArtifacts.review": "审查",
  "generatedArtifacts.reviewAria": "审查 {artifactId}",
} satisfies Record<GeneratedArtifactsMessageId, string>;

export const generatedArtifactsMessageCatalogs = {
  "en-US": enGeneratedArtifactsMessages,
  "zh-Hans": zhHansGeneratedArtifactsMessages,
} as const;
