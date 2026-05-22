# Android Playback Session Integrity

Status: Closed
Last updated: 2026-05-19

## Why This Lane Exists

Android smoke now proves server-backed browse, detail, source picker, Direct
Play advancement, and server **User Playback State** readback. That closes the
first playback depth gap, but it still leaves a deeper contract gap for server
owned playback sessions.

The Public Client API already exposes playback-session inspection and
cancellation routes, and Android already has client methods for them. However,
the current Android launch path does not carry a stable playback session id
into the player lifecycle. `NakoBrowseShell` opens the player with
`sessionId = null`, so remux/HLS playback cannot be inspected or cancelled by
the native shell after playback starts.

This lane makes playback sessions a first-class Android smoke contract instead
of an incidental server implementation detail.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0025-openapi-public-client-sdk-contract.md`
- `docs/adr/0026-native-client-shells-with-shared-rust-client-core.md`
- `docs/adr/0028-user-playback-state-principal-and-public-contract.md`
- `docs/workstreams/android-playback-depth-validation/`
- `docs/workstreams/playback-streaming/`
- `apps/android/SMOKE_FIXTURES.md`
- `crates/nako-api/src/openapi.rs`

## Target State

When this lane closes:

- Android can identify server-owned remux/HLS playback sessions without
  parsing unsafe implementation details.
- Player lifecycle code carries the session id when the Public Client API
  exposes one.
- Player exit code has test-covered cancellation semantics for active
  non-ended session ids through the Public Client API.
- Smoke evidence proves session creation and readback with token-safe
  artifacts.
- Direct Play remains sessionless by design.
- Longer playback-threshold and playback-quality checks are either completed
  or split into explicit follow-ons.

## In Scope

- Public Client API/session identity work needed by native clients.
- Android playback launch/session propagation.
- Focused Android unit tests for session propagation and safe diagnostics.
- Smoke fixture/script evidence that proves remux or HLS session creation and
  server readback.
- Documentation of session evidence and remaining playback-depth gaps.

## Out Of Scope

- V3 irregular Android UI work.
- New media-library, metadata, or catalog semantics.
- Full subtitle/audio-track/chapter UX.
- Downloads, offline playback, external player handoff, PiP, Android TV, or
  media notification controls.
- Admin-only playback diagnostics as the authority for client behavior.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Native clients need a stable session id before they can manage remux/HLS lifecycle correctly. | High | Android has `getPlaybackSession` and `cancelPlaybackSession`, but launch currently sets `sessionId = null`. | Add or revise the Public Client API so session identity is exposed at the right boundary. |
| Direct Play should stay sessionless. | High | Direct Play streams source bytes directly and does not allocate a transcode session. | Keep Direct Play smoke separate from remux/HLS session smoke. |
| HLS/remux session evidence should use Public Client API routes, not server internals. | High | ADR 0026 requires clients to consume public playback decisions, URLs, session inspection, and cancellation. | Split server contract work before Android relies on hidden state. |
| A short fixture can prove session identity and readback, but not realistic watched-threshold policy. | High | Current Android smoke fixture is intentionally short for speed. | Keep longer media threshold validation as a follow-on slice. |

## Architecture Direction

Treat playback session identity as a public client contract. Android should not
derive a session id by scraping local server paths, private logs, admin-only
diagnostics, or HLS playlist implementation details.

The preferred direction is:

- keep Direct Play as a simple byte-stream target with no session id;
- expose remux/HLS session identity at the Public Client API boundary that
  prepares or starts server work;
- carry that identity through Android `PlaybackLaunchRequest`;
- let the player lifecycle use existing session inspection/cancellation client
  methods;
- prove the behavior through structured smoke artifacts.

If the first implementation discovers that the current API cannot expose a
session without a contract change, update the API/OpenAPI/client surfaces
explicitly instead of adding an Android-only workaround.

## Closeout Condition

This lane can close when:

- at least one non-Direct playback path has deterministic Android smoke
  evidence for session creation and readback;
- active-session exit or cancellation is proven or explicitly split with a
  narrower blocker;
- session ids are propagated only through public, token-safe contracts;
- Android and server/client tests pass for touched surfaces;
- `apps/android/SMOKE_FIXTURES.md` and workstream evidence document the final
  gate;
- remaining HLS/remux depth, longer fixture, playback quality, and advanced
  player UX work is split or deferred explicitly.

## Closeout Decision

Closed on 2026-05-19 with the active non-ended remux/HLS runtime smoke split as
a follow-on. The shipped lane proves the public remux/HLS session contract,
Android launch propagation, token-safe smoke readback, and code-level player
exit cancellation semantics. The current `profile-with-media` UI fixture still
uses the short MP4 Direct Play path, so it is not a valid active-session
cancellation smoke. A future long-media or non-Direct runtime fixture should
prove cancellation against an actual active remux/HLS player session.
