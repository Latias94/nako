# Nako Goal Map

This file is the top-level tracker for current and upcoming engineering goals.
Workstream TODO files track tasks; this file tracks goal boundaries,
non-goals, exit criteria, and evidence.

## Goal Format

Each implementation goal should define:

- Objective: the user-visible or architecture-visible outcome.
- Deliverables: concrete files, APIs, crates, or documents expected to change.
- Non-goals: adjacent work intentionally left out.
- Exit criteria: observable behavior that must be true.
- Evidence: commands, tests, docs, or commit IDs proving completion.

Use one goal per meaningful milestone. A goal should be large enough to produce
a coherent commit, but small enough that validation remains clear.

## Numbering Policy

Goal numbers are historical identifiers, not dense release numbers. Do not
reuse earlier gaps such as M10-M12 or M17 for new work. New implementation
goals should use the next number after the highest documented completed or
proposed milestone.

## Current Goal

### No Active Implementation Goal

Status: idle after `generated-artifact-bulk-metadata-apply` closeout on
2026-06-01.

Next planner action:

- choose exactly one focused follow-on before starting more parallel
  implementation;
- preferred candidates are provider-specific Generated Artifact mapping
  breadth, Generated Artifact apply operations repair, Admin settings
  API-backed restoration, or a broader architecture planning pass if priorities
  changed.

## Recent Completed Goals

### Generated Artifact Bulk Metadata Apply

Status: completed on 2026-06-01.

Objective:

- Extend the shipped one-artifact Generated Artifact Metadata Authority apply
  workflow into a guarded bulk workflow for accepted metadata Generated
  Artifacts.
- Keep bulk planning read-only before mutation, then add durable batch
  execution with per-item idempotency and partial-failure reporting.
- Expose the workflow safely through Admin API and Web without leaking raw
  payloads, prompts, Source Locators, paths, tokens, or secrets.

Deliverables:

- `docs/workstreams/generated-artifact-bulk-metadata-apply/`
- bulk apply-plan Admin API contract and route
- durable bulk apply request/job persistence and execution
- Admin bulk status/result read models
- Web Admin bulk plan, confirm, and result workflow

Non-goals:

- no provider-specific Generated Artifact mapping breadth;
- no outcome repair/search tooling beyond batch result display;
- no change to Generated Artifact review acceptance semantics;
- no automatic application of newly accepted artifacts;
- no Public Client API changes;
- no Admin settings restoration.

Exit criteria:

- bulk plan route is read-only and redacted;
- confirmed bulk apply runs durably outside the request path;
- per-item idempotency prevents duplicate mutations on replay;
- partial failures are visible and redacted;
- Web live/fallback behavior is honest and bundle budgets hold;
- focused Rust/Web gates pass.

Evidence:

- `docs/workstreams/generated-artifact-bulk-metadata-apply/`
- gates listed in
  `docs/workstreams/generated-artifact-bulk-metadata-apply/EVIDENCE_AND_GATES.md`
- closeout recorded in
  `docs/workstreams/generated-artifact-bulk-metadata-apply/CLOSEOUT.md`

### Architecture Roadmap Reconciliation

Status: completed on 2026-06-01.

Objective:

- Reconcile Nako's top-level roadmap, architecture lane registry, capability
  maps, and workstream navigation after the latest sub-architecture audit.
- Make the next parallel implementation choices evidence-backed instead of
  relying on stale historical handoffs or old active-lane wording.
- Keep completed MVP, GAMA, Web, storage, playback, provider, addon, and
  control-plane lanes closed unless a focused follow-on is explicitly opened.

Deliverables:

- `docs/workstreams/architecture-roadmap-reconciliation/`
- updated active queue in `docs/architecture/LANES.md`
- refreshed program status in `docs/ROADMAP.md`
- corrected high-risk architecture map drift in `docs/architecture/*.md`
- updated workstream evidence links in
  `docs/architecture/WORKSTREAM_LINKS.md`

Non-goals:

- no Rust, Web, schema, API, generated contract, or runtime behavior changes;
- no release artifact publication or packaging changes;
- no broad rewrite of every historical workstream handoff;
- no implementation of proposed follow-on lanes.

Exit criteria:

- current planner docs no longer route active work to closed implementation
  lanes;
- shipped provider, playback policy, artwork, Web, storage, addon, realtime,
  and control-plane evidence is linked from architecture indexes;
- proposed lanes name real next-depth work instead of already-shipped MVP
  slices;
- docs validation gates pass.

Evidence:

- `docs/workstreams/architecture-roadmap-reconciliation/`
- closeout recorded in
  `docs/workstreams/architecture-roadmap-reconciliation/CLOSEOUT.md`

### MVP Release Shape

Status: completed on 2026-06-01.

Objective:

- Define the first release-shaped Nako MVP before more Jellyfin/Plex-class
  breadth is added.
- Converge current playback, metadata, web, addon, storage, network, and
  release work around a video-first, self-hosted, single-admin user journey.
- Route only true MVP blockers into active implementation lanes and explicitly
  defer P1/P2 capabilities.

Deliverables:

- `docs/workstreams/mvp-release-shape/`
- MVP statement and release cut
- release blocker gap matrix
- MVP validation ladder
- active queue alignment across `PTJCH`, `GAMA`, and `CSAPA`
- architecture links from `docs/architecture/LANES.md` and
  `docs/architecture/WORKSTREAM_LINKS.md`

Non-goals:

- no Rust or frontend implementation inside the MVP planning lane;
- no Jellyfin Plugin Compatibility or native in-process plugin ABI;
- no built-in tunnel provider or first-party relay;
- no Addon Manager process/package lifecycle in the MVP cut;
- no production mobile, TV, or desktop-native client implementation.

Exit criteria:

- P0/P1/P2 scope has been verified against repository evidence.
- MVP blockers are routed to active or newly opened workstreams with exact
  gates and owners.
- Non-MVP breadth is explicitly deferred.
- Release gates prove install, scan, metadata, playback, Admin diagnostics,
  Addon Sidecar foundation, network guidance, redaction, and packaging
  readiness.

Evidence:

- `docs/workstreams/mvp-release-shape/`
- planning gates listed in
  `docs/workstreams/mvp-release-shape/EVIDENCE_AND_GATES.md`
- closeout recorded in `docs/workstreams/mvp-release-shape/CLOSEOUT.md`

### Media Server Architecture Map

Status: completed.

Objective:

- Make Nako's future-facing media-server architecture navigable after the
  recent playback/transcode deepening lanes.
- Clarify the long-term Jellyfin/Plex-class target without turning the roadmap
  into a feature checklist detached from current implementation evidence.

Evidence:

- Workstream docs:
  `docs/workstreams/media-server-architecture-progress-map/`.

### Addon Ecosystem Foundation

Status: completed.

Objective:

- Record and execute the Addon ecosystem deepening wave.
- Keep fine-grained Addon manifests, grants, tasks, and event delivery while
  allowing coarse-grained Addon Package and Addon Suite deployment.
- Harden Addon Task idempotency, official catalog drift prevention, Addon Event
  Delivery, and the first official event-driven addon proof before broad
  notification, watch-state sync, MCP, Arr-stack, DLNA, WebDAV, UPnP, or
  network-tunnel breadth lands.

Evidence:

- `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- `docs/workstreams/addon-ecosystem-foundation/`
- follow-on `docs/workstreams/addon-event-scheduler-and-replay/`

## Completed Goals

### Fearless Future Architecture Refactor

Status: completed.

Objective:

- Complete the next fearless server-side architecture refactor after M61-M63.
- Split the remaining broad runtime, persistence, API, VFS, and inference
  modules before new feature breadth hardens their current shape.
- Make Docker-backed local validation part of the normal refactor closeout.

Deliverables:

- `docs/workstreams/fearless-future-architecture-refactor/` as the
  authoritative execution lane.
- Narrower `nako-server` runtime control-plane modules.
- Clearer `nako-db` backend and domain module ownership.
- Split `nako-api` Admin/Public DTO surfaces with local redaction ownership.
- Deeper VFS, Library File Write, naming, and local inference boundaries.
- Deletion of replaced forwards, stale helpers, and compatibility paths.

Non-goals:

- No new provider breadth, client UI, plugin ABI, network tunnel
  implementation, adaptive bitrate ladder, or AI model runtime.
- No historical compatibility burden.
- No copied source, comments, tests, migrations, or generated code from
  `repo-ref/`.

Exit criteria:

- FFR-020 through FFR-060 are completed or split into named follow-on lanes.
- Any changed cross-crate contract, public API shape, storage policy, or
  runtime resource policy has ADR or workstream evidence.
- Docker/container and PostgreSQL gates are run when applicable.
- Final workspace gates pass.

Evidence:

- Workstream docs:
  `docs/workstreams/fearless-future-architecture-refactor/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo check --workspace --tests`;
  - `cargo nextest run --workspace --no-fail-fast`;
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode container`;
  - `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite all-contracts`;
  - `python -m json.tool docs/workstreams/fearless-future-architecture-refactor/WORKSTREAM.json > $null`;
  - `git diff --check`.

### Admin Web Addon Credential and Grant Onboarding

Status: completed.

Objective:

- Productize the Admin Web credential and authority handoff for registered
  **Addon Sidecars**.
- Let administrators issue and rotate one-time Addon Tokens, revoke Addon
  Tokens, replace accepted Addon Grants, and use an enable readiness checklist.
- Preserve the boundary that Nako authorizes and calls sidecars but does not
  install, start, stop, update, remove, log, or supervise sidecar processes.

Deliverables:

- `docs/workstreams/admin-web-addon-credential-grant-onboarding/` as the
  authoritative execution lane.
- Generated Admin API TypeScript contract coverage for token issue/rotation
  one-time responses and grant replacement request shapes.
- Admin Web client/data-source/UI actions for token issue/rotate/revoke,
  accepted grant replacement, and enable readiness.

Non-goals:

- No Addon Manager lifecycle automation, Docker/systemd/Kubernetes/SSH/host
  agent control, or sidecar process supervision.
- No secret manager integration.
- No arbitrary URL-based manifest fetch.
- No Public Client API exposure.

First executable task:

- AWACG-020 Admin API contract and Admin Web data-source actions for
  token/grant onboarding.

Evidence:

- Workstream docs:
  `docs/workstreams/admin-web-addon-credential-grant-onboarding/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

### Admin Web Addon Onboarding

Status: completed.

Objective:

- Productize a safe Admin Web first-run onboarding path for **Addon Sidecars**.
- Let administrators paste an Addon manifest JSON document, preview key facts,
  and register it through the Admin API with `status: "disabled"` by default.
- Preserve the boundary that registration is not installation: Nako stores and
  validates the manifest snapshot, generates guidance, and verifies health, but
  does not start, stop, install, update, remove, or supervise the sidecar.

Deliverables:

- `docs/workstreams/admin-web-addon-onboarding/` as the authoritative execution
  lane.
- Admin Web client/data-source support for `POST /admin/v1/addons`
  registration from pasted manifest JSON.
- Admin Web onboarding UI that hands off to Addon Operations and the Addon
  Install Guide after successful registration.

Non-goals:

- No Addon Manager lifecycle automation, marketplace, package signing,
  Docker/systemd/Kubernetes/SSH/host-agent control, or sidecar process
  supervision.
- No arbitrary URL-based manifest fetch in this lane.
- No Public Client API exposure.
- No token issuance or grant editor unless needed as a minimal continuation
  handoff.

First executable task:

- AWAON-020 Admin Web client/data-source registration support for pasted
  manifest JSON, defaulting to disabled.

Evidence:

- Workstream docs:
  `docs/workstreams/admin-web-addon-onboarding/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo nextest run -p nako-server register_addon_routes_disabled_by_default_and_validate_contract --no-fail-fast`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

### Addon Install Guide Generation

Status: completed.

Objective:

- Productize an Admin-only **Addon Install Guide** for registered **Addon
  Sidecars**.
- Generate Docker Compose and systemd snippets as inert operator guidance.
- Surface Secret Reference checklist, Addon Health Check verification, and
  registration verification steps without resolving secrets.
- Expose the guide in Admin Web through the generated Admin API TypeScript
  contract and existing data-source seam.

Deliverables:

- `docs/workstreams/addon-install-guide-generation/` as the authoritative
  execution lane.
- `GET /admin/v1/addons/{addon_id}/install-guide` Admin API route and DTO.
- Generated Admin Web contract entries and Addon Operations guide preview.

Non-goals:

- No Addon Manager discovery, install, update, remove, marketplace, package
  signing, Docker socket control, systemd control, Kubernetes adapter, SSH
  host agent, log collection, or **Addon Sidecar** process supervision.
- No resolved secret values or secret-manager integration.
- No Public Client API exposure.

First executable task:

- AIG-020 server-owned install guide route, DTO, generated TypeScript contract,
  and focused Rust tests.

Evidence:

- Workstream docs:
  `docs/workstreams/addon-install-guide-generation/`.
- Closeout proof:
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo nextest run -p nako-server install_guide --no-fail-fast`;
  - `cargo fmt --all -- --check`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

### Admin Web Addon Operations

Status: completed.

Objective:

- Productize the completed Admin Addon Operations backend in the Admin Web
  Console.
- Add generated Admin API TypeScript contract coverage for Addon Operations
  route constants and DTOs.
- Deepen the Admin Web `src/adminApi` seam so Addons are live-capable with
  safe mock fallback.
- Render an Addons operations surface for list/detail facts, lifecycle status,
  grants/tokens summary, **Addon Health Check**, manifest surfaces, and
  resource-call diagnostics.

Deliverables:

- `docs/workstreams/admin-web-addon-operations/` as the authoritative
  execution lane.
- Generated Admin API TypeScript contract entries for Addon Operations.
- `apps/admin-web` data-source, mock fixture, and UI tests for Addon
  Operations.

Non-goals:

- No Addon Manager discovery, install, update, marketplace, package signing,
  Docker socket control, or **Addon Sidecar** process supervision.
- No new Addon Protocol behavior.
- No OAuth-first Addon authorization.
- No embedded trusted frontend plugin runtime.

First executable task:

- AWAO-020 generated Admin API TypeScript contract coverage after AWAO-010
  workstream baseline.

Evidence:

- Workstream docs:
  `docs/workstreams/admin-web-addon-operations/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast`;
  - `cargo check -p nako-api -p nako-server --tests`;
  - `cargo nextest run -p nako-server addons --no-fail-fast`;
  - `npm run check`, `npm test`, and `npm run build` in `apps/admin-web`;
  - `git diff --check`.

### Admin Addon Operations MVP

Status: completed.

Objective:

- Productize the Addon administration surface after release packaging and
  Addon architecture deepening.
- Add explicit Admin lifecycle operations for enable/disable and unregister.
- Add Addon Health Check and resource-call diagnostics without leaking
  credentials, payloads, Source Locators, storage paths, or admin authority.
- Expose Addon Entry Points, Hosted Pages, configuration schema metadata,
  Addon Task declarations, and Event Subscription declarations as Admin read
  models.

Deliverables:

- `docs/workstreams/admin-addon-operations-mvp/` as the authoritative closed
  execution lane.
- Admin API routes under `/admin/v1/addons/{addon_id}` for lifecycle,
  unregister, health, surfaces, and diagnostics.
- Focused Addon protocol/client/API/server/DB tests and docs.

Non-goals:

- No Addon Manager discovery, install, update, marketplace, package signing,
  process supervision, or sidecar process removal.
- No OAuth-first Addon authorization.
- No Native Plugin ABI or Jellyfin Plugin Compatibility.
- No embedded trusted Admin UI.
- No full Addon Task runtime or Addon Event Subscription delivery.

Evidence:

- Workstream docs:
  `docs/workstreams/admin-addon-operations-mvp/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests`;
  - `cargo nextest run -p nako-addon-protocol -p nako-addon-client --no-fail-fast`;
  - `cargo nextest run -p nako-db addon --no-fail-fast`;
  - `cargo nextest run -p nako-server addons --no-fail-fast`;
  - `cargo check --workspace --tests`;
  - `cargo nextest run --workspace --no-fail-fast` with 532 tests passed and
    25 skipped;
  - `git diff --check`.
- PostgreSQL opt-in contracts were skipped because `NAKO_TEST_POSTGRES_URL` was
  unset in the closeout environment.

### Addon Architecture Deepening

Status: completed.

Objective:

- Fearlessly deepen Nako's Addon architecture before broader Addon breadth
  hardens shallow Interfaces.
- Consolidate Addon Side Effect runtime lifecycle behavior into a deep Module.
- Add fingerprinted Addon Side Effect idempotency semantics.
- Move shipped Protected Write payload shapes behind explicit Interfaces.
- Deepen Addon Manifest declarations for the next Addon Protocol concepts.
- Deepen Library File Write beyond the current NFO-only Adapter.
- Finish Admin Addon API DTO shielding and route ownership.
- Preserve `nako-addon-protocol` as a permissive public protocol seam and keep
  SQLite/PostgreSQL Addon state parity.

Deliverables:

- `docs/workstreams/addon-architecture-deepening/` as the authoritative
  execution lane.
- Updated Addon ADR statuses for shipped accepted decisions.
- Focused Addon runtime, protocol, Admin API, and persistence tests as each
  slice lands.

Non-goals:

- No Addon Manager lifecycle automation.
- No OAuth-first Addon authorization.
- No Native Plugin ABI or Jellyfin Plugin Compatibility.
- No embedded JavaScript or WASI runtime.
- No broad subtitle model, provider breadth, AI/vector search, network tunnel,
  or playback feature expansion.

First executable task:

- AAD-020 Addon Side Effect Runtime depth after AAD-010 authority freeze.

Evidence:

- Workstream docs:
  `docs/workstreams/addon-architecture-deepening/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo check --workspace --tests`;
  - `cargo nextest run -p nako-addon-protocol -p nako-addon-client --no-fail-fast`;
  - `cargo nextest run -p nako-db addon --no-fail-fast`;
  - `cargo nextest run -p nako-server addons --no-fail-fast`;
  - `cargo nextest run -p nako-server addon_side_effect --no-fail-fast`;
  - `cargo nextest run -p nako-server library_file_write --no-fail-fast`;
  - `cargo nextest run -p nako-api --no-fail-fast`;
  - `cargo nextest run --workspace --no-fail-fast`;
  - `git diff --check`.

### Admin API TypeScript Contract

Status: completed.

Objective:

- Finish synchronizing the Admin API TypeScript contract consumed by
  `apps/admin-web` while keeping it separate from the Public Client SDK and
  `nako-client-protocol`.

Evidence:

- Workstream docs:
  `docs/workstreams/admin-api-typescript-contract/`.
- Closeout proof:
  - `cargo check -p nako-api --examples`;
  - `cargo nextest run -p nako-api admin_contract --no-fail-fast -j 2`;
  - `cargo nextest run -p nako-api typescript --no-fail-fast -j 2`;
  - `npm run check`, `npm run test`, and `npm run build` in `apps/admin-web`;
  - `npm run generate --prefix sdk/typescript`;
  - `npm run check --prefix sdk/typescript`;
  - `git diff --name-only -- crates/nako-client-protocol sdk/typescript`;
  - `cargo fmt --all -- --check`;
  - `git diff --check`.

### M63: Fearless Architecture Deepening

Status: completed.

Objective:

- Deepen the Modules most likely to harden into long-term architecture debt
  before new provider, plugin, AI, playback, and remote-access breadth is added.
- Split Addon Side Effect behavior into deeper Modules with clearer Interfaces.
- Add a transactional Addon Canonical Metadata commit seam for
  metadata/catalog/search/apply-outcome consistency.
- Narrow Library ingestion around a workflow-shaped commit Interface instead
  of broad caller-side repository knowledge.
- Stabilize playback/transcode request identity and hardware diagnostics before
  multi-profile reuse widens.
- Add measured search semantics before AI/vector search.
- Improve test locality only around touched Interfaces.

Deliverables:

- `docs/workstreams/fearless-architecture-deepening/` as the authoritative
  execution lane.
- Behavior-preserving Addon Side Effect Module split.
- Backend-neutral commit tests for any new persistence seam.
- Playback/transcode identity and diagnostics tests.
- Search semantics fixtures/evaluation tests.
- Updated docs and ADRs if public/internal Interfaces change.

Non-goals:

- No new TMDB, Douban, Bangumi, or AI provider breadth.
- No native plugin ABI or Jellyfin plugin compatibility.
- No Network Tunnel Provider implementation.
- No adaptive bitrate ladder implementation.
- No Managed Artwork PostgreSQL parity inside this lane unless it is explicitly
  activated or split back in from
  `docs/workstreams/managed-artwork-postgresql-parity/`.
- No broad UI work.

Exit criteria:

- FAD-020 through FAD-090 are completed or split into named follow-ons with
  evidence-backed rationale.
- Any new persistence seam is proven by SQLite always-on and PostgreSQL opt-in
  contracts where applicable.
- DTO redaction guarantees remain intact for Source Locators, storage URIs,
  local paths, raw source URLs, cache URIs, content hashes, secrets, and raw
  database details.
- Final workspace gates pass.

First executable task:

- FAD-020 Addon Side Effect Module depth.

Evidence:

- Workstream docs:
  `docs/workstreams/fearless-architecture-deepening/`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo check --workspace --tests`;
  - `cargo nextest run --workspace --no-fail-fast`: 498 tests passed and 19
    skipped;
  - `git diff --check`.
- PostgreSQL opt-in contracts were skipped because `NAKO_TEST_POSTGRES_URL` was
  unset in the closeout environment.

### M62: PostgreSQL Production Readiness

Status: completed.

Objective:

- Move PostgreSQL from the M61 job-lease proof into a production-ready database
  backend shape.
- Expand backend-neutral contract tests beyond jobs/leases.
- Add PostgreSQL migration/schema parity for the supported backend scope.
- Add explicit runtime backend selection through `NakoDatabase` and server
  configuration.
- Remove or isolate SQLite assumptions above the adapter seam.
- Document repeatable local/CI verification for SQLite always-on and
  PostgreSQL opt-in gates.

Deliverables:

- `docs/workstreams/postgresql-production-readiness/` as the authoritative
  execution lane.
- Backend contract-test matrix and reusable contract harness.
- Production-shaped backend kind/config selection for SQLite and PostgreSQL.
- PostgreSQL migrations for contract-proven repository/workflow families.
- Safe database backend diagnostics and config/API documentation.
- Final closeout evidence covering workspace gates and PostgreSQL opt-in gates
  when available.

Non-goals:

- No sharding, read replicas, multi-tenant schema management, or online
  zero-downtime migration tooling.
- No replacement of SQLite as the default local backend.
- No broad performance tuning before functional parity is proven.
- No Network Tunnel Provider, AI runtime, or new Admin UI feature expansion
  except database diagnostics needed for this lane.

Exit criteria:

- `NakoDatabase` can select SQLite or PostgreSQL through explicit production
  configuration without server code depending on concrete adapters.
- Required backend-neutral contract families pass against SQLite and
  PostgreSQL, or remaining broad families are split into named follow-ons with
  expiry gates.
- PostgreSQL migrations cover the supported production backend scope.
- SQLite-specific assumptions above the adapter seam are deleted or isolated.
- Local/CI verification commands are documented.
- Final workspace gates pass, and PostgreSQL opt-in gate evidence is recorded
  or explicitly skipped because no PostgreSQL URL is available.

Evidence:

- Workstream docs:
  `docs/workstreams/postgresql-production-readiness/`.
- Existing persistence ADRs:
  `docs/adr/0029-postgresql-ready-persistence-boundary.md`;
  `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`.
- Starting proof:
  `crates/nako-db/src/contract_tests.rs`;
  `crates/nako-db/src/postgres.rs`;
  `crates/nako-db/migrations/postgres/0001_contract_jobs.sql`.
- Closeout proof:
  - `cargo fmt --all -- --check`;
  - `cargo check --workspace --tests`;
  - `cargo nextest run --workspace --no-fail-fast`;
  - `NAKO_TEST_POSTGRES_URL=<local-test-url> cargo nextest run -p nako-db contract --run-ignored ignored-only --no-fail-fast`;
  - `git diff --check`.

### M61: Future-Ready Architecture Refactor

Status: completed.

Objective:

- Refactor Nako toward a cleaner future-ready architecture while there is no
  production compatibility burden.
- Make persistence PostgreSQL-ready instead of SQLite-shaped.
- Deepen module seams for server runtime composition, Local Inference,
  Metadata Candidate Graph, search semantics, and Admin/API read models.
- Delete redundant MVP paths, shallow pass-through modules, compatibility
  shims, and generated/build noise when safer replacements exist.

Deliverables:

- `docs/workstreams/future-ready-architecture-refactor/` as the authoritative
  execution lane.
- A persistence architecture decision for SQLite plus future PostgreSQL.
- Backend-neutral persistence contract tests and explicit unit-of-work seams.
- Slimmer server composition rooted in cohesive runtime modules where useful.
- Implementation slices for Local Inference, Metadata Candidate Graph, Search,
  and Admin/API hygiene.
- A final deletion sweep with fresh validation evidence.

Non-goals:

- No Jellyfin Plugin Compatibility or in-process plugin ABI.
- No copied Jellyfin, Plex, or other reference source, schema, migration, test,
  comment, or generated code.
- No full network tunnel provider implementation.
- No full AI model runtime or vector database.
- No broad provider feature expansion unless needed to prove a candidate seam.

Exit criteria:

- Persistence architecture can support SQLite and future PostgreSQL without a
  `SqliteStore` god-adapter shape.
- Old production paths introduced or replaced by the workstream are deleted or
  have explicit owner/expiry tasks.
- Public Client API remains free of admin/storage internals.
- Admin API read models remain explicit and redacted.
- Required focused gates and final workspace gates pass or have documented
  narrowed rationale.

Evidence:

- Workstream docs:
  `docs/workstreams/future-ready-architecture-refactor/`.
- Persistence ADR:
  `docs/adr/0029-postgresql-ready-persistence-boundary.md`.
- Active task ledger:
  `docs/workstreams/future-ready-architecture-refactor/TODO.md`.
- First backend-neutral persistence contract suite:
  `crates/nako-db/src/contract_tests.rs`.
- Closeout verification: `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast` with 466 tests passed and
  4 skipped; `cargo fmt --all -- --check`; `git diff --check`.

### M60: Admin Catalog Governance Item Queue

Status: completed.

Objective:

- Add the first Admin API v1 catalog governance read model for unknown and
  low-confidence Media Items.
- Keep Local Inference, Provider Mapping, and duplicate Source relationship
  query shape behind a narrow repository port.
- Preserve Public Client API, public OpenAPI/SDK, and `nako-client-protocol`
  boundaries.

Deliverables:

- `CatalogGovernanceRepository` and SQLite read-model adapter.
- `GET /admin/v1/catalog/governance/items` with optional `library_id`,
  `max_confidence_milli`, `limit`, and `offset` filters.
- Redacted admin-owned DTOs for governance item rows and Local Inference
  summaries.
- Focused repository, DTO, route, redaction, auth, and public-boundary tests.
- Updated HTTP API, admin-web-console, and workstream docs.

Non-goals:

- No catalog repair mutation.
- No provider rematch mutation.
- No NFO import/export behavior changes.
- No Source Variant, Edition, or Duplicate UI workflow.
- No Public Client API route or DTO changes.
- No `nako-client-protocol` changes.

Exit criteria:

- Unknown Media Items are listed.
- Non-unknown Media Items with best Local Inference confidence at or below the
  requested threshold are listed.
- High-confidence items are excluded from the queue.
- Rows include source count, representative source identity/file name,
  Local Inference confidence/inferred fields, Provider Mapping counts, and
  duplicate relationship count.
- Responses do not expose source locators, local paths, raw Local Inference
  `evidence_value`, raw provider responses, NFO sidecar paths, tokens, or
  secret values.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.

Evidence:

- `CatalogGovernanceRepository` keeps the governance SQL joins inside
  `nako-db`.
- `AdminCatalogGovernanceItemListResponse` and related DTOs expose only
  redacted admin fields.
- `GET /admin/v1/catalog/governance/items` returns unknown and low-confidence
  queue rows through Admin API v1.
- Tests cover SQLite filtering/exclusion, DTO redaction, route filtering,
  route redaction, auth protection, and public OpenAPI/SDK exclusion.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-core --tests`, `cargo check -p nako-db --tests`, `cargo nextest run -p
  nako-db catalog_governance --no-fail-fast` with 1 test passed, `cargo check
  -p nako-api --tests`, `cargo nextest run -p nako-api
  admin_catalog_governance --no-fail-fast` with 1 test passed, `cargo check -p
  nako-server --tests`, `cargo nextest run -p nako-server http::tests::system
  --no-fail-fast` with 12 tests passed, public OpenAPI/SDK leakage checks,
  `git diff --check`, and `git diff --name-only --
  crates/nako-client-protocol`.

### M57-M59: Admin Operations Read Models Batch

Status: completed.

Objective:

- Add read-only Admin API v1 operational diagnostics for event outbox
  list/filter, storage staging/cache diagnostics, and sanitized server
  configuration diagnostics.
- Use admin-owned redacted DTOs and route-level tests.
- Preserve Public Client API, public OpenAPI/SDK, and `nako-client-protocol`
  boundaries.

Deliverables:

- `GET /admin/v1/events` for event outbox list/filter by kind, status, Media
  Library, Media Source, and pagination.
- `GET /admin/v1/storage/staging` for redacted staging manifest rows plus
  staging budget/startup cleanup/VFS cache summary diagnostics.
- `GET /admin/v1/system/config` for sanitized auth, library, runtime,
  metadata, transcode, staging, and playback configuration diagnostics.
- Admin-owned DTOs in `nako-api::admin`.
- Focused repository, DTO, route, redaction, auth, and boundary tests.
- Updated HTTP API, admin-web-console, and workstream docs.

Non-goals:

- No Public Client API route or DTO changes.
- No `nako-client-protocol` changes.
- No public OpenAPI or TypeScript SDK expansion.
- No admin mutation routes for event retry, staging cleanup, or config edits.
- No event detail route and no raw event payload exposure.
- No frontend UI implementation.

Exit criteria:

- Admin Console can list/filter event outbox rows through `/admin/v1/events`
  without seeing `payload_json`, idempotency keys, raw errors, local paths, or
  secret values.
- Admin Console can inspect staging/cache state through
  `/admin/v1/storage/staging` without seeing staging local paths, full source
  URIs, raw cache URIs, validation error text, or cache error text.
- Admin Console can inspect sanitized server configuration through
  `/admin/v1/system/config` without seeing database URLs, local roots, FFmpeg
  paths, staging roots, WebDAV URLs/usernames/password references, metadata
  proxy values, literal header values, or resolved secrets.
- Existing Public Client API routes remain compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- Focused API, DB, and server validation gates pass.

Evidence:

- `AdminOutboxEventListResponse`, `AdminStorageStagingDiagnosticsResponse`,
  and `AdminServerConfigDiagnosticsResponse` expose admin-owned redacted DTOs.
- `OutboxEventListFilter` and SQLite filtering support event outbox
  list/filter by kind, status, Media Library, Media Source, and pagination.
- `VfsCacheSummary` and staging manifest reads support safe staging/cache
  diagnostics without exposing cache URIs or raw cache errors.
- Route tests cover event payload/idempotency/error redaction, staging
  path/source/cache redaction, sanitized config redaction, and auth protection.
- Public OpenAPI and TypeScript SDK leakage checks still exclude admin/internal
  surfaces.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-db --tests`, `cargo nextest run -p nako-db outbox --no-fail-fast` with
  2 tests passed, `cargo check -p nako-api --tests`, `cargo nextest run -p
  nako-api --no-fail-fast` with 19 tests passed, `cargo check -p nako-server
  --tests`, `cargo nextest run -p nako-server http::tests::system
  --no-fail-fast` with 11 tests passed, public OpenAPI/SDK leakage checks,
  `git diff --check`, and `git diff --name-only --
  crates/nako-client-protocol`.

### M56: Admin Playback Runtime Diagnostics

Status: completed.

Objective:

- Add a read-only Admin API v1 playback runtime diagnostics surface for the web
  console.
- Explain hardware acceleration policy, selected acceleration, FFmpeg
  capability evidence, transcode budgets, remote playback budgets, and staging
  cleanup configuration.
- Preserve Public Client API, public OpenAPI/SDK, and `nako-client-protocol`
  boundaries.

Deliverables:

- Admin-owned playback runtime diagnostics DTOs in `nako-api::admin`.
- Playback app diagnostics snapshot support in `nako-server`.
- `GET /admin/v1/playback/runtime`.
- Focused API/server tests for shape, redaction, auth, and public leakage.
- Updated admin-web-console and HTTP API data-source notes.
- Workstream evidence and closeout docs.

Non-goals:

- No Public Client API route or DTO changes.
- No `nako-client-protocol` changes.
- No public OpenAPI or TypeScript SDK expansion.
- No playback session mutations.
- No playback source selection deepening.
- No adaptive HLS ladder or FFmpeg runner behavior changes.
- No frontend UI implementation.

Exit criteria:

- Admin Console can read playback runtime diagnostics through
  `/admin/v1/playback/runtime`.
- The response includes hardware policy, selected acceleration, FFmpeg
  capabilities, transcode budgets, remote stream/stage budget summaries, and
  staging cleanup summaries.
- The response does not expose local paths, staging roots, transcode
  `output_path`, secrets, tokens, or process-local runner handles.
- Existing Public Client API playback routes remain compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- Focused API and server validation gates pass.

Evidence:

- `AdminPlaybackRuntimeDiagnosticsResponse` and related admin DTOs expose the
  safe diagnostics shape.
- `PlaybackRuntimeDiagnostics` captures playback runtime state without moving
  server internals into HTTP handlers.
- `GET /admin/v1/playback/runtime` returns hardware policy/selection, FFmpeg
  capability summaries, transcode/remux budgets, remote playback budget
  summaries, and staging cleanup summaries.
- Tests cover DTO serialization, route behavior, local-path redaction, auth
  protection, public OpenAPI exclusion, and TypeScript SDK leakage.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-api --tests`, `cargo nextest run -p nako-api --no-fail-fast` with 17
  tests passed, `cargo check -p nako-server --tests`, `cargo nextest run -p
  nako-server http::tests::system --no-fail-fast` with 8 tests passed, public
  OpenAPI/SDK leakage checks, `git diff --check`, and `git diff --name-only
  -- crates/nako-client-protocol`.

### M55: Admin Playback Session List Read Model

Status: completed.

Objective:

- Add a safe Admin API v1 playback session list/filter read model for the web
  console.
- Back it with focused repository/app support, admin-owned redacted DTOs, and
  HTTP tests.
- Preserve Public Client API, public OpenAPI/SDK, and `nako-client-protocol`
  boundaries.

Deliverables:

- Transcode session list/filter support in `nako-core`/`nako-db`.
- Admin-owned playback session list DTOs in `nako-api::admin`.
- `GET /admin/v1/playback/sessions` route and focused HTTP tests.
- Updated admin-web-console data-source notes after route support lands.
- Workstream evidence and closeout docs.

Non-goals:

- No Public Client API route changes.
- No `nako-client-protocol` changes.
- No public OpenAPI or TypeScript SDK expansion.
- No playback session mutations beyond existing known-ID cancel route.
- No transcode runner, hardware acceleration, FFmpeg, or resource-budget
  behavior changes.
- No frontend UI implementation.

Exit criteria:

- Admin Console can list/filter playback sessions by state, kind, Media Source,
  and pagination through `/admin/v1/playback/sessions`.
- Admin list responses do not expose local `output_path`, staging roots,
  filesystem paths, or process-local runtime internals.
- Existing Public Client API session detail/cancel routes remain compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- Focused API, DB, and server validation gates pass.

Evidence:

- `TranscodeSessionListFilter` and SQLite list/filter support back
  `/admin/v1/playback/sessions`.
- `AdminPlaybackSessionListItem` and `AdminPlaybackSessionListResponse` provide
  redacted admin-owned DTOs without `output_path` or raw failure messages.
- Focused tests cover source/kind/state filtering, pagination, route behavior,
  redaction, and auth protection.
- Existing Public Client API session detail/cancel routes remain unchanged.
- Public OpenAPI and TypeScript SDK tests still exclude admin/internal
  surfaces.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-api --tests`, `cargo nextest run -p nako-api --no-fail-fast`, `cargo
  check -p nako-db --tests`, `cargo nextest run -p nako-db transcode
  --no-fail-fast`, `cargo check -p nako-server --tests`, `cargo nextest run -p
  nako-server http::tests::system --no-fail-fast`, public OpenAPI/SDK leakage
  checks, `git diff --check`, and `git diff --name-only --
  crates/nako-client-protocol`.

### M54: Durable Job Runtime And Admin Job List Read Model

Status: completed.

Objective:

- Deepen Nako's server-side durable job runtime so common job lifecycle
  behavior is owned by one Module instead of being duplicated in scan,
  metadata, and NFO workflows.
- Add `GET /admin/v1/jobs` as the first Admin API v1 Jobs/Tasks read model for
  the web console.
- Preserve Public Client API, public OpenAPI/SDK, and `nako-client-protocol`
  boundaries.

Deliverables:

- A durable job lifecycle Module in `nako-server`.
- Migrated scan, metadata refresh/maintenance, and NFO import/export job
  execution paths.
- Job list/filter repository support in `nako-core`/`nako-db`.
- Admin-owned job list DTOs in `nako-api::admin`.
- `GET /admin/v1/jobs` route and focused HTTP tests.
- Updated admin-web-console data-source notes after job list support lands.
- Workstream evidence and closeout docs.

Non-goals:

- No frontend UI implementation or scaffold.
- No generic distributed queue, retry policy, resumable execution, or worker
  process model.
- No Addon Task execution semantics.
- No broad job cancellation unless a narrow read-model need proves it.
- No playback session list/filter in this slice.
- No Public Client API, public SDK, or `nako-client-protocol` changes.

Exit criteria:

- Existing scan, metadata, and NFO job behavior is preserved.
- Common start/succeed/fail handling and summary serialization have one
  authoritative implementation.
- Admin Console can list/filter jobs through `/admin/v1/jobs`.
- Existing root-level `GET /jobs/{job_id}` remains compatible.
- Public OpenAPI and SDK leakage checks still reject admin/internal surfaces.
- Focused API, DB, and server validation gates pass.

Evidence:

- `nako-server::app::job_runtime` centralizes durable job lifecycle handling
  for scan, metadata, and NFO workflows.
- `GET /admin/v1/jobs` is backed by `JobListFilter`, SQLite list/filter
  support, and redacted `AdminJobListItem` DTOs.
- Summary serialization failures now persist durable jobs as failed.
- Existing root-level `GET /jobs/{job_id}` remains compatible.
- Public OpenAPI and TypeScript SDK tests still exclude admin/internal
  surfaces.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-api --tests`, `cargo nextest run -p nako-api --no-fail-fast` with 15
  tests passed, `cargo check -p nako-db --tests`, `cargo nextest run -p
  nako-db jobs --no-fail-fast` with 2 tests passed, `cargo check -p
  nako-server --tests`, `cargo nextest run -p nako-server app::job_runtime
  --no-fail-fast` with 3 tests passed, `cargo nextest run -p nako-server
  app::tests::nfo --no-fail-fast` with 3 tests passed, `cargo nextest run -p
  nako-server http::tests::system --no-fail-fast` with 6 tests passed, public
  OpenAPI/SDK leakage checks, `git diff --check`, and `git diff --name-only
  -- crates/nako-client-protocol`.

### M53: Admin Web Console V0 Context and v0 Prompt Refresh

Status: completed.

Objective:

- Finish AWC-040 and AWC-050 for the admin-web-console workstream.
- Align the v0 context with the live `GET /admin/v1/overview` seam from M52.
- Capture a concise v0.dev prompt for the first admin console prototype.
- Keep the prototype context framework-neutral and explicit about mock-only
  surfaces.

Deliverables:

- Updated `V0_CONTEXT.md` with a first prototype data-source split.
- Captured v0.dev prompt in the admin-web-console handoff.
- Updated admin-web-console task ledger, milestones, evidence, README, and
  workstream metadata.

Non-goals:

- No frontend UI implementation or scaffold.
- No front-end framework selection.
- No Admin API route, DTO, storage, metadata, playback, NFO, or provider
  behavior changes.
- No Public Client API, OpenAPI, SDK, or `nako-client-protocol` changes.

Exit criteria:

- `V0_CONTEXT.md` distinguishes the live overview route from mock or planned
  Admin API data.
- The prompt covers brand, navigation, first prototype pages, data-source
  boundaries, Nako domain language, and safety rules.
- The prompt avoids hard-coding a framework or component implementation.
- AWC-040 and AWC-050 are marked complete.
- Documentation gate passes.

Evidence:

- `docs/workstreams/admin-web-console/V0_CONTEXT.md` records the first
  prototype data-source split.
- `docs/workstreams/admin-web-console/HANDOFF.md` captures the v0.dev prompt.
- `docs/workstreams/admin-web-console/TODO.md` marks AWC-040 and AWC-050
  complete.
- Close-out validation: `git diff --check`.

### M52: Admin API v1 Overview Read-Only Seam

Status: completed.

Objective:

- Build the first code-backed `/admin/v1/*` seam accepted by ADR 0027.
- Add a small read-only admin overview route for the web console.
- Keep the Public Client API, public OpenAPI, public SDK, and
  `nako-client-protocol` unchanged.

Deliverables:

- Admin-owned overview DTOs in `nako-api::admin`.
- `nako-server` route wiring for `GET /admin/v1/overview`.
- Focused HTTP tests proving the route composes safe existing diagnostics and
  preserves existing root/public routes.
- Public OpenAPI and TypeScript SDK leakage checks that keep admin routes out
  of public client artifacts.
- Updated admin-web-console workstream docs and validation evidence.

Non-goals:

- No frontend UI implementation.
- No Admin API mutations.
- No Public Client API or `nako-client-protocol` changes.
- No storage, NFO, metadata provider, playback, or transcode behavior
  expansion beyond read-only diagnostic summaries.
- No Admin OpenAPI or generated admin SDK in this slice.

Exit criteria:

- `GET /admin/v1/overview` returns an admin-owned DTO with server/API version,
  storage summary, metadata-provider summary, runtime summary, and startup
  summary derived from existing safe diagnostics.
- The overview response does not expose secrets, tokens, unsafe local
  filesystem paths, raw provider responses, or local transcode output paths.
- Existing `/health`, `/libraries`, and `/storage/backends` route behavior is
  preserved.
- Public OpenAPI and TypeScript SDK artifacts still exclude `/admin/*` and
  other admin/internal route groups.
- Focused `nako-api` and `nako-server` gates pass.

Evidence:

- `nako-api::admin` defines `ADMIN_API_VERSION`, `AdminOverviewResponse`, and
  focused storage, metadata, runtime, and startup overview DTOs.
- `nako-server` wires `GET /admin/v1/overview` through a dedicated admin HTTP
  module.
- The overview route composes existing storage backend diagnostics, metadata
  provider diagnostics, runtime supervisor counters, and startup report
  counters without returning root URIs, secrets, tokens, raw provider bodies, or
  local output paths.
- Public OpenAPI and TypeScript SDK tests now explicitly reject `/admin` and
  `/admin/v1` terms.
- `crates/nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-api --tests`, `cargo nextest run -p nako-api --no-fail-fast` with 14
  tests passed, `cargo check -p nako-server --tests`, `cargo nextest run -p
  nako-server http::tests::system --no-fail-fast` with 5 tests passed, `git
  diff --check`, and `git diff --name-only -- crates/nako-client-protocol`.

### M51: Admin API Boundary Decision for Web Console

Status: completed.

Objective:

- Complete AWC-030 by deciding the Admin API boundary needed before generating
  or implementing the web admin console.
- Review ADR 0023, ADR 0025, ADR 0026, current `nako-api`/`nako-server` admin
  surfaces, and the admin web console API matrix.
- Document route namespace, versioning, DTO ownership, leakage/redaction rules,
  and public-client separation.
- Update the admin-web-console workstream with the accepted implementation
  sequence.

Deliverables:

- Accepted Admin API boundary ADR.
- Updated admin web console design, task ledger, evidence, handoff, and v0
  context.
- Updated ADR index and goal evidence.

Non-goals:

- No frontend UI implementation.
- No `nako-client-protocol` or Public Client OpenAPI/SDK changes.
- No storage, NFO, provider, playback, or transcode behavior expansion.
- No auth redesign.

Exit criteria:

- Admin-only route namespace and versioning are decided.
- Admin DTO ownership is decided.
- Public Client API separation is explicit.
- Leakage/redaction rules are explicit.
- The next implementation sequence is documented.
- Documentation gate passes.

Evidence:

- [ADR 0027](adr/0027-admin-api-boundary-for-web-console.md).
- [admin-web-console workstream](workstreams/admin-web-console/README.md).
- `ADMIN_API_MATRIX.md` now points to ADR 0027 instead of leaving namespace and
  versioning undecided.
- `V0_CONTEXT.md` marks admin-only areas as mock or planned `/admin/v1/*` data
  rather than Public Client API coverage.
- `nako-client-protocol` has no diff.
- Documentation gate: `git diff --check`.

### M50: NFO Backup Retention and Admin Diagnostics

Status: completed.

Objective:

- Build on M49 by adding a bounded retention policy for local NFO sidecar
  backups.
- Make backup creation, pruning, and failure states inspectable through
  internal/admin-facing diagnostics.
- Keep XML codec, storage backup mechanics, and API/admin adapter
  responsibilities separated.
- Avoid changing public client protocol crates in this slice.

Deliverables:

- VFS backup retention request/report model.
- `LocalFsBackend` keep-latest pruning for Nako-created backups of the same
  sidecar.
- NFO export wiring that requests retention when it requests backup.
- Internal/admin diagnostics for created, pruned, and failed backup operations.
- M50 workstream documentation and validation evidence.

Non-goals:

- No soft-link or hard-link management.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No public client protocol changes.
- No provider breadth, playback, transcode, or new storage backend work.
- No database schema changes unless volatile job summaries prove insufficient.

Exit criteria:

- Local backup writes prune older Nako backups with a bounded keep-latest
  policy.
- Retention pruning preserves unrelated files and non-matching backups.
- NFO forced export records backup creation and pruning diagnostics.
- Admin/public boundary audit proves public client protocols remain unchanged.
- Focused `nako-vfs`/`nako-nfo` and workspace validation gates pass.

Evidence:

- [nfo-backup-retention-diagnostics workstream]
  (workstreams/nfo-backup-retention-diagnostics/README.md) records design, task
  ledger, milestones, evidence, and handoff.
- `StorageBackupPolicy` and `StorageBackupRetention` express keep-latest backup
  retention at the VFS write boundary.
- `LocalFsBackend` prunes only same-sidecar Nako backup files matching the
  `*.nako-backup-*` prefix and preserves unrelated backups/manual files.
- `NfoExportSummary` reports backup creation, pruned backup counts, and prune
  failures for forced sidecar export.
- NFO retention diagnostics remain persisted as internal job summary data.
  Generic HTTP `JobResponse` no longer exposes raw job summaries; any future
  admin inspection should use a dedicated safe diagnostic DTO.
- `nako-client-protocol` has no diff.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-vfs --tests`, `cargo nextest run -p nako-vfs --no-fail-fast` with 28
  tests passed, `cargo check -p nako-nfo --tests`, `cargo nextest run -p
  nako-nfo --no-fail-fast` with 19 tests passed, `cargo check -p nako-api
  --tests`, `cargo nextest run -p nako-api --no-fail-fast` with 13 tests
  passed, `cargo check -p nako-server --tests`, `cargo nextest run -p
  nako-server nfo --no-fail-fast` with 5 selected tests passed, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  315 tests passed, and `git diff --check`.

### M49: NFO Sidecar Backup and Write Conflict Policy

Status: completed.

Objective:

- Build on M47/M48 by adding an explicit backup boundary for local NFO sidecar
  overwrites.
- Create a same-directory backup before replacing an existing sidecar.
- Keep XML preservation in the NFO codec and backup/write mechanics in
  VFS/storage.
- Make backup creation and backup failure visible in internal/test-visible
  diagnostics.

Deliverables:

- VFS write request/report model for optional existing-file backup.
- `LocalFsBackend` same-directory backup implementation before atomic replace.
- NFO forced-export wiring that requests backup only for existing sidecar
  overwrites.
- Focused diagnostics for backup creation and failure categories.
- M49 workstream documentation and validation evidence.

Non-goals:

- No soft-link or hard-link management.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No public HTTP API, OpenAPI, SDK, or protocol changes.
- No database schema or repository trait changes.
- No remote/WebDAV write support.
- No provider breadth, metadata merge-policy redesign, playback work, or
  transcode work.

Exit criteria:

- Local forced export over an existing NFO creates a backup before replacement.
- Fresh sidecar export does not create a backup.
- Unsupported backup requests fail explicitly.
- Backup failure prevents final sidecar replacement.
- Focused `nako-vfs`/`nako-nfo` and workspace validation gates pass.

Evidence:

- [nfo-sidecar-backup-policy workstream]
  (workstreams/nfo-sidecar-backup-policy/README.md) records design, task
  ledger, milestones, evidence, and handoff.
- `nako-vfs` defines `StorageBackupMode` and `StorageBackupReport`, and storage
  write reports can include backup details.
- `LocalFsBackend` creates same-directory backups before overwriting existing
  sidecars and skips backups for fresh sidecar creation.
- NFO forced export requests backup only after confirming an existing sidecar
  will be overwritten.
- `NfoExportSummary` records backup counts and per-item backup reports.
- Backup failures are classified as `NfoFailureKind::StorageBackup` and prevent
  final sidecar replacement.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-vfs --tests`, `cargo nextest run -p nako-vfs --no-fail-fast` with 25
  tests passed, `cargo check -p nako-nfo --tests`, `cargo nextest run -p
  nako-nfo --no-fail-fast` with 18 tests passed, `cargo check --workspace
  --tests`, `cargo nextest run --workspace --no-fail-fast` with 310 tests
  passed, and `git diff --check`.

### M48: NFO Storage Write Policy and Persistence Diagnostics

Status: completed.

Objective:

- Build on M47 by adding a safe NFO sidecar write boundary for local storage.
- Use atomic temp-file-and-rename writes where supported.
- Keep XML preservation in the NFO codec and write mechanics in VFS/storage.
- Make parse, preservation, conflict, unsupported, and storage write failures
  clearer in internal/test-visible diagnostics.

Deliverables:

- VFS write request/report model for explicit write modes.
- `LocalFsBackend` atomic replace implementation.
- NFO export wiring that requests the safer sidecar write path.
- Focused diagnostics for NFO export failure categories.
- M48 workstream documentation and validation evidence.

Non-goals:

- No soft-link or hard-link management.
- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No public HTTP API, OpenAPI, SDK, or protocol changes.
- No database schema or repository trait changes.
- No provider breadth, metadata merge-policy redesign, playback work, or new
  storage backends.

Exit criteria:

- Local NFO sidecar writes are atomic where supported.
- Unsupported atomic write requests fail explicitly.
- NFO export uses the explicit write policy path.
- NFO export failures carry test-visible diagnostic categories.
- M47 preservation behavior remains covered.
- Focused `nako-vfs`/`nako-nfo` and workspace validation gates pass.

Evidence:

- [nfo-storage-write-policy workstream]
  (workstreams/nfo-storage-write-policy/README.md) records design, task
  ledger, milestones, evidence, and handoff.
- `nako-vfs` defines `StorageWriteMode`, `StorageWriteRequest`, and
  `StorageWriteReport`; unsupported atomic replace requests fail explicitly by
  default.
- `LocalFsBackend` implements atomic replace with a same-directory temp file
  and rename where supported.
- NFO export requests `StorageWriteMode::AtomicReplace` for sidecar writes.
- `NfoFailureKind` classifies parse, preservation, unsupported atomic write,
  storage read/write, missing item, invalid sidecar path, and unknown failures
  in internal/test-visible summaries.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-vfs --tests`, `cargo nextest run -p nako-vfs --no-fail-fast` with 22
  tests passed, `cargo check -p nako-nfo --tests`, `cargo nextest run -p
  nako-nfo --no-fail-fast` with 16 tests passed, `cargo check --workspace
  --tests`, `cargo nextest run --workspace --no-fail-fast` with 305 tests
  passed, and `git diff --check`.

### M47: NFO Round Trip Preservation Model

Status: completed.

Objective:

- Deepen `nako-nfo` so export over an existing sidecar preserves unknown XML
  fields instead of regenerating only Nako-known XML.
- Update only Nako-owned NFO fields from canonical metadata.
- Report duplicate or conflicting Nako-owned fields in a structured,
  test-visible model.
- Protect hand-authored and other-media-server NFO content before VFS library
  file write, backup, soft-link, or hard-link policy work.

Deliverables:

- A preservation-aware movie NFO update path in `nako-nfo`.
- A small NFO preservation report/conflict model.
- Forced export wiring that reads an existing sidecar and applies partial
  preservation-aware update.
- Focused tests proving unknown XML preservation, owned-field update, conflict
  reporting, and export workflow behavior.

Non-goals:

- No broad Jellyfin, Kodi, Plex, or Emby compatibility matrix.
- No public HTTP API, OpenAPI, SDK, or protocol changes.
- No database schema or repository trait changes.
- No provider breadth, catalog graph change, or metadata merge-policy redesign.
- No VFS atomic write, backup, soft-link, or hard-link management.

Exit criteria:

- Forced export over an existing movie NFO preserves unknown XML elements.
- Nako-owned fields are updated deterministically from current metadata.
- Duplicate/conflicting owned fields are reported in codec tests.
- Current import and new-sidecar export behavior remains compatible.
- Focused `nako-nfo` and workspace validation gates pass.

Evidence:

- [nfo-round-trip-preservation workstream]
  (workstreams/nfo-round-trip-preservation/README.md) records design, task
  ledger, milestones, evidence, and handoff.
- `nako-nfo` defines `NfoPreservedRender`, `NfoPreservationReport`,
  `NfoFieldConflict`, and `NfoFieldConflictReason`.
- `MovieNfoCodec::render_preserving` updates Nako-owned movie fields while
  preserving unknown top-level XML elements, comments, and processing
  instructions from the existing sidecar.
- Forced export over an existing sidecar reads old XML and writes
  preservation-aware output; missing sidecar creation remains deterministic
  fresh rendering.
- Codec tests cover unknown field preservation, owned-field update, and
  duplicate/alias owned-field conflicts.
- Service tests cover forced export preservation and import-then-forced-export
  round trip preservation.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-nfo --tests`, `cargo nextest run -p nako-nfo --no-fail-fast` with 12
  tests passed, `cargo check --workspace --tests`, `cargo nextest run
  --workspace --no-fail-fast` with 298 tests passed, and `git diff --check`.

### M46: nako-api Module Split

Status: completed.

Objective:

- Make `nako-api` a thin API adapter crate with explicit module boundaries.
- Separate stable Public Client API mapping from server admin/internal,
  metadata diagnostic, extension, webhook, automation, and addon DTOs.
- Keep root-level module declarations while moving callers to explicit
  `nako_api::{public_client,admin,metadata_diagnostics,extension}` module
  imports.

Deliverables:

- `crates/nako-api/src/public_client.rs` owns Public Client protocol
  re-exports and server model-to-DTO adapters.
- `crates/nako-api/src/admin.rs` owns job, ingestion failure, and storage
  backend diagnostic DTOs.
- `crates/nako-api/src/metadata_diagnostics.rs` owns metadata provider attempt,
  runtime diagnostic, raw response, cleanup, and maintenance DTOs.
- `crates/nako-api/src/extension.rs` owns webhook, automation, and addon DTOs.
- `crates/nako-api/src/lib.rs` declares focused modules only; root-level
  wildcard compatibility re-exports were removed by the future-ready
  architecture deletion sweep.

Non-goals:

- No DTO ownership migration into `nako-client-protocol`.
- No public HTTP route, JSON shape, OpenAPI, SDK behavior, or protocol change.
- No playback, storage, NFO, metadata provider breadth, database schema, or
  server runtime behavior change.
- Server call sites use explicit module imports instead of root-level
  `nako_api::*` compatibility imports.

Evidence:

- [api-module-split workstream](workstreams/api-module-split/README.md)
  records design, task ledger, milestones, evidence, and handoff.
- `public_client.rs` does not contain admin, metadata diagnostics, storage
  diagnostics, webhook, automation, or addon DTO names.
- Root-level `nako_api::*` compatibility re-exports were intentionally removed
  during the future-ready architecture deletion sweep after downstream call
  sites moved to explicit module imports.
- Focused validation: `cargo fmt --all -- --check`, `cargo check -p nako-api
  --tests`, `cargo check -p nako-api --examples`, `cargo nextest run -p
  nako-api --no-fail-fast` with 12 tests passed, `npm run check --prefix
  sdk/typescript`, `cargo check --workspace --tests`, `cargo nextest run
  --workspace --no-fail-fast` with 293 tests passed, and `git diff --check`.

### M45: Typed VFS And Storage Error Classification

Status: completed.

Objective:

- Replace brittle string-based storage error classification with typed storage
  error categories.
- Let VFS/storage backends, staging, playback file IO, and HTTP adapters share
  one storage error vocabulary.
- Preserve current public error codes, status codes, and route behavior while
  removing message parsing from HTTP error mapping.

Deliverables:

- `nako-core` storage error classification type and constructors/helpers.
- VFS/WebDAV/local/staging/playback storage errors classified at the source.
- `nako-server` HTTP error mapping driven by typed classification rather than
  string matching.
- Focused tests proving public error code compatibility and backend-specific
  categories.

Non-goals:

- No new storage backends.
- No public API, OpenAPI, SDK, or protocol expansion.
- No database schema changes.
- No NFO Round Trip or library file write/link policy changes.
- No playback source-selection or transcode planning changes.
- No retry policy or durable storage health redesign beyond classification.

Evidence:

- [typed-storage-errors workstream](workstreams/typed-storage-errors/README.md)
  records design, task ledger, milestones, evidence, and handoff.
- `nako-core` defines `StorageErrorKind` and storage error constructors.
- `NakoError::Storage` now carries a typed storage classification.
- `nako-server` HTTP error mapping uses `StorageErrorKind` instead of parsing
  storage messages.
- WebDAV/local VFS, staging, playback file IO, transcode output IO, and test
  storage fakes classify storage errors at construction sites.
- Public storage-related status/code/message behavior remains compatible.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  293 tests passed, and `git diff --check`.

### M44: Metadata Provider Attempt Runtime Extraction

Status: completed.

Objective:

- Deepen `nako-metadata` by extracting provider attempt execution and
  classification into an internal provider-attempt runtime Module.
- Keep `MetadataStrategyExecutor::refresh_item` externally compatible while
  making it read as high-level refresh orchestration.
- Preserve current provider behavior, attempt records, raw response caching,
  refresh commit behavior, and catalog hydration behavior.

Deliverables:

- Internal provider-attempt runtime Module for registered-provider handling,
  search/fetch, success/no-match/provider-failure/fatal classification, skipped
  attempts, and raw response construction.
- Thinner `MetadataStrategyExecutor` workflow code.
- Focused metadata tests proving behavior is unchanged.
- Workstream evidence and closeout documentation.

Non-goals:

- No new provider breadth.
- No public HTTP API, OpenAPI, SDK, CLI, or protocol changes.
- No repository trait churn unless a real use case proves it necessary.
- No database schema changes.
- No NFO Round Trip work.
- No playback/client-profile work.
- No `nako-api` module split.

Evidence:

- [metadata-provider-attempt-runtime workstream]
  (workstreams/metadata-provider-attempt-runtime/README.md) records design,
  task ledger, milestones, evidence, and handoff.
- `nako-metadata` now has an internal `provider_attempt` Module for provider
  lookup/fetch, skipped attempts, raw response construction, attempt recording,
  and provider error classification.
- `MetadataStrategyExecutor::refresh_item` delegates provider-attempt details
  while keeping refresh commit and catalog hydration orchestration explicit.
- Public HTTP API, OpenAPI, SDK/protocol crates, repository traits, database
  schema, NFO, and playback behavior did not change.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-metadata --tests`, `cargo nextest run -p nako-metadata
  --no-fail-fast` with 27 tests passed, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`, and `git diff --check`.

### M43: Playback Source Selection Deepening

Status: completed.

Objective:

- Deepen **Playback Source Selection** before concrete native/mobile client
  work depends on the current MVP playback decision model.
- Make `nako-streaming` own richer source-selection reasoning and playback
  plan intent.
- Keep `nako-server` responsible for loading facts, enforcing access, and
  executing returned direct-play/remux/transcode decisions.
- Preserve existing Public Client API playback response compatibility where
  possible.

Deliverables:

- A workflow-shaped playback selection request and decision model in
  `nako-streaming`.
- Server playback app migration so mode-choice reasoning lives in the
  streaming selection Module instead of around HTTP/runtime orchestration.
- Explicit public DTO compatibility mapping for playback decisions.
- A documented follow-on list for client profiles, source variants, subtitles,
  HDR, bitrate, remote access endpoints, API module split, NFO Round Trip,
  typed VFS errors, and metadata provider-attempt runtime extraction.

Non-goals:

- No Android, Flutter, Web, or player implementation.
- No full Source Variant schema or UI.
- No adaptive bitrate ladder.
- No durable Optimized Version workflow.
- No full Transcode Profile policy engine.
- No NFO Round Trip preservation work.
- No typed VFS error classification work.
- No metadata provider breadth or provider-attempt runtime extraction.

Evidence:

- [playback-source-selection-deepening workstream]
  (workstreams/playback-source-selection-deepening/README.md) records design,
  task ledger, milestones, evidence, and handoff.
- `nako-streaming` exposes `select_playback_source` with
  `PlaybackSelectionRequest`, `PlaybackSelectionContext`,
  `PlaybackSelectedSource`, and `PlaybackExecutionPlan`.
- `PlaybackDecision` separates selected-source facts from direct-play, remux,
  and transcode execution intent while retaining compatibility fields for
  public DTO mapping.
- `nako-server` playback app loads source, probe, client, storage, remux-output,
  and HLS intent facts, then executes the returned decision execution plan.
- Public playback DTO mapping remains wire-compatible; internal
  `selected_source` and `execution` fields do not enter
  `nako-client-protocol`.
- Close-out validation: `cargo fmt --all -- --check`,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast` with 292 tests passed, and `git diff --check`.

### M42: CatalogHydrationPort Lookup Deepening

Status: completed.

Objective:

- Deepen the catalog hydration seam by making callers request hydration as one
  workflow operation.
- Hide snapshot, lookup, and commit implementation details from non-catalog
  adapters and fake tests.
- Preserve existing catalog graph and search projection behavior.

Deliverables:

- `CatalogHydrationPort` exposes a summary-returning hydration workflow.
- Non-catalog crates no longer import `CatalogHydrationLookup`,
  `CatalogHydrationSnapshot`, or `CatalogHydrationCommit`.
- Metadata fake tests prove hydration requests without constructing catalog
  lookup vectors.
- The M42 workstream records evidence and follow-on tasks.

Non-goals:

- No database schema changes.
- No public HTTP API, SDK, CLI, or license-boundary changes.
- No provider breadth or NFO round-trip work.
- No Android client implementation.

Evidence:

- [catalog-hydration-lookup-deepening workstream]
  (workstreams/catalog-hydration-lookup-deepening/README.md) records design,
  task ledger, milestones, evidence, and closeout.
- `CatalogHydrationPort` now exposes `hydrate_catalog`.
- `CatalogHydrationSnapshot`, `CatalogHydrationLookup`, and
  `CatalogHydrationCommit` remain internal to `nako-catalog`.
- Metadata fake-port tests no longer construct lookup vectors.
- Existing catalog graph/search behavior still passes.
- Close-out validation: `cargo fmt --all -- --check`, focused
  catalog/metadata/NFO gates, `cargo check --workspace --tests`, and
  `cargo nextest run --workspace --no-fail-fast` with 288 tests passed.

### M41: Durable Job Recovery and Abort Semantics

Status: completed.

Objective:

- Prevent durable jobs from remaining permanently queued or running after
  shutdown, task abort, or process restart.
- Add startup recovery for unfinished durable jobs, because in-process abort
  paths cannot reliably await database writes.
- Keep runtime shutdown semantics honest while making the persistent job table
  converge to terminal states after restart.
- Remove the unused old `rebuild_search_projection` entrypoint if no caller
  depends on it.

Deliverables:

- `JobRepository` and `SqliteStore` support stale unfinished job recovery.
- `ServerStartupWorkflow` records recovered durable jobs in
  `ServerStartupReport`.
- SQLite and server startup regression tests cover the recovery behavior.
- The M41 workstream records evidence and follow-on architecture tasks.

Non-goals:

- No durable queue dispatcher, retry policy, or resumable job execution.
- No public HTTP API, SDK, CLI, or license-boundary changes.
- No new job status unless a later workflow needs it.
- No `CatalogHydrationPort` lookup deepening in this goal.

Evidence:

- [durable-job-recovery workstream](workstreams/durable-job-recovery/README.md)
  records design, task ledger, milestones, evidence, and closeout.
- `JobRepository::fail_unfinished_jobs` and `SqliteStore::fail_unfinished_jobs`
  mark queued/running jobs failed during startup recovery while preserving
  terminal jobs.
- `ServerStartupWorkflow` records recovered durable jobs in
  `ServerStartupReport::recovered_jobs`.
- `sqlite_store_marks_unfinished_jobs_failed_on_startup` and
  `app_startup_marks_unfinished_jobs_failed` cover adapter and startup
  behavior.
- Removed unused `rebuild_search_projection` and its dead snapshot projection
  helper from `nako-catalog`.
- Close-out validation: `cargo fmt --all -- --check`, focused db/server/catalog
  gates, `cargo check --workspace --tests`, and `cargo nextest run --workspace
  --no-fail-fast` with 288 tests passed.

### M40: Metadata Refresh Workflow Port and Provider Runtime Seam Deepening

Status: completed.

Objective:

- Continue repository seam deepening after M39 by narrowing metadata refresh
  workflow boundaries.
- Reduce `nako-metadata` exposure to broad repository trait combinations and
  provider-runtime persistence details.
- Start with a workflow-shaped metadata refresh port, then split provider
  runtime or maintenance seams only if the first slice exposes a separate
  boundary.

Evidence:

- [metadata-refresh-seam workstream](workstreams/metadata-refresh-seam/README.md)
  records design, task ledger, milestones, evidence, and closeout.
- `crates/nako-metadata/src/strategy.rs` defines `MetadataRefreshPort`,
  `MetadataAttemptPort`, `MetadataRefreshSnapshot`, and
  `MetadataRefreshCommit`.
- `MetadataRefreshService` and `MetadataStrategyExecutor` depend on
  `CatalogHydrationPort + MetadataRefreshPort + MetadataAttemptPort`.
- Refresh calculation uses a snapshot; refresh persistence, provider subject/
  mapping writes, and library-item confirmation sit behind `commit_refresh`.
- A fake-port behavior test proves refresh and hydration port usage without
  SQLite.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-metadata --tests`, `cargo nextest run -p nako-metadata
  --no-fail-fast` with 27 tests passed, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast`, and `git diff --check`.
- Non-goals preserved: no provider breadth, no public API/SDK/CLI or license
  boundary changes, no NFO Round Trip work, no playback/client-profile
  redesign, and no database schema change.

### M39: Repository Seam Deepening and Workflow Port Extraction

Status: completed.

Objective:

- Deepen repository seams after M38 by extracting workflow-shaped ports instead
  of mechanically splitting every repository trait.
- Reduce workflow crate exposure to SQLite and low-level repository details.
- Start with catalog hydration because metadata refresh and NFO import both
  depend on it today.

Evidence:

- [repository-seam-deepening workstream](workstreams/repository-seam-deepening/README.md)
  records design, task ledger, milestones, evidence, and closeout.
- `nako-catalog` exposes `CatalogHydrationPort`,
  `CatalogHydrationSnapshot`, `CatalogHydrationLookup`, and
  `CatalogHydrationCommit`.
- `hydrate_item_catalog` uses the snapshot/lookup/commit workflow port and has
  a fake-port behavior test that does not require SQLite.
- Existing SQLite-backed catalog hydration tests still pass.
- Metadata refresh, hierarchy confirmation, and NFO import call catalog
  hydration through the workflow port instead of carrying the full
  catalog/media/search trait combination.
- Close-out validation: `cargo fmt --all -- --check`, focused catalog,
  metadata, and NFO checks/nextest gates, `cargo check --workspace --tests`,
  `cargo nextest run --workspace --no-fail-fast` with 285 tests passed, and
  `git diff --check`.
- Non-goals preserved: no playback source selection or transcode plan
  redesign, no NFO Round Trip preservation, no public HTTP API, SDK, CLI, or
  license-boundary change, no database schema change, and no broad mechanical
  repository trait split.

### M38: Server Startup Workflow and Durable Job Runtime Deepening

Status: completed.

Objective:

- Move startup side effects out of `NakoApp::new_with_store` and into a
  test-visible startup workflow.
- Keep `NakoApp` as the server composition root while startup sequencing,
  recovery, cleanup, configured-library persistence, and lifecycle task
  registration live behind a deeper interface.
- Add the first durable job runtime helper to `RuntimeSupervisor` and migrate
  library scan, metadata refresh, and metadata maintenance background jobs.

Evidence:

- [server-runtime-deepening workstream](workstreams/server-runtime-deepening/README.md)
  records design, tasks, evidence, gates, and closeout.
- `crates/nako-server/src/app/startup.rs` owns `ServerStartupWorkflow` and
  `ServerStartupReport`.
- `NakoApp::new_with_store` composes app services, then delegates startup side
  effects to the startup workflow.
- Startup reports cover configured libraries, stale transcode recovery,
  staging cleanup, metadata raw-cache cleanup, and lifecycle task registration.
- `RuntimeSupervisor::spawn_job` records supervised job success/failure counts.
- Library scan, metadata refresh, and metadata maintenance background jobs use
  the durable job runtime helper.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-server --tests`, focused nextest gates for app runtime/startup/metadata,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast` with 284 tests passed, and `git diff --check`.
- Non-goals preserved: no playback source selection or transcode plan redesign,
  no NFO round-trip preservation, no broad repository trait split, and no
  public HTTP API, SDK, CLI, or database schema changes.

### M37: Apache-2.0 Rust Client CLI Entrypoint

Status: completed.

Objective:

- Add the first concrete Rust client entrypoint after M35/M36 validated the
  SDK and shared public route inventory.
- Prove an external program can consume `nako-client` without depending on
  AGPL server/internal crates or reimplementing HTTP DTOs.
- Keep the new CLI Apache-2.0 and narrowly scoped to Public Client API usage.

Evidence:

- [client-cli workstream](workstreams/client-cli/README.md)
- `crates/nako-client-cli` is an Apache-2.0 CLI crate.
- The CLI uses `nako-client` as its Nako API entrypoint and does not depend on
  `nako-api`, `nako-server`, `nako-core`, `nako-streaming`, or
  `nako-transcode`.
- Commands cover health, libraries, items, search, source probe, playback
  decision, playback session get/cancel, and streaming request construction.
- Streaming commands print method, URL, and safe headers with bearer token
  values redacted; they do not execute streaming bodies or implement
  downloads/playback.
- Tests cover mocked SDK transport requests, query/path behavior, unauthenticated
  health preflight, authenticated public routes, token redaction, and manifest
  dependency boundaries.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-client-cli --tests`, `cargo nextest run -p nako-client-cli
  --no-fail-fast` with 5 tests passed, `cargo tree -p nako-client-cli`,
  `cargo check --workspace --tests`, `cargo nextest run --workspace
  --no-fail-fast` with 279 tests passed, and `git diff --check`.
- Non-goals preserved: no crates.io publishing, installer, release automation,
  shell completions, TUI, player, HLS playback, download manager, cache,
  background sync, server-admin/internal CLI commands, Flutter/Dart SDK, Web UI,
  or mobile client.

### M36: Client SDK Contract Inventory and Streaming Request Builders

Status: completed.

Objective:

- Remove public client route inventory duplication between `nako-api`,
  TypeScript SDK generation, and `nako-client`.
- Move neutral public route facts into permissive `nako-client-protocol`
  without making clients depend on the AGPL `nako-api` crate.
- Add future-safe Rust SDK request builders for public streaming/raw byte
  routes without implementing body streaming, download management, or player
  behavior.

Evidence:

- [client-sdk-contract workstream](workstreams/client-sdk-contract/README.md)
- `nako-client-protocol` remains `Apache-2.0`, dependency-light, and owns
  `PUBLIC_CLIENT_ROUTES`, `PublicClientRoute`, `PublicClientHttpMethod`,
  `PublicClientRouteKind`, `PublicClientRustSdkExposure`,
  `public_client_paths`, `public_client_json_routes`, and
  `public_client_streaming_routes`.
- `nako-api` OpenAPI tests and TypeScript SDK generation consume the shared
  protocol inventory instead of a local path list.
- `nako-client` consumes the shared inventory and exposes request builders for
  direct stream GET, direct stream HEAD preflight, remux stream GET, HLS
  playlist GET, and HLS segment GET.
- Rust SDK builder tests cover method, path encoding, query serialization,
  bearer auth, and `Range` header behavior.
- Close-out validation: `cargo fmt --all -- --check`, focused check/nextest
  gates for `nako-client-protocol`, `nako-api`, and `nako-client`, `cargo
  nextest run -p nako-server http::tests::playback --no-fail-fast` with 16
  tests passed, `cargo check --workspace --tests`, `cargo nextest run
  --workspace --no-fail-fast` with 274 tests passed, `cargo tree -p
  nako-client-protocol`, `cargo tree -p nako-client`, `npm run check --prefix
  sdk/typescript`, and `git diff --check`.
- Non-goals preserved: no crates.io/npm publishing, no streaming body
  abstraction, no download manager, no HLS player, no Flutter/Dart SDK, no Rust
  CLI product command, and no server API behavior expansion.

### M35: Rust Client SDK Foundation

Status: completed.

Objective:

- Add the first Rust client SDK foundation after M29-M34 stabilized the Public
  Client API, OpenAPI contract, and TypeScript SDK package.
- Reuse permissive `nako-client-protocol` DTOs instead of duplicating Rust wire
  types from OpenAPI.
- Give future Rust CLI, integration tests, third-party tools, and automation
  clients a clean crate boundary for calling Nako public client APIs.

Evidence:

- [rust-client-sdk workstream](workstreams/rust-client-sdk/README.md)
- `crates/nako-client` is an Apache-2.0 SDK crate with explicit license
  metadata.
- `nako-client` depends on `nako-client-protocol` for public DTOs and does not
  depend on `nako-core`, `nako-api`, `nako-server`, `nako-streaming`, or
  `nako-transcode`.
- The SDK exposes `NakoClient`, `ReqwestTransport`, mockable
  `ClientTransport`, `NakoClientError`, pagination helpers, search/playback
  query helpers, bearer-token injection, API-version checking, and
  `ErrorResponse` parsing.
- JSON route methods cover health, libraries, catalog items/search, source
  probe, playback decision, playback session inspection, and playback session
  cancellation.
- Tests cover auth, health without auth, API-version mismatch, public error
  envelope parsing, pagination, URL/path behavior, playback query parameters,
  route inventory, streaming-route deferral, and internal/admin leakage
  rejection.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check -p
  nako-client --tests`, `cargo nextest run -p nako-client --no-fail-fast`
  with 7 tests passed, `cargo tree -p nako-client`, `cargo tree -p
  nako-client-protocol`, `npm run check --prefix sdk/typescript`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  271 tests passed, and `git diff --check`.
- Streaming/raw byte methods, crates.io publishing, Rust CLI commands,
  Flutter/Dart SDK, npm publishing, and concrete UI clients remain follow-ons.

### M34: TypeScript SDK Package Hardening and Contract Compile Check

Status: completed.

Objective:

- Turn the M33 TypeScript SDK scaffold generator into a minimal package with a
  repeatable generation command and a real TypeScript compile contract.
- Prove the generated SDK can be consumed as a future Web/CLI client API
  surface instead of only passing Rust-side string checks.
- Keep package hardening separate from npm publishing, concrete UI clients,
  Flutter/Dart SDK, and Rust SDK implementation.

Evidence:

- [typescript-sdk-package workstream](workstreams/typescript-sdk-package/README.md)
- `sdk/typescript` is a private TypeScript SDK package with local TypeScript
  tooling, strict `tsconfig.json`, committed generated `src/index.ts`, and
  package README.
- `npm run generate --prefix sdk/typescript` refreshes `src/index.ts` through
  `cargo run -q --manifest-path ../../Cargo.toml -p nako-api --example
  emit-typescript-sdk -- --output src/index.ts`.
- `npm run check --prefix sdk/typescript` runs `tsc --noEmit` against the
  generated SDK with strict settings.
- `nako-api` has a sync test that compares the committed package entry with
  `sdk::typescript_sdk()`.
- Close-out validation: `npm run generate --prefix sdk/typescript`, `npm run
  check --prefix sdk/typescript`, `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo check -p nako-api --examples`, `cargo nextest
  run -p nako-api --no-fail-fast` with 11 tests passed, `cargo nextest run
  --workspace --no-fail-fast` with 264 tests passed, `cargo tree -p
  nako-client-protocol`, and `git diff --check`.

### M33: SDK Generation and Client Integration Scaffold

Status: completed.

Objective:

- Establish a repeatable SDK/client integration scaffold after M32 OpenAPI v1.
- Prove future web, CLI, and Flutter work can start from the same public API
  contract instead of scattered handwritten HTTP calls.
- Produce a dependency-light TypeScript/Web/CLI SDK scaffold with bearer auth,
  API-version inspection, error envelope parsing, pagination helpers, and core
  public route methods.

Evidence:

- [sdk-client-scaffold workstream](workstreams/sdk-client-scaffold/README.md)
- `nako-api` owns `sdk::typescript_sdk()` and the
  `emit-typescript-sdk` example for generating a dependency-free
  TypeScript/Web/CLI client scaffold from the OpenAPI v1 contract.
- Generated scaffold covers API version constants, public path inventory,
  OpenAPI-derived wire interfaces, `NakoClient`, `NakoApiError`, bearer-token
  injection, `x-nako-api-version` inspection, error envelope parsing,
  pagination helpers, and core library/catalog/playback/session route calls.
- SDK generator tests cover route inventory, auth/version/error/pagination
  runtime details, and rejection of admin/internal/secret/local-path terms.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo check -p nako-api --examples`, `cargo
  nextest run -p nako-api --no-fail-fast` with 10 tests passed, `cargo
  nextest run --workspace --no-fail-fast` with 263 tests passed, `cargo tree
  -p nako-client-protocol`, and `git diff --check`.

### M32: OpenAPI and Public Client SDK Contract Foundation

Status: completed.

Objective:

- Establish a machine-readable Public Client API schema after the M29 public
  protocol, M30 version/error contract, and M31 bearer-auth boundary.
- Keep `nako-client-protocol` as the permissive public wire-type owner,
  `nako-api` as the AGPL adapter/schema aggregation layer, and `nako-server`
  as route wiring and behavior evidence.
- Produce the first verifiable OpenAPI v1 artifact for core future
  Flutter/web/CLI/SDK surfaces: health, library, catalog browse/search, source
  probe, playback decision, direct/remux/HLS playback, playback sessions, and
  M30/M31 error/auth envelopes.

Evidence:

- [openapi-client-contract workstream](workstreams/openapi-client-contract/README.md)
- [ADR 0025](adr/0025-openapi-public-client-sdk-contract.md)
- `nako-client-protocol` owns protocol DTOs for library detail and playback
  session responses.
- Public playback session responses no longer expose server-local output paths.
- `nako-api` owns `openapi::public_openapi_v1_json()` and the
  `emit-openapi` example for generating the OpenAPI JSON artifact.
- OpenAPI checker tests cover public route inventory, bearer auth,
  `x-nako-api-version`, shared `ErrorResponse`, pagination, and internal/admin
  leakage rejection.
- `nako-server` exposes and tests `GET /libraries/{library_id}` for the
  public library detail surface.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  260 tests passed, `cargo check -p nako-api --examples`, `cargo tree -p
  nako-client-protocol`, and `git diff --check`.

### M31: Access Boundary and Token Authentication Foundation

Status: completed.

Objective:

- Establish an inbound HTTP access boundary so future Flutter, web, CLI,
  remote access, and tunnel work does not depend on unauthenticated server
  APIs.
- Define the difference between Public Client API, Server Admin/Internal API,
  and outbound addon/provider/webhook integration auth.
- Implement the first bearer-token authentication foundation with safe
  defaults, local-development ergonomics, route-level tests, and no token
  leakage.

Evidence:

- [access-boundary-auth workstream](workstreams/access-boundary-auth/README.md)
- [ADR 0024](adr/0024-inbound-token-authentication-boundary.md)
- `nako-client-protocol` owns public `unauthorized` and `forbidden` error
  codes.
- `nako-server` config exposes `[auth]` with auth enabled by default and
  `NAKO_ADMIN_TOKEN` as the default token environment reference.
- `nako-server` HTTP middleware protects every non-health route when auth is
  enabled, while `GET /health` remains public.
- Auth failures return M30-compatible `401 unauthorized` error envelopes with
  `WWW-Authenticate: Bearer` and no token leakage.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  256 tests passed, `cargo tree -p nako-client-protocol`, and `git diff
  --check`.

### M30: Public API Versioning and Error Envelope Hardening

Status: completed.

Objective:

- Stabilize the HTTP API version, error response, pagination/response
  envelope, and compatibility rules that future Flutter, web, CLI, and SDK
  clients will depend on.
- Clarify Public Client API vs Server Admin/Internal API boundaries for error
  codes, HTTP status mapping, version evolution, and deprecation policy.
- Make catalog/library/playback/system public route success and failure
  behavior test-visible and documentable.

Evidence:

- [public-api-contract workstream](workstreams/public-api-contract/README.md)
- [ADR 0023](adr/0023-public-api-versioning-and-error-envelope-contract.md)
- `nako-client-protocol` owns `ClientErrorCode`, `API_VERSION_HEADER`, and
  the compatible `ErrorResponse` envelope constructor.
- `nako-server` emits `x-nako-api-version: v1` and maps `NakoError` through
  protocol-owned public error codes.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  254 tests passed, `cargo tree -p nako-client-protocol`, and
  `git diff --check`.

### M29: Public Client API Contract and Catalog Browse Surface

Status: completed.

Objective:

- Expand `nako-client-protocol` into the first useful public client contract
  for library/catalog browse, search, list/detail, probe, and playback
  decision responses while keeping `nako-api` as the server adapter over
  internal models.

Evidence:

- [public-client-api workstream](workstreams/public-client-api/README.md)
- `nako-client-protocol` owns protocol DTOs with string wire IDs and public
  protocol enums.
- `nako-api` owns explicit mapping functions from `nako-core`,
  `nako-streaming`, and `nako-transcode`.
- Close-out validation: `cargo fmt --all -- --check`, `cargo check
  --workspace --tests`, `cargo nextest run --workspace --no-fail-fast` with
  253 tests passed, `cargo tree -p nako-client-protocol`, and
  `git diff --check`.

### M0-M2.1: Server Runtime Foundation

Status: completed.

Evidence:

- Rust workspace and crate stubs exist.
- SQLite persistence, server runtime, persisted jobs, pagination, logging, and
  developer docs are implemented.
- Related docs: [server-foundation milestones](workstreams/server-foundation/MILESTONES.md).

### M3.1-M3.6: Metadata, NFO, Profiles, and Catalog Planning

Status: completed for the first movie-focused foundation.

Evidence:

- Metadata merge policy, NFO policy, provider secret policy, library presets,
  catalog graph, scan state, and artwork resource-class ADRs exist.
- TMDB movie refresh, NFO import/export jobs, metadata profile execution, and
  catalog/search planning are implemented or documented.

### M4.0-M4.10: Catalog Ingestion and Playback MVP

Status: completed.

Evidence:

- Catalog ingestion, graph hydration, browse APIs, direct play, FFmpeg command
  planning, remux process runner guard, and remux application service
  integration are implemented.
- HTTP remux playback route is implemented.
- Remux/transcode session records are persisted in SQLite and exposed through
  an app/API lookup path.
- A minimal single-variant HLS transcode path can generate, persist, and serve
  playlists and segments.
- Hardware acceleration capability, policy, fallback, and resource-budget
  models are implemented without requiring real GPU hardware in tests.
- MVP stabilization audited API docs, config docs, error behavior, test gaps,
  performance constraints, and known limitations.
- Last completed implementation goal: M4.10 MVP stabilization.

## Recently Completed Goals

### Planning Docs: Goal Map and Refactoring Policy

Status: completed.

Objective:

- Give the project a single top-level route for roadmap, goal tracking,
  workstream ownership, and fearless refactoring policy.

Deliverables:

- `docs/README.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/workstreams/README.md`
- `docs/development/REFACTORING_POLICY.md`
- server-foundation milestone and TODO updates

Non-goals:

- no runtime code changes;
- no ADR status migration beyond documenting the hygiene rule;
- no workstream directory split yet.

Exit criteria:

- top-level docs link to current focus, roadmap, and active workstream;
- the next recommended implementation goal is explicit;
- refactoring policy documents crate boundaries, dependency direction, and
  validation gates;
- doc consistency checks pass.

Evidence:

- `git diff --check` passed for the docs-only change set.

### M4.5: Remux App Service Integration and Local Staging Policy

Status: completed.

Evidence:

- `nako-server::app` has a remux application service boundary.
- `remux_staging_root` config defines the local staging root.
- Remux outputs are deterministic by source ID and container.
- Completed staged outputs are reused.
- In-flight duplicate requests return `Conflict`.
- Tests cover app-service runner execution, completed-output reuse, duplicate
  conflict behavior, staging path validation, and config defaults.

### M4.6: Remux Playback Route

Status: completed.

Evidence:

- `GET /sources/{source_id}/stream/remux` is implemented.
- The handler calls the remux app service and streams staged output.
- `output_container=mp4|mkv` selects the staged remux container.
- Completed staged outputs are reused.
- In-flight duplicates map to `409 conflict`.
- Tests cover range streaming, completed-output reuse, duplicate conflict, and
  unchanged direct play behavior.

### M4.7: Playback Session Persistence

Status: completed.

Evidence:

- `transcode_sessions` persists remux and future transcode session state.
- Remux app-service requests create planned sessions, mark running sessions,
  and persist finished, failed, cancelled, and stale recovery states.
- Completed persisted remux sessions are reused after app restart.
- Active persisted sessions drive duplicate `409 conflict` behavior.
- `GET /playback/sessions/{session_id}` exposes current persisted state.

### M4.8: HLS Transcode Foundation

Status: completed.

Evidence:

- `nako-transcode` plans and runs minimal single-variant HLS sessions through
  FFmpeg.
- HLS output uses a staging layout with temporary directory promotion.
- HLS app service uses persisted transcode sessions for planned, running,
  finished, failed, cancelled, stale, duplicate, and reuse behavior.
- `GET /sources/{source_id}/stream/hls/playlist.m3u8` returns a rewritten HLS
  playlist.
- `GET /playback/sessions/{session_id}/hls/segments/{segment_name}` serves
  generated segments with path traversal protection.

### M4.9: Hardware Acceleration Policy

Status: completed.

Evidence:

- `nako-transcode` has a hardware acceleration capability report, detector
  boundary, policy selection, fallback behavior, and resource-budget model.
- HLS command planning can select CPU-only, VAAPI, NVENC, or QuickSync encoder
  arguments without requiring real hardware in tests.
- `nako-server` config exposes hardware acceleration, fallback, CPU slots, and
  GPU slots with conservative defaults.
- HLS app-service concurrency uses CPU/GPU resource budgets based on the
  selected acceleration class.

### M4.10: MVP Stabilization

Status: completed.

Evidence:

- HTTP API docs match the current local playback routes, including remux, HLS,
  persisted session lookup, and playback error behavior.
- Local setup docs cover scan, probe, metadata, remux, HLS staging, hardware
  policy, and CPU/GPU resource budget configuration.
- Test strategy docs reflect current coverage for browse, metadata/NFO, direct
  play, remux, HLS, persisted playback sessions, and hardware policy.
- Known MVP limitations are documented in the phase note.
- Focused HLS session readiness tests cover active-session conflict behavior at
  the app and HTTP layers.

## Recently Completed Goal

### M5: Extension and Automation Surface

Status: completed.

Implement webhook outbox, automation jobs, addon manifest schema, and one
reference addon. Keep AI-like experience improvements as explicit external
provider/API-key workflows rather than local model or vector infrastructure.

Deliverables:

- M5.0 Extension/Automation Design Baseline.
- M5.1 Event Outbox Foundation.
- M5.2 Webhook Delivery Worker.
- M5.3 Automation Job Model.
- M5.4 Addon Manifest and Resource Contract.
- M5.5 Reference Addon and Stabilization.

Non-goals:

- no local model runtime or vector database;
- no in-process native plugin ABI;
- no embedded JavaScript runtime in the first M5 slice;
- no remote storage backend implementation.

Evidence for M5.0:

- [ADR 0014](adr/0014-durable-event-outbox-for-webhooks-and-automation.md)
  documents durable event outbox and webhook/automation trigger policy.
- [ADR 0015](adr/0015-capability-scoped-http-addons-and-automation-providers.md)
  documents capability-scoped HTTP addons and external automation providers.
- [addons-automation workstream](workstreams/addons-automation/README.md)
  tracks M5 milestones, TODOs, phase notes, resource classes, and security
  boundaries.

Evidence for M5.1:

- `nako-core` defines domain event kinds, event subjects, outbox status, event
  records, and `EventOutboxRepository`.
- `nako-db` migration `0009_event_outbox.sql` persists durable outbox events
  with idempotency by event kind and key.
- `nako-server` writes outbox events for successful library scan, metadata
  refresh, NFO import/export, and playback session completion paths.
- Tests cover outbox persistence, idempotency, and payload safety constraints
  against plaintext secrets and raw local paths.

Evidence for M5.2:

- `nako-core` defines webhook endpoint configuration, delivery attempt records,
  statuses, and `WebhookRepository`.
- `nako-db` migration `0010_webhooks.sql` persists webhook endpoints and
  delivery attempts with per-event inspection.
- `nako-events` builds versioned webhook envelopes, signs payloads with
  HMAC-SHA256, enforces request timeouts, records failed attempts with retry
  timestamps, and provides a `reqwest` transport.
- `nako-server` exposes webhook endpoint configuration/inspection, per-event
  delivery-attempt inspection, explicit outbox event dispatch, and
  `webhook_concurrency` resource budgeting.
- Tests cover SQLite persistence, signed success delivery, failed retry state,
  real transport delivery to a mocked local webhook server, and HTTP
  configuration/inspection routes.

Evidence for M5.3:

- `nako-core` defines automation provider configuration, automation
  capabilities, job input/summary envelopes, artifact records, and
  `AutomationRepository`.
- `nako-db` migration `0011_automation.sql` persists provider configuration and
  generated artifacts.
- `nako-automation` runs mockable external providers through a timeout and
  cancellation-aware runner, persists proposed artifacts, writes job summaries,
  and rejects implicit canonical metadata mutation.
- `nako-server` exposes provider configuration, automation job enqueue, and
  artifact inspection APIs without calling external providers inline.
- Tests cover provider/artifact persistence, mocked provider execution, secret
  omission from job input, canonical-mutation rejection, and HTTP enqueue and
  inspection routes.

Evidence for M5.4:

- `nako-addon-protocol` defines the manifest, protocol version, resource
  declarations, scopes, auth modes, request/response envelopes, and Addon
  Protected Write payload contracts. `nako-addon-client` owns the mockable
  transport, `ReqwestAddonTransport`, and bounded resource caller.
- `nako-core` defines addon registration status and records plus
  `AddonRepository`.
- `nako-db` migration `0012_addons.sql` persists addon registrations, manifest
  snapshots, granted scopes, and enabled/disabled status.
- `nako-server` exposes addon registration, list, status-filtered list, and
  detail APIs. Registrations are disabled by default and rejected when the
  manifest or granted scopes do not satisfy the resource contract.
- Tests cover manifest validation, invalid manifest rejection, scope denial,
  auth token enforcement, bounded retry behavior, response envelope mapping,
  persistence, and HTTP registration/inspection routes.

Evidence for M5.5:

- `nako-reference-addon` provides a minimal local metadata addon fixture with
  a valid manifest and HTTP resource route.
- `nako-server` end-to-end tests register the reference addon through
  `POST /admin/v1/addons`, query it through
  `GET /admin/v1/addons/{addon_id}`, and call the metadata resource through
  `ReqwestAddonTransport`.
- Addon author, webhook receiver, and automation provider guides document the
  current extension surface.
- [Phase 5.5](workstreams/addons-automation/PHASE5_5_REFERENCE_ADDON_STABILIZATION.md)
  documents M5 known limitations and stabilization evidence.

### M6.0: Remote Storage and VFS Design Baseline

Status: completed.

Objective:

- Define the remote-storage architecture before adding WebDAV or S3-compatible
  backend code.

Deliverables:

- ADR 0016 for remote storage and VFS cache boundaries.
- Dedicated `storage-vfs` workstream.
- Local-path dependency audit for `nako-vfs`, scan/probe, direct play, remux,
  and HLS.
- M6 milestone split with WebDAV selected as the first backend preview.
- Roadmap, goal map, ADR index, and workstream index updates.

Evidence:

- [ADR 0016](adr/0016-remote-storage-and-vfs-cache-boundary.md) documents
  WebDAV-first remote storage, VFS cache, staging, credential, and local-path
  boundaries.
- [storage-vfs workstream](workstreams/storage-vfs/README.md) owns M6 remote
  storage, VFS cache, remote staging, and playback policy work.
- [Phase 6.0](workstreams/storage-vfs/PHASE6_0_REMOTE_STORAGE_DESIGN_BASELINE.md)
  records the local-path dependency audit and M6 milestone split.

### M6.1: WebDAV Read-Only VFS Backend

Status: completed.

Evidence:

- `nako-vfs::WebDavBackend` implements read-only `stat`, `list`, and
  `open_range`.
- `VfsLibraryScanner` can scan a mocked WebDAV library without plaintext
  credentials in source locators.
- [Phase 6.1](workstreams/storage-vfs/PHASE6_1_WEBDAV_READ_ONLY_BACKEND.md)
  records validation and limitations.

### M6.2: Directory and Stat Cache

Status: completed.

Evidence:

- `nako-core` defines VFS cache object, listing, failure, and repository
  contracts.
- `nako-db` migration `0013_vfs_cache.sql` persists cached stat/list metadata
  and transient failure state.
- `nako-vfs::CachedStorageBackend` reuses fresh cache and serves stale cache on
  transient storage errors.
- `LibraryIndexService` skips tombstoning when a scan used stale VFS cache.
- [Phase 6.2](workstreams/storage-vfs/PHASE6_2_DIRECTORY_STAT_CACHE.md)
  records validation and remaining cache gaps.

### M6.3: Remote Probe Staging

Status: completed.

Evidence:

- `nako-vfs` defines `StageRequest`, `StagedFile`, deterministic staging paths,
  and `StorageBackend::stage`.
- `nako-vfs::WebDavBackend` can stage a remote media object to a deterministic
  local path and reuse it when size still matches.
- `LibraryProbeService` uses staging when a backend returns no local path hint.
- [Phase 6.3](workstreams/storage-vfs/PHASE6_3_REMOTE_PROBE_STAGING.md)
  records validation and remaining staging gaps.

### M6.4: Remote Playback Policy

Status: completed.

Evidence:

- `StorageBackend::read_range` gives direct play a VFS byte path when a source
  has no local path hint.
- `nako-vfs::WebDavBackend` uses HTTP `Range` GET for byte windows.
- Remux and HLS input planning stages remote sources under
  `remux_staging_root/inputs` before invoking FFmpeg.
- Tests cover remote direct-play bytes, remote FFmpeg staging, local path-hint
  reuse, WebDAV range GET, and WebDAV staging.
- [Phase 6.4](workstreams/storage-vfs/PHASE6_4_REMOTE_PLAYBACK_POLICY.md)
  records validation and remaining production config/API gaps.

### M6.5: Remote Storage Stabilization

Status: completed.

Evidence:

- `NakoServerConfig` supports `[library.webdav]` preview configuration with
  WebDAV root, base URL, username, password environment reference, timeout,
  and retry attempt limits.
- `nako-server::app` builds configured WebDAV storage through
  `WebDavBackend` wrapped in `CachedStorageBackend`.
- Configured WebDAV library scan/probe uses the configured library root;
  remote probe staging uses
  `remux_staging_root/probe-inputs`.
- HTTP API and local setup docs describe WebDAV direct play, remux/HLS staging,
  secret references, and preview limitations.
- [Phase 6.5](workstreams/storage-vfs/PHASE6_5_REMOTE_STORAGE_STABILIZATION.md)
  records validation and remaining known limitations.

## Recently Completed Goals

### M7: Playback Streaming and Remote Hardening

Status: completed.

Objective:

- Make remote playback practical after the M6 WebDAV preview by removing
  direct-play byte buffering, bounding staged remote inputs, improving playback
  failure visibility, adding remote playback resource budgets, and replacing
  the single-library preview shape with explicit multi-library configuration.

Deliverables:

- M7.0 Playback Streaming Design Baseline.
- M7.1 Remote Direct Body Streaming.
- M7.2 Staging Manifest, Disk Budget, and Cleanup.
- M7.3 Playback Error Taxonomy and HTTP Mapping.
- M7.4 Remote Playback Resource Budgets.
- M7.5 Multi-Library and Multi-Remote Backend Config.
- M7.6 Playback Streaming Stabilization.

Non-goals:

- no remote write/delete support;
- no direct FFmpeg remote URL input before a separate accepted design;
- no adaptive bitrate ladder in the first M7 slice;
- no client UI work before server playback contracts stabilize.

Evidence for M7.0:

- [ADR 0017](adr/0017-playback-streaming-and-remote-hardening-boundaries.md)
  documents playback streaming, staging, error mapping, resource budget, and
  configuration boundaries.
- [playback-streaming workstream](workstreams/playback-streaming/README.md)
  tracks M7 milestones, TODOs, phase notes, resource classes, and boundary
  rules.
- [Phase 7.0](workstreams/playback-streaming/PHASE7_0_PLAYBACK_STREAMING_DESIGN_BASELINE.md)
  records the M6 starting point and M7 implementation sequence.

Recommended next implementation goal:

- Start M8 multi-library correctness and operational hardening.

Evidence for M7.1 foundation:

- `nako-vfs` defines `ReadStream` and `StorageBackend::stream_range`.
- `WebDavBackend::stream_range` proxies remote byte streams without
  accumulating chunks into an in-memory direct-play body.
- `nako-server` direct play returns `DirectPlaySourceBody::Stream` for remote
  sources without local path hints, while local sources still use local file
  streaming.
- `HEAD /sources/{source_id}/stream` uses a preflight plan without opening the
  direct-play body.
- Playback app planning and HTTP response helpers are split into
  `crates/nako-server/src/app/playback.rs` and
  `crates/nako-server/src/http/playback.rs`.
- [Phase 7.1](workstreams/playback-streaming/PHASE7_1_REMOTE_DIRECT_BODY_STREAMING.md)
  records validation and remaining gaps.

Evidence for M7.2 foundation:

- `nako-core` defines staging manifest purpose, state, record, and repository
  contracts.
- `nako-db` migration `0014_staging_manifest.sql` persists staging manifest
  records.
- `nako-db/src/staging.rs` implements the staging repository in a dedicated DB
  module instead of growing the large `lib.rs`.
- `nako-server` records `probe_input` manifest entries when remote probe inputs
  are staged, using an app-side VFS wrapper rather than coupling
  `nako-library` to the staging repository.
- `nako-server` records `ffmpeg_input` manifest entries when remote WebDAV
  sources are staged for remux or HLS.
- `[staging].max_bytes` config and the app-side staging wrapper enforce a
  manifest-backed disk budget before remote probe or FFmpeg input staging.
- `[staging].retention_ms` and `[staging].cleanup_on_startup` drive startup
  cleanup of expired staged inputs; cleanup preserves active leases.
- [Phase 7.2](workstreams/playback-streaming/PHASE7_2_STAGING_MANIFEST_FOUNDATION.md)
  records validation and remaining runtime gaps.

Evidence for M7.3 first error-mapping slice:

- Playback/storage HTTP errors now expose stable codes for staging budget
  exhaustion, staging validation mismatch, storage timeout, storage
  unauthorized, storage rate limiting, and FFmpeg failures.
- [Phase 7.3](workstreams/playback-streaming/PHASE7_3_PLAYBACK_ERROR_MAPPING.md)
  records validation and remaining typed-error gaps.

Evidence for M7.4 resource-budget foundation:

- `[playback].remote_stream_concurrency` and
  `[playback].remote_stage_concurrency` define independent remote playback
  budgets.
- Remote direct-play holds a stream permit for the streamed response body, and
  remote probe/FFmpeg staging acquires a stage permit around staging.
- [Phase 7.4](workstreams/playback-streaming/PHASE7_4_REMOTE_PLAYBACK_RESOURCE_BUDGETS.md)
  records validation and remaining route-level stress-test gaps.

Evidence for NFO/VFS storage boundary:

- `run_nfo_import` and `run_nfo_export` now use
  `storage_backend_for_library_root`.
- NFO export checks `StorageCapabilities::WRITABLE`; WebDAV import works
  through the configured VFS backend and WebDAV export is rejected as read-only.
- [Phase 7.4.1](workstreams/playback-streaming/PHASE7_4_1_NFO_STORAGE_BOUNDARY.md)
  records validation.

Evidence for M7.5 multi-library backend foundation:

- `NakoServerConfig` uses `[[libraries]]` as the only server library
  configuration model.
- Server startup upserts every configured library.
- `MediaSource.library_id` gives scan/probe/NFO/playback/FFmpeg staging a
  direct library identity for backend resolution.
- Mixed local/WebDAV library parsing and runtime backend resolution are covered
  by config and app-level tests.
- [Phase 7.5](workstreams/playback-streaming/PHASE7_5_MULTI_LIBRARY_BACKENDS.md)
  records migration shape and known limitations.

Evidence for M7.6 stabilization:

- [Phase 7.6](workstreams/playback-streaming/PHASE7_6_STABILIZATION_AUDIT.md)
  maps every M7 completion criterion to concrete code, tests, docs, and
  validation gates.

### M8: Multi-Library Correctness and Operational Hardening

Status: completed.

Objective:

- Make multi-library operation data-safe by scoping source locator identity to
  the library, exposing explicit CLI multi-library commands, closing the remote
  staging disk-budget race, and documenting the new invariants.

Deliverables:

- `media_sources` uniqueness is `(library_id, locator)` instead of global
  `locator`.
- Repository source lookup by locator requires `library_id`.
- Local scan/index/probe/search tests cover two libraries with the same
  relative media path and the same resulting `local:///` locator.
- CLI supports `scan --library-id`, `scan-all`, and `list --library-id`.
- Staging budget check, staging, and manifest recording are serialized under a
  shared budget lock.
- The panic-style default library helper is replaced with
  `default_library_from_config`.
- [Phase 8.0](workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md)
  records source identity, CLI, and staging budget invariants.

Later follow-up:

- M13-M23 completed the metadata, runtime, database, storage, ingestion, and
  API boundary hardening needed before the M24 server architecture pass.

### M13-M14: Metadata Maintenance and Scheduling

Status: completed.

Evidence:

- [metadata-operations milestones](workstreams/metadata-operations/MILESTONES.md)
  track library-scale maintenance jobs, scheduling, lifecycle, provider
  diagnostics, and raw-cache cleanup.
- [Phase 13.0](workstreams/metadata-operations/PHASE13_0_MAINTENANCE_JOB_BOUNDARY.md)
  and [Phase 14.0](workstreams/metadata-operations/PHASE14_0_SCHEDULING_AND_LIFECYCLE.md)
  record the implemented boundaries.

### M15-M16: Runtime Foundation and Storage Lease Lifecycle

Status: completed.

Evidence:

- [runtime-foundation milestones](workstreams/runtime-foundation/MILESTONES.md)
  track SQLite runtime behavior, migration execution, secret redaction,
  hardware selection, storage backend registry, and staged-input lease
  lifecycle.
- [Phase 15.0](workstreams/runtime-foundation/PHASE15_0_RUNTIME_HARDENING_BASELINE.md),
  [Phase 15.1](workstreams/runtime-foundation/PHASE15_1_RUNTIME_HARDENING_IMPLEMENTATION.md),
  and [Phase 16](workstreams/runtime-foundation/PHASE16_STORAGE_BACKEND_REGISTRY_AND_LEASE_LIFECYCLE.md)
  record the implementation evidence.

### M18-M19: Provider Runtime and Database Boundary Hardening

Status: completed.

Evidence:

- [Phase 18.0](workstreams/metadata-operations/PHASE18_0_PROVIDER_RUNTIME_PRODUCTIZATION.md)
  records the shared metadata provider runtime, secret resolution, and
  provider configuration cleanup.
- [Phase 19.0](workstreams/runtime-foundation/PHASE19_0_DATABASE_BOUNDARY_HARDENING.md)
  records the SQLite repository split, transaction boundaries, and database
  module cleanup.

### M20-M23: Server Surface, Storage, Ingestion, and API Boundary Cleanup

Status: completed.

Evidence:

- [server-foundation milestones](workstreams/server-foundation/MILESTONES.md)
  track M20-M23.
- [Phase 20.0](workstreams/server-foundation/PHASE20_0_SERVER_SURFACE_DECOMPOSITION.md)
  split oversized app and HTTP tests by bounded context.
- [Phase 21.0](workstreams/server-foundation/PHASE21_0_STORAGE_BACKEND_REGISTRY.md)
  recorded storage backend registry ownership.
- [Phase 22.0](workstreams/server-foundation/PHASE22_0_INGESTION_FAILURE_DIAGNOSTICS.md)
  recorded durable ingestion failure diagnostics.
- [Phase 23.0](workstreams/server-foundation/PHASE23_0_API_HTTP_DB_BOUNDARY_CLEANUP.md)
  recorded API DTO, HTTP router, and DB module cleanup.

## Latest Completed Goal

### M24: Server Architecture Hardening

Status: completed.

Objective:

- Turn `nako-server` back into a thin composition root with focused
  application services, explicit background-worker lifecycle ownership, clear
  repository and transaction boundaries, and no obsolete MVP helper paths.

Deliverables:

- [ADR 0019](adr/0019-server-architecture-hardening-boundaries.md) for server
  composition, service, supervisor, and repository boundaries.
- [server-architecture-hardening workstream](workstreams/server-architecture-hardening/README.md)
  with M24 milestones, TODOs, and a baseline phase note.
- App-service decomposition that moves workflow orchestration out of
  `NakoApp`.
- Runtime supervisor or worker registry for background jobs and cleanup loops.
- Repository/transaction cleanup for multi-record writes and broad concrete
  store dependencies.
- Removal of obsolete single-library, compatibility, or temporary helper code.

Non-goals:

- no new metadata provider feature work;
- no client implementation;
- no split into multiple deployable services;
- no in-process plugin ABI design;
- no adaptive bitrate playback ladder;
- no compatibility shims for deprecated shapes unless they have a testable
  migration purpose.

Exit criteria:

- `NakoApp` is a composition root rather than the main feature orchestration
  object.
- HTTP handlers call focused application services and keep response/error
  translation local to HTTP modules.
- Background workers are registered through one lifecycle owner with
  cancellation and failure visibility.
- Multi-record write sequences have explicit repository or unit-of-work
  boundaries.
- Obsolete MVP helpers are removed after their replacement invariants are
  covered by tests.
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

Evidence for M24.0:

- [ADR 0019](adr/0019-server-architecture-hardening-boundaries.md) documents
  the target server architecture boundaries.
- [server-architecture-hardening workstream](workstreams/server-architecture-hardening/README.md)
  tracks M24 milestones, TODOs, phase notes, and refactor policy.
- [Phase 24.0](workstreams/server-architecture-hardening/PHASE24_0_SERVER_ARCHITECTURE_BASELINE.md)
  records the starting surfaces and implementation sequence.

Evidence for M24.1-M24.4:

- [Phase 24.1](workstreams/server-architecture-hardening/PHASE24_1_IMPLEMENTATION_SLICE.md)
  records the service decomposition, runtime supervisor, catalog transaction
  boundary, removed root-app forwards, and NFO structured parser migration.
- `NakoApp` now composes focused service handles for jobs, library scan/probe,
  library administration, catalog, storage diagnostics, metadata, NFO,
  playback, addon, automation, and webhook workflows.
- Metadata, library scan, NFO jobs, metadata lifecycle loops, and staging lease
  cleanup use `RuntimeSupervisor`; webhook delivery is request-scoped
  structured concurrency and automation enqueue is synchronous.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`: 229 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M25: Transcode Runtime Productization

Status: completed.

Objective:

- Turn playback/transcode from an MVP HLS/remux implementation into a clean
  runtime product boundary for hardware acceleration, session orchestration,
  resource budgets, and future adaptive streaming.

Deliverables:

- Create a dedicated transcode runtime workstream and design baseline.
- Decompose the large playback application service into focused direct-play,
  remux, HLS, staging, and transcode-runtime modules.
- Replace the CPU-only server hardware detector with an FFmpeg-backed
  capability probe when hardware acceleration is configured.
- Make VAAPI, NVENC, and QuickSync selection, fallback, and resource budget
  behavior explicit API/service contracts.
- Define the stable client-facing playback session lifecycle and error model
  before Flutter or web client work depends on it.

Non-goals:

- no adaptive bitrate ladder implementation in the first slice;
- no client UI implementation;
- no direct FFmpeg remote credential input until a separate storage security
  design is accepted.

Evidence:

- [transcode-runtime workstream](workstreams/transcode-runtime/README.md)
  tracks the M25 module split, runtime contracts, and post-M25 follow-ups.
- [Phase 25.1](workstreams/transcode-runtime/PHASE25_1_RUNTIME_PRODUCTIZATION.md)
  records the playback service decomposition, FFmpeg hardware detector, CPU/GPU
  budget selection, session lifecycle, validation evidence, and known follow-up
  work.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-server --no-fail-fast`: 90 tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 231 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M26: Playback API Contract and Client Readiness

Status: completed.

Objective:

- Stabilize playback/session HTTP contracts before future web or Flutter
  clients depend on them.

Deliverables:

- Add a public playback session cancellation route.
- Wire cancellation to live remux/HLS FFmpeg runner tokens, not only persisted
  session rows.
- Keep inspection and successful cancellation on `TranscodeSessionResponse`.
- Document active/terminal playback session lifecycle states and stable error
  DTO behavior.
- Validate with route-level tests for active cancellation, terminal conflicts,
  process-local stale active-session conflicts, session inspection, and HLS
  segment readiness/error behavior.

Non-goals:

- no adaptive bitrate ladder;
- no client UI implementation;
- no distributed transcode queue or cross-process cancellation coordinator.

Evidence:

- [Phase 26.0](workstreams/transcode-runtime/PHASE26_0_PLAYBACK_CLIENT_CONTRACT.md)
  records the playback client contract scope, cancellation semantics, and
  validation gates.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check -p nako-server --tests`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-server http::tests::playback --no-fail-fast`: 16
  playback route tests passed.
- `cargo nextest run --workspace --no-fail-fast`: 234 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

## Latest Completed Goal

### M27.3: Hierarchy Confirmation and Provider/NFO Expansion Slice

Status: completed.

Objective:

- Build on M27.2's **Local Inference Evidence** and **Provisional Hierarchy**
  so NFO and built-in providers can confirm series, season, and episode items
  in place instead of replacing Nako item identity.

Deliverables:

- add a shared **Hierarchy Confirmation** service boundary for provider/NFO
  confirmation of provisional hierarchy;
- confirm provisional series, season, and episode items in place without
  replacing Nako `MediaItem` identity;
- write accepted **Provider Mapping** records when metadata provider refresh
  succeeds;
- connect NFO episode import to the shared confirmation service while
  preserving local/NFO authority;
- add TMDB series, season, and episode provider fetch support;
- keep Douban and Bangumi MVPs inside the shared **Provider Subject** and
  **Provider Mapping** boundary.

Non-goals:

- no Source Variant UI;
- no browse API;
- no artwork candidate, selected artwork, or managed artwork expansion;
- no rating, user state, or browse facet work;
- no automatic duplicate merge;
- no general **Hierarchy Repair** flow.

Evidence:

- [Phase 27.3](workstreams/metadata-catalog/PHASE27_3_HIERARCHY_CONFIRMATION_PROVIDER_NFO.md)
  records the hierarchy confirmation, provider mapping, TMDB series/season/
  episode, and NFO episode confirmation slice.
- `nako-metadata` owns the shared **Hierarchy Confirmation** service boundary.
- Metadata refresh writes accepted **Provider Subject** and **Provider
  Mapping** records for successful TMDB, Douban, and Bangumi fetches.
- `nako-nfo` confirms provisional episode hierarchy in place through the
  shared service.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  provider breadth checklist complete.

Close-out validation:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run -p nako-db --no-fail-fast`: 32 tests passed.
- `cargo nextest run -p nako-library --no-fail-fast`: 15 tests passed.
- `cargo nextest run -p nako-metadata --no-fail-fast`: 26 tests passed.
- `cargo nextest run -p nako-nfo --no-fail-fast`: 8 tests passed.
- `git diff --check`: passed with Git CRLF normalization warnings only.

Next recommended implementation goal:

- M28 crate boundary and public protocol hardening.

## Latest Completed Goal

### M27.1: Catalog Schema and Repository Slice

Status: completed.

Objective:

- Turn the M27.0 metadata-catalog domain baseline into durable `nako-core`
  records, `nako-db` schema, repository traits, SQLite adapters, and focused
  repository tests without adding provider breadth.

Deliverables:

- persist **Provider Subject** and **Provider Mapping** separately from Nako
  **Media Item** identity;
- persist **Source Duplicate Relationship** separately from source identity
  and item merging;
- persist minimal **Local Inference Evidence** for inferred kind, title, year,
  season, episode, confidence, evidence source, and inference version;
- cover the selected video item hierarchy and multi-source item link behavior
  through repository tests;
- keep existing movie MVP `MediaItem` and `MediaSource` behavior compatible.

Evidence:

- [Phase 27.1](workstreams/metadata-catalog/PHASE27_1_CATALOG_SCHEMA_REPOSITORY_SLICE.md)
  records the schema/repository implementation and M27.2 boundaries.
- `crates/nako-db/migrations/0018_metadata_catalog_domain.sql` adds the
  durable catalog-domain tables.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  M27.1 checklist complete.

Close-out validation:

- `cargo nextest run -p nako-db`: 31 tests passed.
- `cargo nextest run -p nako-core`: 3 tests passed.
- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `git diff --check`: passed with Git CRLF normalization warnings only.

Next recommended implementation goal:

- M27.2 local inference and provisional hierarchy slice.

### M27.0: Metadata-Catalog Domain Baseline

Status: completed.

Objective:

- Turn the movie-first metadata and catalog foundation into a video-first
  media-server model using the project language defined in `CONTEXT.md` and
  ADR 0021.

Why this came next:

- the playback/runtime contract is now stable enough for client planning;
- the remaining product risk is the metadata domain shape, not FFmpeg or HTTP;
- the current `server-foundation` backlog mixes metadata, NFO, artwork, and
  search follow-ups that should be owned by a dedicated workstream.

Deliverables:

- create a `metadata-catalog` workstream;
- decide the first stable **Media Item** hierarchy for movie, series, season,
  episode, **Episode-Like Item**, **Extra Item**, **Franchise Collection**,
  and unknown video items;
- define **Provider Subject** and **Provider Mapping** rules for TMDB, Douban,
  Bangumi, and future provider/addon evidence;
- decide how **Media Domain** and **Library Preset** influence defaults without
  becoming item identity;
- decide the source-to-item and duplicate-source model;
- separate **Canonical Metadata**, **Media Technical Facts**, **Library Item
  State**, and **User Playback State**;
- define **Metadata Source Priority**, **NFO Round Trip**, **Browse Facet**,
  and **Sort Key** rules;
- define client-facing artwork concepts and search expansion boundaries;
- move the relevant TODO items out of `server-foundation`.

Non-goals:

- no schema migrations;
- no provider feature implementation;
- no runtime behavior changes;
- no public API changes.

Evidence:

- [Phase 27.0](workstreams/metadata-catalog/PHASE27_0_METADATA_CATALOG_DOMAIN_BASELINE.md)
  records the current code audit, baseline decisions, and M27.1/M27.2 handoff.
- [ADR 0021](adr/0021-video-first-media-server-domain-model.md) is accepted.
- [metadata-catalog TODO](workstreams/metadata-catalog/TODO.md) marks the
  M27.0 design-baseline checklist complete.
- [server-foundation TODO](workstreams/server-foundation/TODO.md) no longer
  owns active metadata/catalog/artwork/search follow-ups.

Close-out validation:

- `git diff --check`: passed with Git CRLF normalization warnings only

Next recommended implementation goal:

- M27.1 catalog schema and repository slice.
