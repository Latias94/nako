import {
  Activity,
  Boxes,
  Cable,
  CircleAlert,
  Database,
  Film,
  FlaskConical,
  HardDrive,
  Library,
  ListChecks,
  PlayCircle,
  Puzzle,
  Settings,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import type { ComponentType } from "react";
import { useEffect, useMemo, useState } from "react";

import type {
  AdminConsoleData,
  AdminDataSource,
  AdminSourceMap,
  DataSourceMode,
} from "./adminApi/dataSource";
import { mockAdminConsoleData } from "./adminApi/mockData";

type LoadState =
  | { status: "loading"; data: AdminConsoleData }
  | { status: "ready"; data: AdminConsoleData }
  | { status: "fallback"; data: AdminConsoleData; message: string };

type NavItem = {
  label: string;
  id: string;
  icon: ComponentType<{ size?: number }>;
  sourceKey?: keyof AdminSourceMap;
  source: DataSourceMode;
};

const navItems: NavItem[] = [
  { label: "Overview", id: "overview", icon: Activity, sourceKey: "overview", source: "hybrid" },
  { label: "Media Libraries", id: "libraries", icon: Library, source: "mock" },
  { label: "Catalog", id: "catalog", icon: Film, sourceKey: "catalogGovernance", source: "planned" },
  { label: "Intake", id: "intake", icon: Database, sourceKey: "acquisitionIntake", source: "planned" },
  { label: "Metadata", id: "metadata", icon: Sparkles, source: "mock" },
  { label: "Jobs", id: "jobs", icon: ListChecks, sourceKey: "jobs", source: "mock" },
  { label: "Playback", id: "playback", icon: PlayCircle, sourceKey: "playbackRuntime", source: "mock" },
  { label: "Storage", id: "storage", icon: HardDrive, sourceKey: "storageStaging", source: "mock" },
  { label: "Automation", id: "automation", icon: Cable, sourceKey: "events", source: "planned" },
  { label: "Addons", id: "addons", icon: Puzzle, source: "planned" },
  { label: "Network", id: "network", icon: Boxes, sourceKey: "systemConfig", source: "mock" },
  { label: "Settings", id: "settings", icon: Settings, sourceKey: "systemConfig", source: "mock" },
];

export function App({ dataSource }: { dataSource: AdminDataSource }) {
  const [loadState, setLoadState] = useState<LoadState>({
    status: "loading",
    data: mockAdminConsoleData,
  });

  useEffect(() => {
    let mounted = true;

    dataSource
      .load()
      .then((data) => {
        if (!mounted) {
          return;
        }
        setLoadState({ status: "ready", data });
      })
      .catch((error: unknown) => {
        if (!mounted) {
          return;
        }
        const message =
          error instanceof Error
            ? error.message
            : "Admin API is unavailable; using safe mock data.";
        setLoadState({
          status: "fallback",
          data: mockAdminConsoleData,
          message,
        });
      });

    return () => {
      mounted = false;
    };
  }, [dataSource]);

  const sourceCounts = useMemo(() => summarizeSources(loadState.data), [loadState.data]);

  return (
    <div className="appShell">
      <aside className="sidebar" aria-label="Primary navigation">
        <div className="brandBlock">
          <div className="brandMark" aria-hidden="true">
            T
          </div>
          <div>
            <div className="brandName">Taru</div>
            <div className="brandSubline">Admin Console</div>
          </div>
        </div>
        <nav className="navList">
          {navItems.map((item) => {
            const Icon = item.icon;
            const source = item.sourceKey ? loadState.data.sources[item.sourceKey] : item.source;
            return (
              <a className={item.id === "overview" ? "navItem active" : "navItem"} href={`#${item.id}`} key={item.id}>
                <Icon size={17} />
                <span>{item.label}</span>
                <SourceDot source={source} />
              </a>
            );
          })}
        </nav>
      </aside>

      <main className="mainSurface">
        <header className="topBar">
          <div>
            <p className="eyebrow">Private media cellar</p>
            <h1>Server operations and media governance</h1>
          </div>
          <div className="topBarActions">
            <div className="dataSourceMeter" aria-label="Data source summary">
              <span>{sourceCounts.live} live</span>
              <span>{sourceCounts.mock} mock</span>
              <span>{sourceCounts.planned} planned</span>
            </div>
            <button className="primaryButton" type="button">
              <ShieldCheck size={16} />
              Copy diagnostics
            </button>
          </div>
        </header>

        {loadState.status === "fallback" ? (
          <div className="notice" role="status">
            <CircleAlert size={17} />
            <span>{loadState.message}</span>
          </div>
        ) : null}

        {Object.keys(loadState.data.errors).length > 0 ? (
          <div className="notice subtle" role="status">
            <CircleAlert size={17} />
            <span>
              {Object.keys(loadState.data.errors).length} Admin API read models are using safe mock
              data.
            </span>
          </div>
        ) : null}

        <section className="overviewBand" id="overview">
          <div>
            <p className="sectionKicker">Overview</p>
            <h2>Operational readout</h2>
          </div>
          <StatusBadge status={loadState.data.overview.status} />
          <MetricGrid data={loadState.data} />
        </section>

        <div className="contentGrid">
          <section className="panel wide" id="libraries">
            <PanelHeader
              title="Media Libraries"
              source="mock"
              action="Run scan"
              description="Configured collection boundaries and recent maintenance state."
            />
            <div className="tableWrap">
              <table>
                <thead>
                  <tr>
                    <th>Name</th>
                    <th>Backend</th>
                    <th>Status</th>
                    <th>Items</th>
                    <th>Last scan</th>
                  </tr>
                </thead>
                <tbody>
                  {loadState.data.libraries.map((library) => (
                    <tr key={library.id}>
                      <td>{library.name}</td>
                      <td>{library.backendKind}</td>
                      <td>
                        <StatusPill label={library.status} tone={library.status === "ready" ? "good" : "warn"} />
                      </td>
                      <td>{library.itemCount.toLocaleString()}</td>
                      <td>{library.lastScan}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className="panel" id="metadata">
            <PanelHeader
              title="Metadata Providers"
              source={loadState.data.sources.overview}
              description="Provider availability and local authority health."
            />
            <div className="stackList">
              {loadState.data.overview.metadata.providers.map((provider) => (
                <div className="listRow" key={provider.provider}>
                  <div>
                    <strong>{provider.provider.toUpperCase()}</strong>
                    <span>Provider Mapping</span>
                  </div>
                  <StatusPill
                    label={provider.status}
                    tone={provider.status === "available" ? "good" : provider.status === "disabled" ? "muted" : "bad"}
                  />
                </div>
              ))}
              {loadState.data.overview.metadata.providers.length === 0 ? (
                <div className="emptyState">No provider runtime diagnostics are reported yet.</div>
              ) : null}
            </div>
          </section>

          <section className="panel" id="jobs">
            <PanelHeader
              title="Jobs"
              source={loadState.data.sources.jobs}
              action="Open queue"
              description="Durable jobs and cancellable runtime work."
            />
            <div className="stackList">
              {loadState.data.jobs.map((job) => (
                <div className="listRow" key={job.id}>
                  <div>
                    <strong>{job.kind}</strong>
                    <span>{job.resourceClass}</span>
                  </div>
                  <StatusPill
                    label={job.status}
                    tone={job.hasError || job.status === "failed" ? "bad" : job.status === "running" ? "info" : "good"}
                  />
                </div>
              ))}
            </div>
          </section>

          <section className="panel wide" id="catalog">
            <PanelHeader
              title="Catalog Governance"
              source={loadState.data.sources.catalogGovernance}
              description="Unknown and low-confidence Media Items needing operator review."
            />
            <div className="tableWrap">
              <table>
                <thead>
                  <tr>
                    <th>Media Item</th>
                    <th>Kind</th>
                    <th>Issues</th>
                    <th>Sources</th>
                    <th>Provider mappings</th>
                  </tr>
                </thead>
                <tbody>
                  {loadState.data.catalog.items.map((item) => (
                    <tr key={item.id}>
                      <td>{item.title}</td>
                      <td>{item.kind}</td>
                      <td>{item.issues.length > 0 ? item.issues.join(", ") : "none"}</td>
                      <td>{item.sourceCount}</td>
                      <td>{item.providerMappingCount}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className="panel wide" id="intake">
            <PanelHeader
              title="Acquisition Intake"
              source={loadState.data.sources.acquisitionIntake}
              description="Watch-folder candidates staged before Managed Import and promotion apply."
            />
            <div className="tableWrap">
              <table>
                <thead>
                  <tr>
                    <th>Candidate</th>
                    <th>Source</th>
                    <th>State</th>
                    <th>Size</th>
                    <th>Diagnostics</th>
                    <th>Managed Import</th>
                  </tr>
                </thead>
                <tbody>
                  {loadState.data.acquisitionIntake.candidates.map((candidate) => (
                    <tr key={candidate.id}>
                      <td>{candidate.id}</td>
                      <td>
                        {candidate.sourceKind} · {candidate.sourceScheme}
                      </td>
                      <td>{candidate.state}</td>
                      <td>{candidate.sizeBytes ?? "unknown"}</td>
                      <td>{candidate.hasDiagnostics ? "available" : "none"}</td>
                      <td>{candidate.linkedArtifactId ?? "not linked"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className="panel wide" id="playback">
            <PanelHeader
              title="Playback & Transcode"
              source={combinedSource(
                loadState.data.sources.playbackSessions,
                loadState.data.sources.playbackRuntime,
              )}
              description="Session state, hardware acceleration, and staging budgets."
            />
            <div className="splitPanel">
              <div>
                <h3>Active sessions</h3>
                <div className="stackList">
                  {loadState.data.playback.sessions.map((session) => (
                    <div className="listRow" key={session.id}>
                      <div>
                        <strong>{session.kind}</strong>
                        <span>{session.sourceTitle}</span>
                      </div>
                      <StatusPill label={session.state} tone={session.state === "running" ? "info" : "muted"} />
                    </div>
                  ))}
                </div>
              </div>
              <div className="runtimeCard">
                <FlaskConical size={18} />
                <h3>Hardware policy</h3>
                <p>{loadState.data.playback.hardwarePolicy}</p>
                <p className="runtimeNote">FFmpeg probe: {loadState.data.playback.ffmpegStatus}</p>
                <div className="capabilityLine">
                  {loadState.data.playback.accelerators.map((accelerator) => (
                    <StatusPill
                      key={accelerator.name}
                      label={accelerator.name}
                      tone={accelerator.available ? "good" : "muted"}
                    />
                  ))}
                </div>
              </div>
            </div>
          </section>

          <section className="panel" id="storage">
            <PanelHeader
              title="Storage"
              source={combinedSource(loadState.data.sources.overview, loadState.data.sources.storageStaging)}
              description="Backend and staging status without root paths or Source Locators."
            />
            <div className="stackList">
              {loadState.data.overview.storage.backends.map((backend) => (
                <div className="listRow" key={backend.library_id}>
                  <div>
                    <strong>{backend.library_name}</strong>
                    <span>{backend.backend_kind}</span>
                  </div>
                  <StatusPill label={backend.status} tone={backend.status === "ready" ? "good" : "bad"} />
                </div>
              ))}
              {loadState.data.storage.records.map((record) => (
                <div className="listRow" key={record.id}>
                  <div>
                    <strong>{record.sourceScheme} staging</strong>
                    <span>{record.purpose}</span>
                  </div>
                  <StatusPill label={record.state} tone={record.hasValidationError ? "bad" : "info"} />
                </div>
              ))}
            </div>
          </section>

          <section className="panel" id="automation">
            <PanelHeader
              title="Automation Events"
              source={loadState.data.sources.events}
              description="Redacted event outbox history for webhooks and automation."
            />
            <div className="stackList">
              {loadState.data.events.events.map((event) => (
                <div className="listRow" key={event.id}>
                  <div>
                    <strong>{event.kind}</strong>
                    <span>{event.attempts} attempts</span>
                  </div>
                  <StatusPill label={event.status} tone={event.hasError ? "bad" : "good"} />
                </div>
              ))}
            </div>
          </section>

          <section className="panel" id="network">
            <PanelHeader
              title="Network Access"
              source={loadState.data.sources.systemConfig}
              description="Remote exposure readiness without URLs, headers, credentials, or local paths."
            />
            <div className="stackList">
              <div className="listRow">
                <div>
                  <strong>{loadState.data.network.exposureMode}</strong>
                  <span>{loadState.data.network.readinessReason}</span>
                </div>
                <StatusPill
                  label={loadState.data.network.readinessStatus}
                  tone={
                    loadState.data.network.readinessStatus === "ready"
                      ? "good"
                      : loadState.data.network.readinessStatus === "degraded"
                        ? "warn"
                        : "bad"
                  }
                />
              </div>
              <div className="listRow">
                <div>
                  <strong>
                    {loadState.data.network.endpointConfigured ? "External endpoint set" : "No external endpoint"}
                  </strong>
                  <span>{loadState.data.network.endpointScheme ?? "no scheme"}</span>
                </div>
                <StatusPill
                  label={loadState.data.network.endpointConfigured ? "configured" : "not configured"}
                  tone={loadState.data.network.endpointConfigured ? "good" : "muted"}
                />
              </div>
              <div className="listRow">
                <div>
                  <strong>
                    {loadState.data.network.trustedProxySourceCount} trusted proxy sources
                  </strong>
                  <span>
                    {loadState.data.network.allowedOriginCount} browser origins ·{" "}
                    {loadState.data.network.tunnelProviderCount} tunnel providers
                  </span>
                </div>
                <StatusPill
                  label={loadState.data.network.trustedProxyHeaders ? "forwarded headers trusted" : "default deny"}
                  tone={loadState.data.network.trustedProxyHeaders ? "info" : "muted"}
                />
              </div>
            </div>
          </section>

          <section className="panel" id="settings">
            <PanelHeader
              title="Settings"
              source={loadState.data.sources.systemConfig}
              description="Read-only diagnostics until mutation routes are designed."
            />
            <div className="settingGrid">
              {loadState.data.settings.map((setting) => (
                <div className="settingRow" key={setting.label}>
                  <span>{setting.label}</span>
                  <strong>{setting.value}</strong>
                </div>
              ))}
            </div>
          </section>
        </div>
      </main>
    </div>
  );
}

function MetricGrid({ data }: { data: AdminConsoleData }) {
  const { overview } = data;
  const metrics = [
    {
      label: "Storage backends",
      value: `${overview.storage.ready_backends}/${overview.storage.total_backends}`,
      detail: `${overview.storage.degraded_backends} degraded`,
    },
    {
      label: "Metadata providers",
      value: `${overview.metadata.available_providers}/${overview.metadata.total_providers}`,
      detail: `${overview.metadata.disabled_providers} disabled`,
    },
    {
      label: "Active tasks",
      value: overview.runtime.active_tasks.toString(),
      detail: `${overview.runtime.failed_tasks} failed tasks`,
    },
    {
      label: "Startup recovery",
      value: overview.startup.recovered_jobs.toString(),
      detail: `${overview.startup.configured_libraries} configured libraries`,
    },
  ];

  return (
    <div className="metricGrid">
      {metrics.map((metric) => (
        <article className="metricTile" key={metric.label}>
          <span>{metric.label}</span>
          <strong>{metric.value}</strong>
          <small>{metric.detail}</small>
        </article>
      ))}
    </div>
  );
}

function PanelHeader({
  title,
  description,
  source,
  action,
}: {
  title: string;
  description: string;
  source: DataSourceMode;
  action?: string;
}) {
  return (
    <div className="panelHeader">
      <div>
        <div className="panelTitleLine">
          <h2>{title}</h2>
          <SourceLabel source={source} />
        </div>
        <p>{description}</p>
      </div>
      {action ? (
        <button className="secondaryButton" type="button">
          {action}
        </button>
      ) : null}
    </div>
  );
}

function StatusBadge({ status }: { status: "healthy" | "degraded" }) {
  return (
    <div className={status === "healthy" ? "statusBadge healthy" : "statusBadge degraded"}>
      {status === "healthy" ? "Healthy" : "Degraded"}
    </div>
  );
}

function StatusPill({
  label,
  tone,
}: {
  label: string;
  tone: "good" | "warn" | "bad" | "info" | "muted";
}) {
  return <span className={`statusPill ${tone}`}>{label}</span>;
}

function SourceLabel({ source }: { source: DataSourceMode }) {
  const label =
    source === "live"
      ? "Live Admin API"
      : source === "hybrid"
        ? "Live + mock"
        : source === "mock"
          ? "Mock data"
          : "Planned";
  return <span className={`sourceLabel ${source}`}>{label}</span>;
}

function SourceDot({ source }: { source: DataSourceMode }) {
  return <span className={`sourceDot ${source}`} aria-label={`${source} data`} />;
}

function summarizeSources(data: AdminConsoleData) {
  const sources: DataSourceMode[] = [
    data.sources.overview,
    "mock",
    data.sources.catalogGovernance,
    data.sources.acquisitionIntake,
    "mock",
    data.sources.jobs,
    combinedSource(data.sources.playbackSessions, data.sources.playbackRuntime),
    combinedSource(data.sources.overview, data.sources.storageStaging),
    data.sources.events,
    "planned",
    data.sources.systemConfig,
    data.sources.systemConfig,
  ];

  return sources.reduce(
    (counts, source) => {
      if (source === "live") {
        counts.live += 1;
      } else if (source === "hybrid" || source === "mock") {
        counts.mock += 1;
      } else {
        counts.planned += 1;
      }
      return counts;
    },
    { live: 0, mock: 0, planned: 0 },
  );
}

function combinedSource(...sources: DataSourceMode[]): DataSourceMode {
  if (sources.every((source) => source === "live")) {
    return "live";
  }

  if (sources.some((source) => source === "live")) {
    return "hybrid";
  }

  if (sources.every((source) => source === "planned")) {
    return "planned";
  }

  return "mock";
}
