# Addon Token Grants Side Effects Milestones

Status: Completed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Outcome: ARF-006 has a focused workstream with authority, scope, and first
audit task.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- Existing `addons-automation` Post-M5 follow-up points to this lane.
- Workstream index links the new lane.
- Docs-only validation passes.

Primary evidence:

- `docs/workstreams/addon-token-grants-side-effects/DESIGN.md`
- `docs/workstreams/addon-token-grants-side-effects/TODO.md`
- `docs/workstreams/addons-automation/TODO.md`

## M1 - Current Boundary Audit

Outcome: Current addon code and docs are classified before schema/API changes.

Exit criteria:

- Manifest auth, registration persistence, granted scope handling, HTTP routes,
  and tests are inventoried.
- Missing token lifecycle, library grant, and side-effect intake boundaries are
  recorded with concrete file anchors.
- ADR amendment need is accepted, rejected, or split.

Primary gates:

- `rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs`
- `git diff --check`

## M2 - Token And Grant Contract

Outcome: Taru has a clear Addon Token and accepted-grant model.

Exit criteria:

- Token issuance returns the raw token only at creation/rotation time.
- Persisted token material uses a non-plaintext storage policy.
- Revocation and rotation do not change addon registration identity.
- Accepted Addon Permissions are distinguishable from manifest-requested
  permissions.
- Library-Scoped Addon Grants are stored and queryable.

Primary gates:

- `cargo check -p taru-core --tests`
- `cargo check -p taru-db --tests`
- `cargo check -p taru-api --tests`
- `cargo check -p taru-server --tests`
- `cargo nextest run -p taru-db addon --no-fail-fast`
- `cargo nextest run -p taru-server addon --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

Evidence anchors:

- `crates/taru-core/src/addon.rs`
- `crates/taru-core/src/repository/addon.rs`
- `crates/taru-db/migrations/0021_addon_tokens_and_grants.sql`
- `crates/taru-db/src/addons.rs`
- `crates/taru-db/src/tests.rs`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-server/src/http/tests/addons.rs`
- `crates/taru-api/src/extension.rs`
- `docs/api/HTTP_API.md`

## M3 - Runtime Addon Principal Enforcement

Outcome: addon-to-Taru calls authenticate as an addon principal, not as an admin
or public client.

Exit criteria:

- Runtime auth resolves token to addon registration identity and grant set.
- Revoked and rotated tokens behave predictably.
- Missing permission and wrong-library access are denied before service work.
- Addon routes are separate from Public Client and Admin API authority.

Primary gates:

- focused `cargo nextest run -p taru-server addon --no-fail-fast`
- `cargo check -p taru-api --tests`

Evidence anchors:

- `crates/taru-core/src/addon.rs`
- `crates/taru-core/src/error.rs`
- `crates/taru-core/src/repository/addon.rs`
- `crates/taru-db/src/addons.rs`
- `crates/taru-server/src/app/addons.rs`
- `crates/taru-server/src/http.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-server/src/http/auth.rs`
- `crates/taru-server/src/http/tests/addons.rs`
- `crates/taru-api/src/extension.rs`
- `docs/api/HTTP_API.md`

## M4 - Addon Side Effect Intake Proof

Outcome: one protected Addon Side Effect path proves the intake seam without
opening broad write features.

Exit criteria:

- Intake records actor, target, library, permission, idempotency key,
  provenance, validation result, and audit state.
- Allowed, denied, wrong-library, revoked-token, duplicate-idempotency, and
  redacted-response behavior is covered by tests.
- The proof does not grant raw filesystem paths, database access, or admin
  credentials to Addon Sidecars.

Primary gates:

- focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`
- relevant `taru-db` tests
- `git diff --check`

## M5 - Docs, Gates, And Closeout

Outcome: the lane is either closed with evidence or split into narrower
protected-write follow-ons.

Result: completed on 2026-05-18. The lane is closed, and concrete
protected-write application behavior is split to
`docs/workstreams/addon-protected-writes/`.

Exit criteria:

- `EVIDENCE_AND_GATES.md` records fresh command evidence.
- User-facing and addon-author docs reflect shipped token/grant/intake
  behavior.
- Remaining metadata/artwork/subtitle/Library File Write breadth is split to
  `docs/workstreams/addon-protected-writes/`.
- `WORKSTREAM.json` status and `HANDOFF.md` match reality.
