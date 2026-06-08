import { ArrowLeft, CheckCircle2, RefreshCw, ShieldCheck, X } from "lucide-react";
import { Link } from "@tanstack/react-router";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState, type ReactNode } from "react";

import {
  isLiveSectionResult,
  requireLiveSectionResult,
} from "../../adminApi/dataSource";
import type {
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationCandidate,
} from "../../adminApi/types";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import type { SourceDuplicateReconciliationDataAdapter } from "./sourceDuplicateReconciliationData";

export type SourceDuplicateReconciliationSearch = {
  library_id?: string;
  limit: number;
  offset: number;
};

export type SourceDuplicateReconciliationPageProps = {
  dataAdapter: SourceDuplicateReconciliationDataAdapter;
  itemId: string;
  sourceId: string;
  search: SourceDuplicateReconciliationSearch;
  onSearchChange(next: Partial<SourceDuplicateReconciliationSearch>): void;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

export function SourceDuplicateReconciliationPage({
  dataAdapter,
  itemId,
  sourceId,
  search,
  onSearchChange,
}: SourceDuplicateReconciliationPageProps) {
  const { locale, t } = useI18n();
  const libraryId = search.library_id;
  const query = useQuery({
    enabled: Boolean(libraryId),
    queryKey: [
      "admin-source-duplicate-reconciliation-plan",
      libraryId,
      sourceId,
      search.limit,
      search.offset,
      locale,
    ],
    queryFn: () =>
      dataAdapter.loadPlan(libraryId ?? "", sourceId, {
        limit: search.limit,
        offset: search.offset,
      }),
  });
  const result =
    query.data ??
    (libraryId
      ? {
          value: dataAdapter.createFallbackPlan(libraryId, sourceId),
          source: "mock" as const,
        }
      : null);
  const plan = result?.value ?? null;
  const canApplySuggestion = isLiveSectionResult(result);
  const hasPaginationDelta = search.limit !== 20 || search.offset !== 0;
  const [pendingDuplicateSourceId, setPendingDuplicateSourceId] = useState<string | null>(null);
  const [applyResult, setApplyResult] =
    useState<AdminSourceDuplicateReconciliationApplyResponse | null>(null);
  const [applyError, setApplyError] = useState<string | null>(null);
  const applyMutation = useMutation({
    mutationFn: async (duplicateSourceId: string) => {
      if (!libraryId) {
        throw new Error(t("sourceDuplicate.missingLibrary"));
      }

      requireLiveSectionResult(result, t("sourceDuplicate.notLiveError"));

      return dataAdapter.applySuggestion(libraryId, sourceId, duplicateSourceId);
    },
    onError(error: unknown) {
      setApplyResult(null);
      setApplyError(
        error instanceof Error ? error.message : t("sourceDuplicate.applyFailed"),
      );
    },
    onSuccess(value) {
      setPendingDuplicateSourceId(null);
      setApplyError(null);
      setApplyResult(value);
      void query.refetch();
    },
  });

  useEffect(() => {
    setPendingDuplicateSourceId(null);
    setApplyResult(null);
    setApplyError(null);
    applyMutation.reset();
  }, [libraryId, sourceId, search.limit, search.offset]);

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            className="routeTextLink routeBackLink"
            params={{ itemId }}
            to="/items/$itemId"
          >
            <ArrowLeft size={16} />
            {t("sourceDuplicate.backToItem")}
          </Link>
          <Button disabled={query.isFetching || !libraryId} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("sourceDuplicate.refresh")}
          </Button>
        </div>
      }
      description={t("sourceDuplicate.description")}
      kicker={t("sourceDuplicate.kicker")}
      status={<SourceLabel source={result?.source ?? "mock"} />}
      title={t("sourceDuplicate.title")}
      titleId="source-duplicate-reconciliation-route-title"
    >
      {!libraryId ? (
        <RouteNotice>{t("sourceDuplicate.missingLibrary")}</RouteNotice>
      ) : null}
      {result?.error ? (
        <RouteNotice>{t("sourceDuplicate.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      <FilterBar label={t("sourceDuplicate.pagination")}>
        <FilterField label={t("sourceDuplicate.limit")}>
          <input
            aria-label={t("sourceDuplicate.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) =>
              onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })
            }
          />
        </FilterField>
        <FilterField label={t("sourceDuplicate.offset")}>
          <input
            aria-label={t("sourceDuplicate.offsetAria")}
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) =>
              onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })
            }
          />
        </FilterField>
        <FilterActions>
          <Badge tone="info">{t("sourceDuplicate.redactedPlan")}</Badge>
          <Badge tone={plan?.stale ? "warning" : "success"}>
            {plan?.stale ? t("sourceDuplicate.stale") : t("sourceDuplicate.current")}
          </Badge>
          <Button
            disabled={!hasPaginationDelta}
            onClick={() => onSearchChange({ limit: 20, offset: 0 })}
            variant="ghost"
          >
            <X size={16} />
            {t("sourceDuplicate.reset")}
          </Button>
        </FilterActions>
      </FilterBar>

      {query.isLoading ? <RowsSkeleton label={t("sourceDuplicate.loading")} /> : null}

      {!query.isLoading && !libraryId ? (
        <EmptyRouteState>{t("sourceDuplicate.missingLibrary")}</EmptyRouteState>
      ) : null}

      {!query.isLoading && plan ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("sourceDuplicate.summary.description", {
              returned: plan.page.returned,
              offset: plan.page.offset,
              limit: plan.page.limit,
            })}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("sourceDuplicate.redactedPlan")}
              </div>
            }
            title={t("sourceDuplicate.summary.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("sourceDuplicate.summary.library")} value={plan.library_id} />
              <Fact label={t("sourceDuplicate.summary.mediaItem")} value={itemId} />
              <Fact label={t("sourceDuplicate.summary.source")} value={plan.source_id} />
              <Fact
                label={t("sourceDuplicate.summary.fingerprintEvidence")}
                value={plan.fingerprint_evidence_kind}
              />
              <Fact
                label={t("sourceDuplicate.summary.confidence")}
                value={confidenceLabel(plan.confidence_milli, t)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("sourceDuplicate.candidates.description")}
            title={t("sourceDuplicate.candidates.title")}
          >
            {plan.candidates.length === 0 ? (
              <EmptyRouteState>{t("sourceDuplicate.candidates.empty")}</EmptyRouteState>
            ) : (
              <div className="librarySourceSamples">
                {plan.candidates.map((candidate) => (
                  <CandidateRow
                    applyMutationPending={applyMutation.isPending}
                    canApply={canApplySuggestion}
                    candidate={candidate}
                    isConfirming={pendingDuplicateSourceId === candidate.duplicate_source_id}
                    key={candidate.duplicate_source_id}
                    onCancel={() => setPendingDuplicateSourceId(null)}
                    onConfirm={() => applyMutation.mutate(candidate.duplicate_source_id)}
                    onPrepare={() => {
                      setApplyError(null);
                      setPendingDuplicateSourceId(candidate.duplicate_source_id);
                    }}
                    t={t}
                  />
                ))}
              </div>
            )}
          </DataPanel>

          <DataPanel
            description={t("sourceDuplicate.result.description")}
            title={t("sourceDuplicate.result.title")}
          >
            <div className="librarySourceSamples">
              {applyError ? <RouteNotice>{applyError}</RouteNotice> : null}
              {applyResult ? <ApplyResult result={applyResult} t={t} /> : null}
              {!applyError && !applyResult ? (
                <EmptyRouteState>{t("sourceDuplicate.result.empty")}</EmptyRouteState>
              ) : null}
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

function CandidateRow({
  applyMutationPending,
  canApply,
  candidate,
  isConfirming,
  onCancel,
  onConfirm,
  onPrepare,
  t,
}: {
  applyMutationPending: boolean;
  canApply: boolean;
  candidate: AdminSourceDuplicateReconciliationCandidate;
  isConfirming: boolean;
  onCancel(): void;
  onConfirm(): void;
  onPrepare(): void;
  t: Translate;
}) {
  const canSuggest = candidate.recommended_action === "suggest_relationship";
  const canPrepare = canSuggest && canApply;

  return (
    <div className="librarySourceSample">
      <div>
        <strong>{candidate.duplicate_source_id}</strong>
        <span>
          {evidenceKindLabel(candidate.evidence_kind)} / {confidenceLabel(candidate.confidence_milli, t)} /{" "}
          {candidate.stale ? t("sourceDuplicate.stale") : t("sourceDuplicate.current")}
        </span>
        <span>
          {t("sourceDuplicate.candidates.relationship", {
            relationship: candidate.relationship_id ?? t("sourceDuplicate.none"),
            status: candidate.existing_status ?? t("sourceDuplicate.none"),
          })}
        </span>
      </div>
      <div className="routeActionGroup">
        <div className="issueBadgeList">
          <Badge tone={actionTone(candidate.recommended_action)}>
            {candidate.recommended_action}
          </Badge>
          <Badge tone={candidate.stale ? "warning" : "success"}>
            {candidate.stale ? t("sourceDuplicate.stale") : t("sourceDuplicate.current")}
          </Badge>
        </div>
        {!isConfirming ? (
          <Button
            disabled={!canPrepare || applyMutationPending}
            onClick={onPrepare}
            size="sm"
            variant={canPrepare ? "outline" : "ghost"}
          >
            <CheckCircle2 size={15} />
            {canSuggest
              ? t("sourceDuplicate.action.prepareSuggestion")
              : t("sourceDuplicate.action.noMutation")}
          </Button>
        ) : (
          <div className="routeActionGroup">
            <Button disabled={applyMutationPending} onClick={onCancel} size="sm" variant="ghost">
              {t("sourceDuplicate.action.cancel")}
            </Button>
            <Button disabled={applyMutationPending || !canApply} onClick={onConfirm} size="sm">
              <CheckCircle2 size={15} />
              {t("sourceDuplicate.action.confirmSuggestion")}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
}

function ApplyResult({
  result,
  t,
}: {
  result: AdminSourceDuplicateReconciliationApplyResponse;
  t: Translate;
}) {
  return (
    <>
      <div className="librarySourceSample">
        <div>
          <strong>{t("sourceDuplicate.result.suggestedRelationship")}</strong>
          <span>
            {result.source_id} / {result.duplicate_source_id}
          </span>
        </div>
        <Badge tone="success">{result.relationship_status}</Badge>
      </div>
      <div className="librarySourceSample">
        <div>
          <strong>{t("sourceDuplicate.result.relationship")}</strong>
          <span>{result.relationship_id}</span>
        </div>
        <Badge tone={result.created ? "success" : "neutral"}>
          {result.created
            ? t("sourceDuplicate.result.created")
            : t("sourceDuplicate.result.idempotent")}
        </Badge>
      </div>
    </>
  );
}

function Fact({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="libraryFactRow">
      <div>
        <strong>{label}</strong>
        <span>{value}</span>
      </div>
    </div>
  );
}

function actionTone(action: string): BadgeTone {
  if (action === "suggest_relationship") {
    return "warning";
  }

  if (action.startsWith("preserve_")) {
    return "neutral";
  }

  return "info";
}

function confidenceLabel(value: number | null, t: Translate) {
  return value === null
    ? t("sourceDuplicate.confidence.unknown")
    : t("sourceDuplicate.confidence.value", { confidence: value });
}

function evidenceKindLabel(value: AdminSourceDuplicateReconciliationCandidate["evidence_kind"]) {
  return typeof value === "string" ? value : value.other;
}

function numberInput(value: string): number | null {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

function nonNegativeNumberInput(value: string): number | null {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}
