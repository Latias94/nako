# Playback Capability Profile And Rendition Planning - Design

Status: Completed
Last updated: 2026-05-28

## Problem

ADR 0044 accepted a profile-driven playback capability planner. The current
implementation has the right direction, but two old shapes still blur the
boundary:

- `PlaybackDecision.execution` mixes playback output shape with execution
  runtime language.
- `PlaybackProfile` is a shallow adapter around `PlaybackTargetProfile`; it
  keeps a second identity format and duplicates transcode-profile helpers.

This makes it too easy for future adaptive HLS, fMP4, subtitle/HDR, DLNA, or
Chromecast work to bypass the capability profile and route around the planner.

## Intent

Make `nako-playback` expose one typed output intent:
`PlaybackRenditionPlan`. Server runtime modules should consume rendition
plans, not infer output shape from scattered decision fields or a shallow
compatibility helper.

This removes a future source of accidental complexity before adaptive HLS,
fMP4, DLNA device profiles, or remote workers increase the number of playable
target shapes.

## Scope

- `crates/nako-playback`: planner decision shape, target profile helpers,
  rendition plan, source-aware transcode requirement placement, tests.
- `crates/nako-server`: playback selection helpers, remux/HLS request identity
  generation, renderer playback transport planning, focused tests.
- `crates/nako-api`: Public Client decision DTO mapping stays redaction-safe
  while using the new rendition boundary.
- `docs/workstreams`: durable lane docs and closeout evidence.

## Non-Goals

- Do not add adaptive HLS ladders.
- Do not add fMP4/CMAF output.
- Do not implement DLNA device profiles.
- Do not change Public Client or Admin wire contracts unless required by the
  refactor.
- Do not evaluate rsmpeg or remote transcode workers in this lane.

## Deletion Plan

- Delete the shallow `PlaybackProfile` type and its second identity key format.
- Delete duplicate `PlaybackDecision` fields that restate the selected output:
  `execution`, `direct_play`, `transcode_plan`, and top-level
  `transcode_requirement`.
- Replace decision consumers with `PlaybackDecision.rendition`.

## Boundary Plan

Target shape:

```text
Media Source facts
  + Media Technical Facts
  + PlaybackTargetProfile
  + EffectivePlaybackPolicy
  + PlaybackSelectionContext
  -> PlaybackDecision
       mode
       reason
       selected_source
       report
       rendition: PlaybackRenditionPlan
```

`PlaybackRenditionPlan` owns direct play, remux, transcode, and denied output
intent. Transcode rendition carries both the public-safe `TranscodePlan` and
the source-aware `TranscodeRequirement` needed by deeper runtime planning.

`PlaybackTargetProfile` owns transcode profile generation and identity. Server
runtime request identities should bind to `PlaybackTargetProfile`-derived
transcode profiles instead of a downgraded compatibility helper.

## Testing Plan

- `cargo nextest run -p nako-playback --no-fail-fast`
- `cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
- `python3 -m json.tool docs/workstreams/playback-capability-profile-and-rendition-planning/WORKSTREAM.json`

## Risk Plan

- **Request key churn:** deleting `PlaybackProfile` changes transcode request
  keys to the richer target-profile identity. Nako has no compatibility-bound
  users yet, and the change is acceptable. Existing tests should assert the new
  identity source.
- **API leakage:** Public DTO mapping must keep hiding source locators, raw
  input paths, command lines, transcode requirements, and internal rendition
  details.
- **Behavior drift:** focused playback and renderer tests must continue to pass.

## Refactor Brief

- **Intent:** remove duplicate planner output shapes before richer target
  capability and rendition work arrives.
- **Scope:** `nako-playback`, `nako-api`, `nako-server` playback runtime, docs.
- **Deletion plan:** remove `PlaybackProfile`; remove duplicate decision plan
  fields; replace `execution` language with `rendition`.
- **Boundary plan:** `PlaybackTargetProfile` owns identity and transcode
  profile generation; `PlaybackRenditionPlan` owns selected output intent.
- **Testing plan:** pure planner tests, API redaction test, server playback
  gate, fmt/diff/JSON checks.
- **Risk plan:** accept request-key churn; protect wire redaction and playback
  behavior with tests.
- **Workflow plan:** durable workstream with one bounded refactor slice,
  followed by review, verification, closeout, and user-confirmed commit.

## Completed Slice

Completed on 2026-05-28. `PlaybackDecision` now owns one selected-output field:
`rendition: PlaybackRenditionPlan`. Transcode rendition carries both the
public-safe `TranscodePlan` and the source-aware `TranscodeRequirement`.

`PlaybackTargetProfile` now owns remux/HLS transcode profile construction,
track selection, output constraints, and request-key identity. The shallow
`PlaybackProfile` compatibility adapter was deleted.
