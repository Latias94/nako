import { Link } from "@tanstack/react-router";
import { ArrowLeft, ExternalLink, RefreshCw, ShieldCheck } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import type { ReactNode } from "react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type { ItemDetailSummary, ItemReadinessSummary, ItemSourceSummary } from "../../adminApi/types";
import { mockItemDetailSummary } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type ItemDetailPageProps = {
  dataSource: AdminDataSource;
  itemId: string;
};

type ItemDetailResult = {
  value: ItemDetailSummary;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

export function ItemDetailPage({ dataSource, itemId }: ItemDetailPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-item-detail", itemId, locale],
    queryFn: () => loadItemDetail(dataSource, itemId, t("itemDetail.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockItemDetailSummary(itemId),
    source: "mock" as const,
  };
  const detail = result.value;
  const firstSourceId = detail.sources[0]?.id;

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup">
          <Link
            className="routeTextLink routeBackLink"
            search={{ q: undefined, facet: undefined, limit: 20, offset: 0 }}
            to="/catalog"
          >
            <ArrowLeft size={16} />
            {t("itemDetail.backToCatalog")}
          </Link>
          <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("itemDetail.refresh")}
          </Button>
        </div>
      }
      description={t("itemDetail.description")}
      kicker={t("itemDetail.kicker")}
      status={<SourceLabel source={result.source} />}
      title={detail.item.title}
      titleId="item-detail-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("itemDetail.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("itemDetail.loading")} /> : null}

      {!query.isLoading && !detail.item.id ? (
        <EmptyRouteState>{t("itemDetail.missing", { itemId })}</EmptyRouteState>
      ) : null}

      {!query.isLoading && detail.item.id ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("itemDetail.facts.description")}
            headerAccessory={<Badge tone={kindTone(detail.item.kind)}>{detail.item.kind}</Badge>}
            title={t("itemDetail.facts.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("itemDetail.facts.mediaItemId")} value={detail.item.id} />
              <Fact label={t("itemDetail.facts.parent")} value={detail.item.parentId ?? t("itemDetail.parent.none")} />
              <Fact label={t("itemDetail.facts.release")} value={detail.item.releaseDate ?? t("itemDetail.release.none")} />
              <Fact label={t("itemDetail.facts.runtime")} value={formatRuntime(detail.item.runtimeMinutes, t)} />
              <Fact label={t("itemDetail.facts.sources")} value={t("itemDetail.sources.count", { count: detail.item.sourceCount })} />
              <Fact label={t("itemDetail.facts.images")} value={t("itemDetail.images.count", { count: detail.item.imageCount })} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("itemDetail.canonical.description")}
            headerAccessory={
              <div className="searchHint">
                <ShieldCheck size={15} />
                {t("itemDetail.redactedSummary")}
              </div>
            }
            title={t("itemDetail.canonical.title")}
          >
            <div className="librarySourceSamples">
              <TokenRow label={t("itemDetail.canonical.genres")} t={t} values={detail.canonical.genres} />
              <TokenRow label={t("itemDetail.canonical.tags")} t={t} values={detail.canonical.tags} />
              <TokenRow
                label={t("itemDetail.canonical.credits")}
                t={t}
                values={detail.canonical.credits.map((credit) =>
                  credit.character
                    ? t("itemDetail.canonical.creditAs", { name: credit.name, character: credit.character })
                    : t("itemDetail.canonical.creditRole", { name: credit.name, role: credit.role }),
                )}
              />
              <TokenRow label={t("itemDetail.canonical.collections")} t={t} values={detail.canonical.collections} />
              <TokenRow label={t("itemDetail.canonical.studios")} t={t} values={detail.canonical.studios} />
              <div className="librarySourceSample">
                <div>
                  <strong>{t("itemDetail.canonical.externalEvidence")}</strong>
                  <span>
                    {t("itemDetail.canonical.externalEvidenceValue", {
                      ratings: detail.canonical.ratingCount,
                      externalIds: detail.canonical.externalIdCount,
                    })}
                  </span>
                </div>
                <Badge tone="neutral">{t("itemDetail.canonical.countsOnly")}</Badge>
              </div>
            </div>
          </DataPanel>

          <DataPanel
            description={t("itemDetail.sources.description")}
            title={t("itemDetail.sources.title")}
          >
            <div className="librarySourceSamples">
              {detail.sources.map((source) => (
                <SourceRow key={source.id} source={source} t={t} />
              ))}
            </div>
          </DataPanel>

          <DataPanel
            description={t("itemDetail.artwork.description")}
            title={t("itemDetail.artwork.title")}
          >
            <div className="librarySourceSamples">
              {detail.images.map((image) => (
                <div className="librarySourceSample" key={image.id}>
                  <div>
                    <strong>{image.kind}</strong>
                    <span>
                      {image.routePath ?? t("itemDetail.artwork.routeUnavailable")} / {image.width ?? "?"}x{image.height ?? "?"}
                    </span>
                  </div>
                  <Badge tone={image.hasEtag ? "success" : "neutral"}>
                    {image.mediaType ?? t("itemDetail.unknownType")}
                  </Badge>
                </div>
              ))}
              {detail.readiness.map((item) => (
                <ReadinessRow item={item} key={item.label} />
              ))}
            </div>
          </DataPanel>

          <DataPanel
            description={t("itemDetail.support.description")}
            title={t("itemDetail.support.title")}
          >
            <div className="librarySourceSamples">
              <SupportLink
                label={t("itemDetail.support.catalogGovernance")}
                value={t("itemDetail.support.catalogGovernanceValue")}
              >
                <Link
                  className="routeTextLink"
                  search={{ library_id: undefined, max_confidence_milli: undefined, limit: 20, offset: 0 }}
                  to="/catalog/governance"
                >
                  {t("itemDetail.support.open")}
                  <ExternalLink size={14} />
                </Link>
              </SupportLink>
              <SupportLink
                label={t("itemDetail.support.artworkGallery")}
                value={t("itemDetail.support.artworkGalleryValue")}
              >
                <Link
                  aria-label={t("itemDetail.support.openArtworkGalleryAria")}
                  className="routeTextLink"
                  params={{ itemId: detail.item.id }}
                  search={{ limit: 20, offset: 0 }}
                  to="/items/$itemId/artwork"
                >
                  {t("itemDetail.support.open")}
                  <ExternalLink size={14} />
                </Link>
              </SupportLink>
              <SupportLink
                label={t("itemDetail.support.generatedArtifacts")}
                value={t("itemDetail.support.generatedArtifactsValue")}
              >
                <Link
                  className="routeTextLink"
                  search={{ limit: 20, offset: 0 }}
                  to="/automation/generated-artifacts"
                >
                  {t("itemDetail.support.open")}
                  <ExternalLink size={14} />
                </Link>
              </SupportLink>
              {firstSourceId ? (
                <SupportLink
                  label={t("itemDetail.support.playbackSessions")}
                  value={t("itemDetail.support.playbackSessionsValue")}
                >
                  <Link
                    className="routeTextLink"
                    search={{
                      source_id: firstSourceId,
                      kind: undefined,
                      state: undefined,
                      limit: 20,
                      offset: 0,
                    }}
                    to="/playback/sessions"
                  >
                    {t("itemDetail.support.open")}
                    <ExternalLink size={14} />
                  </Link>
                </SupportLink>
              ) : null}
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
  unavailableMessage: string,
): Promise<ItemDetailResult> {
  if (!dataSource.loadItemDetail) {
    return {
      value: mockItemDetailSummary(itemId),
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadItemDetail(itemId);
}

function Fact({ label, value }: { label: string; value: string }) {
  return (
    <div className="libraryFactRow">
      <div>
        <strong>{label}</strong>
        <span>{value}</span>
      </div>
    </div>
  );
}

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;

function TokenRow({ label, t, values }: { label: string; t: Translate; values: string[] }) {
  return (
    <div className="librarySourceSample">
      <div>
        <strong>{label}</strong>
        <span>{values.length > 0 ? values.join(" / ") : t("itemDetail.none")}</span>
      </div>
      <Badge tone={values.length > 0 ? "info" : "neutral"}>{values.length}</Badge>
    </div>
  );
}

function SourceRow({ source, t }: { source: ItemSourceSummary; t: Translate }) {
  return (
    <div className="librarySourceSample">
      <div>
        <strong>{source.fileName}</strong>
        <span>
          {source.libraryId} / {formatBytes(source.sizeBytes, t)}
        </span>
        <span>{source.probe ? probeLabel(source.probe, t) : t("itemDetail.sources.probeUnavailable")}</span>
      </div>
      <Badge tone={source.hasFingerprint ? "success" : "warning"}>
        {source.hasFingerprint ? t("itemDetail.sources.fingerprinted") : t("itemDetail.sources.noFingerprint")}
      </Badge>
    </div>
  );
}

function ReadinessRow({ item }: { item: ItemReadinessSummary }) {
  return (
    <div className="librarySourceSample">
      <div>
        <strong>{item.label}</strong>
        <span>{item.detail}</span>
      </div>
      <Badge tone={readinessTone(item.status)}>{item.status}</Badge>
    </div>
  );
}

function SupportLink({
  children,
  label,
  value,
}: {
  children: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="librarySourceSample">
      <div>
        <strong>{label}</strong>
        <span>{value}</span>
      </div>
      {children}
    </div>
  );
}

function kindTone(kind: string): BadgeTone {
  return kind === "unknown" ? "warning" : "neutral";
}

function readinessTone(status: ItemReadinessSummary["status"]): BadgeTone {
  if (status === "ready") {
    return "success";
  }

  if (status === "planned") {
    return "info";
  }

  return "warning";
}

function formatRuntime(value: number | null, t: Translate) {
  return value === null ? t("itemDetail.runtime.unknown") : t("itemDetail.runtime.minutes", { minutes: value });
}

function probeLabel(probe: NonNullable<ItemSourceSummary["probe"]>, t: Translate) {
  return t("itemDetail.sources.probe", {
    container: probe.container ?? t("itemDetail.sources.unknownContainer"),
    duration: formatDuration(probe.durationMs, t),
    streams: probe.streamCount,
  });
}

function formatDuration(value: number | null, t: Translate) {
  if (value === null) {
    return t("itemDetail.duration.unknown");
  }

  const totalMinutes = Math.round(value / 60000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  return hours > 0
    ? t("itemDetail.duration.hoursMinutes", { hours, minutes })
    : t("itemDetail.duration.minutes", { minutes });
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("itemDetail.sizeUnavailable");
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
