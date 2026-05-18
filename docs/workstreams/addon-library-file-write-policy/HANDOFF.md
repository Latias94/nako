# Addon Library File Write Policy Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

This lane is newly split from APW-060. No subtitle, NFO, or sidecar file-write
runtime behavior has been implemented here yet. APW proved Addon Side Effect
intake and apply outcome semantics with `metadata_write`; this lane must audit
Library File Write seams before accepting file-write payloads.

## Active Task

- Task ID: ALFW-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-nfo`, `crates/taru-vfs`, `docs`
- Validation: audit `rg`, then `git diff --check`
- Status: READY
- Review: decide whether first apply target is subtitle import, NFO export, or
  narrower sidecar asset write
- Evidence: write audit notes to `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run ALFW-020. Do not implement file writes until target derivation, NFO Round
  Trip behavior, backup policy, VFS write reports, idempotency, and redacted
  diagnostics are explicit.
- CAD-070 alignment: any NFO-derived metadata apply must reuse
  `commit_nfo_import`; any file-write path that changes discoverable source
  state must reuse `commit_library_scan_source`, `LibraryIndexRepository`, or a
  new first-party commit unit. Do not put those ordered durable writes inside an
  Addon handler.
