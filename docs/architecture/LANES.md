# Architecture Lanes

Last updated: 2026-06-02

This registry routes long-lived Codex terminals and worktrees by capability
area. It is an ownership map for parallel development, not a replacement for
ADRs, architecture maps, or workstream evidence.

Authority order remains:

```text
ADRs -> docs/architecture/*.md -> docs/workstreams/* -> journals/handoff -> chat
```

Use this file when deciding whether a terminal may keep advancing a capability
area across multiple workstreams. For one small task, use the workstream task
ledger instead.

## Active Queue

| Lane | Active workstream | Next task | Recommended terminal role |
| --- | --- | --- | --- |
| library-metadata-control-plane | accepted-review-provider-mapping-application | ARPMA-030 | Candidate review application implementer |

`architecture-roadmap-reconciliation` is closed after `ARR-050`.
`generated-artifact-bulk-metadata-apply` is closed after `GABMA-070`.
`generated-artifact-provider-mapping-breadth` is closed after `GAPM-060`.
`generated-artifact-apply-operations-repair` is closed after `GAOR-040`.
`web-admin-generated-artifact-recovery-ui` is closed after `WAGR-030`.
`generated-artifact-apply-repair-actions` is closed after `GAARA-050`.
The repair path stays preparation-first through existing Metadata Authority
apply routes; do not add a second metadata apply executor or blind retry
mutation.
`metadata-provider-depth-and-precision` is closed after `MPDP-050`.
`MPDP-020` added TMDB series -> season provider graph preview, and `MPDP-030`
proved refresh and Provider Mapping persistence stay root-only. `MPDP-040`
split TMDB episode, Bangumi, Douban, durable candidate review, and Admin/Web
confirmation follow-ons. Open a focused follow-on rather than reopening it.
`tmdb-season-episode-graph-depth` is closed after `TSEG-040`.
It added TMDB season -> episode graph preview and proved season refresh keeps
the same preview-only, root-only persistence boundary. Open a focused follow-on
for Bangumi relations/episodes, Douban subject precision, durable candidate
review, or Admin/Web provider depth governance rather than reopening it.
`bangumi-relations-and-episode-depth` is closed after `BRED-050`.
`BRED-020` narrowed capability claims, `BRED-030` added endpoint-backed
episode graph preview, and `BRED-040` proved refresh remains root-only. Next
focused provider-depth work is now `douban-subject-kind-precision`; durable
candidate review and Admin/Web provider depth governance remain split
follow-ons.
`douban-subject-kind-precision` is closed after `DSKP-030`.
`DSKP-020` narrowed Douban capability claims to movie endpoint-backed behavior
and rejected Series/Season/Episode requests before HTTP. Future Douban
TV/episode depth, durable candidate review, and Admin/Web provider governance
should open focused follow-ons.
`metadata-candidate-durable-review` is closed after `MCDR-050`.
It shipped redaction-safe Candidate Graph review plans, durable review snapshot
persistence, and backend-only accept/reject status transitions without Provider
Mapping writes. Next choose a focused follow-on for Admin/Web provider depth
governance, accepted-review Provider Mapping application, or Douban TV/episode
endpoint depth rather than reopening it.
`accepted-review-provider-mapping-application` is active at `ARPMA-030`.
`ARPMA-020` added a read-only application plan; next apply only root Provider
Subject / Provider Mapping state before Admin/Web routes, Public Client API
changes, or related graph node application.

The `mvp-release-convergence` lane is idle after `mvp-release-shape` closeout.

`generated-artifact-metadata-authority-apply` is closed after `GAMA-070`.
Apply outcome repair tooling and API-backed restoration of placeholder Admin
settings pages remain separate follow-ons. The `client-surfaces-planning` lane
is idle after CSAPA closeout deferred desktop playback to a focused future
spike. The `web-product` lane is idle after `web-mvp-live-smoke` closeout;
open a focused follow-on for backend/API contract, generated SDK, broader
player UX, or desktop/native playback decisions.

The `storage-vfs` lane is idle after
`remote-storage-health-and-circuit-breaker` closeout. Open a new workstream
before starting cache repair, source fingerprint escalation, playback artifact
I/O pressure, scan scheduling, or PostgreSQL runtime harness work.

`audio-compatibility-downmix-normalization`,
`transcode-interface-and-runtime-plan-deepening`, and
`hdr-tone-mapping-pipeline` are closed. Reopen HDR only through a follow-on
workstream for hardware tone mapping, dynamic HDR handling, device profiles, UI
controls, or operator smoke matrices.

`playback-compatibility-matrix-hardening`,
`transcode-capability-inventory-matrix`, `hls-runtime-lifecycle-boundary`,
`hls-progressive-readiness-test-stability`, and
`playback-transcode-jellyfin-class-hardening` are closed. Keep artifact I/O
pressure split to `proposed:hls-artifact-io-pressure-enforcement`. Split
resource admission queueing, remote workers, LL-HLS/CMAF, player UX, hardware
tone-map execution, HEVC/AV1 output policy, subtitle burn-in, Admin/release
reporting, and hardware smoke evidence into separate follow-ons.

## Lane Registry

### architecture-planning

Owns short-lived planner lanes that reconcile roadmap, lane routing,
workstream status, and architecture evidence before parallel implementation
lanes start.

Closed evidence:

- `docs/workstreams/architecture-roadmap-reconciliation/`

Owned scopes:

- `docs/GOALS.md`
- `docs/ROADMAP.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- high-risk capability maps under `docs/architecture/`
- workstream navigation and active queue summaries under `docs/workstreams/`

Shared scopes requiring planner coordination:

- any architecture map that also changes API, schema, crate, runtime, release,
  or product implementation scope;
- broad historical handoff cleanup that could hide useful execution context.

### mvp-release-convergence

Closed `mvp-release-shape` after defining the product/release convergence
overlay for the first self-hosted, video-first, single-admin Nako MVP. Future
work should open a focused follow-on instead of reopening the planning lane.

Closed evidence:

- `docs/workstreams/mvp-release-shape/`
- MVP-related updates in `docs/GOALS.md`
- MVP-related updates in `docs/ROADMAP.md`
- MVP references in `docs/architecture/WORKSTREAM_LINKS.md`
- MVP queue/routing notes in this file

Shared scopes requiring planner coordination:

- all implementation crates and `web/`;
- deployment and release gates;
- public/Admin API contracts;
- related repository work in `nako-official-addons`;
- any ADR/schema/public-contract decision that changes the MVP cut.

### library-metadata-control-plane

Owns Generated Artifact metadata authority, guarded Admin automation routes,
metadata application, audit/outcome persistence, and the control-plane workflow
that turns accepted generated artifacts into Canonical Metadata.

Closed evidence:

- `docs/workstreams/generated-artifact-bulk-metadata-apply/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/`
- `docs/workstreams/generated-artifact-apply-operations-repair/`
- `docs/workstreams/generated-artifact-apply-repair-actions/`
- `docs/workstreams/metadata-provider-depth-and-precision/`
- `docs/workstreams/tmdb-season-episode-graph-depth/`
- `docs/workstreams/bangumi-relations-and-episode-depth/`
- `docs/workstreams/douban-subject-kind-precision/`
- `docs/workstreams/metadata-candidate-durable-review/`

Active evidence:

- `docs/workstreams/accepted-review-provider-mapping-application/`

Owned scopes:

- `crates/nako-core/src/automation.rs`
- `crates/nako-core/src/media/metadata.rs`
- `crates/nako-core/src/repository/automation.rs`
- `crates/nako-core/src/repository/metadata.rs`
- `crates/nako-db/src/**/automation.rs`
- `crates/nako-db/src/**/metadata*.rs`
- `crates/nako-api/src/admin/automation.rs`
- `crates/nako-server/src/app/automation.rs`
- `crates/nako-server/src/app/metadata_application.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-metadata/src/**`
- `crates/nako-api/src/metadata_diagnostics.rs`
- `docs/workstreams/generated-artifact-provider-mapping-breadth/`
- `docs/workstreams/generated-artifact-apply-operations-repair/`
- `docs/workstreams/generated-artifact-metadata-authority-apply/`
- `docs/workstreams/generated-artifact-bulk-metadata-apply/`
- `docs/workstreams/metadata-provider-depth-and-precision/`
- `docs/workstreams/tmdb-season-episode-graph-depth/`
- `docs/workstreams/bangumi-relations-and-episode-depth/`
- `docs/workstreams/douban-subject-kind-precision/`
- `docs/workstreams/metadata-candidate-durable-review/`
- `docs/workstreams/accepted-review-provider-mapping-application/`

Shared scopes requiring planner coordination:

- `web/src/features/admin` and route state for apply workflows;
- generated client contracts;
- schema migrations touching unrelated feature tables;
- metadata provider/addon semantics.

### web-product

Owns the current `web/` product frontend, including Media/Admin route
integration, frontend data-source boundaries, browser smoke evidence, and
bundle-budget gates.

Closed evidence:

- `docs/workstreams/web-admin-generated-artifact-recovery-ui/`

Owned scopes:

- `web/src/api`
- `web/src/features/admin`
- `web/src/features/media`
- `web/src/shell`
- `web/src/test`
- `web/scripts/check-bundle-budget.mjs`
- frontend workstream docs under `docs/workstreams/web-*`
- `docs/workstreams/admin-media-management-context-links/`

Shared scopes requiring planner coordination:

- Admin/Public API DTO shape in `crates/nako-api`;
- generated clients and SDK contract regeneration;
- backend authorization and redaction behavior;
- desktop/Tauri packaging decisions.

### client-surfaces-planning

Closed `client-surface-and-access-product-architecture` after splitting
identity/access, browser Media Web, and Management Context Links, then
deferring desktop playback to a focused future spike.

Closed evidence:

- `docs/workstreams/client-surface-and-access-product-architecture/`
- client/product ADR candidates;
- follow-on workstream split decisions for desktop, mobile, and web product
  surfaces.

Shared scopes requiring planner coordination:

- `web-product` execution tasks;
- Public Client API contracts;
- native client and UniFFI boundaries;
- playback engine and Tauri runtime choices.

### playback-transcode

Owns playback planning, transcode/remux/HLS runtime, streaming transport,
renderer sessions, playback tickets, and playback resource admission.

Owned scopes:

- `crates/nako-playback`
- `crates/nako-transcode`
- `crates/nako-streaming`
- playback routes and app services in `crates/nako-server`
- `docs/architecture/PLAYBACK.md`
- playback/transcode workstreams listed in `WORKSTREAM_LINKS.md`

Shared scopes requiring planner coordination:

- VFS source reads and remote staging;
- state/access policies and playback session writes;
- web/mobile/native player behavior;
- FFmpeg packaging and operations gates.

### storage-vfs

Owns source locators, source identity, VFS backends, remote storage behavior,
staging/cache diagnostics, and storage failure classification.

Owned scopes:

- `crates/nako-vfs`
- storage-facing parts of `crates/nako-library`
- storage diagnostics in `crates/nako-server`
- `docs/architecture/STORAGE_VFS.md`
- storage/VFS workstreams listed in `WORKSTREAM_LINKS.md`

Shared scopes requiring planner coordination:

- library scan and probe orchestration;
- playback input staging;
- metadata/NFO sidecar write policy;
- database source identity projections.

### addons-automation

Owns addon protocol, addon sidecar client behavior, official addon catalog
contracts, automation jobs, outbound task dispatch, and addon lifecycle
integration.

Owned scopes:

- `crates/nako-addon-protocol`
- `crates/nako-addon-client`
- `crates/nako-official-addon-catalog`
- `crates/nako-automation`
- addon/control-plane workstreams listed in `WORKSTREAM_LINKS.md`

Related repositories:

- `nako-official-addons`: official addon implementation and packaging
  checkout. The planner must verify branch, dirty state, and sync point before
  assigning cross-repo addon tasks.

Shared scopes requiring planner coordination:

- cross-repo official addon implementation work;
- generated artifact metadata authority;
- durable jobs and scheduler policy;
- hosted surface/API access decisions.

### operations-release

Owns packaging, release gates, deployment docs, backup/restore, diagnostics
bundles, and operator-facing release readiness.

Owned scopes:

- `docs/deployment/`
- `docs/architecture/OPERATIONS_RELEASE.md`
- release scripts and gate documentation

Shared scopes requiring planner coordination:

- FFmpeg/hardware matrix requirements;
- database migration/recovery policy;
- generated client and native binding release artifacts.

## Shared Coordination Surfaces

These are usually not standalone lane terminals. They cut across many lanes and
should be coordinated by the planner:

- `state-access`: identity, roles, Library Access, session auth, playback
  policy, DTO redaction, and database transaction rules;
- `nako-api` public/Admin DTO contracts;
- schema migrations and PostgreSQL parity;
- `CONTEXT.md` terminology;
- ADR changes.

## Terminal Protocol

Before a lane terminal starts:

1. Start from a clean worktree and sync to the planner-approved baseline.
2. Confirm the lane slug, active workstream, next task, owned scopes, shared
   scopes, and validation commands.
3. Set a Codex goal only for one bounded task from `TODO.md`, not for the whole
   lane.
4. Stop and return to planner coordination when the task needs a shared scope,
   ADR change, schema migration outside the lane, or generated contract update.
5. Review and verify the completed task before starting the next workstream.

Planner terminals own sequencing, branch integration, lane conflicts, closeout,
and follow-on splits. Lane terminals own implementation inside the approved
scope.

## Planner Runtime State

The planner should know local worktree paths, active branches, related
repository paths, dirty state, and last sync points before assigning work. This
is runtime coordination state, not architecture truth.

Record stable lane names and related repo names in this file. Record
machine-specific absolute paths in planner handoff notes, terminal prompts, or
local-only coordination notes when needed.
