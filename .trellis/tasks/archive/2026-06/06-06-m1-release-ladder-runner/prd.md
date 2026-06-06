# M1 Release Ladder Runner

## Goal

Add one Product-Operator M1 release ladder entry point that composes existing
release and smoke gates by product meaning. The runner should make it obvious
which checks are fast, which are expensive, and which release dimensions remain
optional follow-ons, without duplicating the detailed test logic already owned
by existing scripts.

## What I Already Know

- `scripts/release-gate.ps1` already supports `docs`, `fast`, `db`, `api`,
  `playback`, `postgres`, `container`, `workspace`, and `all`.
- `scripts/m1-operator-journey-smoke.ps1` already composes docs-safe,
  self-host server, and Admin Web route/media smoke coverage for the M1
  operator journey.
- `m1-operator-journey-smoke`, scan-originated source hash triggering, and
  source duplicate operator flow are archived as completed M1 convergence
  slices.
- The refreshed M1 queue names `m1-release-ladder-runner` as the next
  executable task.

## Requirements

- Add a runner script that exposes Product-Operator M1 validation modes:
  - `docs`;
  - `smoke`;
  - `fast`;
  - `release-fast`;
  - `playback`;
  - `container`;
  - `postgres`;
  - `workspace`;
  - `all`.
- Default `fast` mode must stay locally repeatable: docs-safe release hygiene
  plus the focused M1 operator journey smoke.
- Do not inline or duplicate release-gate, self-host smoke, Admin Web test, or
  playback test logic. Delegate to existing scripts.
- Make expensive gates explicit: `release-fast`, `playback`, `container`,
  `postgres`, `workspace`, and `all`.
- Keep secrets redacted in output. Do not print PostgreSQL URLs, bearer tokens,
  local media paths, source locators, playback tickets, content hashes, or
  source fingerprints.
- Document the new runner in release/operator docs and operations architecture.
- Update Trellis/spec guidance for the new script contract.

## Acceptance Criteria

- [x] `scripts/m1-release-ladder.ps1` exists and defaults to a fast,
      docs-plus-M1-smoke ladder.
- [x] Runner modes delegate to existing scripts instead of copying gate logic.
- [x] `all` mode sequences the explicit release dimensions and avoids repeating
      redaction inventory after the first docs gate.
- [x] Deployment docs explain when to run default `fast` versus expensive
      modes.
- [x] Operations architecture records the M1 ladder runner as shipped evidence
      and leaves live-browser release proof as a follow-on.
- [x] Trellis/spec records the runner contract and required validation.
- [x] PowerShell parser validation passes.
- [x] Focused docs/fast runner validation passes or any environment blocker is
      recorded.
- [x] Trellis context validation and `git diff --check` pass.

## Definition Of Done

- Runner, docs, spec, and task context are updated.
- Focused validation evidence is recorded.
- Work is committed and pushed.
- Trellis task is archived.

## Technical Approach

Implement the runner as a thin PowerShell orchestrator because the current M1
operator journey smoke entry point is PowerShell-first. Keep existing
cross-platform release-gate scripts as the detailed gate owners and avoid
duplicating their command lists.

## Decision

Use a product-level runner rather than extending `release-gate.ps1` directly.

Consequences:

- `release-gate.ps1` remains the reusable technical gate surface.
- M1 can name a single Product-Operator release ladder without forcing every
  release-gate mode into the default local workflow.
- Future live-browser or package-publication proof can be added as explicit
  modes without changing the existing release-gate semantics.

## Out Of Scope

- No release artifact publication.
- No live browser automation in this slice.
- No new Rust, TypeScript, schema, API, generated contract, or runtime
  behavior.
- No changes to the detailed command list inside `release-gate.ps1` unless a
  current contract bug is found.
- No Bash wrapper unless the PowerShell runner becomes insufficient for CI or
  operator docs.

## Technical Notes

- Primary sources:
  - `scripts/release-gate.ps1`
  - `scripts/m1-operator-journey-smoke.ps1`
  - `.trellis/tasks/archive/2026-06/06-06-m1-operator-journey-smoke/evidence.md`
  - `.trellis/tasks/archive/2026-06/06-06-m1-roadmap-queue-refresh-after-source-duplicate-flow/prd.md`
  - `docs/ROADMAP.md`
  - `docs/architecture/LANES.md`
  - `docs/architecture/OPERATIONS_RELEASE.md`
  - `docs/deployment/RELEASE_CHECKLIST.md`

## Verification Evidence

- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-06-m1-release-ladder-runner`
  passed with 5 implement context entries and 5 check context entries.
- PowerShell parser validation for `scripts/m1-release-ladder.ps1` passed.
- `git diff --check -- .trellis/tasks/06-06-m1-release-ladder-runner scripts/m1-release-ladder.ps1 docs/deployment/RELEASE_CHECKLIST.md docs/architecture/OPERATIONS_RELEASE.md .trellis/spec/nako-server/backend/quality-guidelines.md`
  passed. Git reported LF-to-CRLF working-copy warnings for existing markdown
  files, but no whitespace errors.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode docs -SkipRedactionInventory`
  passed. It delegated to `scripts/release-gate.ps1 -Mode docs
  -SkipRedactionInventory`, and nested `cargo fmt --all -- --check` plus
  `git diff --check` passed.
- `pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/m1-release-ladder.ps1 -Mode fast -SkipRedactionInventory`
  passed. It ran the docs-safe gate once, then delegated to
  `scripts/m1-operator-journey-smoke.ps1 -Mode fast -SkipDocsGate`; nested
  `cargo nextest run -p nako-server self_host_smoke --no-fail-fast` passed 1
  test with 654 skipped, and nested Admin Web Vitest passed 2 files / 116
  tests.
