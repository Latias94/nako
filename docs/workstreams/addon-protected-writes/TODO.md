# Addon Protected Writes TODO

Status: Completed
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

- [x] APW-020 [owner=codex] [deps=APW-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-metadata,crates/taru-catalog,crates/taru-nfo,crates/taru-vfs,docs]
  Goal: Audit current Addon Side Effect intake, Canonical Metadata merge,
  catalog commit, Managed Artwork, subtitle, NFO, and storage/VFS write seams;
  choose the first concrete protected-write apply target.
  Validation: `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`; `git diff --check`.
  Review: no ADR amendment is required for APW-030 if it preserves ADR 0020,
  adds explicit apply outcome state, keeps Addon metadata attribution
  first-class, and routes writes through Taru-owned metadata/catalog seams.
  Split an ADR only for direct storage authority, Public Client write APIs,
  Admin API reuse, or OAuth-first authorization.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with APW-030. Canonical Metadata remains the first concrete
  apply target, but APW-030 must first add explicit side-effect apply outcome
  state and Addon metadata source attribution; do not treat
  `validation_status = accepted` as "domain write applied".

## M2 - Canonical Metadata Apply Slice

- [x] APW-030 [owner=codex] [deps=APW-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-metadata,crates/taru-catalog,docs/api]
  Goal: Implement the smallest concrete `metadata_write` Addon Side Effect
  apply path that turns an accepted intake record into a Taru-owned Canonical
  Metadata update while preserving merge policy, idempotency, audit, redaction,
  and catalog/search consistency.
  Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests`; focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`; relevant metadata/catalog tests; `cargo fmt --all -- --check`; `git diff --check`.
  Review: review-workstream must check that HTTP handlers do not own metadata
  merge logic and that responses do not leak raw payloads, provenance, Source
  Locators, filesystem paths, or provider bodies.
  Evidence: `crates/taru-core/src/addon.rs`,
  `crates/taru-core/src/media/metadata.rs`,
  `crates/taru-db/migrations/0023_addon_side_effect_apply_outcome.sql`,
  `crates/taru-db/src/addons.rs`, `crates/taru-db/src/codec.rs`,
  `crates/taru-server/src/app/addons.rs`,
  `crates/taru-server/src/http/tests/addons.rs`, `docs/api/HTTP_API.md`, and
  APW-030 notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Minimal field breadth stayed bounded to title-like, overview,
  runtime, genre, and tag fields. Wider Canonical Metadata fields, field-level
  provenance tables, and addon-specific domain events should be split rather
  than expanding this slice.

## M3 - Managed Artwork And Artifact Intake

- [x] APW-040 [owner=planner] [deps=APW-030] [scope=docs/workstreams/addon-protected-writes,docs/workstreams/addon-managed-artwork-artifacts,docs/workstreams/README.md]
  Goal: Split `artwork_write`, Artwork Candidate, Managed Artwork, and
  Taru-Managed Artifact storage into a dedicated follow-on lane without
  implementing artwork runtime behavior in APW.
  Validation: focused artwork/addon tests selected after APW-020; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `git diff --check`.
  Review: verify resource budgets, external fetch ownership, artifact
  provenance, and redacted response shape.
  Evidence: `docs/workstreams/addon-managed-artwork-artifacts/`.
  Handoff: Continue with AMAA-010 before accepting `artwork_write` payloads or
  Managed Artwork artifacts.

## M4 - Subtitle, NFO, And Library File Write Policy

- [x] APW-050 [owner=planner] [deps=APW-020] [scope=docs/workstreams/addon-protected-writes,docs/workstreams/addon-library-file-write-policy,docs/workstreams/README.md]
  Goal: Split addon-initiated subtitle, NFO, and sidecar-asset Library File
  Write behavior into a dedicated follow-on lane instead of broadening APW.
  Validation: focused NFO/storage/addon tests selected after APW-020; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no Addon response or audit summary exposes raw Source
  Locators, filesystem paths, remote storage handles, or unredacted file-write
  payloads.
  Evidence: `docs/workstreams/addon-library-file-write-policy/`.
  Handoff: Continue with ALFW-010 before accepting subtitle, NFO, or sidecar
  file-write payloads.

## M5 - Closeout Or Split

- [x] APW-060 [owner=planner] [deps=APW-030] [scope=docs/workstreams/addon-protected-writes,docs/api,docs/adr]
  Goal: Close the lane after concrete protected writes are proven, or split
  remaining metadata/artwork/subtitle/NFO/Library File Write breadth into
  narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: closeout review found one provenance issue in APW-030 and fixed it:
  scalar Addon metadata patches now refresh search without rewriting catalog
  label sources, while genre/tag patches only replace touched label sets with
  Addon source attribution.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Lane is closed. Continue with `addon-managed-artwork-artifacts` or
  `addon-library-file-write-policy` depending on the next product priority.
