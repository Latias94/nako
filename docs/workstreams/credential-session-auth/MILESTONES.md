# Credential Session Auth - Milestones

Status: Complete
Last updated: 2026-05-26

## M0 - Scope And Evidence Freeze

Status: Complete.

Exit criteria:

- The credential/session problem and target state are explicit.
- Public registration, frontend UI, cookies, invitations, and SSO are explicit
  non-goals.
- ADR 0037 records the chosen backend auth direction.
- CSA-010 is ready as the first executable storage-contract slice.

Primary evidence:

- `docs/workstreams/credential-session-auth/DESIGN.md`
- `docs/workstreams/credential-session-auth/TODO.md`
- `docs/adr/0037-local-credential-and-session-auth.md`

## M1 - Credential And Session Storage Contract

Status: Complete. CSA-010 added core credential/session records, repository
methods, SQLite/PostgreSQL baseline schema support, and focused DB contract
coverage.

Exit criteria:

- Core records model local password credentials and sessions without raw secret
  material.
- SQLite and PostgreSQL baselines contain credential/session tables and indexes.
- Repository contracts can upsert credentials, create/list/revoke sessions, and
  look up sessions by token hash.
- Contract tests cover both backends where local infrastructure is available.

Primary gates:

- `cargo nextest run -p nako-core identity --no-fail-fast`
- `cargo nextest run -p nako-db credential_session --no-fail-fast`

## M2 - Admin Credential Provisioning

Status: Complete. CSA-020 added Admin API password set/rotate/delete behavior
for existing users and exposes only a `local_password_configured` flag in
account records.

Exit criteria:

- Admin API can set, rotate, and delete local password credentials for existing
  users.
- The bootstrap administrator cannot be locked out accidentally by ambiguous
  password operations.
- API responses never expose password hashes or credential secret material.

Primary gates:

- `cargo nextest run -p nako-server admin_local_password --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`

## M3 - Public Login And Session Principal

Status: Complete. CSA-030 added Public Client login/current-account/logout,
Bearer session resolution, disabled-user rejection, bootstrap admin token
compatibility, and refreshed generated TypeScript/Kotlin SDK contracts.

Exit criteria:

- Public Client API supports login, current-account, and logout.
- Session bearer tokens resolve to active user principals and roles.
- Disabled users cannot create or use sessions.
- Existing bootstrap admin bearer token behavior remains intact.
- OpenAPI, Rust client, TypeScript SDK, and Kotlin SDK route inventories are
  current if the public contract changes.

Primary gates:

- `cargo nextest run -p nako-server local_session_auth --no-fail-fast`
- `cargo nextest run -p nako-api public_openapi --no-fail-fast`
- `cargo nextest run -p nako-client account --no-fail-fast`

## M4 - Closeout

Status: Complete. CSA-040 recorded fresh focused evidence and split frontend UI,
cookie transport, invitation onboarding, recovery, SSO, and Management Context
Links to follow-on lanes.

Exit criteria:

- Fresh verification evidence is recorded.
- `WORKSTREAM.json` status and completed task list match reality.
- Follow-ons for Admin Web account UI, Media Web login/account switching,
  invitations, cookie transport, account recovery, and SSO are split or
  explicitly deferred.

Primary gates:

- `cargo fmt --all -- --check`
- `python -m json.tool docs/workstreams/credential-session-auth/WORKSTREAM.json`
- `git diff --check`
