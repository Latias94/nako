# 2026 Roadmap Reconciliation And M1 Release Convergence

## Goal

Reconcile Nako's top-level roadmap into a product-release-oriented plan that
connects the long-term Jellyfin/Plex-class ambition with the next executable
M1 release convergence queue. The immediate outcome is a clear docs-only
planning update that tells future agents which capability lanes matter now,
which breadth is deferred, and what the next implementation tasks should be.

## What I Already Know

- Nako is a self-hosted media server backend with a video-first current phase
  and a broader long-term media-server scope.
- Existing architecture documents define strong lane boundaries across
  storage/VFS, playback/transcode, metadata governance, web product,
  control-plane, operations/release, and client surfaces.
- `docs/ROADMAP.md` and `docs/GOALS.md` currently contain a large amount of
  historical completed-lane evidence. They are useful for traceability but make
  the next release direction hard to scan.
- `docs/architecture/LANES.md` already says there is no active implementation
  lane and points to candidate next actions across storage, playback, metadata,
  web, and control-plane.
- `docs/workstreams/mvp-release-shape/CLOSEOUT.md` says the first
  self-hosted, video-first, single-admin MVP had a validated release ladder,
  but future release execution should happen as focused Trellis tasks.
- Recent work advanced source fingerprint hashing, source duplicate
  reconciliation, Admin diagnostics, and storage/source identity boundaries.
- The current working tree already contains unrelated Trellis archive moves.
  This task must not restore, delete, or commit those unrelated changes unless
  a separate explicit decision is made.

## Assumptions

- The roadmap should be reorganized around user-visible release milestones,
  not around crate names or historical workstream IDs.
- The first planning target should be an M1 release convergence plan for a
  self-hosted, video-first, single-admin/operator journey.
- Long-term M2-M5 breadth should stay visible, but should not compete with the
  M1 implementation queue.
- This task is docs/planning only: no Rust, TypeScript, schema, route, or
  runtime changes.

## Open Questions

- Resolved: M1 should use Product-Operator M1 as the roadmap anchor. The
  primary user journey is a real self-hosted operator configuring one library,
  scanning media, browsing catalog entries, playing video, and diagnosing or
  repairing common failures. The backend release ladder remains the quality
  gate for this journey rather than the product definition.

## Requirements

- Update roadmap planning docs so they clearly separate:
  - current release convergence target;
  - next implementation queue;
  - longer-term capability milestones;
  - deferred breadth and non-goals.
- Anchor M1 around the Product-Operator journey:
  - configure one Media Library;
  - scan and index media;
  - browse catalog entries;
  - play a video through the best available Direct Play, Remux, or HLS path;
  - use Admin diagnostics and repair actions when scan, storage, source
    identity, metadata, playback, or jobs fail.
- Treat the release ladder as the M1 quality gate, including install/config,
  scan, metadata, playback, Admin diagnostics, redaction, packaging/container,
  and focused smoke evidence.
- Preserve historical traceability to completed workstreams and Trellis task
  evidence without making historical completion logs the primary reading path.
- Define a milestone ladder similar to:
  - M0: roadmap and planning hygiene;
  - M1: video-first single-admin usable release convergence;
  - M2: large-library reliability and storage/scan resilience;
  - M3: playback/transcode maturity;
  - M4: metadata governance maturity;
  - M5: addon ecosystem maturity.
- Identify the recommended first M1 implementation queue with 3-5 focused
  task candidates and lane ownership.
- Keep active lane routing in `docs/architecture/LANES.md` consistent with the
  chosen roadmap direction.
- Keep `docs/architecture/WORKSTREAM_LINKS.md` as navigation/evidence, not as
  the primary product roadmap.
- Avoid opening implementation workstreams; new execution should use Trellis
  tasks.

## Acceptance Criteria

- [ ] `docs/ROADMAP.md` exposes a concise current plan before historical
      sections.
- [ ] `docs/GOALS.md` records the roadmap reconciliation goal with objective,
      deliverables, non-goals, exit criteria, and evidence.
- [ ] `docs/architecture/LANES.md` names the selected next planning focus and
      current lane candidate queue.
- [ ] The docs agree that M1 is product-operator-first and that the backend
      release ladder is the quality gate.
- [ ] Proposed follow-on task names are concrete enough to become Trellis
      tasks.
- [ ] Historical evidence links remain available and are not deleted.
- [ ] No code, schema, generated contract, or runtime behavior changes are
      included.

## Definition Of Done

- `git diff --check` passes for the touched docs and task files.
- Markdown changes are internally consistent and link to existing documents
  where useful.
- Trellis task context files validate.
- Any remaining unrelated dirty working-tree changes are explicitly reported
  and left untouched.

## Out Of Scope

- No implementation tasks in Rust, TypeScript, schema migrations, API routes,
  generated contracts, or runtime supervisors.
- No release artifact publication.
- No new workstream directories.
- No Public Client API design.
- No Addon Manager implementation.
- No automatic cleanup of unrelated existing Trellis archive moves.

## Technical Notes

- Relevant docs inspected:
  - `CONTEXT.md`;
  - `docs/ARCHITECTURE.md`;
  - `docs/ROADMAP.md`;
  - `docs/GOALS.md`;
  - `docs/architecture/LANES.md`;
  - `docs/architecture/WORKSTREAM_LINKS.md`;
  - `docs/workstreams/mvp-release-shape/CLOSEOUT.md`.
- Relevant skills:
  - `trellis-brainstorm` for task-first requirements discovery;
  - `technical-spec` for design-doc structure and alternatives;
  - `plan-engineering-program` for lane-level roadmap planning.

## Candidate Approaches

### Option A: Product-Operator M1 (Recommended)

Make M1 the first release where a home/self-hosted operator can configure one
library, scan media, browse catalog entries, play video, and diagnose/repair
common failures through Admin surfaces.

Pros:
- Aligns roadmap with user-visible value.
- Forces storage, scan, playback, catalog, and diagnostics to converge.
- Keeps future breadth honest by requiring a complete journey.

Cons:
- Requires careful scope control because every lane has tempting follow-ons.
- May defer some backend release-gate automation that is useful for developers.

### Option B: Backend Release Ladder M1

Make M1 primarily a release engineering target: commandable validation ladder,
packaging, container gates, config checks, and smoke evidence.

Pros:
- Easier to verify mechanically.
- Reduces release risk before broader product polish.
- Fits the existing MVP release ladder evidence.

Cons:
- Can produce a technically validated release that still feels incomplete to a
  real operator.
- May keep Web/player/operator workflows under-prioritized.

### Option C: Capability Breadth Roadmap

Keep advancing the strongest open capability lanes in parallel: playback
device profiles, metadata undo, source hash automation, addon lifecycle, and
web player UX.

Pros:
- Maximizes architecture depth.
- Lets each lane continue from existing momentum.

Cons:
- Risk of many strong subsystems without a coherent release cut.
- Harder to decide what is truly blocking.

## Recommendation

Use Option A as the roadmap anchor and borrow Option B as M1's validation
ladder. Treat Option C as M2+ breadth unless a capability is directly required
for the M1 operator journey.
