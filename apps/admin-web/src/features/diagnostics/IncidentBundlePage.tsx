import { Copy, Download, RefreshCw } from "lucide-react";
import { useQuery } from "@tanstack/react-query";
import { useState } from "react";

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
  const [exportStatus, setExportStatus] = useState<{
    tone: "success" | "error";
    message: string;
  } | null>(null);
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
  const exportJson = incidentBundleExportJson(bundle);
  const sectionSummary = incidentBundleSectionSummary(bundle, t);

  async function copyBundle() {
    try {
      await copyTextToClipboard(exportJson);
      setExportStatus({ tone: "success", message: t("incidentBundle.copySuccess") });
    } catch {
      setExportStatus({ tone: "error", message: t("incidentBundle.copyFailure") });
    }
  }

  function downloadBundle() {
    const blob = new Blob([exportJson], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");
    link.href = url;
    link.download = incidentBundleFilename(bundle);
    link.click();
    URL.revokeObjectURL(url);
    setExportStatus({ tone: "success", message: t("incidentBundle.downloadReady") });
  }

  return (
    <RoutePage
      actions={
        <div className="routeActionGroup" role="group" aria-label={t("incidentBundle.actions")}>
          <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
            <RefreshCw size={16} />
            {t("incidentBundle.refresh")}
          </Button>
          <Button disabled={query.isLoading} onClick={() => void copyBundle()} variant="outline">
            <Copy size={16} />
            {t("incidentBundle.copy")}
          </Button>
          <Button disabled={query.isLoading} onClick={downloadBundle} variant="outline">
            <Download size={16} />
            {t("incidentBundle.download")}
          </Button>
        </div>
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
      {exportStatus ? (
        <RouteNotice>
          <span className={exportStatus.tone === "success" ? "statusPositive" : "statusDanger"}>
            {exportStatus.message}
          </span>
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("incidentBundle.loading")} /> : null}

      {!query.isLoading ? (
        <>
          <DataPanel
            description={t("incidentBundle.summary.description")}
            headerAccessory={
              <Badge tone={redactionComplete(bundle) ? "success" : "danger"}>
                {bundle.redaction.status}
              </Badge>
            }
            title={t("incidentBundle.summary.title")}
          >
            <div className="settingsRowList">
              {sectionSummary.map((section) => (
                <div className="settingsDiagnosticRow" key={section.key}>
                  <div>
                    <strong>{section.label}</strong>
                    <span>{section.detail}</span>
                  </div>
                  <Badge tone={section.tone}>{section.status}</Badge>
                </div>
              ))}
            </div>
          </DataPanel>

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
        </>
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

type BadgeTone = "danger" | "info" | "neutral" | "success" | "warning";

type Translate = (
  id: MessageId,
  values?: Record<string, boolean | number | string | null | undefined>,
) => string;

function incidentBundleSectionSummary(
  bundle: AdminIncidentBundleResponse,
  t: Translate,
): Array<{
  key: string;
  label: string;
  detail: string;
  status: string;
  tone: BadgeTone;
}> {
  const failedPressureGroups = bundle.jobs.queue_pressure.filter(
    (pressure) => pressure.status === "failed",
  ).length;
  const redactedFamilies = [
    bundle.redaction.raw_paths_redacted,
    bundle.redaction.locators_redacted,
    bundle.redaction.tokens_redacted,
    bundle.redaction.credentials_redacted,
    bundle.redaction.ffmpeg_command_lines_redacted,
    bundle.redaction.provider_payloads_redacted,
    bundle.redaction.backend_urls_redacted,
    bundle.redaction.query_strings_redacted,
    bundle.redaction.raw_job_payloads_redacted,
    bundle.redaction.unbounded_logs_redacted,
  ].filter(Boolean).length;
  const artifactSafe =
    bundle.artifact.format === "json_only"
    && !bundle.artifact.zip_archive_included
    && !bundle.artifact.upload_transport_included
    && !bundle.artifact.unbounded_logs_included;

  return [
    {
      key: "artifact",
      label: t("incidentBundle.artifact.title"),
      detail: t("incidentBundle.summary.artifactDetail"),
      status: bundle.artifact.format,
      tone: artifactSafe ? "success" : "danger",
    },
    {
      key: "overview",
      label: t("incidentBundle.overview.title"),
      detail: t("incidentBundle.summary.overviewDetail", {
        failed: bundle.overview.runtime.failed_jobs,
        readiness: bundle.overview.operator_readiness.status,
      }),
      status: bundle.overview.operator_readiness.status,
      tone: networkTone(bundle.overview.operator_readiness.status),
    },
    {
      key: "system",
      label: t("incidentBundle.system.title"),
      detail: t("incidentBundle.summary.systemDetail", {
        libraries: bundle.system.libraries.configured_count,
        providers: bundle.system.metadata.provider_count,
      }),
      status: bundle.system.auth_enabled
        ? t("incidentBundle.system.authEnabled")
        : t("incidentBundle.system.authDisabled"),
      tone: bundle.system.auth_enabled ? "success" : "warning",
    },
    {
      key: "network",
      label: t("incidentBundle.network.title"),
      detail: t("incidentBundle.summary.networkDetail", {
        exposure: bundle.network.exposure_mode,
        origins: bundle.network.allowed_origin_count,
        tunnels: bundle.network.tunnel_provider_count,
      }),
      status: bundle.network.readiness.status,
      tone: networkTone(bundle.network.readiness.status),
    },
    {
      key: "playback",
      label: t("incidentBundle.playback.title"),
      detail: t("incidentBundle.summary.playbackDetail", {
        available: bundle.playback.runtime.ffmpeg.available_gpu_capabilities,
        ffmpeg: bundle.playback.runtime.ffmpeg.probe_status,
        total: bundle.playback.runtime.ffmpeg.hardware_capability_count,
      }),
      status: bundle.playback.runtime.readiness.status,
      tone: networkTone(bundle.playback.runtime.readiness.status),
    },
    {
      key: "storage",
      label: t("incidentBundle.storage.title"),
      detail: t("incidentBundle.summary.storageDetail", {
        action: bundle.storage.vfs_cache_repair_action_plan.action,
        records: bundle.storage.staging.pressure.total_records,
      }),
      status: bundle.storage.vfs_cache_repair_action_plan.status,
      tone: storageSummaryTone(bundle.storage.vfs_cache_repair_action_plan.status),
    },
    {
      key: "jobs",
      label: t("incidentBundle.jobs.title"),
      detail: t("incidentBundle.summary.jobsDetail", {
        failed: failedPressureGroups,
        groups: bundle.jobs.queue_pressure.length,
      }),
      status: t("incidentBundle.summary.jobsStatus", {
        count: bundle.jobs.queue_pressure.length,
      }),
      tone:
        failedPressureGroups > 0
          ? "danger"
          : bundle.jobs.queue_pressure.length > 0
            ? "warning"
            : "success",
    },
    {
      key: "redaction",
      label: t("incidentBundle.redaction.title"),
      detail: t("incidentBundle.summary.redactionDetail", {
        families: redactedFamilies,
      }),
      status: bundle.redaction.status,
      tone: redactionComplete(bundle) ? "success" : "danger",
    },
  ];
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

function storageSummaryTone(status: string): BadgeTone {
  if (status === "no_action") {
    return "success";
  }

  if (status === "executable" || status === "plan_only") {
    return "warning";
  }

  return "danger";
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

export function incidentBundleExportJson(bundle: AdminIncidentBundleResponse) {
  return `${JSON.stringify(safeIncidentBundleExport(bundle), null, 2)}\n`;
}

function safeIncidentBundleExport(bundle: AdminIncidentBundleResponse): AdminIncidentBundleResponse {
  return {
    admin_api_version: bundle.admin_api_version,
    public_api_version: bundle.public_api_version,
    generated_at_ms: bundle.generated_at_ms,
    artifact: {
      format: bundle.artifact.format,
      zip_archive_included: bundle.artifact.zip_archive_included,
      upload_transport_included: bundle.artifact.upload_transport_included,
      unbounded_logs_included: bundle.artifact.unbounded_logs_included,
    },
    overview: safeExportObject(bundle.overview),
    system: {
      auth_enabled: bundle.system.auth_enabled,
      database: {
        configured_backend_kind: bundle.system.database.configured_backend_kind,
        active_backend_kind: bundle.system.database.active_backend_kind,
        url_scheme: bundle.system.database.url_scheme,
        runtime_supported: bundle.system.database.runtime_supported,
        migrated_on_startup: bundle.system.database.migrated_on_startup,
      },
      runtime: {
        scan_concurrency: bundle.system.runtime.scan_concurrency,
        probe_concurrency: bundle.system.runtime.probe_concurrency,
        metadata_concurrency: bundle.system.runtime.metadata_concurrency,
        remux_concurrency: bundle.system.runtime.remux_concurrency,
        webhook_concurrency: bundle.system.runtime.webhook_concurrency,
      },
      libraries: {
        configured_count: bundle.system.libraries.configured_count,
        local_count: bundle.system.libraries.local_count,
        webdav_count: bundle.system.libraries.webdav_count,
      },
      metadata: {
        provider_count: bundle.system.metadata.provider_count,
        enabled_provider_count: bundle.system.metadata.enabled_provider_count,
        disabled_provider_count: bundle.system.metadata.disabled_provider_count,
        providers_with_secret_reference_count:
          bundle.system.metadata.providers_with_secret_reference_count,
        providers_with_runtime_override_count:
          bundle.system.metadata.providers_with_runtime_override_count,
      },
    },
    network: {
      exposure_mode: bundle.network.exposure_mode,
      readiness: bundle.network.readiness,
      external_endpoint_configured: bundle.network.external_endpoint_configured,
      external_endpoint_scheme: bundle.network.external_endpoint_scheme,
      trusted_proxy_headers_enabled: bundle.network.trusted_proxy_headers_enabled,
      trusted_proxy_source_count: bundle.network.trusted_proxy_source_count,
      allowed_origin_count: bundle.network.allowed_origin_count,
      tunnel_provider_count: bundle.network.tunnel_provider_count,
      tunnel_providers_with_endpoint_count: bundle.network.tunnel_providers_with_endpoint_count,
      tunnel_providers_with_token_reference_count:
        bundle.network.tunnel_providers_with_token_reference_count,
    },
    playback: {
      runtime: safeExportObject(bundle.playback.runtime),
      support_evidence: safeExportObject(bundle.playback.support_evidence),
    },
    storage: {
      staging: safeExportObject(bundle.storage.staging),
      vfs_cache_repair_action_plan: safeExportObject(bundle.storage.vfs_cache_repair_action_plan),
    },
    jobs: {
      queue_pressure: bundle.jobs.queue_pressure.map((item) => ({
        kind: item.kind,
        status: item.status,
        resource_class: item.resource_class,
        count: item.count,
        claimable_count: item.claimable_count,
        delayed_retry_count: item.delayed_retry_count,
        oldest_queued_at: item.oldest_queued_at,
        next_attempt_at: item.next_attempt_at,
      })),
    },
    redaction: {
      status: bundle.redaction.status,
      raw_paths_redacted: bundle.redaction.raw_paths_redacted,
      locators_redacted: bundle.redaction.locators_redacted,
      tokens_redacted: bundle.redaction.tokens_redacted,
      credentials_redacted: bundle.redaction.credentials_redacted,
      ffmpeg_command_lines_redacted: bundle.redaction.ffmpeg_command_lines_redacted,
      provider_payloads_redacted: bundle.redaction.provider_payloads_redacted,
      backend_urls_redacted: bundle.redaction.backend_urls_redacted,
      query_strings_redacted: bundle.redaction.query_strings_redacted,
      raw_job_payloads_redacted: bundle.redaction.raw_job_payloads_redacted,
      unbounded_logs_redacted: bundle.redaction.unbounded_logs_redacted,
    },
  };
}

function incidentBundleFilename(bundle: AdminIncidentBundleResponse) {
  return `nako-incident-bundle-${bundle.generated_at_ms}.json`;
}

type SafeJsonValue = null | boolean | number | string | SafeJsonValue[] | { [key: string]: SafeJsonValue };

function safeExportObject<T>(value: T): T {
  return safeExportValue(value) as unknown as T;
}

function safeExportValue(value: unknown): SafeJsonValue | undefined {
  if (value === null) {
    return null;
  }

  if (typeof value === "string") {
    return unsafeExportString(value) ? "<redacted>" : value;
  }

  if (typeof value === "boolean" || typeof value === "number") {
    return value;
  }

  if (Array.isArray(value)) {
    return value
      .map((item) => safeExportValue(item))
      .filter((item): item is SafeJsonValue => item !== undefined);
  }

  if (typeof value !== "object" || value === undefined) {
    return undefined;
  }

  const result: { [key: string]: SafeJsonValue } = {};
  for (const [key, childValue] of Object.entries(value)) {
    if (forbiddenExportKey(key)) {
      continue;
    }

    const safeChildValue = safeExportValue(childValue);
    if (safeChildValue !== undefined) {
      result[key] = safeChildValue;
    }
  }

  return result;
}

function forbiddenExportKey(key: string) {
  const lower = key.toLowerCase();

  if (
    lower.endsWith("_redacted")
    || lower.endsWith("_count")
    || lower.startsWith("has_")
    || lower === "source_fingerprint_hash"
    || lower === "url_scheme"
    || lower === "root_scheme"
    || lower === "source_scheme"
    || lower === "external_endpoint_scheme"
  ) {
    return false;
  }

  if (
    lower === "command"
    || lower === "stderr"
    || lower === "file_name"
    || lower === "query_string"
  ) {
    return true;
  }

  return [
    "_url",
    "_uri",
    "_path",
    "_locator",
    "_payload",
    "_command",
    "_stderr",
    "_token",
    "_secret",
    "_password",
    "_credential",
    "_credentials",
    "_fingerprint",
    "_etag",
    "_ref",
  ].some((suffix) => lower.endsWith(suffix));
}

function unsafeExportString(value: string) {
  return (
    /^[a-z][a-z0-9+.-]*:\/\//iu.test(value)
    || /[a-z]:[\\/]/iu.test(value)
    || /(^|[\s"'])\/(users|home|var|tmp|mnt|media|srv|opt|etc)\//iu.test(value)
    || /\?.*(token|secret|password)=/iu.test(value)
  );
}

async function copyTextToClipboard(text: string) {
  if (!navigator.clipboard?.writeText) {
    throw new Error("clipboard unavailable");
  }

  await navigator.clipboard.writeText(text);
}
