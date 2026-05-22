# Addon Runtime And Distribution — Handoff

Status: Active
Last updated: 2026-05-22

## Current State

This lane is open as the next mainline child of
`post-rpd-product-hardening` after AI Assisted Library Ops closed.

Prerequisites are complete:

- `addon-architecture-deepening` deepened Addon Side Effect runtime, manifest
  declarations, protected write payloads, Library File Write seam, and Admin
  Addon API boundaries.
- `admin-addon-operations-mvp` added `/admin/v1/addons` lifecycle mutation,
  unregister semantics, health checks, hosted surface read models,
  resource-call diagnostics, token management, and grant management.
- `downloads-watch-folder-intake` added acquisition intake candidates and
  Managed Import handoff without direct library writes.
- `network-access-boundary` added network policy/readiness, trusted proxy/origin
  enforcement, and Admin-only network diagnostics without built-in tunnel
  runtime.
- `ai-assisted-library-ops` added Generated Artifact proposal/readiness,
  Admin-only diagnostics, and explicit accept/reject planning without
  autonomous writes.

ARD-010 is complete. The lane is scoped to Addon Sidecar package/install/runtime
readiness and routing, not Addon Manager automation or Native Plugin runtime.

## Current Task

- Task ID: ARD-030
- Owner: codex
- Files:
  - `crates/taru-addon-client`
  - `crates/taru-api/src/extension.rs`
  - `crates/taru-server/src/app`
  - `crates/taru-server/src/http/addons.rs`
  - `apps/admin-web/src/adminApi`
  - `docs/workstreams/addon-runtime-and-distribution`
- Validation:
  - focused addon-client/app/Admin tests
  - `cargo nextest run -p taru-server addons --no-fail-fast`
  - `cargo nextest run -p taru-api --no-fail-fast`
  - `npm run check` from `apps/admin-web`
  - `git diff --name-only -- crates/taru-client-protocol`
  - `cargo fmt --all -- --check`
  - `git diff --check`
- Status: READY
- Review: add runtime readiness diagnostics without automatic Addon process or
  container supervision.

Progress so far:

- Added protocol-level Addon install descriptor, runtime requirement,
  runtime-reference summary, Secret Reference binding, and install-guide DTOs.
- Added validation that rejects missing/multiple runtime references, local
  paths, credential-bearing runtime references, unknown/duplicate Secret
  Reference bindings, and likely raw secret values without echoing rejected
  values.
- Added Admin DTOs, app validation, and `POST
  /admin/v1/addons/install-guide-preview`.
- Added focused Admin route tests proving no admin token, Addon Token, raw
  secret value, local path, or raw package content is echoed.
- Ran focused protocol/Admin tests plus `cargo nextest run -p taru-server addons
  --no-fail-fast` and `cargo nextest run -p taru-api --no-fail-fast`.

## Decisions Since Opening

- Start with Addon Sidecar package/install/runtime readiness, not an Addon
  Manager.
- Keep Addon code outside Taru. Taru validates package/manifest/install facts
  and calls sidecars through the Addon Protocol.
- Install guidance may help operators run a sidecar, but Taru does not own
  process/container lifecycle in this lane.
- Addon Tasks, Event Subscriptions, Generated Artifacts, acquisition candidates,
  and Library File Writes must reuse existing Taru-owned job/outbox/proposal/
  intake/side-effect boundaries.
- `taru-addon-protocol` remains permissive and dependency-light.
- Admin diagnostics are allowed; Public Client API and `taru-client-protocol`
  changes are not part of the first slice.

## Blockers

- None for ARD-030.

## Next Recommended Action

Start ARD-030 by adding Admin-only runtime readiness diagnostics that classify
sidecar reachability, protocol version mismatch, manifest mismatch, grant gaps,
missing Secret References, network policy blockers, and unsafe responses
without echoing raw network errors or sidecar payloads. Keep Addon Manager
discovery/install/update, package signing, process supervision, logs, rollback,
Native Plugin ABI, downloader protocols, local AI runtime, Public Client API
changes, and direct library writes out of scope.
