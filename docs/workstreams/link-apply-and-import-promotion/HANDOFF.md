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
creation only behind `StorageBackend::apply`. VFS now also exposes a typed
storage cleanup primitive for post-apply compensation: the local backend can
remove file or symlink targets without deleting directories or exposing OS
paths, and unsupported backends report typed cleanup-pending evidence. The
server app service can now apply an accepted promotion by revalidating current
plan facts, invoking only `StorageBackend::apply` for storage mutation,
committing the Media Item / Media Source / library state / duplicate
relationship state after target creation, marking the artifact promoted,
replaying already-promoted apply records, recording failed-before-mutation audit
outcomes for stale or source-missing apply attempts, and recording
cleanup-complete or cleanup-pending terminal audits when catalog commit fails
after storage target creation.

## Active Task

- Task ID: LAIP-080
- Owner: planner
- Files: `docs/workstreams/link-apply-and-import-promotion`
- Validation: evidence gates are fresh; parent umbrella points to the next lane;
  NFO sidecar follow-on is explicitly represented.
- Status: READY
- Evidence: LAIP-010 through LAIP-070 are recorded in `EVIDENCE_AND_GATES.md`

## Decisions

- Promotion preview is explanatory only; it is not an authorization token.
- Apply requires an explicit operator-confirmed command and idempotency key.
- Apply must revalidate plan facts before storage mutation.
- Server code must not copy, link, move, delete, or inspect raw OS paths
  directly; storage mutation belongs behind VFS/storage APIs.
- Move/delete source behavior is deferred from the first apply slice.
- NFO sidecar import/export mutation is deferred until backup, authority,
  rollback, and audit requirements are explicitly decided.
- Partial failure after storage target creation must produce cleanup-complete or
  cleanup-pending audit state, not a false promoted state.
- LAIP-020 records apply/audit state only. It does not add copy/link/move/delete
  behavior and does not promote Media Sources.
- LAIP-030 records accepted apply attempts only. It does not add copy/link/move
  /delete behavior and does not promote Media Sources.
- LAIP-040 adds VFS storage apply primitives only. It does not promote Media
  Sources, write catalog rows, delete staged artifacts, or implement rollback.
- LAIP-050 composes the first successful apply path and pre-mutation failure
  audits.
- LAIP-060 adds VFS-mediated cleanup after storage target creation followed by
  catalog commit failure. It records cleanup-complete when target cleanup is
  confirmed, cleanup-pending when cleanup is unsupported or fails, and never
  marks failed applies or artifacts as promoted.
- LAIP-060 does not move/delete source artifacts and does not mutate NFO
  sidecars.
- LAIP-070 splits NFO sidecar import/export mutation to
  `docs/workstreams/nfo-sidecar-promotion-apply` because it is a separate
  accepted **Library File Write** and metadata-authority workflow. It needs
  NFO Round Trip preservation, backup, atomic replace, backup retention,
  field-lock/local-authority application, hierarchy confirmation, rollback or
  repair-pending semantics, idempotent replay, and redacted audit evidence.
- LAIP must not call `taru-nfo` to mutate sidecars or run an implicit NFO
  post-hook during Managed Import promotion.

## Blockers

- None for LAIP-080.

## Next Recommended Action

- Execute LAIP-080: verify docs/JSON hygiene, close or split the LAIP lane, and
  return to `post-rpd-product-hardening` for next-lane scoring. The NFO sidecar
  follow-on starts at `nfo-sidecar-promotion-apply` NSPA-020 when chosen.
