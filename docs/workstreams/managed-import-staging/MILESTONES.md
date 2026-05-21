# Managed Import Staging — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Lane Open

Exit criteria:

- Workstream docs exist and agree.
- Scope excludes generic downloaders and direct library writes.
- First executable slice is durable domain/schema work.

Primary evidence:

- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/managed-import-staging/TODO.md`

## M1 — Durable Import Artifact Domain

Exit criteria:

- Core domain records and state enums exist.
- Repository trait is explicit.
- SQLite/PostgreSQL migrations preserve backend parity.
- Contract tests round-trip Managed Import artifacts.

## M2 — App Service Diagnostics

Exit criteria:

- Server service can create/list staged import artifact diagnostics.
- Diagnostics are redacted and library-scoped.
- No external fetch or library write is performed.

## M3 — Promotion Plan Preview

Exit criteria:

- Promotion planning is non-mutating.
- Plan includes destination, duplicate/link, metadata/NFO, and blocked reasons.
- Tests prove library roots are unchanged by planning.

## M4 — Apply/Follow-On Decision

Exit criteria:

- Apply is either implemented with rollback/audit proof or explicitly split.
- Hardlink/symlink mutation is not smuggled into planning.

## M5 — Closeout

Exit criteria:

- Fresh validation is recorded.
- Parent umbrella and workstream index agree on status.
- Follow-ons are explicit.