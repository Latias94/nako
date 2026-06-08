import { useQuery } from "@tanstack/react-query";
import { RefreshCw, Search, ShieldCheck, X } from "lucide-react";
import { useMemo } from "react";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  ManagedArtworkLifecycleArtifactRow,
  ManagedArtworkMaintenanceLifecycleQuery,
  ManagedArtworkMaintenanceRemediationPlanQuery,
  ManagedArtworkMaintenanceStorageDriftQuery,
  ManagedArtworkMaintenanceSummary,
  ManagedArtworkRemediationMissingArtifactRow,
  ManagedArtworkRemediationStrayFileRow,
  ManagedArtworkStorageDriftArtifactRow,
  ManagedArtworkStorageDriftFileRow,
} from "../../adminApi/types";
import { SourceLabel } from "../../components/SourceLabel";
import { EmptyRouteState, RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { FilterActions, FilterBar, FilterField } from "../../components/ui/FilterBar";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../../components/ui/Table";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type ManagedArtworkMaintenanceSearch = {
  limit: number;
  offset: number;
  cleanup_candidates_only: boolean;
  file_scan_limit: number;
};

export type ManagedArtworkMaintenancePageProps = {
  dataSource: AdminDataSource;
  search: ManagedArtworkMaintenanceSearch;
  onSearchChange(next: Partial<ManagedArtworkMaintenanceSearch>): void;
};

type MaintenanceResult = {
  value: ManagedArtworkMaintenanceSummary;
  source: DataSourceMode;
  error?: string;
};

type Translate = (id: MessageId, values?: Record<string, number | string>) => string;
type BadgeTone = "danger" | "info" | "neutral" | "success" | "warning";

export function ManagedArtworkMaintenancePage({
  dataSource,
  onSearchChange,
  search,
}: ManagedArtworkMaintenancePageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-managed-artwork-maintenance", search, locale],
    queryFn: () =>
      loadManagedArtworkMaintenance(
        dataSource,
        search,
        t("artworkMaintenance.dataSourceUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: emptyMaintenanceSummary(search),
    source: "mock" as const,
  };
  const summary = result.value;
  const activeFilterCount = useMemo(
    () =>
      [
        search.cleanup_candidates_only ? "cleanup" : null,
        search.limit !== 20 ? "limit" : null,
        search.offset !== 0 ? "offset" : null,
        search.file_scan_limit !== 500 ? "scan-limit" : null,
      ].filter(Boolean).length,
    [search],
  );

  return (
    <RoutePage
      actions={
        <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
          <RefreshCw size={16} />
          {t("artworkMaintenance.refresh")}
        </Button>
      }
      description={t("artworkMaintenance.description")}
      kicker={t("artworkMaintenance.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("artworkMaintenance.title")}
      titleId="managed-artwork-maintenance-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("artworkMaintenance.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      <FilterBar label={t("artworkMaintenance.filters")}>
        <FilterField label={t("artworkMaintenance.filter.limit")}>
          <input
            aria-label={t("artworkMaintenance.filter.limitAria")}
            min={1}
            onChange={(event) =>
              onSearchChange({
                limit: positiveNumberInput(event.target.value, search.limit, 1),
                offset: 0,
              })
            }
            type="number"
            value={search.limit}
          />
        </FilterField>
        <FilterField label={t("artworkMaintenance.filter.offset")}>
          <input
            aria-label={t("artworkMaintenance.filter.offsetAria")}
            min={0}
            onChange={(event) =>
              onSearchChange({
                offset: positiveNumberInput(event.target.value, search.offset, 0),
              })
            }
            type="number"
            value={search.offset}
          />
        </FilterField>
        <FilterField label={t("artworkMaintenance.filter.cleanupOnly")}>
          <label className="artworkMaintenanceToggle">
            <input
              aria-label={t("artworkMaintenance.filter.cleanupOnlyAria")}
              checked={search.cleanup_candidates_only}
              onChange={(event) =>
                onSearchChange({
                  cleanup_candidates_only: event.target.checked,
                  offset: 0,
                })
              }
              type="checkbox"
            />
            <span>{t("artworkMaintenance.filter.cleanupOnlyValue")}</span>
          </label>
        </FilterField>
        <FilterField label={t("artworkMaintenance.filter.fileScanLimit")}>
          <input
            aria-label={t("artworkMaintenance.filter.fileScanLimitAria")}
            min={1}
            onChange={(event) =>
              onSearchChange({
                file_scan_limit: positiveNumberInput(event.target.value, search.file_scan_limit, 1),
                offset: 0,
              })
            }
            type="number"
            value={search.file_scan_limit}
          />
        </FilterField>
        <FilterActions>
          <Badge tone={activeFilterCount > 0 ? "info" : "neutral"}>
            {t("artworkMaintenance.filter.active", { count: activeFilterCount })}
          </Badge>
          <Button
            disabled={activeFilterCount === 0}
            onClick={() =>
              onSearchChange({
                cleanup_candidates_only: false,
                file_scan_limit: 500,
                limit: 20,
                offset: 0,
              })
            }
            variant="ghost"
          >
            <X size={16} />
            {t("artworkMaintenance.clear")}
          </Button>
        </FilterActions>
      </FilterBar>

      {query.isLoading ? <RowsSkeleton label={t("artworkMaintenance.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <div className="artworkSummaryGrid">
            <SummaryTile
              label={t("artworkMaintenance.summary.totalArtifacts")}
              tone="info"
              value={summary.lifecycle.totals.totalArtifacts}
            />
            <SummaryTile
              label={t("artworkMaintenance.summary.cleanupCandidates")}
              tone={summary.lifecycle.totals.cleanupCandidateArtifacts > 0 ? "warning" : "neutral"}
              value={summary.lifecycle.totals.cleanupCandidateArtifacts}
            />
            <SummaryTile
              label={t("artworkMaintenance.summary.missingArtifacts")}
              tone={summary.storageDrift.totals.dbBackedMissingArtifacts > 0 ? "danger" : "success"}
              value={summary.storageDrift.totals.dbBackedMissingArtifacts}
            />
            <SummaryTile
              label={t("artworkMaintenance.summary.strayFiles")}
              tone={summary.remediationPlan.totals.cleanableStrayFiles > 0 ? "warning" : "neutral"}
              value={summary.remediationPlan.totals.cleanableStrayFiles}
            />
          </div>

          <div className="artworkMaintenanceGrid">
            <DataPanel
              description={t("artworkMaintenance.lifecycle.description", {
                returned: summary.lifecycle.page.returned,
                offset: summary.lifecycle.page.offset,
                limit: summary.lifecycle.page.limit,
              })}
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  {summary.lifecycle.dryRun
                    ? t("artworkMaintenance.dryRun")
                    : t("artworkMaintenance.liveRead")}
                </div>
              }
              title={t("artworkMaintenance.lifecycle.title")}
            >
              <LifecycleTable artifacts={summary.lifecycle.artifacts} t={t} />
            </DataPanel>

            <DataPanel
              description={t("artworkMaintenance.storage.description", {
                scanned: summary.storageDrift.totals.scannedFiles,
                limit: summary.storageDrift.totals.fileScanLimit,
              })}
              headerAccessory={
                <div className="searchHint">
                  <Search size={15} />
                  {summary.storageDrift.totals.fileScanTruncated
                    ? t("artworkMaintenance.scanTruncated")
                    : t("artworkMaintenance.scanComplete")}
                </div>
              }
              title={t("artworkMaintenance.storage.title")}
            >
              <StorageDriftPanel
                missingArtifacts={summary.storageDrift.missingArtifacts}
                strayFiles={summary.storageDrift.strayFiles}
                t={t}
              />
            </DataPanel>

            <DataPanel
              description={t("artworkMaintenance.remediation.description", {
                missing: summary.remediationPlan.totals.missingDbBackedArtifacts,
                stray: summary.remediationPlan.totals.cleanableStrayFiles,
              })}
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  {summary.remediationPlan.dryRun
                    ? t("artworkMaintenance.dryRun")
                    : t("artworkMaintenance.liveRead")}
                </div>
              }
              title={t("artworkMaintenance.remediation.title")}
            >
              <RemediationPlanPanel
                missingArtifacts={summary.remediationPlan.missingArtifacts}
                strayFiles={summary.remediationPlan.strayFiles}
                t={t}
              />
            </DataPanel>
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadManagedArtworkMaintenance(
  dataSource: AdminDataSource,
  search: ManagedArtworkMaintenanceSearch,
  unavailableMessage: string,
): Promise<MaintenanceResult> {
  if (!dataSource.loadManagedArtworkMaintenance) {
    return {
      value: emptyMaintenanceSummary(search),
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadManagedArtworkMaintenance(
    toLifecycleQuery(search),
    toStorageDriftQuery(search),
    toRemediationPlanQuery(search),
  );
}

function toLifecycleQuery(
  search: ManagedArtworkMaintenanceSearch,
): ManagedArtworkMaintenanceLifecycleQuery {
  return {
    cleanup_candidates_only: search.cleanup_candidates_only,
    limit: search.limit,
    offset: search.offset,
  };
}

function toStorageDriftQuery(
  search: ManagedArtworkMaintenanceSearch,
): ManagedArtworkMaintenanceStorageDriftQuery {
  return {
    file_scan_limit: search.file_scan_limit,
    limit: search.limit,
    offset: search.offset,
  };
}

function toRemediationPlanQuery(
  search: ManagedArtworkMaintenanceSearch,
): ManagedArtworkMaintenanceRemediationPlanQuery {
  return {
    file_scan_limit: search.file_scan_limit,
    limit: search.limit,
    offset: search.offset,
  };
}

function LifecycleTable({
  artifacts,
  t,
}: {
  artifacts: ManagedArtworkLifecycleArtifactRow[];
  t: Translate;
}) {
  if (artifacts.length === 0) {
    return <EmptyRouteState>{t("artworkMaintenance.lifecycle.empty")}</EmptyRouteState>;
  }

  return (
    <div className="tableScroll">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>{t("artworkMaintenance.column.artifact")}</TableHead>
            <TableHead>{t("artworkMaintenance.column.scope")}</TableHead>
            <TableHead>{t("artworkMaintenance.column.media")}</TableHead>
            <TableHead>{t("artworkMaintenance.column.state")}</TableHead>
            <TableHead>{t("artworkMaintenance.column.updated")}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {artifacts.map((artifact) => (
            <TableRow key={artifact.id}>
              <TableCell>
                <div className="routePrimaryCell">
                  <strong>{artifact.id}</strong>
                  <span>{t("artworkMaintenance.lifecycle.ingest", { ingestId: artifact.ingestId })}</span>
                </div>
              </TableCell>
              <TableCell>
                <span className="artworkMaintenanceMeta">
                  {artifact.libraryId} / {artifact.itemId} / {artifact.kind}
                </span>
              </TableCell>
              <TableCell>
                <span className="artworkMaintenanceMeta">
                  {dimensionLabel(artifact.width, artifact.height, t)} /{" "}
                  {formatBytes(artifact.byteLen, t)} /{" "}
                  {artifact.mediaType ?? t("artworkMaintenance.unknownMediaType")}
                </span>
              </TableCell>
              <TableCell>
                <div className="artworkMaintenanceBadgeStack">
                  <Badge tone={artifact.cleanupCandidate ? "warning" : "success"}>
                    {artifact.cleanupCandidate
                      ? t("artworkMaintenance.cleanupCandidate")
                      : t("artworkMaintenance.protected")}
                  </Badge>
                  <Badge tone={artifact.hasContentHash ? "info" : "neutral"}>
                    {artifact.hasContentHash
                      ? t("artworkMaintenance.hashPresent")
                      : t("artworkMaintenance.hashAbsent")}
                  </Badge>
                  <Badge tone={artifact.selectedArtworkCount > 0 ? "success" : "neutral"}>
                    {t("artworkMaintenance.selectedCount", {
                      count: artifact.selectedArtworkCount,
                    })}
                  </Badge>
                </div>
              </TableCell>
              <TableCell>{timestampLabel(artifact.updatedAt)}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function StorageDriftPanel({
  missingArtifacts,
  strayFiles,
  t,
}: {
  missingArtifacts: ManagedArtworkStorageDriftArtifactRow[];
  strayFiles: ManagedArtworkStorageDriftFileRow[];
  t: Translate;
}) {
  return (
    <div className="artworkMaintenanceList">
      <h3>{t("artworkMaintenance.storage.missingTitle")}</h3>
      {missingArtifacts.length === 0 ? (
        <EmptyRouteState>{t("artworkMaintenance.storage.noMissing")}</EmptyRouteState>
      ) : (
        missingArtifacts.map((artifact) => (
          <ArtifactIssueRow
            badge={artifact.issue}
            id={artifact.id}
            key={artifact.id}
            mediaType={artifact.mediaType}
            scope={`${artifact.libraryId} / ${artifact.itemId} / ${artifact.kind}`}
            size={artifact.byteLen}
            t={t}
            tone={artifact.cleanupCandidate ? "warning" : "danger"}
          />
        ))
      )}
      <h3>{t("artworkMaintenance.storage.strayTitle")}</h3>
      {strayFiles.length === 0 ? (
        <EmptyRouteState>{t("artworkMaintenance.storage.noStray")}</EmptyRouteState>
      ) : (
        strayFiles.map((file, index) => (
          <StrayFileRow
            action={null}
            file={file}
            key={`${file.reason}:${file.recognizedArtifactId ?? "unknown"}:${index}`}
            t={t}
          />
        ))
      )}
    </div>
  );
}

function RemediationPlanPanel({
  missingArtifacts,
  strayFiles,
  t,
}: {
  missingArtifacts: ManagedArtworkRemediationMissingArtifactRow[];
  strayFiles: ManagedArtworkRemediationStrayFileRow[];
  t: Translate;
}) {
  return (
    <div className="artworkMaintenanceList">
      <h3>{t("artworkMaintenance.remediation.missingTitle")}</h3>
      {missingArtifacts.length === 0 ? (
        <EmptyRouteState>{t("artworkMaintenance.remediation.noMissing")}</EmptyRouteState>
      ) : (
        missingArtifacts.map((artifact) => (
          <ArtifactIssueRow
            badge={artifact.recommendation}
            id={artifact.id}
            key={artifact.id}
            mediaType={artifact.mediaType}
            scope={`${artifact.issue} / ${artifact.libraryId} / ${artifact.itemId}`}
            size={artifact.byteLen}
            t={t}
            tone={artifact.cleanupCandidate ? "warning" : "danger"}
          />
        ))
      )}
      <h3>{t("artworkMaintenance.remediation.strayTitle")}</h3>
      {strayFiles.length === 0 ? (
        <EmptyRouteState>{t("artworkMaintenance.remediation.noStray")}</EmptyRouteState>
      ) : (
        strayFiles.map((file, index) => (
          <StrayFileRow
            action={file.action}
            file={file}
            key={`${file.reason}:${file.action}:${file.recognizedArtifactId ?? "unknown"}:${index}`}
            t={t}
          />
        ))
      )}
    </div>
  );
}

function ArtifactIssueRow({
  badge,
  id,
  mediaType,
  scope,
  size,
  t,
  tone,
}: {
  badge: string;
  id: string;
  mediaType: string | null;
  scope: string;
  size: number | null;
  t: Translate;
  tone: BadgeTone;
}) {
  return (
    <div className="artworkMaintenanceRow">
      <div>
        <strong>{id}</strong>
        <span>{scope}</span>
        <small>
          {formatBytes(size, t)} / {mediaType ?? t("artworkMaintenance.unknownMediaType")}
        </small>
      </div>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function StrayFileRow({
  action,
  file,
  t,
}: {
  action: string | null;
  file: ManagedArtworkRemediationStrayFileRow | ManagedArtworkStorageDriftFileRow;
  t: Translate;
}) {
  return (
    <div className="artworkMaintenanceRow">
      <div>
        <strong>{file.reason}</strong>
        <span>
          {file.recognizedArtifactId ??
            t("artworkMaintenance.stray.unrecognizedArtifact")}
        </span>
        <small>
          {file.extension ?? t("artworkMaintenance.stray.unknownExtension")} /{" "}
          {formatBytes(file.byteLen, t)}
        </small>
      </div>
      <Badge tone={action === "delete_stray_file" ? "warning" : "neutral"}>
        {action ?? t("artworkMaintenance.stray.inspectOnly")}
      </Badge>
    </div>
  );
}

function SummaryTile({
  label,
  tone,
  value,
}: {
  label: string;
  tone: BadgeTone;
  value: number | string;
}) {
  return (
    <div className="artworkSummaryTile">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{label}</Badge>
    </div>
  );
}

function emptyMaintenanceSummary(
  search: ManagedArtworkMaintenanceSearch,
): ManagedArtworkMaintenanceSummary {
  const page = {
    limit: search.limit,
    offset: search.offset,
    returned: 0,
  };

  return {
    lifecycle: {
      totals: {
        totalArtifacts: 0,
        protectedArtifacts: 0,
        cleanupCandidateArtifacts: 0,
        knownTotalBytes: 0,
        knownProtectedBytes: 0,
        knownCleanupCandidateBytes: 0,
        unknownByteLenArtifacts: 0,
      },
      artifacts: [],
      page,
      dryRun: true,
    },
    storageDrift: {
      totals: {
        scannedDbArtifacts: 0,
        dbBackedPresentArtifacts: 0,
        dbBackedMissingArtifacts: 0,
        dbBackedUnresolvableArtifacts: 0,
        dbBackedMetadataReadFailedArtifacts: 0,
        fileScanLimit: search.file_scan_limit,
        scannedFiles: 0,
        strayFiles: 0,
        untrackedArtifactFiles: 0,
        unexpectedActiveArtifactFiles: 0,
        unsupportedExtensionFiles: 0,
        unrecognizedLayoutFiles: 0,
        fileScanTruncated: false,
      },
      missingArtifacts: [],
      strayFiles: [],
      page,
      dryRun: true,
    },
    remediationPlan: {
      totals: {
        scannedDbArtifacts: 0,
        missingDbBackedArtifacts: 0,
        selectedMissingArtifacts: 0,
        cleanupCandidateMissingArtifacts: 0,
        fileScanLimit: search.file_scan_limit,
        scannedFiles: 0,
        cleanableStrayFiles: 0,
        blockedStrayFiles: 0,
        fileScanTruncated: false,
      },
      missingArtifacts: [],
      strayFiles: [],
      page,
      dryRun: true,
    },
  };
}

function positiveNumberInput(value: string, fallback: number, minimum: number) {
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed >= minimum ? parsed : fallback;
}

function dimensionLabel(width: number | null, height: number | null, t: Translate) {
  return width === null || height === null
    ? t("artworkMaintenance.dimensionsUnavailable")
    : `${width}x${height}`;
}

function formatBytes(value: number | null, t: Translate) {
  if (value === null) {
    return t("artworkMaintenance.sizeUnavailable");
  }

  return `${value.toLocaleString()} B`;
}

function timestampLabel(value: string) {
  return new Date(value).toLocaleString();
}
