# M1 Admin Diagnostics And Repair Coverage

Status: Product-Operator M1 coverage audit
Last updated: 2026-06-06

This matrix audits Admin diagnostics and repair coverage for Product-Operator
M1. It answers one release-planning question: when the M1 ladder says an
operator can diagnose or repair common failures, which concrete Admin surfaces
prove that claim, and which follow-on tasks are still justified by evidence?

This document is not a new implementation plan. It is a routing matrix for
opening focused Trellis tasks from current evidence.

## Scope

M1 is video-first and operator-first. The relevant failure classes are the ones
a single self-hosted operator can plausibly hit while configuring one Media
Library, scanning/indexing media, browsing catalog entries, playing video, and
recovering from common failures.

In scope:

- Admin diagnostics and repair surfaces that are already exposed through Admin
  API, Admin Web, release gates, or documented operator evidence;
- redaction and auth/access proof for those surfaces;
- concrete follow-on candidates when coverage is read-only, backend-only, or
  not yet connected to the operator journey.

Out of scope:

- broad Public Client metadata governance;
- Addon Manager lifecycle;
- automatic source merge, Media Item merge, or destructive cache cleanup;
- LL-HLS/CMAF, hardware tone-map execution, subtitle burn-in, or player polish
  unless the M1 ladder exposes a concrete playback blocker;
- new schema, API, generated contract, runtime, or Web implementation changes
  in this audit slice.

## Coverage Classification

| Classification | Meaning | M1 routing |
| --- | --- | --- |
| Shipped | Admin-visible diagnostics or repair behavior exists with focused evidence. | Keep as release evidence; do not reopen as generic work. |
| Adequate for M1 | Coverage is not complete long-term, but sufficient for Product-Operator M1. | Defer breadth unless a ladder gate fails. |
| Backend-only | Backend/Admin API behavior exists, but the operator journey may not expose it. | Open Web/product task only if the M1 flow requires direct operator use. |
| Read-only only | Diagnostics identify pressure, but mutation/repair is deliberately deferred. | Open repair task only for an M1-common failure, with explicit non-destructive boundary. |
| Deferred | Valid architecture work, but not needed to claim M1. | Keep as post-M1 or future milestone candidate. |

## M1 Coverage Matrix

| M1 area | Existing diagnostic or repair surface | Evidence | Classification | M1 gap decision |
| --- | --- | --- | --- | --- |
| Release ladder and evidence recording | `scripts/m1-release-ladder.ps1` modes plus `docs/deployment/M1_LADDER_EVIDENCE_MATRIX.md` record docs, smoke, fast, release-fast, playback, container, postgres, workspace, and all-mode evidence. | `.trellis/tasks/archive/2026-06/06-06-m1-release-ladder-runner/`; `.trellis/tasks/archive/2026-06/06-06-m1-ladder-evidence-matrix/` | Shipped | Move `m1-ladder-evidence-matrix` to completed evidence. Future release failures should route by failed ladder mode, not by reopening the matrix. |
| Operator journey smoke | Focused M1 smoke composes docs-safe release gate, server self-host smoke, Admin Web route/media tests, and Media Web route/player assertions. | `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/` | Shipped | Adequate for deterministic local M1 confidence. Live-browser release proof remains optional future ladder breadth. |
| Library scan and source identity pressure | Source fingerprint hash execution, durable job contract, scheduler integration, evidence persistence, Admin overview/Jobs diagnostics, Admin manual enqueue, retry/requeue, and scan-originated triggering are shipped. | `docs/architecture/STORAGE_VFS.md`; `docs/architecture/CONTROL_PLANE.md`; `.trellis/tasks/archive/2026-06/06-06-scan-originated-source-hash-triggering/`; `.trellis/tasks/archive/2026-06/06-06-admin-source-fingerprint-hash-trigger-first-slice/`; `.trellis/tasks/archive/2026-06/06-06-source-hash-retry-requeue-admin-command/` | Shipped | No immediate M1 source-hash task. Automatic duplicate reconciliation remains deferred until policy/undo evidence exists. |
| Source duplicate repair | Admin plan/apply routes create redaction-safe Suggested Source Duplicate Relationships; Admin Web exposes the source-scoped operator flow with explicit confirmation. | `.trellis/tasks/archive/2026-06/06-06-admin-source-duplicate-reconciliation-plan-api/`; `.trellis/tasks/archive/2026-06/06-06-admin-source-duplicate-reconciliation-apply-first-slice/`; `.trellis/tasks/archive/2026-06/06-06-source-duplicate-reconciliation-operator-flow/` | Shipped | M1 repair story is covered for duplicate-source suggestion. Do not open automatic merge, confirm/reject/undo, or Media Item merge for M1 unless a ladder failure proves it blocks the operator journey. |
| VFS cache repair pressure | Storage staging diagnostics, latest-failure preview/action plan, target inventory, target-scoped preview, selected-target refresh, and read-only remediation plan exist. | `docs/architecture/STORAGE_VFS.md`; `.trellis/tasks/archive/2026-06/06-06-vfs-cache-repair-non-destructive-remediation-plan-first-slice/`; predecessor VFS cache repair tasks linked there | Adequate for M1 | M1 has non-destructive visibility and selected refresh. Durable repair queues, purge/delete/invalidation, backend configuration mutation, and automated repair workers remain M2 reliability work unless a release ladder storage failure is unrecoverable without them. |
| Storage staging/backend health | Admin storage staging and backend diagnostics expose redaction-safe pressure and policy slices without local paths, Source Locators, etags, raw backend URLs, or credentials. | `docs/api/HTTP_API.md`; `docs/architecture/STORAGE_VFS.md`; `.trellis/tasks/archive/2026-06/06-03-05a-staging-budget-per-backend-policy/` | Shipped | No new M1 task. Route storage failures to focused VFS/cache work only when the ladder produces a concrete blocked scan/playback case. |
| Job queue visibility and cancellation | Admin Jobs list, filters, job pressure summaries, and cancellation behavior are documented. Source-hash jobs reuse generic redacted job rows and source-specific retry. | `docs/api/HTTP_API.md`; `docs/architecture/CONTROL_PLANE.md`; `.trellis/tasks/archive/2026-06/06-06-source-hash-retry-requeue-admin-command/` | Adequate for M1 | General durable retry UI and broader job-kind scheduler migration are post-M1 unless a release-candidate run exposes an untriageable stuck job class. |
| Playback runtime diagnostics | Admin playback runtime diagnostics, playback release-gate mode, and hardware report baseline expose FFmpeg/FFprobe and hardware readiness safely. | `docs/architecture/OPERATIONS_RELEASE.md`; `docs/architecture/PLAYBACK.md`; `.trellis/tasks/archive/2026-06/06-05-06-05-playback-release-hardware-report/` | Adequate for M1 | Keep `media-web-library-browse-and-player-smoke` conditional. Open it only for a concrete browse/play failure from the M1 ladder or release playback mode. |
| Catalog governance repair context | Admin catalog governance exposes queued item repair context and action codes without raw provider payloads or unsafe source facts. | `docs/api/HTTP_API.md`; M1 smoke route coverage in `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/` | Adequate for M1 | No broad catalog rewrite. Open a focused task only for a specific catalog repair action missing from the M1 operator path. |
| Metadata Candidate Review governance | Candidate Review detail, item/global queues, batch plan/apply, durable batch execution, and related hierarchy Admin plan/apply are shipped. Web coverage exists for the main review and batch flows; related hierarchy Web UX remains deferred. | `docs/architecture/LIBRARY_PIPELINE.md`; `docs/architecture/CONTROL_PLANE.md`; provider governance workstreams and archived Trellis tasks linked from those maps | Adequate for M1 | Provider governance mutation undo and related hierarchy Web polish are valid M4 tasks, not M1 blockers unless the video-first operator smoke needs them. |
| Generated Artifact apply recovery | Generated Artifact apply recovery read paths, Web recovery UI, and preparation-first repair seam are shipped. | `docs/architecture/CONTROL_PLANE.md`; `docs/architecture/LIBRARY_PIPELINE.md`; `docs/workstreams/generated-artifact-apply-operations-repair/`; `docs/workstreams/generated-artifact-apply-repair-actions/` | Adequate for M1 | One-click repair wrapper and Web copy polish remain deferred. Do not add a second metadata apply executor. |
| Managed artwork repair and cleanup | Failed Managed Artwork ingests can be requeued; orphan/lifecycle diagnostics remain read-only or dry-run where documented. | `docs/api/HTTP_API.md`; artwork lifecycle task evidence linked from `docs/architecture/LIBRARY_PIPELINE.md` | Adequate for M1 | Missing artifact repair, cleanup automation, and selected-artwork invalidation policy are post-M1 unless release evidence shows artwork failure blocks browsing/playback. |
| System/config diagnostics | Admin system config diagnostics expose sanitized endpoint/config/database/concurrency/runtime facts without database URLs or secret values. | `docs/api/HTTP_API.md`; `docs/architecture/OPERATIONS_RELEASE.md` | Shipped | No M1 task. Hot-apply/restart-required modeling remains a future operations hardening topic. |
| Incident bundles and realtime diagnostics | Safe realtime diagnostics and redacted incident bundles are still future control-plane work. | `docs/architecture/CONTROL_PLANE.md` | Deferred | Not an M1 blocker. M1 requires focused redaction assertions and Admin-visible diagnostics, not a full incident export system. |

## Follow-On Routing

Open follow-ons only from these evidence-backed conditions:

| Condition | Candidate task | Owner lane | Open when |
| --- | --- | --- | --- |
| M1 ladder or playback mode exposes a concrete Media Web browse/play blocker. | `media-web-library-browse-and-player-smoke` | web-product + playback-transcode | Browser/player failure is reproducible from `fast`, `smoke`, or `playback` evidence. |
| Release-candidate evidence shows Admin Web cannot discover or execute an already-shipped Admin repair route. | `admin-web-repair-flow-gap-<surface>` | web-product + owning backend lane | The backend route exists, but the operator journey cannot reach it. |
| VFS cache failures remain unrecoverable with selected-target refresh and read-only remediation planning. | `vfs-cache-durable-repair-queue-first-slice` | storage-vfs + control-plane | A real release ladder storage failure requires durable non-destructive repair, not just diagnostics. |
| Operators cannot triage a failed durable job class from Admin Jobs plus feature-specific retry/requeue. | `admin-jobs-retry-and-drilldown-gap-<job-kind>` | control-plane + owning feature lane | A concrete stuck job class lacks safe retry, cancellation, or enough redacted detail. |
| Provider governance undo becomes necessary for M1 catalog correctness. | `provider-governance-mutation-undo` | library-metadata-control-plane | M1 evidence shows local authority can be harmed without undo. Otherwise keep it M4. |

## Decision

The M1 Admin diagnostics and repair gap is not a broad missing platform. The
current evidence shows enough coverage for Product-Operator M1 if release
candidate runs keep passing:

- source identity and duplicate repair have both backend and Admin Web flows;
- storage/VFS cache repair has non-destructive diagnostics, planning, and
  selected refresh;
- jobs, playback runtime, system config, catalog governance, metadata
  governance, generated artifact recovery, and managed artwork each have
  bounded Admin visibility or repair commands appropriate to M1;
- remaining work should be opened from failed ladder evidence, not from stale
  candidate queues.

## Alternatives Considered

### Option A: Evidence-Driven Coverage Matrix (Chosen)

Pros:

- Matches the release ladder model: run evidence first, then route failures.
- Prevents reopening shipped source hash, duplicate repair, and release matrix
  work.
- Keeps M1 narrow without losing post-M1 architecture follow-ons.

Cons:

- Does not itself improve a UI or backend path.

Decision: chosen because current evidence points to routing discipline rather
than an obvious missing implementation slice.

### Option B: Open Media Web Browse/Player Smoke Immediately

Pros:

- Would add more browser-facing confidence.

Cons:

- The existing fast ladder did not expose a browser/player blocker.
- It would violate the current rule that Media Web/player work should be
  blocker-driven.

Decision: rejected until ladder evidence identifies a concrete browse/play
failure.

### Option C: Start A Broad Admin Repair Platform

Pros:

- Could unify repair commands, retry UI, and diagnostics presentation.

Cons:

- Over-scoped for M1.
- Risks duplicating existing feature-owned repair boundaries and weakening
  redaction semantics.

Decision: rejected. Keep repairs feature-owned and use ADR 0053 control-plane
rules for durable background work.

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Matrix hides a real UI gap | Medium | Medium | Require follow-ons when release evidence proves a shipped Admin route is not reachable from the operator journey. |
| Operators treat read-only diagnostics as repair completion | Medium | Medium | Classify VFS remediation and incident bundles explicitly as read-only/deferred where mutation is not shipped. |
| Future agents reopen completed source hash or duplicate repair work | Medium | Low | Keep completed evidence in roadmap and lanes; route only from failing ladder modes. |
| Redaction regressions appear in new diagnostics | High | Low | Keep Admin DTO rules in `.trellis/spec/nako-api/backend/admin-and-public-contracts.md` and run focused redaction tests when contracts change. |

## Success Criteria

| Criterion | Target | Measurement |
| --- | --- | --- |
| Completed evidence separated from candidates | `m1-ladder-evidence-matrix` and shipped repair slices appear only as evidence, not next work. | Review `docs/ROADMAP.md`, `docs/GOALS.md`, and `docs/architecture/LANES.md`. |
| M1 repair gaps are evidence-backed | Every proposed follow-on has an opening condition tied to ladder or release-candidate evidence. | Review this matrix. |
| Scope containment | No Rust, TypeScript, generated contract, schema, runtime, or script changes in this audit slice. | `git status --short`. |
| Redaction discipline preserved | Matrix routes new Admin DTO/route work back to existing spec gates. | Review spec links and task context. |
