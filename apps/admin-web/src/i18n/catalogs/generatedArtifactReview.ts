export const enGeneratedArtifactReviewMessages = {
  "generatedArtifactReview.title": "Generated Artifact Review",
  "generatedArtifactReview.kicker": "Automation",
  "generatedArtifactReview.description":
    "Review-plan preview for one Generated Artifact proposal. Confirmation and accept/reject commands remain in the guarded action flow.",
  "generatedArtifactReview.refresh": "Refresh",
  "generatedArtifactReview.backToQueue": "Back to queue",
  "generatedArtifactReview.fallback":
    "{error}. Showing deterministic mock review-plan fallback.",
  "generatedArtifactReview.loading": "Loading Generated Artifact review plan",
  "generatedArtifactReview.missing":
    "Generated Artifact {artifactId} could not be loaded.",
  "generatedArtifactReview.reviewPlan.title": "Review plan",
  "generatedArtifactReview.reviewPlan.description":
    "Decision-scoped plan returned before any mutation is submitted.",
  "generatedArtifactReview.reviewPlan.artifactId": "Generated Artifact ID",
  "generatedArtifactReview.reviewPlan.decision": "Decision",
  "generatedArtifactReview.reviewPlan.status": "Status",
  "generatedArtifactReview.reviewPlan.action": "Action",
  "generatedArtifactReview.reviewPlan.reasons": "Reasons",
  "generatedArtifactReview.decisionGroup": "Generated Artifact review decision",
  "generatedArtifactReview.accept": "Accept",
  "generatedArtifactReview.reject": "Reject",
  "generatedArtifactReview.summary.title": "Safe summary",
  "generatedArtifactReview.summary.description":
    "Payload and target facts are reduced to IDs, counts, confidence, and fingerprints.",
  "generatedArtifactReview.summary.capability": "Capability",
  "generatedArtifactReview.summary.kind": "Kind",
  "generatedArtifactReview.summary.target": "Target",
  "generatedArtifactReview.summary.payload": "Payload",
  "generatedArtifactReview.summary.confidence": "Confidence",
  "generatedArtifactReview.summary.fingerprint": "Fingerprint",
  "generatedArtifactReview.boundaries.title": "Review boundaries",
  "generatedArtifactReview.boundaries.description":
    "Boundary flags tell the operator what the plan can touch.",
  "generatedArtifactReview.boundaries.redacted": "Redacted plan",
  "generatedArtifactReview.boundaries.acceptedIntoCanonicalMetadata":
    "Accepted into Canonical Metadata",
  "generatedArtifactReview.boundaries.requiresMetadataAuthorityApply":
    "Requires Metadata Authority apply",
  "generatedArtifactReview.boundaries.writesSidecar": "Writes sidecars",
  "generatedArtifactReview.boundaries.writesLibraryFiles":
    "Writes library files",
  "generatedArtifactReview.boundaries.appliesImmediately":
    "Applies immediately",
  "generatedArtifactReview.boundaries.included": "included in plan",
  "generatedArtifactReview.boundaries.excluded": "not included in plan",
  "generatedArtifactReview.boundaries.yes": "yes",
  "generatedArtifactReview.boundaries.no": "no",
  "generatedArtifactReview.readiness.title": "Readiness",
  "generatedArtifactReview.readiness.description":
    "Readiness is safe status text and reason codes only.",
  "generatedArtifactReview.readiness.actionable": "actionable",
  "generatedArtifactReview.readiness.blocked": "blocked",
  "generatedArtifactReview.confirmed.title": "Confirmed action",
  "generatedArtifactReview.confirmed.description":
    "The command is sent only after confirming the selected decision.",
  "generatedArtifactReview.selectedDecision": "Selected decision",
  "generatedArtifactReview.prepareConfirmation": "Prepare confirmation",
  "generatedArtifactReview.prepareCopy":
    "Review plan is visible; no mutation has been submitted.",
  "generatedArtifactReview.prepareButton": "Prepare {decision} review",
  "generatedArtifactReview.confirmLabel": "Confirm {decision}",
  "generatedArtifactReview.confirmCopy":
    "This sends the selected review command for {artifactId}.",
  "generatedArtifactReview.cancel": "Cancel",
  "generatedArtifactReview.confirmButton": "Confirm {decision}",
  "generatedArtifactReview.result.selectedDecision":
    "{decision} for {artifactId}",
  "generatedArtifactReview.result.reviewResult": "Review result",
  "generatedArtifactReview.result.decision": "Decision",
  "generatedArtifactReview.result.idempotentReplay": "idempotent replay",
  "generatedArtifactReview.result.newResult": "new result",
  "generatedArtifactReview.result.acceptedAt": "Accepted at",
  "generatedArtifactReview.result.notAccepted": "not accepted",
  "generatedArtifactReview.planUnavailable":
    "Generated Artifact review-plan data source is unavailable",
  "generatedArtifactReview.notLiveError":
    "Generated Artifact review mutations require a live Admin API response.",
  "generatedArtifactReview.reviewUnavailable":
    "Generated Artifact review action is unavailable",
  "generatedArtifactReview.reviewFailed":
    "Generated Artifact review action failed",
  "generatedArtifactReview.none": "none",
  "generatedArtifactReview.unknown": "unknown",
  "generatedArtifactReview.noFingerprint": "no fingerprint",
} as const;

export type GeneratedArtifactReviewMessageId =
  keyof typeof enGeneratedArtifactReviewMessages;

export const zhHansGeneratedArtifactReviewMessages = {
  "generatedArtifactReview.title": "Generated Artifact 审查",
  "generatedArtifactReview.kicker": "自动化",
  "generatedArtifactReview.description":
    "针对单个 Generated Artifact proposal 的 review-plan 预览。确认和 accept/reject 命令保持在受保护动作流中。",
  "generatedArtifactReview.refresh": "刷新",
  "generatedArtifactReview.backToQueue": "返回队列",
  "generatedArtifactReview.fallback":
    "{error}。正在显示确定性 mock review-plan 回退。",
  "generatedArtifactReview.loading": "正在加载 Generated Artifact review 计划",
  "generatedArtifactReview.missing":
    "Generated Artifact {artifactId} 无法加载。",
  "generatedArtifactReview.reviewPlan.title": "Review 计划",
  "generatedArtifactReview.reviewPlan.description":
    "任何 mutation 提交前返回的决策级计划。",
  "generatedArtifactReview.reviewPlan.artifactId": "Generated Artifact ID",
  "generatedArtifactReview.reviewPlan.decision": "决策",
  "generatedArtifactReview.reviewPlan.status": "状态",
  "generatedArtifactReview.reviewPlan.action": "动作",
  "generatedArtifactReview.reviewPlan.reasons": "原因",
  "generatedArtifactReview.decisionGroup": "Generated Artifact 审查决策",
  "generatedArtifactReview.accept": "接受",
  "generatedArtifactReview.reject": "拒绝",
  "generatedArtifactReview.summary.title": "安全摘要",
  "generatedArtifactReview.summary.description":
    "Payload 和 target 事实缩减到 ID、计数、置信度和指纹。",
  "generatedArtifactReview.summary.capability": "能力",
  "generatedArtifactReview.summary.kind": "类型",
  "generatedArtifactReview.summary.target": "目标",
  "generatedArtifactReview.summary.payload": "Payload",
  "generatedArtifactReview.summary.confidence": "置信度",
  "generatedArtifactReview.summary.fingerprint": "指纹",
  "generatedArtifactReview.boundaries.title": "Review 边界",
  "generatedArtifactReview.boundaries.description":
    "边界标志会告诉操作员此计划能触碰什么。",
  "generatedArtifactReview.boundaries.redacted": "已脱敏计划",
  "generatedArtifactReview.boundaries.acceptedIntoCanonicalMetadata":
    "接受进 Canonical Metadata",
  "generatedArtifactReview.boundaries.requiresMetadataAuthorityApply":
    "需要 Metadata Authority apply",
  "generatedArtifactReview.boundaries.writesSidecar": "写入 sidecar",
  "generatedArtifactReview.boundaries.writesLibraryFiles": "写入媒体库文件",
  "generatedArtifactReview.boundaries.appliesImmediately": "立即应用",
  "generatedArtifactReview.boundaries.included": "包含在计划中",
  "generatedArtifactReview.boundaries.excluded": "不包含在计划中",
  "generatedArtifactReview.boundaries.yes": "是",
  "generatedArtifactReview.boundaries.no": "否",
  "generatedArtifactReview.readiness.title": "就绪度",
  "generatedArtifactReview.readiness.description":
    "就绪度只包含安全状态文本和 reason code。",
  "generatedArtifactReview.readiness.actionable": "可操作",
  "generatedArtifactReview.readiness.blocked": "阻塞",
  "generatedArtifactReview.confirmed.title": "已确认操作",
  "generatedArtifactReview.confirmed.description":
    "只有确认所选决策后才会发送命令。",
  "generatedArtifactReview.selectedDecision": "已选决策",
  "generatedArtifactReview.prepareConfirmation": "准备确认",
  "generatedArtifactReview.prepareCopy":
    "review 计划已可见，还没有提交 mutation。",
  "generatedArtifactReview.prepareButton": "准备 {decision} 审查",
  "generatedArtifactReview.confirmLabel": "确认 {decision}",
  "generatedArtifactReview.confirmCopy":
    "这会发送针对 {artifactId} 的所选 review 命令。",
  "generatedArtifactReview.cancel": "取消",
  "generatedArtifactReview.confirmButton": "确认 {decision}",
  "generatedArtifactReview.result.selectedDecision":
    "{artifactId} 的决策 {decision}",
  "generatedArtifactReview.result.reviewResult": "审查结果",
  "generatedArtifactReview.result.decision": "决策",
  "generatedArtifactReview.result.idempotentReplay": "幂等重放",
  "generatedArtifactReview.result.newResult": "新结果",
  "generatedArtifactReview.result.acceptedAt": "接受时间",
  "generatedArtifactReview.result.notAccepted": "未接受",
  "generatedArtifactReview.planUnavailable":
    "Generated Artifact review-plan 数据源不可用",
  "generatedArtifactReview.notLiveError":
    "Generated Artifact 审查 mutation 需要实时 Admin API 响应。",
  "generatedArtifactReview.reviewUnavailable":
    "Generated Artifact 审查动作不可用",
  "generatedArtifactReview.reviewFailed": "Generated Artifact 审查动作失败",
  "generatedArtifactReview.none": "无",
  "generatedArtifactReview.unknown": "未知",
  "generatedArtifactReview.noFingerprint": "无指纹",
} satisfies Record<GeneratedArtifactReviewMessageId, string>;

export const generatedArtifactReviewMessageCatalogs = {
  "en-US": enGeneratedArtifactReviewMessages,
  "zh-Hans": zhHansGeneratedArtifactReviewMessages,
} as const;
