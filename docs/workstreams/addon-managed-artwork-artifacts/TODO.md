# Addon Managed Artwork Artifacts TODO

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] AMAA-010 [owner=planner] [deps=APW-060] [scope=docs/workstreams/addon-managed-artwork-artifacts,docs/workstreams/addon-protected-writes,docs/workstreams/README.md]
  Goal: Open the focused artwork/artifact follow-on lane split from APW.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, APW closeout docs.
  Handoff: Continue with AMAA-020 before accepting `artwork_write` payloads.

## M1 - Artwork Seam Audit

- [x] AMAA-020 [owner=codex] [deps=AMAA-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs]
  Goal: Audit current artwork, artifact, storage/VFS, catalog image hydration,
  and Addon Side Effect seams; choose the first bounded `artwork_write` target.
  Validation: `rg -n "artwork|ImageAsset|ArtworkTask|Managed Artwork|Taru-Managed Artifact|artwork_write|thumbnail|cache_uri|source_uri" crates docs`; `git diff --check`.
  Review: decide whether first apply should create Artwork Candidates, import
  Managed Artwork, or create Taru-Managed Artifacts. If catalog-visible artwork
  state needs multiple durable writes, introduce or reuse a first-party artwork
  commit boundary instead of placing write ordering in the Addon handler.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`; `DESIGN.md` selected
  Addon Artwork Candidate proposal as the first bounded apply target.
  Handoff: Continue with AMAA-030 by introducing a typed candidate proposal
  boundary. Do not directly create selected public `ImageAsset` rows, managed
  cache artifacts, or sidecar files in the first slice.

## M2 - First Artwork Apply Slice

- [x] AMAA-030 [owner=codex] [deps=AMAA-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs/api]
  Goal: Implement the smallest safe `artwork_write` apply path selected by
  AMAA-020: a MediaItem-targeted Addon Artwork Candidate proposal that records
  addon artwork intent without exposing raw source details as public client
  artwork.
  Validation: focused artwork/addon tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no response exposes raw payload, Source Locators, filesystem
  paths, remote storage handles, or unsafe provider hotlinks as client artwork.
  Route any sidecar-file export behavior to
  `addon-library-file-write-policy` rather than adding a parallel writer here.
  Evidence: code/tests/API docs and AMAA notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Candidate proposal is implemented. Continue with AMAA-040 to close
  or split image processing, thumbnailing, selected-artwork workflow, managed
  cache/artifact import, and sidecar export into narrower follow-ons.

## M3 - Closeout Or Split

- [x] AMAA-040 [owner=planner] [deps=AMAA-030] [scope=docs/workstreams/addon-managed-artwork-artifacts,docs/api,docs/workstreams/managed-artwork-ingest-selection]
  Goal: Close the artwork/artifact lane or split remaining cache/export
  breadth into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: no blocking findings. AMAA-030 satisfies the selected first-slice
  contract without exposing raw payloads, Source Locators, filesystem paths,
  remote handles, cache URIs, or unverified addon URLs as public client
  artwork. The Addon handler records an internal candidate and does not own
  fetch/cache/thumbnail/selection ordering.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane is closed. Continue with
  `managed-artwork-ingest-selection` when the next product priority is
  accepting candidates into Taru-managed cached/selected public artwork.
