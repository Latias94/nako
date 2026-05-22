# Addon Runtime And Distribution — Milestones

Status: Complete
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

Status: completed on 2026-05-22.

Exit criteria:

- [x] Addon package/install descriptor vocabulary exists in a safe protocol/admin
  boundary.
- [x] Install-guide previews are redacted and do not include admin tokens, resolved
  secrets, local paths, or credential-bearing URLs.
- [x] Protocol crates remain permissive and free of server internals.

Primary evidence:

- `crates/nako-addon-protocol`
- `crates/nako-api/src/extension.rs`
- `crates/nako-server/src/app`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`

## M2 — Runtime Readiness And Sidecar Compatibility

Status: completed on 2026-05-22.

Exit criteria:

- [x] Admin diagnostics classify sidecar reachability, protocol mismatch, manifest
  mismatch, grants/config gaps, network blockers, and unsafe responses.
- [x] Diagnostics are bounded and redacted.
- [x] No automatic process/container supervision is introduced.

Primary evidence:

- `crates/nako-api/src/extension.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `apps/admin-web/src/adminApi`
- `docs/workstreams/addon-runtime-and-distribution/EVIDENCE_AND_GATES.md`

## M3 — Declared Task/Event Routing Without Hidden Schedulers

Status: completed on 2026-05-22.

Exit criteria:

- [x] Manifest-declared Addon Tasks and Event Subscriptions produce explicit
  routing plans.
- [x] Executable plans reuse existing job/outbox/addon side-effect boundaries.
- [x] Deferred runtime behavior is blocked with typed reasons rather than
  hidden schedulers.

Primary evidence:

- `crates/nako-core/src/addon.rs`
- `crates/nako-server/src/app`
- `crates/nako-db`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `crates/nako-api/src/extension.rs`
- `apps/admin-web/src/adminApi`

## M4 — Addon Artifact And Intake Handoff

Status: completed on 2026-05-22.

Exit criteria:

- [x] Addon-produced Generated Artifacts enter AILO proposal/review semantics.
- [x] Addon-produced acquisition candidates enter DWI intake semantics.
- [x] Tests prove no autonomous Canonical Metadata, NFO sidecar, Media Source,
  Managed Import, or library-file writes.

Primary evidence:

- `crates/nako-server/src/app`
- `crates/nako-core/src/automation.rs`
- `crates/nako-core/src/acquisition_intake.rs`
- `crates/nako-api/src/extension.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-server/src/http/tests/addons.rs`
- `docs/workstreams/addon-runtime-and-distribution/EVIDENCE_AND_GATES.md`

## M5 — Closeout And Follow-On Split

Status: completed on 2026-05-22.

Exit criteria:

- [x] Final gates pass with fresh evidence.
- [x] Workstream status and completed tasks are updated.
- [x] Parent post-RPD umbrella re-scores Addon Manager, package signing,
  marketplace, process supervision, downloader protocols, Public Client
  surfaces, and local AI/model runtime follow-ons.
- [x] Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/addon-runtime-and-distribution/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
