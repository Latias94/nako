# Addon Library File Write Policy TODO

Status: Active
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] ALFW-010 [owner=planner] [deps=APW-060] [scope=docs/workstreams/addon-library-file-write-policy,docs/workstreams/addon-protected-writes,docs/workstreams/README.md]
  Goal: Open the focused subtitle/NFO/sidecar Library File Write follow-on
  lane split from APW.
  Validation: `git diff --check`.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, APW closeout docs.
  Handoff: Continue with ALFW-020 before accepting subtitle, NFO, or sidecar
  file-write payloads.

## M1 - File Write Seam Audit

- [x] ALFW-020 [owner=codex] [deps=ALFW-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-nfo,crates/taru-vfs,docs]
  Goal: Audit current subtitle, NFO, storage/VFS, backup, and Addon Side Effect
  seams; choose the first bounded Library File Write target.
  Validation: `rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs`; `git diff --check`.
  Review: decide whether first apply should be subtitle import, NFO export, or
  narrower sidecar asset write. The audit must explicitly choose how any
  NFO-derived canonical metadata uses `commit_nfo_import`, and how any
  discoverable source/state/search changes use `commit_library_scan_source` or
  a new first-party commit unit.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`; `DESIGN.md` selected
  MediaSource-targeted Taru-owned NFO Export as the first apply target.
  Handoff: Continue with ALFW-030 by implementing a typed `library_file_write`
  side-effect payload for NFO export. Target derivation, atomic replace,
  backup/retention, truthful queued-or-applied semantics, and redacted report
  semantics are explicit.

## M2 - First File Write Apply Slice

- [x] ALFW-030 [owner=codex] [deps=ALFW-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-nfo,crates/taru-vfs,docs/api]
  Goal: Implement the selected addon Library File Write apply path: an accepted
  `library_file_write` side effect with a MediaSource target requests
  Taru-owned NFO Export without addon-provided paths, Source Locators, remote
  handles, or raw NFO payloads.
  Validation: focused NFO/storage/addon tests, including create-missing,
  replace-existing-preserving, idempotent replay, unsupported permission/target,
  and redacted response/report cases; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no response exposes raw payload, Source Locators, filesystem
  paths, remote storage handles, or unredacted backup/write reports. Verify the
  implementation does not re-create Addon-specific NFO import, scan-source, or
  search-projection write ordering.
  Evidence: code/tests/API docs and ALFW notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with ALFW-040. The first NFO export slice is implemented
  and verified; close the lane or split subtitle import/export, broader NFO
  write behavior, and arbitrary sidecar asset writes into narrower follow-ons.

## M3 - Closeout Or Split

- [ ] ALFW-040 [owner=planner] [deps=ALFW-030] [scope=docs/workstreams/addon-library-file-write-policy,docs/api]
  Goal: Close the file-write lane or split remaining subtitle/NFO/sidecar
  breadth into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane only after Library File Write authority and
  redaction guarantees are stable.
