# Managed Artwork Fetch Artifact Storage TODO

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] MAFA-010 [owner=planner] [deps=MAIS-040] [scope=docs/workstreams/managed-artwork-fetch-artifact-storage,docs/workstreams/managed-artwork-ingest-selection,docs/workstreams/README.md]
  Goal: Open a focused follow-on for turning queued managed artwork ingest jobs
  into internal managed artifact bytes without public artwork publication.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, MAIS closeout docs.
  Handoff: Continue with MAFA-020 before implementing a worker or choosing
  artifact byte storage mechanics.

## M1 - Fetch And Storage Seam Audit

- [x] MAFA-020 [owner=codex] [deps=MAFA-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-vfs,docs]
  Goal: Audit job runtime claim/update seams, HTTP fetch policy, image
  validation options, storage/VFS/cache/staging boundaries, and artifact commit
  requirements; choose the first internal artifact byte storage policy.
  Validation: `rg -n "ManagedArtworkIngest|managed_artwork_ingest|managed_artwork_artifacts|JobKind::ManagedArtworkIngest|artwork.ingest|storage_uri|ImageAsset|cache_uri|source_uri|thumbnail" crates docs`; `git diff --check`.
  Review: decide whether the first artifact byte store is local app data, VFS
  mediated storage, or a new internal artifact storage port. If artifact bytes
  and rows need multi-step persistence, introduce or reuse a first-party commit
  boundary instead of placing ordering in the worker body.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`; selected storage policy in
  `DESIGN.md`.
  Result: DONE. First target is a dedicated managed artwork worker/runtime
  boundary with managed-artwork-specific claim/commit repository methods,
  bounded HTTP(S) byte fetch, image validation, and a server-local internal
  artifact storage port that persists opaque `managed-artwork://...`
  references.
  Handoff: Continue with MAFA-030 by adding the claim/commit model, artifact
  storage port/config, fetch/validation seam, and focused success/failure tests
  without public `ImageAsset`, thumbnail, or selection behavior.

## M2 - First Fetch/Artifact Slice

- [x] MAFA-030 [owner=codex] [deps=MAFA-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-vfs,docs/api]
  Goal: Process one queued managed artwork ingest into an internal artifact or
  safe failure state under bounded fetch and validation policy.
  Validation: focused managed artwork worker tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify worker input, job summary, admin responses, and public
  responses do not expose raw candidate source URLs, Source Locators,
  filesystem paths, storage handles, cache URIs, or raw validation details.
  Evidence: code/tests/API docs and MAFA notes in `EVIDENCE_AND_GATES.md`.
  Result: DONE. Added managed-artwork-specific claim, commit, failure, and
  artifact lookup repository methods; a server-local artwork artifact config
  and storage port; bounded HTTP(S) byte fetch; static image validation for
  JPEG, PNG, and WebP; safe Admin `process-next` response DTOs; and focused
  server/db/API redaction tests. Successful processing writes internal bytes
  and `managed_artwork_artifacts` metadata without creating public
  `ImageAsset`, thumbnail, or selected artwork state.
  Handoff: Continue with MAFA-040 to harden failure/retry semantics or split if
  worker/runtime scope grows.

## M3 - Failure, Retry, And Redaction Hardening

- [x] MAFA-040 [owner=codex] [deps=MAFA-030] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,docs/api]
  Goal: Harden retry/cancellation, safe failure codes, job summaries, and admin
  diagnostics for managed artwork ingest failures.
  Validation: focused failure/redaction tests; `cargo nextest run -p taru-server artwork --no-fail-fast`; `cargo nextest run -p taru-db artwork --no-fail-fast`; `git diff --check`.
  Review: failure reports must be actionable for admins without leaking raw
  URLs, local paths, storage URIs, provider tokens, or decoder internals.
  Evidence: tests and `EVIDENCE_AND_GATES.md`.
  Result: DONE. Internal failure codes are now a bounded enum mapped to safe
  strings, failed ingests write safe job summaries with `failure_code` and
  `status`, and Admin process-next failure tests cover unsupported media type
  and invalid image bodies without leaking source URLs, response bodies, token
  material, cache URIs, storage URIs, or public `ImageAsset` state. Retry is
  limited to the configured in-process fetch attempts; failed ingest requeue
  and durable cancellation APIs remain deferred for MAFA-050 closeout/split.
  Handoff: Continue with MAFA-050 closeout or split public image-serving if
  internal artifacts are stable.

## M4 - Closeout Or Split

- [x] MAFA-050 [owner=planner] [deps=MAFA-040] [scope=docs/workstreams/managed-artwork-fetch-artifact-storage,docs/api]
  Goal: Close the fetch/artifact lane or split public serving, thumbnails, and
  selected artwork publication into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE. Workstream closed after fresh closeout gates. Internal managed
  artifact authority, byte storage, success redaction, and failure redaction are
  stable enough for follow-ons. Public image serving, thumbnails, selected
  artwork publication, durable retry/requeue, cancellation, and orphan artifact
  cleanup are deferred/split candidates.
  Handoff: Recommend the next lane only after internal artifact authority,
  byte storage, and redaction guarantees are stable.
