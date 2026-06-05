# Next Lane Synthesis

Date: 2026-06-05

## Inputs

- `research/next-parallel-contract-gates.md`
- `research/next-product-development-lanes.md`
- `research/next-architecture-refactor-lanes.md`
- `research/next-operations-release-lanes.md`
- Completed child task:
  `.trellis/tasks/06-05-admin-route-inventory-parity-gate/`
- Completed child task:
  `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/`

## Executive Decision

Use a mixed plan with explicit serial ownership for shared contracts.

Do not start a broad fearless refactor campaign now. The four follow-up audits
agree that Nako's current bottleneck is not shallow global architecture. The
near-term risk is contract drift across Public Client playback capability,
Admin generated contracts, shared identity/config surfaces, Addon protocol
shape, and durable job/runtime policy.

The completed Admin route inventory parity gate should become a standing gate
for Admin work. The new highest-priority serial task is Public Client playback
capability contract parity.

## What Changed Since The Original Synthesis

The original parent synthesis recommended the Admin route inventory parity gate
as the first contract-safety task. That task is now complete. Its result does
not make Admin work free to parallelize; it means future Admin route, DTO, and
generated contract work must pass the parity gate and should still have one
contract owner per wave.

The playback output/profile child audit is also complete. It found that future
HEVC/AV1, hardware tone mapping, image subtitle execution, device-profile, and
profile-v2 work should wait behind a Public Client playback capability parity
gate.

## Ranked Recommendation

| Rank | Lane | Type | Parallel rule | Decision |
| --- | --- | --- | --- | --- |
| 1 | `public-client-playback-capability-contract-parity-gate` | contract gate | serial-first | Create this as the next shared contract task before playback execution expansion. |
| 2 | `06-05-remote-access-cookbook-config-gates` | docs/ops implementation | can parallel | Start now if it stays in cookbook docs, deploy/config fixtures, and release/config assertions. |
| 3 | `06-05-addon-resource-flow-pattern-audit` | architecture audit | can parallel as research | Continue as design/research before Addon Manager, process lifecycle, or more Addon Resource implementation. |
| 4 | `06-05-source-hash-triggering-reconciliation-policy` | architecture audit | can parallel as research | Continue as policy/research before scan-originated enqueue, Admin manual trigger, retry/requeue, or duplicate suggestion implementation. |
| 5 | Admin contract generator deepening | selective refactor | serial-first only if Admin-heavy lanes are next | Defer unless the next queue needs multiple Admin DTO/generated contract changes. |

## Recommended Parallel Queue Shape

1. Assign one serial owner for
   `public-client-playback-capability-contract-parity-gate`.
2. In parallel, run the remote access cookbook/config fixture task as the
   operator-visible implementation lane.
3. In parallel, keep Addon resource flow and Source Fingerprint hash
   reconciliation as architecture/research lanes with docs-only writable
   scope.
4. Hold Admin generated contract, shared identity, shared config/settings,
   Addon protocol/manifest, and broad durable-job/runtime changes behind a
   single owner when they are needed.

## Safe Parallel Work

- Remote access cookbook sections and deploy/config fixtures that do not
  change config structs, Admin DTOs, Public Client endpoint discovery, or
  tunnel runtime.
- Addon host-owned resource-flow research, ADR/spec drafts, and future
  server-local pattern planning that does not change `nako-addon-protocol` or
  generated Admin contracts.
- Source Fingerprint hash trigger/reconciliation policy research that does
  not edit `jobs.rs`, source identity repositories, schema, or Admin DTOs.
- Playback architecture-map reconciliation that does not edit playback DTOs,
  generated SDKs, or `nako-playback`/`nako-transcode` identity files.

## Serial Or Unique-Owner Work

- Public Client playback capability DTOs, OpenAPI, SDK generation, Rust client
  builders, server playback/renderer mapping, and HTTP docs.
- Admin route inventory, Admin DTOs, `admin_contract.rs`, and generated Admin
  TypeScript contracts.
- Shared identity/access changes such as IDs, principals, Library Access,
  Source Duplicate Relationship semantics, and source identity exposure.
- Shared config/settings changes such as `NetworkAccessConfig`, preflight
  classifications, runtime settings, trusted proxy behavior, and Admin
  settings DTOs.
- Addon manifest/protocol version, permission/scope, hosted surface, task,
  event, resource, health, or official catalog wire-shape changes.
- Durable job scheduler/resource policy, HLS artifact identity, staging
  manifest leases, and hidden background work.

## Do Not Start Yet

- HEVC/AV1 executable output paths.
- Hardware tone-map execution.
- Image subtitle burn-in execution.
- Built-in tunnel provider runtime, endpoint discovery, or LAN/remote client
  endpoint selection.
- Automatic Source Duplicate Relationship mutation from hash evidence.
- Addon Manager process lifecycle, Docker socket access, automatic updates, or
  hosted-page credential mediation.
- Mechanical splits of large Addon protocol/client/catalog files.
- Disk-scan executor registry with only two disk-scan job kinds.
- Global fearless refactor cleanup.

## Follow-On Task To Create

Create `public-client-playback-capability-contract-parity-gate` before any
playback profile-v2 or execution-expansion task.

Suggested purpose:

- Inventory current Public Client playback capability fields across protocol
  DTOs, OpenAPI, generated SDKs, Rust client builders, server query/body
  mapping, renderer mapping, and HTTP docs.
- Add parity tests or route/schema assertions that fail when those surfaces
  drift.
- Confirm Public Client capability fields remain client/player facts only and
  do not expose FFmpeg, GPU, operator policy, resource pressure, local paths,
  bearer tokens, or principal/source identity internals.

Suggested validation:

```powershell
cargo nextest run -p nako-client-protocol public_route_inventory --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
cargo nextest run -p nako-client-core -p nako-client --no-fail-fast
cargo check -p nako-client-protocol -p nako-client-core -p nako-client --tests
cargo check -p nako-api -p nako-server --tests
git diff --check
```

## Bottom Line

Continue product development and architecture audits together, but only after
the shared contract owners are explicit. The next best move is not broad
refactor; it is a serial Public Client playback capability parity gate plus
parallel remote-access operations work and docs-only Addon/source-hash
architecture audits.
