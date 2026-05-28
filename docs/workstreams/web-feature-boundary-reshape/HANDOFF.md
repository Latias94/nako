# Web Feature Boundary Reshape - Handoff

Status: Active
Last updated: 2026-05-28

## Current State

`web-test-harness-and-route-contracts` is complete. Route and data-source
contract tests are in place, so large component boundary moves can start.

## Active Task

- Task ID: WFBR-040
- Owner: Codex
- Status: READY
- Validation: `npm --prefix web run test && npm --prefix web run build`

## Next Recommended Action

- Move setup/account/notifications/settings/TV route surfaces out of `components/nako`.
