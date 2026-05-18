# Addon Library File Write Policy Handoff

Status: Active
Last updated: 2026-05-18

## Current State

ALFW-020 is complete. No subtitle, NFO, or sidecar file-write runtime behavior
has been implemented here yet, but the first apply target is selected:
MediaSource-targeted addon-initiated Taru-owned NFO Export.

The audit found that NFO export already has the strongest first-party seams:
NFO Round Trip rendering, VFS `StorageWriteRequest::atomic_replace`,
existing-file backup, keep-latest retention diagnostics, and an NFO durable job
service. Addon Side Effect intake already stores accepted/rejected requests and
apply outcomes, but runtime apply currently only supports `metadata_write`;
`library_file_write` is still unsupported until ALFW-030.

## Active Task

- Task ID: ALFW-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-nfo`, `crates/taru-vfs`, `docs`
- Validation: focused NFO/storage/addon tests, package checks, formatting, and
  `git diff --check`
- Status: READY
- Review: implement the selected NFO export apply path without leaking paths,
  Source Locators, remote handles, backup URIs, write reports, or raw payloads
- Evidence: update `EVIDENCE_AND_GATES.md` with code/test/API evidence

## Blockers

- None known.

## Next Recommended Action

- Run ALFW-030. Add a typed `library_file_write` payload for NFO export with a
  MediaSource target. The addon supplies intent and policy, not path/content.
- Reuse first-party NFO/VFS boundaries: derive the sidecar inside Taru, render
  through NFO Round Trip when replacing, write with atomic replace, request
  existing-file backup and retention when replacing, and expose only redacted
  outcome facts.
- If implementation queues an NFO job, add truthful queued/job association
  semantics before exposing completion. Do not mark a side effect `applied`
  merely because a job was enqueued.
- CAD-070 alignment remains binding: future NFO-derived metadata apply must
  reuse `commit_nfo_import`; file-write paths that change discoverable source
  state must reuse `commit_library_scan_source`, `LibraryIndexRepository`, or a
  new first-party commit unit. Do not put those ordered durable writes inside an
  Addon handler.
