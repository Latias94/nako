# NFO Storage Write Policy Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] NSW-010 [owner=codex] [deps=none] [scope=docs/workstreams/nfo-storage-write-policy,docs/GOALS.md]
  Goal: Open M48 with storage write scope, non-goals, diagnostic model, and
  validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/nfo-storage-write-policy/DESIGN.md`.
  Handoff: Continue with VFS atomic write support.

## M1 - VFS Atomic Local Write Boundary

- [x] NSW-020 [owner=codex] [deps=NSW-010] [scope=crates/nako-vfs/src/lib.rs,crates/nako-vfs/src/local.rs,crates/nako-vfs/src/cache.rs]
  Goal: Add a storage-owned write request/report API and implement local
  `AtomicReplace` with same-directory temp file and rename.
  Validation: `cargo check -p nako-vfs --tests`;
  `cargo nextest run -p nako-vfs local::tests::local_backend_atomic --no-fail-fast`.
  Evidence: local backend tests prove atomic write creates/updates the final
  file and leaves no temp file on success.
  Handoff: Wire NFO export to request atomic replace.

## M2 - NFO Export Write Policy And Diagnostics

- [x] NSW-030 [owner=codex] [deps=NSW-020] [scope=crates/nako-nfo/src/export.rs,crates/nako-nfo/src/summary.rs,crates/nako-nfo/src/lib.rs]
  Goal: Have NFO export request atomic replace for sidecar writes and classify
  parse, preservation, unsupported, and storage write failures internally.
  Validation: `cargo check -p nako-nfo --tests`;
  `cargo nextest run -p nako-nfo nfo_service_export --no-fail-fast`.
  Evidence: service tests prove successful export uses atomic write and failure
  summaries expose diagnostic categories without changing public API routes.
  Handoff: Run focused and workspace closeout gates.

## M3 - Validation And Closeout

- [x] NSW-040 [owner=codex] [deps=NSW-030] [scope=workspace,docs]
  Goal: Close M48 with focused and workspace validation, evidence updates, and
  follow-on notes.
  Validation: `cargo fmt --all -- --check`; `cargo check -p nako-vfs --tests`;
  `cargo nextest run -p nako-vfs --no-fail-fast`; `cargo check -p nako-nfo --tests`;
  `cargo nextest run -p nako-nfo --no-fail-fast`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Recommend follow-ons from backup policy, public diagnostics, or
  broader NFO compatibility only after M48 evidence is complete.
