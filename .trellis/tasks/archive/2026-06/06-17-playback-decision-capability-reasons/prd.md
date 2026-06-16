# feat: Playback decision capability reasons first slice

## Goal

Make Nako's public playback decision output explain why a request is direct
play, remux, transcode, or denied from explicit client/device capability facts.
This is the first backend slice for playback product maturity: operators and
clients should be able to see safe, actionable compatibility reasons without
exposing local paths, source internals, or implementation-only details.

## Closeout Conclusion

Code and contract inspection showed this capability is already implemented and
covered in the current branch. The correct action for this task is to record
verification evidence and close it as a no-code validation slice, then move the
U2 playback maturity work to the next real gap: named client/device capability
profiles on top of the existing flat v1 fields.

## What I Already Know

* The roadmap identifies device capability profiles and playback compatibility
  reasons as a required self-hosted media-server maturity gap.
* `docs/architecture/PLAYBACK.md` treats `ClientPlaybackCapabilities + policy +
  user preferences -> PlaybackRenditionPlan` as the target chain.
* Existing server playback routes already accept flat client capability query
  fields and map playback decisions into public client API DTOs.
* The current code already exposes
  `ClientPlaybackDecisionReport.selection_reasons` with stable compatibility
  reason codes across protocol, OpenAPI, SDK, server route tests, and HTTP
  docs.
* This task should stay backend-focused. Android, Admin Web, and full device
  profile catalogs are not part of the first slice unless a contract adjustment
  forces a narrow client fixture update.

## Requirements

* Preserve existing public playback route compatibility; do not remove or rename
  existing query fields or response fields in this slice.
* Derive decision explanations from typed playback/capability inputs, not from
  ad hoc HTTP string handling.
* Expose safe decision reasons for at least direct play, remux, transcode, and
  denial outcomes when the underlying planner has enough information.
* Represent capability blockers by category where practical: container, video,
  audio, subtitle, HDR, policy, and source availability.
* Keep public reason text/data free of local filesystem paths, private storage
  identifiers, raw probe payloads, secrets, or stack traces.
* Add focused regression coverage at the playback planner and HTTP/public-client
  mapping boundary.

## Acceptance Criteria

* [x] A request with an unsupported video capability returns a transcode outcome
      with a stable, public-safe reason tied to the video capability blocker.
* [x] A request that can avoid full transcode via container or audio adaptation
      exposes a remux/adaptation reason rather than a generic fallback reason,
      if current planner inputs can express the case.
* [x] A selected subtitle or HDR/policy incompatibility is represented as a
      category-specific reason when current planner inputs can express it.
* [x] Denied/no-rendition decisions include a public-safe explanation without
      leaking source-local details.
* [x] Existing playback query compatibility tests still pass.
* [x] Focused `cargo nextest` coverage passes for touched Rust crates.

## Definition of Done

* Tests are added or updated at the narrowest useful layer and broadened where
  public API behavior changes.
* `cargo fmt --all` or targeted formatting is run for touched Rust code.
* Public API or protocol docs are updated if response shape changes.
* Trellis context files point at the relevant specs and architecture docs.
* The task is completed and committed with a Conventional Commit message.

## Technical Approach

Start by inspecting the current playback planner, public API DTOs, and server
HTTP mapping. Prefer deepening the existing playback decision/reason model over
adding a parallel response wrapper. If the current DTO already has a reason
field, make the first slice improve its typed detail and mapping coverage. If it
does not, add a backward-compatible optional compatibility explanation block.

The inspection found that the existing DTO already has the needed
`selection_reasons` field and that planner/API/server tests already cover the
important first-slice cases. No product-code change is needed for this task.

## Decision (ADR-lite)

**Context**: Playback troubleshooting is a core self-hosted media-server workflow
and should be explainable to both users and operators.

**Decision**: Implement a narrow, backward-compatible public explanation layer
based on existing typed playback capability facts before introducing a full
device profile catalog.

**Consequences**: This keeps the first slice shippable and testable, but it will
not solve every TV/browser profile permutation. Later slices can add named
device profiles, richer client capability reporting, and UI presentation on top
of the same reason model.

## Out of Scope

* Full Jellyfin/Plex-style device profile database.
* FFmpeg runtime execution changes.
* Android playback UI changes beyond contract fixtures if required.
* Admin Web presentation of playback reasons.
* Schema migrations unless inspection proves the current data model cannot
  represent the first-slice reasons.

## Technical Notes

* Reference docs:
  * `docs/plans/2026-06-10-001-feat-media-server-maturity-roadmap-plan.md`
  * `docs/plans/MEDIA_SERVER_PARITY_GAP_MATRIX.md`
  * `docs/architecture/PLAYBACK.md`
* Likely code areas:
  * `crates/nako-playback`
  * `crates/nako-api/src/public_client.rs`
  * `crates/nako-client-protocol`
  * `crates/nako-server/src/app/playback`
  * `crates/nako-server/src/http/playback.rs`
  * `crates/nako-server/src/http/tests/playback.rs`

## Verification Evidence

* `cargo nextest run -p nako-playback playback_compatibility --no-fail-fast`
  passed: 2 tests.
* `cargo nextest run -p nako-server playback_decision --no-fail-fast` passed:
  4 tests.
* `cargo nextest run -p nako-client-protocol public_playback --no-fail-fast`
  passed: 2 tests.
* `cargo nextest run -p nako-api public_openapi_playback_decision_uses_typed_reason_and_capabilities playback_decision_dto_hides_internal_selection_plan --no-fail-fast`
  passed: 2 tests.
* `python ./.trellis/scripts/task.py validate 06-17-playback-decision-capability-reasons`
  passed.
* `git diff --check -- .trellis/tasks/06-17-playback-decision-capability-reasons`
  passed.

## Follow-On

Open a new U2 task for named client/device capability profiles. Keep the current
flat v1 query/body fields as the compatibility baseline, and add profile
identity only as an additive contract after the protocol, OpenAPI, SDK, server
mapping, HTTP docs, and playback profile identity tests are planned together.
