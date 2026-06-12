export const enCatalogGovernanceMessages = {
  "catalogGovernance.title": "Catalog Governance",
  "catalogGovernance.kicker": "Catalog governance",
  "catalogGovernance.description":
    "Unknown and low-confidence Media Items with route-owned filters and safe fallback.",
  "catalogGovernance.refresh": "Refresh",
  "catalogGovernance.fallback":
    "{error}. Showing deterministic mock fallback data.",
  "catalogGovernance.filters": "Catalog governance filters",
  "catalogGovernance.filter.library": "Library",
  "catalogGovernance.filter.libraryAria": "Catalog library filter",
  "catalogGovernance.filter.libraryPlaceholder": "library-id",
  "catalogGovernance.filter.confidence": "Max confidence",
  "catalogGovernance.filter.confidenceAria": "Catalog max confidence filter",
  "catalogGovernance.filter.confidencePlaceholder": "500",
  "catalogGovernance.filter.limit": "Limit",
  "catalogGovernance.filter.limitAria": "Catalog page limit",
  "catalogGovernance.filter.active": "{count} filters",
  "catalogGovernance.clear": "Clear",
  "catalogGovernance.clearAria": "Clear catalog governance filters",
  "catalogGovernance.queue.title": "Governance queue",
  "catalogGovernance.queue.description":
    "{returned} returned, offset {offset}, limit {limit}",
  "catalogGovernance.queue.redacted": "URL filters are authoritative",
  "catalogGovernance.loading": "Loading Catalog Governance items",
  "catalogGovernance.empty":
    "No Catalog Governance items match the current filters.",
  "catalogGovernance.column.mediaItem": "Media Item",
  "catalogGovernance.column.kind": "Kind",
  "catalogGovernance.column.mediaLibrary": "Media Library",
  "catalogGovernance.column.localInference": "Local Inference",
  "catalogGovernance.column.issues": "Issues",
  "catalogGovernance.column.sources": "Sources",
  "catalogGovernance.column.mappings": "Mappings",
  "catalogGovernance.column.search": "Search",
  "catalogGovernance.review": "Review",
  "catalogGovernance.item.kind.unknown": "unknown",
  "catalogGovernance.item.kind.collection": "collection",
  "catalogGovernance.item.kind.default": "neutral",
  "catalogGovernance.inference.none": "No inference",
  "catalogGovernance.inference.confidence": "{confidence} confidence",
  "catalogGovernance.issues.none": "none",
  "catalogGovernance.relations.accepted": "{accepted}/{total} accepted",
  "catalogGovernance.detail.routeUnavailable":
    "Catalog Governance route data source is unavailable",
  "catalogGovernance.reviewLink": "Review {itemId}",
  "catalogGovernance.repair.title": "Catalog Governance Repair",
  "catalogGovernance.repair.kicker": "Catalog governance",
  "catalogGovernance.repair.description":
    "Decision-scoped repair context for one Media Item and one Provider Mapping.",
  "catalogGovernance.repair.refresh": "Refresh",
  "catalogGovernance.repair.backToQueue": "Back to queue",
  "catalogGovernance.repair.detailFallback":
    "{error}. Showing deterministic mock detail fallback.",
  "catalogGovernance.repair.reviewPlanFallback":
    "{error}. Showing deterministic mock review-plan fallback.",
  "catalogGovernance.repair.loading":
    "Loading Catalog Governance repair context",
  "catalogGovernance.repair.missingItem":
    "Catalog Governance item {itemId} could not be loaded.",
  "catalogGovernance.repair.itemContext.title": "Media Item context",
  "catalogGovernance.repair.itemContext.description":
    "Safe queue row, local inference summary, and issue codes for the selected Media Item.",
  "catalogGovernance.repair.item.kind.unknown": "unknown",
  "catalogGovernance.repair.item.kind.neutral": "neutral",
  "catalogGovernance.repair.itemId": "Media Item ID",
  "catalogGovernance.repair.itemTitle": "Title",
  "catalogGovernance.repair.mediaLibrary": "Media Library",
  "catalogGovernance.repair.sources": "Sources",
  "catalogGovernance.repair.providerMappings": "Provider Mappings",
  "catalogGovernance.repair.accepted": "{accepted}/{total} accepted",
  "catalogGovernance.repair.issues": "Issues",
  "catalogGovernance.repair.localInference": "Local Inference",
  "catalogGovernance.repair.providerSelection.title":
    "Provider Mapping selection",
  "catalogGovernance.repair.providerSelection.description":
    "Choose the mapping and decision that will be sent through the review-plan and confirmed mutation routes.",
  "catalogGovernance.repair.providerSelection.none": "none",
  "catalogGovernance.repair.providerSelection.noMapping":
    "No Provider Mapping is available for this item.",
  "catalogGovernance.repair.providerSelection.selector":
    "Provider Mapping selector",
  "catalogGovernance.repair.providerSelection.selectLabel":
    "{provider}:{key} / {status}",
  "catalogGovernance.repair.reviewPlan.title": "Review plan",
  "catalogGovernance.repair.reviewPlan.description":
    "Review-plan preview returned before any mutation is submitted.",
  "catalogGovernance.repair.reviewPlan.loading":
    "Loading Provider Mapping review plan",
  "catalogGovernance.repair.reviewPlan.empty":
    "Select a Provider Mapping before preparing review.",
  "catalogGovernance.repair.reviewPlan.decision": "Decision",
  "catalogGovernance.repair.reviewPlan.planStatus": "Plan status",
  "catalogGovernance.repair.reviewPlan.currentStatus": "Current status",
  "catalogGovernance.repair.reviewPlan.targetStatus": "Target status",
  "catalogGovernance.repair.reviewPlan.readiness": "Readiness",
  "catalogGovernance.repair.reviewPlan.readinessGroup": "{status} / {reasons}",
  "catalogGovernance.repair.decisionGroup": "Provider Mapping review decision",
  "catalogGovernance.repair.accept": "Accept",
  "catalogGovernance.repair.reject": "Reject",
  "catalogGovernance.repair.boundaries.title": "Repair boundaries",
  "catalogGovernance.repair.boundaries.description":
    "Boundary flags define exactly what the mutation can change.",
  "catalogGovernance.repair.boundaries.redacted": "Redacted plan",
  "catalogGovernance.repair.boundaries.noPlan":
    "No review-plan boundary is available.",
  "catalogGovernance.repair.boundaries.included": "included in plan",
  "catalogGovernance.repair.boundaries.excluded": "not included in plan",
  "catalogGovernance.repair.boundaries.yes": "yes",
  "catalogGovernance.repair.boundaries.no": "no",
  "catalogGovernance.repair.boundaries.providerMappingStatus":
    "Updates Provider Mapping status",
  "catalogGovernance.repair.boundaries.canonicalMetadata":
    "Updates Canonical Metadata",
  "catalogGovernance.repair.boundaries.providerSubject":
    "Updates Provider Subject",
  "catalogGovernance.repair.boundaries.localInference":
    "Updates Local Inference",
  "catalogGovernance.repair.boundaries.sourceDuplicates":
    "Updates Source Duplicates",
  "catalogGovernance.repair.boundaries.hierarchy": "Updates hierarchy",
  "catalogGovernance.repair.boundaries.writesNfo": "Writes NFO",
  "catalogGovernance.repair.boundaries.writesLibraryFiles":
    "Writes Library Files",
  "catalogGovernance.repair.boundaries.artwork": "Updates artwork",
  "catalogGovernance.repair.boundaries.playbackState": "Updates playback state",
  "catalogGovernance.repair.confirmed.title": "Confirmed action",
  "catalogGovernance.repair.confirmed.description":
    "The mutation is sent only after confirming the selected decision.",
  "catalogGovernance.repair.reviewError": "Review action failed",
  "catalogGovernance.repair.selectedDecision": "Selected decision",
  "catalogGovernance.repair.prepareConfirmation": "Prepare confirmation",
  "catalogGovernance.repair.prepareCopy":
    "Review plan is visible; no Provider Mapping mutation has been submitted.",
  "catalogGovernance.repair.prepareButton": "Prepare {decision} mapping review",
  "catalogGovernance.repair.confirmLabel": "Confirm {decision}",
  "catalogGovernance.repair.confirmCopy":
    "This sends the selected Provider Mapping review command for {mappingId}.",
  "catalogGovernance.repair.cancel": "Cancel",
  "catalogGovernance.repair.confirmButton": "Confirm {decision}",
  "catalogGovernance.repair.result.accepted": "Review result",
  "catalogGovernance.repair.result.statusChange": "Status change",
  "catalogGovernance.repair.result.idempotentReplay": "idempotent replay",
  "catalogGovernance.repair.result.newResult": "new result",
  "catalogGovernance.repair.result.selectedDecision":
    "{decision} for {mappingId} on {itemId}",
  "catalogGovernance.repair.itemUnavailable":
    "Catalog Governance repair detail data source is unavailable",
  "catalogGovernance.repair.planUnavailable":
    "Catalog Governance Provider Mapping review-plan data source is unavailable",
  "catalogGovernance.repair.selectionUnavailable":
    "Catalog Governance Provider Mapping is unavailable",
  "catalogGovernance.repair.notLiveError":
    "Catalog Governance Provider Mapping review requires a live Admin API response.",
  "catalogGovernance.repair.reviewUnavailable":
    "Catalog Governance Provider Mapping review action is unavailable",
  "catalogGovernance.repair.reviewFailed":
    "Catalog Governance Provider Mapping review action failed",
} as const;

export type CatalogGovernanceMessageId =
  keyof typeof enCatalogGovernanceMessages;

export const zhHansCatalogGovernanceMessages = {
  "catalogGovernance.title": "Catalog Governance",
  "catalogGovernance.kicker": "目录治理",
  "catalogGovernance.description":
    "未知和低置信度 Media Item，支持路由自有过滤器和安全回退。",
  "catalogGovernance.refresh": "刷新",
  "catalogGovernance.fallback": "{error}。正在显示确定性 mock 回退数据。",
  "catalogGovernance.filters": "Catalog governance 过滤器",
  "catalogGovernance.filter.library": "媒体库",
  "catalogGovernance.filter.libraryAria": "Catalog library 过滤器",
  "catalogGovernance.filter.libraryPlaceholder": "library-id",
  "catalogGovernance.filter.confidence": "最大置信度",
  "catalogGovernance.filter.confidenceAria": "Catalog 最大置信度过滤器",
  "catalogGovernance.filter.confidencePlaceholder": "500",
  "catalogGovernance.filter.limit": "Limit",
  "catalogGovernance.filter.limitAria": "Catalog 页面 limit",
  "catalogGovernance.filter.active": "{count} 个过滤器",
  "catalogGovernance.clear": "清除",
  "catalogGovernance.clearAria": "清除 Catalog governance 过滤器",
  "catalogGovernance.queue.title": "治理队列",
  "catalogGovernance.queue.description":
    "返回 {returned} 条，offset {offset}，limit {limit}",
  "catalogGovernance.queue.redacted": "URL 过滤条件具有权威性",
  "catalogGovernance.loading": "正在加载 Catalog Governance 条目",
  "catalogGovernance.empty":
    "当前过滤条件下没有匹配的 Catalog Governance 条目。",
  "catalogGovernance.column.mediaItem": "媒体条目",
  "catalogGovernance.column.kind": "类型",
  "catalogGovernance.column.mediaLibrary": "媒体库",
  "catalogGovernance.column.localInference": "本地推断",
  "catalogGovernance.column.issues": "问题",
  "catalogGovernance.column.sources": "来源",
  "catalogGovernance.column.mappings": "映射",
  "catalogGovernance.column.search": "搜索",
  "catalogGovernance.review": "审查",
  "catalogGovernance.item.kind.unknown": "unknown",
  "catalogGovernance.item.kind.collection": "collection",
  "catalogGovernance.item.kind.default": "neutral",
  "catalogGovernance.inference.none": "无推断",
  "catalogGovernance.inference.confidence": "{confidence} 置信度",
  "catalogGovernance.issues.none": "无",
  "catalogGovernance.relations.accepted": "{accepted}/{total} 已接受",
  "catalogGovernance.detail.routeUnavailable":
    "Catalog Governance 路由数据源不可用",
  "catalogGovernance.reviewLink": "审查 {itemId}",
  "catalogGovernance.repair.title": "Catalog Governance 修复",
  "catalogGovernance.repair.kicker": "目录治理",
  "catalogGovernance.repair.description":
    "针对单个 Media Item 和单个 Provider Mapping 的决策范围修复上下文。",
  "catalogGovernance.repair.refresh": "刷新",
  "catalogGovernance.repair.backToQueue": "返回队列",
  "catalogGovernance.repair.detailFallback":
    "{error}。正在显示确定性 mock 详情回退。",
  "catalogGovernance.repair.reviewPlanFallback":
    "{error}。正在显示确定性 mock review-plan 回退。",
  "catalogGovernance.repair.loading": "正在加载 Catalog Governance 修复上下文",
  "catalogGovernance.repair.missingItem":
    "Catalog Governance 条目 {itemId} 无法加载。",
  "catalogGovernance.repair.itemContext.title": "Media Item 上下文",
  "catalogGovernance.repair.itemContext.description":
    "所选 Media Item 的安全队列行、本地推断摘要和问题代码。",
  "catalogGovernance.repair.item.kind.unknown": "unknown",
  "catalogGovernance.repair.item.kind.neutral": "neutral",
  "catalogGovernance.repair.itemId": "Media Item ID",
  "catalogGovernance.repair.itemTitle": "标题",
  "catalogGovernance.repair.mediaLibrary": "媒体库",
  "catalogGovernance.repair.sources": "来源",
  "catalogGovernance.repair.providerMappings": "Provider 映射",
  "catalogGovernance.repair.accepted": "{accepted}/{total} 已接受",
  "catalogGovernance.repair.issues": "问题",
  "catalogGovernance.repair.localInference": "本地推断",
  "catalogGovernance.repair.providerSelection.title": "Provider Mapping 选择",
  "catalogGovernance.repair.providerSelection.description":
    "选择将通过 review-plan 和已确认 mutation 路由发送的映射与决策。",
  "catalogGovernance.repair.providerSelection.none": "无",
  "catalogGovernance.repair.providerSelection.noMapping":
    "此条目没有可用的 Provider Mapping。",
  "catalogGovernance.repair.providerSelection.selector":
    "Provider Mapping 选择器",
  "catalogGovernance.repair.providerSelection.selectLabel":
    "{provider}:{key} / {status}",
  "catalogGovernance.repair.reviewPlan.title": "Review 计划",
  "catalogGovernance.repair.reviewPlan.description":
    "任何 mutation 提交前返回的 review-plan 预览。",
  "catalogGovernance.repair.reviewPlan.loading":
    "正在加载 Provider Mapping review 计划",
  "catalogGovernance.repair.reviewPlan.empty":
    "准备审查前先选择一个 Provider Mapping。",
  "catalogGovernance.repair.reviewPlan.decision": "决策",
  "catalogGovernance.repair.reviewPlan.planStatus": "计划状态",
  "catalogGovernance.repair.reviewPlan.currentStatus": "当前状态",
  "catalogGovernance.repair.reviewPlan.targetStatus": "目标状态",
  "catalogGovernance.repair.reviewPlan.readiness": "就绪度",
  "catalogGovernance.repair.reviewPlan.readinessGroup": "{status} / {reasons}",
  "catalogGovernance.repair.decisionGroup": "Provider Mapping 审查决策",
  "catalogGovernance.repair.accept": "接受",
  "catalogGovernance.repair.reject": "拒绝",
  "catalogGovernance.repair.boundaries.title": "修复边界",
  "catalogGovernance.repair.boundaries.description":
    "边界标志会精确说明 mutation 能改什么。",
  "catalogGovernance.repair.boundaries.redacted": "已脱敏计划",
  "catalogGovernance.repair.boundaries.noPlan": "没有可用的 review-plan 边界。",
  "catalogGovernance.repair.boundaries.included": "包含在计划中",
  "catalogGovernance.repair.boundaries.excluded": "不包含在计划中",
  "catalogGovernance.repair.boundaries.yes": "是",
  "catalogGovernance.repair.boundaries.no": "否",
  "catalogGovernance.repair.boundaries.providerMappingStatus":
    "更新 Provider Mapping 状态",
  "catalogGovernance.repair.boundaries.canonicalMetadata":
    "更新 Canonical Metadata",
  "catalogGovernance.repair.boundaries.providerSubject":
    "更新 Provider Subject",
  "catalogGovernance.repair.boundaries.localInference": "更新本地推断",
  "catalogGovernance.repair.boundaries.sourceDuplicates": "更新 Source 重复项",
  "catalogGovernance.repair.boundaries.hierarchy": "更新层级",
  "catalogGovernance.repair.boundaries.writesNfo": "写入 NFO",
  "catalogGovernance.repair.boundaries.writesLibraryFiles": "写入媒体库文件",
  "catalogGovernance.repair.boundaries.artwork": "更新 artwork",
  "catalogGovernance.repair.boundaries.playbackState": "更新播放状态",
  "catalogGovernance.repair.confirmed.title": "已确认操作",
  "catalogGovernance.repair.confirmed.description":
    "只有在确认所选决策后才会发送 mutation。",
  "catalogGovernance.repair.reviewError": "审查操作失败",
  "catalogGovernance.repair.selectedDecision": "已选决策",
  "catalogGovernance.repair.prepareConfirmation": "准备确认",
  "catalogGovernance.repair.prepareCopy":
    "review 计划已可见，还没有提交 Provider Mapping mutation。",
  "catalogGovernance.repair.prepareButton": "准备 {decision} 映射审查",
  "catalogGovernance.repair.confirmLabel": "确认 {decision}",
  "catalogGovernance.repair.confirmCopy":
    "这会发送针对 {mappingId} 的所选 Provider Mapping 审查命令。",
  "catalogGovernance.repair.cancel": "取消",
  "catalogGovernance.repair.confirmButton": "确认 {decision}",
  "catalogGovernance.repair.result.accepted": "审查结果",
  "catalogGovernance.repair.result.statusChange": "状态变更",
  "catalogGovernance.repair.result.idempotentReplay": "幂等重放",
  "catalogGovernance.repair.result.newResult": "新结果",
  "catalogGovernance.repair.result.selectedDecision":
    "{itemId} 上的 {mappingId}，决策 {decision}",
  "catalogGovernance.repair.itemUnavailable":
    "Catalog Governance 修复详情数据源不可用",
  "catalogGovernance.repair.planUnavailable":
    "Catalog Governance Provider Mapping review-plan 数据源不可用",
  "catalogGovernance.repair.selectionUnavailable":
    "Catalog Governance Provider Mapping 不可用",
  "catalogGovernance.repair.notLiveError":
    "Catalog Governance Provider Mapping 审查需要实时 Admin API 响应。",
  "catalogGovernance.repair.reviewUnavailable":
    "Catalog Governance Provider Mapping 审查动作不可用",
  "catalogGovernance.repair.reviewFailed":
    "Catalog Governance Provider Mapping 审查动作失败",
} satisfies Record<CatalogGovernanceMessageId, string>;

export const catalogGovernanceMessageCatalogs = {
  "en-US": enCatalogGovernanceMessages,
  "zh-Hans": zhHansCatalogGovernanceMessages,
} as const;
