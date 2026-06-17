# Playback Capability Reason Projection

## Goal

Make playback decisions more product-readable for Client Applications by adding
a stable, redaction-safe reason detail projection to the existing Public Client
playback decision contract. Clients should not need to hard-code their own
explanation table for every `ClientPlaybackCompatibilityCondition`.

## Requirements

- Extend the existing `GET /sources/{source_id}/playback/decision` response.
- Keep the existing `mode`, `reason`, `report.selection_reasons`,
  `report.direct_play.reasons`, `report.remux.reasons`, and
  `report.transcode.reasons` fields unchanged.
- Add a protocol-owned reason detail DTO for every known
  `ClientPlaybackCompatibilityCondition`.
- Include reason details for:
  - selected-mode reasons;
  - Direct Play capability reasons;
  - Remux capability reasons;
  - Transcode capability reasons.
- Each detail must include only stable public facts:
  `condition`, short `summary`, and operator/client-safe `detail`.
- Keep summaries/details as English baseline strings for Public Client v1. Do
  not introduce localization, frontend copy, or user-customized text in this
  backend slice.
- Preserve additive compatibility for future condition strings. Unknown/future
  conditions must still deserialize through the existing `Other(String)` enum
  path and should map to a generic safe explanation when projected.
- Do not add a new route, query parameter, database field, Admin API mutation,
  device profile database, persisted preference, or playback planner behavior
  change.
- Do not expose Source Locators, local paths, bearer tokens, FFmpeg commands,
  stderr, transcode output paths, raw probe payloads, raw policy internals, or
  backend errors.

## Acceptance Criteria

- [ ] `PlaybackDecisionResponse.decision.report` includes reason detail arrays
  alongside the existing reason code arrays.
- [ ] `video_codec_unsupported` route responses include a safe detail explaining
  that the selected source video codec is not in the client capability profile.
- [ ] `policy_denied` responses include a safe detail explaining that the
  effective playback policy blocked the mode.
- [ ] Protocol serde tests prove every known
  `ClientPlaybackCompatibilityCondition` has a detail projection.
- [ ] Unknown/future condition strings round-trip and project to a generic safe
  detail.
- [ ] OpenAPI/SDK contract tests cover the new DTO fields without hand-editing
  generated outputs.
- [ ] Focused playback protocol/API/server tests pass.

## Definition Of Done

- `cargo fmt --all`
- `cargo nextest run -p nako-client-protocol playback --no-fail-fast`
- `cargo nextest run -p nako-api playback --no-fail-fast`
- `cargo nextest run -p nako-server playback_decision --no-fail-fast`
- Trellis context validates.
- Task is committed and archived without staging unrelated user changes.

## Technical Approach

- Add `ClientPlaybackCompatibilityConditionDetail` in
  `nako-client-protocol`.
- Add helper projection functions in the protocol crate so the public contract
  owns condition-to-copy mapping, not every client.
- Add `selection_reason_details`, `reason_details` fields to the relevant
  report/evaluation DTOs with serde defaults for older responses.
- Map details in `nako-api::public_client` while preserving the existing code
  arrays.
- Update OpenAPI schema generation for the new DTO shape.
- Add focused protocol, API, and server route tests.

## Decision (ADR-lite)

**Context**: M3 playback/transcode maturity needs clients to explain why Nako
selected Direct Play, Remux, Transcode, or Denied. The planner already emits
stable condition codes, but every client would otherwise need to duplicate
reason copy and fallback behavior.

**Decision**: Add a small protocol-owned reason detail projection to the
existing playback decision response. Keep planner behavior and route shape
unchanged.

**Consequences**: Public Client v1 gains additive fields and a reusable reason
catalog. The first slice uses English baseline copy only; localization and
full device profile matrices remain separate product/client work.

## Out Of Scope

- Full device profile database.
- New playback route or Admin API surface.
- Frontend rendering changes.
- Generated SDK package refresh unless required by existing generator tests.
- Planner decision behavior changes.
- Transcode/FFmpeg execution changes.
- Localization.

## Technical Notes

- Roadmap: `docs/ROADMAP.md` M3 and parity matrix both call out device
  capability profiles and playback compatibility reasons as must-build before
  beta.
- Architecture: `docs/architecture/PLAYBACK.md` Lane A says planner should emit
  precise compatibility reasons and keep public DTOs redaction-safe.
- Existing planner: `nako-playback` already emits `PlaybackDecisionReport` and
  `PlaybackCompatibilityCondition` values.
- Existing protocol: `nako-client-protocol` uses additive
  `public_string_value!` enums for future-safe condition strings.
- Existing route tests already assert safe `selection_reasons` and redaction
  for playback decision responses.
