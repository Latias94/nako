# Typed Storage Errors Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] TSE-010 [owner=codex] [deps=none] [scope=docs/workstreams/typed-storage-errors,docs/GOALS.md]
  Goal: Open M45 with typed storage error classification scope, non-goals, and
  validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/typed-storage-errors/DESIGN.md`
  Handoff: Continue with the core error type and HTTP mapping slice.

## M1 - Core Classification And HTTP Mapping

- [x] TSE-020 [owner=codex] [deps=TSE-010] [scope=crates/nako-core/src/error.rs,crates/nako-server/src/http/error.rs,crates/nako-server/src/http/tests/system.rs]
  Goal: Add typed storage error classification and remove HTTP message parsing
  for storage errors while preserving public error codes.
  Validation: `cargo check -p nako-server --tests`;
  `cargo nextest run -p nako-server http::tests::system::api_errors_map_playback_storage_categories --no-fail-fast`.
  passed.
  Evidence: HTTP mapping matches `StorageErrorKind`.
  Handoff: Continue with VFS/WebDAV/staging source classification.

## M2 - VFS And Runtime Source Classification

- [x] TSE-030 [owner=codex] [deps=TSE-020] [scope=crates/nako-vfs/src,crates/nako-server/src/app,crates/nako-db/src/staging.rs]
  Goal: Classify local/WebDAV/cache/staging/playback storage errors at their
  source.
  Validation: `cargo check -p nako-vfs --tests`;
  `cargo nextest run -p nako-vfs --no-fail-fast`;
  `cargo nextest run -p nako-server http::tests::system::api_errors_map_playback_storage_categories --no-fail-fast`.
  passed.
  Evidence: WebDAV timeout/auth/rate-limit/status paths and staging budget/
  validation paths use typed categories.
  Handoff: Public behavior remains unchanged.

## M3 - Validation And Closeout

- [x] TSE-040 [owner=codex] [deps=TSE-030] [scope=workspace,docs]
  Goal: Close M45 with focused and workspace gates.
  Validation: `cargo fmt --all -- --check`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Recommend the next goal among `nako-api` module split and NFO Round
  Trip preservation.
