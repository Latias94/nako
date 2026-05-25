import { RefreshCw, ShieldCheck } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type {
  AdminDataSource,
  DataSourceMode,
} from "../../adminApi/dataSource";
import type { AdminServerConfigDiagnosticsResponse } from "../../adminApi/types";
import { mockSystemConfig } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";

export type SettingsPageProps = {
  dataSource: AdminDataSource;
};

type SettingsResult = {
  value: AdminServerConfigDiagnosticsResponse;
  source: DataSourceMode;
  error?: string;
};

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";
type MetadataProvider = AdminServerConfigDiagnosticsResponse["metadata"]["providers"][number];

export function SettingsPage({ dataSource }: SettingsPageProps) {
  const query = useQuery({
    queryKey: ["admin-settings"],
    queryFn: () => loadSettings(dataSource),
  });
  const result = query.data ?? {
    value: mockSystemConfig,
    source: "mock" as const,
  };
  const settings = result.value;
  const enabledProviders = settings.metadata.providers.filter((provider) => provider.enabled).length;

  return (
    <RoutePage
      actions={
        <Button
          disabled={query.isFetching}
          onClick={() => void query.refetch()}
          variant="outline"
        >
          <RefreshCw size={16} />
          Refresh
        </Button>
      }
      description="Read-only system diagnostics from redacted Admin config. Mutation semantics stay out of this route."
      kicker="Configuration"
      status={<SourceLabel source={result.source} />}
      title="System Settings"
      titleId="settings-route-title"
    >
      {result.error ? (
        <RouteNotice>
          {result.error}. Showing deterministic mock fallback data.
        </RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label="Loading System Settings" /> : null}

      {!query.isLoading ? (
        <>
          <div className="settingsSummaryGrid">
            <SummaryCard
              badge={settings.auth.enabled ? "Configured" : "Open"}
              label="Admin auth"
              tone={settings.auth.enabled ? "success" : "warning"}
              value={settings.auth.enabled ? "Enabled" : "Disabled"}
            />
            <SummaryCard
              badge={settings.network.readiness.reason}
              label="Network"
              tone={readinessTone(settings.network.readiness.status)}
              value={settings.network.readiness.status}
            />
            <SummaryCard
              badge={settings.database.migrated_on_startup ? "Migrated" : "Pending"}
              label="Database backend"
              tone={settings.database.runtime_supported ? "success" : "danger"}
              value={settings.database.active_backend_kind}
            />
            <SummaryCard
              badge={`${enabledProviders} enabled`}
              label="Metadata providers"
              tone={enabledProviders > 0 ? "success" : "warning"}
              value={`${enabledProviders}/${settings.metadata.providers.length}`}
            />
            <SummaryCard
              badge={settings.transcode.gpu_concurrency > 0 ? "GPU slots" : "CPU only"}
              label="Transcode policy"
              tone={settings.transcode.gpu_concurrency > 0 ? "info" : "neutral"}
              value={`${settings.transcode.cpu_concurrency} CPU / ${settings.transcode.gpu_concurrency} GPU`}
            />
            <SummaryCard
              badge={settings.staging.cleanup_on_startup ? "Startup cleanup" : "Manual cleanup"}
              label="Staging budget"
              tone="neutral"
              value={formatBytes(settings.staging.max_bytes)}
            />
          </div>

          <div className="settingsPanelGrid">
            <DataPanel
              description="Exposure readiness without endpoint hosts, URLs, tokens, or header values."
              headerAccessory={
                <div className="searchHint">
                  <ShieldCheck size={15} />
                  Sensitive endpoints redacted
                </div>
              }
              title="Network readiness"
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={settings.network.readiness.status}
                  label="Exposure mode"
                  tone={readinessTone(settings.network.readiness.status)}
                  value={settings.network.exposure_mode}
                />
                <DiagnosticRow
                  badge={settings.network.external_endpoint.configured ? "configured" : "not configured"}
                  detail={settings.network.external_endpoint.scheme ?? "no scheme"}
                  label="External endpoint"
                  tone={settings.network.external_endpoint.configured ? "success" : "neutral"}
                  value={settings.network.external_endpoint.configured ? "Endpoint configured" : "No endpoint"}
                />
                <DiagnosticRow
                  badge={settings.network.trusted_proxy.headers_enabled ? "trusted" : "default deny"}
                  detail={`${settings.network.trusted_proxy.source_count} proxy sources`}
                  label="Trusted proxy"
                  tone={settings.network.trusted_proxy.headers_enabled ? "info" : "neutral"}
                  value="Forwarded headers"
                />
                <DiagnosticRow
                  badge={`${settings.network.tunnel_providers.length} tunnel providers`}
                  detail={`${settings.network.origins.allowed_origin_count} browser origins`}
                  label="Browser access"
                  tone={settings.network.origins.configured ? "success" : "warning"}
                  value={settings.network.origins.configured ? "Origin policy configured" : "Origin policy missing"}
                />
              </div>
            </DataPanel>

            <DataPanel
              description="Storage and runtime backend capability summary without connection strings."
              title="Database"
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={settings.database.runtime_supported ? "supported" : "unsupported"}
                  label="Active backend"
                  tone={settings.database.runtime_supported ? "success" : "danger"}
                  value={settings.database.active_backend_kind}
                />
                <DiagnosticRow
                  badge={settings.database.migrated_on_startup ? "migrated" : "not migrated"}
                  detail={`configured as ${settings.database.configured_backend_kind}`}
                  label="Startup migration"
                  tone={settings.database.migrated_on_startup ? "success" : "warning"}
                  value={settings.database.url_scheme}
                />
                <DiagnosticRow
                  badge={`${enabledCapabilityCount(settings)} enabled`}
                  detail="core stores and projections"
                  label="Capabilities"
                  tone="info"
                  value={`${enabledCapabilityCount(settings)}/${Object.keys(settings.database.capabilities).length}`}
                />
              </div>
            </DataPanel>

            <DataPanel
              description="Provider and cache policy summary without env var names, API keys, or proxy targets."
              title="Metadata policy"
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={settings.metadata.raw_cache_cleanup_on_startup ? "startup cleanup" : "scheduled cleanup"}
                  detail={`cleanup every ${formatDuration(settings.metadata.raw_cache_cleanup_interval_ms)}`}
                  label="Raw cache"
                  tone="neutral"
                  value={formatDuration(settings.metadata.raw_cache_retention_ms)}
                />
                <DiagnosticRow
                  badge={`${settings.metadata.maintenance_policies} policies`}
                  detail={`${settings.metadata.runtime.concurrency} concurrent requests`}
                  label="Runtime"
                  tone="info"
                  value={`${formatDuration(settings.metadata.runtime.timeout_ms)} timeout`}
                />
              </div>
              <div className="settingsProviderList">
                {settings.metadata.providers.map((provider) => (
                  <ProviderRow key={provider.provider} provider={provider} />
                ))}
              </div>
            </DataPanel>

            <DataPanel
              description="Playback, staging, and artwork worker policies without roots, paths, or fetch proxy values."
              title="Runtime policies"
            >
              <div className="settingsRowList">
                <DiagnosticRow
                  badge={`${settings.runtime.scan_concurrency} scan workers`}
                  detail={`${settings.runtime.probe_concurrency} probe / ${settings.runtime.metadata_concurrency} metadata`}
                  label="Library workers"
                  tone="info"
                  value={`${settings.runtime.webhook_concurrency} webhook workers`}
                />
                <DiagnosticRow
                  badge={`${settings.playback.remote_stream_concurrency} streams`}
                  detail={`${settings.playback.remote_stage_concurrency} remote stage workers`}
                  label="Playback"
                  tone="neutral"
                  value={`${settings.runtime.remux_concurrency} remux worker`}
                />
                <DiagnosticRow
                  badge={settings.staging.cleanup_on_startup ? "cleanup enabled" : "cleanup disabled"}
                  detail={`${formatDuration(settings.staging.retention_ms)} retention`}
                  label="Staging"
                  tone={settings.staging.cleanup_on_startup ? "success" : "warning"}
                  value={formatBytes(settings.staging.max_bytes)}
                />
                <DiagnosticRow
                  badge={settings.artwork.ingest_worker_enabled ? "worker enabled" : "worker disabled"}
                  detail={`${settings.artwork.fetch_concurrency} fetch workers`}
                  label="Artwork"
                  tone={settings.artwork.ingest_worker_enabled ? "success" : "warning"}
                  value={`${settings.artwork.max_width} x ${settings.artwork.max_height}`}
                />
              </div>
            </DataPanel>
          </div>
        </>
      ) : null}
    </RoutePage>
  );
}

async function loadSettings(dataSource: AdminDataSource): Promise<SettingsResult> {
  if (!dataSource.loadSettings) {
    return {
      value: mockSystemConfig,
      source: "mock",
      error: "System Settings route data source is unavailable",
    };
  }

  return dataSource.loadSettings();
}

function SummaryCard({
  badge,
  label,
  tone,
  value,
}: {
  badge: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="settingsSummaryCard">
      <span>{label}</span>
      <strong>{value}</strong>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function DiagnosticRow({
  badge,
  detail,
  label,
  tone,
  value,
}: {
  badge: string;
  detail?: string;
  label: string;
  tone: BadgeTone;
  value: string;
}) {
  return (
    <div className="settingsDiagnosticRow">
      <div>
        <span>{label}</span>
        <strong>{value}</strong>
        {detail ? <small>{detail}</small> : null}
      </div>
      <Badge tone={tone}>{badge}</Badge>
    </div>
  );
}

function ProviderRow({ provider }: { provider: MetadataProvider }) {
  return (
    <div className="settingsProviderRow">
      <div>
        <strong>{provider.provider.toUpperCase()}</strong>
        <span>{provider.language ?? "default language"}</span>
      </div>
      <Badge tone={provider.enabled ? "success" : "warning"}>
        {provider.enabled ? "enabled" : "disabled"}
      </Badge>
      <span>{provider.has_api_base_url ? "API base configured" : "default API base"}</span>
      <span>{provider.has_image_base_url ? "image base configured" : "default image base"}</span>
      <span>{provider.secret_header_count} secret headers</span>
    </div>
  );
}

function readinessTone(status: string): BadgeTone {
  if (status === "ready") {
    return "success";
  }

  if (status === "degraded") {
    return "warning";
  }

  return "danger";
}

function enabledCapabilityCount(settings: AdminServerConfigDiagnosticsResponse) {
  return Object.values(settings.database.capabilities).filter(Boolean).length;
}

function formatBytes(bytes: number) {
  if (bytes >= 1024 ** 3) {
    return `${Math.round(bytes / 1024 ** 3)} GiB`;
  }

  if (bytes >= 1024 ** 2) {
    return `${Math.round(bytes / 1024 ** 2)} MiB`;
  }

  if (bytes >= 1024) {
    return `${Math.round(bytes / 1024)} KiB`;
  }

  return `${bytes} B`;
}

function formatDuration(ms: number) {
  if (ms >= 24 * 60 * 60 * 1000) {
    return `${Math.round(ms / (24 * 60 * 60 * 1000))} d`;
  }

  if (ms >= 60 * 60 * 1000) {
    return `${Math.round(ms / (60 * 60 * 1000))} h`;
  }

  if (ms >= 1000) {
    return `${Math.round(ms / 1000)} s`;
  }

  return `${ms} ms`;
}
