# Library Watcher Planned-Write Suppression And Fallback Reconciliation

## Goal

Continue the watcher runtime productization after 06a by making host-owned
library writes and unreliable watcher signals safe. The first useful slice
should prevent Nako's own file writes from creating watcher loops while leaving
clear follow-ons for degraded watcher reconciliation and operator diagnostics.

## What I Already Know

* `06-03-06a-library-watcher-runtime-productization` established the supervised
  watch-folder runtime direction and tied watcher work to existing
  server/control-plane boundaries.
* `06-03-06c-targeted-jellyfin-watcher-reference` concluded that watcher events
  should be treated as reconciliation hints, not authoritative media lifecycle
  facts.
* 06c specifically called out host-owned write suppression with scope, owner,
  reason, TTL, completion semantics, optional reconciliation intent, and
  redaction-safe diagnostics.
* Nako already has stable-candidate intake evidence for watch-folder discovery
  and must preserve that contract for slow copies, NAS paths, stale cache, and
  remote-like local mounts.
* ADR 0053 control-plane rules still apply: watcher work must stay supervised
  and should hand durable scan/intake work to existing authorities rather than
  adding a hidden executor.

## Assumptions

* The MVP should prefer one durable product slice over a broad watcher rewrite.
* Planned-write suppression is the best first slice because NFO, artwork,
  sidecar, import, and future VFS write workflows can otherwise self-trigger
  watcher intake loops.
* Fallback/degraded reconciliation can be modeled in the same design language
  but may remain out of scope unless needed for the suppression API shape.

## Requirements

* Define a Nako-native planned-write suppression boundary for watcher-visible
  library writes.
* Suppression facts must include enough structure to make behavior auditable:
  scope, owner/workflow, reason, TTL or expiry, completion behavior, and whether
  completion should enqueue or mark reconciliation intent.
* Suppression matching must work in source-locator or `StorageUri` terms, not
  by exposing or trusting raw local paths.
* Suppressed watcher events must avoid duplicate intake/scan work while still
  allowing explicit completion-driven reconciliation when appropriate.
* Runtime and Admin diagnostics must stay redaction-safe: no raw paths,
  credentials, source fingerprints, raw locators, provider errors, or backend
  secrets.
* Preserve scheduled reconciliation as the correction path for missed,
  unreliable, permission-blocked, or degraded watcher events.

## Acceptance Criteria

* [x] Planned-write suppression behavior is specified with owner, scope, TTL,
      completion, and reconciliation semantics.
* [x] The PRD chooses whether this task implements suppression only, fallback
      state only, or a combined thin slice.
* [x] Implementation context points to the relevant watcher/runtime,
      stable-candidate, VFS redaction, and control-plane specs.
* [x] Out-of-scope watcher work is explicit enough to open follow-up tasks
      without rediscovering the same decisions.
* [x] Suppressed watch-folder observations do not update intake candidates,
      advance stable-candidate evidence, or enqueue library scan jobs.
* [x] Admin discovery diagnostics expose suppression count and active
      suppression summaries without raw paths or source locators.

## Selected MVP Direction

**Planned-write suppression first.**

Implement the narrow contract that lets Nako-owned write workflows bracket a
watcher-visible source scope and prevent self-triggered watcher loops. Completion
can optionally report reconciliation intent through the existing supervised
watch-folder/intake path.

Pros:

* Solves a concrete product correctness issue before broad watcher expansion.
* Creates the suppression vocabulary needed by NFO/artwork/sidecar/import write
  workflows.
* Keeps fallback/degraded reconciliation as a clean follow-up instead of mixing
  multiple watcher failure modes into one task.

Cons:

* Does not fully solve watcher overflow, permission failure, or backend
  capability degradation in the same slice.

## Decision (ADR-lite)

Context: 06a created the supervised watcher runtime slice, and 06c identified
planned host writes as a watcher-loop risk. Nako needs a suppression vocabulary
before NFO, artwork, sidecar, import, and future VFS write workflows can safely
write into watched library scopes.

Decision: this task implements planned-write suppression first. It may model
completion-driven reconciliation intent only as needed to finish suppression
semantics, but it does not implement broad degraded watcher state.

Consequences: the first follow-up stays small and concrete. Watcher overflow,
permission failure, backend unreliability, and Admin degraded-state dashboards
remain separate follow-ons.

## Alternatives Considered

**Fallback/degraded reconciliation first.**

Model watcher health states, pending reconciliation scopes, and degraded reasons
before adding planned-write suppression. This helps Admin/operator visibility,
but it does not immediately prevent self-triggered write loops.

**Combined thin slice.**

Add a small suppression contract plus minimal degraded/pending reconciliation
state. This reduces future glue work but risks making the first follow-up too
wide if persistence, Admin DTOs, and runtime behavior all change together.

## Definition Of Done

* Tests are added or updated for the selected watcher behavior.
* `cargo fmt --all -- --check` passes.
* Focused `cargo check` / `cargo nextest run` gates pass for touched packages.
* PostgreSQL harness is run if shared persistence contracts change.
* Docs/spec/task evidence are updated for durable watcher behavior.
* `git diff --check` passes.

## Out Of Scope

* No broad Jellyfin audit or code copying from reference projects.
* No raw `tokio::spawn` watcher runtime outside ADR 0053 supervision.
* No promise that remote backends provide trustworthy watch events unless VFS
  capability facts support it.
* No full scan scheduler rewrite.
* No broad degraded watcher state, watcher overflow handling, permission
  failure dashboard, or backend capability matrix beyond what is required for
  planned-write suppression.
* No exposure of raw paths, raw source locators, credentials, fingerprints, or
  backend/provider error strings in public or Admin diagnostics.
* No PR requirement for this repository while it has no external users.

## Technical Notes

* 06a reference: `.trellis/tasks/archive/2026-06/06-03-06a-library-watcher-runtime-productization/prd.md`
* 06c evidence: `.trellis/tasks/archive/2026-06/06-03-06c-targeted-jellyfin-watcher-reference/evidence.md`
* Architecture references: `docs/architecture/LIBRARY_PIPELINE.md`,
  `docs/architecture/STORAGE_VFS.md`, `docs/architecture/CONTROL_PLANE.md`
* Likely code areas after implementation starts:
  `crates/nako-server/src/app/watch_folder_runtime.rs`,
  `crates/nako-server/src/app/acquisition_intake.rs`,
  `crates/nako-library/src/intake.rs`, and VFS/storage capability modules.
