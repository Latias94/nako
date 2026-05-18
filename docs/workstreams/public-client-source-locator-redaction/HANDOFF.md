# Public Client Source Locator Redaction Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The workstream is open from ARF-005. PCLR-020 has completed the exposure audit
and public contract decision.

Public Client locator leakage is real and concentrated in
`MediaSourceDto.locator`, `ClientTranscodePlan.input_locator`,
`taru-api::public_client` mapping, public OpenAPI schemas, and generated
TypeScript SDK types.

## Active Task

- Task ID: PCLR-030
- Owner: codex
- Files: `crates/taru-client-protocol`, `crates/taru-api`,
  `crates/taru-server/src/http/tests`
- Validation: `cargo check -p taru-client-protocol --tests`; `cargo check -p
  taru-api --tests`; focused public route tests
- Status: READY
- Review: public contract and leakage review required
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep Media Library reconciliation in `multi-library-hardening`.
- Keep internal `MediaSource.locator` unchanged.
- Public Client DTOs must not expose raw Source Locator values.
- `MediaSourceDto` should keep stable IDs and safe display facts such as
  `file_name`, `size_bytes`, and `fingerprint`; remove `locator`.
- `ClientTranscodePlan` should remove `input_locator`. Public clients already
  address playback through `source_id`, stream/remux/HLS routes, and playback
  session IDs.
- Internal execution paths, including direct stream, remux, HLS, and probe
  staging, must continue using full locators inside server/app/storage crates.
- Admin-only diagnostics may introduce redacted locator summaries later, but
  that is not part of the Public Client DTO contract.

## Blockers

- None known. Compatibility posture: Taru has no stable external public
  locator contract yet, so PCLR-030 may remove these fields rather than
  deprecate them first.

## Next Recommended Action

Run PCLR-030. Remove raw public locator fields from protocol DTOs, API mapping,
and route JSON tests while preserving internal locator use.
