"use client"

import { useMemo } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  AlertTriangle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Database,
  Loader2,
  RefreshCw,
  ShieldCheck,
  Sparkles,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminGeneratedArtifactProposalReadModel,
} from "@/src/api/admin/read-models-data-source"
import type { AdminGeneratedArtifactProposalsQuery } from "@/src/api/admin/generated/contract"

const DEFAULT_LIMIT = 50
const DEFAULT_OFFSET = 0
const LIMIT_OPTIONS = [25, 50, 100]

export interface AdminGeneratedArtifactsRouteState {
  limit?: number
  offset?: number
}

interface AdminGeneratedArtifactsProps {
  routeState?: AdminGeneratedArtifactsRouteState
  onRouteStateChange?: (state: AdminGeneratedArtifactsRouteState) => void
}

export function AdminGeneratedArtifacts({
  routeState,
  onRouteStateChange,
}: AdminGeneratedArtifactsProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeRouteState(routeState), [routeState])
  const query = useMemo(() => routeStateToQuery(normalizedRouteState), [normalizedRouteState])
  const { data = ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE, isLoading, isFetching, refetch } = useQuery({
    queryKey: ["nako", "admin", "generated-artifacts", query],
    queryFn: () => createAdminReadModelsDataSource().loadGeneratedArtifacts(query),
    staleTime: 15 * 1000,
    retry: 0,
  })

  const movePage = (direction: "previous" | "next") => {
    const nextOffset =
      direction === "previous"
        ? Math.max(DEFAULT_OFFSET, normalizedRouteState.offset - normalizedRouteState.limit)
        : normalizedRouteState.offset + normalizedRouteState.limit

    onRouteStateChange?.({
      ...normalizedRouteState,
      offset: nextOffset,
    })
  }

  const changeLimit = (value: string) => {
    const parsed = Number.parseInt(value, 10)
    onRouteStateChange?.({
      limit: Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_LIMIT,
      offset: DEFAULT_OFFSET,
    })
  }

  const canPageBack = normalizedRouteState.offset > 0
  const canPageForward = data.page.returned >= normalizedRouteState.limit

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="max-w-3xl">
          <div className="mb-2 flex items-center gap-2">
            <Sparkles className="h-5 w-5 text-primary" />
            <h1 className="text-xl font-semibold text-foreground">生成产物</h1>
          </div>
          <p className="text-sm text-muted-foreground">
            查看 Generated Artifact 提案的目标、来源、payload 摘要、就绪状态和审核边界。
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={data.source === "live" ? "default" : "secondary"}>
            {data.source === "live" ? "Live Admin API" : "Fixture"}
          </Badge>
          <Badge variant="outline">
            <ShieldCheck className="h-3 w-3" />
            只读
          </Badge>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-2"
            onClick={() => {
              void refetch()
            }}
          >
            {isFetching ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            刷新
          </Button>
        </div>
      </div>

      {data.fallback && data.error && (
        <div className="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/5 p-4 text-sm">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
          <div>
            <p className="font-medium text-foreground">Admin API 不可用，正在显示 fixture 数据</p>
            <p className="mt-1 text-muted-foreground">{data.error}</p>
          </div>
        </div>
      )}

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex flex-col gap-3 border-b border-border/50 p-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">提案队列</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              返回 {data.page.returned} 项，偏移 {data.page.offset}，每页 {data.page.limit}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="outline">Admin API {data.versions.adminApi}</Badge>
            <Badge variant="outline">Public API {data.versions.publicApi}</Badge>
            <Select value={String(normalizedRouteState.limit)} onValueChange={changeLimit}>
              <SelectTrigger className="h-8 w-[116px]" aria-label="生成产物每页数量">
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
          </div>
        </div>

        {isLoading ? (
          <GeneratedArtifactsSkeleton />
        ) : data.proposals.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 p-10 text-center">
            <Database className="h-8 w-8 text-muted-foreground" />
            <p className="text-sm font-medium text-foreground">没有生成产物提案</p>
            <p className="max-w-md text-sm text-muted-foreground">
              等待 Automation Provider 或 Addon 提交新的 Generated Artifact。
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>产物</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>目标</TableHead>
                  <TableHead>Provider</TableHead>
                  <TableHead>Payload</TableHead>
                  <TableHead>就绪</TableHead>
                  <TableHead>指纹</TableHead>
                  <TableHead>更新时间</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.proposals.map((proposal) => (
                  <GeneratedArtifactRow key={proposal.id} proposal={proposal} />
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
            <ChevronLeft className="h-3.5 w-3.5" />
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
            <ChevronRight className="h-3.5 w-3.5" />
          </Button>
        </div>
      </section>
    </div>
  )
}

function GeneratedArtifactRow({ proposal }: { proposal: AdminGeneratedArtifactProposalReadModel }) {
  return (
    <TableRow>
      <TableCell>
        <div className="space-y-1">
          <div className="font-mono text-xs font-medium text-foreground">{proposal.id}</div>
          <div className="text-xs text-muted-foreground">{proposal.kind}</div>
          <div className="text-xs text-muted-foreground">{proposal.capability}</div>
        </div>
      </TableCell>
      <TableCell>
        <Badge variant={statusBadgeVariant(proposal.status)}>{proposal.status}</Badge>
      </TableCell>
      <TableCell>
        <div className="space-y-1 text-xs">
          <Badge variant="outline">{proposal.target.kind}</Badge>
          <div className="font-mono text-muted-foreground">{proposal.target.libraryId ?? "library unknown"}</div>
          <div className="font-mono text-muted-foreground">{proposal.target.itemId ?? "item unknown"}</div>
          <div className="font-mono text-muted-foreground">{proposal.target.sourceId ?? "source unknown"}</div>
        </div>
      </TableCell>
      <TableCell>
        <div className="space-y-1 text-xs">
          <div className="font-medium text-foreground">{proposal.provenance.providerName ?? proposal.provenance.providerId}</div>
          <div className="font-mono text-muted-foreground">{proposal.provenance.jobId}</div>
          <div className="text-muted-foreground">attempt {proposal.provenance.attemptCount ?? 0}</div>
        </div>
      </TableCell>
      <TableCell>
        <div className="space-y-1 text-xs">
          <div className="text-foreground">{proposal.payload.shape} · {formatBytes(proposal.payload.payloadBytes)}</div>
          <div className="text-muted-foreground">
            {proposal.payload.validJson ? "valid JSON" : "invalid JSON"} · confidence {confidenceLabel(proposal.payload.confidenceMilli)}
          </div>
          <div className="text-muted-foreground">
            fields {proposal.payload.objectFieldCount ?? 0} · items {proposal.payload.arrayItemCount ?? 0}
          </div>
        </div>
      </TableCell>
      <TableCell>
        <div className="space-y-1">
          <ReadinessBadge ready={proposal.readiness.actionable} label={proposal.readiness.status} />
          <div className="max-w-[16rem] truncate text-xs text-muted-foreground">
            {proposal.readiness.reasons.length > 0 ? proposal.readiness.reasons.join(", ") : "no reasons"}
          </div>
        </div>
      </TableCell>
      <TableCell>
        <div className="max-w-[18rem] space-y-1 text-xs">
          <Fingerprint label="payload" value={proposal.payload.payloadFingerprint} />
          <Fingerprint label="prompt" value={proposal.provenance.promptFingerprint} />
          <Fingerprint label="idem" value={proposal.provenance.idempotencyKeyFingerprint} />
        </div>
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">{formatDateTime(proposal.updatedAt)}</TableCell>
    </TableRow>
  )
}

function Fingerprint({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="truncate font-mono text-muted-foreground">
      {label}: {value ?? "none"}
    </div>
  )
}

function ReadinessBadge({ ready, label }: { ready: boolean; label: string }) {
  return (
    <Badge variant={ready ? "secondary" : "outline"} className="gap-1">
      {ready ? <CheckCircle2 className="h-3 w-3" /> : <AlertTriangle className="h-3 w-3" />}
      {label}
    </Badge>
  )
}

function GeneratedArtifactsSkeleton() {
  return (
    <div className="space-y-3 p-4">
      {Array.from({ length: 4 }, (_, index) => (
        <div key={index} className="grid gap-3 md:grid-cols-[2fr_1fr_1fr_1fr]">
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
          <Skeleton className="h-12" />
        </div>
      ))}
    </div>
  )
}

export function normalizeGeneratedArtifactsRouteState(
  routeState?: AdminGeneratedArtifactsRouteState,
): Required<AdminGeneratedArtifactsRouteState> {
  return normalizeRouteState(routeState)
}

function normalizeRouteState(routeState?: AdminGeneratedArtifactsRouteState): Required<AdminGeneratedArtifactsRouteState> {
  return {
    limit: routeState?.limit && routeState.limit > 0 ? routeState.limit : DEFAULT_LIMIT,
    offset: routeState?.offset && routeState.offset > 0 ? routeState.offset : DEFAULT_OFFSET,
  }
}

function routeStateToQuery(routeState: Required<AdminGeneratedArtifactsRouteState>): AdminGeneratedArtifactProposalsQuery {
  return {
    limit: routeState.limit,
    offset: routeState.offset,
  }
}

function statusBadgeVariant(status: string): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "accepted":
      return "default"
    case "rejected":
    case "failed":
      return "destructive"
    case "pending_review":
    case "ready":
      return "secondary"
    default:
      return "outline"
  }
}

function confidenceLabel(value: number | null) {
  return value === null ? "unknown" : `${value} / 1000`
}

function formatBytes(value: number) {
  const units = ["B", "KiB", "MiB", "GiB"]
  let amount = value
  let unitIndex = 0

  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024
    unitIndex += 1
  }

  const precision = amount >= 10 || unitIndex === 0 ? 0 : 1
  return `${amount.toFixed(precision)} ${units[unitIndex]}`
}

function formatDateTime(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value
  }

  return new Intl.DateTimeFormat("zh-CN", {
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  }).format(date)
}
