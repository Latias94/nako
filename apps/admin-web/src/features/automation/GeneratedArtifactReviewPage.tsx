import { Link } from "@tanstack/react-router";
import { ArrowLeft, CheckCircle2, RefreshCw, ShieldCheck, XCircle } from "lucide-react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState, type ReactNode } from "react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  GeneratedArtifactReviewDecision,
  GeneratedArtifactReviewPlanSummary,
  GeneratedArtifactReviewResultSummary,
} from "../../adminApi/types";
import { mockGeneratedArtifactReviewPlan } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type GeneratedArtifactReviewSearch = {
  decision: GeneratedArtifactReviewDecision;
};

export type GeneratedArtifactReviewPageProps = {
  artifactId: string;
  dataSource: AdminDataSource;
  search: GeneratedArtifactReviewSearch;
  onSearchChange(next: Partial<GeneratedArtifactReviewSearch>): void;
};

type ReviewPlanResult = {
  value: GeneratedArtifactReviewPlanSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function GeneratedArtifactReviewPage({
  artifactId,
  dataSource,
  search,
  onSearchChange,
}: GeneratedArtifactReviewPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-generated-artifact-review-plan", artifactId, search.decision, locale],
    queryFn: () =>
      loadReviewPlan(
        dataSource,
        artifactId,
        search.decision,
        t("generatedArtifactReview.planUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: mockReviewPlanSummary(artifactId, search.decision),
    source: "mock" as const,
  };
  const plan = result.value;
  const [isConfirming, setIsConfirming] = useState(false);
  const [reviewResult, setReviewResult] = useState<GeneratedArtifactReviewResultSummary | null>(null);
  const [reviewError, setReviewError] = useState<string | null>(null);
  const reviewMutation = useMutation({
    mutationFn: async () => {
      if (!dataSource.reviewGeneratedArtifact) {
        throw new Error(t("generatedArtifactReview.reviewUnavailable"));
      }

      return dataSource.reviewGeneratedArtifact(artifactId, search.decision);
    },
    onError(error: unknown) {
      setReviewResult(null);
      setReviewError(error instanceof Error ? error.message : t("generatedArtifactReview.reviewFailed"));
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
  }, [artifactId, search.decision]);

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            className="routeTextLink routeBackLink"
            search={{ limit: 20, offset: 0 }}
            to="/automation/generated-artifacts"
          >
            <ArrowLeft size={16} />
            {t("generatedArtifactReview.backToQueue")}
          </Link>
          <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("generatedArtifactReview.refresh")}
          </Button>
        </div>
      }
      description={t("generatedArtifactReview.description")}
      kicker={t("generatedArtifactReview.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("generatedArtifactReview.title")}
      titleId="generated-artifact-review-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("generatedArtifactReview.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("generatedArtifactReview.loading")} /> : null}

      {!query.isLoading && !plan.artifactId ? (
        <EmptyRouteState>{t("generatedArtifactReview.missing", { artifactId })}</EmptyRouteState>
      ) : null}

      {!query.isLoading && plan.artifactId ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("generatedArtifactReview.reviewPlan.description")}
            headerAccessory={
              <div className="routeActionGroup" role="group" aria-label={t("generatedArtifactReview.decisionGroup")}>
                <Button
                  aria-pressed={search.decision === "accept"}
                  onClick={() => onSearchChange({ decision: "accept" })}
                  disabled={reviewMutation.isPending}
                  size="sm"
                  variant={search.decision === "accept" ? "default" : "outline"}
                >
                  <CheckCircle2 size={15} />
                  {t("generatedArtifactReview.accept")}
                </Button>
                <Button
                  aria-pressed={search.decision === "reject"}
                  onClick={() => onSearchChange({ decision: "reject" })}
                  disabled={reviewMutation.isPending}
                  size="sm"
                  variant={search.decision === "reject" ? "default" : "outline"}
                >
                  <XCircle size={15} />
                  {t("generatedArtifactReview.reject")}
                </Button>
              </div>
            }
            title={t("generatedArtifactReview.reviewPlan.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("generatedArtifactReview.reviewPlan.artifactId")} value={plan.artifactId} />
              <Fact label={t("generatedArtifactReview.reviewPlan.decision")} value={plan.decision} />
              <Fact label={t("generatedArtifactReview.reviewPlan.status")} value={plan.status} />
              <Fact label={t("generatedArtifactReview.reviewPlan.action")} value={plan.action} />
              <Fact label={t("generatedArtifactReview.reviewPlan.reasons")} value={listLabel(plan.reasons, t)} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("generatedArtifactReview.summary.description")}
            headerAccessory={<Badge tone={statusTone(plan.status)}>{plan.status}</Badge>}
            title={t("generatedArtifactReview.summary.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("generatedArtifactReview.summary.capability")} value={plan.capability} />
              <Fact label={t("generatedArtifactReview.summary.kind")} value={plan.kind} />
              <Fact label={t("generatedArtifactReview.summary.target")} value={targetLabel(plan)} />
              <Fact label={t("generatedArtifactReview.summary.payload")} value={`${plan.payload.shape} / ${formatBytes(plan.payload.payloadBytes)}`} />
              <Fact label={t("generatedArtifactReview.summary.confidence")} value={confidenceLabel(plan.payload.confidenceMilli, t)} />
              <Fact label={t("generatedArtifactReview.summary.fingerprint")} value={shortFingerprint(plan.payload.payloadFingerprint, t)} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("generatedArtifactReview.boundaries.description")}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("generatedArtifactReview.boundaries.redacted")}
              </div>
            }
            title={t("generatedArtifactReview.boundaries.title")}
          >
            <div className="librarySourceSamples">
              <BoundaryRow
                enabled={plan.boundary.acceptedIntoCanonicalMetadata}
                label={t("generatedArtifactReview.boundaries.acceptedIntoCanonicalMetadata")}
                t={t}
              />
              <BoundaryRow
                enabled={plan.boundary.requiresMetadataAuthorityApply}
                label={t("generatedArtifactReview.boundaries.requiresMetadataAuthorityApply")}
                t={t}
              />
              <BoundaryRow enabled={plan.boundary.writesSidecar} label={t("generatedArtifactReview.boundaries.writesSidecar")} t={t} />
              <BoundaryRow enabled={plan.boundary.writesLibraryFiles} label={t("generatedArtifactReview.boundaries.writesLibraryFiles")} t={t} />
              <BoundaryRow enabled={plan.boundary.appliesImmediately} label={t("generatedArtifactReview.boundaries.appliesImmediately")} t={t} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("generatedArtifactReview.readiness.description")}
            title={t("generatedArtifactReview.readiness.title")}
          >
            <div className="librarySourceSamples">
              <div className="librarySourceSample">
                <div>
                  <strong>{plan.readiness.status}</strong>
                  <span>{listLabel(plan.readiness.reasons, t)}</span>
                </div>
                <Badge tone={plan.readiness.actionable ? "success" : "warning"}>
                  {plan.readiness.actionable
                    ? t("generatedArtifactReview.readiness.actionable")
                    : t("generatedArtifactReview.readiness.blocked")}
                </Badge>
              </div>
            </div>
          </DataPanel>

          <DataPanel
            description={t("generatedArtifactReview.confirmed.description")}
            title={t("generatedArtifactReview.confirmed.title")}
          >
            <div className="librarySourceSamples">
              <div className="librarySourceSample">
                <div>
                  <strong>{t("generatedArtifactReview.selectedDecision")}</strong>
                  <span>
                    {t("generatedArtifactReview.result.selectedDecision", {
                      decision: search.decision,
                      artifactId: plan.artifactId,
                    })}
                  </span>
                </div>
                <Badge tone={search.decision === "accept" ? "success" : "danger"}>
                  {search.decision}
                </Badge>
              </div>

              {reviewError ? <RouteNotice>{reviewError}</RouteNotice> : null}

              {!reviewResult && !isConfirming ? (
                <div className="librarySourceSample">
                  <div>
                    <strong>{t("generatedArtifactReview.prepareConfirmation")}</strong>
                    <span>{t("generatedArtifactReview.prepareCopy")}</span>
                  </div>
                  <Button
                    disabled={query.isLoading || reviewMutation.isPending}
                    onClick={() => {
                      setReviewError(null);
                      setIsConfirming(true);
                    }}
                    variant="outline"
                  >
                    {search.decision === "accept" ? <CheckCircle2 size={15} /> : <XCircle size={15} />}
                    {t("generatedArtifactReview.prepareButton", { decision: search.decision })}
                  </Button>
                </div>
              ) : null}

              {!reviewResult && isConfirming ? (
                <div className="librarySourceSample">
                  <div>
                    <strong>{t("generatedArtifactReview.confirmLabel", { decision: search.decision })}</strong>
                    <span>{t("generatedArtifactReview.confirmCopy", { artifactId: plan.artifactId })}</span>
                  </div>
                  <div className="routeActionGroup">
                    <Button
                      disabled={reviewMutation.isPending}
                      onClick={() => setIsConfirming(false)}
                      variant="ghost"
                    >
                      {t("generatedArtifactReview.cancel")}
                    </Button>
                    <Button
                      disabled={reviewMutation.isPending}
                      onClick={() => reviewMutation.mutate()}
                    >
                      {search.decision === "accept" ? <CheckCircle2 size={15} /> : <XCircle size={15} />}
                      {t("generatedArtifactReview.confirmButton", { decision: search.decision })}
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

async function loadReviewPlan(
  dataSource: AdminDataSource,
  artifactId: string,
  decision: GeneratedArtifactReviewDecision,
  unavailableMessage: string,
): Promise<ReviewPlanResult> {
  if (!dataSource.loadGeneratedArtifactReviewPlan) {
    return {
      value: mockReviewPlanSummary(artifactId, decision),
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadGeneratedArtifactReviewPlan(artifactId, decision);
}

function mockReviewPlanSummary(
  artifactId: string,
  decision: GeneratedArtifactReviewDecision,
): GeneratedArtifactReviewPlanSummary {
  const { plan } = mockGeneratedArtifactReviewPlan(artifactId, decision);

  return {
    artifactId: plan.artifact_id,
    decision: plan.decision,
    status: plan.status,
    action: plan.action,
    reasons: plan.reasons,
    capability: plan.capability,
    kind: plan.kind,
    target: {
      kind: plan.target.kind,
      libraryId: plan.target.library_id,
      itemId: plan.target.item_id,
      sourceId: plan.target.source_id,
    },
    payload: {
      validJson: plan.payload.valid_json,
      shape: plan.payload.shape,
      payloadFingerprint: plan.payload.payload_fingerprint,
      payloadBytes: plan.payload.payload_bytes,
      objectFieldCount: plan.payload.object_field_count,
      arrayItemCount: plan.payload.array_item_count,
      hasTextualValues: plan.payload.has_textual_values,
      hasExplanation: plan.payload.has_explanation,
      confidenceMilli: plan.payload.confidence_milli,
    },
    readiness: {
      status: plan.readiness.status,
      actionable: plan.readiness.actionable,
      reasons: plan.readiness.reasons,
    },
    boundary: {
      acceptedIntoCanonicalMetadata: plan.boundary.accepted_into_canonical_metadata,
      writesSidecar: plan.boundary.writes_sidecar,
      writesLibraryFiles: plan.boundary.writes_library_files,
      appliesImmediately: plan.boundary.applies_immediately,
      requiresMetadataAuthorityApply: plan.boundary.requires_metadata_authority_apply,
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
            ? t("generatedArtifactReview.boundaries.included")
            : t("generatedArtifactReview.boundaries.excluded")}
        </span>
      </div>
      <Badge tone={enabled ? "warning" : "neutral"}>
        {enabled
          ? t("generatedArtifactReview.boundaries.yes")
          : t("generatedArtifactReview.boundaries.no")}
      </Badge>
    </div>
  );
}

function ReviewResult({ result, t }: { result: GeneratedArtifactReviewResultSummary; t: Translate }) {
  return (
    <>
      <div className="librarySourceSample">
        <div>
          <strong>{t("generatedArtifactReview.result.reviewResult")}</strong>
          <span>{result.artifactId}</span>
        </div>
        <Badge tone={result.artifactStatus === "accepted" ? "success" : "danger"}>
          {result.artifactStatus}
        </Badge>
      </div>
      <div className="librarySourceSample">
        <div>
          <strong>{t("generatedArtifactReview.result.decision")}</strong>
          <span>{result.decision}</span>
        </div>
        <Badge tone={result.idempotentReplay ? "warning" : "neutral"}>
          {result.idempotentReplay
            ? t("generatedArtifactReview.result.idempotentReplay")
            : t("generatedArtifactReview.result.newResult")}
        </Badge>
      </div>
      <div className="librarySourceSample">
        <div>
          <strong>{t("generatedArtifactReview.result.acceptedAt")}</strong>
          <span>{result.acceptedAt ?? t("generatedArtifactReview.result.notAccepted")}</span>
        </div>
        <Badge tone="info">{result.plan.action}</Badge>
      </div>
    </>
  );
}

function targetLabel(plan: GeneratedArtifactReviewPlanSummary) {
  return [plan.target.kind, plan.target.libraryId, plan.target.itemId, plan.target.sourceId]
    .filter(Boolean)
    .join(" / ");
}

function confidenceLabel(value: number | null, t: Translate) {
  return value === null ? t("generatedArtifactReview.unknown") : `${value} / 1000`;
}

function listLabel(values: string[], t: Translate) {
  return values.length > 0 ? values.join(", ") : t("generatedArtifactReview.none");
}

function statusTone(status: string): BadgeTone {
  if (status === "ready") {
    return "success";
  }

  if (status === "blocked") {
    return "warning";
  }

  return "neutral";
}

function shortFingerprint(value: string | null, t: Translate) {
  if (!value) {
    return t("generatedArtifactReview.noFingerprint");
  }

  return value.length > 24 ? `${value.slice(0, 21)}...` : value;
}

function formatBytes(value: number) {
  if (value < 1024) {
    return `${value} B`;
  }

  const units = ["KiB", "MiB", "GiB", "TiB"];
  let amount = value / 1024;
  let unitIndex = 0;

  while (amount >= 1024 && unitIndex < units.length - 1) {
    amount /= 1024;
    unitIndex += 1;
  }

  return `${amount.toFixed(amount >= 10 ? 1 : 2)} ${units[unitIndex]}`;
}
