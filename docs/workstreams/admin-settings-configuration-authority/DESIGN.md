# Admin Settings Configuration Authority

Status: Closed
Last updated: 2026-05-26

## Why This Lane Exists

`admin-web-v2-settings-mutation-authority` proved that the current `/settings`
surface is diagnostic-only. Global settings are loaded from `NakoServerConfig`
at process start, copied into router middleware and service resources, and
never persisted through an Admin API path.

Admin Web V2 cannot honestly offer settings editing until the server has a
source of truth for Admin-mutated settings.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/adr/0009-resolve-provider-secrets-from-environment.md`
- `docs/workstreams/admin-web-v2-settings-mutation-authority/`
- `docs/workstreams/network-access-boundary/`
- `docs/workstreams/metadata-profile-configuration-authority/`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/app/composition.rs`
- `crates/nako-server/src/http.rs`
- `crates/nako-server/src/http/network.rs`
- `crates/nako-db/`

## Problem

The server has no accepted place to persist global Admin settings:

- `NakoServerConfig` is read from TOML by `load_config()`.
- `config.rs` has no save/update function.
- `NakoAppComposition` stores an immutable cloned config.
- `build_router()` creates auth and network middleware state from config.
- runtime semaphores and services are built from config values at startup.
- `nako-db` has no global settings repository or migration.

Therefore a naive `PUT /admin/v1/system/config` would either:

- update a value that does not survive restart;
- update a value without rebuilding the router/runtime resource that uses it;
- conflict with TOML on the next startup;
- or expose raw config material that current diagnostics intentionally redact.

## Target State

When this lane closes, Nako has a backend-owned configuration authority for the
first Admin-mutated global settings slice:

- a persisted desired-state model or explicit runtime-only model;
- deterministic startup merge rules with TOML;
- redaction-safe Admin API DTOs;
- validation and conflict semantics;
- clear `applies_immediately` / `requires_restart` reporting;
- tests proving restart behavior and no unsafe data leakage.

## In Scope

- Decide the first field group to support.
- Add a backend persistence model when required.
- Define startup merge behavior between TOML and Admin-mutated settings.
- Define Admin API route(s) for review-plan and confirmed mutation.
- Add SQLite/PostgreSQL parity if persistence is introduced.
- Add focused tests for validation, idempotency/conflicts, restart behavior,
  and redaction.
- Update HTTP API docs and generated Admin Web contract.

## Out Of Scope

- Admin Web form implementation.
- Raw TOML editing.
- Secret value storage or rotation.
- Provider credentials, tokens, endpoint URLs, raw paths, storage roots, or env
  var editing.
- User, Role, or Library Access.
- Public Client API, public OpenAPI, generated public SDK, or
  `nako-client-protocol` changes.
- Full dynamic runtime reconfiguration for every service.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| There is no global settings repository today. | High | `nako-db` has no config/settings migration or repository trait. | Reuse the discovered repository instead of adding one. |
| Network/auth changes cannot be hot-applied with the current router. | High | `build_router()` copies `AuthConfig` and `NetworkAccessConfig` into middleware state. | A narrower hot-apply API could be implemented first. |
| Runtime budgets are partially startup-owned. | High | `NakoRuntimeResources::build()` creates semaphores from config. | A live budget service may be a valid first slice if added deliberately. |
| TOML should remain bootstrap authority unless explicitly superseded. | Medium | Existing server commands load TOML; Metadata Profile authority uses explicit source markers. | A different operator model requires an ADR or design update. |

## Architecture Direction

Do not expose a raw config editor. Model Admin settings as explicit field groups
with source tracking and effect semantics:

- `configured`: supplied by TOML at startup.
- `admin`: persisted through Admin API.
- `runtime`: changed for the current process only.
- `effective`: the value used by the current process.

For the first slice, prefer a field group that avoids secrets, URLs, local
paths, roots, and provider credentials. If persistence is introduced, implement
it behind a dedicated repository instead of overloading unrelated domain tables.

Admin routes stay under `/admin/v1/*` and return safe summaries, plan facts,
and readiness/conflict results. Public client contracts remain unchanged.

## Accepted First Slice

ASCA-020 selects the metadata raw cache retention group:

- `metadata.raw_cache_retention_ms`
- `metadata.maintenance.raw_cache_cleanup_on_startup`

This slice is safe for the first backend authority because it is a global
operator setting, contains no secrets, URLs, local paths, storage roots, hosts,
provider credentials, or environment variable names, and already has observable
startup behavior.

Semantics:

- Admin writes are persisted as a desired-state override in `nako-db`.
- Startup merges TOML first, then applies the persisted Admin override when
  present.
- The current process does not hot-apply the changed values. A PUT response can
  report `requires_restart` when the stored desired value differs from the
  active process config.
- After restart, the same persisted Admin override becomes the active effective
  value and the settings response reports `effect = active`.
- `GET /admin/v1/settings/metadata/raw-cache` returns either the configured
  value (`source = configured`) or the persisted override (`source = admin`).
- `PUT /admin/v1/settings/metadata/raw-cache` replaces the field group and
  validates `retention_ms > 0`.
- Public Client API, public OpenAPI, and generated public SDK inventories remain
  unchanged.

## Closeout Condition

This lane can close when:

- the first Admin settings field group has accepted source-of-truth semantics;
- the backend route(s) and persistence/runtime behavior are implemented or a
  deliberate runtime-only model is documented;
- restart and redaction tests pass;
- generated Admin Web contract and HTTP docs are updated;
- `admin-web-v2-settings-mutation-authority` can safely continue with UI
  controls for the implemented route.
