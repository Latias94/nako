"use client"

import { useEffect, useMemo, useState } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import {
  ADMIN_GENERATED_ARTIFACT_METADATA_APPLY_PLAN_FIXTURE,
  createAdminReadModelsDataSource,
  type AdminGeneratedArtifactMetadataApplyFieldPlanReadModel,
  type AdminGeneratedArtifactMetadataApplyPlanReadModel,
} from "@/src/api/admin/read-models-data-source"
import {
  createAdminMutationDataSource,
  type AdminGeneratedArtifactMetadataApplyMutationResult,
} from "@/src/api/admin/mutations-data-source"

export interface AdminGeneratedArtifactMetadataApplyRouteState {
  artifactId?: string
}

interface AdminGeneratedArtifactMetadataApplyProps {
  routeState?: AdminGeneratedArtifactMetadataApplyRouteState
  onRouteStateChange?: (state: AdminGeneratedArtifactMetadataApplyRouteState) => void
  onBackToQueue?: () => void
}

const DEFAULT_APPLY_STATE: Required<AdminGeneratedArtifactMetadataApplyRouteState> = {
  artifactId: "",
}

export function AdminGeneratedArtifactMetadataApply({
  routeState,
  onBackToQueue,
}: AdminGeneratedArtifactMetadataApplyProps = {}) {
  const normalizedRouteState = useMemo(() => normalizeApplyRouteState(routeState), [routeState])
  const [armed, setArmed] = useState(false)
  const [idempotencyKey, setIdempotencyKey] = useState("")
  const queryClient = useQueryClient()
  const mutationDataSource = useMemo(() => createAdminMutationDataSource(), [])
  const artifactId = normalizedRouteState.artifactId

  useEffect(() => {
    setArmed(false)
    setIdempotencyKey(artifactId ? createMetadataApplyIdempotencyKey(artifactId) : "")
  }, [artifactId])

  const {
    data = artifactId
      ? ADMIN_GENERATED_ARTIFACT_METADATA_APPLY_PLAN_FIXTURE
      : undefined,
    isLoading,
    isFetching,
    refetch,
  } = useQuery({
    queryKey: generatedArtifactMetadataApplyPlanQueryKey(artifactId),
    queryFn: () => createAdminReadModelsDataSource().loadGeneratedArtifactMetadataApplyPlan(artifactId),
    enabled: artifactId.length > 0,
    staleTime: 10 * 1000,
    retry: 0,
  })

  const applyMutation = useMutation({
    mutationFn: () => mutationDataSource.applyGeneratedArtifactMetadata(artifactId, idempotencyKey),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["nako", "admin", "generated-artifacts"] })
      void queryClient.invalidateQueries({
        queryKey: generatedArtifactMetadataApplyPlanQueryKey(artifactId),
      })
    },
  })

  if (!artifactId) {
    return (
      <div className="space-y-6">
        <ApplyHeader
          source="fixture"
          isFetching={false}
          onBackToQueue={onBackToQueue}
          onRefresh={undefined}
        />
        <section className="rounded-lg border border-border/50 bg-card p-8 text-center">
          <h2 className="text-sm font-medium text-foreground">缺少生成产物 ID</h2>
          <p className="mt-1 text-sm text-muted-foreground">返回队列选择一个已接受的 Generated Artifact。</p>
          <Button type="button" variant="outline" size="sm" className="mt-4" onClick={onBackToQueue}>
            返回队列
          </Button>
        </section>
      </div>
    )
  }

  if (isLoading || !data) {
    return (
      <div className="space-y-6">
        <ApplyHeader
          source="fixture"
          isFetching={isFetching}
          onBackToQueue={onBackToQueue}
          onRefresh={() => {
            void refetch()
          }}
        />
        <ApplySkeleton />
      </div>
    )
  }

  const mutationUnavailable = !mutationDataSource.canMutate
  const planFallback = data.fallback && mutationDataSource.canMutate
  const planUnavailable = planFallback || !data.executable
  const canSubmit =
    !mutationUnavailable && !planUnavailable && armed && !applyMutation.isPending && idempotencyKey.length > 0
  const result = applyMutation.data

  return (
    <div className="space-y-6">
      <ApplyHeader
        source={data.source}
        isFetching={isFetching}
        onBackToQueue={onBackToQueue}
        onRefresh={() => {
          void refetch()
        }}
      />

      {data.fallback && data.error && (
        <div className="flex items-start gap-3 rounded-lg border border-warning/30 bg-warning/5 p-4 text-sm">
          <div>
            <p className="font-medium text-foreground">Admin API 不可用，正在显示 fixture 应用计划</p>
            <p className="mt-1 text-muted-foreground">{data.error}</p>
          </div>
        </div>
      )}

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex flex-col gap-3 border-b border-border/50 p-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">应用计划</h2>
            <p className="mt-1 font-mono text-xs text-muted-foreground">{data.artifactId}</p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={data.executable ? "default" : "secondary"}>
              {data.executable ? "executable" : "blocked"}
            </Badge>
            <Badge variant="outline">{data.status}</Badge>
          </div>
        </div>

        <div className="grid gap-4 p-4 md:grid-cols-3">
          <Fact label="Target" value={`${data.target.kind} · ${data.target.itemId ?? "item unknown"}`} />
          <Fact label="Payload" value={`${data.payload.shape} · ${formatBytes(data.payload.payloadBytes)}`} />
          <Fact label="Fingerprint" value={data.payload.payloadFingerprint} />
          <div className="md:col-span-3">
            <div className="text-xs font-medium text-muted-foreground">Reasons</div>
            <div className="mt-1 flex flex-wrap gap-1.5">
              {data.reasons.length > 0 ? (
                data.reasons.map((reason) => (
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
      </section>

      <section className="rounded-lg border border-border/50 bg-card">
        <div className="flex flex-col gap-3 border-b border-border/50 p-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">字段计划</h2>
            <p className="mt-1 text-xs text-muted-foreground">
              只显示字段动作和摘要指纹，不显示原始 Generated Artifact payload。
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant="outline">apply {data.applyFieldCount}</Badge>
            <Badge variant="outline">skip {data.skippedFieldCount}</Badge>
            <Badge variant="outline">noop {data.noopFieldCount}</Badge>
          </div>
        </div>
        <FieldPlanTable fields={data.fields} />
      </section>

      <section className="rounded-lg border border-border/50 bg-card p-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 className="text-sm font-medium text-foreground">确认应用</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              通过 Metadata Authority 将可应用字段写入 Canonical Metadata。
            </p>
            {mutationUnavailable && (
              <p className="mt-2 text-sm text-warning">
                {mutationDataSource.unavailableReason ?? "当前连接不能执行管理操作"}
              </p>
            )}
            {planFallback && (
              <p className="mt-2 text-sm text-warning">
                应用计划不是 live Admin API 返回，不能执行确认。
              </p>
            )}
            {!data.executable && (
              <p className="mt-2 text-sm text-warning">
                {data.reasons.length > 0 ? data.reasons.join(", ") : "应用计划不可执行"}
              </p>
            )}
          </div>

          <div className="flex flex-wrap gap-2">
            {!armed ? (
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={mutationUnavailable || planUnavailable || applyMutation.isPending}
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
                  disabled={!canSubmit}
                  onClick={() => applyMutation.mutate()}
                >
                  {applyMutation.isPending ? "正在应用" : "确认应用"}
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

        {result && <ApplyResult result={result} />}
      </section>
    </div>
  )
}

export function normalizeGeneratedArtifactMetadataApplyRouteState(
  routeState?: AdminGeneratedArtifactMetadataApplyRouteState,
): Required<AdminGeneratedArtifactMetadataApplyRouteState> {
  return normalizeApplyRouteState(routeState)
}

function normalizeApplyRouteState(
  routeState?: AdminGeneratedArtifactMetadataApplyRouteState,
): Required<AdminGeneratedArtifactMetadataApplyRouteState> {
  return {
    artifactId: routeState?.artifactId?.trim() || DEFAULT_APPLY_STATE.artifactId,
  }
}

function generatedArtifactMetadataApplyPlanQueryKey(artifactId: string) {
  return ["nako", "admin", "generated-artifact-metadata-apply-plan", artifactId] as const
}

function ApplyHeader({
  source,
  isFetching,
  onBackToQueue,
  onRefresh,
}: {
  source: AdminGeneratedArtifactMetadataApplyPlanReadModel["source"]
  isFetching: boolean
  onBackToQueue?: () => void
  onRefresh?: () => void
}) {
  return (
    <div className="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
      <div className="max-w-3xl">
        <div className="mb-2 flex items-center gap-2">
          <h1 className="text-xl font-semibold text-foreground">Metadata Authority apply</h1>
        </div>
        <p className="text-sm text-muted-foreground">
          查看字段级 apply-plan 后确认 Canonical Metadata mutation。
        </p>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Badge variant={source === "live" ? "default" : "secondary"}>
          {source === "live" ? "Live Admin API" : "Fixture"}
        </Badge>
        <Button type="button" variant="outline" size="sm" onClick={onBackToQueue}>
          返回队列
        </Button>
        {onRefresh && (
          <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
            {isFetching ? "刷新中" : "刷新"}
          </Button>
        )}
      </div>
    </div>
  )
}

function FieldPlanTable({
  fields,
}: {
  fields: AdminGeneratedArtifactMetadataApplyFieldPlanReadModel[]
}) {
  return (
    <div className="overflow-x-auto">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>字段</TableHead>
            <TableHead>动作</TableHead>
            <TableHead>原因</TableHead>
            <TableHead>Current fingerprint</TableHead>
            <TableHead>Incoming fingerprint</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {fields.map((field) => (
            <TableRow key={field.field}>
              <TableCell className="font-mono text-xs text-foreground">{field.field}</TableCell>
              <TableCell>
                <Badge variant={fieldActionVariant(field.action)}>{field.action}</Badge>
              </TableCell>
              <TableCell>
                <div className="flex max-w-[18rem] flex-wrap gap-1.5">
                  {field.reasons.length > 0 ? (
                    field.reasons.map((reason) => (
                      <Badge key={reason} variant="secondary">
                        {reason}
                      </Badge>
                    ))
                  ) : (
                    <span className="text-xs text-muted-foreground">none</span>
                  )}
                </div>
              </TableCell>
              <TableCell className="font-mono text-xs text-muted-foreground">
                {field.current.valueFingerprint ?? "none"}
              </TableCell>
              <TableCell className="font-mono text-xs text-muted-foreground">
                {field.incoming.valueFingerprint ?? "none"}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

function ApplyResult({ result }: { result: AdminGeneratedArtifactMetadataApplyMutationResult }) {
  return (
    <div className="mt-4 rounded-lg border border-success/30 bg-success/5 p-4">
      <div className="flex flex-wrap items-center gap-2">
        <span className="text-sm font-medium text-foreground">{result.message}</span>
        <Badge variant="outline">{result.status}</Badge>
        <Badge variant={result.idempotentReplay ? "secondary" : "outline"}>
          {result.idempotentReplay ? "idempotent replay" : "new apply"}
        </Badge>
      </div>
      <div className="mt-3 grid gap-3 sm:grid-cols-4">
        <Fact label="Artifact" value={result.artifactId} />
        <Fact label="Outcome" value={result.outcomeId ?? "none"} />
        <Fact label="Applied" value={result.applied ? "true" : "false"} />
        <Fact label="Changed" value={result.changed ? "true" : "false"} />
      </div>
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

function ApplySkeleton() {
  return (
    <div className="space-y-4">
      <div className="h-40 rounded-lg bg-muted/70" />
      <div className="h-72 rounded-lg bg-muted/60" />
      <div className="h-32 rounded-lg bg-muted/50" />
    </div>
  )
}

function fieldActionVariant(action: string): "default" | "secondary" | "destructive" | "outline" {
  switch (action) {
    case "apply":
      return "default"
    case "skip":
      return "secondary"
    case "blocked":
      return "destructive"
    default:
      return "outline"
  }
}

function createMetadataApplyIdempotencyKey(artifactId: string) {
  const safeArtifactId = artifactId.replace(/[^A-Za-z0-9._:-]+/g, "-").slice(0, 64) || "artifact"
  const nonce = globalThis.crypto?.randomUUID?.() ?? `${Date.now()}-${Math.random().toString(36).slice(2)}`
  return `web-generated-artifact-metadata-apply:${safeArtifactId}:${nonce}`
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
