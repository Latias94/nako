# VFS Repair Diagnostics Architecture Reconciliation

## Goal

Reconcile the Storage/VFS and Control Plane architecture maps after the shipped
VFS cache repair job diagnostics projection so future agents do not reopen the
same "broader operator diagnostics" slice by mistake.

## What I Already Know

- `AdminJobListItem` and `JobResponse` now include optional redaction-safe
  diagnostics for `JobKind::VfsCacheRepair`.
- Successful VFS cache repair jobs project the typed safe summary when
  `summary_json` parses correctly.
- Failed VFS cache repair jobs expose only stable redacted failure facts and do
  not return raw durable job errors.
- `docs/architecture/STORAGE_VFS.md`,
  `docs/architecture/CONTROL_PLANE.md`, and
  `docs/architecture/WORKSTREAM_LINKS.md` still contain older follow-on wording
  that can make the shipped job diagnostics appear unstarted.

## Requirements

- Update architecture maps to show VFS repair job diagnostics as shipped.
- Keep remaining follow-ons explicit:
  - cache purge/delete/invalidation;
  - backend configuration mutation;
  - library file writes;
  - automated repair policy;
  - broader realtime diagnostics / incident bundles if needed.
- Replace stale proposed lane names with a narrower follow-on name when the old
  name now points at completed job diagnostics.
- Do not change Rust code, API DTOs, generated contracts, runtime behavior,
  task scheduling, storage mutation behavior, or Admin Web UI.

## Acceptance Criteria

- [x] `STORAGE_VFS.md` VFS cache status, shipped list, and next lanes are
      consistent with the completed diagnostics projection.
- [x] `CONTROL_PLANE.md` VFS cache repair section lists job diagnostics as
      shipped and does not keep generic repair-job operator diagnostics as the
      next slice.
- [x] `WORKSTREAM_LINKS.md` points at the archived diagnostics task and uses a
      follow-on slug that no longer implies the shipped job projection is
      missing.
- [x] Trellis task context is curated and validates.
- [x] `git diff --check` passes.

## Definition Of Done

- Documentation and task evidence are committed together.
- No source-code or generated-contract changes are included.
- The remaining follow-on language is specific enough that a future architecture
  review can choose an actual next reliability slice instead of rediscovering
  the completed diagnostics projection.

## Out Of Scope

- No new Admin endpoint or UI workflow.
- No new durable job state, retry behavior, scheduler behavior, or runtime
  supervisor behavior.
- No cache purge/delete/invalidation or backend configuration mutation.
- No broader incident bundle implementation.
