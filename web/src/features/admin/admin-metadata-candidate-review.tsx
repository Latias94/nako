"use client"

import { useEffect, useMemo, useState, type FormEvent } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  createAdminReadModelsDataSource,
  type AdminMetadataCandidateReviewListItemReadModel,
  type AdminMetadataCandidateReviewListReadModel,
  type AdminMetadataCandidateReviewNodeReadModel,
  type AdminMetadataCandidateReviewQueueReadModel,
  type AdminMetadataCandidateReviewReadModel,
  type AdminMetadataCandidateReviewRelationshipReadModel,
} from "@/src/api/admin/read-models-data-source"
import {
  createAdminMutationDataSource,
  type AdminMetadataCandidateReviewApplyMutationResult,
} from "@/src/api/admin/mutations-data-source"

export interface AdminMetadataCandidateReviewRouteState {
  mode?: AdminMetadataCandidateReviewMode
  itemId?: string
  reviewId?: string
  status?: AdminMetadataCandidateReviewStatusFilter
  provider?: AdminMetadataCandidateReviewProviderFilter
  limit?: number
  offset?: number
}

interface AdminMetadataCandidateReviewProps {
  routeState?: AdminMetadataCandidateReviewRouteState
  onRouteStateChange?: (state: AdminMetadataCandidateReviewRouteState) => void
}

const DEFAULT_LIMIT = 50
const DEFAULT_OFFSET = 0
const DEFAULT_STATUS: AdminMetadataCandidateReviewStatusFilter = "accepted"
const DEFAULT_PROVIDER: AdminMetadataCandidateReviewProviderFilter = "all"

export type AdminMetadataCandidateReviewMode = "queue" | "item"
export type AdminMetadataCandidateReviewStatusFilter =
  | "all"
  | "pending"
  | "accepted"
  | "rejected"
  | "superseded"
  | "expired"
export type AdminMetadataCandidateReviewProviderFilter =
  | "all"
  | "tmdb"
  | "douban"
  | "bangumi"
  | "imdb"
  | "local"

const STATUS_FILTERS: Array<{ value: AdminMetadataCandidateReviewStatusFilter; label: string }> = [
  { value: "accepted", label: "accepted" },
  { value: "pending", label: "pending" },
  { value: "rejected", label: "rejected" },
  { value: "superseded", label: "superseded" },
  { value: "expired", label: "expired" },
  { value: "all", label: "全部状态" },
]

const PROVIDER_FILTERS: Array<{ value: AdminMetadataCandidateReviewProviderFilter; label: string }> = [
  { value: "all", label: "全部 Provider" },
  { value: "bangumi", label: "bangumi" },
  { value: "tmdb", label: "tmdb" },
  { value: "douban", label: "douban" },
  { value: "imdb", label: "imdb" },
  { value: "local", label: "local" },
]

const DEFAULT_REVIEW_STATE: Required<AdminMetadataCandidateReviewRouteState> = {
  mode: "queue",
  itemId: "",
  reviewId: "",
  status: DEFAULT_STATUS,
  provider: DEFAULT_PROVIDER,
  limit: DEFAULT_LIMIT,
  offset: DEFAULT_OFFSET,
}

export function AdminMetadataCandidateReview({
  routeState,
  onRouteStateChange,
}: AdminMetadataCandidateReviewProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeReviewRouteState(routeState), [routeState])
  const [draftItemId, setDraftItemId] = useState(normalizedRouteState.itemId)
  const [armed, setArmed] = useState(false)
  const [idempotencyKey, setIdempotencyKey] = useState("")
  const queryClient = useQueryClient()
  const readDataSource = useMemo(() => createAdminReadModelsDataSource(), [])
  const mutationDataSource = useMemo(() => createAdminMutationDataSource(), [])
  const mode = normalizedRouteState.mode
  const itemId = normalizedRouteState.itemId
  const reviewId = normalizedRouteState.reviewId
  const shouldLoadQueue = mode === "queue" && (routeState?.mode === "queue" || reviewId.length === 0)
  const listQuery = useMemo(
    () => ({
      limit: normalizedRouteState.limit,
      offset: normalizedRouteState.offset,
    }),
    [normalizedRouteState.limit, normalizedRouteState.offset],
  )
  const queueQuery = useMemo(
    () => ({
      status: normalizedRouteState.status === "all" ? undefined : normalizedRouteState.status,
      provider: normalizedRouteState.provider === "all" ? undefined : normalizedRouteState.provider,
      limit: normalizedRouteState.limit,
      offset: normalizedRouteState.offset,
    }),
    [
      normalizedRouteState.limit,
      normalizedRouteState.offset,
      normalizedRouteState.provider,
      normalizedRouteState.status,
    ],
  )

  useEffect(() => {
    setDraftItemId(itemId)
    setArmed(false)
    setIdempotencyKey(reviewId ? createCandidateReviewApplyIdempotencyKey(reviewId) : "")
  }, [itemId, reviewId])

  const {
    data: queueData,
    isLoading: isQueueLoading,
  } = useQuery({
    queryKey: metadataCandidateReviewQueueQueryKey(
      normalizedRouteState.status,
      normalizedRouteState.provider,
      queueQuery.limit,
      queueQuery.offset,
    ),
    queryFn: () => readDataSource.loadMetadataCandidateReviews(queueQuery),
    enabled: shouldLoadQueue,
    staleTime: 10 * 1000,
    retry: 0,
  })

  const {
    data: itemListData,
    isLoading: isItemListLoading,
  } = useQuery({
    queryKey: metadataCandidateReviewListQueryKey(itemId, listQuery.limit, listQuery.offset),
    queryFn: () => readDataSource.loadMetadataCandidateReviewsForItem(itemId, listQuery),
    enabled: mode === "item" && itemId.length > 0,
    staleTime: 10 * 1000,
    retry: 0,
  })

  const {
    data,
    isLoading,
  } = useQuery({
    queryKey: metadataCandidateReviewQueryKey(reviewId),
    queryFn: () => readDataSource.loadMetadataCandidateReview(reviewId),
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
      if (mode === "queue") {
        void queryClient.invalidateQueries({
          queryKey: metadataCandidateReviewQueueQueryKey(
            normalizedRouteState.status,
            normalizedRouteState.provider,
            queueQuery.limit,
            queueQuery.offset,
          ),
        })
      }
      if (mode === "item" && itemId) {
        void queryClient.invalidateQueries({
          queryKey: metadataCandidateReviewListQueryKey(itemId, listQuery.limit, listQuery.offset),
        })
      }
    },
  })

  const submitItemId = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault()
    const nextItemId = draftItemId.trim()
    onRouteStateChange?.({
      mode: "item",
      itemId: nextItemId,
      reviewId: "",
      status: normalizedRouteState.status,
      provider: normalizedRouteState.provider,
      limit: DEFAULT_LIMIT,
      offset: DEFAULT_OFFSET,
    })
  }

  const selectReview = (nextReviewId: string) => {
    onRouteStateChange?.({
      ...normalizedRouteState,
      reviewId: nextReviewId,
    })
  }

  const selectMode = (nextMode: AdminMetadataCandidateReviewMode) => {
    onRouteStateChange?.({
      ...normalizedRouteState,
      mode: nextMode,
      itemId: nextMode === "item" ? normalizedRouteState.itemId : "",
      reviewId: "",
      offset: DEFAULT_OFFSET,
    })
  }

  const selectStatus = (status: AdminMetadataCandidateReviewStatusFilter) => {
    onRouteStateChange?.({
      ...normalizedRouteState,
      mode: "queue",
      status,
      reviewId: "",
      offset: DEFAULT_OFFSET,
    })
  }

  const selectProvider = (provider: AdminMetadataCandidateReviewProviderFilter) => {
    onRouteStateChange?.({
      ...normalizedRouteState,
      mode: "queue",
      provider,
      reviewId: "",
      offset: DEFAULT_OFFSET,
    })
  }

  const moveListPage = (direction: "previous" | "next") => {
    const nextOffset =
      direction === "previous"
        ? Math.max(DEFAULT_OFFSET, normalizedRouteState.offset - normalizedRouteState.limit)
        : normalizedRouteState.offset + normalizedRouteState.limit

    onRouteStateChange?.({
      ...normalizedRouteState,
      offset: nextOffset,
    })
  }

  const listData = mode === "queue" ? queueData : itemListData
  const isListLoading = mode === "queue" ? isQueueLoading : isItemListLoading
  const showList = mode === "queue" ? shouldLoadQueue : itemId.length > 0

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
        source={data?.source ?? listData?.source ?? "fixture"}
      />

      <section className="rounded-lg border border-border/50 bg-card p-4">
        <div className="flex flex-col gap-4 xl:flex-row xl:items-end xl:justify-between">
          <div className="flex flex-wrap gap-2">
            <Button
              type="button"
              variant={mode === "queue" ? "secondary" : "outline"}
              size="sm"
              onClick={() => selectMode("queue")}
            >
              全局队列
            </Button>
            <Button
              type="button"
              variant={mode === "item" ? "secondary" : "outline"}
              size="sm"
              onClick={() => selectMode("item")}
            >
              按 Item
            </Button>
          </div>

          {mode === "queue" ? (
            <div className="grid min-w-0 flex-1 gap-3 sm:grid-cols-[minmax(10rem,12rem)_minmax(10rem,12rem)] xl:flex-none">
              <div>
                <label className="text-xs font-medium text-muted-foreground" htmlFor="candidate-review-status">
                  状态
                </label>
                <Select
                  value={normalizedRouteState.status}
                  onValueChange={(value) =>
                    selectStatus(value as AdminMetadataCandidateReviewStatusFilter)
                  }
                >
                  <SelectTrigger id="candidate-review-status" className="mt-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {STATUS_FILTERS.map((filter) => (
                      <SelectItem key={filter.value} value={filter.value}>
                        {filter.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div>
                <label className="text-xs font-medium text-muted-foreground" htmlFor="candidate-review-provider">
                  Provider
                </label>
                <Select
                  value={normalizedRouteState.provider}
                  onValueChange={(value) =>
                    selectProvider(value as AdminMetadataCandidateReviewProviderFilter)
                  }
                >
                  <SelectTrigger id="candidate-review-provider" className="mt-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {PROVIDER_FILTERS.map((filter) => (
                      <SelectItem key={filter.value} value={filter.value}>
                        {filter.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          ) : (
            <form className="flex min-w-0 flex-1 flex-col gap-3 sm:flex-row sm:items-end" onSubmit={submitItemId}>
              <div className="min-w-0 flex-1">
                <label className="text-xs font-medium text-muted-foreground" htmlFor="candidate-review-item-id">
                  Media Item ID
                </label>
                <Input
                  id="candidate-review-item-id"
                  aria-label="Media Item ID"
                  className="mt-1 font-mono text-sm"
                  value={draftItemId}
                  onChange={(event) => setDraftItemId(event.target.value)}
                  placeholder="item id"
                />
              </div>
              <Button type="submit" size="sm" disabled={!draftItemId.trim()}>
                加载列表
              </Button>
            </form>
          )}
        </div>
      </section>

      {showList && (
        <CandidateReviewList
          list={listData}
          isLoading={isListLoading}
          selectedReviewId={reviewId}
          routeState={normalizedRouteState}
          mode={mode}
          onSelectReview={selectReview}
          onMovePage={moveListPage}
        />
      )}

      {mode === "item" && !itemId && !reviewId && (
        <section className="rounded-lg border border-border/50 bg-card p-8 text-center">
          <h2 className="text-sm font-medium text-foreground">缺少 Media Item ID</h2>
          <p className="mt-1 text-sm text-muted-foreground">
            输入 Media Item ID 查看 durable Candidate Reviews。
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
    mode: normalizeMode(routeState),
    itemId: routeState?.itemId?.trim() || DEFAULT_REVIEW_STATE.itemId,
    reviewId: routeState?.reviewId?.trim() || DEFAULT_REVIEW_STATE.reviewId,
    status: normalizeStatus(routeState?.status),
    provider: normalizeProvider(routeState?.provider),
    limit: routeState?.limit && routeState.limit > 0 ? routeState.limit : DEFAULT_REVIEW_STATE.limit,
    offset: routeState?.offset && routeState.offset > 0 ? routeState.offset : DEFAULT_REVIEW_STATE.offset,
  }
}

function normalizeMode(
  routeState?: AdminMetadataCandidateReviewRouteState,
): AdminMetadataCandidateReviewMode {
  if (routeState?.mode === "item" || routeState?.mode === "queue") {
    return routeState.mode
  }

  return routeState?.itemId ? "item" : DEFAULT_REVIEW_STATE.mode
}

function normalizeStatus(
  value: AdminMetadataCandidateReviewRouteState["status"],
): AdminMetadataCandidateReviewStatusFilter {
  return STATUS_FILTERS.some((filter) => filter.value === value) ? value ?? DEFAULT_STATUS : DEFAULT_STATUS
}

function normalizeProvider(
  value: AdminMetadataCandidateReviewRouteState["provider"],
): AdminMetadataCandidateReviewProviderFilter {
  return PROVIDER_FILTERS.some((filter) => filter.value === value) ? value ?? DEFAULT_PROVIDER : DEFAULT_PROVIDER
}

function metadataCandidateReviewQueueQueryKey(
  status: AdminMetadataCandidateReviewStatusFilter,
  provider: AdminMetadataCandidateReviewProviderFilter,
  limit: number,
  offset: number,
) {
  return ["nako", "admin", "metadata-candidate-review-queue", status, provider, limit, offset] as const
}

function metadataCandidateReviewListQueryKey(itemId: string, limit: number, offset: number) {
  return ["nako", "admin", "metadata-candidate-review-list", itemId, limit, offset] as const
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

function CandidateReviewList({
  list,
  isLoading,
  selectedReviewId,
  routeState,
  mode,
  onSelectReview,
  onMovePage,
}: {
  list?: AdminMetadataCandidateReviewListReadModel | AdminMetadataCandidateReviewQueueReadModel
  isLoading: boolean
  selectedReviewId: string
  routeState: Required<AdminMetadataCandidateReviewRouteState>
  mode: AdminMetadataCandidateReviewMode
  onSelectReview: (reviewId: string) => void
  onMovePage: (direction: "previous" | "next") => void
}) {
  const canPageBack = routeState.offset > 0
  const canPageForward = Boolean(list && list.page.returned >= routeState.limit)

  return (
    <section className="rounded-lg border border-border/50 bg-card">
      <div className="flex flex-col gap-3 border-b border-border/50 p-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-sm font-medium text-foreground">
            {mode === "queue" ? "Global Candidate Review queue" : "Candidate Reviews"}
          </h2>
          <p className="mt-1 text-xs text-muted-foreground">
            返回 {list?.page.returned ?? 0} 项，偏移 {routeState.offset}，每页 {routeState.limit}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          {mode === "queue" ? (
            <>
              <Badge variant="outline">status {routeState.status}</Badge>
              <Badge variant="outline">provider {routeState.provider}</Badge>
            </>
          ) : (
            <Badge variant="outline" className="font-mono">
              {routeState.itemId}
            </Badge>
          )}
          {list && <Badge variant="outline">Admin API {list.versions.adminApi}</Badge>}
        </div>
      </div>

      {isLoading ? (
        <ReviewSkeleton />
      ) : !list || list.reviews.length === 0 ? (
        <div className="p-8 text-center text-sm text-muted-foreground">没有 Candidate Reviews。</div>
      ) : (
        <div className="divide-y divide-border/50">
          {list.reviews.map((review) => (
            <CandidateReviewListRow
              key={review.reviewId}
              review={review}
              selected={review.reviewId === selectedReviewId}
              onSelectReview={onSelectReview}
            />
          ))}
        </div>
      )}

      <div className="flex items-center justify-between border-t border-border/50 p-4">
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="gap-2"
          disabled={!canPageBack}
          onClick={() => onMovePage("previous")}
        >
          上一页
        </Button>
        <span className="text-xs text-muted-foreground">Offset {routeState.offset}</span>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="gap-2"
          disabled={!canPageForward}
          onClick={() => onMovePage("next")}
        >
          下一页
        </Button>
      </div>
    </section>
  )
}

function CandidateReviewListRow({
  review,
  selected,
  onSelectReview,
}: {
  review: AdminMetadataCandidateReviewListItemReadModel
  selected: boolean
  onSelectReview: (reviewId: string) => void
}) {
  return (
    <div className={`grid gap-3 p-4 text-sm lg:grid-cols-[minmax(10rem,0.8fr)_1fr_minmax(8rem,0.5fr)_10rem] ${selected ? "bg-muted/50" : ""}`}>
      <div className="min-w-0 font-mono text-xs text-foreground">{review.reviewId}</div>
      <div className="min-w-0">
        <div className="truncate text-foreground">
          {review.root.metadata.title ?? review.root.subject?.title ?? "unknown"}
        </div>
        <div className="mt-1 truncate font-mono text-xs text-muted-foreground">
          {review.status} · {review.applicationAction} · {review.sourceLabel}:{review.sourceKey}
        </div>
      </div>
      <div className="min-w-0 font-mono text-xs text-muted-foreground">{review.itemId}</div>
      <div className="flex items-center justify-between gap-3 lg:justify-end">
        <Button
          type="button"
          variant={selected ? "secondary" : "outline"}
          size="sm"
          aria-label={`查看 Candidate Review ${review.reviewId}`}
          onClick={() => onSelectReview(review.reviewId)}
        >
          查看
        </Button>
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
