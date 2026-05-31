# Architecture Lanes

Last updated: 2026-05-31

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
| `library-metadata-control-plane` | `generated-artifact-metadata-authority-apply` | `GAMA-050` | Backend/control-plane execution |
| `client-surfaces-planning` | `client-surface-and-access-product-architecture` | `CSAPA-050` | Planner/docs split or defer decision |
| `playback-transcode` | `playback-transcode-jellyfin-class-hardening` | `PTJCH-220` | Playback runtime worker |

Do not start `GAMA-060` until `GAMA-050` has reviewed and verified the final
Admin apply route. Do not close `CSAPA` until desktop playback is split,
deferred, or explicitly scoped. The `web-product` lane is idle after
`admin-media-management-context-links` closeout and should receive a new
planner-approved workstream before more frontend execution starts.

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
`transcode-capability-inventory-matrix`, `hls-runtime-lifecycle-boundary`, and
`hls-progressive-readiness-test-stability` are closed.
`playback-transcode-jellyfin-class-hardening` is now the active lane for the
first parallel playback/transcode hardening batch after seam freeze. Keep
artifact I/O pressure split to a PAIP follow-on unless `PTJCH-310` explicitly
accepts it. Split resource admission queueing, remote workers, LL-HLS/CMAF,
player UX, hardware tone-map execution, HEVC/AV1 output policy, subtitle
burn-in, Admin/release reporting, and hardware smoke evidence into separate
follow-ons.

## Lane Registry

### library-metadata-control-plane

Owns Generated Artifact metadata authority, guarded Admin automation routes,
metadata application, audit/outcome persistence, and the control-plane workflow
that turns accepted generated artifacts into Canonical Metadata.

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
- `docs/workstreams/generated-artifact-metadata-authority-apply/`

Shared scopes requiring planner coordination:

- `web/src/features/admin` and route state for apply workflows;
- generated client contracts;
- schema migrations touching unrelated feature tables;
- metadata provider/addon semantics.

### web-product

Owns the current `web/` product frontend, including Media/Admin route
integration, frontend data-source boundaries, browser smoke evidence, and
bundle-budget gates.

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

Owns product architecture decisions for Admin Web, Media Web, desktop playback,
mobile clients, access-gated context switching, and follow-on splits.

Owned scopes:

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
