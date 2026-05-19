# Addon Library File Write Policy Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is closed. The first Library File Write runtime behavior is now
implemented: accepted MediaSource-targeted `library_file_write` side effects
can request Taru-owned NFO Export without addon-supplied paths, Source
Locators, remote handles, backup URIs, or raw NFO payloads.

The implementation uses a typed payload with `file_role: "nfo"` and policy
`create_missing` or `replace_existing_preserving`. It applies synchronously via
the first-party NFO/VFS path, marks the side effect `applied` only after the
write completes, stores a redacted aggregate `apply_report`, and preserves
idempotent replay behavior.

## Closeout Task

- Task ID: ALFW-040
- Owner: planner
- Files: `docs/workstreams/addon-library-file-write-policy`, `docs/api`, and
  any follow-on workstream docs created during split
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: DONE
- Review: ALFW-030 has no blocking findings. Remaining subtitle/NFO/sidecar
  breadth is deferred to future narrower follow-ons instead of widening this
  completed first slice.
- Evidence: `EVIDENCE_AND_GATES.md` contains the implementation and validation
  evidence for ALFW-030 and closeout evidence for ALFW-040

## Blockers

- None known.

## Decisions Since Last Update

- Close ALFW after proving one Taru-owned Library File Write path instead of
  keeping all subtitle, NFO, sidecar, and queued execution breadth in one lane.
- Keep the shipped path synchronous. `apply_status = applied` means the
  first-party NFO/VFS write completed, not merely that a job was queued.
- Keep `apply_report` redacted to aggregate counters only. Do not expose raw
  `StorageWriteReport`, `StorageUri`, Source Locators, filesystem paths,
  backup URIs, remote handles, or raw payload content.
- Treat MediaItem-targeted NFO export as future work until multi-source and
  Source Variant behavior is explicit.

## Residual Risks

- The current runtime request performs synchronous storage work. If remote
  write latency or larger payloads become product requirements, introduce a
  queued Addon Task or durable job association before reporting deferred work as
  complete.
- Subtitle write behavior still needs a first-party subtitle/track model,
  language/format validation, conflict policy, and safe report shape.
- Arbitrary sidecar asset writes still need a content-type matrix, target
  derivation rules, backup policy, and redaction tests.
- Future NFO-derived metadata apply remains bound to `commit_nfo_import` /
  `NfoImportPersistenceCommit`; do not shortcut it through the Addon handler.

## Next Recommended Action

- Choose the next workstream by product value:
  - `addon-managed-artwork-artifacts` if the next user-visible plugin value is
    poster/backdrop/artwork import.
  - a new subtitle-focused follow-on if the next value is downloaded or
    addon-supplied subtitles.
  - a new sidecar-asset follow-on only after content-type and target-derivation
    rules are clear.
- CAD-070 alignment remains binding: future NFO-derived metadata apply must
  reuse `commit_nfo_import`; file-write paths that change discoverable source
  state must reuse `commit_library_scan_source`, `LibraryIndexRepository`, or a
  new first-party commit unit. Do not put those ordered durable writes inside an
  Addon handler.
