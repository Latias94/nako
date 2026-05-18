# Addon Managed Artwork Artifacts TODO

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] AMAA-010 [owner=planner] [deps=APW-060] [scope=docs/workstreams/addon-managed-artwork-artifacts,docs/workstreams/addon-protected-writes,docs/workstreams/README.md]
  Goal: Open the focused artwork/artifact follow-on lane split from APW.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, APW closeout docs.
  Handoff: Continue with AMAA-020 before accepting `artwork_write` payloads.

## M1 - Artwork Seam Audit

- [ ] AMAA-020 [owner=codex] [deps=AMAA-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs]
  Goal: Audit current artwork, artifact, storage/VFS, catalog image hydration,
  and Addon Side Effect seams; choose the first bounded `artwork_write` target.
  Validation: `rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs`; `git diff --check`.
  Review: decide whether first apply should create Artwork Candidates, import
  Managed Artwork, or create Taru-Managed Artifacts.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with AMAA-030 only after fetch/cache/storage policy is
  explicit.

## M2 - First Artwork Apply Slice

- [ ] AMAA-030 [owner=codex] [deps=AMAA-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs/api]
  Goal: Implement the smallest safe `artwork_write` apply path selected by
  AMAA-020.
  Validation: focused artwork/addon tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no response exposes raw payload, Source Locators, filesystem
  paths, remote storage handles, or unsafe provider hotlinks as client artwork.
  Evidence: code/tests/API docs and AMAA notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Split image processing, thumbnailing, selected-artwork workflow, or
  sidecar export if it exceeds the first apply slice.

## M3 - Closeout Or Split

- [ ] AMAA-040 [owner=planner] [deps=AMAA-030] [scope=docs/workstreams/addon-managed-artwork-artifacts,docs/api]
  Goal: Close the artwork/artifact lane or split remaining cache/export
  breadth into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane only after artwork/artifact authority and
  redaction guarantees are stable.
