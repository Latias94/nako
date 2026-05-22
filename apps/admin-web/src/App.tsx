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
import type {
  AddonGrantAssignmentInput,
  AddonManifestPreview,
  AddonOnboardingResult,
  AddonTokenActionResult,
} from "./adminApi/types";
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
  { label: "Automation", id: "automation", icon: Cable, sourceKey: "generatedArtifactProposals", source: "planned" },
  { label: "Addons", id: "addons", icon: Puzzle, source: "planned" },
  { label: "Network", id: "network", icon: Boxes, sourceKey: "systemConfig", source: "mock" },
  { label: "Settings", id: "settings", icon: Settings, sourceKey: "systemConfig", source: "mock" },
];

export function App({ dataSource }: { dataSource: AdminDataSource }) {
  const [loadState, setLoadState] = useState<LoadState>({
    status: "loading",
    data: mockAdminConsoleData,
  });
  const [addonActionMessage, setAddonActionMessage] = useState<string | null>(null);
  const [manifestJson, setManifestJson] = useState("");
  const [manifestPreview, setManifestPreview] = useState<AddonManifestPreview | null>(null);
  const [onboardingResult, setOnboardingResult] = useState<AddonOnboardingResult | null>(null);
  const [tokenLabel, setTokenLabel] = useState("sidecar runtime");
  const [grantPermission, setGrantPermission] = useState<AddonGrantAssignmentInput["permission"]>("metadata_write");
  const [oneTimeToken, setOneTimeToken] = useState<AddonTokenActionResult | null>(null);

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
  const selectedAddon = loadState.data.addons.selectedAddon;

  const runAddonStatusAction = async (status: "enabled" | "disabled") => {
    if (!selectedAddon || !dataSource.setAddonStatus) {
      return;
    }

    const addons = await dataSource.setAddonStatus(selectedAddon.id, status);
    setLoadState((current) => ({ ...current, data: { ...current.data, addons } }));
    setAddonActionMessage(`${selectedAddon.name} ${status}`);
  };

  const runAddonHealthCheck = async () => {
    if (!selectedAddon || !dataSource.checkAddonHealth) {
      return;
    }

    const health = await dataSource.checkAddonHealth(selectedAddon.id);
    setLoadState((current) => ({
      ...current,
      data: { ...current.data, addons: { ...current.data.addons, health } },
    }));
    setAddonActionMessage(`${selectedAddon.name} health ${health.status}`);
  };

  const runAddonDiagnostic = async () => {
    if (!selectedAddon || !dataSource.diagnoseAddonResource) {
      return;
    }

    const resource = selectedAddon.resourceKinds[0] ?? "metadata";
    const diagnostic = await dataSource.diagnoseAddonResource(selectedAddon.id, resource);
    setLoadState((current) => ({
      ...current,
      data: { ...current.data, addons: { ...current.data.addons, diagnostic } },
    }));
    setAddonActionMessage(`${selectedAddon.name} diagnostic ${diagnostic.status}`);
  };

  const updateManifestJson = (value: string) => {
    setManifestJson(value);
    setOnboardingResult(null);
    if (!value.trim()) {
      setManifestPreview(null);
      return;
    }

    setManifestPreview(
      dataSource.previewAddonManifestJson?.(value) ?? {
        status: "invalid_json",
        error: "Manifest preview is unavailable.",
      },
    );
  };

  const registerAddonManifest = async () => {
    if (!dataSource.registerAddonManifestJson || !manifestJson.trim()) {
      return;
    }

    const result = await dataSource.registerAddonManifestJson(manifestJson);
    setOnboardingResult(result);
    if (result.status === "registered") {
      setAddonActionMessage(`${result.addon.name} registered as ${result.addon.status}`);
    }
  };

  const issueAddonToken = async () => {
    if (!selectedAddon || !dataSource.issueAddonToken) {
      return;
    }
    const result = await dataSource.issueAddonToken(selectedAddon.id, tokenLabel);
    setOneTimeToken(result);
    setLoadState((current) => ({
      ...current,
      data: {
        ...current.data,
        addons: {
          ...current.data.addons,
          tokens: [result.token, ...current.data.addons.tokens],
        },
      },
    }));
  };

  const rotateFirstAddonToken = async () => {
    if (!selectedAddon || !dataSource.rotateAddonToken || !loadState.data.addons.tokens[0]) {
      return;
    }
    const result = await dataSource.rotateAddonToken(
      selectedAddon.id,
      loadState.data.addons.tokens[0].id,
      `${tokenLabel} rotated`,
    );
    setOneTimeToken(result);
    setLoadState((current) => ({
      ...current,
      data: {
        ...current.data,
        addons: {
          ...current.data.addons,
          tokens: [result.token, ...current.data.addons.tokens],
        },
      },
    }));
  };

  const revokeFirstAddonToken = async () => {
    if (!selectedAddon || !dataSource.revokeAddonToken || !loadState.data.addons.tokens[0]) {
      return;
    }
    const revoked = await dataSource.revokeAddonToken(selectedAddon.id, loadState.data.addons.tokens[0].id);
    setLoadState((current) => ({
      ...current,
      data: {
        ...current.data,
        addons: {
          ...current.data.addons,
          tokens: current.data.addons.tokens.map((token) => (token.id === revoked.id ? revoked : token)),
        },
      },
    }));
    setAddonActionMessage(`Token ${revoked.id} revoked`);
  };

  const replaceAddonGrants = async () => {
    if (!selectedAddon || !dataSource.replaceAddonGrants) {
      return;
    }
    const grants = await dataSource.replaceAddonGrants(selectedAddon.id, [
      {
        permission: grantPermission,
        libraryId: null,
      },
    ]);
    setLoadState((current) => ({
      ...current,
      data: {
        ...current.data,
        addons: {
          ...current.data.addons,
          grants,
        },
      },
    }));
  };

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
              title="Generated Artifacts"
              source={loadState.data.sources.generatedArtifactProposals}
              description="AI-assisted proposals with prompt and payload content reduced to fingerprints and readiness."
            />
            <div className="stackList">
              {loadState.data.generatedArtifactProposals.proposals.map((proposal) => (
                <div className="listRow" key={proposal.id}>
                  <div>
                    <strong>{proposal.capability}</strong>
                    <span>
                      {proposal.targetKind} · {proposal.providerName ?? "unknown provider"} ·{" "}
                      {proposal.payloadShape}
                    </span>
                  </div>
                  <StatusPill
                    label={proposal.readinessStatus}
                    tone={
                      proposal.readinessStatus === "ready"
                        ? "good"
                        : proposal.readinessStatus === "stale"
                          ? "warn"
                          : "bad"
                    }
                  />
                </div>
              ))}
            </div>
          </section>

          <section className="panel wide" id="addons">
            <PanelHeader
              title="Addon Operations"
              source={combinedSource(
                loadState.data.sources.addons,
                loadState.data.sources.addonHealth,
                loadState.data.sources.addonSurfaces,
                loadState.data.sources.addonInstallGuide,
                loadState.data.sources.addonTokens,
                loadState.data.sources.addonGrants,
              )}
              description="Manage Addon Sidecars without installing, launching, or trusting their processes."
            />
            <section className="addonOnboardingPanel" aria-label="Addon Onboarding">
              <div>
                <h3>Addon Onboarding</h3>
                <p>
                  Paste an Addon manifest JSON document to register the Addon
                  as disabled. Registration does not install or start the
                  sidecar.
                </p>
              </div>
              <div className="onboardingGrid">
                <label className="manifestEditor">
                  <span>Addon manifest JSON</span>
                  <textarea
                    aria-label="Addon manifest JSON"
                    value={manifestJson}
                    onChange={(event) => updateManifestJson(event.target.value)}
                    placeholder='{"id":"dev.taru.example","protocol_version":"2026-05-15","base_url":"http://example-addon:9100"}'
                    rows={9}
                  />
                </label>
                <div className="manifestPreviewCard">
                  <strong>Manifest preview</strong>
                  {manifestPreview?.status === "ready" && manifestPreview.summary ? (
                    <>
                      <p>{manifestPreview.summary.name}</p>
                      <span>
                        {manifestPreview.summary.manifestId} · {manifestPreview.summary.resourceCount} resources
                      </span>
                      <div className="capabilityLine">
                        <StatusPill label={manifestPreview.summary.protocolVersion} tone="info" />
                        <StatusPill label={`${manifestPreview.summary.declaredScopes.length} scopes`} tone="muted" />
                        <StatusPill label={`${manifestPreview.summary.secretReferenceCount} secrets`} tone="muted" />
                      </div>
                    </>
                  ) : (
                    <p>{manifestPreview?.error ?? "Paste manifest JSON to preview registration facts."}</p>
                  )}
                  <button
                    className="secondaryButton"
                    disabled={
                      manifestPreview?.status !== "ready" ||
                      !dataSource.registerAddonManifestJson
                    }
                    onClick={registerAddonManifest}
                    type="button"
                  >
                    Register disabled Addon
                  </button>
                </div>
              </div>
              {onboardingResult?.status === "registered" ? (
                <div className="notice subtle" role="status">
                  <ShieldCheck size={17} />
                  <div>
                    <strong>
                      {onboardingResult.addon.name} registered as {onboardingResult.addon.status}
                    </strong>
                    <ul>
                      {onboardingResult.nextSteps.map((step) => (
                        <li key={step}>{step}</li>
                      ))}
                    </ul>
                  </div>
                </div>
              ) : null}
              {onboardingResult?.status === "invalid_json" || onboardingResult?.status === "server_error" ? (
                <div className="notice warning" role="alert">
                  <CircleAlert size={17} />
                  <span>{onboardingResult.error}</span>
                </div>
              ) : null}
            </section>
            <div className="splitPanel">
              <div>
                <h3>Registered Addons</h3>
                <div className="stackList">
                  {loadState.data.addons.addons.map((addon) => (
                    <div className="listRow" key={addon.id}>
                      <div>
                        <strong>{addon.name}</strong>
                        <span>
                          {addon.manifestId} · {addon.protocolVersion}
                        </span>
                      </div>
                      <StatusPill
                        label={addon.status}
                        tone={
                          addon.status === "enabled"
                            ? "good"
                            : addon.status === "disabled"
                              ? "warn"
                              : "muted"
                        }
                      />
                    </div>
                  ))}
                </div>
              </div>

              <div className="runtimeCard">
                <Puzzle size={18} />
                <h3>{loadState.data.addons.selectedAddon?.name ?? "No Addon selected"}</h3>
                <p>{loadState.data.addons.selectedAddon?.description ?? "Select an Addon to inspect its operations."}</p>
                <p className="runtimeNote">
                  Health: {loadState.data.addons.health?.status ?? "unknown"} ·{" "}
                  {loadState.data.addons.health?.latencyMs ?? 0} ms
                </p>
                <div className="capabilityLine">
                  <StatusPill
                    label={`${loadState.data.addons.selectedAddon?.resourceCount ?? 0} resources`}
                    tone="info"
                  />
                  <StatusPill label={`${loadState.data.addons.grants.length} grants`} tone="info" />
                  <StatusPill label={`${loadState.data.addons.tokens.length} tokens`} tone="muted" />
                </div>
              </div>
            </div>

            <div className="capabilityLine">
              <button
                className="secondaryButton"
                disabled={!selectedAddon || !dataSource.setAddonStatus}
                onClick={() => runAddonStatusAction("enabled")}
                type="button"
              >
                Enable Addon
              </button>
              <button
                className="secondaryButton"
                disabled={!selectedAddon || !dataSource.setAddonStatus}
                onClick={() => runAddonStatusAction("disabled")}
                type="button"
              >
                Disable Addon
              </button>
              <button
                className="secondaryButton"
                disabled={!selectedAddon || !dataSource.checkAddonHealth}
                onClick={runAddonHealthCheck}
                type="button"
              >
                Run health check
              </button>
              <button
                className="secondaryButton"
                disabled={!selectedAddon || !dataSource.diagnoseAddonResource}
                onClick={runAddonDiagnostic}
                type="button"
              >
                Run resource diagnostic
              </button>
            </div>
            {addonActionMessage ? (
              <div className="notice subtle" role="status">
                <ShieldCheck size={17} />
                <span>{addonActionMessage}</span>
              </div>
            ) : null}

            <section className="credentialGrantPanel" aria-label="Addon Credentials and Grants">
              <div>
                <h3>Addon Credentials & Grants</h3>
                <p>
                  Create one-time Addon Tokens and accepted grants before enabling the sidecar.
                  Raw tokens are shown only immediately after issue or rotation.
                </p>
              </div>
              <div className="credentialGrid">
                <div className="credentialCard">
                  <h4>Addon Tokens</h4>
                  <label className="compactField">
                    <span>Addon token label</span>
                    <input
                      aria-label="Addon token label"
                      value={tokenLabel}
                      onChange={(event) => setTokenLabel(event.target.value)}
                    />
                  </label>
                  <div className="capabilityLine">
                    <button
                      className="secondaryButton"
                      disabled={!selectedAddon || !dataSource.issueAddonToken}
                      onClick={issueAddonToken}
                      type="button"
                    >
                      Issue token
                    </button>
                    <button
                      className="secondaryButton"
                      disabled={!selectedAddon || !dataSource.rotateAddonToken || loadState.data.addons.tokens.length === 0}
                      onClick={rotateFirstAddonToken}
                      type="button"
                    >
                      Rotate first token
                    </button>
                    <button
                      className="secondaryButton"
                      disabled={!selectedAddon || !dataSource.revokeAddonToken || loadState.data.addons.tokens.length === 0}
                      onClick={revokeFirstAddonToken}
                      type="button"
                    >
                      Revoke first token
                    </button>
                  </div>
                  {oneTimeToken ? (
                    <div className="oneTimeSecret" role="status">
                      <strong>Copy this Addon Token now. It will not be shown again.</strong>
                      <code>{oneTimeToken.rawToken}</code>
                      <span>
                        Saved summary: {oneTimeToken.token.label} · {oneTimeToken.token.tokenPrefix}
                      </span>
                    </div>
                  ) : null}
                  <div className="stackList">
                    {loadState.data.addons.tokens.map((token) => (
                      <div className="listRow" key={token.id}>
                        <div>
                          <strong>{token.label}</strong>
                          <span>{token.id} · {token.tokenPrefix}</span>
                        </div>
                        <StatusPill label={token.status} tone={token.status === "active" ? "good" : "muted"} />
                      </div>
                    ))}
                  </div>
                </div>

                <div className="credentialCard">
                  <h4>Accepted Grants</h4>
                  <label className="compactField">
                    <span>Addon grant permission</span>
                    <select
                      aria-label="Addon grant permission"
                      value={grantPermission}
                      onChange={(event) =>
                        setGrantPermission(event.target.value as AddonGrantAssignmentInput["permission"])
                      }
                    >
                      <option value="metadata_write">metadata_write</option>
                      <option value="artwork_write">artwork_write</option>
                      <option value="subtitle_write">subtitle_write</option>
                      <option value="library_file_write">library_file_write</option>
                    </select>
                  </label>
                  <button
                    className="secondaryButton"
                    disabled={!selectedAddon || !dataSource.replaceAddonGrants}
                    onClick={replaceAddonGrants}
                    type="button"
                  >
                    Replace grants
                  </button>
                  <div className="stackList">
                    {loadState.data.addons.grants.map((grant) => (
                      <div className="listRow" key={grant.id}>
                        <div>
                          <strong>{grant.permission} · {grant.libraryId ?? "global"}</strong>
                          <span>{grant.id}</span>
                        </div>
                        <StatusPill label="accepted" tone="info" />
                      </div>
                    ))}
                  </div>
                </div>

                <div className="credentialCard readinessCard">
                  <h4>Enable readiness</h4>
                  <ReadinessItem
                    label="Manifest registered"
                    ready={Boolean(selectedAddon)}
                  />
                  <ReadinessItem
                    label="Health reachable or checked"
                    ready={loadState.data.addons.health?.status === "reachable"}
                  />
                  <ReadinessItem
                    label="Active token available"
                    ready={loadState.data.addons.tokens.some((token) => token.status === "active")}
                  />
                  <ReadinessItem
                    label="Accepted grants configured"
                    ready={loadState.data.addons.grants.length > 0}
                  />
                  <ReadinessItem
                    label="Sidecar lifecycle remains external"
                    ready
                  />
                </div>
              </div>
            </section>

            <div className="tableWrap">
              <table>
                <thead>
                  <tr>
                    <th>Surface</th>
                    <th>Kind</th>
                    <th>Path</th>
                    <th>Safety</th>
                  </tr>
                </thead>
                <tbody>
                  {loadState.data.addons.surfaces?.entryPoints.map((entryPoint) => (
                    <tr key={entryPoint.id}>
                      <td>{entryPoint.label}</td>
                      <td>{entryPoint.kind}</td>
                      <td>{entryPoint.path}</td>
                      <td>{entryPoint.hostedPageId ? "external hosted page" : "Taru action"}</td>
                    </tr>
                  ))}
                  {loadState.data.addons.surfaces?.hostedPages.map((page) => (
                    <tr key={page.id}>
                      <td>{page.title}</td>
                      <td>hosted_page</td>
                      <td>{page.path}</td>
                      <td>external and untrusted</td>
                    </tr>
                  ))}
                  {loadState.data.addons.surfaces?.tasks.map((task) => (
                    <tr key={task.id}>
                      <td>{task.name}</td>
                      <td>addon_task</td>
                      <td>{task.path}</td>
                      <td>declaration only</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>

            <div className="settingGrid">
              <div className="settingRow">
                <span>Resource diagnostic</span>
                <strong>
                  {loadState.data.addons.diagnostic?.resource ?? "none"} ·{" "}
                  {loadState.data.addons.diagnostic?.status ?? "not run"}
                </strong>
              </div>
              <div className="settingRow">
                <span>Diagnostic attempts</span>
                <strong>
                  {loadState.data.addons.diagnostic?.attempts ?? 0}
                  {loadState.data.addons.diagnostic?.httpStatus
                    ? ` · HTTP ${loadState.data.addons.diagnostic.httpStatus}`
                    : ""}
                </strong>
              </div>
              <div className="settingRow">
                <span>Secret references</span>
                <strong>{loadState.data.addons.surfaces?.secretReferenceFieldCount ?? 0} configured fields</strong>
              </div>
              <div className="settingRow">
                <span>Configuration schema</span>
                <strong>{loadState.data.addons.surfaces?.configurationSchemaId ?? "not declared"}</strong>
              </div>
            </div>

            <section className="installGuidePanel" aria-label="Addon Install Guide">
              <div>
                <h3>Addon Install Guide</h3>
                <p>
                  {loadState.data.addons.installGuide?.lifecycleBoundary.message ??
                    "Install guide is unavailable for this Addon."}
                </p>
              </div>
              <div className="capabilityLine">
                <StatusPill
                  label={
                    loadState.data.addons.installGuide?.lifecycleBoundary.taruManagesContainers
                      ? "Taru controls containers"
                      : "No container control"
                  }
                  tone="muted"
                />
                <StatusPill
                  label={
                    loadState.data.addons.installGuide?.lifecycleBoundary.taruManagesProcesses
                      ? "Taru controls processes"
                      : "No process control"
                  }
                  tone="muted"
                />
                <StatusPill
                  label={`${loadState.data.addons.installGuide?.secretReferences.length ?? 0} Secret References`}
                  tone="info"
                />
              </div>

              {loadState.data.addons.installGuide ? (
                <>
                  <div className="snippetGrid">
                    <SnippetPreview snippet={loadState.data.addons.installGuide.dockerCompose} />
                    <SnippetPreview snippet={loadState.data.addons.installGuide.systemd} />
                  </div>

                  <div className="guideStepGrid">
                    <GuideStepList
                      title="Health-check verification"
                      steps={loadState.data.addons.installGuide.healthCheckSteps}
                    />
                    <GuideStepList
                      title="Registration verification"
                      steps={loadState.data.addons.installGuide.registrationVerificationSteps}
                    />
                  </div>

                  <div className="tableWrap">
                    <table>
                      <thead>
                        <tr>
                          <th>Secret Reference</th>
                          <th>Env var</th>
                          <th>Placeholder</th>
                          <th>Required</th>
                        </tr>
                      </thead>
                      <tbody>
                        {loadState.data.addons.installGuide.secretReferences.map((secret) => (
                          <tr key={secret.id}>
                            <td>{secret.label}</td>
                            <td>{secret.envVar}</td>
                            <td>{secret.placeholder}</td>
                            <td>{secret.required ? "required" : "optional"}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </>
              ) : null}
            </section>
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

function SnippetPreview({
  snippet,
}: {
  snippet: {
    title: string;
    filename: string;
    content: string;
    notes: string[];
  };
}) {
  return (
    <article className="snippetPreview">
      <div className="snippetHeader">
        <strong>{snippet.title}</strong>
        <span>{snippet.filename}</span>
      </div>
      <pre>{snippet.content}</pre>
      <ul>
        {snippet.notes.map((note) => (
          <li key={note}>{note}</li>
        ))}
      </ul>
    </article>
  );
}

function GuideStepList({
  title,
  steps,
}: {
  title: string;
  steps: Array<{
    title: string;
    command: string;
    expectedResult: string;
  }>;
}) {
  return (
    <article className="guideSteps">
      <h4>{title}</h4>
      <ol>
        {steps.map((step) => (
          <li key={step.title}>
            <strong>{step.title}</strong>
            <code>{step.command}</code>
            <span>{step.expectedResult}</span>
          </li>
        ))}
      </ol>
    </article>
  );
}

function ReadinessItem({ label, ready }: { label: string; ready: boolean }) {
  return (
    <div className="readinessItem">
      <StatusPill label={ready ? "ready" : "needs action"} tone={ready ? "good" : "warn"} />
      <span>{label}</span>
    </div>
  );
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
    data.sources.generatedArtifactProposals,
    "planned",
    data.sources.events,
    combinedSource(
      data.sources.addons,
      data.sources.addonHealth,
      data.sources.addonSurfaces,
      data.sources.addonInstallGuide,
      data.sources.addonTokens,
      data.sources.addonGrants,
    ),
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
