import { RefreshCw } from "lucide-react";
import { useQuery } from "@tanstack/react-query";

import type { AdminDataSource, DataSourceMode } from "../../adminApi/dataSource";
import type {
  AdminPlaybackSupportEvidenceResponse,
  AdminPlaybackSupportQuery,
} from "../../adminApi/types";
import { mockPlaybackSupport } from "../../adminApi/mockData";
import { SourceLabel } from "../../components/SourceLabel";
import { RouteNotice, RoutePage } from "../../components/layout/RoutePage";
import { Badge } from "../../components/ui/Badge";
import { Button } from "../../components/ui/Button";
import { DataPanel } from "../../components/ui/DataPanel";
import { RowsSkeleton } from "../../components/ui/RowsSkeleton";
import { useI18n } from "../../i18n/I18nProvider";

export type PlaybackSupportSearch = AdminPlaybackSupportQuery;

export type PlaybackSupportPageProps = {
  dataSource: AdminDataSource;
  search: PlaybackSupportSearch;
};

type PlaybackSupportResult = {
  value: AdminPlaybackSupportEvidenceResponse;
  source: DataSourceMode;
  error?: string;
};

type PlaybackSupportSession = NonNullable<AdminPlaybackSupportEvidenceResponse["session"]>;

export function PlaybackSupportPage({ dataSource, search }: PlaybackSupportPageProps) {
  const { locale, t } = useI18n();
  const query = useQuery({
    queryKey: ["admin-playback-support", search, locale],
    queryFn: () => loadPlaybackSupport(dataSource, search, t("playbackSupport.dataSourceUnavailable")),
  });
  const result = query.data ?? {
    value: mockPlaybackSupport,
    source: "mock" as const,
  };
  const support = result.value;
  const session = support.session;
  const source = support.source;

  return (
    <RoutePage
      actions={
        <Button disabled={query.isFetching} onClick={() => void query.refetch()} variant="outline">
          <RefreshCw size={16} />
          {t("playbackSupport.refresh")}
        </Button>
      }
      description={t("playbackSupport.description")}
      kicker={t("playbackSupport.kicker")}
      status={<SourceLabel source={result.source} />}
      title={t("playbackSupport.title")}
      titleId="playback-support-route-title"
    >
      {result.error ? (
        <RouteNotice>{t("playbackSupport.fallback", { error: result.error })}</RouteNotice>
      ) : null}

      {query.isLoading ? <RowsSkeleton label={t("playbackSupport.loading")} /> : null}

      {!query.isLoading ? (
        <div className="libraryDetailGrid">
          <DataPanel
            description={t("playbackSupport.subject.description")}
            title={t("playbackSupport.subject.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("playbackSupport.subject.sessionId")}
                value={support.subject.session_id ?? t("playback.none")}
              />
              <Fact
                label={t("playbackSupport.subject.sourceId")}
                value={support.subject.source_id ?? t("playback.none")}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("playbackSupport.session.description")}
            headerAccessory={
              <Badge tone={session ? sessionTone(session.state) : "neutral"}>
                {session ? session.state : t("playback.none")}
              </Badge>
            }
            title={t("playbackSupport.session.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("playbackSupport.session.id")} value={session?.id ?? t("playback.none")} />
              <Fact
                label={t("playbackSupport.session.sourceId")}
                value={session?.source_id ?? t("playback.none")}
              />
              <Fact label={t("playbackSupport.session.kind")} value={session?.kind ?? t("playback.none")} />
              <Fact
                label={t("playbackSupport.session.failureCategory")}
                value={session?.failure_category ?? t("playback.none")}
              />
              <Fact
                label={t("playbackSupport.session.hasFailureMessage")}
                value={boolText(session?.has_failure_message)}
              />
              <Fact label={t("playbackSupport.session.active")} value={boolText(session?.active)} />
              <Fact label={t("playbackSupport.session.terminal")} value={boolText(session?.terminal)} />
              <Fact
                label={t("playbackSupport.session.artifact")}
                value={session?.output_artifact_kind ?? t("playback.none")}
              />
              <Fact
                label={t("playbackSupport.session.metrics")}
                value={formatSessionMetrics(session?.runtime_metrics)}
              />
              <Fact
                label={t("playbackSupport.session.timing")}
                value={formatSessionTiming(session)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("playbackSupport.source.description")}
            headerAccessory={
              <Badge tone={source ? (source.has_fingerprint ? "success" : "warning") : "neutral"}>
                {source ? (source.has_fingerprint ? t("playbackSupport.source.fingerprintYes") : t("playbackSupport.source.fingerprintNo")) : t("playback.none")}
              </Badge>
            }
            title={t("playbackSupport.source.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("playbackSupport.source.id")} value={source?.source_id ?? t("playback.none")} />
              <Fact
                label={t("playbackSupport.source.libraryId")}
                value={source?.library_id ?? t("playback.none")}
              />
              <Fact label={t("playbackSupport.source.itemId")} value={source?.item_id ?? t("playback.none")} />
              <Fact
                label={t("playbackSupport.source.scheme")}
                value={source?.source_scheme ?? t("playback.none")}
              />
              <Fact label={t("playbackSupport.source.sizeBytes")} value={formatNullableNumber(source?.size_bytes)} />
              <Fact
                label={t("playbackSupport.source.fingerprint")}
                value={boolText(source?.has_fingerprint)}
              />
            </div>
          </DataPanel>

          <DataPanel
            description={t("playbackSupport.runtime.description")}
            headerAccessory={
              <Badge tone={runtimeTone(support.runtime.readiness.status)}>
                {support.runtime.readiness.status}
              </Badge>
            }
            title={t("playbackSupport.runtime.title")}
          >
            <div className="libraryFactList">
              <Fact
                label={t("playbackSupport.runtime.readiness")}
                value={formatReadiness(support.runtime.readiness.status, support.runtime.readiness.reason)}
              />
              <Fact label={t("playbackSupport.runtime.policy")} value={formatPolicy(support.runtime.policy)} />
              <Fact label={t("playbackSupport.runtime.ffmpeg")} value={formatFfmpeg(support.runtime.ffmpeg)} />
              <Fact label={t("playbackSupport.runtime.hardware")} value={formatHardware(support.runtime.hardware)} />
              <Fact
                label={t("playbackSupport.runtime.transcode")}
                value={formatTranscode(support.runtime.transcode)}
              />
              <Fact
                label={t("playbackSupport.runtime.remux")}
                value={formatRemux(support.runtime.remux)}
              />
              <Fact
                label={t("playbackSupport.runtime.remotePlayback")}
                value={formatRemotePlayback(support.runtime.remote_playback)}
              />
              <Fact label={t("playbackSupport.runtime.staging")} value={formatStaging(support.runtime.staging)} />
              <Fact
                label={t("playbackSupport.runtime.artifactLifecycle")}
                value={formatArtifactLifecycle(support.runtime.artifact_lifecycle)}
              />
              <Fact label={t("playbackSupport.runtime.throttle")} value={formatThrottle(support.runtime.throttle)} />
            </div>
          </DataPanel>

          <DataPanel
            description={t("playbackSupport.redaction.description")}
            headerAccessory={<Badge tone={allRedactionPassed(support.redaction) ? "success" : "danger"}>{t("playbackSupport.redaction.accessory")}</Badge>}
            title={t("playbackSupport.redaction.title")}
          >
            <div className="libraryFactList">
              <Fact label={t("playbackSupport.redaction.paths")} value={boolText(support.redaction.paths_redacted)} />
              <Fact
                label={t("playbackSupport.redaction.sourceReferences")}
                value={boolText(support.redaction.source_references_redacted)}
              />
              <Fact
                label={t("playbackSupport.redaction.ffmpegCommands")}
                value={boolText(support.redaction.ffmpeg_commands_redacted)}
              />
              <Fact label={t("playbackSupport.redaction.stderr")} value={boolText(support.redaction.stderr_redacted)} />
              <Fact
                label={t("playbackSupport.redaction.credentials")}
                value={boolText(support.redaction.credentials_redacted)}
              />
            </div>
          </DataPanel>
        </div>
      ) : null}
    </RoutePage>
  );
}

async function loadPlaybackSupport(
  dataSource: AdminDataSource,
  search: PlaybackSupportSearch,
  unavailableMessage: string,
): Promise<PlaybackSupportResult> {
  if (!dataSource.loadPlaybackSupport) {
    return {
      value: mockPlaybackSupport,
      source: "mock",
      error: unavailableMessage,
    };
  }

  return dataSource.loadPlaybackSupport(search);
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

function sessionTone(state: string) {
  if (state === "failed") {
    return "danger" as const;
  }

  if (state === "running") {
    return "info" as const;
  }

  if (state === "starting") {
    return "warning" as const;
  }

  return "neutral" as const;
}

function runtimeTone(status: string) {
  if (status === "ready") {
    return "success" as const;
  }

  if (status === "degraded") {
    return "warning" as const;
  }

  return "danger" as const;
}

function boolText(value: boolean | null | undefined) {
  if (value === undefined || value === null) {
    return "n/a";
  }

  return value ? "yes" : "no";
}

function formatNullableNumber(value: number | null | undefined) {
  return value === null || value === undefined ? "n/a" : String(value);
}

function formatSessionMetrics(
  metrics: PlaybackSupportSession["runtime_metrics"] | null | undefined,
) {
  if (!metrics) {
    return "n/a";
  }

  return [
    `frames=${formatNullableNumber(metrics.frame_count)}`,
    `fps_millis=${formatNullableNumber(metrics.fps_millis)}`,
    `bitrate_kbps=${formatNullableNumber(metrics.bitrate_kbps)}`,
    `size_bytes=${formatNullableNumber(metrics.total_size_bytes)}`,
    `output_time_ms=${formatNullableNumber(metrics.output_time_ms)}`,
    `dup_frames=${formatNullableNumber(metrics.dup_frames)}`,
    `drop_frames=${formatNullableNumber(metrics.drop_frames)}`,
    `speed_millis=${formatNullableNumber(metrics.speed_millis)}`,
    `progress=${metrics.progress ?? "n/a"}`,
  ].join(" / ");
}

function formatSessionTiming(session: PlaybackSupportSession | null) {
  if (!session) {
    return "n/a";
  }

  return [
    `created=${session.created_at}`,
    `updated=${session.updated_at}`,
    `started=${session.started_at ?? "n/a"}`,
    `completed=${session.completed_at ?? "n/a"}`,
  ].join(" / ");
}

function formatReadiness(status: string, reason: string) {
  return `${status} / ${reason}`;
}

function formatPolicy(policy: AdminPlaybackSupportEvidenceResponse["runtime"]["policy"]) {
  return [
    `user_policy_rows_supported=${boolText(policy.user_policy_rows_supported)}`,
    `role_policy_rows_supported=${boolText(policy.role_policy_rows_supported)}`,
    `effective_resolution_supported=${boolText(policy.effective_resolution_supported)}`,
    `library_access_required=${boolText(policy.library_access_required)}`,
    `user_policy_overrides_role_policy=${boolText(policy.user_policy_overrides_role_policy)}`,
    `role_policy_merge=${policy.role_policy_merge}`,
    `permissions=${policy.permissions.join(", ") || "n/a"}`,
  ].join(" / ");
}

function formatFfmpeg(ffmpeg: AdminPlaybackSupportEvidenceResponse["runtime"]["ffmpeg"]) {
  return [
    `probe_status=${ffmpeg.probe_status}`,
    `has_probe_error=${boolText(ffmpeg.has_probe_error)}`,
    `hardware_capability_count=${ffmpeg.hardware_capability_count}`,
    `available_gpu_capabilities=${ffmpeg.available_gpu_capabilities}`,
  ].join(" / ");
}

function formatHardware(hardware: AdminPlaybackSupportEvidenceResponse["runtime"]["hardware"]) {
  return [
    `selected_acceleration=${hardware.selected_acceleration}`,
    `fallback_used=${boolText(hardware.fallback_used)}`,
    `capability_count=${hardware.capability_count}`,
    `unavailable_capabilities=${hardware.unavailable_capabilities.length}`,
  ].join(" / ");
}

function formatTranscode(transcode: AdminPlaybackSupportEvidenceResponse["runtime"]["transcode"]) {
  return [
    `configured_cpu_slots=${transcode.configured_cpu_slots}`,
    `configured_gpu_slots=${transcode.configured_gpu_slots}`,
    `effective_cpu_slots=${transcode.effective_cpu_slots}`,
    `effective_gpu_slots=${transcode.effective_gpu_slots}`,
    `selected_hls_slots=${transcode.selected_hls_slots}`,
  ].join(" / ");
}

function formatRemux(remux: AdminPlaybackSupportEvidenceResponse["runtime"]["remux"]) {
  return [
    `max_concurrent_sessions=${remux.max_concurrent_sessions}`,
    `timeout_ms=${remux.timeout_ms}`,
  ].join(" / ");
}

function formatRemotePlayback(remotePlayback: AdminPlaybackSupportEvidenceResponse["runtime"]["remote_playback"]) {
  return [
    `backend_count=${remotePlayback.backend_count}`,
    `stream_permits=${remotePlayback.stream_permits_available}/${remotePlayback.stream_permits_max}`,
    `stage_permits=${remotePlayback.stage_permits_available}/${remotePlayback.stage_permits_max}`,
    `state_scope=${remotePlayback.state_scope}`,
  ].join(" / ");
}

function formatStaging(staging: AdminPlaybackSupportEvidenceResponse["runtime"]["staging"]) {
  return [
    `max_bytes=${staging.max_bytes}`,
    `retention_ms=${staging.retention_ms}`,
    `cleanup_on_startup=${boolText(staging.cleanup_on_startup)}`,
    `startup_deleted_records=${staging.startup_deleted_records}`,
    `startup_deleted_files=${staging.startup_deleted_files}`,
  ].join(" / ");
}

function formatArtifactLifecycle(
  artifactLifecycle: AdminPlaybackSupportEvidenceResponse["runtime"]["artifact_lifecycle"],
) {
  return [
    `transcode_artifact_retention_ms=${artifactLifecycle.transcode_artifact_retention_ms}`,
    `transcode_artifact_cleanup_on_startup=${boolText(
      artifactLifecycle.transcode_artifact_cleanup_on_startup,
    )}`,
    `hls_segment_cleanup_enabled=${boolText(artifactLifecycle.hls_segment_cleanup_enabled)}`,
    `hls_segment_keep_ms=${artifactLifecycle.hls_segment_keep_ms}`,
    `deleted_artifacts=${artifactLifecycle.startup_deleted_artifacts}`,
    `deleted_bytes=${artifactLifecycle.startup_deleted_bytes}`,
    `deleted_directories=${artifactLifecycle.startup_deleted_directories}`,
    `deleted_files=${artifactLifecycle.startup_deleted_files}`,
    `examined=${artifactLifecycle.startup_examined_artifacts}`,
    `skipped_security=${artifactLifecycle.startup_skipped_security}`,
  ].join(" / ");
}

function formatThrottle(throttle: AdminPlaybackSupportEvidenceResponse["runtime"]["throttle"]) {
  return [`enabled=${boolText(throttle.enabled)}`, `delay_ms=${throttle.delay_ms}`].join(" / ");
}

function allRedactionPassed(redaction: AdminPlaybackSupportEvidenceResponse["redaction"]) {
  return (
    redaction.paths_redacted &&
    redaction.source_references_redacted &&
    redaction.ffmpeg_commands_redacted &&
    redaction.stderr_redacted &&
    redaction.credentials_redacted
  );
}
