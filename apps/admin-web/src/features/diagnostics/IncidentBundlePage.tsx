import { RefreshCw } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type { AdminIncidentBundleResponse } from "../../adminApi/types";
import { mockIncidentBundle } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";
import type { MessageId } from "../../i18n/messages";

export type IncidentBundlePageProps = {
  dataSource: AdminDataSource;
};

type IncidentBundleResult = {
  value: AdminIncidentBundleResponse;
  source: DataSourceMode;
  error?: string;
};

export function IncidentBundlePage({ dataSource }: IncidentBundlePageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-incident-bundle", locale],
    queryFn: () => loadIncidentBundle(dataSource, t("incidentBundle.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockIncidentBundle,
    source: "mock" as const,
  };
  const bundle = result.value;
  const staging = bundle.storage.staging;
  const repairPlan = bundle.storage.vfs_cache_repair_action_plan;
  const support = bundle.playback.support_evidence;

  return (
    <RoutePage
      actions={
        <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
          <RefreshCw size={16} />
          {t("incidentBundle.refresh")}
        </Button>
      }
      description={t("incidentBundle.description")}
      kicker={t("incidentBundle.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("incidentBundle.title")}
      titleId="incident-bundle-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("incidentBundle.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("incidentBundle.loading")} /> : null}

      {!query.isLoading ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("incidentBundle.artifact.description")}
            headerAccessory={
              <Badge tone={bundle.artifact.format === "json_only" ? "success" : "warning"}>
                {bundle.artifact.format}
              </Badge>
            }
            title={t("incidentBundle.artifact.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.artifact.generatedAt")}
                value={String(bundle.generated_at_ms)}
              />
              <Fact
                label={t("incidentBundle.artifact.zip")}
                value={boolText(bundle.artifact.zip_archive_included, t)}
              />
              <Fact
                label={t("incidentBundle.artifact.upload")}
                value={boolText(bundle.artifact.upload_transport_included, t)}
              />
              <Fact
                label={t("incidentBundle.artifact.logs")}
                value={boolText(bundle.artifact.unbounded_logs_included, t)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.overview.description")}
            headerAccessory={<Badge tone={statusTone(bundle.overview.status)}>{bundle.overview.status}</Badge>}
            title={t("incidentBundle.overview.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("incidentBundle.overview.server")} value={bundle.overview.status} />
              <Fact
                label={t("incidentBundle.overview.readiness")}
                value={bundle.overview.operator_readiness.status}
              />
              <Fact
                label={t("incidentBundle.overview.storage")}
                value={t("incidentBundle.overview.storageValue", {
                  ready: bundle.overview.storage.ready_backends,
                  total: bundle.overview.storage.total_backends,
                })}
              />
              <Fact
                label={t("incidentBundle.overview.failedJobs")}
                value={String(bundle.overview.runtime.failed_jobs)}
              />
              <Fact
                label={t("incidentBundle.overview.sourceHash")}
                value={t("incidentBundle.overview.sourceHashValue", {
                  fingerprinted: bundle.overview.source_fingerprint_hash.fingerprinted_sources,
                  total: bundle.overview.source_fingerprint_hash.total_sources,
                })}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.system.description")}
            headerAccessory={
              <Badge tone={bundle.system.auth_enabled ? "success" : "warning"}>
                {bundle.system.auth_enabled
                  ? t("incidentBundle.system.authEnabled")
                  : t("incidentBundle.system.authDisabled")}
              </Badge>
            }
            title={t("incidentBundle.system.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.system.database")}
                value={`${bundle.system.database.active_backend_kind} / ${bundle.system.database.url_scheme}`}
              />
              <Fact
                label={t("incidentBundle.system.libraries")}
                value={String(bundle.system.libraries.configured_count)}
              />
              <Fact
                label={t("incidentBundle.system.runtime")}
                value={t("incidentBundle.system.runtimeValue", {
                  scan: bundle.system.runtime.scan_concurrency,
                  probe: bundle.system.runtime.probe_concurrency,
                  metadata: bundle.system.runtime.metadata_concurrency,
                })}
              />
              <Fact
                label={t("incidentBundle.system.providers")}
                value={String(bundle.system.metadata.provider_count)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.network.description")}
            headerAccessory={
              <Badge tone={networkTone(bundle.network.readiness.status)}>
                {bundle.network.readiness.status}
              </Badge>
            }
            title={t("incidentBundle.network.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.network.exposure")}
                value={bundle.network.exposure_mode}
              />
              <Fact
                label={t("incidentBundle.network.endpoint")}
                value={
                  bundle.network.external_endpoint_configured
                    ? t("incidentBundle.network.endpointConfigured", {
                        scheme: bundle.network.external_endpoint_scheme ?? t("incidentBundle.none"),
                      })
                    : t("incidentBundle.network.endpointMissing")
                }
              />
              <Fact
                label={t("incidentBundle.network.trustedProxy")}
                value={t("incidentBundle.network.trustedProxyValue", {
                  count: bundle.network.trusted_proxy_source_count,
                })}
              />
              <Fact
                label={t("incidentBundle.network.origins")}
                value={t("incidentBundle.network.originsValue", {
                  count: bundle.network.allowed_origin_count,
                })}
              />
              <Fact
                label={t("incidentBundle.network.tunnels")}
                value={String(bundle.network.tunnel_provider_count)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.playback.description")}
            headerAccessory={
              <Badge tone={networkTone(bundle.playback.runtime.readiness.status)}>
                {bundle.playback.runtime.readiness.status}
              </Badge>
            }
            title={t("incidentBundle.playback.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.playback.ffmpeg")}
                value={bundle.playback.runtime.ffmpeg.probe_status}
              />
              <Fact
                label={t("incidentBundle.playback.hardware")}
                value={t("incidentBundle.playback.hardwareValue", {
                  available: bundle.playback.runtime.ffmpeg.available_gpu_capabilities,
                  total: bundle.playback.runtime.ffmpeg.hardware_capability_count,
                })}
              />
              <Fact
                label={t("incidentBundle.playback.supportSubject")}
                value={support.subject.source_id ?? support.subject.session_id ?? t("incidentBundle.none")}
              />
              <Fact
                label={t("incidentBundle.playback.failure")}
                value={support.session?.failure_category ?? t("incidentBundle.none")}
              />
              <Fact
                label={t("incidentBundle.playback.redaction")}
                value={boolText(
                  support.redaction.paths_redacted
                    && support.redaction.source_references_redacted
                    && support.redaction.ffmpeg_commands_redacted
                    && support.redaction.credentials_redacted,
                  t,
                )}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.storage.description")}
            headerAccessory={<Badge tone={repairTone(repairPlan.status)}>{repairPlan.status}</Badge>}
            title={t("incidentBundle.storage.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.storage.stagingBytes")}
                value={t("incidentBundle.storage.stagingBytesValue", {
                  used: staging.used_manifest_bytes,
                  max: staging.configured_max_bytes,
                })}
              />
              <Fact
                label={t("incidentBundle.storage.records")}
                value={String(staging.pressure.total_records)}
              />
              <Fact
                label={t("incidentBundle.storage.vfsObjects")}
                value={String(staging.vfs_cache.object_count)}
              />
              <Fact label={t("incidentBundle.storage.repairAction")} value={repairPlan.action} />
              <Fact
                label={t("incidentBundle.storage.repairMessage")}
                value={repairPlan.repair?.safe_message ?? t("incidentBundle.none")}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.jobs.description")}
            headerAccessory={
              <Badge tone={bundle.jobs.queue_pressure.length > 0 ? "warning" : "success"}>
                {t("incidentBundle.jobs.pressureCount", {
                  count: bundle.jobs.queue_pressure.length,
                })}
              </Badge>
            }
            title={t("incidentBundle.jobs.title")}
          >
            {bundle.jobs.queue_pressure.length > 0 ? (
              <div className="settingsRowList">
                {bundle.jobs.queue_pressure.map((item) => (
                  <div className="settingsDiagnosticRow" key={`${item.kind}:${item.status}:${item.resource_class}`}>
                    <div>
                      <strong>{item.kind}</strong>
                      <span>
                        {t("incidentBundle.jobs.queuePressureDetail", {
                          claimable: item.claimable_count,
                          count: item.count,
                          delayed: item.delayed_retry_count,
                          resource: item.resource_class,
                          status: item.status,
                        })}
                      </span>
                    </div>
                    <Badge tone={item.status === "failed" ? "danger" : "neutral"}>{item.status}</Badge>
                  </div>
                ))}
              </div>
            ) : (
              <RouteNotice>{t("incidentBundle.jobs.empty")}</RouteNotice>
            )}
          </DataPanel>

          <DataPanel
            description={t("incidentBundle.redaction.description")}
            headerAccessory={<Badge tone={redactionComplete(bundle) ? "success" : "danger"}>{bundle.redaction.status}</Badge>}
            title={t("incidentBundle.redaction.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("incidentBundle.redaction.rawPaths")}
                value={boolText(bundle.redaction.raw_paths_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.locators")}
                value={boolText(bundle.redaction.locators_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.tokens")}
                value={boolText(bundle.redaction.tokens_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.credentials")}
                value={boolText(bundle.redaction.credentials_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.ffmpeg")}
                value={boolText(bundle.redaction.ffmpeg_command_lines_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.providerPayloads")}
                value={boolText(bundle.redaction.provider_payloads_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.backendUrls")}
                value={boolText(bundle.redaction.backend_urls_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.queryStrings")}
                value={boolText(bundle.redaction.query_strings_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.rawJobPayloads")}
                value={boolText(bundle.redaction.raw_job_payloads_redacted, t)}
              />
              <Fact
                label={t("incidentBundle.redaction.unboundedLogs")}
                value={boolText(bundle.redaction.unbounded_logs_redacted, t)}
              />
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

async function loadIncidentBundle(
  dataSource: AdminDataSource,
  unavailableMessage: string,
): Promise<IncidentBundleResult> {
  if (!dataSource.loadIncidentBundle) {
    return {
      value: mockIncidentBundle,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadIncidentBundle();
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

function boolText(value: boolean, t: (id: MessageId) => string) {
  return value ? t("incidentBundle.boolean.yes") : t("incidentBundle.boolean.no");
}

function statusTone(status: string) {
  if (status === "healthy") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

function networkTone(status: string) {
  if (status === "ready") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

function repairTone(status: string) {
  if (status === "executable") {
    return "success";
  }

  if (status === "not_needed") {
    return "neutral";
  }

  return "warning";
}

function redactionComplete(bundle: AdminIncidentBundleResponse) {
  return (
    bundle.redaction.status === "complete"
    && bundle.redaction.raw_paths_redacted
    && bundle.redaction.locators_redacted
    && bundle.redaction.tokens_redacted
    && bundle.redaction.credentials_redacted
    && bundle.redaction.ffmpeg_command_lines_redacted
    && bundle.redaction.provider_payloads_redacted
    && bundle.redaction.backend_urls_redacted
    && bundle.redaction.query_strings_redacted
    && bundle.redaction.raw_job_payloads_redacted
    && bundle.redaction.unbounded_logs_redacted
  );
}
