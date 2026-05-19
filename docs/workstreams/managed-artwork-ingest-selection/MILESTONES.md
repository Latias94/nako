# Managed Artwork Ingest Selection Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Outcome: the managed artwork follow-on is split from AMAA with clear authority
and gates.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- AMAA closeout points to this lane.
- Workstream index links the new lane.

Primary evidence:

- `docs/workstreams/managed-artwork-ingest-selection/DESIGN.md`
- `docs/workstreams/addon-managed-artwork-artifacts/HANDOFF.md`

## M1 - Managed Artwork Seam Audit

Outcome: candidate, artifact/cache, artwork task, catalog image, and API seams
are classified before accepting candidates into public artwork.

Exit criteria:

- Candidate repository, `ImageAsset`, `ArtworkTask`, VFS/cache, staging,
  catalog hydration, and public/admin DTO seams are inventoried.
- First acceptance target is selected: unselected managed artifact, selected
  public artwork, or queued candidate ingest.
- ADR amendment need is accepted, rejected, or split.

Primary gates:

- `rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs`
- `git diff --check`

## M2 - First Managed Ingest Slice

Outcome: one candidate acceptance path creates Taru-managed artwork state
without leaking raw candidate source details.

Exit criteria:

- The command is first-party and does not run inside Addon proposal handling.
- Remote fetch/cache/validation semantics are truthful and bounded.
- Public artwork publication and selected state have redaction tests.

Primary gates:

- focused managed artwork tests
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 - Closeout Or Split

Outcome: managed artwork ingest/selection is complete enough to close, or
thumbnail/artifact/admin review breadth is split to narrower lanes.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped behavior.
- Residual thumbnail, admin review, or sidecar export work is completed,
  deferred, or split.

Result:

- Closed after MAIS-030 shipped the queued candidate-ingest boundary.
- Remote fetch/content validation, managed artifact byte storage, public image
  serving/redacted references, thumbnails, and selected artwork publication are
  split to follow-on work.
