# Addon Protected Writes TODO

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] APW-010 [owner=planner] [deps=ATGSE-060] [scope=docs/workstreams/addon-protected-writes,docs/workstreams/addon-token-grants-side-effects,docs/workstreams/README.md]
  Goal: Split the concrete protected-write apply work from the completed
  Addon Token Grants Side Effects lane with problem, target state, non-goals,
  gates, and first executable audit task.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, `docs/workstreams/README.md`.
  Handoff: Continue with APW-020 before applying metadata, artwork, subtitle,
  NFO, or Library File Write behavior.

## M1 - Protected Write Seam Audit

- [ ] APW-020 [owner=codex] [deps=APW-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-metadata,crates/taru-catalog,crates/taru-nfo,crates/taru-vfs,docs]
  Goal: Audit current Addon Side Effect intake, Canonical Metadata merge,
  catalog commit, Managed Artwork, subtitle, NFO, and storage/VFS write seams;
  choose the first concrete protected-write apply target.
  Validation: `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`; `git diff --check`.
  Review: no ADR amendment is required if the next slice preserves ADR 0020
  and Taru-owned write boundaries; split an ADR only for direct storage
  authority, Public Client write APIs, or OAuth-first authorization.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with APW-030 if Canonical Metadata is still the safest
  first apply slice; otherwise update this ledger with the narrower target.

## M2 - Canonical Metadata Apply Slice

- [ ] APW-030 [owner=codex] [deps=APW-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-metadata,crates/taru-catalog,docs/api]
  Goal: Implement the smallest concrete `metadata_write` Addon Side Effect
  apply path that turns an accepted intake record into a Taru-owned Canonical
  Metadata update while preserving merge policy, idempotency, audit, redaction,
  and catalog/search consistency.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests`; focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`; relevant metadata/catalog tests; `cargo fmt --all -- --check`; `git diff --check`.
  Review: review-workstream must check that HTTP handlers do not own metadata
  merge logic and that responses do not leak raw payloads, provenance, Source
  Locators, filesystem paths, or provider bodies.
  Evidence: code/tests/docs proving allowed apply, denied apply, duplicate
  idempotency, failed validation, redacted response, and catalog/search update.
  Handoff: Split field breadth if the payload schema grows beyond a minimal
  video-first Canonical Metadata slice.

## M3 - Managed Artwork And Artifact Intake

- [ ] APW-040 [owner=codex] [deps=APW-030] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-vfs,docs/api]
  Goal: Define and, if bounded, implement the first `artwork_write` path from
  Addon Side Effect into Artwork Candidate, Managed Artwork, or
  Taru-Managed Artifact storage without hotlinking unsafe provider URLs or
  exposing library paths.
  Validation: focused artwork/addon tests selected after APW-020; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `git diff --check`.
  Review: verify resource budgets, external fetch ownership, artifact
  provenance, and redacted response shape.
  Evidence: artifact/artwork model notes, tests, and HTTP API docs.
  Handoff: Split a dedicated artwork/artifact lane if image processing,
  thumbnailing, or cache policy becomes the dominant scope.

## M4 - Subtitle, NFO, And Library File Write Policy

- [ ] APW-050 [owner=codex] [deps=APW-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-nfo,crates/taru-vfs,docs/api]
  Goal: Route addon-initiated subtitle, NFO, and sidecar-asset writes through
  Library File Write policy, NFO Round Trip, backup retention, and storage/VFS
  boundaries instead of raw path writes.
  Validation: focused NFO/storage/addon tests selected after APW-020; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no Addon response or audit summary exposes raw Source
  Locators, filesystem paths, remote storage handles, or unredacted file-write
  payloads.
  Evidence: NFO/storage policy tests and docs.
  Handoff: Split subtitle import/export or NFO export into separate lanes if
  they require independent acceptance workflow or storage policy changes.

## M5 - Closeout Or Split

- [ ] APW-060 [owner=planner] [deps=APW-030] [scope=docs/workstreams/addon-protected-writes,docs/api,docs/adr]
  Goal: Close the lane after concrete protected writes are proven, or split
  remaining metadata/artwork/subtitle/NFO/Library File Write breadth into
  narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane only after the protected-write apply model
  and redaction guarantees are stable.
