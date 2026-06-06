# Cross-lane architecture audit for parallel development

## Goal

Audit Nako's major architecture lanes broadly enough to choose the next
parallel development queue, instead of continuing narrowly from the latest
storage/VFS source fingerprint work. The output should decide where to run
architecture planning first, where a bounded feature follow-on is ready, and
where fearless refactor cleanup is justified by real shallow-module evidence.

## What I already know

- The current `main` branch has been pushed and the working tree started clean.
- The previous source fingerprint hash slices finished Admin overview, Jobs
  drill-down filters, scheduler integration, execution, and evidence
  persistence.
- The user explicitly wants a broader look across decoding/playback, network
  tunnel/remote access, Addon boundaries, Admin/API surfaces, storage/VFS, and
  control-plane concerns.
- The user approved using subagents for parallel review.
- This task is an audit/planning task. It should not implement production code.

## Requirements

- Review multiple capability areas, not only storage/VFS.
- Use parallel subagent review where scopes are independent.
- Preserve Nako vocabulary from `CONTEXT.md` and lane authority order:
  ADRs -> `docs/architecture/*.md` -> `.trellis/spec/*` ->
  `.trellis/tasks/*` -> historical workstreams -> chat.
- Classify findings into:
  - architecture-map/doc reconciliation,
  - ready bounded implementation follow-on,
  - fearless refactor cleanup candidate,
  - needs more product decision before implementation,
  - unsafe to parallelize.
- Identify shared scopes that would serialize otherwise parallel work:
  `nako-api`, generated contracts, Admin Web route state, schema migrations,
  durable job scheduler/runtime policy, auth/redaction, and `CONTEXT.md`.
- Produce a ranked next-work recommendation and a suggested parallel queue
  shape.

## Acceptance Criteria

- [x] Research notes exist for playback/decoding/transcode.
- [x] Research notes exist for remote access/network tunnel and operations.
- [x] Research notes exist for Addon Protocol, Addon Sidecar, and extension
      boundary work.
- [x] Research notes exist for Admin/API/Web product and generated contract
      pressure.
- [x] Research notes exist for storage/VFS/library/control-plane source
      identity and durable job work.
- [x] A synthesis ranks candidate next tasks by value, risk, parallel safety,
      and refactor readiness.
- [x] The synthesis explicitly answers: architecture audit next, fearless
      refactor next, or a mixed plan.
- [x] No production code is changed by this audit task.

## Definition of Done

- Subagent review results are persisted under this task's `research/` directory.
- `prd.md` is updated with the final recommendation and any ADR-lite decisions.
- If docs are touched, changes are limited to this task directory unless the
  user explicitly approves architecture-map reconciliation.
- `git diff --check` passes before any commit.

## Out of Scope

- No Rust, TypeScript, schema, generated contract, or API implementation
  changes.
- No broad code formatting or cleanup.
- No new architecture lane or worktree until the audit recommends one and the
  user approves it.
- No copying from reference repositories; reference material may only inform
  behavior and boundary analysis.

## Technical Notes

- Primary docs to inspect:
  - `CONTEXT.md`
  - `docs/ARCHITECTURE.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/LIBRARY_PIPELINE.md`
  - `docs/architecture/CONTROL_PLANE.md`
  - `docs/architecture/OPERATIONS_RELEASE.md`
  - relevant ADRs, especially ADR 0038, 0044, 0045, 0052, and 0053
- Candidate lane areas:
  - playback/decoding/transcode/resource admission,
  - remote access/network tunnel/endpoint discovery,
  - Addon Protocol and Addon Sidecar boundaries,
  - Admin/API/Admin Web/generated contract governance,
  - storage/VFS/library/control-plane source identity and durable jobs.

## Research References

- [`research/playback-decoding-transcode.md`](research/playback-decoding-transcode.md)
  recommends a playback output profile/device capability audit before HEVC/AV1,
  hardware tone-map, or subtitle execution follow-ons.
- [`research/remote-access-network-operations.md`](research/remote-access-network-operations.md)
  recommends remote access cookbook/config-check fixture work before endpoint
  discovery or tunnel runtime.
- [`research/addon-boundary-automation.md`](research/addon-boundary-automation.md)
  recommends a host-owned Addon resource flow pattern and task/event execution
  policy audit before Addon Manager implementation.
- [`research/admin-api-web-contracts.md`](research/admin-api-web-contracts.md)
  recommends an Admin route inventory parity gate as the first contract-safety
  task.
- [`research/storage-library-control-plane.md`](research/storage-library-control-plane.md)
  recommends source fingerprint hash triggering/reconciliation policy before
  automatic source duplicate behavior.
- [`research/synthesis.md`](research/synthesis.md) ranks the cross-lane queue
  and parallelization risks.
- [`research/next-parallel-contract-gates.md`](research/next-parallel-contract-gates.md)
  records shared contract gates that must serialize the next parallel wave.
- [`research/next-product-development-lanes.md`](research/next-product-development-lanes.md)
  re-ranks product-facing lanes after the first two child tasks completed.
- [`research/next-architecture-refactor-lanes.md`](research/next-architecture-refactor-lanes.md)
  audits whether more fearless refactor should precede feature work.
- [`research/next-operations-release-lanes.md`](research/next-operations-release-lanes.md)
  confirms the remote access cookbook/config gate task is the highest-value
  operator-facing slice.
- [`research/next-lane-synthesis.md`](research/next-lane-synthesis.md)
  consolidates the post-child-task recommendation.

## Decision (ADR-lite)

**Context**: Nako has several mature first-slice lanes, and future work will be
parallelized. The risk is no longer simply "which feature next"; it is avoiding
shared Admin/API/runtime contract collisions and avoiding premature broad
refactor.

**Decision**: Use a mixed plan. Start with an Admin route inventory parity gate
as the contract-safety lane. In parallel, run architecture audits for playback
output/device capability, Addon host-owned resource flows, and source
fingerprint hash triggering/reconciliation. Run low-conflict operations work
for remote access cookbook/config-check fixtures. Defer broad fearless refactor
until a selected lane creates concrete file-local pressure.

**Consequences**: Parallel agents get safer ownership boundaries before editing
shared DTO/runtime surfaces. Some feature implementation is delayed by short
architecture audits, but the delay is cheaper than conflicting generated
contract, playback capability, Addon resource-flow, or source identity changes.

## Recommended Next Queue

1. `06-05-admin-route-inventory-parity-gate` — ready bounded implementation.
2. `06-05-playback-output-profile-device-capability-audit` — architecture
   audit.
3. `06-05-remote-access-cookbook-config-gates` — ready low-conflict docs/ops
   implementation.
4. `06-05-addon-resource-flow-pattern-audit` — architecture audit.
5. `06-05-source-hash-triggering-reconciliation-policy` — architecture audit.

## Post-Child-Audit Update (ADR-lite)

**Context**: The Admin route inventory parity gate and the playback
output/profile device capability audit are now complete. Follow-up research in
the parent task reviewed product lanes, architecture/refactor lanes, contract
gates, and operations/release readiness.

**Decision**: Keep the mixed plan, but update the queue. Do not start a broad
fearless refactor campaign. Treat Admin route parity as a standing gate and
make `public-client-playback-capability-contract-parity-gate` the next
serial-first contract task. In parallel, run
`06-05-remote-access-cookbook-config-gates` as the operator-visible docs/ops
implementation lane, and continue
`06-05-addon-resource-flow-pattern-audit` plus
`06-05-source-hash-triggering-reconciliation-policy` as architecture/research
lanes.

**Consequences**: Playback profile-v2, HEVC/AV1 execution, hardware tone-map
execution, image subtitle burn-in, Public Client endpoint discovery, built-in
tunnel runtime, automatic duplicate mutation, and Addon Manager process
lifecycle remain deferred until their shared contracts are owned. Generated
Admin contracts, Public Client playback DTOs/SDKs, shared identity/config,
Addon manifest/protocol, and durable-job/runtime policy must not be changed as
side effects of parallel feature lanes.

## Current Recommended Next Queue

1. `public-client-playback-capability-contract-parity-gate` — created
   serial-first shared contract task to start.
2. `06-05-remote-access-cookbook-config-gates` — low-conflict docs/ops
   implementation, safe to run in parallel if it avoids config-shape,
   Admin DTO, endpoint discovery, and tunnel-runtime changes.
3. `06-05-addon-resource-flow-pattern-audit` — architecture audit, safe as
   docs/research while avoiding Admin DTO and Addon Protocol wire-shape edits.
4. `06-05-source-hash-triggering-reconciliation-policy` — architecture audit,
   safe as docs/research while avoiding durable job scheduler, schema,
   source identity, and Admin DTO edits.
5. Admin contract generator deepening — defer unless the next wave is
   Admin-heavy enough to justify a serial refactor owner.

## Created Subtasks

- `.trellis/tasks/06-05-admin-route-inventory-parity-gate/`
- `.trellis/tasks/06-05-playback-output-profile-device-capability-audit/`
- `.trellis/tasks/06-05-remote-access-cookbook-config-gates/`
- `.trellis/tasks/06-05-addon-resource-flow-pattern-audit/`
- `.trellis/tasks/06-05-source-hash-triggering-reconciliation-policy/`
- `.trellis/tasks/06-05-public-client-playback-capability-contract-parity-gate/`

The parent task remains the cross-lane audit record. Start child tasks
individually when assigning implementation or architecture-audit lanes.
