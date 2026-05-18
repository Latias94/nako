# Addon Token Grants Side Effects Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Smallest Current Repro

```powershell
rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs
```

Current known anchors include:

- `crates/taru-addon-protocol/src/lib.rs`
- `crates/taru-core/src/addon.rs`
- `crates/taru-core/src/repository/addon.rs`
- `crates/taru-db/src/addons.rs`
- `crates/taru-db/migrations/0012_addons.sql`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-api/src/extension.rs`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Gate Set

### Audit Gate

```powershell
rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs
git diff --check
```

Proves the current addon boundary inventory is fresh before schema, API, or
runtime auth changes.

### Token And Grant Gate

```powershell
cargo check -p taru-core --tests
cargo check -p taru-db --tests
cargo check -p taru-api --tests
cargo check -p taru-server --tests
cargo nextest run -p taru-db addon --no-fail-fast
cargo nextest run -p taru-server addon --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Add focused `taru-server` addon route tests when token issuance, revocation, or
rotation routes are introduced.

### Runtime Principal Gate

```powershell
cargo nextest run -p taru-server addon --no-fail-fast
cargo check -p taru-api --tests
```

Proves addon-to-Taru calls authenticate as addon principals and enforce
accepted permissions plus Media Library grants.

### Side Effect Intake Gate

```powershell
cargo nextest run -p taru-server addon_side_effect --no-fail-fast
cargo nextest run -p taru-db addon --no-fail-fast
git diff --check
```

Proves the first Addon Side Effect path validates actor, target, library scope,
idempotency, audit, and safe response behavior.

### Closeout Gate

```powershell
cargo fmt --all -- --check
git diff --check
```

Broaden to `cargo check --workspace --tests` and `cargo nextest run --workspace
--no-fail-fast` if token/grant changes affect shared auth, API, or repository
boundaries across the workspace.

### Review Gate

Run `review-workstream` before accepting schema/API changes, before accepting
the side-effect proof, and before lane closeout. Record blocking findings,
missing gates, and residual risks here.

## Evidence Anchors

- `docs/workstreams/addon-token-grants-side-effects/DESIGN.md`
- `docs/workstreams/addon-token-grants-side-effects/TODO.md`
- `docs/workstreams/addon-token-grants-side-effects/MILESTONES.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- code/test paths proving Addon Token, grant, principal, and side-effect
  behavior after implementation

## Fresh Evidence

2026-05-18, ATGSE-010:

- Workstream opened from the ARF-006 Post-M5 follow-up.
- First executable task set to current boundary audit before changing addon
  token, grant, or side-effect code.
- Existing `addons-automation` TODO redirected to this focused lane.
- Workstream index updated.
- Validation: `git diff --check`.

2026-05-18, ATGSE-020:

- Audit commands run:
  - `rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs`
  - `rg -n "Router|routes\(|require_auth|InboundAuthState|addons::routes|merge\(|nest\(" crates/taru-server/src`
  - `rg -n "secret|Secret|hash|sha|argon|constant_time|token_env|SecretString|Authorization|Bearer" crates/taru-core crates/taru-server crates/taru-db crates/taru-api Cargo.toml`
  - `rg -n "LibraryId|library_id|Library Access|Library-Scoped|grant|granted_scopes" crates/taru-core crates/taru-server crates/taru-db crates/taru-api docs/adr docs/workstreams/addon-token-grants-side-effects docs/workstreams/addons-automation`
- Current protocol boundary:
  - `crates/taru-addon-protocol/src/lib.rs` owns `AddonManifest`,
    `AddonResourceDeclaration`, `AddonResource`, `AddonScope`, `AddonAuth`,
    resource request/response envelopes, `ensure_scope_grant`, and
    `call_addon_resource`.
  - `AddonAuth::Bearer` and `AddonAuth::SharedSecret` are outbound
    Taru-to-Addon authentication modes. The caller supplies a runtime
    `bearer_token`; the protocol crate only emits headers and does not issue,
    store, rotate, or validate Addon Tokens for addon-to-Taru calls.
  - `ensure_scope_grant` checks resource `required_scopes` against a caller
    supplied `granted_scopes` slice. It has no Media Library dimension and no
    accepted-grant record identity.
- Current core/DB boundary:
  - `crates/taru-core/src/addon.rs` has `NewAddonRegistration` and
    `AddonRegistrationRecord` with manifest identity, base URL, status, and
    `granted_scopes: Vec<String>`.
  - `crates/taru-core/src/repository/addon.rs` only exposes registration
    upsert/get/list queries.
  - `crates/taru-db/migrations/0012_addons.sql` only creates
    `addon_registrations`; there is no token table, token hash, token prefix,
    revoked/rotated timestamps, grant table, library grant table, or
    side-effect intake table.
  - `crates/taru-db/src/addons.rs` serializes `granted_scopes_json` directly on
    the registration row. This is adequate for M5 resource-call gating but too
    coarse for accepted Addon Permissions, Library-Scoped Addon Grants, or
    token-specific revocation.
  - `crates/taru-db/src/codec.rs::row_to_addon_registration` round-trips only
    registration state. Existing DB tests near
    `sqlite_store_round_trips_addon_registration` prove only registration,
    status filtering, and scope JSON persistence.
- Current server/API boundary:
  - `crates/taru-server/src/app/addons.rs` validates manifests, de-duplicates
    requested scopes, enforces every declared resource's required scopes, stores
    `manifest_json`, and defaults registrations to `Disabled`.
  - `crates/taru-server/src/http/addons.rs` exposes only `POST /addons`,
    `GET /addons`, and `GET /addons/{addon_id}`. There are no issue, rotate,
    revoke, token inspection, grant update, addon-principal, or side-effect
    routes.
  - `crates/taru-api/src/extension.rs::RegisterAddonRequest` accepts
    `granted_scopes: Vec<AddonScope>` but no accepted-permission metadata,
    library grants, token issuance intent, or side-effect DTOs.
  - `crates/taru-server/src/http.rs` merges `addons::routes()` into the main
    router behind the same inbound bearer middleware as public/admin/internal
    routes. `crates/taru-server/src/http/auth.rs` validates exactly one
    configured bearer token from `AuthConfig`.
  - Public OpenAPI and TypeScript SDK checks explicitly exclude `/addons`,
    `/webhooks`, `/automation`, `/jobs`, and `/admin`, so addon management is
    not part of the Public Client API contract.
- Current tests:
  - `crates/taru-server/src/http/tests/addons.rs` covers disabled-by-default
    registration, invalid manifest rejection, missing scope rejection, list/get,
    and reference addon resource calls.
  - The reference addon test calls `call_addon_resource` manually from the
    server test after fetching the stored manifest. It does not exercise a
    server-side protected route invoked by an Addon Sidecar.
  - Existing secret-leakage tests cover admin diagnostics, automation,
    webhooks, metadata jobs, and public SDK/OpenAPI exclusions, but not
    generated Addon Tokens or side-effect responses.
- Missing Addon Token lifecycle:
  - No raw token generation path.
  - No one-time token issuance response.
  - No token hash or prefix storage policy.
  - No lookup by token hash.
  - No revocation or rotation record.
  - No token-created/rotated/revoked audit event.
  - No tests proving persisted token material is not plaintext.
- Missing accepted grants and library scope:
  - `granted_scopes_json` currently means "registration was allowed to call the
    declared outbound addon resources." It should not be stretched to mean
    token-bound accepted Addon Permissions for addon-to-Taru side effects.
  - There is no model for global grants versus Library-Scoped Addon Grants.
  - There is no API shape to grant a permission for one or more `LibraryId`
    values.
  - There is no runtime check that a target item/source belongs to a granted
    Media Library before accepting a protected addon write.
- Missing Addon Side Effect intake:
  - No side-effect ID, kind, actor addon ID, token ID, target IDs, library ID,
    permission, idempotency key, provenance JSON, payload JSON, validation
    state, audit timestamps, or safe-error state exists in core/DB/API.
  - No route family exists for Addon Sidecars to submit metadata/artwork/
    subtitle/Library File Write requests through Taru-owned APIs.
  - No idempotency behavior exists for addon-initiated side effects.
  - No tests cover allowed, denied, wrong-library, revoked-token,
    duplicate-idempotency, malformed-target, or redacted-response behavior.
- ADR impact:
  - ADR 0020 already contains the needed strategic decision: sidecar addons use
    revocable, rotatable Addon Tokens scoped to accepted permissions and
    library grants, and side effects must pass through Taru APIs.
  - No ADR amendment is required before ATGSE-030 if the next work follows ADR
    0020. A new ADR or ADR amendment should be split only if implementation
    chooses OAuth-first, remote multi-tenant addon authorization, broad Admin
    API reuse, or direct storage/file authority.
- Implementation direction for ATGSE-030:
  - Add first-class core records for Addon Token metadata and accepted grants
    instead of overloading `AddonRegistrationRecord.granted_scopes`.
  - Prefer a separate token table keyed by token ID and addon registration ID,
    with non-plaintext token verifier storage, prefix/label for admin
    diagnostics, created/last-used/revoked/rotated timestamps, and no raw token
    in normal list/get responses.
  - Add a separate accepted-grant model that can represent global grants and
    Library-Scoped Addon Grants by `LibraryId`.
  - Keep existing `AddonAuth` outbound semantics intact; Addon Token is an
    inbound addon-to-Taru credential and should not be confused with the
    outbound bearer/shared-secret header used when Taru calls an addon.
- Implementation direction for ATGSE-040:
  - Introduce an addon-principal auth path for addon-owned route families
    instead of letting Addon Tokens satisfy the admin bearer middleware.
  - Deny addon tokens on Public Client and Admin API routes unless a future
    route explicitly opts into addon-principal handling.
  - Resolve target Media Library before service mutation, then enforce the
    accepted grant against that library.
- Implementation direction for ATGSE-050:
  - Start with a narrow intake proof that persists a proposed side effect and
    validation result without broad canonical metadata or Library File Write
    mutation.
  - Treat concrete metadata, Managed Artwork, subtitle, and NFO/storage write
    handlers as follow-on breadth unless the proof slice remains small.
- Task status: DONE. This was a docs/audit task only; no Rust behavior changed.
- Validation passed:
  - `rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs`
  - `git diff --check`

2026-05-18, ATGSE-030:

- Addon Token lifecycle and accepted grant contract implemented across core,
  DB, app service, HTTP routes, API DTOs, docs, and tests.
- Token issue/rotate responses return the raw token only once, while list and
  revoke responses stay redacted.
- Persisted token verifier material uses `token_hash`; token rotation is gated
  by addon ownership in the DB layer and rolls back on cross-addon mismatch.
- Accepted Addon Permissions are stored separately from manifest
  `granted_scopes`, with optional Library-Scoped Addon Grants.
- Validation passed:
  - `cargo check -p taru-core --tests`
  - `cargo check -p taru-db --tests`
  - `cargo check -p taru-api --tests`
  - `cargo check -p taru-server --tests`
  - `cargo nextest run -p taru-db addon --no-fail-fast`
  - `cargo nextest run -p taru-server addon --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Focused test results:
  - `taru-db`: 3 addon-filtered tests passed.
  - `taru-server`: 3 addon-filtered tests passed.
- Review status: self-review against the workstream contract and ADR 0020 found
  no remaining blocking issues. Runtime addon-principal enforcement remains
  intentionally deferred to ATGSE-040.

Fresh verification is required before marking any later task, Codex goal, or
lane complete.
