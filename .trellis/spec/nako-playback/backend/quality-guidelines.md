# Quality Guidelines

Playback planner changes must remain deterministic and side-effect free.

## Required Patterns

- Prefer Direct Play when source and target are compatible. Transcode is a
  fallback or explicit request, not the default media path.
- Evaluate selected subtitle delivery before choosing Remux. A container
  fallback must not bypass a subtitle requirement that only HLS transcode can
  satisfy, such as ASS/SSA burn-in.
- Keep profile identity stable and include every request fact that changes the
  planning result.
- Model track selection, audio output, HDR/color pipeline, subtitle strategy,
  and HLS output as typed values.
- Treat unknown subtitle codec facts as an explicit policy choice. If unknown
  codecs preserve a legacy sidecar path, cover that behavior with a named test
  and document it rather than relying on implicit `None` handling.
- Treat sidecar-capable subtitle codec facts as a mixed vocabulary. Probe facts
  may carry FFmpeg codec names such as `webvtt` or sidecar extension aliases
  such as `vtt`; both must map to sidecar delivery when the client supports
  subtitles.
- Keep `PlaybackDecisionReport` useful even when playback is denied.
- Keep storage facts abstract: remote/range-readable are planning inputs, not
  backend calls.

## Forbidden Patterns

- Do not add process execution, filesystem staging, HTTP serving, or database
  writes to this crate.
- Do not make Source Variant Labels decide compatibility; use Media Technical
  Facts, client capabilities, and policy.
- Do not assume all clients support HLS TS/fMP4, AAC, H264, HDR, subtitles, or
  range requests.
- Do not hide policy denial by selecting a fallback mode the policy disallows.
- Do not select Remux before checking whether the selected subtitle track
  requires burn-in or another transcode-only delivery strategy.

## Tests Required

- Unit tests for Direct Play/Remux/Transcode/Denied selection.
- Regression tests for container-unsupported/remux-supported sources with
  selected subtitle tracks that require burn-in.
- Tests that name the chosen behavior for missing or blank subtitle codec facts.
- Tests for sidecar subtitle codec aliases when changing HLS sidecar-versus-
  burn-in classification, including both codec-name and file-extension forms
  such as `webvtt` and `vtt`.
- Tests for profile identity changes when request facts change.
- Tests for audio downmix/normalization and HDR/color pipeline requirements
  when those values change.
- Server integration tests when resource admission or HTTP route behavior
  changes.

## Gate Selection

- Focused planner:
  `cargo nextest run -p nako-playback <filter> --no-fail-fast`
- Playback/server cross-crate:
  `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`

## Scenario: Playback Decision Selection Reasons

### 1. Scope / Trigger

- Trigger: changing a playback decision report, Public Client playback DTO, or
  SDK/OpenAPI playback decision shape.
- Scope: `PlaybackDecisionReport.selection_reasons` is the selected-mode reason
  summary produced by `nako-playback` and mapped through `nako-api`,
  `nako-client-protocol`, OpenAPI, TypeScript SDK, Kotlin SDK, and server route
  tests. Public Client v1 also exposes protocol-owned reason detail projections
  alongside the reason codes.

### 2. Signatures

- Planner: `PlaybackDecisionReport { selected_mode, selection_reasons,
  direct_play, remux, transcode, denial }`.
- Public protocol: `ClientPlaybackDecisionReport { selected_mode,
  selection_reasons, selection_reason_details, direct_play, remux, transcode,
  denial }`.
- Public protocol: `ClientPlaybackCapabilityEvaluation { supported, reasons,
  reason_details }`.
- Public protocol:
  `ClientPlaybackCompatibilityConditionDetail { condition, summary, detail }`.
- Wire field: `selection_reasons: ClientPlaybackCompatibilityCondition[]`.
- Wire field:
  `selection_reason_details: ClientPlaybackCompatibilityConditionDetail[]`.
- Wire field:
  `reason_details: ClientPlaybackCompatibilityConditionDetail[]`.

### 3. Contracts

- `selection_reasons` explains why the selected mode was chosen; it is not a
  dump of every failed mode.
- `selection_reason_details` must be derived from `selection_reasons`.
- Each capability `reason_details` must be derived from that capability's
  `reasons`.
- Reason-detail copy belongs to `nako-client-protocol`, not server handlers,
  frontend code, or generated SDK artifacts.
- Unknown/future compatibility condition strings may round-trip through the
  existing additive enum path, but their `summary`/`detail` copy must stay
  generic and must not echo raw backend diagnostics.
- Direct Play uses its direct-play compatibility reasons.
- Remux includes the decision reason and non-compatible Direct Play reasons
  that caused the fallback.
- Transcode includes the decision reason plus non-compatible Direct Play and
  Remux reasons relevant to the selected transcode requirement.
- Denied uses `policy_denied`.
- Empty selected reasons must normalize to `compatible` for compatible
  decisions.
- Public/Admin/client surfaces may expose only stable compatibility enum/string
  codes; never expose Source Locators, local paths, bearer tokens, FFmpeg
  commands, provider payloads, or raw runtime stderr.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Direct Play selected and compatible | `selection_reasons == ["compatible"]` |
| Remux selected because source container is unsupported but codecs are compatible | includes `container_unsupported` |
| Transcode selected because video codec is unsupported | includes `video_codec_unsupported`; selected and direct-play detail arrays contain the same condition |
| Transcode selected because selected subtitle requires burn-in | includes `subtitle_delivery_unsupported` |
| Playback denied by policy | `selection_reasons == ["policy_denied"]` and detail copy says policy blocked the mode without exposing policy rows |
| Future compatibility condition is deserialized | Preserve the future condition string and project generic safe summary/detail copy |
| Generated SDK lacks `selection_reasons` | `nako-api` package-entry parity tests fail |
| Generated SDK lacks `selection_reason_details` or `reason_details` | `nako-api` package-entry parity tests fail |
| Route response contains raw path/token/FFmpeg command instead of reason code | contract violation |

### 5. Good/Base/Bad Cases

- Good: a browser flat capability query that lacks the source video codec
  returns a Transcode decision with `selection_reasons:
  ["video_codec_unsupported"]` and matching reason-detail entries.
- Good: a policy denial returns `policy_denied` without constructing Direct
  Play, Remux, or Transcode artifacts, and exposes only safe policy-denial
  detail copy.
- Base: older clients may omit `selection_reasons` during deserialization;
  serde/default-compatible DTOs must treat it as an empty list.
- Base: older responses may omit reason detail arrays during deserialization;
  serde/default-compatible DTOs must treat them as empty lists.
- Bad: deriving UI text from raw FFmpeg errors, file paths, or backend locator
  strings.
- Bad: duplicating compatibility reason copy in route handlers or generated SDK
  package files instead of using the protocol-owned projection.

### 6. Tests Required

- Planner matrix tests assert `decision.report.selection_reasons` for Direct
  Play, Remux, Transcode, subtitle, HDR/audio, requested-transcode, and Denied
  cases.
- Public API serialization tests assert the field is present and mapped to
  `ClientPlaybackCompatibilityCondition`.
- Protocol serde tests assert every known
  `ClientPlaybackCompatibilityCondition` has non-empty detail copy and unknown
  future strings project to generic safe copy.
- OpenAPI and SDK generator tests assert TypeScript/Kotlin package entries are
  regenerated.
- Server playback route tests assert flat capability query compatibility and
  redaction-safe reason/detail exposure.

### 7. Wrong vs Correct

#### Wrong

```rust
report.selection_reasons = vec![PlaybackCompatibilityCondition::Other(raw_error)];
```

#### Correct

```rust
report = report.with_selection_reasons(vec![
    PlaybackCompatibilityCondition::VideoCodecUnsupported,
]);
```

Selection reasons are stable product facts from planner evaluation, not raw
runtime diagnostics.

## Review Checklist

- Is the planner still pure?
- Are every decision reason and denial testable?
- Are new client/source facts included in profile identity?
- Can selected subtitle delivery change the result before Remux is selected?
- Does runtime work stay outside this crate?

## Wrong vs Correct

### Wrong

```rust
if report.remux.supported {
    return PlaybackMode::Remux;
}

let subtitle_requirement = selected_subtitle_requirement(...);
```

This can let a remux-capable container bypass a selected subtitle that requires
burn-in.

### Correct

```rust
let subtitle_requirement = selected_subtitle_requirement(...);
let report = evaluate_remux(..., subtitle_requirement);

if report.remux.supported {
    return PlaybackMode::Remux;
}
```

Subtitle delivery is part of compatibility, not a later decoration.
