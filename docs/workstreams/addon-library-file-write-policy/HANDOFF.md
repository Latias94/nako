# Addon Library File Write Policy Handoff

Status: Active
Last updated: 2026-05-18

## Current State

ALFW-030 is complete. The first Library File Write runtime behavior is now
implemented: accepted MediaSource-targeted `library_file_write` side effects
can request Taru-owned NFO Export without addon-supplied paths, Source
Locators, remote handles, backup URIs, or raw NFO payloads.

The implementation uses a typed payload with `file_role: "nfo"` and policy
`create_missing` or `replace_existing_preserving`. It applies synchronously via
the first-party NFO/VFS path, marks the side effect `applied` only after the
write completes, stores a redacted aggregate `apply_report`, and preserves
idempotent replay behavior.

## Active Task

- Task ID: ALFW-040
- Owner: planner
- Files: `docs/workstreams/addon-library-file-write-policy`, `docs/api`, and
  any follow-on workstream docs created during split
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: READY
- Review: confirm ALFW-030 has no blocking findings, then close this lane or
  split remaining subtitle/NFO/sidecar write breadth into dedicated follow-ons
- Evidence: `EVIDENCE_AND_GATES.md` contains the implementation and validation
  evidence for ALFW-030

## Blockers

- None known.

## Next Recommended Action

- Run ALFW-040. Review the ALFW-030 diff and gate evidence, then decide whether
  to close this lane or split follow-ons for subtitle import/export, broader NFO
  export/import behavior, arbitrary sidecar asset writes, and any queued
  library-file-write execution semantics.
- CAD-070 alignment remains binding: future NFO-derived metadata apply must
  reuse `commit_nfo_import`; file-write paths that change discoverable source
  state must reuse `commit_library_scan_source`, `LibraryIndexRepository`, or a
  new first-party commit unit. Do not put those ordered durable writes inside an
  Addon handler.
