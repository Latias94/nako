# Managed Artwork Ingest Selection TODO

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] MAIS-010 [owner=planner] [deps=AMAA-040] [scope=docs/workstreams/managed-artwork-ingest-selection,docs/workstreams/addon-managed-artwork-artifacts,docs/workstreams/README.md]
  Goal: Open the focused managed artwork ingest/selection follow-on lane split
  from AMAA.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, AMAA closeout docs.
  Handoff: Continue with MAIS-020 before accepting candidates into public
  artwork or managed cache/artifact storage.

## M1 - Managed Artwork Seam Audit

- [x] MAIS-020 [owner=codex] [deps=MAIS-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs]
  Goal: Audit current candidate, managed artifact/cache, artwork task,
  staging, catalog image hydration, and API seams; choose the first candidate
  acceptance target.
  Validation: `rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs`; `git diff --check`.
  Review: decide whether first acceptance should create an unselected managed
  artifact, selected public artwork, or queued candidate-ingest job. If
  catalog-visible artwork state needs multiple durable writes, introduce or
  reuse a first-party artwork/catalog commit boundary instead of placing write
  ordering in HTTP handlers.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`; selected target in
  `DESIGN.md`.
  Result: DONE. First target is a queued candidate-ingest boundary that creates
  internal Managed Artwork state, not selected public `ImageAsset` rows.
  Handoff: Continue with MAIS-030 by adding the managed ingest/job/artifact
  model and redacted admin response before any public artwork publication.

## M2 - First Managed Ingest Slice

- [x] MAIS-030 [owner=codex] [deps=MAIS-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs/api]
  Goal: Implement a Taru-owned queued candidate-ingest path that accepts an
  Addon Artwork Candidate into internal Managed Artwork state without
  publishing selected public artwork yet.
  Validation: focused managed artwork tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify durable job input/summary, admin response, addon response, and
  public responses do not expose raw candidate source URLs, Source Locators,
  filesystem paths, internal cache paths, remote storage handles, cache URIs,
  or unvalidated addon hotlinks.
  Evidence: code/tests/API docs and MAIS notes in `EVIDENCE_AND_GATES.md`.
  Result: DONE. Admin candidate accept now marks the candidate accepted,
  creates an internal managed artwork ingest record, and queues a redacted
  `managed_artwork_ingest` job without writing public `ImageAsset` rows.
  Handoff: Continue with MAIS-040 to close this lane or split remote fetch,
  artifact byte storage, thumbnailing, and public artwork publication into
  narrower follow-ons.

## M3 - Closeout Or Split

- [x] MAIS-040 [owner=planner] [deps=MAIS-030] [scope=docs/workstreams/managed-artwork-ingest-selection,docs/api]
  Goal: Close the managed artwork ingest lane or split remaining
  thumbnail/admin/artifact breadth into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: no blocking findings. MAIS-030 satisfies the selected first-slice
  contract: candidate acceptance is first-party, creates internal managed
  ingest state and a durable redacted job, and does not fetch remote bytes,
  write public `ImageAsset` rows, select artwork, or expose source/cache
  internals.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane is closed as the queued candidate-ingest boundary. Continue
  with a follow-on for managed artwork fetch/content validation and artifact
  byte storage before public image serving, thumbnails, or selected artwork
  publication.
