# Addon Library File Write Policy TODO

Status: Proposed
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

- [ ] ALFW-020 [owner=codex] [deps=ALFW-010] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-nfo,crates/taru-vfs,docs]
  Goal: Audit current subtitle, NFO, storage/VFS, backup, and Addon Side Effect
  seams; choose the first bounded Library File Write target.
  Validation: `rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs`; `git diff --check`.
  Review: decide whether first apply should be subtitle import, NFO export, or
  narrower sidecar asset write.
  Evidence: audit notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with ALFW-030 only after target derivation, write mode,
  backup, and redacted report semantics are explicit.

## M2 - First File Write Apply Slice

- [ ] ALFW-030 [owner=codex] [deps=ALFW-020] [scope=crates/taru-core,crates/taru-db,crates/taru-server,crates/taru-api,crates/taru-nfo,crates/taru-vfs,docs/api]
  Goal: Implement the smallest safe addon Library File Write apply path
  selected by ALFW-020.
  Validation: focused NFO/storage/addon tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: verify no response exposes raw payload, Source Locators, filesystem
  paths, remote storage handles, or unredacted backup/write reports.
  Evidence: code/tests/API docs and ALFW notes in `EVIDENCE_AND_GATES.md`.
  Handoff: Split subtitle import/export, NFO export, or arbitrary sidecar asset
  writes if any dominates the first slice.

## M3 - Closeout Or Split

- [ ] ALFW-040 [owner=planner] [deps=ALFW-030] [scope=docs/workstreams/addon-library-file-write-policy,docs/api]
  Goal: Close the file-write lane or split remaining subtitle/NFO/sidecar
  breadth into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Recommend the next lane only after Library File Write authority and
  redaction guarantees are stable.
