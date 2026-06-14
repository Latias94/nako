export const enSourceDuplicateMessages = {
  "sourceDuplicate.backToItem": "Back to Item",
  "sourceDuplicate.refresh": "Refresh",
  "sourceDuplicate.description":
    "Source-scoped duplicate reconciliation plan and guarded apply flow from redacted Admin API summaries.",
  "sourceDuplicate.kicker": "Media Source",
  "sourceDuplicate.title": "Source duplicate reconciliation",
  "sourceDuplicate.fallback":
    "{error}. Showing deterministic mock fallback data.",
  "sourceDuplicate.pagination": "Source duplicate pagination",
  "sourceDuplicate.limit": "Limit",
  "sourceDuplicate.limitAria": "Source duplicate page limit",
  "sourceDuplicate.offset": "Offset",
  "sourceDuplicate.offsetAria": "Source duplicate page offset",
  "sourceDuplicate.redactedPlan": "Redacted plan",
  "sourceDuplicate.stale": "stale",
  "sourceDuplicate.current": "current",
  "sourceDuplicate.reset": "Reset",
  "sourceDuplicate.loading": "Loading source duplicate reconciliation plan",
  "sourceDuplicate.summary.title": "Plan summary",
  "sourceDuplicate.summary.description":
    "{returned} returned, offset {offset}, limit {limit}",
  "sourceDuplicate.summary.library": "Media Library",
  "sourceDuplicate.summary.mediaItem": "Media Item",
  "sourceDuplicate.summary.source": "Media Source",
  "sourceDuplicate.summary.fingerprintEvidence": "Fingerprint evidence",
  "sourceDuplicate.summary.confidence": "Confidence",
  "sourceDuplicate.reviewSummary.title": "Review summary",
  "sourceDuplicate.reviewSummary.description":
    "{total} returned candidates on this page, summarized before row-level review.",
  "sourceDuplicate.reviewSummary.total": "Returned candidates",
  "sourceDuplicate.reviewSummary.totalValue": "{count} candidates on this page",
  "sourceDuplicate.reviewSummary.actionable": "Actionable suggestions",
  "sourceDuplicate.reviewSummary.actionableValue":
    "{count} recommend suggest_relationship",
  "sourceDuplicate.reviewSummary.preserved": "Preserved/read-only candidates",
  "sourceDuplicate.reviewSummary.preservedValue":
    "{count} preserved or read-only candidates",
  "sourceDuplicate.reviewSummary.staleRefresh": "Stale/refresh candidates",
  "sourceDuplicate.reviewSummary.staleRefreshValue":
    "{count} stale or refresh candidates",
  "sourceDuplicate.candidates.title": "Duplicate candidates",
  "sourceDuplicate.candidates.description":
    "Candidates reduced to IDs, evidence class, confidence, stale state, relationship status, and recommended action.",
  "sourceDuplicate.candidates.empty":
    "No duplicate candidates match the current page.",
  "sourceDuplicate.candidates.relationship":
    "relationship {relationship} / status {status}",
  "sourceDuplicate.action.prepareSuggestion": "Prepare suggestion",
  "sourceDuplicate.action.confirmSuggestion": "Confirm suggestion",
  "sourceDuplicate.action.prepareConfirm": "Prepare confirm",
  "sourceDuplicate.action.confirmRelationship": "Confirm relationship",
  "sourceDuplicate.action.prepareReject": "Prepare reject",
  "sourceDuplicate.action.rejectRelationship": "Reject relationship",
  "sourceDuplicate.action.noMutation": "No mutation",
  "sourceDuplicate.action.cancel": "Cancel",
  "sourceDuplicate.result.title": "Confirmed action result",
  "sourceDuplicate.result.description":
    "Suggest relationship commands require a prepared confirmation before Admin API mutation.",
  "sourceDuplicate.result.empty":
    "No source duplicate mutation has been submitted from this route.",
  "sourceDuplicate.result.suggestedRelationship": "Suggested relationship",
  "sourceDuplicate.result.relationship": "Relationship ID",
  "sourceDuplicate.result.created": "created",
  "sourceDuplicate.result.idempotent": "idempotent",
  "sourceDuplicate.confidence.value": "{confidence}/1000",
  "sourceDuplicate.confidence.unknown": "confidence unknown",
  "sourceDuplicate.none": "none",
  "sourceDuplicate.missingLibrary":
    "Media Library ID is required for source duplicate reconciliation.",
  "sourceDuplicate.planUnavailable":
    "Source duplicate reconciliation plan data source is unavailable",
  "sourceDuplicate.notLiveError":
    "Source duplicate reconciliation apply requires a live Admin API response.",
  "sourceDuplicate.applyUnavailable":
    "Source duplicate reconciliation apply action is unavailable",
  "sourceDuplicate.applyFailed":
    "Source duplicate reconciliation apply action failed",
} as const;

export type SourceDuplicateMessageId = keyof typeof enSourceDuplicateMessages;

export const zhHansSourceDuplicateMessages = {
  "sourceDuplicate.backToItem": "返回条目",
  "sourceDuplicate.refresh": "刷新",
  "sourceDuplicate.description":
    "基于脱敏 Admin API 摘要的 source 级重复调和计划和受保护 apply 流。",
  "sourceDuplicate.kicker": "Media Source",
  "sourceDuplicate.title": "Source duplicate 调和",
  "sourceDuplicate.fallback": "{error}。正在显示确定性 mock 回退数据。",
  "sourceDuplicate.pagination": "Source duplicate 分页",
  "sourceDuplicate.limit": "Limit",
  "sourceDuplicate.limitAria": "Source duplicate 页面 limit",
  "sourceDuplicate.offset": "Offset",
  "sourceDuplicate.offsetAria": "Source duplicate 页面 offset",
  "sourceDuplicate.redactedPlan": "已脱敏计划",
  "sourceDuplicate.stale": "过期",
  "sourceDuplicate.current": "当前",
  "sourceDuplicate.reset": "重置",
  "sourceDuplicate.loading": "正在加载 source duplicate 调和计划",
  "sourceDuplicate.summary.title": "计划摘要",
  "sourceDuplicate.summary.description":
    "返回 {returned} 条，offset {offset}，limit {limit}",
  "sourceDuplicate.summary.library": "媒体库",
  "sourceDuplicate.summary.mediaItem": "Media Item",
  "sourceDuplicate.summary.source": "Media Source",
  "sourceDuplicate.summary.fingerprintEvidence": "指纹证据",
  "sourceDuplicate.summary.confidence": "置信度",
  "sourceDuplicate.reviewSummary.title": "审核摘要",
  "sourceDuplicate.reviewSummary.description":
    "当前页返回 {total} 个候选，先汇总再逐行审核。",
  "sourceDuplicate.reviewSummary.total": "返回候选",
  "sourceDuplicate.reviewSummary.totalValue": "当前页 {count} 个候选",
  "sourceDuplicate.reviewSummary.actionable": "可操作建议",
  "sourceDuplicate.reviewSummary.actionableValue":
    "{count} 个推荐 suggest_relationship",
  "sourceDuplicate.reviewSummary.preserved": "保留/只读候选",
  "sourceDuplicate.reviewSummary.preservedValue": "{count} 个保留或只读候选",
  "sourceDuplicate.reviewSummary.staleRefresh": "过期/刷新候选",
  "sourceDuplicate.reviewSummary.staleRefreshValue": "{count} 个过期或刷新候选",
  "sourceDuplicate.candidates.title": "重复候选",
  "sourceDuplicate.candidates.description":
    "候选只缩减到 ID、证据类别、置信度、过期状态、关系状态和推荐动作。",
  "sourceDuplicate.candidates.empty": "当前页没有匹配的重复候选。",
  "sourceDuplicate.candidates.relationship":
    "关系 {relationship} / 状态 {status}",
  "sourceDuplicate.action.prepareSuggestion": "准备建议",
  "sourceDuplicate.action.confirmSuggestion": "确认建议",
  "sourceDuplicate.action.prepareConfirm": "准备确认",
  "sourceDuplicate.action.confirmRelationship": "确认关系",
  "sourceDuplicate.action.prepareReject": "准备拒绝",
  "sourceDuplicate.action.rejectRelationship": "拒绝关系",
  "sourceDuplicate.action.noMutation": "无 mutation",
  "sourceDuplicate.action.cancel": "取消",
  "sourceDuplicate.result.title": "已确认操作结果",
  "sourceDuplicate.result.description":
    "Suggest relationship 命令需要先准备确认，随后才会发送 Admin API mutation。",
  "sourceDuplicate.result.empty": "此路由尚未提交 source duplicate mutation。",
  "sourceDuplicate.result.suggestedRelationship": "已建议关系",
  "sourceDuplicate.result.relationship": "关系 ID",
  "sourceDuplicate.result.created": "已创建",
  "sourceDuplicate.result.idempotent": "幂等",
  "sourceDuplicate.confidence.value": "{confidence}/1000",
  "sourceDuplicate.confidence.unknown": "置信度未知",
  "sourceDuplicate.none": "无",
  "sourceDuplicate.missingLibrary": "Source duplicate 调和需要媒体库 ID。",
  "sourceDuplicate.planUnavailable": "Source duplicate 调和计划数据源不可用",
  "sourceDuplicate.notLiveError":
    "Source duplicate 调和 apply 需要实时 Admin API 响应。",
  "sourceDuplicate.applyUnavailable": "Source duplicate 调和 apply 动作不可用",
  "sourceDuplicate.applyFailed": "Source duplicate 调和 apply 动作失败",
} satisfies Record<SourceDuplicateMessageId, string>;

export const sourceDuplicateMessageCatalogs = {
  "en-US": enSourceDuplicateMessages,
  "zh-Hans": zhHansSourceDuplicateMessages,
} as const;
