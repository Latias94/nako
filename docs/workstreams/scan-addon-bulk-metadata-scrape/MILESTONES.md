# Scan Addon Bulk Metadata Scrape — Milestones

Status: Complete
Last updated: 2026-05-25

## M0 — Scope Locked

Exit criteria:

- Workstream docs exist and agree on scope.
- The lane is explicitly separate from Addon Event scheduler/replay.
- Follow-ons are named rather than hidden in implementation.

## M1 — Policy And Public Contract

Exit criteria:

- `MetadataScanPolicy` includes `addon_scrape`.
- Defaults keep existing libraries from auto-triggering Addon scrape.
- Config and DTO/OpenAPI surfaces expose the policy.

## M2 — Automatic TaskRun Creation

Exit criteria:

- Scan metadata acquisition uses the plan's `addon_scrape` flag.
- Eligible Addons are enabled, declare `bulk-metadata-scrape`, and have executable task routing plans.
- Task payloads are bounded and contain query facts only.
- Scan summary records created/skipped Addon scrape work.

## M3 — Verification

Exit criteria:

- Focused Rust tests pass.
- Formatting/checks are run on touched Rust code.
- Evidence and handoff docs capture shipped behavior and residual risks.
