import { RefreshCw } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type {
  AdminJobQueuePressureSummary,
  AdminOperatorReadinessCheck,
  AdminOperatorReadinessResponse,
} from "../../adminApi/types";
import { mockOperatorReadiness } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";
import {
  operatorReadinessActionLabel,
  operatorReadinessAreaLabel,
  operatorReadinessReasonLabel,
  operatorReadinessStatusLabel,
  operatorReadinessTone,
  safeOperatorReadinessDisplayValue,
  safeOperatorReadinessSourceReason,
  type OperatorReadinessTranslate,
} from "./operatorReadinessFormatters";

export type OperatorReadinessPageProps = {
  dataSource: AdminDataSource;
};

type OperatorReadinessResult = {
  value: AdminOperatorReadinessResponse;
  source: DataSourceMode;
  error?: string;
};

type FactItem = {
  label: string;
  value: string;
};

export function OperatorReadinessPage({
  dataSource,
}: OperatorReadinessPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-operator-readiness", locale],
    queryFn: () =>
      loadOperatorReadiness(
        dataSource,
        t("operatorReadiness.dataSourceUnavailable"),
      ),
  });
  const result = query.data ?? {
    value: mockOperatorReadiness,
    source: "mock" as const,
  };
  const readiness = result.value;

  return (
    <RoutePage
      actions={
        <div
          className="routeActionGroup"
          role="group"
          aria-label={t("operatorReadiness.actions")}
        >
          <a className="routeTextLink" href="/overview">
            {t("operatorReadiness.backToOverview")}
          </a>
          <Button
            disabled={query.isFetching}
            onClick={() => void query.refetch()}
            variant="outline"
          >
            <RefreshCw size={16} />
            {t("operatorReadiness.refresh")}
          </Button>
        </div>
      }
      description={t("operatorReadiness.description")}
      kicker={t("operatorReadiness.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("operatorReadiness.title")}
      titleId="operator-readiness-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {t("operatorReadiness.fallback", { error: result.error })}
        </RouteNotice>
      ) : null}

      {query.isLoading ? (
        <RowsSkeleton label={t("operatorReadiness.loading")} />
      ) : null}

      {!query.isLoading ? (
        <>
          <div className="overviewMetricGrid">
            <ReadinessMetric
              badge={operatorReadinessStatusLabel(readiness.summary.status, t)}
              label={t("operatorReadiness.summary.status")}
              tone={operatorReadinessTone(readiness.summary.status)}
              value={readiness.summary.status}
            />
            <ReadinessMetric
              badge={t("operatorReadiness.summary.attentionBadge")}
              label={t("operatorReadiness.summary.attention")}
              tone={attentionCount(readiness.summary.checks) > 0 ? "warning" : "success"}
              value={String(attentionCount(readiness.summary.checks))}
            />
            <ReadinessMetric
              badge={t("operatorReadiness.summary.readyBadge")}
              label={t("operatorReadiness.summary.readyAreas")}
              tone="info"
              value={t("operatorReadiness.summary.readyValue", {
                ready: readyAreaCount(readiness.summary.checks),
                total: readiness.summary.checks.length,
              })}
            />
          </div>

          <DataPanel
            description={t("operatorReadiness.summary.description")}
            headerAccessory={
              <Badge tone={operatorReadinessTone(readiness.summary.status)}>
                {operatorReadinessStatusLabel(readiness.summary.status, t)}
              </Badge>
            }
            title={t("operatorReadiness.summary.title")}
          >
            <div className="overviewReadinessGrid">
              {readiness.summary.checks.map((check) => (
                <ReadinessCheckCard check={check} key={check.area} t={t} />
              ))}
            </div>
          </DataPanel>

          <div className="libraryDetailGrid">
            <ReadinessDetailPanel
              check={readiness.details.setup.check}
              facts={[
                {
                  label: t("operatorReadiness.setup.auth"),
                  value: yesNo(readiness.details.setup.auth_enabled, t),
                },
                {
                  label: t("operatorReadiness.setup.tokenReference"),
                  value: yesNo(
                    readiness.details.setup.token_reference_configured,
                    t,
                  ),
                },
                {
                  label: t("operatorReadiness.setup.exposure"),
                  value: readiness.details.setup.exposure_mode,
                },
              ]}
              title={operatorReadinessAreaLabel(readiness.details.setup.check, t)}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.media_library_scan.check}
              facts={[
                {
                  label: t("operatorReadiness.scan.configuredLibraries"),
                  value: String(
                    readiness.details.media_library_scan.configured_libraries,
                  ),
                },
                {
                  label: t("operatorReadiness.scan.libraryPosture"),
                  value: t("operatorReadiness.scan.libraryPostureValue", {
                    failed:
                      readiness.details.media_library_scan.library_scan
                        .failed_libraries,
                    pending:
                      readiness.details.media_library_scan.library_scan
                        .pending_libraries,
                    succeeded:
                      readiness.details.media_library_scan.library_scan
                        .succeeded_libraries,
                  }),
                },
                {
                  label: t("operatorReadiness.scan.sourceHashCoverage"),
                  value: t("operatorReadiness.scan.sourceHashCoverageValue", {
                    fingerprinted:
                      readiness.details.media_library_scan.source_fingerprint_hash
                        .fingerprinted_sources,
                    total:
                      readiness.details.media_library_scan.source_fingerprint_hash
                        .total_sources,
                  }),
                },
                {
                  label: t("operatorReadiness.scan.watchFolders"),
                  value: t("operatorReadiness.scan.watchFoldersValue", {
                    started:
                      readiness.details.media_library_scan.watch_folder_runtime
                        .started_libraries,
                    total:
                      readiness.details.media_library_scan.watch_folder_runtime
                        .configured_libraries,
                  }),
                },
              ]}
              title={operatorReadinessAreaLabel(
                readiness.details.media_library_scan.check,
                t,
              )}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.playback.check}
              facts={[
                {
                  label: t("operatorReadiness.playback.readiness"),
                  value: readiness.details.playback.readiness.status,
                },
                {
                  label: t("operatorReadiness.playback.reason"),
                  value: readiness.details.playback.readiness.reason,
                },
                {
                  label: t("operatorReadiness.playback.checks"),
                  value: t("operatorReadiness.playback.checksValue", {
                    count: readiness.details.playback.readiness.checks.length,
                  }),
                },
              ]}
              title={operatorReadinessAreaLabel(
                readiness.details.playback.check,
                t,
              )}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.durable_jobs.check}
              facts={durableJobFacts(
                readiness.details.durable_jobs.queue_pressure,
                t,
              )}
              title={operatorReadinessAreaLabel(
                readiness.details.durable_jobs.check,
                t,
              )}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.storage.check}
              facts={[
                {
                  label: t("operatorReadiness.storage.backends"),
                  value: t("operatorReadiness.storage.backendsValue", {
                    ready: readiness.details.storage.summary.ready_backends,
                    total: readiness.details.storage.summary.total_backends,
                  }),
                },
                {
                  label: t("operatorReadiness.storage.degraded"),
                  value: String(
                    readiness.details.storage.summary.degraded_backends,
                  ),
                },
                {
                  label: t("operatorReadiness.storage.vfsRepair"),
                  value: readiness.details.storage.vfs_cache_repair
                    ? t("operatorReadiness.storage.vfsRepairValue", {
                        classification:
                          readiness.details.storage.vfs_cache_repair
                            .primary_classification,
                        count:
                          readiness.details.storage.vfs_cache_repair
                            .total_unresolved_targets,
                      })
                    : t("operatorReadiness.none"),
                },
              ]}
              title={operatorReadinessAreaLabel(
                readiness.details.storage.check,
                t,
              )}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.network.check}
              facts={[
                {
                  label: t("operatorReadiness.network.readiness"),
                  value: readiness.details.network.readiness.status,
                },
                {
                  label: t("operatorReadiness.network.reason"),
                  value: readiness.details.network.readiness.reason,
                },
                {
                  label: t("operatorReadiness.network.checks"),
                  value: t("operatorReadiness.network.checksValue", {
                    count: readiness.details.network.readiness.checks.length,
                  }),
                },
              ]}
              title={operatorReadinessAreaLabel(
                readiness.details.network.check,
                t,
              )}
              t={t}
            />

            <ReadinessDetailPanel
              check={readiness.details.backup.check}
              facts={[
                {
                  label: t("operatorReadiness.backup.durableDatabase"),
                  value: yesNo(
                    readiness.details.backup.durable_database_configured,
                    t,
                  ),
                },
              ]}
              title={operatorReadinessAreaLabel(
                readiness.details.backup.check,
                t,
              )}
              t={t}
            />
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadOperatorReadiness(
  dataSource: AdminDataSource,
  unavailableMessage: string,
): Promise<OperatorReadinessResult> {
  if (!dataSource.loadOperatorReadiness) {
    return {
      value: mockOperatorReadiness,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadOperatorReadiness();
}

function ReadinessMetric({
  badge,
  label,
  tone,
  value,
}: {
  badge: string;
  label: string;
  tone: "danger" | "info" | "success" | "warning";
  value: string;
}) {
  return (
    <div className="overviewMetric">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function ReadinessCheckCard({
  check,
  t,
}: {
  check: AdminOperatorReadinessCheck;
  t: OperatorReadinessTranslate;
}) {
  const sourceReason = check.source_reason
    ? safeOperatorReadinessSourceReason(check.source_reason, t)
    : null;
  const actionLabel = check.action
    ? operatorReadinessActionLabel(check.action.route_key, t)
    : null;

  return (
    <div className="overviewReadinessItem">
      <div className="overviewReadinessHeader">
        <strong>{operatorReadinessAreaLabel(check, t)}</strong>
        <Badge tone={operatorReadinessTone(check.status)}>
          {operatorReadinessStatusLabel(check.status, t)}
        </Badge>
      </div>
      <span>{operatorReadinessReasonLabel(check, t)}</span>
      {sourceReason ? (
        <small>
          {t("overview.operatorReadiness.sourceReason", {
            reason: sourceReason,
          })}
        </small>
      ) : null}
      {actionLabel ? (
        <small>
          {t("operatorReadiness.actionHint", { route: actionLabel })}
        </small>
      ) : null}
    </div>
  );
}

function ReadinessDetailPanel({
  check,
  facts,
  title,
  t,
}: {
  check: AdminOperatorReadinessCheck;
  facts: FactItem[];
  title: string;
  t: OperatorReadinessTranslate;
}) {
  return (
    <DataPanel
      description={operatorReadinessReasonLabel(check, t)}
      headerAccessory={
        <Badge tone={operatorReadinessTone(check.status)}>
          {operatorReadinessStatusLabel(check.status, t)}
        </Badge>
      }
      title={title}
    >
      <div className="libraryFactList">
        {facts.map((fact) => (
          <Fact key={fact.label} label={fact.label} value={fact.value} />
        ))}
        <Fact
          label={t("operatorReadiness.detail.reasonCode")}
          value={check.reason}
        />
        {check.source_reason ? (
          <Fact
            label={t("operatorReadiness.detail.sourceReason")}
            value={safeOperatorReadinessSourceReason(check.source_reason, t)}
          />
        ) : null}
        {check.action ? (
          <Fact
            label={t("operatorReadiness.detail.action")}
            value={safeOperatorReadinessDisplayValue(
              operatorReadinessActionLabel(check.action.route_key, t),
              t,
            )}
          />
        ) : null}
      </div>
    </DataPanel>
  );
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

function durableJobFacts(
  queuePressure: AdminJobQueuePressureSummary[],
  t: OperatorReadinessTranslate,
): FactItem[] {
  if (queuePressure.length === 0) {
    return [
      {
        label: t("operatorReadiness.jobs.queuePressure"),
        value: t("operatorReadiness.none"),
      },
    ];
  }

  return queuePressure.slice(0, 4).map((pressure) => ({
    label: `${pressure.kind} / ${pressure.status}`,
    value: t("operatorReadiness.jobs.queuePressureValue", {
      claimable: pressure.claimable_count,
      count: pressure.count,
      delayed: pressure.delayed_retry_count,
      resource: safeOperatorReadinessDisplayValue(pressure.resource_class, t),
    }),
  }));
}

function attentionCount(checks: AdminOperatorReadinessCheck[]) {
  return checks.reduce((sum, check) => sum + check.attention_count, 0);
}

function readyAreaCount(checks: AdminOperatorReadinessCheck[]) {
  return checks.filter((check) => check.status === "ready").length;
}

function yesNo(value: boolean, t: (id: MessageId) => string) {
  return value
    ? t("operatorReadiness.boolean.yes")
    : t("operatorReadiness.boolean.no");
}
