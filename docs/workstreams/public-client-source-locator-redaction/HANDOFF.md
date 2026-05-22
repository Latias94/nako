# Public Client Source Locator Redaction Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

The workstream is closed. PCLR-020, PCLR-030, PCLR-040, and PCLR-050 are
complete. Public Client DTOs, OpenAPI schema, generated TypeScript SDK output,
route tests, and HTTP API docs all reflect the redacted Source Locator
contract.

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
- OpenAPI and TypeScript SDK sync landed with PCLR-030 because `nako-api`
  enforces checked-in SDK consistency.
- No follow-on was split for the public locator redaction lane.

## Blockers

- None known.

## Follow-Ons

- Open a separate Admin API diagnostics lane only if redacted locator
  summaries are actually required later.
