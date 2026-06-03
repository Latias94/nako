# Source Fingerprint Escalation Policy Evidence

Date: 2026-06-04

## Scope

- Added a typed `nako-core` source fingerprint escalation decision.
- Exposed the decision on `nako-library` source observation persistence plans.
- Kept the slice advisory only: no hash execution, repository/schema change,
  Admin/Public API exposure, or weak-evidence source merge.

## Verification

- `cargo fmt --all -- --check`: passed.
- `cargo check -p nako-core -p nako-library --tests`: passed.
- `cargo nextest run -p nako-core source_fingerprint_escalation --no-fail-fast`:
  passed, 3 tests.
- `cargo nextest run -p nako-library source_observation_plan_recommends --no-fail-fast`:
  passed, 2 tests.
- `git diff --check`: passed with LF/CRLF normalization warnings only.
- `python .\.trellis\scripts\task.py validate 06-04-06-04-source-fingerprint-escalation-policy-first-slice`:
  passed.

## Spec And Architecture Sync

- `.trellis/spec/nako-core/backend/quality-guidelines.md` now records the pure,
  redaction-safe core policy boundary.
- `.trellis/spec/nako-library/backend/quality-guidelines.md` now records the
  source observation plan contract and forbidden behavior.
- `docs/architecture/STORAGE_VFS.md` now treats hash execution as a follow-on
  after the advisory policy seam.
- `docs/architecture/LIBRARY_PIPELINE.md` now lists source fingerprint
  escalation decisions in the source identity foundation.
