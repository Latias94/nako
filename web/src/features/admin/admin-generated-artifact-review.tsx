"use client"

import { useEffect, useMemo, useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import {
  AlertTriangle,
  ArrowLeft,
  CheckCircle2,
  FileJson,
  Loader2,
  RefreshCw,
  ShieldAlert,
  ShieldCheck,
  XCircle,
} from "lucide-react"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Skeleton } from "@/components/ui/skeleton"
import {
  ADMIN_GENERATED_ARTIFACT_REVIEW_PLAN_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminGeneratedArtifactAcceptanceBoundaryReadModel,
  type AdminGeneratedArtifactReviewDecision,
  type AdminGeneratedArtifactReviewPlanReadModel,
} from "@/src/api/admin/read-models-data-source"
import {
  createAdminMutationDataSource,
  type AdminGeneratedArtifactReviewMutationResult,
} from "@/src/api/admin/mutations-data-source"

export interface AdminGeneratedArtifactReviewRouteState {
  artifactId?: string
  decision?: AdminGeneratedArtifactReviewDecision
}

interface AdminGeneratedArtifactReviewProps {
  routeState?: AdminGeneratedArtifactReviewRouteState
  onRouteStateChange?: (state: AdminGeneratedArtifactReviewRouteState) => void
  onBackToQueue?: () => void
  onMetadataApplyRequest?: (artifactId: string) => void
}

const DEFAULT_REVIEW_STATE: Required<AdminGeneratedArtifactReviewRouteState> = {
  artifactId: "",
  decision: "accept",
}

export function AdminGeneratedArtifactReview({
  routeState,
  onRouteStateChange,
  onBackToQueue,
  onMetadataApplyRequest,
}: AdminGeneratedArtifactReviewProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeReviewRouteState(routeState), [routeState])
  const [armed, setArmed] = useState(false)
  const queryClient = useQueryClient()
  const mutationDataSource = useMemo(() => createAdminMutationDataSource(), [])
  const artifactId = normalizedRouteState.artifactId
  const decision = normalizedRouteState.decision

  useEffect(() => {
    setArmed(false)
  }, [artifactId, decision])

  const {
    data = artifactId
      ? ADMIN_GENERATED_ARTIFACT_REVIEW_PLAN_FIXTURE
      : undefined,
    isLoading,
    isFetching,
    refetch,
  } = useQuery({
    queryKey: generatedArtifactReviewPlanQueryKey(artifactId, decision),
    queryFn: () => createAdminReadModelsDataSource().loadGeneratedArtifactReviewPlan(artifactId, decision),
    enabled: artifactId.length > 0,
    staleTime: 10 * 1000,
    retry: 0,
  })

  const reviewMutation = useMutation({
    mutationFn: () => mutationDataSource.reviewGeneratedArtifact(artifactId, decision),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "generated-artifacts"] })
      void queryClient.invalidateQueries({
        queryKey: generatedArtifactReviewPlanQueryKey(artifactId, decision),
      })
    },
  })

  const changeDecision = (nextDecision: AdminGeneratedArtifactReviewDecision) => {
    onRouteStateChange?.({
      artifactId,
      decision: nextDecision,
    })
  }

  if (!artifactId) {
    return (
      <div className="space-y-6">
        <ReviewHeader
          decision={decision}
          source="fixture"
          isFetching={false}
          onBackToQueue={onBackToQueue}
          onRefresh={undefined}
        />
        <section className="rounded-lg border border-border/50 bg-card p-8 text-center">
          <FileJson className="mx-auto h-9 w-9 text-muted-foreground" />
          <h2 className="mt-3 text-sm font-medium text-foreground">缺少生成产物 ID</h2>
          <p className="mt-1 text-sm text-muted-foreground">返回队列选择一个待审核提案。</p>
          <Button type="button" variant="outline" size="sm" className="mt-4 gap-2" onClick={onBackToQueue}>
            <ArrowLeft className="h-3.5 w-3.5" />
            返回队列
          </Button>
        </section>
      </div>
    )
  }

  if (isLoading || !data) {
    return (
      <div className="space-y-6">
        <ReviewHeader
          decision={decision}
          source="fixture"
          isFetching={isFetching}
          onBackToQueue={onBackToQueue}
          onRefresh={() => {
            void refetch()
          }}
        />
        <ReviewSkeleton />
      </div>
    )
  }

  const mutationUnavailable = !mutationDataSource.canMutate
  const planFallback = data.fallback && mutationDataSource.canMutate
  const readinessUnavailable = !data.readiness.actionable
  const planUnavailable = planFallback || readinessUnavailable
  const canSubmit = !mutationUnavailable && !planUnavailable && armed && !reviewMutation.isPending
  const result = reviewMutation.data

  return (
    <div className="space-y-6">
      <ReviewHeader
        decision={decision}
        source={data.source}
        isFetching={isFetching}
        onBackToQueue={onBackToQueue}
        onRefresh={() => {
          void refetch()
        }}
      />

      {data.fallback && data.error && (
        <div className="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/5 p-4 text-sm">
          <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0 text-warning" />
          <div>
            <p className="font-medium text-foreground">Admin API 不可用，正在显示 fixture 审核计划</p>
            <p className="mt-1 text-muted-foreground">{data.error}</p>
          </div>
        </div>
      )}

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex flex-col gap-3 border-b border-border/50 p-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">审核计划</h2>
            <p className="mt-1 font-mono text-xs text-muted-foreground">{data.artifactId}</p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <DecisionButton
              decision="accept"
              active={decision === "accept"}
              onClick={() => changeDecision("accept")}
            />
            <DecisionButton
              decision="reject"
              active={decision === "reject"}
              onClick={() => changeDecision("reject")}
            />
          </div>
        </div>

        <div className="grid gap-4 p-4 xl:grid-cols-[1.15fr_0.85fr]">
          <PlanSummary plan={data} />
          <BoundarySummary boundary={data.boundary} />
        </div>
      </section>

      <section className="grid gap-4 xl:grid-cols-2">
        <TargetSummary plan={data} />
        <PayloadSummary plan={data} />
      </section>

      <section className="rounded-lg border border-border/50 bg-card p-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">确认审核</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              {decision === "accept" ? "接受" : "拒绝"} {data.artifactId}
            </p>
            {mutationUnavailable && (
              <p className="mt-2 text-sm text-warning">
                {mutationDataSource.unavailableReason ?? "当前连接不能执行管理操作"}
              </p>
            )}
            {planFallback && (
              <p className="mt-2 text-sm text-warning">
                审核计划不是 live Admin API 返回，不能执行确认。
              </p>
            )}
            {readinessUnavailable && (
              <p className="mt-2 text-sm text-warning">
                {data.readiness.reasons.length > 0 ? data.readiness.reasons.join(", ") : "审核计划不可执行"}
              </p>
            )}
          </div>

          <div className="flex flex-wrap gap-2">
            {!armed ? (
              <Button
                type="button"
                variant="outline"
                size="sm"
                className="gap-2"
                disabled={mutationUnavailable || planUnavailable || reviewMutation.isPending}
                onClick={() => setArmed(true)}
              >
                <ShieldCheck className="h-3.5 w-3.5" />
                准备确认
              </Button>
            ) : (
              <>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  disabled={reviewMutation.isPending}
                  onClick={() => setArmed(false)}
                >
                  取消
                </Button>
                <Button
                  type="button"
                  variant={decision === "reject" ? "destructive" : "default"}
                  size="sm"
                  className="gap-2"
                  disabled={!canSubmit}
                  onClick={() => reviewMutation.mutate()}
                >
                  {reviewMutation.isPending ? (
                    <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  ) : decision === "accept" ? (
                    <CheckCircle2 className="h-3.5 w-3.5" />
                  ) : (
                    <XCircle className="h-3.5 w-3.5" />
                  )}
                  确认{decision === "accept" ? "接受" : "拒绝"}
                </Button>
              </>
            )}
          </div>
        </div>

        {reviewMutation.error instanceof Error && (
          <div className="mt-4 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
            {reviewMutation.error.message}
          </div>
        )}

        {result && <ReviewResult result={result} onMetadataApplyRequest={onMetadataApplyRequest} />}
      </section>
    </div>
  )
}

export function normalizeGeneratedArtifactReviewRouteState(
  routeState?: AdminGeneratedArtifactReviewRouteState,
): Required<AdminGeneratedArtifactReviewRouteState> {
  return normalizeReviewRouteState(routeState)
}

function normalizeReviewRouteState(
  routeState?: AdminGeneratedArtifactReviewRouteState,
): Required<AdminGeneratedArtifactReviewRouteState> {
  return {
    artifactId: routeState?.artifactId?.trim() || DEFAULT_REVIEW_STATE.artifactId,
    decision: routeState?.decision === "reject" ? "reject" : DEFAULT_REVIEW_STATE.decision,
  }
}

function generatedArtifactReviewPlanQueryKey(
  artifactId: string,
  decision: AdminGeneratedArtifactReviewDecision,
) {
  return ["nako", "admin", "generated-artifact-review-plan", artifactId, decision] as const
}

function ReviewHeader({
  decision,
  source,
  isFetching,
  onBackToQueue,
  onRefresh,
}: {
  decision: AdminGeneratedArtifactReviewDecision
  source: AdminGeneratedArtifactReviewPlanReadModel["source"]
  isFetching: boolean
  onBackToQueue?: () => void
  onRefresh?: () => void
}) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
      <div className="max-w-3xl">
        <div className="mb-2 flex items-center gap-2">
          <ShieldAlert className="h-5 w-5 text-primary" />
          <h1 className="text-xl font-semibold text-foreground">生成产物审核</h1>
        </div>
        <p className="text-sm text-muted-foreground">
          查看 review-plan 边界后确认 {decision === "accept" ? "接受" : "拒绝"}。
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant={source === "live" ? "default" : "secondary"}>
          {source === "live" ? "Live Admin API" : "Fixture"}
        </Badge>
        <Button type="button" variant="outline" size="sm" className="gap-2" onClick={onBackToQueue}>
          <ArrowLeft className="h-3.5 w-3.5" />
          返回队列
        </Button>
        {onRefresh && (
          <Button type="button" variant="outline" size="sm" className="gap-2" onClick={onRefresh}>
            {isFetching ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <RefreshCw className="h-3.5 w-3.5" />}
            刷新
          </Button>
        )}
      </div>
    </div>
  )
}

function DecisionButton({
  decision,
  active,
  onClick,
}: {
  decision: AdminGeneratedArtifactReviewDecision
  active: boolean
  onClick: () => void
}) {
  const accepting = decision === "accept"
  const Icon = accepting ? CheckCircle2 : XCircle

  return (
    <Button
      type="button"
      variant={active ? (accepting ? "default" : "destructive") : "outline"}
      size="sm"
      className="gap-2"
      aria-pressed={active}
      onClick={onClick}
    >
      <Icon className="h-3.5 w-3.5" />
      {accepting ? "接受" : "拒绝"}
    </Button>
  )
}

function PlanSummary({ plan }: { plan: AdminGeneratedArtifactReviewPlanReadModel }) {
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap gap-2">
        <Badge variant={plan.decision === "accept" ? "default" : "destructive"}>
          {plan.decision === "accept" ? "接受" : "拒绝"}
        </Badge>
        <Badge variant="outline">{plan.status}</Badge>
        <Badge variant="outline">{plan.action}</Badge>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <Fact label="Capability" value={plan.capability} />
        <Fact label="Kind" value={plan.kind} />
        <Fact label="Readiness" value={plan.readiness.status} />
        <Fact label="Actionable" value={plan.readiness.actionable ? "true" : "false"} />
      </div>
      <div>
        <div className="text-xs font-medium text-muted-foreground">Reasons</div>
        <div className="mt-1 flex flex-wrap gap-1.5">
          {plan.reasons.length > 0 ? (
            plan.reasons.map((reason) => (
              <Badge key={reason} variant="secondary">
                {reason}
              </Badge>
            ))
          ) : (
            <span className="text-sm text-muted-foreground">none</span>
          )}
        </div>
      </div>
    </div>
  )
}

function BoundarySummary({ boundary }: { boundary: AdminGeneratedArtifactAcceptanceBoundaryReadModel }) {
  return (
    <div className="grid gap-2 sm:grid-cols-2">
      <BoundaryFlag label="Canonical metadata" enabled={boundary.acceptedIntoCanonicalMetadata} />
      <BoundaryFlag label="Metadata Authority apply" enabled={boundary.requiresMetadataAuthorityApply} />
      <BoundaryFlag label="Sidecar writes" enabled={boundary.writesSidecar} />
      <BoundaryFlag label="Library file writes" enabled={boundary.writesLibraryFiles} />
      <BoundaryFlag label="Immediate apply" enabled={boundary.appliesImmediately} />
    </div>
  )
}

function BoundaryFlag({ label, enabled }: { label: string; enabled: boolean }) {
  return (
    <div className="flex items-center justify-between gap-3 rounded-md border border-border/50 px-3 py-2 text-sm">
      <span className="text-muted-foreground">{label}</span>
      <Badge variant={enabled ? "default" : "outline"}>{enabled ? "yes" : "no"}</Badge>
    </div>
  )
}

function TargetSummary({ plan }: { plan: AdminGeneratedArtifactReviewPlanReadModel }) {
  return (
    <section className="rounded-lg border border-border/50 bg-card p-4">
      <h2 className="text-sm font-medium text-foreground">目标</h2>
      <div className="mt-3 grid gap-3 sm:grid-cols-2">
        <Fact label="Kind" value={plan.target.kind} />
        <Fact label="Library" value={plan.target.libraryId ?? "unknown"} />
        <Fact label="Item" value={plan.target.itemId ?? "unknown"} />
        <Fact label="Source" value={plan.target.sourceId ?? "unknown"} />
      </div>
    </section>
  )
}

function PayloadSummary({ plan }: { plan: AdminGeneratedArtifactReviewPlanReadModel }) {
  return (
    <section className="rounded-lg border border-border/50 bg-card p-4">
      <h2 className="text-sm font-medium text-foreground">Payload 摘要</h2>
      <div className="mt-3 grid gap-3 sm:grid-cols-2">
        <Fact label="Shape" value={plan.payload.shape} />
        <Fact label="Bytes" value={formatBytes(plan.payload.payloadBytes)} />
        <Fact label="Fingerprint" value={plan.payload.payloadFingerprint} />
        <Fact label="Confidence" value={confidenceLabel(plan.payload.confidenceMilli)} />
        <Fact label="Fields" value={String(plan.payload.objectFieldCount ?? 0)} />
        <Fact label="Items" value={String(plan.payload.arrayItemCount ?? 0)} />
        <Fact label="Textual values" value={plan.payload.hasTextualValues ? "true" : "false"} />
        <Fact label="Explanation" value={plan.payload.hasExplanation ? "true" : "false"} />
      </div>
    </section>
  )
}

function ReviewResult({
  result,
  onMetadataApplyRequest,
}: {
  result: AdminGeneratedArtifactReviewMutationResult
  onMetadataApplyRequest?: (artifactId: string) => void
}) {
  const canApplyMetadata =
    result.decision === "accept" && result.plan.boundary.requiresMetadataAuthorityApply

  return (
    <div className="mt-4 rounded-lg border border-success/30 bg-success/5 p-4">
      <div className="flex flex-wrap items-center gap-2">
        <CheckCircle2 className="h-4 w-4 text-success" />
        <span className="text-sm font-medium text-foreground">{result.message}</span>
        <Badge variant="outline">{result.artifactStatus}</Badge>
        <Badge variant={result.idempotentReplay ? "secondary" : "outline"}>
          {result.idempotentReplay ? "idempotent replay" : "new review"}
        </Badge>
      </div>
      <div className="mt-3 grid gap-3 sm:grid-cols-3">
        <Fact label="Artifact" value={result.artifactId} />
        <Fact label="Decision" value={result.decision} />
        <Fact label="Accepted at" value={result.acceptedAt ? formatDateTime(result.acceptedAt) : "none"} />
      </div>
      {canApplyMetadata && onMetadataApplyRequest && (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="mt-4 gap-2"
          onClick={() => onMetadataApplyRequest(result.artifactId)}
        >
          <ShieldCheck className="h-3.5 w-3.5" />
          进入 Metadata Authority apply
        </Button>
      )}
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

function ReviewSkeleton() {
  return (
    <div className="space-y-4">
      <Skeleton className="h-40" />
      <div className="grid gap-4 xl:grid-cols-2">
        <Skeleton className="h-40" />
        <Skeleton className="h-40" />
      </div>
    </div>
  )
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
