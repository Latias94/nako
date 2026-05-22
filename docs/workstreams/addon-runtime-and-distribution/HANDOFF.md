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

## Active Task

- Task ID: ARD-020
- Owner: codex
- Files:
  - `crates/taru-addon-protocol`
  - `crates/taru-api/src/admin.rs`
  - `crates/taru-server/src/app`
  - `crates/taru-server/src/http/admin.rs`
  - `docs/workstreams/addon-runtime-and-distribution`
- Validation:
  - focused `taru-addon-protocol` tests
  - focused Admin DTO/server tests for redacted install-guide previews
  - `cargo fmt --all -- --check`
  - `git diff --check`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: READY
- Review: define package/install descriptor semantics without adding process
  supervision, marketplace behavior, package signing, Native Plugin ABI, admin
  token leakage, Public Client API churn, or direct library writes.

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

- None for ARD-020.

## Next Recommended Action

Execute ARD-020: add or refine the package/install descriptor and redacted
install-guide preview boundary. Keep Addon Manager discovery/install/update,
package signing, process supervision, logs, rollback, Native Plugin ABI,
downloader protocols, local AI runtime, Public Client API changes, and direct
library writes out of scope.
