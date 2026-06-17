# Playback Reason Public Contract

## Goal

Make Public Client playback decisions explainable with a stable,
redaction-safe reason summary for the selected Direct Play, Remux, HLS
Transcode, or Denied outcome. This is the next backend maturity slice after
named playback profile selection: clients can now select a profile, and they
also need to understand why the server selected or rejected a playback mode.

## Requirements

- Add a Public Client wire contract for playback decision reasons that is safe
  for browser, Android, CLI, and future TV clients.
- Keep the first slice focused on the selected decision summary. Do not expose
  FFmpeg command lines, host paths, source locators, raw probe payloads,
  hardware device paths, resource pressure, or operator-only policy internals.
- Preserve existing playback decision semantics. Reason DTOs explain the
  planner result; they must not change Direct Play, Remux, HLS, session,
  staging, or transcode runtime behavior.
- Represent the reason as an additive vocabulary with future-safe string
  handling in `nako-client-protocol`.
- Map at least the current high-value outcomes:
  - Direct Play selected because the source matches client capabilities.
  - Remux selected because the container must change.
  - HLS Transcode selected because one or more client capabilities are missing
    or incompatible.
  - Playback denied because no playable plan is available.
- Include compact safe requirement facts when available, such as container,
  video codec, audio codec, subtitle mode, HDR/tone-map, or policy category,
  without exposing raw source identity.
- Update `nako-api` OpenAPI/SDK generation, generated TypeScript and Kotlin
  SDK artifacts, HTTP docs, and server route mapping in the same task.
- Add focused route/contract tests proving the reason shape is present and
  redaction-safe.

## Acceptance Criteria

- [ ] `PlaybackDecisionResponse` includes a stable optional or required reason
      summary in protocol/API/generated SDK outputs.
- [ ] Reason vocabulary tolerates future server reason strings without breaking
      older generated/client protocol consumers.
- [ ] Server playback decision route maps current planner output into a safe
      reason summary for Direct Play, Remux, HLS, and Denied cases.
- [ ] Existing playback mode selection behavior remains unchanged.
- [ ] Tests reject source locators, paths, FFmpeg command fragments, tokens,
      and raw probe/runtime facts in the public decision reason response.
- [ ] HTTP API docs describe the reason field and its redaction boundary.
- [ ] Trellis specs are updated if this task establishes new public playback
      reason vocabulary rules.

## Definition of Done

- Focused `nako-client-protocol`, `nako-api`, and `nako-server` playback tests
  pass.
- Generated TypeScript and Kotlin SDK files are refreshed from `nako-api` if
  the public DTO changes.
- `cargo fmt --all -- --check` and `git diff --check` pass for touched files.
- No unrelated dirty files are staged.

## Technical Approach

- Inspect the existing playback decision DTOs and planner output before
  choosing the DTO shape.
- Prefer a compact `PlaybackDecisionReasonDto` with:
  - a future-safe `code`;
  - a safe human-readable or client-displayable `message`;
  - optional safe categories/facts if current planner data supports them
    cleanly.
- Keep richer per-candidate diagnostics, all failed candidate matrices, device
  profile databases, and Admin-only runtime support evidence out of this first
  public slice.
- If the planner already has typed reason-like fields, map them instead of
  inventing a parallel explanation model. If not, add the smallest pure helper
  needed to summarize the existing selected plan.

## Decision (ADR-lite)

**Context**: Named playback profiles now influence playback startup, but
clients still receive a selected mode without a stable explanation. Mature
self-hosted media servers make playback support debuggable by telling users why
Direct Play, Remux, or Transcode was needed.

**Decision**: Start with a selected-decision reason summary in the Public
Client playback decision response, not a full compatibility matrix. Make the
vocabulary additive and redaction-safe.

**Consequences**: Clients can immediately display actionable playback reasons.
The API remains small enough to stabilize now, while richer per-stream/device
diagnostics can land later without breaking this contract.

## Out of Scope

- Full per-candidate compatibility matrix.
- Device profile database or browser/TV profile presets beyond the existing
  capability profile work.
- Admin playback runtime diagnostics.
- FFmpeg command planning or runtime behavior changes.
- Playback policy CRUD or access-policy enforcement changes.
- HLS segment/session/ticket route changes.

## Technical Notes

- Strategic plan: `docs/plans/2026-06-16-001-feat-backend-self-hosted-maturity-execution-plan.md`
  U3.
- Architecture: `docs/architecture/PLAYBACK.md`.
- Protocol/API specs:
  `.trellis/spec/nako-client-protocol/backend/index.md`,
  `.trellis/spec/nako-api/backend/admin-and-public-contracts.md`.
- Server specs:
  `.trellis/spec/nako-server/backend/http-api-patterns.md`,
  `.trellis/spec/nako-server/backend/error-handling.md`,
  `.trellis/spec/nako-server/backend/quality-guidelines.md`.
- Playback/transcode specs:
  `.trellis/spec/nako-playback/backend/index.md`,
  `.trellis/spec/nako-transcode/backend/index.md`.
