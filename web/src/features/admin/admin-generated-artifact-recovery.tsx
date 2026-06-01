"use client"

import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import { RefreshCw, RotateCcw, Sparkles } from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  ADMIN_GENERATED_ARTIFACT_APPLY_RECOVERY_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminGeneratedArtifactMetadataApplyRecoveryEntryReadModel,
} from "@/src/api/admin/read-models-data-source"
import type { AdminGeneratedArtifactApplyRecoveryQuery } from "@/src/api/admin/generated/contract"

const DEFAULT_LIMIT = 50
const DEFAULT_OFFSET = 0
const LIMIT_OPTIONS = [25, 50, 100]
const ATTENTION_OPTIONS = ["all", "needs_repair", "needs_review", "replay_only", "resolved"] as const

export type AdminGeneratedArtifactRecoveryAttention = (typeof ATTENTION_OPTIONS)[number]

export interface AdminGeneratedArtifactRecoveryRouteState {
  attention?: AdminGeneratedArtifactRecoveryAttention
  limit?: number
  offset?: number
}

interface AdminGeneratedArtifactRecoveryProps {
  routeState?: AdminGeneratedArtifactRecoveryRouteState
  onRouteStateChange?: (state: AdminGeneratedArtifactRecoveryRouteState) => void
  onApplyRequest?: (artifactId: string) => void
  onBackToArtifacts?: () => void
}

export function AdminGeneratedArtifactRecovery({
  routeState,
  onRouteStateChange,
  onApplyRequest,
  onBackToArtifacts,
}: AdminGeneratedArtifactRecoveryProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeRouteState(routeState), [routeState])
  const query = useMemo(() => routeStateToQuery(normalizedRouteState), [normalizedRouteState])
  const readDataSource = useMemo(() => createAdminReadModelsDataSource(), [])
  const { data = ADMIN_GENERATED_ARTIFACT_APPLY_RECOVERY_READ_MODEL_FIXTURE, isLoading, isFetching, refetch } = useQuery({
    queryKey: ["nako", "admin", "generated-artifact-apply-recovery", query],
    queryFn: () => readDataSource.loadGeneratedArtifactApplyRecovery(query),
    staleTime: 10 * 1000,
    retry: 0,
  })

  const canPageBack = normalizedRouteState.offset > 0
  const canPageForward = data.page.returned >= normalizedRouteState.limit

  const updateRouteState = (next: Partial<Required<AdminGeneratedArtifactRecoveryRouteState>>) => {
    onRouteStateChange?.({
      ...normalizedRouteState,
      ...next,
    })
  }

  const movePage = (direction: "previous" | "next") => {
    updateRouteState({
      offset:
        direction === "previous"
          ? Math.max(DEFAULT_OFFSET, normalizedRouteState.offset - normalizedRouteState.limit)
          : normalizedRouteState.offset + normalizedRouteState.limit,
    })
  }

  const changeAttention = (value: string) => {
    updateRouteState({
      attention: parseAttention(value),
      offset: DEFAULT_OFFSET,
    })
  }

  const changeLimit = (value: string) => {
    const parsed = Number.parseInt(value, 10)
    updateRouteState({
      limit: Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_LIMIT,
      offset: DEFAULT_OFFSET,
    })
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="max-w-3xl">
          <div className="mb-2 flex items-center gap-2">
            <RotateCcw className="h-5 w-5 text-primary" />
            <h1 className="text-xl font-semibold text-foreground">生成产物恢复</h1>
          </div>
          <p className="text-sm text-muted-foreground">
            查看 Metadata Authority apply 的失败、陈旧、跳过和重放状态；本页只读，不执行修复。
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={data.source === "live" ? "default" : "secondary"}>
            {data.source === "live" ? "Live Admin API" : "Fixture"}
          </Badge>
          <Badge variant="outline">只读恢复队列</Badge>
          <Button type="button" variant="outline" size="sm" onClick={onBackToArtifacts}>
            返回生成产物
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-2"
            onClick={() => {
              void refetch()
            }}
          >
            <RefreshCw className="h-3.5 w-3.5" />
            {isFetching ? "刷新中" : "刷新"}
          </Button>
        </div>
      </div>

      {data.fallback && data.error && (
        <div className="rounded-lg border border-warning/30 bg-warning/5 p-4 text-sm">
          <div>
            <p className="font-medium text-foreground">Admin API 不可用，正在显示 fixture 恢复队列</p>
            <p className="mt-1 text-muted-foreground">{data.error}</p>
          </div>
        </div>
      )}

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="grid gap-3 border-b border-border/50 p-4 sm:grid-cols-2 lg:grid-cols-5">
          <SummaryFact label="返回" value={String(data.summary.returnedEntryCount)} />
          <SummaryFact label="需要修复" value={String(data.summary.needsRepairCount)} tone="repair" />
          <SummaryFact label="需要复核" value={String(data.summary.needsReviewCount)} tone="review" />
          <SummaryFact label="仅重放" value={String(data.summary.replayOnlyCount)} />
          <SummaryFact label="已解决" value={String(data.summary.resolvedCount)} tone="resolved" />
        </div>

        <div className="flex flex-col gap-3 border-b border-border/50 p-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">恢复队列</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              返回 {data.page.returned} 项，偏移 {data.page.offset}，每页 {data.page.limit}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Select value={normalizedRouteState.attention} onValueChange={changeAttention}>
              <SelectTrigger className="h-8 w-[150px]" aria-label="恢复关注状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {ATTENTION_OPTIONS.map((attention) => (
                  <SelectItem key={attention} value={attention}>
                    {attentionLabel(attention)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={String(normalizedRouteState.limit)} onValueChange={changeLimit}>
              <SelectTrigger className="h-8 w-[116px]" aria-label="恢复队列每页数量">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LIMIT_OPTIONS.map((limit) => (
                  <SelectItem key={limit} value={String(limit)}>
                    每页 {limit}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Badge variant="outline">Admin API {data.versions.adminApi}</Badge>
            <Badge variant="outline">Public API {data.versions.publicApi}</Badge>
          </div>
        </div>

        {isLoading ? (
          <RecoverySkeleton />
        ) : data.entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 p-10 text-center">
            <p className="text-sm font-medium text-foreground">没有恢复条目</p>
            <p className="max-w-md text-sm text-muted-foreground">
              当前筛选下没有 Generated Artifact apply 状态需要展示。
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>关注</TableHead>
                  <TableHead>产物</TableHead>
                  <TableHead>来源</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>计划</TableHead>
                  <TableHead>错误</TableHead>
                  <TableHead>更新时间</TableHead>
                  <TableHead className="sticky right-0 z-10 min-w-[8rem] bg-card text-right">动作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.entries.map((entry) => (
                  <RecoveryRow
                    key={`${entry.source}-${entry.artifactId}-${entry.outcomeId ?? entry.batchId ?? entry.updatedAt}`}
                    entry={entry}
                    onApplyRequest={onApplyRequest}
                  />
                ))}
              </TableBody>
            </Table>
          </div>
        )}

        <div className="flex items-center justify-between border-t border-border/50 p-4">
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-2"
            disabled={!canPageBack}
            onClick={() => movePage("previous")}
          >
            上一页
          </Button>
          <span className="text-xs text-muted-foreground">
            Offset {normalizedRouteState.offset}
          </span>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-2"
            disabled={!canPageForward}
            onClick={() => movePage("next")}
          >
            下一页
          </Button>
        </div>
      </section>
    </div>
  )
}

function RecoveryRow({
  entry,
  onApplyRequest,
}: {
  entry: AdminGeneratedArtifactMetadataApplyRecoveryEntryReadModel
  onApplyRequest?: (artifactId: string) => void
}) {
  return (
    <TableRow>
      <TableCell>
        <div className="space-y-1">
          <Badge variant={attentionBadgeVariant(entry.attention)}>{attentionLabel(entry.attention)}</Badge>
          <div className="text-xs text-muted-foreground">{entry.reason}</div>
        </div>
      </TableCell>
      <TableCell>
        <div className="max-w-[18rem] space-y-1 text-xs">
          <div className="truncate font-mono font-medium text-foreground" title={entry.artifactId}>
            {entry.artifactId}
          </div>
          <div className="truncate font-mono text-muted-foreground" title={entry.itemId ?? "item unknown"}>
            item {entry.itemId ?? "unknown"}
          </div>
        </div>
      </TableCell>
      <TableCell>
        <div className="max-w-[16rem] space-y-1 text-xs">
          <Badge variant="outline">{entry.source}</Badge>
          <div className="truncate font-mono text-muted-foreground" title={entry.outcomeId ?? "outcome none"}>
            outcome {entry.outcomeId ?? "none"}
          </div>
          <div className="truncate font-mono text-muted-foreground" title={entry.batchId ?? "batch none"}>
            batch {entry.batchId ?? "none"}
          </div>
        </div>
      </TableCell>
      <TableCell>
        <div className="space-y-1 text-xs">
          <StatusLine label="batch" value={entry.batchItemStatus} />
          <StatusLine label="outcome" value={entry.outcomeStatus} />
        </div>
      </TableCell>
      <TableCell>
        {entry.plan ? (
          <div className="max-w-[18rem] space-y-1 text-xs">
            <div className="flex flex-wrap gap-1.5">
              <Badge variant={entry.plan.executable ? "default" : "secondary"}>
                {entry.plan.status}
              </Badge>
              <Badge variant="outline">
                fields {entry.plan.applyFieldCount}/{entry.plan.skippedFieldCount}/{entry.plan.noopFieldCount}
              </Badge>
            </div>
            <div className="truncate text-muted-foreground">
              {entry.plan.reasons.length > 0 ? entry.plan.reasons.join(", ") : "no reasons"}
            </div>
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">no plan snapshot</span>
        )}
      </TableCell>
      <TableCell>
        <div className="max-w-[18rem] space-y-1 text-xs">
          <div className="truncate font-mono text-muted-foreground" title={entry.errorCode ?? "none"}>
            {entry.errorCode ?? "none"}
          </div>
          <div className="truncate text-muted-foreground" title={entry.errorMessage ?? "no message"}>
            {entry.errorMessage ?? "no message"}
          </div>
        </div>
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">
        {formatDateTime(entry.updatedAt)}
      </TableCell>
      <TableCell className="sticky right-0 z-10 min-w-[8rem] bg-card text-right">
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="h-8 gap-1.5 px-2 text-xs"
          onClick={() => onApplyRequest?.(entry.artifactId)}
        >
          <Sparkles className="h-3.5 w-3.5" />
          应用计划
        </Button>
      </TableCell>
    </TableRow>
  )
}

function SummaryFact({
  label,
  value,
  tone = "default",
}: {
  label: string
  value: string
  tone?: "default" | "repair" | "review" | "resolved"
}) {
  const toneClass =
    tone === "repair"
      ? "text-destructive"
      : tone === "review"
        ? "text-warning"
        : tone === "resolved"
          ? "text-success"
          : "text-foreground"

  return (
    <div className="min-w-0 rounded-md border border-border/50 p-3">
      <div className="text-xs font-medium text-muted-foreground">{label}</div>
      <div className={`mt-1 font-mono text-lg font-semibold ${toneClass}`}>{value}</div>
    </div>
  )
}

function StatusLine({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="flex items-center gap-1.5">
      <span className="text-muted-foreground">{label}</span>
      <Badge variant="outline">{value ?? "none"}</Badge>
    </div>
  )
}

function RecoverySkeleton() {
  return (
    <div className="space-y-3 p-4">
      {Array.from({ length: 4 }, (_, index) => (
        <div key={index} className="grid gap-3 md:grid-cols-[1fr_2fr_1fr_1fr]">
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
        </div>
      ))}
    </div>
  )
}

function normalizeRouteState(
  routeState?: AdminGeneratedArtifactRecoveryRouteState,
): Required<AdminGeneratedArtifactRecoveryRouteState> {
  return {
    attention: parseAttention(routeState?.attention),
    limit: routeState?.limit && routeState.limit > 0 ? routeState.limit : DEFAULT_LIMIT,
    offset: routeState?.offset && routeState.offset > 0 ? routeState.offset : DEFAULT_OFFSET,
  }
}

function routeStateToQuery(
  routeState: Required<AdminGeneratedArtifactRecoveryRouteState>,
): AdminGeneratedArtifactApplyRecoveryQuery {
  return {
    attention: routeState.attention === "all" ? undefined : routeState.attention,
    limit: routeState.limit,
    offset: routeState.offset,
  }
}

function parseAttention(value: unknown): AdminGeneratedArtifactRecoveryAttention {
  return typeof value === "string" && ATTENTION_OPTIONS.includes(value as AdminGeneratedArtifactRecoveryAttention)
    ? (value as AdminGeneratedArtifactRecoveryAttention)
    : "needs_repair"
}

function attentionLabel(value: string) {
  switch (value) {
    case "all":
      return "全部"
    case "needs_repair":
      return "需要修复"
    case "needs_review":
      return "需要复核"
    case "replay_only":
      return "仅重放"
    case "resolved":
      return "已解决"
    default:
      return value
  }
}

function attentionBadgeVariant(value: string): "default" | "secondary" | "destructive" | "outline" {
  switch (value) {
    case "needs_repair":
      return "destructive"
    case "needs_review":
      return "secondary"
    case "resolved":
      return "default"
    default:
      return "outline"
  }
}

function formatDateTime(value: string) {
  return value.includes("T") ? value.replace("T", " ").slice(5, 16) : value
}
