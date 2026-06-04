# Addon Install Health Guide

## Goal

Make addon sidecar onboarding more practical by improving install guide and
health-readiness behavior while preserving the out-of-process Addon Protocol
boundary.

## Requirements

- Audit current Addon Protocol manifest, install descriptor, addon client health
  checks, official addon catalog descriptors, and any Admin/API surfaces before
  selecting the exact slice.
- Prefer generated install guidance and health readiness facts over Nako-managed
  process or Docker lifecycle control.
- Use Nako terms: Addon, Addon Sidecar, Addon Package, Addon Install Guide,
  Addon Health Check, Addon Token, Addon Protocol Version, and Secret Reference.
- Keep addon tokens, endpoint credentials, local runtime paths, and secret
  values redacted in install guides, health output, logs, and tests.
- Keep Addon Protocol changes additive unless a versioned contract decision is
  explicitly approved.

## Acceptance Criteria

- [x] The selected addon onboarding slice is documented with the reason it is
  the smallest useful step.
- [x] Install guide or health-readiness output is typed, redaction-safe, and
  testable.
- [x] Addon Protocol compatibility is preserved or version impact is explicitly
  documented.
- [x] Official addon catalog/client behavior remains compatible.
- [x] Follow-ons are recorded for Addon Manager, hosted settings, token
  rotation, official provider breadth, or cross-repo addon implementation.

## Definition of Done

- Focused addon protocol/client/catalog/API tests pass for changed behavior.
- Generated contracts and Web/Admin checks pass if API/UI surfaces change.
- `cargo fmt --all -- --check` passes.
- Evidence notes record selected slice, validation, and deferred follow-ons.

## Out of Scope

- No Docker socket control.
- No process supervision or addon lifecycle manager.
- No OAuth flow.
- No cross-repo `nako-official-addons` implementation unless planner approves
  related-repo work.
- No native in-process plugin ABI.

## Technical Notes

- Likely files: `crates/nako-addon-protocol`, `crates/nako-addon-client`,
  `crates/nako-official-addon-catalog`, `crates/nako-api`, and
  `crates/nako-server` addon/control-plane routes if surfaced.
- Coordinate with control-plane and operations-release if install guidance
  changes deployment docs.
