# NFO Sidecar Promotion Apply — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

This lane is opened as the follow-on split from
`link-apply-and-import-promotion` LAIP-070. The split is intentional: Managed
Import promotion applies staged artifacts into a Media Library, while NFO
sidecar import/export apply is a separate **Library File Write** and
metadata-authority workflow.

Existing prerequisites are in place:

- `taru-nfo` round-trip preservation exists.
- VFS-backed NFO storage write policy exists.
- sidecar backup policy and backup retention diagnostics exist.
- `nfo-link-authority` can produce non-mutating NFO authority previews.
- LAIP promotion apply proves accepted, idempotent, cleanup-aware storage and
  catalog mutation for Media Sources, but it does not mutate sidecars.

## Active Task

- Task ID: NSPA-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`
- Validation: focused DB contract tests for durable sidecar apply records,
  idempotency-key lookup, state transitions, and redacted audit snapshots.
- Status: READY after LAIP-080 closeout or umbrella re-scoring
- Evidence: NSPA-010 planning docs

## Decisions

- NFO preview is explanatory only; it is not an authorization token.
- Sidecar apply requires explicit operator or policy acceptance and an
  idempotency key.
- Export sidecar apply must use `taru-nfo` round-trip preservation and VFS
  storage write APIs. Server code must not write raw OS paths directly.
- Import sidecar apply must update canonical metadata, field locks/local
  authority, and hierarchy confirmation only through durable acceptance.
- Backup-required, force-overwrite, retention, field-lock, and hierarchy
  warnings must be part of accepted preview facts.
- Partial failure must produce failed-before-mutation, rollback-complete, or
  repair-pending audit state, not false committed state.
- Operator-facing diagnostics must not leak raw local paths, raw XML, raw
  provider payloads, or secrets.
- Addon side effects and AI-generated artifacts may consume this lane later,
  but they must not bypass it.

## Blockers

- None for planning.
- NSPA implementation should start after LAIP-080 closeout or explicit umbrella
  re-scoring to avoid two active mutation lanes competing for the same mental
  model.

## Next Recommended Action

- Finish LAIP-080 closeout first.
- If the umbrella selects NFO sidecar apply next, execute NSPA-020 with TDD:
  add failing persistence contract tests, then implement durable sidecar apply
  domain and repository storage.
