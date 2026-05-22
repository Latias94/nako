# Android Device-Local Playback Position - Handoff

Status: Closed
Last updated: 2026-05-19

## Current State

This lane is closed. `ADP-010`, `ADP-020`, and `ADP-030` are complete.

## Closed Task

- Task ID: ADP-030
- Owner: codex
- Files: `apps/android/app/src/main/java/dev/nako/android/player`,
  `apps/android/app/src/main/java/dev/nako/android/ui`,
  `apps/android/app/src/test/java/dev/nako/android/player`
- Validation: focused `PlaybackLaunchTest`, `Validate-AndroidLocal.ps1
  -SkipSmoke`, and `git diff --check`.
- Status: DONE
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions

- Persist only Android device-local resume state.
- Scope records by server profile id, Media Item id, and Media Source id.
- Do not add Public Client API routes or cross-device state claims.
- Keep in-memory store for previews and tests.

## Blockers

- None known.

## Next Recommended Action

- Open a separate Public Client API/server workstream for authoritative
  **User Playback State** before Android claims cross-device Continue Watching,
  watched state, or server-owned resume.

## Latest Evidence

Validation passed on 2026-05-19:

- `apps\android\gradlew.bat -p apps\android :app:testDebugUnitTest --tests dev.nako.android.player.PlaybackLaunchTest --no-daemon`
- `pwsh -NoProfile -File apps\android\scripts\Validate-AndroidLocal.ps1 -SkipSmoke`
- `git diff --check`

Report:

- `apps/android/build/validation/20260519-100247/report.md`

## Residual Risks And Follow-ons

- Device-local resume is still local to one Android installation.
- Server-authoritative **User Playback State** and cross-device Continue
  Watching need a Public Client API/server lane.
- Remux/HLS stream responses still do not expose a public session id envelope
  to Android for full lifecycle cancellation without playlist parsing.
