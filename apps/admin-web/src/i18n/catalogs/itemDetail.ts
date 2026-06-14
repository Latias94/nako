export const enItemDetailMessages = {
  "itemDetail.backToCatalog": "Back to Catalog",
  "itemDetail.refresh": "Refresh",
  "itemDetail.description":
    "Administration-supporting Media Item facts from safe public read bridges. Repair and playback workflows stay out of this route.",
  "itemDetail.kicker": "Media Item",
  "itemDetail.fallback":
    "{error}. Showing deterministic mock fallback where needed.",
  "itemDetail.loading": "Loading Media Item detail",
  "itemDetail.missing": "Media Item {itemId} could not be loaded.",
  "itemDetail.dataSourceUnavailable":
    "Media Item detail route data source is unavailable",
  "itemDetail.facts.title": "Item facts",
  "itemDetail.facts.description":
    "Provider-neutral Media Item identity and relationship counts.",
  "itemDetail.facts.mediaItemId": "Media Item ID",
  "itemDetail.facts.parent": "Parent",
  "itemDetail.facts.release": "Release",
  "itemDetail.facts.runtime": "Runtime",
  "itemDetail.facts.sources": "Sources",
  "itemDetail.facts.images": "Images",
  "itemDetail.parent.none": "No parent",
  "itemDetail.release.none": "No release date",
  "itemDetail.runtime.unknown": "Runtime unknown",
  "itemDetail.runtime.minutes": "{minutes} min",
  "itemDetail.sources.count": "{count} Media Sources",
  "itemDetail.images.count": "{count} public image refs",
  "itemDetail.redactedSummary": "Redacted summary",
  "itemDetail.canonical.title": "Canonical Metadata",
  "itemDetail.canonical.description":
    "Canonical Metadata counts and display labels only, without raw provider payloads.",
  "itemDetail.canonical.genres": "Genres",
  "itemDetail.canonical.tags": "Tags",
  "itemDetail.canonical.credits": "Credits",
  "itemDetail.canonical.collections": "Collections",
  "itemDetail.canonical.studios": "Studios",
  "itemDetail.canonical.creditAs": "{name} as {character}",
  "itemDetail.canonical.creditRole": "{name} ({role})",
  "itemDetail.canonical.externalEvidence": "External evidence",
  "itemDetail.canonical.externalEvidenceValue":
    "{ratings} ratings / {externalIds} external IDs",
  "itemDetail.canonical.countsOnly": "counts only",
  "itemDetail.sources.title": "Media Sources",
  "itemDetail.sources.description":
    "Safe Media Source filenames and bounded probe summaries. Raw locators are never rendered.",
  "itemDetail.sources.probeUnavailable": "Probe summary unavailable",
  "itemDetail.sources.fingerprinted": "fingerprinted",
  "itemDetail.sources.noFingerprint": "no fingerprint",
  "itemDetail.sources.unknownContainer": "unknown container",
  "itemDetail.sources.probe": "{container} / {duration} / {streams} streams",
  "itemDetail.sources.openDuplicateReview": "Review duplicates",
  "itemDetail.sources.openDuplicateReviewAria":
    "Review duplicates for {sourceId}",
  "itemDetail.artwork.title": "Artwork and readiness",
  "itemDetail.artwork.description":
    "Public image refs and follow-on readiness. Selection and repair mutations remain split.",
  "itemDetail.artwork.routeUnavailable": "route path unavailable",
  "itemDetail.unknownType": "unknown type",
  "itemDetail.support.title": "Support links",
  "itemDetail.support.description":
    "Support links stay read-only and route-local.",
  "itemDetail.support.catalogGovernance": "Catalog Governance",
  "itemDetail.support.catalogGovernanceValue":
    "Review unknown and low-confidence queue",
  "itemDetail.support.artworkGallery": "Artwork Gallery",
  "itemDetail.support.artworkGalleryValue":
    "Review Managed Artwork candidates and Selected Artwork",
  "itemDetail.support.generatedArtifacts": "Generated Artifacts",
  "itemDetail.support.generatedArtifactsValue":
    "Review route-level automation proposals",
  "itemDetail.support.sourceDuplicateReview": "Source Duplicate Review",
  "itemDetail.support.sourceDuplicateReviewValue":
    "Open the first source's duplicate suggestion review",
  "itemDetail.support.playbackSupport": "Playback Support Evidence",
  "itemDetail.support.playbackSupportValue":
    "Open redacted playback evidence for the selected source",
  "itemDetail.support.playbackSessions": "Playback Sessions",
  "itemDetail.support.playbackSessionsValue":
    "Open source-scoped diagnostics list",
  "itemDetail.support.open": "Open",
  "itemDetail.support.openArtworkGalleryAria": "Open Artwork Gallery",
  "itemDetail.support.openSourceDuplicateReviewAria":
    "Open Source Duplicate Review",
  "itemDetail.support.openPlaybackSupportAria":
    "Open Playback Support Evidence",
  "itemDetail.none": "none",
  "itemDetail.duration.unknown": "duration unknown",
  "itemDetail.duration.hoursMinutes": "{hours}h {minutes}m",
  "itemDetail.duration.minutes": "{minutes}m",
  "itemDetail.sizeUnavailable": "size unavailable",
} as const;

export type ItemDetailMessageId = keyof typeof enItemDetailMessages;

export const zhHansItemDetailMessages = {
  "itemDetail.backToCatalog": "返回目录",
  "itemDetail.refresh": "刷新",
  "itemDetail.description":
    "来自安全公共读取桥接、用于管理支持的 Media Item 事实。修复和播放工作流不在此路由中。",
  "itemDetail.kicker": "Media Item",
  "itemDetail.fallback": "{error}。正在显示需要的确定性 mock 回退。",
  "itemDetail.loading": "正在加载 Media Item 详情",
  "itemDetail.missing": "Media Item {itemId} 无法加载。",
  "itemDetail.dataSourceUnavailable": "Media Item 详情路由数据源不可用",
  "itemDetail.facts.title": "条目事实",
  "itemDetail.facts.description": "Provider 中立的 Media Item 身份和关系计数。",
  "itemDetail.facts.mediaItemId": "Media Item ID",
  "itemDetail.facts.parent": "父级",
  "itemDetail.facts.release": "发行",
  "itemDetail.facts.runtime": "片长",
  "itemDetail.facts.sources": "来源",
  "itemDetail.facts.images": "图片",
  "itemDetail.parent.none": "无父级",
  "itemDetail.release.none": "无发行日期",
  "itemDetail.runtime.unknown": "片长未知",
  "itemDetail.runtime.minutes": "{minutes} 分钟",
  "itemDetail.sources.count": "{count} 个 Media Source",
  "itemDetail.images.count": "{count} 个公共图片引用",
  "itemDetail.redactedSummary": "已脱敏摘要",
  "itemDetail.canonical.title": "Canonical Metadata",
  "itemDetail.canonical.description":
    "只显示 Canonical Metadata 计数和展示标签，不显示原始 provider payload。",
  "itemDetail.canonical.genres": "类型",
  "itemDetail.canonical.tags": "标签",
  "itemDetail.canonical.credits": "演职员",
  "itemDetail.canonical.collections": "合集",
  "itemDetail.canonical.studios": "制作方",
  "itemDetail.canonical.creditAs": "{name} 饰 {character}",
  "itemDetail.canonical.creditRole": "{name}（{role}）",
  "itemDetail.canonical.externalEvidence": "外部证据",
  "itemDetail.canonical.externalEvidenceValue":
    "{ratings} 个评分 / {externalIds} 个外部 ID",
  "itemDetail.canonical.countsOnly": "仅计数",
  "itemDetail.sources.title": "Media Sources",
  "itemDetail.sources.description":
    "安全的 Media Source 文件名和有界 probe 摘要。永不渲染原始 locator。",
  "itemDetail.sources.probeUnavailable": "Probe 摘要不可用",
  "itemDetail.sources.fingerprinted": "已指纹化",
  "itemDetail.sources.noFingerprint": "无指纹",
  "itemDetail.sources.unknownContainer": "未知容器",
  "itemDetail.sources.probe": "{container} / {duration} / {streams} 条流",
  "itemDetail.sources.openDuplicateReview": "审查重复来源",
  "itemDetail.sources.openDuplicateReviewAria": "审查 {sourceId} 的重复来源",
  "itemDetail.artwork.title": "Artwork 与就绪度",
  "itemDetail.artwork.description":
    "公共图片引用和后续就绪度。选择和修复 mutation 保持拆分。",
  "itemDetail.artwork.routeUnavailable": "route path 不可用",
  "itemDetail.unknownType": "未知类型",
  "itemDetail.support.title": "支持链接",
  "itemDetail.support.description": "支持链接保持只读并限定在路由本地。",
  "itemDetail.support.catalogGovernance": "Catalog Governance",
  "itemDetail.support.catalogGovernanceValue": "审查 unknown 和低置信度队列",
  "itemDetail.support.artworkGallery": "Artwork Gallery",
  "itemDetail.support.artworkGalleryValue":
    "审查 Managed Artwork 候选和 Selected Artwork",
  "itemDetail.support.generatedArtifacts": "Generated Artifacts",
  "itemDetail.support.generatedArtifactsValue": "审查路由级自动化 proposal",
  "itemDetail.support.sourceDuplicateReview": "Source Duplicate 审核",
  "itemDetail.support.sourceDuplicateReviewValue":
    "打开第一个 source 的重复建议审核",
  "itemDetail.support.playbackSupport": "播放支持证据",
  "itemDetail.support.playbackSupportValue": "打开所选 source 的脱敏播放证据",
  "itemDetail.support.playbackSessions": "Playback Sessions",
  "itemDetail.support.playbackSessionsValue": "打开按 source 限定的诊断列表",
  "itemDetail.support.open": "打开",
  "itemDetail.support.openArtworkGalleryAria": "打开 Artwork Gallery",
  "itemDetail.support.openSourceDuplicateReviewAria": "打开 Source Duplicate 审核",
  "itemDetail.support.openPlaybackSupportAria": "打开播放支持证据",
  "itemDetail.none": "无",
  "itemDetail.duration.unknown": "时长未知",
  "itemDetail.duration.hoursMinutes": "{hours} 小时 {minutes} 分钟",
  "itemDetail.duration.minutes": "{minutes} 分钟",
  "itemDetail.sizeUnavailable": "大小不可用",
} satisfies Record<ItemDetailMessageId, string>;

export const itemDetailMessageCatalogs = {
  "en-US": enItemDetailMessages,
  "zh-Hans": zhHansItemDetailMessages,
} as const;
