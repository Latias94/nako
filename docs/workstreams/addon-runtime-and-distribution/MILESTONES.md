# Addon Runtime And Distribution — Milestones

Status: Active
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Status: completed on 2026-05-22.

Exit criteria:

- [x] Workstream docs exist and agree.
- [x] Scope is Addon Sidecar package/install/runtime readiness first.
- [x] Addon Manager automation, package signing, marketplace, process
  supervision, Native Plugin ABI, direct filesystem writes, and Public Client
  API changes are out of scope.
- [x] Parent `post-rpd-product-hardening` points at this lane.

Primary evidence:

- `docs/workstreams/addon-runtime-and-distribution/DESIGN.md`
- `docs/workstreams/addon-runtime-and-distribution/TODO.md`

## M1 — Package And Install Guide Boundary

Exit criteria:

- Addon package/install descriptor vocabulary exists in a safe protocol/admin
  boundary.
- Install-guide previews are redacted and do not include admin tokens, resolved
  secrets, local paths, or credential-bearing URLs.
- Protocol crates remain permissive and free of server internals.

Primary evidence:

- `crates/taru-addon-protocol`
- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/app`

## M2 — Runtime Readiness And Sidecar Compatibility

Exit criteria:

- Admin diagnostics classify sidecar reachability, protocol mismatch, manifest
  mismatch, grants/config gaps, network blockers, and unsafe responses.
- Diagnostics are bounded and redacted.
- No automatic process/container supervision is introduced.

Primary evidence:

- `crates/taru-addon-client`
- `crates/taru-server/src/http/admin.rs`
- `apps/admin-web/src/adminApi`

## M3 — Declared Task/Event Routing Without Hidden Schedulers

Exit criteria:

- Manifest-declared Addon Tasks and Event Subscriptions produce explicit routing
  plans.
- Executable plans reuse existing job/outbox/addon side-effect boundaries.
- Deferred runtime behavior is blocked with typed reasons rather than hidden
  schedulers.

Primary evidence:

- `crates/taru-core/src/addon.rs`
- `crates/taru-server/src/app`
- `crates/taru-db`

## M4 — Addon Artifact And Intake Handoff

Exit criteria:

- Addon-produced Generated Artifacts enter AILO proposal/review semantics.
- Addon-produced acquisition candidates enter DWI intake semantics.
- Tests prove no autonomous Canonical Metadata, NFO sidecar, Media Source,
  Managed Import, or library-file writes.

Primary evidence:

- `crates/taru-server/src/app`
- `crates/taru-core/src/automation.rs`
- `crates/taru-core/src/acquisition_intake.rs`

## M5 — Closeout And Follow-On Split

Exit criteria:

- Final gates pass with fresh evidence.
- Workstream status and completed tasks are updated.
- Parent post-RPD umbrella re-scores Addon Manager, package signing,
  marketplace, process supervision, downloader protocols, Public Client
  surfaces, and local AI/model runtime follow-ons.
- Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/addon-runtime-and-distribution/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
