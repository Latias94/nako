import { ArrowLeft, CheckCircle2, RefreshCw, ShieldCheck, X } from "lucide-react";
import { Link } from "@tanstack/react-router";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState, type ReactNode } from "react";

import {
  isLiveSectionResult,
  requireLiveSectionResult,
} from "../../adminApi/dataSource";
import type {
  AdminSourceDuplicateReconciliationApplyExpectedAction,
  AdminSourceDuplicateReconciliationApplyResponse,
  AdminSourceDuplicateReconciliationAction,
  AdminSourceDuplicateReconciliationCandidate,
  AdminSourceDuplicateRelationshipStatus,
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
  action?: SourceDuplicateReconciliationActionFilter;
  freshness?: SourceDuplicateReconciliationFreshnessFilter;
  library_id?: string;
  limit: number;
  offset: number;
  status?: SourceDuplicateReconciliationStatusFilter;
};

export type SourceDuplicateReconciliationStatusFilter =
  | "confirmed"
  | "none"
  | "rejected"
  | "suggested";

export type SourceDuplicateReconciliationActionFilter =
  AdminSourceDuplicateReconciliationAction;

export type SourceDuplicateReconciliationFreshnessFilter = "current" | "stale";

export type SourceDuplicateReconciliationPageProps = {
  dataAdapter: SourceDuplicateReconciliationDataAdapter;
  itemId: string;
  sourceId: string;
  search: SourceDuplicateReconciliationSearch;
  onSearchChange(next: Partial<SourceDuplicateReconciliationSearch>): void;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type Translate = (id: MessageId, values?: Record<string, number | string>) => string;
type ReviewSummary = {
  actionableSuggestions: number;
  preservedOrReadOnlyCandidates: number;
  staleOrRefreshCandidates: number;
};
type PendingReviewAction = {
  duplicateSourceId: string;
  expectedAction: AdminSourceDuplicateReconciliationApplyExpectedAction;
};
type QuickFilter = {
  action?: SourceDuplicateReconciliationActionFilter;
  freshness?: SourceDuplicateReconciliationFreshnessFilter;
  id: MessageId;
  status?: SourceDuplicateReconciliationStatusFilter;
};

const quickFilters = [
  {
    id: "sourceDuplicate.quick.pendingSuggestion",
    status: "none",
    action: "suggest_relationship",
    freshness: "current",
  },
  {
    id: "sourceDuplicate.quick.suggestedReview",
    status: "suggested",
    freshness: "current",
  },
  {
    id: "sourceDuplicate.quick.confirmed",
    status: "confirmed",
  },
  {
    id: "sourceDuplicate.quick.rejected",
    status: "rejected",
  },
  {
    id: "sourceDuplicate.quick.refreshNeeded",
    action: "refresh_source_fingerprint",
  },
  {
    id: "sourceDuplicate.quick.stale",
    freshness: "stale",
  },
] satisfies QuickFilter[];

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
  const canApplyReviewAction = isLiveSectionResult(result);
  const hasPaginationDelta = search.limit !== 20 || search.offset !== 0;
  const [pendingReviewAction, setPendingReviewAction] =
    useState<PendingReviewAction | null>(null);
  const [applyResult, setApplyResult] =
    useState<AdminSourceDuplicateReconciliationApplyResponse | null>(null);
  const [applyError, setApplyError] = useState<string | null>(null);
  const filteredCandidates = plan
    ? filterCandidates(plan.candidates, search)
    : [];
  const reviewSummary = plan ? summarizeReview(filteredCandidates) : null;
  const activeRelationshipFilterCount =
    (search.status ? 1 : 0) +
    (search.action ? 1 : 0) +
    (search.freshness ? 1 : 0);
  const applyMutation = useMutation({
    mutationFn: async (action: PendingReviewAction) => {
      if (!libraryId) {
        throw new Error(t("sourceDuplicate.missingLibrary"));
      }

      requireLiveSectionResult(result, t("sourceDuplicate.notLiveError"));

      return dataAdapter.applySuggestion(
        libraryId,
        sourceId,
        action.duplicateSourceId,
        action.expectedAction,
      );
    },
    onError(error: unknown) {
      setApplyResult(null);
      setApplyError(
        error instanceof Error ? error.message : t("sourceDuplicate.applyFailed"),
      );
    },
    onSuccess(value) {
      setPendingReviewAction(null);
      setApplyError(null);
      setApplyResult(value);
      void query.refetch();
    },
  });

  useEffect(() => {
    setPendingReviewAction(null);
    setApplyResult(null);
    setApplyError(null);
    applyMutation.reset();
  }, [
    libraryId,
    sourceId,
    search.action,
    search.freshness,
    search.limit,
    search.offset,
    search.status,
  ]);

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
        <FilterField label={t("sourceDuplicate.filter.status")}>
          <select
            aria-label={t("sourceDuplicate.filter.statusAria")}
            onChange={(event) =>
              onSearchChange({
                status: statusFilterInput(event.target.value),
                offset: 0,
              })
            }
            value={search.status ?? ""}
          >
            <option value="">{t("sourceDuplicate.filter.anyStatus")}</option>
            <option value="none">{t("sourceDuplicate.status.none")}</option>
            <option value="suggested">{t("sourceDuplicate.status.suggested")}</option>
            <option value="confirmed">{t("sourceDuplicate.status.confirmed")}</option>
            <option value="rejected">{t("sourceDuplicate.status.rejected")}</option>
          </select>
        </FilterField>
        <FilterField label={t("sourceDuplicate.filter.action")}>
          <select
            aria-label={t("sourceDuplicate.filter.actionAria")}
            onChange={(event) =>
              onSearchChange({
                action: actionFilterInput(event.target.value),
                offset: 0,
              })
            }
            value={search.action ?? ""}
          >
            <option value="">{t("sourceDuplicate.filter.anyAction")}</option>
            <option value="suggest_relationship">suggest_relationship</option>
            <option value="preserve_suggested">preserve_suggested</option>
            <option value="preserve_confirmed">preserve_confirmed</option>
            <option value="preserve_rejected">preserve_rejected</option>
            <option value="refresh_source_fingerprint">refresh_source_fingerprint</option>
          </select>
        </FilterField>
        <FilterField label={t("sourceDuplicate.filter.freshness")}>
          <select
            aria-label={t("sourceDuplicate.filter.freshnessAria")}
            onChange={(event) =>
              onSearchChange({
                freshness: freshnessFilterInput(event.target.value),
                offset: 0,
              })
            }
            value={search.freshness ?? ""}
          >
            <option value="">{t("sourceDuplicate.filter.anyFreshness")}</option>
            <option value="current">{t("sourceDuplicate.current")}</option>
            <option value="stale">{t("sourceDuplicate.stale")}</option>
          </select>
        </FilterField>
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
          {quickFilters.map((filter) => {
            const active = quickFilterActive(search, filter);
            return (
              <Button
                aria-pressed={active}
                key={filter.id}
                onClick={() =>
                  onSearchChange({
                    status: filter.status,
                    action: filter.action,
                    freshness: filter.freshness,
                    offset: 0,
                  })
                }
                size="sm"
                variant={active ? "default" : "outline"}
              >
                {t(filter.id)}
              </Button>
            );
          })}
          <Badge tone={activeRelationshipFilterCount > 0 ? "info" : "neutral"}>
            {t("sourceDuplicate.filter.active", {
              count: activeRelationshipFilterCount,
            })}
          </Badge>
          <Badge tone="info">{t("sourceDuplicate.redactedPlan")}</Badge>
          <Badge tone={plan?.stale ? "warning" : "success"}>
            {plan?.stale ? t("sourceDuplicate.stale") : t("sourceDuplicate.current")}
          </Badge>
          <Button
            disabled={!hasPaginationDelta && activeRelationshipFilterCount === 0}
            onClick={() =>
              onSearchChange({
                status: undefined,
                action: undefined,
                freshness: undefined,
                limit: 20,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("sourceDuplicate.clear")}
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

          {reviewSummary ? (
            <DataPanel
              description={t("sourceDuplicate.reviewSummary.description", {
                filtered: filteredCandidates.length,
                total: plan.page.returned,
              })}
              title={t("sourceDuplicate.reviewSummary.title")}
            >
              <div className="libraryFactList">
                <Fact
                  label={t("sourceDuplicate.reviewSummary.total")}
                  value={t("sourceDuplicate.reviewSummary.filteredValue", {
                    filtered: filteredCandidates.length,
                    total: plan.page.returned,
                  })}
                />
                <Fact
                  label={t("sourceDuplicate.reviewSummary.actionable")}
                  value={t("sourceDuplicate.reviewSummary.actionableValue", {
                    count: reviewSummary.actionableSuggestions,
                  })}
                />
                <Fact
                  label={t("sourceDuplicate.reviewSummary.preserved")}
                  value={t("sourceDuplicate.reviewSummary.preservedValue", {
                    count: reviewSummary.preservedOrReadOnlyCandidates,
                  })}
                />
                <Fact
                  label={t("sourceDuplicate.reviewSummary.staleRefresh")}
                  value={t("sourceDuplicate.reviewSummary.staleRefreshValue", {
                    count: reviewSummary.staleOrRefreshCandidates,
                  })}
                />
              </div>
            </DataPanel>
          ) : null}

          <DataPanel
            description={t("sourceDuplicate.candidates.description")}
            title={t("sourceDuplicate.candidates.title")}
          >
            {filteredCandidates.length === 0 ? (
              <EmptyRouteState>{t("sourceDuplicate.candidates.empty")}</EmptyRouteState>
            ) : (
              <div className="librarySourceSamples">
                {filteredCandidates.map((candidate) => (
                  <CandidateRow
                    applyMutationPending={applyMutation.isPending}
                    canApply={canApplyReviewAction}
                    candidate={candidate}
                    pendingAction={
                      pendingReviewAction?.duplicateSourceId === candidate.duplicate_source_id
                        ? pendingReviewAction.expectedAction
                        : null
                    }
                    key={candidate.duplicate_source_id}
                    onCancel={() => setPendingReviewAction(null)}
                    onConfirm={(action) =>
                      applyMutation.mutate({
                        duplicateSourceId: candidate.duplicate_source_id,
                        expectedAction: action,
                      })
                    }
                    onPrepare={(action) => {
                      setApplyError(null);
                      setPendingReviewAction({
                        duplicateSourceId: candidate.duplicate_source_id,
                        expectedAction: action,
                      });
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
  pendingAction,
  onCancel,
  onConfirm,
  onPrepare,
  t,
}: {
  applyMutationPending: boolean;
  canApply: boolean;
  candidate: AdminSourceDuplicateReconciliationCandidate;
  pendingAction: AdminSourceDuplicateReconciliationApplyExpectedAction | null;
  onCancel(): void;
  onConfirm(action: AdminSourceDuplicateReconciliationApplyExpectedAction): void;
  onPrepare(action: AdminSourceDuplicateReconciliationApplyExpectedAction): void;
  t: Translate;
}) {
  const canSuggest = candidate.recommended_action === "suggest_relationship";
  const canReviewSuggested = candidate.existing_status === "suggested" && !candidate.stale;
  const canPrepareSuggestion = canSuggest && canApply;
  const canPrepareReview = canReviewSuggested && canApply;

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
            status: relationshipStatusLabel(candidate.existing_status, t),
          })}
        </span>
      </div>
      <div className="routeActionGroup">
        <div className="issueBadgeList">
          <Badge tone={relationshipStatusTone(candidate.existing_status)}>
            {relationshipStatusLabel(candidate.existing_status, t)}
          </Badge>
          <Badge tone={actionTone(candidate.recommended_action)}>
            {candidate.recommended_action}
          </Badge>
          <Badge tone={candidate.stale ? "warning" : "success"}>
            {candidate.stale ? t("sourceDuplicate.stale") : t("sourceDuplicate.current")}
          </Badge>
        </div>
        {!pendingAction ? (
          <div className="routeActionGroup">
            {canSuggest ? (
              <Button
                disabled={!canPrepareSuggestion || applyMutationPending}
                onClick={() => onPrepare("suggest_relationship")}
                size="sm"
                variant={canPrepareSuggestion ? "outline" : "ghost"}
              >
                <CheckCircle2 size={15} />
                {t("sourceDuplicate.action.prepareSuggestion")}
              </Button>
            ) : null}
            {canReviewSuggested ? (
              <>
                <Button
                  disabled={!canPrepareReview || applyMutationPending}
                  onClick={() => onPrepare("confirm_suggested")}
                  size="sm"
                  variant={canPrepareReview ? "outline" : "ghost"}
                >
                  <CheckCircle2 size={15} />
                  {t("sourceDuplicate.action.prepareConfirm")}
                </Button>
                <Button
                  disabled={!canPrepareReview || applyMutationPending}
                  onClick={() => onPrepare("reject_suggested")}
                  size="sm"
                  variant="ghost"
                >
                  <X size={15} />
                  {t("sourceDuplicate.action.prepareReject")}
                </Button>
              </>
            ) : null}
            {!canSuggest && !canReviewSuggested ? (
              <Button disabled size="sm" variant="ghost">
                {t("sourceDuplicate.action.noMutation")}
              </Button>
            ) : null}
          </div>
        ) : (
          <div className="routeActionGroup">
            <Button disabled={applyMutationPending} onClick={onCancel} size="sm" variant="ghost">
              {t("sourceDuplicate.action.cancel")}
            </Button>
            <Button
              disabled={applyMutationPending || !canApply}
              onClick={() => onConfirm(pendingAction)}
              size="sm"
            >
              <CheckCircle2 size={15} />
              {confirmActionLabel(pendingAction, t)}
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
          <span>{result.applied_action}</span>
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

function confirmActionLabel(
  action: AdminSourceDuplicateReconciliationApplyExpectedAction,
  t: Translate,
) {
  switch (action) {
    case "confirm_suggested":
      return t("sourceDuplicate.action.confirmRelationship");
    case "reject_suggested":
      return t("sourceDuplicate.action.rejectRelationship");
    case "suggest_relationship":
      return t("sourceDuplicate.action.confirmSuggestion");
  }
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

function relationshipStatusTone(
  status: AdminSourceDuplicateRelationshipStatus | null,
): BadgeTone {
  switch (status) {
    case "confirmed":
      return "success";
    case "rejected":
      return "danger";
    case "suggested":
      return "warning";
    case null:
      return "neutral";
  }
}

function relationshipStatusLabel(
  status: AdminSourceDuplicateRelationshipStatus | null,
  t: Translate,
) {
  switch (status) {
    case "confirmed":
      return t("sourceDuplicate.status.confirmed");
    case "rejected":
      return t("sourceDuplicate.status.rejected");
    case "suggested":
      return t("sourceDuplicate.status.suggested");
    case null:
      return t("sourceDuplicate.status.none");
  }
}

function confidenceLabel(value: number | null, t: Translate) {
  return value === null
    ? t("sourceDuplicate.confidence.unknown")
    : t("sourceDuplicate.confidence.value", { confidence: value });
}

function evidenceKindLabel(value: AdminSourceDuplicateReconciliationCandidate["evidence_kind"]) {
  return typeof value === "string" ? value : value.other;
}

function summarizeReview(candidates: AdminSourceDuplicateReconciliationCandidate[]): ReviewSummary {
  return candidates.reduce(
    (summary, candidate) => {
      if (candidate.recommended_action === "suggest_relationship") {
        summary.actionableSuggestions += 1;
      }

      if (candidate.recommended_action !== "suggest_relationship") {
        summary.preservedOrReadOnlyCandidates += 1;
      }

      if (
        candidate.stale ||
        candidate.recommended_action === "refresh_source_fingerprint"
      ) {
        summary.staleOrRefreshCandidates += 1;
      }

      return summary;
    },
    {
      actionableSuggestions: 0,
      preservedOrReadOnlyCandidates: 0,
      staleOrRefreshCandidates: 0,
    },
  );
}

function filterCandidates(
  candidates: AdminSourceDuplicateReconciliationCandidate[],
  search: SourceDuplicateReconciliationSearch,
) {
  return candidates.filter((candidate) => {
    if (search.status) {
      const status = candidate.existing_status ?? "none";
      if (status !== search.status) {
        return false;
      }
    }

    if (search.action && candidate.recommended_action !== search.action) {
      return false;
    }

    if (search.freshness === "current" && candidate.stale) {
      return false;
    }

    if (search.freshness === "stale" && !candidate.stale) {
      return false;
    }

    return true;
  });
}

function quickFilterActive(
  search: SourceDuplicateReconciliationSearch,
  filter: QuickFilter,
) {
  return (
    search.status === filter.status &&
    search.action === filter.action &&
    search.freshness === filter.freshness
  );
}

function statusFilterInput(
  value: string,
): SourceDuplicateReconciliationStatusFilter | undefined {
  return value === "none" ||
    value === "suggested" ||
    value === "confirmed" ||
    value === "rejected"
    ? value
    : undefined;
}

function actionFilterInput(
  value: string,
): SourceDuplicateReconciliationActionFilter | undefined {
  return value === "suggest_relationship" ||
    value === "preserve_suggested" ||
    value === "preserve_confirmed" ||
    value === "preserve_rejected" ||
    value === "refresh_source_fingerprint"
    ? value
    : undefined;
}

function freshnessFilterInput(
  value: string,
): SourceDuplicateReconciliationFreshnessFilter | undefined {
  return value === "current" || value === "stale" ? value : undefined;
}

function numberInput(value: string): number | null {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

function nonNegativeNumberInput(value: string): number | null {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= 0 ? parsed : null;
}
