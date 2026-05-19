# Admin API TypeScript Contract Milestones

Status: Active
Last updated: 2026-05-19

## M-AATC.0 Contract Boundary Freeze

Exit criteria:

- Admin-web baseline is closed.
- Admin API TypeScript contract lane is active.
- Public Client SDK separation, DTO ownership, artifact location, and
  non-goals are documented.

Primary evidence:

- `docs/workstreams/admin-api-typescript-contract/README.md`
- `docs/workstreams/admin-api-typescript-contract/DESIGN.md`
- `docs/workstreams/admin-web-console/`

## M-AATC.1 Generator Shape Decision

Exit criteria:

- Existing hand-written admin-web wire DTOs are inventoried.
- First generated artifact shape is accepted.
- File-level implementation scope is known.

Primary evidence:

- Updated `DESIGN.md`
- Updated `HANDOFF.md`
- `ADMIN_CONTRACT_INVENTORY.md`

Status: completed for AATC-020. The accepted artifact shape is route constants
plus wire/query interfaces, generated app-locally under `apps/admin-web`,
without a generated fetch client.

## M-AATC.2 Generator Proof And Sync

Exit criteria:

- A `taru-api` command or test path emits the Admin API TypeScript contract.
- Generated output is committed app-locally or otherwise sync-checked.
- Tests reject leakage into Public Client SDK generation.

Primary gates:

```bash
cargo check -p taru-api --examples
cargo nextest run -p taru-api admin --no-fail-fast
cargo nextest run -p taru-api typescript --no-fail-fast
```

Status: completed for AATC-030. `taru-api` now emits an app-local Admin API
TypeScript contract, sync-checks the generated artifact, and keeps the Public
Client SDK admin-route leakage guard in the focused contract test set.

## M-AATC.3 Admin-Web Consumption

Exit criteria:

- Covered `/admin/v1/*` wire DTOs are imported from the generated contract.
- UI-only types remain local to admin-web.
- Existing live/mock fallback and redaction tests still pass.

Primary gates:

```bash
cd apps/admin-web
npm run check
npm run test
npm run build
```

## M-AATC.4 Closeout

Exit criteria:

- Evidence gates are fresh and recorded.
- Docs explain generation and sync commands.
- Follow-ons for admin SDK packaging or UI detail pages are split.
- `WORKSTREAM.json` is marked completed or deliberately left active with a
  concrete next task.
