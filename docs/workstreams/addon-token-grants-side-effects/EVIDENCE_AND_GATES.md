# Addon Token Grants Side Effects Evidence And Gates

Status: Proposed
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
cargo nextest run -p taru-db addon --no-fail-fast
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

Fresh verification is required before marking any later task, Codex goal, or
lane complete.

