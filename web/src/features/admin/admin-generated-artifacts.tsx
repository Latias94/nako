"use client"

import { useEffect, useMemo, useState } from "react"
import { useMutation, useQuery, useQueryClient, type UseMutationResult } from "@tanstack/react-query"
import {
  AlertTriangle,
  CheckCircle2,
  ChevronLeft,
  ChevronRight,
  Database,
  Eye,
  Loader2,
  RefreshCw,
  ShieldCheck,
  Sparkles,
  XCircle,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminGeneratedArtifactMetadataBulkApplyBatchReadModel,
  type AdminGeneratedArtifactMetadataBulkApplyPlanReadModel,
  type AdminGeneratedArtifactReviewDecision,
  type AdminGeneratedArtifactProposalReadModel,
} from "@/src/api/admin/read-models-data-source"
import {
  createAdminMutationDataSource,
  type AdminGeneratedArtifactMetadataBulkApplyMutationResult,
} from "@/src/api/admin/mutations-data-source"
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
  onReviewRequest?: (artifactId: string, decision: AdminGeneratedArtifactReviewDecision) => void
}

export function AdminGeneratedArtifacts({
  routeState,
  onRouteStateChange,
  onReviewRequest,
}: AdminGeneratedArtifactsProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeRouteState(routeState), [routeState])
  const query = useMemo(() => routeStateToQuery(normalizedRouteState), [normalizedRouteState])
  const readDataSource = useMemo(() => createAdminReadModelsDataSource(), [])
  const mutationDataSource = useMemo(() => createAdminMutationDataSource(), [])
  const queryClient = useQueryClient()
  const [selectedArtifactIds, setSelectedArtifactIds] = useState<string[]>([])
  const [bulkPlanArtifactIds, setBulkPlanArtifactIds] = useState<string[]>([])
  const [bulkApplyArmed, setBulkApplyArmed] = useState(false)
  const [bulkIdempotencyKey, setBulkIdempotencyKey] = useState("")
  const [bulkBatchId, setBulkBatchId] = useState<string | null>(null)
  const { data = ADMIN_GENERATED_ARTIFACTS_READ_MODEL_FIXTURE, isLoading, isFetching, refetch } = useQuery({
    queryKey: ["nako", "admin", "generated-artifacts", query],
    queryFn: () => readDataSource.loadGeneratedArtifacts(query),
    staleTime: 15 * 1000,
    retry: 0,
  })
  const visibleSelectableArtifactIds = useMemo(
    () => data.proposals.filter(isBulkMetadataApplySelectable).map((proposal) => proposal.id),
    [data.proposals],
  )
  const selectedVisibleCount = visibleSelectableArtifactIds.filter((artifactId) =>
    selectedArtifactIds.includes(artifactId),
  ).length
  const allVisibleSelected =
    visibleSelectableArtifactIds.length > 0 && selectedVisibleCount === visibleSelectableArtifactIds.length

  useEffect(() => {
    const visibleArtifactIdSet = new Set(data.proposals.map((proposal) => proposal.id))
    setSelectedArtifactIds((current) => current.filter((artifactId) => visibleArtifactIdSet.has(artifactId)))
  }, [data.proposals])

  useEffect(() => {
    setBulkApplyArmed(false)
    setBulkBatchId(null)
    setBulkIdempotencyKey(
      bulkPlanArtifactIds.length > 0
        ? createMetadataBulkApplyIdempotencyKey(bulkPlanArtifactIds)
        : "",
    )
  }, [bulkPlanArtifactIds])

  const bulkPlanQuery = useQuery({
    queryKey: ["nako", "admin", "generated-artifact-metadata-bulk-apply-plan", bulkPlanArtifactIds],
    queryFn: () => readDataSource.loadGeneratedArtifactMetadataBulkApplyPlan(bulkPlanArtifactIds),
    enabled: bulkPlanArtifactIds.length > 0,
    staleTime: 10 * 1000,
    retry: 0,
  })
  const bulkBatchQuery = useQuery({
    queryKey: ["nako", "admin", "generated-artifact-metadata-bulk-apply-batch", bulkBatchId],
    queryFn: () => readDataSource.loadGeneratedArtifactMetadataBulkApplyBatch(bulkBatchId ?? ""),
    enabled: Boolean(bulkBatchId),
    staleTime: 5 * 1000,
    retry: 0,
  })
  const bulkApplyMutation = useMutation({
    mutationFn: () =>
      mutationDataSource.confirmGeneratedArtifactMetadataBulkApplyBatch(
        bulkPlanArtifactIds,
        bulkIdempotencyKey,
      ),
    onSuccess: (batch) => {
      setBulkBatchId(batch.id)
      setBulkApplyArmed(false)
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "generated-artifacts"] })
    },
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
  const currentBulkBatch = bulkBatchQuery.data ?? bulkApplyMutation.data

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
            受控审核
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

      <BulkMetadataApplyPanel
        selectedArtifactIds={selectedArtifactIds}
        plan={bulkPlanQuery.data}
        planLoading={bulkPlanQuery.isLoading || bulkPlanQuery.isFetching}
        planError={bulkPlanQuery.error}
        batch={currentBulkBatch}
        batchFetching={bulkBatchQuery.isFetching}
        mutationDataSource={mutationDataSource}
        mutation={bulkApplyMutation}
        armed={bulkApplyArmed}
        idempotencyKey={bulkIdempotencyKey}
        onPlan={() => {
          setBulkPlanArtifactIds(selectedArtifactIds)
        }}
        onClearSelection={() => {
          setSelectedArtifactIds([])
          setBulkPlanArtifactIds([])
        }}
        onArm={() => setBulkApplyArmed(true)}
        onCancel={() => setBulkApplyArmed(false)}
        onConfirm={() => bulkApplyMutation.mutate()}
        onRefreshBatch={() => {
          void bulkBatchQuery.refetch()
        }}
      />

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
                  <TableHead className="w-10">
                    <input
                      type="checkbox"
                      aria-label="选择当前页可批量应用产物"
                      className="size-4 rounded border border-input bg-background text-primary"
                      checked={allVisibleSelected}
                      disabled={visibleSelectableArtifactIds.length === 0}
                      onChange={(event) => {
                        const checked = event.currentTarget.checked
                        setSelectedArtifactIds((current) => {
                          const visibleIdSet = new Set(visibleSelectableArtifactIds)
                          const remaining = current.filter((artifactId) => !visibleIdSet.has(artifactId))
                          return checked
                            ? [...remaining, ...visibleSelectableArtifactIds]
                            : remaining
                        })
                      }}
                    />
                  </TableHead>
                  <TableHead>产物</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>目标</TableHead>
                  <TableHead>Provider</TableHead>
                  <TableHead>Payload</TableHead>
                  <TableHead>就绪</TableHead>
                  <TableHead>指纹</TableHead>
                  <TableHead>更新时间</TableHead>
                  <TableHead className="sticky right-0 z-10 min-w-[9rem] bg-card text-right">审核</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {data.proposals.map((proposal) => (
                  <GeneratedArtifactRow
                    key={proposal.id}
                    proposal={proposal}
                    selected={selectedArtifactIds.includes(proposal.id)}
                    onBulkSelectionChange={(selected) => {
                      setSelectedArtifactIds((current) =>
                        selected
                          ? current.includes(proposal.id)
                            ? current
                            : [...current, proposal.id]
                          : current.filter((artifactId) => artifactId !== proposal.id),
                      )
                    }}
                    onReviewRequest={onReviewRequest}
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

function GeneratedArtifactRow({
  proposal,
  selected,
  onBulkSelectionChange,
  onReviewRequest,
}: {
  proposal: AdminGeneratedArtifactProposalReadModel
  selected: boolean
  onBulkSelectionChange: (selected: boolean) => void
  onReviewRequest?: (artifactId: string, decision: AdminGeneratedArtifactReviewDecision) => void
}) {
  const canReview = proposal.readiness.actionable && proposal.status === "pending_review"
  const canBulkApply = isBulkMetadataApplySelectable(proposal)

  return (
    <TableRow>
      <TableCell className="align-top">
        <input
          type="checkbox"
          aria-label={`选择批量应用 ${proposal.id}`}
          className="size-4 rounded border border-input bg-background text-primary"
          checked={selected}
          disabled={!canBulkApply}
          title={canBulkApply ? "加入批量 Metadata Authority apply" : "只有 accepted 产物可批量应用"}
          onChange={(event) => onBulkSelectionChange(event.currentTarget.checked)}
        />
      </TableCell>
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
      <TableCell className="sticky right-0 z-10 min-w-[9rem] bg-card">
        <div className="flex flex-wrap justify-end gap-1.5">
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-8 gap-1.5 px-2 text-xs"
            disabled={!canReview}
            title={canReview ? "查看接受计划" : "当前提案不可审核"}
            aria-label={`查看接受计划 ${proposal.id}`}
            onClick={() => onReviewRequest?.(proposal.id, "accept")}
          >
            <CheckCircle2 className="h-3.5 w-3.5" />
            接受
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="h-8 gap-1.5 px-2 text-xs"
            disabled={!canReview}
            title={canReview ? "查看拒绝计划" : "当前提案不可审核"}
            aria-label={`查看拒绝计划 ${proposal.id}`}
            onClick={() => onReviewRequest?.(proposal.id, "reject")}
          >
            <XCircle className="h-3.5 w-3.5" />
            拒绝
          </Button>
          {!onReviewRequest && (
            <Badge variant="outline" className="gap-1 text-xs">
              <Eye className="h-3 w-3" />
              队列
            </Badge>
          )}
        </div>
      </TableCell>
    </TableRow>
  )
}

function BulkMetadataApplyPanel({
  selectedArtifactIds,
  plan,
  planLoading,
  planError,
  batch,
  batchFetching,
  mutationDataSource,
  mutation,
  armed,
  idempotencyKey,
  onPlan,
  onClearSelection,
  onArm,
  onCancel,
  onConfirm,
  onRefreshBatch,
}: {
  selectedArtifactIds: string[]
  plan?: AdminGeneratedArtifactMetadataBulkApplyPlanReadModel
  planLoading: boolean
  planError: Error | null
  batch?: AdminGeneratedArtifactMetadataBulkApplyBatchReadModel | AdminGeneratedArtifactMetadataBulkApplyMutationResult
  batchFetching: boolean
  mutationDataSource: ReturnType<typeof createAdminMutationDataSource>
  mutation: UseMutationResult<AdminGeneratedArtifactMetadataBulkApplyMutationResult, Error, void, unknown>
  armed: boolean
  idempotencyKey: string
  onPlan: () => void
  onClearSelection: () => void
  onArm: () => void
  onCancel: () => void
  onConfirm: () => void
  onRefreshBatch: () => void
}) {
  const mutationUnavailable = !mutationDataSource.canMutate
  const planFallback = Boolean(plan?.fallback && mutationDataSource.canMutate)
  const planUnavailable = !plan || planFallback || plan.summary.executableArtifactCount === 0
  const canConfirm =
    !mutationUnavailable &&
    !planUnavailable &&
    armed &&
    !mutation.isPending &&
    idempotencyKey.length > 0

  return (
    <section className="rounded-lg border border-border/50 bg-card">
      <div className="flex flex-col gap-3 border-b border-border/50 p-4 lg:flex-row lg:items-start lg:justify-between">
        <div>
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-4 w-4 text-primary" />
            <h2 className="text-sm font-medium text-foreground">批量 Metadata Authority apply</h2>
          </div>
          <p className="mt-1 text-xs text-muted-foreground">
            只对 accepted Generated Artifact 建立计划，确认后通过 live Admin API 执行。
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant="outline">已选 {selectedArtifactIds.length}</Badge>
          <Button
            type="button"
            variant="outline"
            size="sm"
            className="gap-2"
            disabled={selectedArtifactIds.length === 0 || planLoading}
            onClick={onPlan}
          >
            {planLoading ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <CheckCircle2 className="h-3.5 w-3.5" />}
            生成批量计划
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={selectedArtifactIds.length === 0}
            onClick={onClearSelection}
          >
            清空
          </Button>
        </div>
      </div>

      <div className="space-y-4 p-4">
        {mutationUnavailable && (
          <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-sm text-warning">
            {mutationDataSource.unavailableReason ?? "当前连接不能执行管理操作"}
          </div>
        )}

        {selectedArtifactIds.length === 0 && (
          <div className="text-sm text-muted-foreground">
            从队列表格选择 accepted 产物后生成批量应用计划。
          </div>
        )}

        {planError instanceof Error && (
          <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
            {planError.message}
          </div>
        )}

        {planLoading && <BulkPlanSkeleton />}

        {plan && !planLoading && (
          <>
            {plan.fallback && plan.error && (
              <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-sm text-warning">
                Admin API 不可用，正在显示 fixture 批量计划: {plan.error}
              </div>
            )}
            {planFallback && (
              <div className="rounded-lg border border-warning/30 bg-warning/5 p-3 text-sm text-warning">
                批量应用计划不是 live Admin API 返回，不能执行确认。
              </div>
            )}

            <div className="grid gap-3 md:grid-cols-5">
              <Fact label="Selected" value={String(plan.selection.selectedArtifactCount)} />
              <Fact label="Executable" value={String(plan.summary.executableArtifactCount)} />
              <Fact label="Missing" value={String(plan.summary.missingArtifactCount)} />
              <Fact label="Apply fields" value={String(plan.summary.applyFieldCount)} />
              <Fact label="Skip fields" value={String(plan.summary.skippedFieldCount)} />
            </div>

            <div className="overflow-x-auto rounded-md border border-border/50">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>产物</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>字段动作</TableHead>
                    <TableHead>原因</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {plan.items.map((item) => (
                    <TableRow key={item.artifactId}>
                      <TableCell className="font-mono text-xs text-foreground">{item.artifactId}</TableCell>
                      <TableCell>
                        <Badge variant={item.executable ? "default" : "secondary"}>
                          {item.status}
                        </Badge>
                      </TableCell>
                      <TableCell className="text-xs text-muted-foreground">
                        apply {item.plan?.applyFieldCount ?? 0} · skip {item.plan?.skippedFieldCount ?? 0} · noop {item.plan?.noopFieldCount ?? 0}
                      </TableCell>
                      <TableCell>
                        <div className="flex max-w-[22rem] flex-wrap gap-1.5">
                          {item.reasons.length > 0 ? (
                            item.reasons.map((reason) => (
                              <Badge key={reason} variant="secondary">
                                {reason}
                              </Badge>
                            ))
                          ) : (
                            <span className="text-xs text-muted-foreground">none</span>
                          )}
                        </div>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>

            <div className="flex flex-col gap-3 rounded-lg border border-border/50 p-3 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <div className="text-sm font-medium text-foreground">确认批量应用</div>
                <div className="mt-1 text-xs text-muted-foreground">
                  Idempotency key 只发送到 Admin API，不在界面显示。
                </div>
              </div>
              <div className="flex flex-wrap gap-2">
                {!armed ? (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={mutationUnavailable || planUnavailable || mutation.isPending}
                    onClick={onArm}
                  >
                    准备确认批量应用
                  </Button>
                ) : (
                  <>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={mutation.isPending}
                      onClick={onCancel}
                    >
                      取消
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      className="gap-2"
                      disabled={!canConfirm}
                      onClick={onConfirm}
                    >
                      {mutation.isPending ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <CheckCircle2 className="h-3.5 w-3.5" />}
                      {mutation.isPending ? "正在提交" : "确认批量应用"}
                    </Button>
                  </>
                )}
              </div>
            </div>
          </>
        )}

        {mutation.error instanceof Error && (
          <div className="rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
            {mutation.error.message}
          </div>
        )}

        {batch && (
          <BulkApplyBatchResult
            batch={batch}
            fetching={batchFetching}
            onRefresh={onRefreshBatch}
          />
        )}
      </div>
    </section>
  )
}

function BulkApplyBatchResult({
  batch,
  fetching,
  onRefresh,
}: {
  batch: AdminGeneratedArtifactMetadataBulkApplyBatchReadModel | AdminGeneratedArtifactMetadataBulkApplyMutationResult
  fetching: boolean
  onRefresh: () => void
}) {
  const message =
    "message" in batch ? batch.message : generatedArtifactMetadataBulkApplyBatchMessage(batch.status)

  return (
    <div className="rounded-lg border border-success/30 bg-success/5 p-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm font-medium text-foreground">{message}</span>
            <Badge variant="outline">{batch.status}</Badge>
          </div>
          <div className="mt-2 grid gap-3 sm:grid-cols-4">
            <Fact label="Batch" value={batch.id} />
            <Fact label="Job" value={batch.jobId} />
            <Fact label="Applied" value={String(batch.executionSummary.appliedItemCount)} />
            <Fact label="Skipped" value={String(batch.executionSummary.skippedItemCount)} />
          </div>
        </div>
        <Button type="button" variant="outline" size="sm" className="gap-2" onClick={onRefresh}>
          {fetching ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
          刷新批次状态
        </Button>
      </div>

      <div className="mt-4 overflow-x-auto rounded-md border border-success/20">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>产物</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>Outcome</TableHead>
              <TableHead>错误</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {batch.items.map((item) => (
              <TableRow key={`${item.position}-${item.artifactId}`}>
                <TableCell className="font-mono text-xs text-foreground">{item.artifactId}</TableCell>
                <TableCell>
                  <Badge variant={item.status === "applied" ? "default" : "secondary"}>
                    {item.status}
                  </Badge>
                </TableCell>
                <TableCell className="font-mono text-xs text-muted-foreground">
                  {item.outcomeId ?? "none"}
                </TableCell>
                <TableCell className="text-xs text-muted-foreground">
                  {item.errorCode ?? "none"}
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
    </div>
  )
}

function BulkPlanSkeleton() {
  return (
    <div className="space-y-3">
      <div className="grid gap-3 md:grid-cols-5">
        {Array.from({ length: 5 }, (_, index) => (
          <Skeleton key={index} className="h-12" />
        ))}
      </div>
      <Skeleton className="h-36" />
    </div>
  )
}

function Fingerprint({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="truncate font-mono text-muted-foreground">
      {label}: {value ?? "none"}
    </div>
  )
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="min-w-0">
      <div className="text-xs font-medium text-muted-foreground">{label}</div>
      <div className="mt-1 truncate font-mono text-xs text-foreground" title={value}>
        {value}
      </div>
    </div>
  )
}

function isBulkMetadataApplySelectable(proposal: AdminGeneratedArtifactProposalReadModel) {
  return proposal.status === "accepted"
}

function createMetadataBulkApplyIdempotencyKey(artifactIds: string[]) {
  const safeSeed =
    artifactIds
      .slice(0, 2)
      .join("-")
      .replace(/[^A-Za-z0-9._:-]+/g, "-")
      .slice(0, 80) || "artifacts"
  const nonce = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`
  return `web-generated-artifact-metadata-bulk-apply:${artifactIds.length}:${safeSeed}:${nonce}`
}

function generatedArtifactMetadataBulkApplyBatchMessage(status: string) {
  switch (status) {
    case "completed":
      return "批量元数据应用已完成"
    case "failed":
      return "批量元数据应用失败"
    case "cancelled":
      return "批量元数据应用已取消"
    default:
      return "批量元数据应用批次已提交"
  }
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
