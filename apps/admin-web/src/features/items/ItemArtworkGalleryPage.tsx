import { Link } from "@tanstack/react-router";
import { ArrowLeft, CheckCircle2, RefreshCw, ShieldCheck, Trash2, X } from "lucide-react";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  ItemArtworkArtifactSummary,
  ItemArtworkGalleryQuery,
  ItemArtworkGallerySummary,
  ItemArtworkMutationResultSummary,
  ItemArtworkSelectedSummary,
} from "../../adminApi/types";
import { mockItemArtworkGallerySummary } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type ItemArtworkGallerySearch = {
  limit: number;
  offset: number;
};

export type ItemArtworkGalleryPageProps = {
  dataSource: AdminDataSource;
  itemId: string;
  search: ItemArtworkGallerySearch;
  onSearchChange(next: Partial<ItemArtworkGallerySearch>): void;
};

type ItemArtworkGalleryResult = {
  value: ItemArtworkGallerySummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

type ArtworkActionDraft =
  | {
      action: "select";
      artifactId: string;
      kind: string;
    }
  | {
      action: "unpublish";
      kind: string;
      selectedArtworkId: string;
    };

export function ItemArtworkGalleryPage({
  dataSource,
  itemId,
  search,
  onSearchChange,
}: ItemArtworkGalleryPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-item-artwork-gallery", itemId, search, locale],
    queryFn: () =>
      loadItemArtworkGallery(dataSource, itemId, search, t("itemArtwork.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockItemArtworkGallerySummary(itemId),
    source: "mock" as const,
  };
  const gallery = result.value;
  const hasPaginationDelta = search.limit !== 20 || search.offset !== 0;
  const [actionDraft, setActionDraft] = useState<ArtworkActionDraft | null>(null);
  const [mutationResult, setMutationResult] = useState<ItemArtworkMutationResultSummary | null>(null);
  const [mutationError, setMutationError] = useState<string | null>(null);
  const artworkMutation = useMutation({
    mutationFn: async (draft: ArtworkActionDraft) => {
      if (draft.action === "select") {
        if (!dataSource.selectItemArtwork) {
          throw new Error(t("itemArtwork.selectUnavailable"));
        }

        return dataSource.selectItemArtwork(itemId, draft.kind, draft.artifactId);
      }

      if (!dataSource.unpublishItemArtwork) {
        throw new Error(t("itemArtwork.unpublishUnavailable"));
      }

      return dataSource.unpublishItemArtwork(itemId, draft.kind);
    },
    onError(error: unknown) {
      setMutationResult(null);
      setMutationError(error instanceof Error ? error.message : t("itemArtwork.actionFailed"));
    },
    onSuccess(value) {
      setActionDraft(null);
      setMutationError(null);
      setMutationResult(value);
      void query.refetch();
    },
  });

  useEffect(() => {
    setActionDraft(null);
    setMutationResult(null);
    setMutationError(null);
    artworkMutation.reset();
  }, [itemId, search.limit, search.offset]);

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
            {t("itemArtwork.backToItem")}
          </Link>
          <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("itemArtwork.refresh")}
          </Button>
        </div>
      }
      description={t("itemArtwork.description")}
      kicker={t("itemArtwork.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("itemArtwork.title")}
      titleId="item-artwork-gallery-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("itemArtwork.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      <FilterBar label={t("itemArtwork.pagination")}>
        <FilterField label={t("itemArtwork.limit")}>
          <input
            aria-label={t("itemArtwork.limitAria")}
            min={1}
            type="number"
            value={search.limit}
            onChange={(event) => onSearchChange({ limit: numberInput(event.target.value) ?? 20, offset: 0 })}
          />
        </FilterField>
        <FilterField label={t("itemArtwork.offset")}>
          <input
            aria-label={t("itemArtwork.offsetAria")}
            min={0}
            type="number"
            value={search.offset}
            onChange={(event) => onSearchChange({ offset: nonNegativeNumberInput(event.target.value) ?? 0 })}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={gallery.totals.selectedCount > 0 ? "success" : "neutral"}>
            {t("itemArtwork.selectedCount", { count: gallery.totals.selectedCount })}
          </Badge>
          <Badge tone="info">{t("itemArtwork.guardedActions")}</Badge>
          <Button
            disabled={!hasPaginationDelta}
            onClick={() => onSearchChange({ limit: 20, offset: 0 })}
            variant="ghost"
          >
            <X size={16} />
            {t("itemArtwork.reset")}
          </Button>
        </FilterActions>
      </FilterBar>

      {query.isLoading ? <RowsSkeleton label={t("itemArtwork.loading")} /> : null}

      {!query.isLoading ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("itemArtwork.summary.description", {
              returned: gallery.page.returned,
              offset: gallery.page.offset,
              limit: gallery.page.limit,
            })}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("itemArtwork.redactedSummary")}
              </div>
            }
            title={t("itemArtwork.summary.title")}
          >
            <div className="artworkSummaryGrid">
              <SummaryTile label={t("itemArtwork.summary.candidates")} value={gallery.totals.candidateCount} />
              <SummaryTile label={t("itemArtwork.summary.artifacts")} value={gallery.totals.artifactCount} />
              <SummaryTile label={t("itemArtwork.summary.selectedArtwork")} value={gallery.totals.selectedCount} />
              <SummaryTile label={t("itemArtwork.summary.mediaItem")} value={gallery.itemId} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("itemArtwork.selected.description")}
            title={t("itemArtwork.selected.title")}
          >
            {gallery.selected.length === 0 ? (
              <EmptyRouteState>{t("itemArtwork.selected.empty")}</EmptyRouteState>
            ) : (
              <div className="artworkGalleryGrid">
                {gallery.selected.map((selection) => {
                  const routePath = safeArtworkRoutePath(selection.routePath);

                  return (
                    <div className="artworkTile" key={selection.selectedArtworkId}>
                      <div className="artworkPreview">
                        {routePath ? (
                          <img
                            alt={`${selection.kind} ${selection.imageId}`}
                            loading="lazy"
                            src={routePath}
                          />
                        ) : (
                          <span>{t("itemArtwork.noImageRoute")}</span>
                        )}
                      </div>
                      <div className="artworkTileBody">
                        <strong>{selection.kind}</strong>
                        <span>{selection.selectedArtworkId}</span>
                        <span>{selection.artifactId}</span>
                        <span>{routePath ?? t("itemArtwork.routeUnavailable")}</span>
                        <div className="issueBadgeList">
                          <Badge tone="success">{t("itemArtwork.badge.selected")}</Badge>
                          <Badge tone="neutral">{dimensionLabel(selection.width, selection.height, t)}</Badge>
                        </div>
                        <ArtworkActionControls
                          actionDraft={actionDraft}
                          isPending={artworkMutation.isPending}
                          onCancel={() => setActionDraft(null)}
                          onConfirm={(draft) => artworkMutation.mutate(draft)}
                          onPrepare={(draft) => {
                            setMutationError(null);
                            setActionDraft(draft);
                          }}
                          selection={selection}
                          t={t}
                        />
                      </div>
                    </div>
                  );
                })}
              </div>
            )}
          </DataPanel>

          <DataPanel
            description={t("itemArtwork.candidates.description")}
            title={t("itemArtwork.candidates.title")}
          >
            {gallery.candidates.length === 0 ? (
              <EmptyRouteState>{t("itemArtwork.candidates.empty")}</EmptyRouteState>
            ) : (
              <div className="librarySourceSamples">
                {gallery.candidates.map((candidate) => (
                  <div className="librarySourceSample" key={candidate.id}>
                    <div>
                      <strong>{candidate.id}</strong>
                      <span>
                        {candidate.kind} / {candidate.sourceKind} / {candidate.status}
                      </span>
                      <span>
                        {candidate.artifactId ?? t("itemArtwork.noArtifact")} / {dimensionLabel(candidate.width, candidate.height, t)}
                      </span>
                    </div>
                    <div className="issueBadgeList">
                      <Badge tone={candidateTone(candidate.status, candidate.selected)}>
                        {candidate.selected ? t("itemArtwork.badge.selected") : candidate.status}
                      </Badge>
                      <Badge tone={candidate.hasStoredArtifact ? "success" : "warning"}>
                        {candidate.hasStoredArtifact ? t("itemArtwork.badge.stored") : t("itemArtwork.badge.notStored")}
                      </Badge>
                      <Badge tone={candidate.hasIngestFailure ? "danger" : "neutral"}>
                        {candidate.ingestStatus ?? t("itemArtwork.badge.noIngest")}
                      </Badge>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </DataPanel>

          <DataPanel
            description={t("itemArtwork.artifacts.description")}
            title={t("itemArtwork.artifacts.title")}
          >
            {gallery.artifacts.length === 0 ? (
              <EmptyRouteState>{t("itemArtwork.artifacts.empty")}</EmptyRouteState>
            ) : (
              <div className="librarySourceSamples">
                {gallery.artifacts.map((artifact) => (
                  <div className="librarySourceSample" key={artifact.id}>
                    <div>
                      <strong>{artifact.id}</strong>
                      <span>
                        {artifact.kind} / {t("itemArtwork.artifacts.candidate", { candidateId: artifact.candidateId })}
                      </span>
                      <span>
                        {dimensionLabel(artifact.width, artifact.height, t)} / {formatBytes(artifact.byteLen, t)} /{" "}
                        {artifact.mediaType ?? t("itemArtwork.unknownType")}
                      </span>
                    </div>
                    <div className="issueBadgeList">
                      <Badge tone={artifact.selected ? "success" : "neutral"}>
                        {artifact.selected ? t("itemArtwork.badge.selected") : t("itemArtwork.badge.available")}
                      </Badge>
                      <Badge tone={artifact.hasContentHash ? "info" : "warning"}>
                        {artifact.hasContentHash ? t("itemArtwork.badge.hashPresent") : t("itemArtwork.badge.hashAbsent")}
                      </Badge>
                    </div>
                    <ArtworkActionControls
                      actionDraft={actionDraft}
                      artifact={artifact}
                      isPending={artworkMutation.isPending}
                      onCancel={() => setActionDraft(null)}
                      onConfirm={(draft) => artworkMutation.mutate(draft)}
                      onPrepare={(draft) => {
                        setMutationError(null);
                        setActionDraft(draft);
                      }}
                      t={t}
                    />
                  </div>
                ))}
              </div>
            )}
          </DataPanel>

          <DataPanel
            description={t("itemArtwork.result.description")}
            title={t("itemArtwork.result.title")}
          >
            <div className="librarySourceSamples">
              {mutationError ? <RouteNotice>{mutationError}</RouteNotice> : null}
              {mutationResult ? <ArtworkMutationResult result={mutationResult} t={t} /> : null}
              {!mutationError && !mutationResult ? (
                <EmptyRouteState>{t("itemArtwork.result.empty")}</EmptyRouteState>
              ) : null}
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

async function loadItemArtworkGallery(
  dataSource: AdminDataSource,
  itemId: string,
  search: ItemArtworkGallerySearch,
  unavailableMessage: string,
): Promise<ItemArtworkGalleryResult> {
  if (!dataSource.loadItemArtworkGallery) {
    return {
      value: mockItemArtworkGallerySummary(itemId),
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadItemArtworkGallery(itemId, toItemArtworkGalleryQuery(search));
}

function toItemArtworkGalleryQuery(search: ItemArtworkGallerySearch): ItemArtworkGalleryQuery {
  return {
    limit: search.limit,
    offset: search.offset,
  };
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function SummaryTile({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="artworkSummaryTile">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}

function ArtworkActionControls({
  actionDraft,
  artifact,
  isPending,
  onCancel,
  onConfirm,
  onPrepare,
  selection,
  t,
}: {
  actionDraft: ArtworkActionDraft | null;
  artifact?: ItemArtworkArtifactSummary;
  isPending: boolean;
  onCancel: () => void;
  onConfirm: (draft: ArtworkActionDraft) => void;
  onPrepare: (draft: ArtworkActionDraft) => void;
  selection?: ItemArtworkSelectedSummary;
  t: Translate;
}) {
  if (artifact) {
    const draft: ArtworkActionDraft = {
      action: "select",
      artifactId: artifact.id,
      kind: String(artifact.kind),
    };
    const isConfirming =
      actionDraft?.action === "select" && actionDraft.artifactId === artifact.id;

    return (
      <div className="artworkActionBlock">
        {isConfirming ? (
          <>
            <div>
              <strong>{t("itemArtwork.action.selectConfirmation")}</strong>
              <span>
                {t("itemArtwork.action.selectCopy", { kind: artifact.kind, artifactId: artifact.id })}
              </span>
            </div>
            <div className="routeActionGroup">
              <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
                {t("itemArtwork.action.cancel")}
              </Button>
              <Button
                aria-label={t("itemArtwork.action.confirmSelectAria", { artifactId: artifact.id })}
                disabled={isPending}
                onClick={() => onConfirm(draft)}
                size="sm"
              >
                <CheckCircle2 size={15} />
                {t("itemArtwork.action.confirmSelect")}
              </Button>
            </div>
          </>
        ) : (
          <Button
            aria-label={t("itemArtwork.action.prepareSelectAria", { artifactId: artifact.id })}
            disabled={isPending}
            onClick={() => onPrepare(draft)}
            size="sm"
            variant="outline"
          >
            <CheckCircle2 size={15} />
            {t("itemArtwork.action.select")}
          </Button>
        )}
      </div>
    );
  }

  if (selection) {
    const draft: ArtworkActionDraft = {
      action: "unpublish",
      kind: String(selection.kind),
      selectedArtworkId: selection.selectedArtworkId,
    };
    const isConfirming =
      actionDraft?.action === "unpublish" &&
      actionDraft.selectedArtworkId === selection.selectedArtworkId;

    return (
      <div className="artworkActionBlock">
        {isConfirming ? (
          <>
            <div>
              <strong>{t("itemArtwork.action.unpublishConfirmation")}</strong>
              <span>
                {t("itemArtwork.action.unpublishCopy", {
                  kind: selection.kind,
                  selectedArtworkId: selection.selectedArtworkId,
                })}
              </span>
            </div>
            <div className="routeActionGroup">
              <Button disabled={isPending} onClick={onCancel} size="sm" variant="ghost">
                {t("itemArtwork.action.cancel")}
              </Button>
              <Button
                aria-label={t("itemArtwork.action.confirmUnpublishAria", { kind: selection.kind })}
                disabled={isPending}
                onClick={() => onConfirm(draft)}
                size="sm"
              >
                <Trash2 size={15} />
                {t("itemArtwork.action.confirmUnpublish")}
              </Button>
            </div>
          </>
        ) : (
          <Button
            aria-label={t("itemArtwork.action.prepareUnpublishAria", { kind: selection.kind })}
            disabled={isPending}
            onClick={() => onPrepare(draft)}
            size="sm"
            variant="outline"
          >
            <Trash2 size={15} />
            {t("itemArtwork.action.unpublish")}
          </Button>
        )}
      </div>
    );
  }

  return null;
}

function ArtworkMutationResult({ result, t }: { result: ItemArtworkMutationResultSummary; t: Translate }) {
  const routePath = safeArtworkRoutePath(result.routePath);

  return (
    <>
      <div className="librarySourceSample">
        <div>
          <strong>
            {result.action === "select"
              ? t("itemArtwork.result.selectionUpdated")
              : t("itemArtwork.result.selectionUnpublished")}
          </strong>
          <span>
            {result.kind} / {result.itemId}
          </span>
        </div>
        <Badge tone={result.changed ? "success" : "warning"}>
          {result.changed ? t("itemArtwork.result.changed") : t("itemArtwork.result.idempotent")}
        </Badge>
      </div>
      <div className="librarySourceSample">
        <div>
          <strong>{result.selectedArtworkId ?? t("itemArtwork.noSelectedArtwork")}</strong>
          <span>{result.artifactId ?? t("itemArtwork.noArtifact")}</span>
          <span>{routePath ?? t("itemArtwork.routeUnavailable")}</span>
        </div>
        <Badge tone="info">{dimensionLabel(result.width, result.height, t)}</Badge>
      </div>
    </>
  );
}

function candidateTone(status: string, selected: boolean): BadgeTone {
  if (selected) {
    return "success";
  }

  if (status === "failed" || status === "rejected") {
    return "danger";
  }

  if (status === "pending") {
    return "warning";
  }

  return "neutral";
}

function dimensionLabel(width: number | null, height: number | null, t: Translate) {
  return width === null || height === null ? t("itemArtwork.dimensionsUnavailable") : `${width}x${height}`;
}

function safeArtworkRoutePath(value: string | null) {
  return value?.startsWith("/images/") ? value : null;
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("itemArtwork.sizeUnavailable");
  }

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

function numberInput(value: string) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    return undefined;
  }

  return parsed;
}

function nonNegativeNumberInput(value: string) {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed < 0) {
    return undefined;
  }

  return parsed;
}
