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
mutation.

## Active Task

- Task ID: LAIP-040
- Owner: codex
- Files: `crates/taru-vfs`
- Validation: `cargo nextest run -p taru-vfs link --no-fail-fast`; focused copy
  apply tests; `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: LAIP-010 through LAIP-030 are recorded in `EVIDENCE_AND_GATES.md`

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

## Blockers

- None for LAIP-040.

## Next Recommended Action

- Execute LAIP-040: add VFS-mediated copy/hardlink/symlink apply primitives
  that reuse planning safety and return typed outcomes without exposing OS path
  mutation to server code.
