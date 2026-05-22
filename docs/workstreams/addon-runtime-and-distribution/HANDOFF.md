# Addon Runtime And Distribution — Handoff

Status: Complete
Last updated: 2026-05-22

## Current State

This lane is complete and has returned to `post-rpd-product-hardening` for
final umbrella closeout.

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

ARD-020 through ARD-060 are complete. The lane shipped Addon Sidecar
package/install/runtime readiness, routing, and proposal/intake handoff, not
Addon Manager automation or Native Plugin runtime.

## Current Task

- Task ID: ARD-060
- Owner: planner
- Files:
  - `docs/workstreams/addon-runtime-and-distribution`
  - `docs/workstreams/post-rpd-product-hardening`
  - `docs/workstreams/README.md`
- Validation:
  - `verify-rust-workstream` final evidence for the lane
  - `python -m json.tool` for local and parent WORKSTREAM.json files
  - `git diff --check`
  - `git diff --name-only -- crates/taru-client-protocol`
- Status: DONE
- Review: close the lane or split follow-ons for Addon Manager discovery/
  install/update, package signing, marketplace, process supervision, logs,
  rollback, Native Plugin ABI, downloader protocols, Public Client surfaces,
  and local AI/model runtime. Do not hide those scopes in this lane.

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
- Added Admin-only runtime readiness DTOs, app checks, and `POST
  /admin/v1/addons/{addon_id}/runtime-readiness`.
- Added Admin Web generated contract/client support for runtime readiness.
- Added route tests proving typed classification of ready/degraded/unavailable
  sidecars, protocol mismatch, manifest mismatch, missing grants, missing
  Secret Reference configuration, network policy blockers, and unsafe responses
  without echoing tokens, raw network errors, sidecar payloads, URLs, or secret
  fields.
- Added durable task/event routing plans, `JobKind::AddonTask`, Admin
  `POST /admin/v1/addons/{addon_id}/routing-plans`, generated Admin Web
  contract/client support, and DB/Admin tests proving idempotent manifest
  replacement, stale-plan removal, disabled/missing-grant/unsupported-event
  deferral, no hidden scheduler/outbox side effects, and no manifest secret
  echo.
- Added Addon Token runtime routes `POST /addon/v1/generated-artifacts` and
  `POST /addon/v1/acquisition/intake/candidates`.
- Generated Artifact submissions now require Addon scopes, revalidate
  library/item/source ownership, upsert an Addon-backed Automation Provider,
  preserve existing provider capabilities across later capability submissions,
  enqueue an Addon Task job, and create Proposed AILO automation artifacts.
- Generated Artifact idempotency now replays identical idempotency keys and
  rejects conflicting payloads with safe `409` diagnostics.
- Acquisition candidates now enter DWI `AddonProposed` acquisition-intake
  records with redacted source diagnostics and no promotion/apply authority.
- Focused tests prove no Canonical Metadata, NFO sidecar, Media Source, Managed
  Import, or library-file writes occur during Addon artifact/intake handoff.

Validation so far:

- `cargo check -p taru-api -p taru-server --tests`
- `cargo nextest run -p taru-server addon_generated_artifact_handoff --no-fail-fast`
- `cargo nextest run -p taru-server addon_acquisition_candidate_handoff --no-fail-fast`
- `cargo nextest run -p taru-server addon_handoff --no-fail-fast`
- `cargo nextest run -p taru-api admin_contract --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `git diff --name-only -- crates/taru-client-protocol`

Broad-gate note: `cargo nextest run -p taru-server addons --no-fail-fast` and a
retry with `-j 1` were attempted, but the Windows host hit socket error 10055
after the ARD-050 focused tests had passed. Treat that as an environment-limited
attempt, not as passing closeout evidence.

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

- None.

## Next Recommended Action

Return to `post-rpd-product-hardening` and close the umbrella. If new work is
needed, open a dedicated follow-on lane for Addon Manager discovery/install/
update, marketplace hosting, package signing trust roots, process/container
supervision, logs/rollback, Native Plugin ABI, downloader protocol adapters,
local AI/model runtime, Public Client surfaces, or concrete Addon distribution
automation. Do not hide those scopes inside this closed lane.
