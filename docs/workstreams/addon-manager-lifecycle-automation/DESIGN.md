# Addon Manager Lifecycle Automation

Status: Active
Last updated: 2026-05-23

## Why This Lane Exists

Nako's released alpha now proves the manual addon path:

- a published server image can run;
- `nako-addon-protocol` and `nako-addon-client` are public crates;
- `nako-metadata-scraper@0.1.0-alpha.1` can be installed and smoke-tested;
- the first official companion addon can register, health-check, and serve one
  hosted metadata resource call through Nako.

That proves the manual operator path, not the manager path. The next product gap
is a built-in Addon Manager control plane that can own lifecycle orchestration
for manually or later-package-sourced sidecars without collapsing marketplace,
package signing, provider breadth, or Native Plugin ABI into the first slice.

## Problem

Operators still have to start and manage addon sidecars manually. Nako can
describe, register, validate, and diagnose addons, but it does not yet own the
discover/install/update/remove/supervise/log loop for addon sidecars as a first
class runtime surface.

## Target State

When this lane closes, Nako should be able to:

- identify a managed addon source and resolve an installable addon package or
  descriptor;
- install an addon into a Nako-owned managed lifecycle slot;
- update or remove that addon with explicit operator confirmation;
- supervise addon process state, readiness, and logs through a Nako-owned
  control plane;
- keep marketplace hosting, package signing, and provider breadth as separate
  follow-ons;
- keep the Addon Protocol contract and the official addon smoke stable while
  manager features evolve.

## Scope

- Addon Manager discovery and lifecycle plan modeling.
- Operator-confirmed install/update/remove flows.
- Addon process supervision, restart/stop semantics, and log exposure through
  redacted diagnostics.
- Minimal UI/API contract changes needed to surface managed addon lifecycle
  state.
- Validation and documentation for the first manager-owned addon slice.

## Non-Goals

- Marketplace hosting or distribution policy.
- Package signing trust roots.
- Native Plugin ABI or in-process addon execution.
- Broad provider breadth beyond the official companion addon path.
- OAuth-first addon auth redesign.
- Public client API changes unrelated to addon management.

## Architecture Direction

Treat the manager as a Nako-owned lifecycle controller, not as a source of
hidden process magic. The manager should own:

- install/update/remove intent capture;
- source resolution and version selection policy;
- supervised process lifecycle;
- operator-visible health/log/readiness state;
- safe rollback boundaries.

The sidecar should still own its provider logic and protocol envelope. The
manager should not smuggle admin credentials into addons or convert the sidecar
boundary into an in-process plugin ABI.

## Related Docs

- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- `docs/workstreams/official-addon-e2e-alpha2/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/admin-addon-operations-mvp/`
