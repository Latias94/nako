# Credential Session Auth - Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Smallest Current Repro

```powershell
cargo nextest run -p nako-server -E 'test(local_session_auth) | test(admin_v1_access_management) | test(bearer_auth)' --no-fail-fast
```

This proves the shipped backend credential/session path: Admin credential
provisioning, Public Client login/current-account/logout, local session Bearer
auth, disabled-user rejection, and bootstrap admin token compatibility.

## Gate Set

### Documentation Gate

```powershell
python -m json.tool docs/workstreams/credential-session-auth/WORKSTREAM.json
git diff --check -- docs/workstreams/credential-session-auth docs/workstreams/README.md docs/adr
```

Proves the durable lane metadata and documentation edits are syntactically
valid and whitespace-clean.

### Storage Contract Gate

```powershell
cargo nextest run -p nako-core identity --no-fail-fast
cargo nextest run -p nako-db credential_session --no-fail-fast
```

Proves credential/session records and repository behavior.

### Admin Credential Gate

```powershell
cargo nextest run -p nako-server admin_local_password --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
```

Proves administrator password provisioning remains Admin API-owned and
redaction-safe.

### Public Session Gate

```powershell
cargo nextest run -p nako-server local_session_auth --no-fail-fast
cargo nextest run -p nako-api public_openapi --no-fail-fast
cargo nextest run -p nako-client account --no-fail-fast
```

Proves Public Client login/current-account/logout contracts and Bearer session
auth resolution.

### Formatting Gate

```powershell
cargo fmt --all -- --check
```

Proves Rust formatting stayed consistent.

## Evidence Anchors

- `docs/workstreams/credential-session-auth/DESIGN.md`
- `docs/workstreams/credential-session-auth/TODO.md`
- `docs/workstreams/credential-session-auth/MILESTONES.md`
- `docs/workstreams/credential-session-auth/HANDOFF.md`
- `docs/adr/0037-local-credential-and-session-auth.md`
- `crates/nako-core/src/identity.rs`
- `crates/nako-core/src/repository/identity.rs`
- `crates/nako-db/src/sqlite/identity.rs`
- `crates/nako-db/src/postgres/identity.rs`
- `crates/nako-server/src/http/auth.rs`
- `crates/nako-server/src/http/account.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-api/src/openapi.rs`
- `crates/nako-client-protocol/src/lib.rs`
- `crates/nako-client/src/lib.rs`
- `sdk/typescript/src/index.ts`
- `sdk/kotlin/src/main/kotlin/dev/nako/sdk/NakoClientSdk.kt`

## Final Verification - 2026-05-26

All commands were run from `F:\SourceCodes\Rust\nako`.

| Command | Result | Proves |
| --- | --- | --- |
| `cargo nextest run -p nako-api -E 'test(public_openapi) | test(admin_contract) | test(sdk)' --no-fail-fast` | Passed, 19 tests | Public OpenAPI, Admin contract, TypeScript SDK, and Kotlin SDK match the new account/session contract without leaking admin/internal surfaces. |
| `cargo nextest run -p nako-client -E 'test(account)' --no-fail-fast` | Passed, 1 test | Rust client login skips bearer auth; current-account and logout require bearer auth. |
| `cargo nextest run -p nako-client-protocol -E 'test(public_route_inventory)' --no-fail-fast` | Passed, 2 tests | Public route inventory includes account routes and still rejects internal/secret surfaces. |
| `cargo nextest run -p nako-server -E 'test(local_session_auth) | test(admin_v1_access_management) | test(bearer_auth)' --no-fail-fast` | Passed, 4 tests | Admin local password provisioning, Public Client login/me/logout, disabled-user rejection, session Bearer auth, and bootstrap admin token behavior. |
| `cargo nextest run -p nako-db -E 'test(credential_session)' --no-fail-fast` | Passed, 1 test | SQLite credential/session repository lifecycle; PostgreSQL equivalent remains behind local PostgreSQL test configuration. |
| `cargo nextest run -p nako-core -E 'test(identity)' --no-fail-fast` | Passed, 5 tests | Existing identity, role, and effective Library Access behavior remains intact. |
| `cargo fmt --all -- --check` | Passed | Rust formatting is clean. |
| `python -m json.tool docs/workstreams/credential-session-auth/WORKSTREAM.json` | Passed | Workstream metadata is valid JSON. |
| `git diff --check` | Passed | Working diff has no whitespace errors. |

## Review Notes

- Workstream compliance: no blocking findings. CSA-010, CSA-020, CSA-030, and
  CSA-040 satisfy the backend-only target state.
- Code quality: no blocking findings. Passwords are Argon2 hashed before
  persistence; session tokens are generated as opaque bearer-compatible values
  and stored only as SHA-256 hashes.
- Boundary review: Admin API exposes password provisioning and a boolean
  credential state; Public Client exposes login/current-account/logout only.
  Public generated contracts were refreshed for Rust, TypeScript, and Kotlin
  clients.
- Residual risk: PostgreSQL credential/session behavior was implemented against
  the shared repository contract but the local run did not execute the ignored
  PostgreSQL contract without `NAKO_TEST_POSTGRES_URL`.

## Notes

- Fresh verification is required before any task or lane is marked complete.
- Do not count frontend checks as required for this lane unless a later task
  intentionally touches frontend code.
