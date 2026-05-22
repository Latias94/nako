# Android Server-Backed Demo Fixtures

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

The Android client now has a Material 3 Expressive V2 shell and a local
emulator smoke harness. That harness can prove setup, Home, Settings, and
Server Profile shell behavior, but it cannot yet prove real media browsing,
detail, source picker, or player entry points because there is no deterministic
server-backed fixture state exposed through the Public Client API.

Android should not solve this by inventing fake Media Items or playback data in
the app. The demo state has to come from the same public boundary that real
Client Applications use, otherwise the smoke evidence will validate a different
product than Nako ships.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/workstreams/android-client-foundation/`
- `docs/workstreams/android-material-expressive-ui/`
- `docs/workstreams/android-client-qa-harness/`
- `apps/android/README.md`
- `apps/android/SMOKE_FIXTURES.md`
- `crates/nako-api/src/openapi.rs`

## Problem

- The emulator smoke harness has repeatable shell evidence, but not repeatable
  media content evidence.
- `profile-with-media` and `playback-ready` are intentionally deferred because
  they require Public Client API-backed Media Libraries, Media Items, Item
  Detail, Media Sources, and playback decisions.
- Android can already render Home, browse, search, detail, source picker, and
  player surfaces, but current smoke checks cannot prove those surfaces against
  realistic server data.
- If Android adds fixture-only media models locally, future UI work will drift
  away from Public Client API semantics and hide contract gaps.
- Server-side demo state is not yet documented as a stable contract for client
  smoke tests, so different agents may seed different data or overreach into
  server internals.

## Target State

- A deterministic local fixture path exists for Android smoke checks that need
  real media content.
- Fixture data reaches Android only through Public Client API responses or an
  explicit local test-server harness that implements those same public routes.
- The first fixture state can drive Home, one Media Item detail view, source
  picker, and player-safe playback launch surfaces without exposing secrets,
  local paths, provider payloads, FFmpeg command lines, or token values.
- The Android smoke script can select this state by name and writes reproducible
  evidence under `apps/android/build/smoke/`.
- Fixture boundaries, startup commands, safety rules, and verification gates
  are documented before broadening to CI or golden visual diffing.

## In Scope

- Public Client API fixture contract discovery for the minimal Android media
  smoke path.
- A local server-backed or public-route-compatible demo fixture strategy for:
  - Media Libraries;
  - Media Items;
  - Item Detail;
  - people/tags/genres enough to exercise detail chips and facet navigation;
  - Media Sources;
  - source probe or playback decision responses needed for source picker and
    player launch.
- Android smoke harness changes that consume a server-backed profile and
  capture Home, detail, source picker, and player evidence.
- Documentation for fixture startup, safe data, evidence paths, and gates.
- Focused tests for fixture request construction, redaction, and smoke state
  selection.

## Out Of Scope

- Android-only fake media data presented as if it came from Nako.
- Full golden screenshot diff infrastructure.
- CI device-farm integration.
- Production catalog schema changes unless a later task proves they are needed
  and updates the relevant ADR or workstream.
- Full playback runtime validation, transcoding quality validation, subtitles,
  audio tracks, downloads, external player handoff, or User Playback State.
- V3 irregular layout exploration.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The Public Client API already has enough route vocabulary for the first fixture: libraries, items, item detail, relations, source probe, and playback decision. | High | `crates/nako-api/src/openapi.rs` lists the current route set. | Narrow the first slice to route discovery and split API contract work before Android smoke implementation. |
| The fixture must be server-backed or public-route-compatible, not Android-local media data. | High | ADR 0026 says native clients consume Public Client API and must not bypass playback selection or server policy. | Stop Android smoke broadening until a compliant fixture provider exists. |
| A local harness fixture is enough before CI/golden infrastructure. | High | `android-client-qa-harness` closed with local emulator smoke evidence and deferred CI/golden work. | Split a CI/golden lane only after local fixture behavior is stable. |
| Player smoke can initially prove player-safe launch and error handling without validating full streaming quality. | Medium | Current Android player route owns Media3 launch behavior, but fixture media availability may vary. | Keep full playback runtime validation out of this lane and record it as a later playback lane. |

## Architecture Direction

Keep Android honest by treating the Public Client API as the only media fixture
boundary. The fixture provider may be a real seeded Nako server or a small local
test-server harness, but Android must see only public route shapes and safe
public DTO fields.

The lane should prefer a small vertical proof over a broad demo universe:

- one safe demo Server Profile;
- one or two Media Libraries;
- a small set of Media Items with poster/backdrop-safe metadata;
- enough people/tags/genres to prove detail chips and facet links;
- one playable or player-safe Media Source;
- a playback decision response that drives the existing Android source picker
  and player launch surfaces.

Server work, if needed, belongs on the server side of the Public Client API
boundary. Android work should be limited to profile seeding, smoke navigation,
fixture-state selection, request construction, and evidence capture.

## Closeout Condition

This lane can close when:

- the server-backed fixture boundary is documented and implemented or a narrow
  follow-on is split for missing server capability;
- Android smoke can run a named media fixture state against an explicit local
  endpoint;
- Home, detail, source picker, and player-safe launch evidence are captured
  without fake Android media data;
- unit/build gates and emulator smoke gates pass fresh;
- safety rules around token values, local paths, locator leakage, and unsafe
  diagnostics are enforced or documented as non-negotiable;
- remaining CI, golden visual diff, or deeper playback validation work is
  split or explicitly deferred.
