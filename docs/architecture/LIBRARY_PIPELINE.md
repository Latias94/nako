# Library And Asset Pipeline Architecture

Last updated: 2026-06-03

This document maps the media lifecycle after files become visible through VFS.
It covers scan, watcher, probe, metadata, artwork, and addon-assisted intake.

## Target Chain

```text
Storage event or scheduled scan
  -> stable file candidate
  -> source record and tombstone reconciliation
  -> ffprobe facts
  -> local inference
  -> NFO/provider/addon metadata evidence
  -> canonical metadata merge
  -> catalog graph and search projection
  -> artwork derivatives and client delivery
```

## Progress Matrix

| Capability | Status | Authority | Next Lane |
| --- | --- | --- | --- |
| Durable scan state | Shipped | `docs/adr/0012-durable-scan-state-and-source-tombstones.md` | Watcher/debounce productization. |
| Source tombstones | Shipped foundation | `docs/adr/0012-durable-scan-state-and-source-tombstones.md`; `docs/workstreams/storage-vfs-resilience-and-source-identity/` | Watcher/debounce productization and repair workflows. |
| Local inference | Shipped foundation | `CONTEXT.md`; metadata/catalog lanes | Anime/series path heuristics and confidence reporting. |
| Media probe | Shipped foundation | playback/transcode lanes | More HDR/audio/subtitle technical facts. |
| NFO authority | Shipped foundation | `docs/adr/0008-nfo-as-local-metadata-boundary.md` | Round-trip/writeback polish and backup policy. |
| Metadata merge policy | Shipped foundation; durable candidate review, accepted-review root mapping application, Admin/Web governance, item-scoped Candidate Review list/navigation, global Candidate Review queue/navigation, read-only batch planning, bounded batch confirmation, Web bulk governance, and durable batch execution shipped | `docs/adr/0007-metadata-merge-policy-and-local-authority.md`; `docs/workstreams/metadata-candidate-durable-review/`; `docs/workstreams/accepted-review-provider-mapping-application/`; `docs/workstreams/admin-web-provider-depth-governance/`; `docs/workstreams/admin-candidate-review-list-navigation/`; `docs/workstreams/provider-review-global-queue-search/`; `docs/workstreams/provider-governance-bulk-review/`; `docs/workstreams/provider-governance-durable-batch-execution/` | Keep related hierarchy application, provider endpoint depth, Public Client API exposure, and audit/undo governance in focused follow-ons. |
| TMDB provider | Shipped movie plus series/season/episode graph preview foundation | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/`; `docs/workstreams/metadata-provider-depth-and-precision/`; `docs/workstreams/tmdb-season-episode-graph-depth/` | Accepted-review application or Admin/Web governance before preview graph depth becomes accepted hierarchy. |
| Douban provider | Shipped MVP plus endpoint-backed capability precision | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/`; `docs/workstreams/douban-subject-kind-precision/` | Accepted-review application, Admin/Web governance, or endpoint-backed TV/episode follow-on. |
| Bangumi provider | Shipped MVP plus endpoint-backed episode graph preview | `docs/workstreams/metadata-catalog/`; `docs/workstreams/metadata-provider-breadth/`; `docs/workstreams/bangumi-relations-and-episode-depth/` | Accepted-review application or Admin/Web governance before preview graph depth becomes accepted hierarchy. |
| Addon-assisted metadata | Shipped guarded apply, bulk apply, provider mapping breadth, read-only apply recovery foundation, Web recovery UI, and repair seam proof | addon architecture lanes; `docs/workstreams/generated-artifact-metadata-authority-apply/`; `docs/workstreams/generated-artifact-bulk-metadata-apply/`; `docs/workstreams/generated-artifact-provider-mapping-breadth/`; `docs/workstreams/generated-artifact-apply-operations-repair/`; `docs/workstreams/web-admin-generated-artifact-recovery-ui/`; `docs/workstreams/generated-artifact-apply-repair-actions/` | Optional one-click repair wrapper or Web repair copy polish. |
| Artwork artifact lifecycle | Shipped selection, lifecycle, variant, and remediation foundation | managed artwork lanes | Delivery cache placeholders and broader derivative policy. |
| Watcher/debounce | Weak; stable-candidate evidence foundation shipped | This document | Continue `library-watcher-and-media-intake-stability` toward runtime/product integration. |

## Workstream Evidence

Use
`docs/architecture/WORKSTREAM_LINKS.md#library-metadata-nfo-and-artwork` as the
consolidated index for library, metadata, NFO, and artwork workstreams. Keep
this document focused on intake and asset pipeline capability state.

## Next Work Lanes

### provider-governance-durable-batch-execution

Status: Closed at
`docs/workstreams/provider-governance-durable-batch-execution/`.

Goal: Add a durable, job-backed Admin execution boundary for Metadata Candidate
Review batch apply while preserving the existing single-review application
authority and PGBR redaction semantics.

Shipped:

- `PGDBE-020` added durable batch state and repository contracts.
- `PGDBE-030` added Admin create/status routes.
- `PGDBE-040` added job-backed execution.
- `PGDBE-050` added Web Admin durable Candidate Review batch status.
- `PGDBE-060` closed the lane and split follow-ons.

Non-goals:

- no Public Client API route;
- no related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application;
- no provider endpoint breadth;
- no Generated Artifact apply outcome table reuse;
- no raw background execution outside ADR 0053 runtime boundaries.

### provider-governance-bulk-review

Status: Closed at
`docs/workstreams/provider-governance-bulk-review/`.

Goal: Add guarded batch governance for durable Metadata Candidate Reviews while
preserving existing single-review stale guard, idempotency, root-only Provider
Mapping application, and redaction semantics.

Shipped:

- `PGBR-020` added a read-only Admin API batch Candidate Review application
  plan.
- `PGBR-030` added bounded synchronous Admin API batch apply through the
  existing single-review application authority.
- `PGBR-040` added Web Admin selection, batch plan inspection, explicit
  confirmation, and partial-result rendering.
- `PGBR-050` closed the lane and split follow-ons.

Non-goals:

- no unbounded or hidden batch mutation;
- no Web behavior that bypasses backend plan/apply confirmation semantics;
- no Public Client API route;
- no related graph node hierarchy application;
- no provider endpoint breadth;
- no raw provider payloads, secrets, proxy URLs, headers, paths, source
  fingerprints, raw provider bodies, or raw idempotency keys;
- no hidden raw background execution outside the control-plane boundary.

Follow-ons:

- `proposed:provider-review-related-hierarchy-application`;
- `proposed:douban-tv-episode-endpoint-depth`;
- `proposed:provider-review-public-client-governance`;
- `proposed:provider-governance-audit-and-undo`.

### provider-review-global-queue-search

Goal: Add a read-only global Metadata Candidate Review queue/filter surface for
Admin operators who are triaging review work across items.

Status: Closed at
`docs/workstreams/provider-review-global-queue-search/`.

Shipped:

- `PRGQ-020` added a read-only Admin API global Candidate Review queue route,
  repository query contract, and generated Admin contract sync.
- `PRGQ-030` added Web Admin global queue navigation into the existing
  detail/apply page.
- `PRGQ-040` closed the lane and kept batch governance, related hierarchy
  application, and provider endpoint depth split.

Non-goals:

- no Public Client API route;
- no status mutation, apply mutation, batch governance, or bulk apply;
- no related graph node hierarchy application;
- no raw provider payloads, secrets, proxy URLs, headers, paths, source
  fingerprints, or provider bodies.

Follow-ons:

- `docs/workstreams/provider-governance-bulk-review/` (closed);
- `proposed:provider-review-related-hierarchy-application`;
- `proposed:douban-tv-episode-endpoint-depth`.

### admin-candidate-review-list-navigation

Status: Closed at
`docs/workstreams/admin-candidate-review-list-navigation/`.

Goal: Make durable Metadata Candidate Reviews discoverable from an item-scoped
Admin context and route operators into the existing Candidate Review detail/apply
flow.

Shipped:

- `ACRN-020` added a redaction-safe Admin API list route for durable Metadata
  Candidate Reviews scoped to one Media Item.
- `ACRN-030` added Web Admin item-scoped Candidate Review list/navigation into
  the existing detail/apply page.
- `ACRN-040` closed the lane and split global queue/search, batch governance,
  and related hierarchy application into follow-ons.

Non-goals:

- no Public Client API route;
- no related graph node hierarchy application;
- no global Candidate Review queue/search;
- no batch governance or batch apply;
- no raw provider payloads, secrets, proxy URLs, headers, paths, or provider
  bodies.

Follow-ons:

- `docs/workstreams/provider-review-global-queue-search/` (closed);
- `docs/workstreams/provider-governance-bulk-review/` (closed);
- `proposed:provider-review-related-hierarchy-application`;
- `proposed:douban-tv-episode-endpoint-depth`.

### admin-web-provider-depth-governance

Status: Closed at
`docs/workstreams/admin-web-provider-depth-governance/`.

Goal: Expose durable Metadata Candidate Review evidence and accepted-review
application through Admin API/Web governance.

Shipped:

- `accepted-review-provider-mapping-application` closed after backend
  plan/apply semantics.
- `AWPDG-020` exposes durable Candidate Review detail and application plan facts
  through a redaction-safe read-only Admin API boundary.
- `AWPDG-030` applies accepted reviews explicitly through the backend
  `MetadataCandidateReviewApplicationService` with stale guards and
  idempotency-key fingerprinting.
- `AWPDG-040` exposes the Web Admin direct route for review evidence and
  two-step apply confirmation.
- `AWPDG-050` closes the lane and splits follow-ons.

Non-goals:

- no Public Client API route;
- no related graph node hierarchy application;
- no raw provider payloads, secrets, proxy URLs, headers, or provider bodies;
- no Generated Artifact apply outcome reuse.

Follow-ons:

- `proposed:provider-review-related-hierarchy-application`;
- `proposed:douban-tv-episode-endpoint-depth`;
- `docs/workstreams/provider-governance-bulk-review/` (closed).

### accepted-review-provider-mapping-application

Status: Closed at
`docs/workstreams/accepted-review-provider-mapping-application/`.

Goal: Apply accepted Metadata Candidate Reviews to root Provider Subject and
Provider Mapping state through a named backend boundary.

Shipped:

- `ARPMA-020` defines a read-only accepted-review Provider Mapping application
  plan before mutation.
- `ARPMA-030` applies only the root Provider Subject / Provider Mapping
  idempotently through the backend service.
- `ARPMA-040` splits Admin API/Web mutation scope to provider depth governance.
- `ARPMA-050` closes the backend lane.

### metadata-candidate-durable-review

Status: Closed at `docs/workstreams/metadata-candidate-durable-review/`.

Goal: Define a durable, redaction-safe review boundary for provider Candidate
Graph previews before Admin/Web governance or accepted Provider Mapping
mutation depends on them.

Shipped:

- pure Candidate Graph -> review plan contract;
- durable SQLite/PostgreSQL review snapshot persistence;
- backend-only accept/reject status transitions with stale guards and expiry
  handling;
- Provider Mapping application remains separate from review status changes.

Follow-ons:

- `docs/workstreams/admin-web-provider-depth-governance/` (closed);
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed);
- `proposed:douban-tv-episode-endpoint-depth`.

### douban-subject-kind-precision

Status: Closed at `docs/workstreams/douban-subject-kind-precision/`.

Goal: Make Douban capability claims match its current movie search/detail
endpoint contract before durable candidate review or Admin/Web governance
depends on provider diagnostics.

Shipped:

- narrowed Douban media and Provider Subject capability claims to endpoint-backed
  movie behavior;
- added unsupported-kind regression coverage for Series, Season, and Episode;
- preserved current movie search/fetch and root-only candidate graph behavior.

Non-goals:

- no schema, Public Client API, Admin/Web, or Generated Artifact apply changes;
- no hierarchy graph preview or child Provider Mapping writes;
- no raw Douban payload, API key, header, or proxy URL exposure.

Follow-ons:

- `proposed:douban-tv-episode-endpoint-depth`;
- `docs/workstreams/metadata-candidate-durable-review/` (closed);
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed);
- `docs/workstreams/admin-web-provider-depth-governance/` (closed).

### bangumi-relations-and-episode-depth

Status: Closed at `docs/workstreams/bangumi-relations-and-episode-depth/`.

Goal: Make Bangumi depth claims anime-first and endpoint-backed before Admin
diagnostics or durable candidate review depend on them.

Shipped:

- narrow current Bangumi capability claims to executable subject behavior;
- use official Bangumi relation and episode endpoints before adding graph depth;
- preserve root-only refresh and Provider Mapping persistence;
- do not change schema, Public Client API, Admin/Web, or Generated Artifact
  apply behavior.

Follow-ons:

- `docs/workstreams/metadata-candidate-durable-review/` (closed);
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed);
- `docs/workstreams/admin-web-provider-depth-governance/` (closed).

### tmdb-season-episode-graph-depth

Status: Closed at `docs/workstreams/tmdb-season-episode-graph-depth/`.

Goal: Extend TMDB graph preview from series -> season to season -> episode
without changing persistence semantics.

Shipped:

- parse TMDB season episode summaries;
- add episode Provider Subjects and `contains` relationships under the season
  root graph;
- preserve root-only refresh and Provider Mapping persistence;
- do not change schema, Public Client API, Admin/Web, or Generated Artifact
  apply behavior.

Follow-ons:

- `docs/workstreams/metadata-candidate-durable-review/` (closed);
- `docs/workstreams/accepted-review-provider-mapping-application/` (closed);
- `docs/workstreams/admin-web-provider-depth-governance/` (closed).

### metadata-provider-depth-and-precision

Status: Closed at
`docs/workstreams/metadata-provider-depth-and-precision/`.

Goal: Tighten built-in provider depth, subject identity, and candidate
precision before adding Admin/Web confirmation or durable candidate review.

Initial boundary:

- add TMDB series -> season provider graph evidence before hierarchy mutation;
- preserve root-only refresh and Provider Mapping persistence for graph
  preview data;
- split TMDB episode, Bangumi, Douban, durable candidate review, and Admin/Web
  confirmation follow-ons through
  `docs/workstreams/metadata-provider-depth-and-precision/FOLLOW_ONS.md`;
- keep candidate review non-mutating and redaction-safe;
- do not change schema, Public Client API, or Generated Artifact apply behavior
  in the first task.

Shipped:

- TMDB series -> season provider graph preview;
- root-only refresh and Provider Mapping persistence guard;
- proposed follow-ons for TMDB episode depth, Bangumi, Douban, durable
  candidate review, and Admin/Web provider depth governance.

### source-identity-foundation

Status: The first source identity resilience slice shipped in
`docs/workstreams/storage-vfs-resilience-and-source-identity/`.

Shipped behavior:

- layered source fingerprint evidence without mandatory full-file hashing;
- strong-evidence move/rename reconciliation;
- duplicate-source suggestions instead of automatic weak-evidence merges;
- source-scoped storage failure diagnostics.

### library-watcher-and-media-intake-stability

Goal: Make incremental library intake safe for large files, slow copies, and
remote storage.

Shipped MVP foundation:

- `nako-library::intake` defines a small stable-candidate evidence seam for
  repeated watch observations.
- Intake candidates remain `Inspecting` on the first observation, become
  `Stable` after the same redaction-safe observation key is seen twice
  consecutively, and reset when the key changes.
- Admin overview reports redaction-safe watch-folder runtime coverage so
  operators can see which configured libraries started a supervised watcher and
  which were skipped because monitoring was disabled, unsupported, or missing a
  parseable root.
- This slice does not add an OS watcher daemon, storage-pressure admission, or
  scan scheduler behavior; those remain follow-on productization work.

Scope:

- filesystem watcher integration;
- debounce and stable-size detection;
- copy-in-progress handling;
- scheduled reconciliation scan;
- per-library intake diagnostics.

Exit criteria:

- a large file copy does not trigger premature probe;
- moved/renamed sources do not lose metadata when evidence is strong;
- scan failures stay source-scoped.

### artwork-delivery-cache-placeholder

Goal: Serve artwork in client-appropriate forms instead of sending raw provider
images everywhere.

Scope:

- derivative generation;
- WebP or other client-appropriate output formats;
- size presets;
- Blurhash or placeholder evidence;
- cache invalidation and selected artwork policy.

### generated-artifact-bulk-metadata-apply

Status: Closed at `docs/workstreams/generated-artifact-bulk-metadata-apply/`.

Goal: Turn one-artifact Metadata Authority apply into guarded bulk planning,
durable execution, partial-failure reporting, and Web Admin operator controls.

Shipped:

- read-only bulk apply-plan contract;
- durable batch confirmation and job-backed execution;
- per-item idempotency and partial-failure reporting;
- Admin status/result routes and Web Admin operator workflow.

Next lanes:

- `docs/workstreams/generated-artifact-provider-mapping-breadth/`;
- `docs/workstreams/generated-artifact-apply-operations-repair/`;
- `docs/workstreams/generated-artifact-apply-repair-actions/`.

### generated-artifact-provider-mapping-breadth

Status: Closed at
`docs/workstreams/generated-artifact-provider-mapping-breadth/`.

Goal: Extend accepted metadata Generated Artifact apply so it can plan and
apply Provider Subject and Provider Mapping proposals without turning review
acceptance into a catalog mutation, then surface those effects through
bulk/Admin/Web results.

Shipped backend slices:

- `GAPM-020` adds redaction-safe read-only Provider Mapping plan entries and
  counters to the existing Generated Artifact metadata apply plan.
- `GAPM-030` adds durable/idempotent Provider Subject and accepted Provider
  Mapping apply through the single-artifact Metadata Authority outcome
  transaction.
- `GAPM-040` adds Provider Mapping apply/skip/noop counters to bulk plan
  summaries, batch snapshots, Admin HTTP responses, generated contracts, and
  Web read-model mapping.

Closeout:

- Web Admin renders Provider Mapping plan/result facts in single and bulk
  Metadata Authority apply workflows without adding backend apply behavior.
- Continue through the closed
  `docs/workstreams/generated-artifact-apply-operations-repair/` evidence and
  `proposed:provider-identity-mapping-breadth`, not by reopening this lane.

Boundaries:

- no provider search/depth or hierarchy repair;
- no raw provider payload exposure;
- no Public Client API changes;
- no operations repair tooling in this lane.

### generated-artifact-apply-operations-repair

Status: Closed at
`docs/workstreams/generated-artifact-apply-operations-repair/`.

Goal: Turn durable Generated Artifact apply outcomes and bulk batch state into
an operator-facing recovery workflow for stale, failed, skipped, and noop
results without weakening Metadata Authority boundaries or redaction.

Shipped:

- one-artifact apply outcomes are queryable through Admin list/detail routes;
- outcome-only records and bulk batch terminal items flow into an Admin
  recovery queue;
- recovery entries classify `needs_repair`, `needs_review`, `replay_only`,
  and `resolved`;
- generated Admin contracts and Web Admin read models carry recovery facts
  without raw internal leakage.

Boundaries:

- no blind retry button without plan/result semantics;
- no provider-depth precision expansion in this lane;
- no Public Client API changes.

Follow-ons:

- `docs/workstreams/web-admin-generated-artifact-recovery-ui/` (closed)
- `docs/workstreams/generated-artifact-apply-repair-actions/` (closed)
- `proposed:provider-identity-mapping-breadth`

### generated-artifact-apply-repair-actions

Status: Closed at
`docs/workstreams/generated-artifact-apply-repair-actions/`.

Goal: Prove the recovery queue can route operators into bounded repair
preparation while preserving Metadata Authority apply as the execution kernel.

Shipped:

- existing single/bulk apply routes are selected as the repair execution
  boundary for the current product shape;
- Web recovery rows route to the current apply plan and require confirmation
  with a new idempotency key;
- no backend recovery wrapper or second metadata apply executor is added.

Follow-ons:

- `proposed:generated-artifact-recovery-one-click-wrapper`
- `proposed:web-generated-artifact-repair-copy-polish`
- `docs/workstreams/metadata-provider-depth-and-precision/` (closed)

## Risk Register

### Watch Events Are Not Stable Media Events

Create/modify events often fire before a media file is complete.

Mitigation:

- debounce events;
- require size stability or closed-file evidence where available;
- use scheduled scans as correction, not as the only intake path.

### Addons Must Not Own Canonical State

Addon or AI metadata can be powerful, but host policy must own application,
field locks, provenance, and conflict resolution.

Mitigation:

- addons submit evidence or proposed changes;
- Nako applies through metadata application policy;
- user/admin review controls ambiguous changes.

### Artwork Can Dominate Client Perceived Performance

Large raw images can make a good catalog feel slow.

Mitigation:

- generate derivatives;
- cache by selected artwork and size;
- expose placeholder hashes for skeleton/loading states.

## Agent Notes

Do not put provider-specific hierarchy directly into `MediaItem` shape when a
Nako term exists. Use `Provider Subject`, `Provider Mapping`, `Local Inference`,
and `Canonical Metadata` from `CONTEXT.md`.
