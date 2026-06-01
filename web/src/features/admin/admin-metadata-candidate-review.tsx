"use client"

import { useEffect, useMemo, useState, type FormEvent } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  createAdminReadModelsDataSource,
  type AdminMetadataCandidateReviewNodeReadModel,
  type AdminMetadataCandidateReviewReadModel,
  type AdminMetadataCandidateReviewRelationshipReadModel,
} from "@/src/api/admin/read-models-data-source"
import {
  createAdminMutationDataSource,
  type AdminMetadataCandidateReviewApplyMutationResult,
} from "@/src/api/admin/mutations-data-source"

export interface AdminMetadataCandidateReviewRouteState {
  reviewId?: string
}

interface AdminMetadataCandidateReviewProps {
  routeState?: AdminMetadataCandidateReviewRouteState
  onRouteStateChange?: (state: AdminMetadataCandidateReviewRouteState) => void
}

const DEFAULT_REVIEW_STATE: Required<AdminMetadataCandidateReviewRouteState> = {
  reviewId: "",
}

export function AdminMetadataCandidateReview({
  routeState,
  onRouteStateChange,
}: AdminMetadataCandidateReviewProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeReviewRouteState(routeState), [routeState])
  const [draftReviewId, setDraftReviewId] = useState(normalizedRouteState.reviewId)
  const [armed, setArmed] = useState(false)
  const [idempotencyKey, setIdempotencyKey] = useState("")
  const queryClient = useQueryClient()
  const mutationDataSource = useMemo(() => createAdminMutationDataSource(), [])
  const reviewId = normalizedRouteState.reviewId

  useEffect(() => {
    setDraftReviewId(reviewId)
    setArmed(false)
    setIdempotencyKey(reviewId ? createCandidateReviewApplyIdempotencyKey(reviewId) : "")
  }, [reviewId])

  const {
    data,
    isLoading,
  } = useQuery({
    queryKey: metadataCandidateReviewQueryKey(reviewId),
    queryFn: () => createAdminReadModelsDataSource().loadMetadataCandidateReview(reviewId),
    enabled: reviewId.length > 0,
    staleTime: 10 * 1000,
    retry: 0,
  })

  const applyMutation = useMutation({
    mutationFn: () => {
      if (!data) {
        throw new Error("Metadata Candidate Review is not loaded")
      }

      return mutationDataSource.applyMetadataCandidateReview(reviewId, {
        itemId: data.itemId,
        expectedUpdatedAtMs: data.updatedAtMs,
        idempotencyKey,
      })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: metadataCandidateReviewQueryKey(reviewId) })
    },
  })

  const submitReviewId = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    onRouteStateChange?.({ reviewId: draftReviewId.trim() })
  }

  const mutationUnavailable = !mutationDataSource.canMutate
  const planFallback = Boolean(data?.fallback && mutationDataSource.canMutate)
  const planSkipped = data?.applicationPlan.action === "skip"
  const notAccepted = data?.status !== "accepted"
  const canSubmit =
    Boolean(data) &&
    !mutationUnavailable &&
    !planFallback &&
    !planSkipped &&
    !notAccepted &&
    armed &&
    !applyMutation.isPending &&
    idempotencyKey.length > 0

  return (
    <div className="space-y-6">
      <ReviewHeader
        source={data?.source ?? "fixture"}
      />

      <form
        className="flex flex-col gap-3 rounded-lg border border-border/50 bg-card p-4 sm:flex-row sm:items-end"
        onSubmit={submitReviewId}
      >
        <div className="min-w-0 flex-1">
          <label className="text-xs font-medium text-muted-foreground" htmlFor="candidate-review-id">
            Candidate Review ID
          </label>
          <Input
            id="candidate-review-id"
            aria-label="Candidate Review ID"
            className="mt-1 font-mono text-sm"
            value={draftReviewId}
            onChange={(event) => setDraftReviewId(event.target.value)}
            placeholder="review id"
          />
        </div>
        <Button type="submit" size="sm" disabled={!draftReviewId.trim()}>
          查看
        </Button>
      </form>

      {!reviewId && (
        <section className="rounded-lg border border-border/50 bg-card p-8 text-center">
          <h2 className="text-sm font-medium text-foreground">缺少 Candidate Review ID</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            输入 durable Metadata Candidate Review ID 后查看证据和 apply plan。
          </p>
        </section>
      )}

      {reviewId && (isLoading || !data) && <ReviewSkeleton />}

      {data && (
        <>
          {data.fallback && data.error && (
            <div className="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/5 p-4 text-sm">
              <div>
                <p className="font-medium text-foreground">Admin API 不可用，正在显示 fixture Candidate Review</p>
                <p className="mt-1 text-muted-foreground">{data.error}</p>
              </div>
            </div>
          )}

          <section className="rounded-lg border border-border/50 bg-card">
            <div className="flex flex-col gap-3 border-b border-border/50 p-4 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <h2 className="text-sm font-medium text-foreground">Review evidence</h2>
                <p className="mt-1 font-mono text-xs text-muted-foreground">{data.reviewId}</p>
              </div>
              <div className="flex flex-wrap gap-2">
                <Badge variant={data.status === "accepted" ? "default" : "secondary"}>{data.status}</Badge>
                <Badge variant="outline">{data.sourceLabel}</Badge>
                <Badge variant="outline">Admin API {data.versions.adminApi}</Badge>
              </div>
            </div>
            <div className="grid gap-4 p-4 lg:grid-cols-[1fr_1fr]">
              <RootCandidate node={data.root} itemId={data.itemId} sourceKey={data.sourceKey} />
              <PlanSummary review={data} />
            </div>
          </section>

          <section className="rounded-lg border border-border/50 bg-card">
            <div className="flex flex-col gap-3 border-b border-border/50 p-4 sm:flex-row sm:items-center sm:justify-between">
              <div>
                <h2 className="text-sm font-medium text-foreground">Related preview graph</h2>
                <p className="mt-1 text-xs text-muted-foreground">
                  Related nodes are evidence only; this apply path does not persist hierarchy.
                </p>
              </div>
              <div className="flex flex-wrap gap-2">
                <Badge variant="outline">related {data.relatedCount}</Badge>
                <Badge variant="outline">relationships {data.relationshipCount}</Badge>
              </div>
            </div>
            <RelatedPreviewTable related={data.related} relationships={data.relationships} />
          </section>

          <section className="rounded-lg border border-border/50 bg-card p-4">
            <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <h2 className="text-sm font-medium text-foreground">确认应用</h2>
                <p className="mt-1 text-sm text-muted-foreground">
                  仅应用 root Provider Subject / Provider Mapping。idempotency key 只发送到 Admin API，不在界面显示。
                </p>
                {mutationUnavailable && (
                  <p className="mt-2 text-sm text-warning">
                    {mutationDataSource.unavailableReason ?? "当前连接不能执行管理操作"}
                  </p>
                )}
                {planFallback && (
                  <p className="mt-2 text-sm text-warning">
                    Candidate Review 不是 live Admin API 返回，不能执行确认。
                  </p>
                )}
                {notAccepted && (
                  <p className="mt-2 text-sm text-warning">只有 accepted Candidate Review 可以应用。</p>
                )}
                {planSkipped && (
                  <p className="mt-2 text-sm text-warning">
                    {data.applicationPlan.reasons.join(", ") || "application plan is skipped"}
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
                    disabled={mutationUnavailable || planFallback || notAccepted || planSkipped || applyMutation.isPending}
                    onClick={() => setArmed(true)}
                  >
                    准备应用
                  </Button>
                ) : (
                  <>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      disabled={applyMutation.isPending}
                      onClick={() => setArmed(false)}
                    >
                      取消
                    </Button>
                    <Button
                      type="button"
                      size="sm"
                      className="gap-2"
                      disabled={!canSubmit}
                      onClick={() => applyMutation.mutate()}
                    >
                      {applyMutation.isPending ? "应用中" : "确认应用"}
                    </Button>
                  </>
                )}
              </div>
            </div>

            {applyMutation.error instanceof Error && (
              <div className="mt-4 rounded-lg border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive">
                {applyMutation.error.message}
              </div>
            )}

            {applyMutation.data && <ApplyResult result={applyMutation.data} />}
          </section>
        </>
      )}
    </div>
  )
}

export function normalizeMetadataCandidateReviewRouteState(
  routeState?: AdminMetadataCandidateReviewRouteState,
): Required<AdminMetadataCandidateReviewRouteState> {
  return normalizeReviewRouteState(routeState)
}

function normalizeReviewRouteState(
  routeState?: AdminMetadataCandidateReviewRouteState,
): Required<AdminMetadataCandidateReviewRouteState> {
  return {
    reviewId: routeState?.reviewId?.trim() || DEFAULT_REVIEW_STATE.reviewId,
  }
}

function metadataCandidateReviewQueryKey(reviewId: string) {
  return ["nako", "admin", "metadata-candidate-review", reviewId] as const
}

function ReviewHeader({
  source,
}: {
  source: AdminMetadataCandidateReviewReadModel["source"]
}) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
      <div className="max-w-3xl">
        <div className="mb-2 flex items-center gap-2">
          <h1 className="text-xl font-semibold text-foreground">Metadata Candidate Review</h1>
        </div>
        <p className="text-sm text-muted-foreground">
          Inspect durable Candidate Review evidence, root-only application plan, and confirmed apply result.
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant={source === "live" ? "default" : "secondary"}>
          {source === "live" ? "Live Admin API" : "Fixture"}
        </Badge>
      </div>
    </div>
  )
}

function RootCandidate({
  node,
  itemId,
  sourceKey,
}: {
  node: AdminMetadataCandidateReviewNodeReadModel
  itemId: string
  sourceKey: string
}) {
  return (
    <div className="space-y-4">
      <div className="grid gap-3 sm:grid-cols-2">
        <Fact label="Media Item" value={itemId} />
        <Fact label="Source key" value={sourceKey} />
        <Fact label="Kind" value={node.kind} />
        <Fact label="Source" value={node.sourceLabel} />
      </div>
      <div className="rounded-md border border-border/50 p-3">
        <div className="text-xs font-medium text-muted-foreground">Root subject</div>
        <div className="mt-2 grid gap-3 sm:grid-cols-2">
          <Fact label="Provider" value={node.subject?.provider ?? "unknown"} />
          <Fact label="Subject kind" value={node.subject?.subjectKind ?? "unknown"} />
          <Fact label="Subject key" value={node.subject?.subjectKey ?? "unknown"} />
          <Fact label="Title" value={node.subject?.title ?? "unknown"} />
        </div>
      </div>
    </div>
  )
}

function PlanSummary({ review }: { review: AdminMetadataCandidateReviewReadModel }) {
  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2">
        <Badge variant={review.applicationPlan.action === "apply" ? "default" : "secondary"}>
          {review.applicationPlan.action}
        </Badge>
        {review.applicationPlan.reasons.map((reason) => (
          <Badge key={reason} variant="outline">
            {reason}
          </Badge>
        ))}
      </div>
      <div className="grid gap-2 sm:grid-cols-2">
        <BoundaryFlag label="Root Provider Mapping" enabled={review.boundary.applyUpdatesRootProviderMapping} />
        <BoundaryFlag label="Root Provider Subject" enabled={review.boundary.applyUpdatesRootProviderSubject} />
        <BoundaryFlag label="Related preview only" enabled={!review.boundary.applyUpdatesRelatedProviderSubjects && !review.boundary.updatesHierarchy} />
        <BoundaryFlag label="Canonical Metadata" enabled={review.boundary.updatesCanonicalMetadata} />
      </div>
    </div>
  )
}

function RelatedPreviewTable({
  related,
  relationships,
}: {
  related: AdminMetadataCandidateReviewNodeReadModel[]
  relationships: AdminMetadataCandidateReviewRelationshipReadModel[]
}) {
  if (related.length === 0 && relationships.length === 0) {
    return <div className="p-4 text-sm text-muted-foreground">没有 related preview 节点。</div>
  }

  return (
    <div className="divide-y divide-border/50">
      {related.map((node) => {
        const relationship = relationships.find(
          (entry) => entry.childSubject.subjectKey === node.subject?.subjectKey,
        )

        return (
          <div
            key={`${node.kind}-${node.subject?.subjectKey ?? node.metadata.title ?? "node"}`}
            className="grid gap-3 p-4 text-sm lg:grid-cols-[8rem_minmax(0,1fr)_minmax(0,1fr)_10rem]"
          >
            <Badge className="w-fit" variant="outline">
              {node.kind}
            </Badge>
            <div className="min-w-0">
              <div className="truncate font-mono text-xs text-foreground">
                {node.subject?.subjectKey ?? "unknown"}
              </div>
              <div className="text-xs text-muted-foreground">{node.subject?.subjectKind ?? "unknown"}</div>
            </div>
            <div className="min-w-0 truncate">{node.metadata.title ?? node.subject?.title ?? "unknown"}</div>
            <div className="flex flex-wrap gap-1.5">
              <Badge variant="secondary">{relationship?.kind ?? "preview"}</Badge>
              <Badge variant="outline">not applied</Badge>
            </div>
          </div>
        )
      })}
    </div>
  )
}

function ApplyResult({ result }: { result: AdminMetadataCandidateReviewApplyMutationResult }) {
  return (
    <div className="mt-4 rounded-lg border border-success/30 bg-success/5 p-4">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm font-medium text-foreground">{result.message}</span>
        <Badge variant="outline">{result.applied ? "applied" : "not applied"}</Badge>
        <Badge variant={result.idempotentReplay ? "secondary" : "outline"}>
          {result.idempotentReplay ? "idempotent replay" : "new apply"}
        </Badge>
      </div>
      <div className="mt-3 grid gap-3 sm:grid-cols-4">
        <Fact label="Review" value={result.reviewId} />
        <Fact label="Item" value={result.itemId} />
        <Fact label="Changed" value={result.changed ? "true" : "false"} />
        <Fact label="Key fingerprint" value={result.idempotencyKeyFingerprint} />
        <Fact label="Mapping" value={result.providerMapping?.mappingId ?? "none"} />
        <Fact label="Mapping status" value={result.providerMapping?.status ?? "none"} />
        <Fact label="Subject" value={result.providerSubject?.subjectKey ?? "none"} />
        <Fact label="Related applied" value={result.boundary.applyUpdatesRelatedProviderSubjects ? "true" : "false"} />
      </div>
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
    <div className="rounded-lg border border-border/50 bg-card p-6 text-sm text-muted-foreground">
      正在加载 Candidate Review...
    </div>
  )
}

function createCandidateReviewApplyIdempotencyKey(reviewId: string) {
  const safeReviewId = reviewId.replace(/[^A-Za-z0-9._:-]+/g, "-").slice(0, 64) || "review"
  const nonce = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`
  return `web-metadata-candidate-review-apply:${safeReviewId}:${nonce}`
}
