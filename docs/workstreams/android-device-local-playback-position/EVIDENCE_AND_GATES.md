# Android Device-Local Playback Position - Evidence And Gates

Status: Closed
Last updated: 2026-05-19

## Gate Set

### Focused Android Unit Gate

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon
```

### Local Android Validation

```powershell
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
```

### Diff Hygiene

```powershell
git diff --check
```

## Evidence Anchors

- `apps/android/app/src/main/java/dev/taru/android/player/DevicePlaybackPosition.kt`
- `apps/android/app/src/main/java/dev/taru/android/ui/TaruAndroidApp.kt`
- `apps/android/app/src/test/java/dev/taru/android/player/PlaybackLaunchTest.kt`
- `apps/android/build/validation/<timestamp>/report.md`

## Notes

This lane must not claim server-authoritative **User Playback State** or
cross-device Continue Watching. Generated validation reports remain local under
`apps/android/build/`.

## ADP-020 / ADP-030 Evidence

Validated on 2026-05-19:

```powershell
apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.taru.android.player.PlaybackLaunchTest --no-daemon
pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke
git diff --check
```

Validation report:

- `apps/android/build/validation/20260519-100247/report.md`

What this proves:

- `SharedPreferencesDevicePlaybackPositionStore` persists local resume points
  across store instances.
- Stored positions remain scoped by server profile id, Media Item id, and Media
  Source id.
- Non-positive positions clear the local resume point.
- Corrupt local stored data is dropped instead of crashing or leaking into UI.
- The app composition now uses the persistent store by default.
- The broader no-emulator Android local validation gate passed after the
  persistence change.

Closeout decision:

- Close this Android-local persistence lane. Server-authoritative **User
  Playback State**, cross-device Continue Watching, progress reporting, and
  playback stream session-envelope changes remain separate follow-ons.
