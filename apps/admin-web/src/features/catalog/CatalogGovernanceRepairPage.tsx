import { Link } from "@tanstack/react-router";
import { useMutation, useQuery } from "@tanstack/react-query";
import { ArrowLeft, CheckCircle2, RefreshCw, ShieldCheck, XCircle } from "lucide-react";
import { useEffect, useMemo, useState, type ReactNode } from "react";

import {
  isLiveDataSource,
  requireLiveDataSource,
  type AdminDataSource,
  type DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  CatalogGovernanceItemDetailSummary,
  CatalogGovernanceProviderMappingReviewDecision,
  CatalogGovernanceProviderMappingReviewPlanSummary,
  CatalogGovernanceProviderMappingReviewResultSummary,
} from "../../adminApi/types";
import {
  mockCatalogGovernanceItemDetail,
  mockCatalogGovernanceProviderMappingReviewPlan,
} from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterField } from "../../components/ui/FilterBar";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type CatalogGovernanceRepairSearch = {
  mapping_id?: string;
  decision: CatalogGovernanceProviderMappingReviewDecision;
};

export type CatalogGovernanceRepairPageProps = {
  dataSource: AdminDataSource;
  itemId: string;
  search: CatalogGovernanceRepairSearch;
  onSearchChange(next: Partial<CatalogGovernanceRepairSearch>): void;
};

type DetailResult = {
  value: CatalogGovernanceItemDetailSummary;
  source: DataSourceMode;
  error?: string;
};

type ReviewPlanResult = {
  value: CatalogGovernanceProviderMappingReviewPlanSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function CatalogGovernanceRepairPage({
  dataSource,
  itemId,
  search,
  onSearchChange,
}: CatalogGovernanceRepairPageProps) {
  const { locale, t } = useI18n();
  const detailQuery = useQuery({
    queryKey: ["admin-catalog-governance-detail", itemId, locale],
    queryFn: () => loadItemDetail(dataSource, itemId, t("catalogGovernance.repair.itemUnavailable")),
  });
  const detailResult = detailQuery.data ?? {
    value: mockItemDetailSummary(itemId),
    source: "mock" as const,
  };
  const detail = detailResult.value;
  const defaultMappingId = detail.providerMappings[0]?.id;
  const selectedMappingId = search.mapping_id ?? defaultMappingId;
  const selectedMapping = detail.providerMappings.find(
    (mapping) => mapping.id === selectedMappingId,
  );
  const planQuery = useQuery({
    enabled: Boolean(selectedMappingId),
    queryKey: [
      "admin-catalog-governance-provider-mapping-review-plan",
      itemId,
      selectedMappingId,
      search.decision,
    ],
    queryFn: () =>
      loadReviewPlan(
        dataSource,
        itemId,
        selectedMappingId ?? "",
        search.decision,
        t("catalogGovernance.repair.planUnavailable"),
      ),
  });
  const planResult =
    planQuery.data ??
    (selectedMappingId
      ? {
          value: mockReviewPlanSummary(itemId, selectedMappingId, search.decision),
          source: "mock" as const,
        }
      : null);
  const plan = planResult?.value ?? null;
  const source = combineSources(detailResult.source, planResult?.source);
  const canPrepareReview = isLiveDataSource(source);
  const [isConfirming, setIsConfirming] = useState(false);
  const [reviewResult, setReviewResult] =
    useState<CatalogGovernanceProviderMappingReviewResultSummary | null>(null);
  const [reviewError, setReviewError] = useState<string | null>(null);
  const reviewMutation = useMutation({
    mutationFn: async () => {
      if (!selectedMappingId) {
        throw new Error(t("catalogGovernance.repair.selectionUnavailable"));
      }

      requireLiveDataSource(source, t("catalogGovernance.repair.notLiveError"));

      if (!dataSource.reviewCatalogGovernanceProviderMapping) {
        throw new Error(t("catalogGovernance.repair.reviewUnavailable"));
      }

      return dataSource.reviewCatalogGovernanceProviderMapping(
        itemId,
        selectedMappingId,
        search.decision,
      );
    },
    onError(error: unknown) {
      setReviewResult(null);
      setReviewError(
        error instanceof Error
          ? error.message
          : t("catalogGovernance.repair.reviewFailed"),
      );
    },
    onSuccess(value) {
      setIsConfirming(false);
      setReviewError(null);
      setReviewResult(value);
    },
  });

  useEffect(() => {
    setIsConfirming(false);
    setReviewResult(null);
    setReviewError(null);
    reviewMutation.reset();
  }, [itemId, selectedMappingId, search.decision]);

  useEffect(() => {
    if (!search.mapping_id && defaultMappingId) {
      onSearchChange({ mapping_id: defaultMappingId });
    }
  }, [defaultMappingId, onSearchChange, search.mapping_id]);

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            className="routeTextLink routeBackLink"
            search={{
              library_id: undefined,
              max_confidence_milli: undefined,
              limit: 20,
              offset: 0,
            }}
            to="/catalog/governance"
          >
            <ArrowLeft size={16} />
            {t("catalogGovernance.repair.backToQueue")}
          </Link>
          <Button
            disabled={detailQuery.isFetching || planQuery.isFetching}
            onClick={() => {
              void detailQuery.refetch();
              void planQuery.refetch();
            }}
            variant="outline"
          >
            <RefreshCw size={16} />
            {t("catalogGovernance.repair.refresh")}
          </Button>
        </div>
      }
      description={t("catalogGovernance.repair.description")}
      kicker={t("catalogGovernance.repair.kicker")}
      status={<SourceLabel source={source} />}
      title={t("catalogGovernance.repair.title")}
      titleId="catalog-governance-repair-route-title"
    >
      {detailResult.error ? (
        <RouteNotice>
          {t("catalogGovernance.repair.detailFallback", { error: detailResult.error })}
        </RouteNotice>
      ) : null}
      {planResult?.error ? (
        <RouteNotice>
          {t("catalogGovernance.repair.reviewPlanFallback", { error: planResult.error })}
        </RouteNotice>
      ) : null}

      {detailQuery.isLoading ? (
        <RowsSkeleton label={t("catalogGovernance.repair.loading")} />
      ) : null}

      {!detailQuery.isLoading && !detail.item.id ? (
        <EmptyRouteState>
          {t("catalogGovernance.repair.missingItem", { itemId })}
        </EmptyRouteState>
      ) : null}

      {!detailQuery.isLoading && detail.item.id ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("catalogGovernance.repair.itemContext.description")}
            headerAccessory={<Badge tone={detail.item.kind === "unknown" ? "warning" : "neutral"}>{detail.item.kind}</Badge>}
            title={t("catalogGovernance.repair.itemContext.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("catalogGovernance.repair.itemId")} value={detail.item.id} />
              <Fact label={t("catalogGovernance.repair.itemTitle")} value={detail.item.title} />
              <Fact label={t("catalogGovernance.repair.mediaLibrary")} value={detail.item.libraryId} />
              <Fact label={t("catalogGovernance.repair.sources")} value={detail.item.sourceCount} />
              <Fact
                label={t("catalogGovernance.repair.providerMappings")}
                value={t("catalogGovernance.repair.accepted", {
                  accepted: detail.item.acceptedProviderMappingCount,
                  total: detail.item.providerMappingCount,
                })}
              />
              <Fact label={t("catalogGovernance.repair.issues")} value={listLabel(detail.item.issues, t)} />
              <Fact
                label={t("catalogGovernance.repair.localInference")}
                value={localInferenceLabel(detail.item, t)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("catalogGovernance.repair.providerSelection.description")}
            headerAccessory={<Badge tone={selectedMapping ? mappingStatusTone(selectedMapping.status) : "warning"}>{selectedMapping?.status ?? t("catalogGovernance.repair.providerSelection.none")}</Badge>}
            title={t("catalogGovernance.repair.providerSelection.title")}
          >
            <div className="libraryFactList">
              <FilterField label={t("catalogGovernance.repair.providerMappings")}>
                <select
                  aria-label={t("catalogGovernance.repair.providerSelection.selector")}
                  value={selectedMappingId ?? ""}
                  onChange={(event) => onSearchChange({ mapping_id: event.target.value || undefined })}
                >
                  {detail.providerMappings.map((mapping) => (
                    <option key={mapping.id} value={mapping.id}>
                      {t("catalogGovernance.repair.providerSelection.selectLabel", {
                        provider: mapping.subject.provider,
                        key: mapping.subject.key,
                        status: mapping.status,
                      })}
                    </option>
                  ))}
                </select>
              </FilterField>
            </div>
            {selectedMapping ? (
              <div className="librarySourceSamples">
                <div className="librarySourceSample">
                  <div>
                    <strong>{selectedMapping.subject.title ?? selectedMapping.subject.key}</strong>
                    <span>{selectedMapping.subject.provider}:{selectedMapping.subject.key}</span>
                  </div>
                  <Badge tone={mappingStatusTone(selectedMapping.status)}>{selectedMapping.status}</Badge>
                </div>
                <div className="librarySourceSample">
                  <div>
                    <strong>{t("catalogGovernance.repair.sources")}</strong>
                    <span>{selectedMapping.source}</span>
                  </div>
                  <Badge tone="info">{confidenceLabel(selectedMapping.confidenceMilli, t)}</Badge>
                </div>
              </div>
            ) : (
              <EmptyRouteState>
                {t("catalogGovernance.repair.providerSelection.noMapping")}
              </EmptyRouteState>
            )}
          </DataPanel>

          <DataPanel
            description={t("catalogGovernance.repair.reviewPlan.description")}
            headerAccessory={
              <div className="routeActionGroup" role="group" aria-label={t("catalogGovernance.repair.decisionGroup")}>
                <Button
                  aria-pressed={search.decision === "accept"}
                  disabled={reviewMutation.isPending}
                  onClick={() => onSearchChange({ decision: "accept" })}
                  size="sm"
                  variant={search.decision === "accept" ? "default" : "outline"}
                >
                  <CheckCircle2 size={15} />
                  {t("catalogGovernance.repair.accept")}
                </Button>
                <Button
                  aria-pressed={search.decision === "reject"}
                  disabled={reviewMutation.isPending}
                  onClick={() => onSearchChange({ decision: "reject" })}
                  size="sm"
                  variant={search.decision === "reject" ? "default" : "outline"}
                >
                  <XCircle size={15} />
                  {t("catalogGovernance.repair.reject")}
                </Button>
              </div>
            }
            title={t("catalogGovernance.repair.reviewPlan.title")}
          >
            {planQuery.isLoading ? <RowsSkeleton label={t("catalogGovernance.repair.reviewPlan.loading")} /> : null}
            {!planQuery.isLoading && plan ? (
              <div className="libraryFactList">
                <Fact label={t("catalogGovernance.repair.reviewPlan.decision")} value={plan.decision} />
                <Fact label={t("catalogGovernance.repair.reviewPlan.planStatus")} value={plan.status} />
                <Fact label={t("catalogGovernance.repair.reviewPlan.currentStatus")} value={plan.currentStatus} />
                <Fact label={t("catalogGovernance.repair.reviewPlan.targetStatus")} value={plan.targetStatus} />
                <Fact
                  label={t("catalogGovernance.repair.reviewPlan.readiness")}
                  value={t("catalogGovernance.repair.reviewPlan.readinessGroup", {
                    status: plan.readiness.status,
                    reasons: listLabel(plan.readiness.reasons, t),
                  })}
                />
              </div>
            ) : null}
            {!planQuery.isLoading && !plan ? (
              <EmptyRouteState>{t("catalogGovernance.repair.reviewPlan.empty")}</EmptyRouteState>
            ) : null}
          </DataPanel>

          <DataPanel
            description={t("catalogGovernance.repair.boundaries.description")}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("catalogGovernance.repair.boundaries.redacted")}
              </div>
            }
            title={t("catalogGovernance.repair.boundaries.title")}
          >
            <div className="librarySourceSamples">
              {plan ? (
                <>
                  <BoundaryRow enabled={plan.boundary.updatesProviderMappingStatus} label={t("catalogGovernance.repair.boundaries.providerMappingStatus")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesCanonicalMetadata} label={t("catalogGovernance.repair.boundaries.canonicalMetadata")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesProviderSubject} label={t("catalogGovernance.repair.boundaries.providerSubject")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesLocalInference} label={t("catalogGovernance.repair.boundaries.localInference")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesSourceDuplicates} label={t("catalogGovernance.repair.boundaries.sourceDuplicates")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesHierarchy} label={t("catalogGovernance.repair.boundaries.hierarchy")} t={t} />
                  <BoundaryRow enabled={plan.boundary.writesNfo} label={t("catalogGovernance.repair.boundaries.writesNfo")} t={t} />
                  <BoundaryRow enabled={plan.boundary.writesLibraryFiles} label={t("catalogGovernance.repair.boundaries.writesLibraryFiles")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesArtwork} label={t("catalogGovernance.repair.boundaries.artwork")} t={t} />
                  <BoundaryRow enabled={plan.boundary.updatesPlaybackState} label={t("catalogGovernance.repair.boundaries.playbackState")} t={t} />
                </>
              ) : (
                <EmptyRouteState>{t("catalogGovernance.repair.boundaries.noPlan")}</EmptyRouteState>
              )}
            </div>
          </DataPanel>

          <DataPanel
            description={t("catalogGovernance.repair.confirmed.description")}
            title={t("catalogGovernance.repair.confirmed.title")}
          >
            <div className="librarySourceSamples">
              {reviewError ? <RouteNotice>{reviewError}</RouteNotice> : null}

              {plan ? (
                <div className="librarySourceSample">
                  <div>
                    <strong>{t("catalogGovernance.repair.selectedDecision")}</strong>
                    <span>
                      {t("catalogGovernance.repair.result.selectedDecision", {
                        decision: plan.decision,
                        mappingId: plan.mapping.id,
                        itemId: plan.item.id,
                      })}
                    </span>
                  </div>
                  <Badge tone={plan.decision === "accept" ? "success" : "danger"}>{plan.decision}</Badge>
                </div>
              ) : null}

              {!reviewResult && !isConfirming ? (
                <div className="librarySourceSample">
                  <div>
                    <strong>{t("catalogGovernance.repair.prepareConfirmation")}</strong>
                    <span>{t("catalogGovernance.repair.prepareCopy")}</span>
                  </div>
                  <Button
                    disabled={!plan || planQuery.isLoading || reviewMutation.isPending || !canPrepareReview}
                    onClick={() => {
                      setReviewError(null);
                      setIsConfirming(true);
                    }}
                    variant="outline"
                  >
                    {search.decision === "accept" ? <CheckCircle2 size={15} /> : <XCircle size={15} />}
                    {t("catalogGovernance.repair.prepareButton", {
                      decision: search.decision,
                    })}
                  </Button>
                </div>
              ) : null}

              {!reviewResult && isConfirming ? (
                <div className="librarySourceSample">
                  <div>
                    <strong>
                      {t("catalogGovernance.repair.confirmLabel", {
                        decision: search.decision,
                      })}
                    </strong>
                    <span>
                      {t("catalogGovernance.repair.confirmCopy", {
                        mappingId: selectedMappingId ?? "",
                      })}
                    </span>
                  </div>
                  <div className="routeActionGroup">
                    <Button disabled={reviewMutation.isPending} onClick={() => setIsConfirming(false)} variant="ghost">
                      {t("catalogGovernance.repair.cancel")}
                    </Button>
                    <Button
                      disabled={reviewMutation.isPending || !canPrepareReview}
                      onClick={() => reviewMutation.mutate()}
                    >
                      {search.decision === "accept" ? <CheckCircle2 size={15} /> : <XCircle size={15} />}
                      {t("catalogGovernance.repair.confirmButton", {
                        decision: search.decision,
                      })}
                    </Button>
                  </div>
                </div>
              ) : null}

              {reviewResult ? <ReviewResult result={reviewResult} t={t} /> : null}
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

async function loadItemDetail(
  dataSource: AdminDataSource,
  itemId: string,
  missingDataSourceMessage: string,
): Promise<DetailResult> {
  if (!dataSource.loadCatalogGovernanceItemDetail) {
    return {
      value: mockItemDetailSummary(itemId),
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadCatalogGovernanceItemDetail(itemId);
}

async function loadReviewPlan(
  dataSource: AdminDataSource,
  itemId: string,
  mappingId: string,
  decision: CatalogGovernanceProviderMappingReviewDecision,
  missingDataSourceMessage: string,
): Promise<ReviewPlanResult> {
  if (!dataSource.loadCatalogGovernanceProviderMappingReviewPlan) {
    return {
      value: mockReviewPlanSummary(itemId, mappingId, decision),
      source: "mock",
      error: missingDataSourceMessage,
    };
  }

  return dataSource.loadCatalogGovernanceProviderMappingReviewPlan(
    itemId,
    mappingId,
    decision,
  );
}

function mockItemDetailSummary(itemId: string): CatalogGovernanceItemDetailSummary {
  const detail = mockCatalogGovernanceItemDetail(itemId);

  return {
    item: {
      id: detail.item.item_id,
      libraryId: detail.item.library_id,
      kind: detail.item.kind,
      parentId: detail.item.parent_id,
      title: detail.item.title,
      releaseDate: detail.item.release_date,
      issues: detail.item.issues,
      sourceCount: detail.item.source_count,
      representativeSourceId: detail.item.representative_source_id,
      representativeFileName: detail.item.representative_file_name,
      providerMappingCount: detail.item.provider_mapping_count,
      acceptedProviderMappingCount: detail.item.accepted_provider_mapping_count,
      duplicateRelationshipCount: detail.item.duplicate_relationship_count,
      localInference: detail.item.local_inference
        ? {
            sourceId: detail.item.local_inference.source_id,
            inferredKind: detail.item.local_inference.inferred_kind,
            inferredTitle: detail.item.local_inference.inferred_title,
            inferredYear: detail.item.local_inference.inferred_year,
            inferredSeason: detail.item.local_inference.inferred_season,
            inferredEpisode: detail.item.local_inference.inferred_episode,
            confidenceMilli: detail.item.local_inference.confidence_milli,
            evidenceSource: detail.item.local_inference.evidence_source,
            hasEvidence: detail.item.local_inference.has_evidence,
            inferenceVersion: detail.item.local_inference.inference_version,
          }
        : null,
    },
    providerMappings: detail.provider_mappings.map((mapping) => ({
      id: mapping.mapping_id,
      itemId: mapping.item_id,
      status: mapping.status,
      confidenceMilli: mapping.confidence_milli,
      source: typeof mapping.source === "string" ? mapping.source : "provider:tmdb",
      subject: {
        id: mapping.subject.subject_id,
        provider: typeof mapping.subject.provider === "string" ? mapping.subject.provider : mapping.subject.provider.other,
        kind: typeof mapping.subject.subject_kind === "string" ? mapping.subject.subject_kind : mapping.subject.subject_kind.other,
        key: mapping.subject.subject_key,
        title: mapping.subject.title,
        releaseYear: mapping.subject.release_year,
        locale: mapping.subject.locale,
      },
    })),
    repairActions: detail.repair_actions,
  };
}

function mockReviewPlanSummary(
  itemId: string,
  mappingId: string,
  decision: CatalogGovernanceProviderMappingReviewDecision,
): CatalogGovernanceProviderMappingReviewPlanSummary {
  const { plan } = mockCatalogGovernanceProviderMappingReviewPlan(
    itemId,
    mappingId,
    decision,
  );
  const detail = mockItemDetailSummary(itemId);
  const mapping =
    detail.providerMappings.find((candidate) => candidate.id === mappingId) ??
    detail.providerMappings[0];

  return {
    item: detail.item,
    mapping: {
      ...mapping,
      id: mappingId,
    },
    decision: plan.decision,
    currentStatus: plan.current_status,
    targetStatus: plan.target_status,
    status: plan.status,
    readiness: {
      status: plan.readiness.status,
      actionable: plan.readiness.actionable,
      reasons: plan.readiness.reasons,
    },
    boundary: {
      updatesProviderMappingStatus: plan.boundary.updates_provider_mapping_status,
      updatesCanonicalMetadata: plan.boundary.updates_canonical_metadata,
      updatesProviderSubject: plan.boundary.updates_provider_subject,
      updatesLocalInference: plan.boundary.updates_local_inference,
      updatesSourceDuplicates: plan.boundary.updates_source_duplicates,
      updatesHierarchy: plan.boundary.updates_hierarchy,
      writesNfo: plan.boundary.writes_nfo,
      writesLibraryFiles: plan.boundary.writes_library_files,
      updatesArtwork: plan.boundary.updates_artwork,
      updatesPlaybackState: plan.boundary.updates_playback_state,
    },
  };
}

function Fact({ label, value }: { label: string; value: ReactNode }) {
  return (
    <div className="libraryFactRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
      </div>
    </div>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function BoundaryRow({ enabled, label, t }: { enabled: boolean; label: string; t: Translate }) {
  return (
    <div className="librarySourceSample">
      <div>
        <strong>{label}</strong>
        <span>
          {enabled
            ? t("catalogGovernance.repair.boundaries.included")
            : t("catalogGovernance.repair.boundaries.excluded")}
        </span>
      </div>
      <Badge tone={enabled ? "warning" : "neutral"}>
        {enabled
          ? t("catalogGovernance.repair.boundaries.yes")
          : t("catalogGovernance.repair.boundaries.no")}
      </Badge>
    </div>
  );
}

function ReviewResult({
  result,
  t,
}: {
  result: CatalogGovernanceProviderMappingReviewResultSummary;
  t: Translate;
}) {
  return (
    <>
      <div className="librarySourceSample">
        <div>
          <strong>{t("catalogGovernance.repair.result.accepted")}</strong>
          <span>{result.mappingId}</span>
        </div>
        <Badge tone={result.currentStatus === "accepted" ? "success" : "danger"}>
          {result.currentStatus}
        </Badge>
      </div>
      <div className="librarySourceSample">
        <div>
          <strong>{t("catalogGovernance.repair.result.statusChange")}</strong>
          <span>
            {result.previousStatus} to {result.currentStatus}
          </span>
        </div>
        <Badge tone={result.changed ? "success" : "neutral"}>
          {result.idempotentReplay
            ? t("catalogGovernance.repair.result.idempotentReplay")
            : t("catalogGovernance.repair.result.newResult")}
        </Badge>
      </div>
    </>
  );
}

function combineSources(
  detailSource: DataSourceMode,
  planSource: DataSourceMode | undefined,
): DataSourceMode {
  if (!planSource || detailSource === planSource) {
    return detailSource;
  }

  if (detailSource === "live" || planSource === "live") {
    return "hybrid";
  }

  return "mock";
}

function listLabel(values: string[], t: Translate) {
  return values.length > 0 ? values.join(", ") : t("catalogGovernance.issues.none");
}

function localInferenceLabel(item: CatalogGovernanceItemDetailSummary["item"], t: Translate) {
  if (!item.localInference) {
    return t("catalogGovernance.issues.none");
  }

  return `${item.localInference.inferredKind} / ${confidenceLabel(item.localInference.confidenceMilli, t)}`;
}

function confidenceLabel(value: number | null, t: Translate) {
  return value === null
    ? t("catalogGovernance.inference.none")
    : t("catalogGovernance.inference.confidence", { confidence: value });
}

function mappingStatusTone(status: string): BadgeTone {
  if (status === "accepted") {
    return "success";
  }

  if (status === "rejected") {
    return "danger";
  }

  return "warning";
}
