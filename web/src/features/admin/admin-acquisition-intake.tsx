"use client"

import { FormEvent, useEffect, useMemo, useState } from "react"
import { useQuery } from "@tanstack/react-query"
import {
  AlertTriangle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Database,
  FileSearch,
  Loader2,
  RefreshCw,
  ShieldCheck,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminAcquisitionIntakeCandidateReadModel,
} from "@/src/api/admin/read-models-data-source"
import type { AdminAcquisitionIntakeCandidatesQuery } from "@/src/api/admin/generated/contract"

const DEFAULT_LIMIT = 50
const DEFAULT_OFFSET = 0
const STATE_OPTIONS = ["discovered", "inspecting", "ready", "blocked", "accepted", "rejected", "failed", "superseded"]
const SOURCE_KIND_OPTIONS = [
  "watch_folder",
  "operator_submitted",
  "external_download_output",
  "addon_proposed",
  "resource_search_selection",
]

export interface AdminAcquisitionIntakeRouteState {
  libraryId?: string
  state?: string
  sourceKind?: string
  managedImportArtifactId?: string
  limit?: number
  offset?: number
}

interface AdminAcquisitionIntakeProps {
  routeState?: AdminAcquisitionIntakeRouteState
  onRouteStateChange?: (state: AdminAcquisitionIntakeRouteState) => void
}

interface FilterDraft {
  libraryId: string
  state: string
  sourceKind: string
  managedImportArtifactId: string
  limit: string
}

export function AdminAcquisitionIntake({ routeState, onRouteStateChange }: AdminAcquisitionIntakeProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeRouteState(routeState), [routeState])
  const query = useMemo(() => routeStateToQuery(normalizedRouteState), [normalizedRouteState])
  const [filterDraft, setFilterDraft] = useState(() => routeStateToDraft(normalizedRouteState))
  const { data = ADMIN_ACQUISITION_INTAKE_READ_MODEL_FIXTURE, isLoading, isFetching, refetch } = useQuery({
    queryKey: ["nako", "admin", "acquisition-intake", query],
    queryFn: () => createAdminReadModelsDataSource().loadAcquisitionIntake(query),
    staleTime: 15 * 1000,
    retry: 0,
  })

  useEffect(() => {
    setFilterDraft(routeStateToDraft(normalizedRouteState))
  }, [normalizedRouteState])

  const submitFilters = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    onRouteStateChange?.({
      libraryId: cleanFilterValue(filterDraft.libraryId),
      state: selectValueToRouteValue(filterDraft.state),
      sourceKind: selectValueToRouteValue(filterDraft.sourceKind),
      managedImportArtifactId: cleanFilterValue(filterDraft.managedImportArtifactId),
      limit: parseLimit(filterDraft.limit),
      offset: DEFAULT_OFFSET,
    })
  }

  const clearFilters = () => {
    onRouteStateChange?.({
      limit: DEFAULT_LIMIT,
      offset: DEFAULT_OFFSET,
    })
  }

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

  const canPageBack = normalizedRouteState.offset > 0
  const canPageForward = data.page.returned >= normalizedRouteState.limit

  return (
    <div className="space-y-6">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div className="max-w-3xl">
          <div className="mb-2 flex items-center gap-2">
            <FileSearch className="h-5 w-5 text-primary" />
            <h1 className="text-xl font-semibold text-foreground">采集入口</h1>
          </div>
          <p className="text-sm text-muted-foreground">
            只读查看 Acquisition Intake 候选项，确认来源摘要、目标媒体库、诊断状态和 Managed Import 关联。
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

      <form
        className="rounded-lg border border-border/50 bg-card p-4"
        onSubmit={submitFilters}
      >
        <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_180px_220px_minmax(0,1fr)_140px_auto] lg:items-end">
          <div className="space-y-2">
            <Label htmlFor="acquisition-library-id">媒体库 ID</Label>
            <Input
              id="acquisition-library-id"
              value={filterDraft.libraryId}
              placeholder="library-movies"
              onChange={(event) => {
                setFilterDraft((draft) => ({ ...draft, libraryId: event.target.value }))
              }}
            />
          </div>
          <div className="space-y-2">
            <Label>状态</Label>
            <Select
              value={filterDraft.state}
              onValueChange={(value) => {
                setFilterDraft((draft) => ({ ...draft, state: value }))
              }}
            >
              <SelectTrigger aria-label="状态筛选">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="any">全部状态</SelectItem>
                {STATE_OPTIONS.map((state) => (
                  <SelectItem key={state} value={state}>
                    {stateLabel(state)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label>来源类型</Label>
            <Select
              value={filterDraft.sourceKind}
              onValueChange={(value) => {
                setFilterDraft((draft) => ({ ...draft, sourceKind: value }))
              }}
            >
              <SelectTrigger aria-label="来源类型筛选">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="any">全部来源</SelectItem>
                {SOURCE_KIND_OPTIONS.map((sourceKind) => (
                  <SelectItem key={sourceKind} value={sourceKind}>
                    {sourceKindLabel(sourceKind)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="managed-import-artifact-id">Managed Import Artifact</Label>
            <Input
              id="managed-import-artifact-id"
              value={filterDraft.managedImportArtifactId}
              placeholder="artifact-id"
              onChange={(event) => {
                setFilterDraft((draft) => ({ ...draft, managedImportArtifactId: event.target.value }))
              }}
            />
          </div>
          <div className="space-y-2">
            <Label>每页</Label>
            <Select
              value={filterDraft.limit}
              onValueChange={(value) => {
                setFilterDraft((draft) => ({ ...draft, limit: value }))
              }}
            >
              <SelectTrigger aria-label="每页数量">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="25">25</SelectItem>
                <SelectItem value="50">50</SelectItem>
                <SelectItem value="100">100</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex gap-2">
            <Button type="submit" size="sm">
              应用筛选
            </Button>
            <Button type="button" variant="ghost" size="sm" onClick={clearFilters}>
              清除
            </Button>
          </div>
        </div>
      </form>

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex flex-col gap-3 border-b border-border/50 p-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">候选项</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              返回 {data.page.returned} 项，偏移 {data.page.offset}，每页 {data.page.limit}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="outline">Admin API {data.versions.adminApi}</Badge>
            <Badge variant="outline">Public API {data.versions.publicApi}</Badge>
          </div>
        </div>

        {isLoading ? (
          <IntakeSkeleton />
        ) : data.candidates.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 p-10 text-center">
            <Database className="h-8 w-8 text-muted-foreground" />
            <p className="text-sm font-medium text-foreground">没有匹配的候选项</p>
            <p className="max-w-md text-sm text-muted-foreground">
              调整筛选条件，或等待后台采集流程发现新的候选项。
            </p>
          </div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>候选</TableHead>
                <TableHead>目标媒体库</TableHead>
                <TableHead>来源</TableHead>
                <TableHead>状态</TableHead>
                <TableHead>诊断</TableHead>
                <TableHead>Managed Import</TableHead>
                <TableHead>更新时间</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {data.candidates.map((candidate) => (
                <CandidateRow key={candidate.id} candidate={candidate} />
              ))}
            </TableBody>
          </Table>
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

function CandidateRow({ candidate }: { candidate: AdminAcquisitionIntakeCandidateReadModel }) {
  return (
    <TableRow>
      <TableCell>
        <div className="space-y-1">
          <div className="font-mono text-xs font-medium text-foreground">{candidate.id}</div>
          <div className="max-w-[22rem] truncate text-xs text-muted-foreground">{candidate.sourceSummary}</div>
          <div className="font-mono text-[11px] text-muted-foreground">{candidate.sourceKeyFingerprint}</div>
        </div>
      </TableCell>
      <TableCell className="font-mono text-xs">{candidate.targetLibraryId}</TableCell>
      <TableCell>
        <div className="space-y-1">
          <Badge variant="outline">{sourceKindLabel(candidate.sourceKind)}</Badge>
          <div className="text-xs text-muted-foreground">{candidate.sourceScheme ?? "scheme unknown"}</div>
        </div>
      </TableCell>
      <TableCell>
        <Badge variant={stateBadgeVariant(candidate.state)}>{stateLabel(candidate.state)}</Badge>
      </TableCell>
      <TableCell>
        <div className="flex flex-wrap gap-1">
          <ReadinessBadge ready={candidate.readiness.hasDiagnostics} label="诊断" />
          <ReadinessBadge ready={candidate.readiness.hasIntendedLocator} label="Locator" />
          <ReadinessBadge ready={candidate.readiness.hasFingerprint} label="指纹" />
        </div>
      </TableCell>
      <TableCell>
        <div className="space-y-1">
          <div className="font-mono text-xs text-foreground">{candidate.managedImportArtifactId ?? "未关联"}</div>
          <div className="text-xs text-muted-foreground">{formatBytes(candidate.sizeBytes)}</div>
        </div>
      </TableCell>
      <TableCell className="text-xs text-muted-foreground">{formatDateTime(candidate.updatedAt)}</TableCell>
    </TableRow>
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

function IntakeSkeleton() {
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

export function normalizeAcquisitionIntakeRouteState(
  routeState?: AdminAcquisitionIntakeRouteState,
): Required<AdminAcquisitionIntakeRouteState> {
  return normalizeRouteState(routeState)
}

function normalizeRouteState(routeState?: AdminAcquisitionIntakeRouteState): Required<AdminAcquisitionIntakeRouteState> {
  return {
    libraryId: routeState?.libraryId?.trim() ?? "",
    state: routeState?.state?.trim() ?? "",
    sourceKind: routeState?.sourceKind?.trim() ?? "",
    managedImportArtifactId: routeState?.managedImportArtifactId?.trim() ?? "",
    limit: routeState?.limit && routeState.limit > 0 ? routeState.limit : DEFAULT_LIMIT,
    offset: routeState?.offset && routeState.offset > 0 ? routeState.offset : DEFAULT_OFFSET,
  }
}

function routeStateToQuery(routeState: Required<AdminAcquisitionIntakeRouteState>): AdminAcquisitionIntakeCandidatesQuery {
  return {
    library_id: cleanFilterValue(routeState.libraryId),
    state: cleanFilterValue(routeState.state),
    source_kind: cleanFilterValue(routeState.sourceKind),
    managed_import_artifact_id: cleanFilterValue(routeState.managedImportArtifactId),
    limit: routeState.limit,
    offset: routeState.offset,
  }
}

function routeStateToDraft(routeState: Required<AdminAcquisitionIntakeRouteState>): FilterDraft {
  return {
    libraryId: routeState.libraryId,
    state: routeState.state || "any",
    sourceKind: routeState.sourceKind || "any",
    managedImportArtifactId: routeState.managedImportArtifactId,
    limit: String(routeState.limit),
  }
}

function cleanFilterValue(value: string | undefined) {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

function selectValueToRouteValue(value: string) {
  return value === "any" ? undefined : cleanFilterValue(value)
}

function parseLimit(value: string) {
  const parsed = Number.parseInt(value, 10)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : DEFAULT_LIMIT
}

function stateLabel(state: string) {
  switch (state) {
    case "discovered":
      return "已发现"
    case "inspecting":
      return "检查中"
    case "ready":
      return "就绪"
    case "blocked":
      return "阻塞"
    case "accepted":
      return "已接收"
    case "rejected":
      return "已拒绝"
    case "failed":
      return "失败"
    case "superseded":
      return "已取代"
    default:
      return state
  }
}

function sourceKindLabel(sourceKind: string) {
  switch (sourceKind) {
    case "watch_folder":
      return "Watch Folder"
    case "operator_submitted":
      return "Operator Submitted"
    case "external_download_output":
      return "External Download"
    case "addon_proposed":
      return "Addon Proposed"
    case "resource_search_selection":
      return "Resource Search"
    default:
      return sourceKind
  }
}

function stateBadgeVariant(state: string): "default" | "secondary" | "destructive" | "outline" {
  switch (state) {
    case "ready":
    case "accepted":
      return "default"
    case "blocked":
    case "failed":
      return "destructive"
    case "inspecting":
    case "discovered":
      return "secondary"
    default:
      return "outline"
  }
}

function formatBytes(value: number | null) {
  if (value === null) {
    return "大小未知"
  }

  const units = ["B", "KiB", "MiB", "GiB", "TiB"]
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
