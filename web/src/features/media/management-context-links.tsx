"use client"

import {
  Activity,
  ExternalLink,
  ListChecks,
  RefreshCw,
  ShieldCheck,
  SlidersHorizontal,
  Wrench,
  type LucideIcon,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { useManagementContextLinks } from "@/lib/use-media"
import type {
  PublicManagementContext,
  PublicManagementContextLink,
} from "@/src/api/public/management-context-data-source"
import {
  resolveManagementContextLink,
  type ResolvedManagementContextRoute,
} from "@/src/shell/management-context-routes"

type ManagementContextLinksTone = "surface" | "hero" | "inline"

export interface ManagementContextLinksProps {
  context: PublicManagementContext
  routeNames?: string[]
  title?: string
  tone?: ManagementContextLinksTone
  className?: string
}

type ManagementLinkPresentation = {
  label: string
  icon: LucideIcon
}

const MANAGEMENT_LINK_PRESENTATION: Record<string, ManagementLinkPresentation> = {
  "library.scan": {
    label: "扫描媒体库",
    icon: RefreshCw,
  },
  "library.metadata_profile": {
    label: "元数据配置",
    icon: SlidersHorizontal,
  },
  "item.metadata_refresh": {
    label: "刷新元数据",
    icon: RefreshCw,
  },
  "jobs.filtered": {
    label: "相关任务",
    icon: ListChecks,
  },
  "playback.support": {
    label: "播放诊断",
    icon: Wrench,
  },
  "playback.runtime": {
    label: "转码运行时",
    icon: Activity,
  },
  "access.library_policies": {
    label: "访问策略",
    icon: ShieldCheck,
  },
}

export function ManagementContextLinks({
  context,
  routeNames,
  title = "管理入口",
  tone = "surface",
  className,
}: ManagementContextLinksProps) {
  const query = useManagementContextLinks(context)
  const payload = query.data

  if (query.isLoading) {
    return (
      <div className={cn(containerClass(tone), className)} aria-label={title}>
        <div className="h-7 w-28 animate-pulse rounded-md bg-muted/60" />
        <div className="h-7 w-36 animate-pulse rounded-md bg-muted/50" />
      </div>
    )
  }

  if (!payload || (payload.fallback && payload.error)) {
    return null
  }

  const allowed = routeNames ? new Set(routeNames) : null
  const items = payload.links
    .filter((link) => !allowed || allowed.has(link.routeName))
    .flatMap((link) => managementLinkItem(link, context))

  if (items.length === 0) {
    return null
  }

  return (
    <section className={cn(containerClass(tone), className)} aria-label={title}>
      <div className="flex min-w-0 items-center gap-2">
        <span className={cn("text-xs font-medium", titleClass(tone))}>{title}</span>
        {payload.fallback ? (
          <Badge variant="outline" className={cn("h-5 text-[10px]", badgeClass(tone))}>
            演示链接
          </Badge>
        ) : null}
      </div>
      <div className="flex min-w-0 flex-wrap items-center gap-2">
        {items.map((item) => (
          <ManagementContextLinkButton
            key={item.link.routeName}
            link={item.link}
            presentation={item.presentation}
            route={item.route}
            tone={tone}
          />
        ))}
      </div>
    </section>
  )
}

function ManagementContextLinkButton({
  link,
  presentation,
  route,
  tone,
}: {
  link: PublicManagementContextLink
  presentation: ManagementLinkPresentation
  route: ResolvedManagementContextRoute | null
  tone: ManagementContextLinksTone
}) {
  const Icon = presentation.icon

  if (!route) {
    const disabledReason = managementDisabledReason(link.disabledReason)

    return (
      <Button
        type="button"
        variant="outline"
        size="sm"
        className={cn("h-8 cursor-not-allowed gap-1.5 text-xs", buttonClass(tone, false))}
        disabled
      >
        <Icon className="h-3.5 w-3.5" />
        <span>{presentation.label}</span>
        {disabledReason ? <span className="text-[10px] opacity-75">{disabledReason}</span> : null}
      </Button>
    )
  }

  return (
    <Button asChild variant="outline" size="sm" className={cn("h-8 gap-1.5 text-xs", buttonClass(tone, true))}>
      <a href={managementRouteHref(route)} aria-label={`${presentation.label}，打开管理面板`}>
        <Icon className="h-3.5 w-3.5" />
        <span>{presentation.label}</span>
        <ExternalLink className="h-3 w-3 opacity-70" />
      </a>
    </Button>
  )
}

function managementLinkItem(
  link: PublicManagementContextLink,
  context: PublicManagementContext,
) {
  const presentation = MANAGEMENT_LINK_PRESENTATION[link.routeName]
  if (!presentation) {
    return []
  }

  const route = resolveManagementContextLink({
    ...link,
    target: {
      ...link.target,
      mediaType: context.mediaType,
    },
  })
  if (!route && link.enabled) {
    return []
  }

  return [{ link, presentation, route }]
}

function managementRouteHref(route: ResolvedManagementContextRoute) {
  const params = new URLSearchParams(route.search)
  const search = params.toString()

  return search ? `${route.path}?${search}` : route.path
}

function managementDisabledReason(reason: PublicManagementContextLink["disabledReason"]) {
  switch (reason) {
    case "insufficient_permission":
      return "权限不足"
    case "missing_context":
      return "缺少上下文"
    case null:
      return null
  }
}

function containerClass(tone: ManagementContextLinksTone) {
  switch (tone) {
    case "hero":
      return "flex max-w-2xl flex-wrap items-center gap-2 rounded-lg border border-white/15 bg-black/25 px-3 py-2 text-white backdrop-blur-sm"
    case "inline":
      return "flex flex-wrap items-center gap-2 border-t border-border/50 bg-muted/25 px-4 py-2"
    case "surface":
      return "flex flex-wrap items-center gap-2 rounded-lg border border-border bg-card px-3 py-2"
  }
}

function titleClass(tone: ManagementContextLinksTone) {
  return tone === "hero" ? "text-white/70" : "text-muted-foreground"
}

function badgeClass(tone: ManagementContextLinksTone) {
  return tone === "hero" ? "border-white/20 text-white/70" : "text-muted-foreground"
}

function buttonClass(tone: ManagementContextLinksTone, enabled: boolean) {
  if (tone === "hero") {
    return enabled
      ? "border-white/20 bg-white/10 text-white hover:bg-white/20 hover:text-white"
      : "border-white/15 bg-white/5 text-white/70"
  }

  return enabled ? "bg-background" : "bg-muted/40 text-muted-foreground"
}
