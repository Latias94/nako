# Public Client Source Locator Redaction Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The workstream is open from ARF-005. No code changes have started.

Known public exposure anchors include `MediaSourceDto.locator` and
`ClientTranscodePlan.input_locator` in protocol/OpenAPI/API mapping paths.

## Active Task

- Task ID: PCLR-020
- Owner: unassigned
- Files: `crates/taru-client-protocol`, `crates/taru-api`,
  `crates/taru-server/src/http`, `docs/api`
- Validation: `rg "locator|input_locator" crates/taru-client-protocol
  crates/taru-api crates/taru-server/src/http`; `git diff --check`
- Status: READY
- Review: contract review required before DTO field removal
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep Media Library reconciliation in `multi-library-hardening`.
- Keep internal `MediaSource.locator` unchanged.
- Start with an exposure audit and contract decision before changing public
  wire shapes.

## Blockers

- Compatibility posture must be explicit before removing public fields.

## Next Recommended Action

Run PCLR-020. Classify locator exposures into Public Client, Admin API,
internal execution, and test fixtures, then choose the public replacement or
redaction policy.
