# Admin Settings Configuration Authority - Milestones

Status: Closed
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

Exit criteria:

- Backend authority problem is separated from Admin Web UI work.
- Current lack of global settings persistence is documented.
- First executable task is ASCA-020.

Primary evidence:

- `DESIGN.md`
- `TODO.md`

## M1 - First Field Group And Authority Model

Exit criteria:

- First field group is selected.
- Source tracking and TOML/admin/runtime precedence are explicit.
- Restart-required versus hot-applied behavior is explicit.
- Route shape is ready for implementation.

Primary gates:

- Settings/config source inventory.
- `git diff --check`

Status: Complete. Metadata raw cache retention was selected as the first field
group with persisted Admin desired-state, TOML-first startup merge,
Admin-overrides-on-startup precedence, and restart-required effect reporting.

## M2 - Backend Implementation

Exit criteria:

- Persistence/runtime behavior is implemented.
- Admin API route(s) are generated and documented.
- Tests prove restart behavior, validation, redaction, and conflicts.

Primary gates:

- Focused `cargo nextest` for server behavior.
- Focused `cargo nextest` for database contract when persistence is added.
- `cargo test -p nako-api admin_contract_includes_route_constants -- --nocapture`

Status: Complete with concerns. Backend persistence, startup merge, Admin API
GET/PUT route, generated Admin contract, HTTP docs, and focused tests are in
place. Concern: UI controls must remain limited to this field group and must
not imply hot-apply for other settings.

## M3 - Closeout

Exit criteria:

- Review has no blocking findings.
- Evidence is fresh.
- Admin Web settings mutation lane has a clear handoff.

Status: Complete with concerns. Review and verification found no blocking
issues. The lane is handed back to Admin Web settings mutation for metadata raw
cache UI controls only. PostgreSQL runtime parity remains a skipped local gate
until `NAKO_TEST_POSTGRES_URL` is available.
