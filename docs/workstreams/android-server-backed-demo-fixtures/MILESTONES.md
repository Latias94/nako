# Android Server-Backed Demo Fixtures — Milestones

Status: Closed
Last updated: 2026-05-18

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Android fake media data is rejected as a fixture strategy.
- Public Client API route shapes are named as the fixture boundary.
- First proof target and gates are chosen.

Primary evidence:

- `docs/workstreams/android-server-backed-demo-fixtures/DESIGN.md`
- `docs/workstreams/android-server-backed-demo-fixtures/TODO.md`

## M1 — Fixture Contract Discovery

Exit criteria:

- Required Public Client API routes for Home, detail, source picker, and
  player-safe launch are listed.
- The first implementation strategy is chosen: seeded Taru server or
  public-route-compatible local test-server harness.
- Missing contract gaps are recorded before implementation broadens.

Primary gates:

- Focused route inventory review against `crates/taru-api/src/openapi.rs`.
- Focused Android client request construction tests when Android code changes.

Outcome:

- Completed on 2026-05-18 in `ROUTE_MATRIX.md`.
- First strategy: real local `taru-server` seeded fixture, with Android access
  through `adb reverse`.
- Fallback: public-route-compatible test-server harness only if real seeded
  startup becomes too brittle.

## M2 — Server-Backed Fixture Provider

Exit criteria:

- A deterministic fixture provider or startup path exists.
- Fixture responses are safe: no access-token values, token references,
  local filesystem paths, provider payloads, FFmpeg command lines, or unsafe
  diagnostics.
- Demo Media Libraries, Media Items, Item Detail, Media Sources, and playback
  decisions are available through public route shapes.

Primary gates:

- Focused server/API tests for the fixture provider.
- Request-level checks for redaction and DTO safety.

Outcome:

- Completed on 2026-05-18.
- `apps/android/scripts/Start-DemoFixtureServer.ps1` prepares a generated
  Movies library, seeds `Night Harbor` through real `taru-server scan` and
  `import-nfo`, and can start the fixture server on loopback.
- Android `ClientTranscodePlan` no longer requires the internal
  `input_locator` field.
- A short-lived local server validated `/health`, `/libraries`, `/items`,
  `/items/{item_id}`, `/sources/{source_id}/playback/decision`, and direct
  stream HEAD without unsafe response text.

## M3 — Android Media Smoke State

Exit criteria:

- Android smoke can select a named media fixture state.
- The smoke state seeds a safe Server Profile and reaches the fixture provider.
- Home, detail, source picker, and player-safe launch evidence is captured
  under `apps/android/build/smoke/`.
- README and fixture docs teach the command and safety rules.

Primary gates:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon`
- `apps\android\gradlew.bat -p apps\android :app:assembleDebug --no-daemon`
- `pwsh -NoProfile -File apps\android\scripts\Smoke-Emulator.ps1 -FixtureState profile-with-media`

Outcome:

- Completed on 2026-05-18.
- `Smoke-Emulator.ps1` now accepts `profile-with-media`, prepares the
  server-backed `Night Harbor` fixture, starts a local `taru-server`, applies
  `adb reverse`, and seeds the debug APK through real profile and token stores.
- The smoke captures Home, detail, source picker, and player evidence under
  `apps/android/build/smoke/`.
- The debug-only fixture writer is not present in release source.

## M4 — Safety, Verification, And Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- `WORKSTREAM.json`, `HANDOFF.md`, and evidence docs reflect shipped behavior.
- Remaining CI, golden visual diff, and deeper playback validation work is
  either completed, deferred, or split into follow-ons.

Outcome:

- Completed on 2026-05-18.
- Fresh gates passed for Android unit tests, debug assemble, server fixture
  prepare, full `profile-with-media` smoke, and `git diff --check`.
- No blocking review findings remain.
- Follow-ons are explicitly deferred: CI/device-farm integration, golden visual
  diffing, HLS/remux/session depth, and longer playback quality validation.
