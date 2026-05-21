# Link Apply And Import Promotion — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open as the follow-on split from `managed-import-staging` and
`nfo-link-authority`. Managed Import can now produce a non-mutating promotion
preview, and VFS can perform non-mutating link dry-run diagnostics. Durable
promotion acceptance/audit records now exist before any storage mutation is
added. The server app service can now explicitly accept a current promotion
plan, record the durable apply attempt, replay matching idempotency keys, reject
mismatched keys, and reject blocked plans without storage or Media Source
mutation. VFS now exposes typed copy/hardlink/symlink apply primitives with
redacted reports, default unsupported backend behavior, cached-backend
forwarding, local planning-safety reuse, no overwrite, and local target
creation only behind `StorageBackend::apply`.

## Active Task

- Task ID: LAIP-050
- Owner: codex
- Files: `crates/taru-server`
- Validation: server tests prove successful apply, blocked/stale apply,
  duplicate evidence behavior, no direct OS path mutation, and catalog writes
  after target creation only; keep `cargo nextest run -p taru-vfs link
  --no-fail-fast` available as the storage boundary regression gate.
- Status: READY
- Evidence: LAIP-010 through LAIP-040 are recorded in `EVIDENCE_AND_GATES.md`

## Decisions

- Promotion preview is explanatory only; it is not an authorization token.
- Apply requires an explicit operator-confirmed command and idempotency key.
- Apply must revalidate plan facts before storage mutation.
- Server code must not copy, link, move, delete, or inspect raw OS paths
  directly; storage mutation belongs behind VFS/storage APIs.
- Move/delete source behavior is deferred from the first apply slice.
- NFO sidecar import/export mutation is deferred until backup, authority,
  rollback, and audit requirements are explicitly decided.
- Partial failure must produce rollback-complete or cleanup-pending audit state,
  not a false promoted state.
- LAIP-020 records apply/audit state only. It does not add copy/link/move/delete
  behavior and does not promote Media Sources.
- LAIP-030 records accepted apply attempts only. It does not add copy/link/move
  /delete behavior and does not promote Media Sources.
- LAIP-040 adds VFS storage apply primitives only. It does not promote Media
  Sources, write catalog rows, delete staged artifacts, or implement rollback.

## Blockers

- None for LAIP-050.

## Next Recommended Action

- Execute LAIP-050: compose server promotion apply orchestration. Revalidate
  accepted plan facts, call `StorageBackend::apply` for the selected
  copy/hardlink/symlink operation, commit Media Source / duplicate relationship
  state only after the target exists, and record terminal audit outcomes.
