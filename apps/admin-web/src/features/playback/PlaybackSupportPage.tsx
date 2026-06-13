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
import {
  allRedactionPassed,
  boolText,
  formatArtifactLifecycle,
  formatFfmpeg,
  formatHardware,
  formatNullableNumber,
  formatPolicy,
  formatReadiness,
  formatRemotePlayback,
  formatRemux,
  formatSessionMetrics,
  formatSessionTiming,
  formatStaging,
  formatThrottle,
  formatTranscode,
  runtimeTone,
  sessionTone,
} from "./playbackSupportFormatters";

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
  const hasDirectAccessHint = search.session_id === undefined && search.source_id === undefined;

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

      {hasDirectAccessHint ? <RouteNotice>{t("playbackSupport.directAccessNotice")}</RouteNotice> : null}

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
