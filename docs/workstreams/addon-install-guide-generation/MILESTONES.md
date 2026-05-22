# Addon Install Guide Generation Milestones

Status: Completed
Last updated: 2026-05-22

## M0 — Boundary Freeze

Exit criteria:

- Addon Install Guide and Addon Manager are separated in workstream docs.
- Route, DTO, UI, docs, and validation scope are explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/addon-install-guide-generation/DESIGN.md`
- `docs/workstreams/addon-install-guide-generation/TODO.md`

Status: completed.

## M1 — Server-Owned Guide

Exit criteria:

- Admin API route returns guide sections from a registered Addon manifest.
- Generated Admin API TypeScript contract includes the route and DTOs.
- Focused Rust tests prove redaction and non-manager semantics.

Primary gates:

- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `cargo nextest run -p taru-server install_guide --no-fail-fast`

Status: completed.

## M2 — Admin Web Preview

Exit criteria:

- Admin Web loads guide data through `AdminApiClient` and `dataSource.ts`.
- Addon Operations panel renders Docker Compose/systemd/checklist/verification
  previews.
- UI tests prove safe copy and absence of sensitive strings.

Primary gates:

- `npm test`
- `npm run build`

Status: completed.

## M3 — Docs And Closeout

Exit criteria:

- HTTP API and Addon author docs describe the guide boundary.
- Final gates are recorded.
- Remaining work is completed, deferred, or split into a follow-on.

Primary gates:

- `cargo fmt --all -- --check`
- focused Rust gates
- Admin Web gates
- `git diff --check`

Status: completed.
