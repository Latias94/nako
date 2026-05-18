# Addon Managed Artwork Artifacts Milestones

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Outcome: artwork/artifact protected writes are split from APW with clear
authority and gates.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- APW closeout points to this lane.
- Workstream index links the new lane.

Primary evidence:

- `docs/workstreams/addon-managed-artwork-artifacts/DESIGN.md`
- `docs/workstreams/addon-protected-writes/HANDOFF.md`

## M1 - Artwork Seam Audit

Outcome: current artwork, artifact, storage, catalog, and task seams are
classified before accepting `artwork_write`.

Exit criteria:

- `ImageAsset`, `ArtworkTask`, storage/VFS, catalog image hydration, and Addon
  Side Effect apply outcome boundaries are inventoried.
- The first artwork apply target is selected with risk notes.
- ADR amendment need is accepted, rejected, or split.

Primary gates:

- `rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs`
- `git diff --check`

## M2 - First Artwork Apply Slice

Outcome: one accepted `artwork_write` side effect can safely create or update
an artwork/artifact record through Taru-owned seams.

Exit criteria:

- The payload is normalized into a bounded Taru artwork command.
- The command does not expose raw Source Locators, filesystem paths, or remote
  storage handles.
- Apply outcome, idempotency, provenance, and redacted response behavior are
  tested.

Primary gates:

- focused artwork/addon tests
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 - Closeout Or Split

Outcome: artwork/artifact behavior is complete enough to close, or image
processing/cache/export breadth is split to narrower lanes.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped `artwork_write` behavior.
- Residual image-processing, thumbnail, or sidecar export work is completed,
  deferred, or split.
