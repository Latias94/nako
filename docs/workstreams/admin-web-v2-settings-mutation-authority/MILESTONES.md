# Admin Web V2 Settings Mutation Authority - Milestones

Status: Closed
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Current read-only `/settings` baseline is documented.
- Non-goals reject raw config, secrets, URLs, paths, and fake UI saves.
- First executable task is ASM-020.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `ROUTE_API_READINESS.md`

## M1 - Route/API Readiness And First Slice Decision

Exit criteria:

- Current Admin API, Admin Web, and config authority seams are audited.
- Candidate settings groups are compared.
- First mutation slice is selected, or backend configuration authority is split.
- Decision covers runtime-only, persisted, restart-required, and rejected
  semantics.

Primary gates:

- `rg` inventory over settings/config/Admin API surfaces.
- `git diff --check`

## M2 - First Real Mutation Slice Or Backend Split

Exit criteria:

- A real Admin API route/review-plan for the accepted slice exists, or a
  backend follow-on is opened and UI mutation remains blocked.
- DTOs stay under the Admin API boundary.
- Rust tests prove validation, redaction, and idempotency/conflict semantics.

Primary gates:

- `cargo nextest run -p nako-server <settings-mutation-test-filter> --no-fail-fast`
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`
- `cargo fmt --all --check`

Closeout result:

- Completed with ASCA backend predecessor and Admin Web contract consumption.
- Accepted slice is metadata raw cache settings only.
- Broader global settings remain separate authority follow-ons.

## M3 - Admin Web Mutation UI

Exit criteria:

- `/settings` exposes controls only for the implemented mutation path.
- Mock fallback cannot report a fake save.
- UI tests cover confirm, success, failure, fallback-disabled, and unsafe text
  exclusion states.

Primary gates:

- `cd apps/admin-web && npm run check && npm run test -- App.test.tsx adminApi/client.test.ts adminApi/dataSource.test.ts`

Closeout result:

- Completed with live-backed prepare/confirm controls, mock fallback disabled
  state, mutation success and error rendering, and focused Admin Web tests.

## M4 - Verification And Closeout

Exit criteria:

- Focused and broad gates are recorded.
- Browser smoke covers desktop and mobile.
- Workstream review has no blocking findings.
- Follow-ons are either completed, deferred, or split.
- `WORKSTREAM.json` status is updated.

Closeout result:

- Completed with fresh focused/broad gates, desktop and mobile smoke, and
  residual PostgreSQL contract risk recorded.
