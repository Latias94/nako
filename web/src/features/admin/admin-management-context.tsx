import { ArrowLeft, ExternalLink, Link2 } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import {
  hasAdminManagementContextState,
  type AdminManagementContextRouteState,
} from "./admin-management-context-state"

export type AdminManagementContextAction = {
  label: string
  onClick?: () => void
  href?: string
  disabled?: boolean
}

export interface AdminManagementContextNoticeProps {
  state?: AdminManagementContextRouteState
  title?: string
  description?: string
  actions?: AdminManagementContextAction[]
  className?: string
}

export function AdminManagementContextNotice({
  state,
  title = "管理上下文",
  description,
  actions = [],
  className,
}: AdminManagementContextNoticeProps) {
  if (!hasAdminManagementContextState(state)) {
    return null
  }

  return (
    <section
      className={cn(
        "flex flex-col gap-3 rounded-lg border border-primary/20 bg-primary/5 px-4 py-3 text-sm",
        className,
      )}
      aria-label={title}
    >
      <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
        <div className="min-w-0 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="secondary" className="gap-1">
              <Link2 className="h-3 w-3" />
              Management Link
            </Badge>
            <h2 className="text-sm font-semibold text-foreground">{title}</h2>
          </div>
          {description ? (
            <p className="text-xs text-muted-foreground">{description}</p>
          ) : null}
          <div className="flex flex-wrap gap-1.5">
            <ContextBadge label="library_id" value={state?.libraryId} />
            <ContextBadge label="item_id" value={state?.itemId} />
            <ContextBadge label="source_id" value={state?.sourceId} />
            <ContextBadge label="playback_session_id" value={state?.playbackSessionId} />
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          {state?.libraryId ? (
            <Button asChild variant="outline" size="sm" className="h-8 gap-1.5 text-xs">
              <a href={mediaLibraryHref(state.libraryId)}>
                <ArrowLeft className="h-3.5 w-3.5" />
                返回媒体库
              </a>
            </Button>
          ) : null}
          {state?.itemId ? (
            <Button asChild variant="outline" size="sm" className="h-8 gap-1.5 text-xs">
              <a href={mediaItemHref(state.itemId, state.mediaType ?? "movie")}>
                <ArrowLeft className="h-3.5 w-3.5" />
                返回媒体详情
              </a>
            </Button>
          ) : null}
          {actions.map((action) => (
            <ManagementContextActionButton key={action.label} action={action} />
          ))}
        </div>
      </div>
    </section>
  )
}

function ContextBadge({ label, value }: { label: string; value?: string }) {
  if (!value) {
    return null
  }

  return (
    <Badge variant="outline" className="gap-1.5 bg-background/70 font-normal">
      <span className="text-muted-foreground">{label}</span>
      <span className="font-mono text-foreground">{value}</span>
    </Badge>
  )
}

function ManagementContextActionButton({
  action,
}: {
  action: AdminManagementContextAction
}) {
  if (action.href) {
    return (
      <Button asChild variant="default" size="sm" className="h-8 gap-1.5 text-xs">
        <a href={action.href}>
          {action.label}
          <ExternalLink className="h-3 w-3 opacity-70" />
        </a>
      </Button>
    )
  }

  return (
    <Button
      type="button"
      variant="default"
      size="sm"
      className="h-8 text-xs"
      disabled={action.disabled}
      onClick={action.onClick}
    >
      {action.label}
    </Button>
  )
}

function mediaLibraryHref(libraryId: string) {
  const search = new URLSearchParams({ id: libraryId })
  return `/media/library?${search.toString()}`
}

function mediaItemHref(itemId: string, mediaType: string) {
  const search = new URLSearchParams({ id: itemId, type: mediaType })
  return `/media/detail?${search.toString()}`
}
