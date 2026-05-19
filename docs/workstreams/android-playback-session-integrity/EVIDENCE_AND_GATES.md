# Android Playback Session Integrity - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Gate Set

Run the narrowest relevant gates first, then broaden before closeout.

Planned focused gates:

```powershell
cargo nextest run -p taru-api -p taru-client -p taru-server playback
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --no-daemon
pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null"
pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media -SkipAppBuild -SkipFixtureServerBuild
git diff --check
```

Adjust focused test filters as the implementation lands.

## APSI-010 - Scope Evidence

Status: Complete

Evidence:

- `docs/workstreams/android-playback-session-integrity/DESIGN.md`
- `docs/workstreams/android-playback-session-integrity/TODO.md`
- `docs/workstreams/android-playback-session-integrity/WORKSTREAM.json`

Notes:

- First slice targets Public Client API session identity and Android launch
  propagation.
- HLS/remux quality, long watched-threshold semantics, PiP, subtitles, and
  track UX remain out of the first slice.

## APSI-020 - Complete

Evidence from 2026-05-19:

- `cargo test -p taru-client-protocol -p taru-api -p taru-client --lib`
  passed. Proves the shared public route inventory, OpenAPI session header
  contract, Rust client remux `HEAD` builder, and generated TypeScript SDK
  output remain consistent.
- `cargo test -p taru-server remux_stream_route -- --nocapture` passed.
  Proves remux `GET` keeps streaming behavior and remux `HEAD` exposes
  `x-taru-playback-session-id` without a response body.
- `cargo test -p taru-server hls_playlist_and_segment_routes_work -- --nocapture`
  passed. Proves HLS playlist `GET` exposes the public playback session id
  header while segment routes still resolve through the session route.
- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon`
  passed. Proves Android prepares remux/HLS playback targets by reading the
  public session header, keeps Direct Play sessionless, and does not leak
  bearer tokens through playback target diagnostics.
- `apps\android\gradlew.bat -p apps\android :app:compileDebugKotlin --no-daemon`
  passed.
- `npm run check --prefix sdk/typescript` passed.
- `cargo fmt --check` passed.
- `git diff --check` passed.

Notes:

- Android now carries the observed remux/HLS session id through
  `PlaybackRequestTarget.sessionId` and `PlaybackLaunchRequest.sessionId`.
- Android does not parse HLS playlist text, server local paths, server logs, or
  admin-only diagnostics to discover the session id.
- Smoke evidence is intentionally deferred to APSI-030.

## APSI-030 - Complete

Evidence from 2026-05-19:

- `pwsh -NoProfile -Command "[scriptblock]::Create((Get-Content -LiteralPath 'apps/android/scripts/Smoke-Emulator.ps1' -Raw)) | Out-Null; 'Smoke-Emulator parse ok'"`
  passed.
- `pwsh -NoProfile -File apps/android/scripts/Smoke-Emulator.ps1 -FixtureState profile-with-media`
  passed on emulator `emulator-5554`.
- Evidence directory:
  `apps/android/build/smoke/20260519-190847-profile-with-media-emulator-5554/`
- Smoke report:
  `apps/android/build/smoke/20260519-190847-profile-with-media-emulator-5554/report.md`
- Public session artifact:
  `apps/android/build/smoke/20260519-190847-profile-with-media-emulator-5554/profile-with-media-session-readback.txt`

Observed artifact summary:

- Remux preflight route:
  `/sources/{source_id}/stream/remux?container=mkv&video_codec=h264&audio_codec=aac&output_container=mkv`
- Preflight method: `HEAD`
- Preflight status: `200`
- Session header: `x-taru-playback-session-id`
- Session readback route: `/playback/sessions/{session_id}`
- Observed kind: `remux`
- Observed state: `finished`
- Artifact records `Created before Android player exit: true` and
  `Observed after Android player exit: true`.

Safety review:

- `profile-with-media-session-readback.txt` contains `Access token: <redacted>`.
- Checked the session artifact and report for
  `demo-fixture-token`, `output_path`, `file://`, `local://`, `ffmpeg`, and
  Windows drive-path patterns. The session artifact did not contain those
  forbidden values. `report.md` still contains existing APK/repo local paths,
  which are legacy smoke report metadata and not part of the token-safe session
  readback artifact.

Notes:

- The current Android UI path still chooses Direct Play for the MP4 demo
  fixture. APSI-030 therefore proves the non-Direct remux session through the
  Public Client API smoke artifact rather than claiming the UI player consumed
  that remux session.
- APSI-040 should either add an active remux/HLS player runtime path that can
  prove cancellation, or split that runtime blocker explicitly.

## APSI-040 - Complete

Evidence from 2026-05-19:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackExitEffectsTest --tests dev.taru.android.player.UserPlaybackReportingTest --tests dev.taru.android.playback.TaruPlaybackClientTest --no-daemon`
  passed. Proves player-exit side effects are testable outside Compose and
  that unfinished playback with a non-blank session id saves local position,
  reports progress, and requests Public Client API session cancellation.
- `apps\android\gradlew.bat -p apps\android :app:compileDebugKotlin --no-daemon`
  passed after the player exit refactor.
- `git diff --check` passed.

Closeout split:

- The current `profile-with-media` UI fixture plays the short MP4 source by
  Direct Play, so it cannot prove active non-ended remux/HLS player
  cancellation.
- The lane closes with code-level cancellation semantics covered and smoke
  session creation/readback covered. A follow-on should add a deliberate
  long-media or non-Direct player runtime fixture that leaves a remux/HLS
  session active long enough to verify cancellation through the Public Client
  API.

## Closeout

Status: Closed

Fresh closeout evidence:

- Public session contract and Rust/TypeScript SDK gates under APSI-020.
- `profile-with-media-session-readback.txt` smoke evidence under APSI-030.
- `PlaybackExitEffectsTest` under APSI-040.

Residual follow-ons:

- Add active non-ended remux/HLS player cancellation smoke with a fixture that
  does not naturally finish before the player exits.
- Add full playback-quality, subtitles/audio-track/chapter UX, PiP, Android TV,
  and media notification behavior in separate lanes.
