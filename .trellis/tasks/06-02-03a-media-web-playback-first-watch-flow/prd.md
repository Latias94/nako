# Media Web Playback First Watch Flow

## Goal

Make the browser Media Web surface complete one practical first-watch flow from
catalog detail to playback start using Nako's public client and playback
contracts.

## Requirements

- Audit the current `web/src/features/media` playback path, public data source,
  and `video-player` behavior before changing code.
- Use Public Client API and SDK/client-core request builders where available;
  do not call Admin routes for user playback.
- Support the first viable playback start path for Direct Play, Remux, or HLS
  based on existing playback decision/session contracts.
- Keep bearer tokens and playback tickets redacted in visible UI state, route
  state, logs, and tests.
- Preserve Media Web layout quality on desktop and mobile.
- Add route/data-source/component tests for the chosen watch flow.

## Acceptance Criteria

- [ ] A user can navigate from Media Web item/detail context into a playback
  route or player state.
- [ ] Playback decision/session request construction uses public playback
  contracts and safe previews.
- [ ] The player handles at least one realistic playable URL mode without
  exposing tokens.
- [ ] Tests cover route state, data-source mapping, and player action behavior.
- [ ] Build/bundle checks for the touched Web package pass.
- [ ] Browser or Playwright smoke evidence is recorded if UI behavior changes.

## Definition of Done

- Focused Web tests pass.
- Relevant Rust public-client/playback contract tests pass if touched.
- Redaction and route-state behavior are explicitly tested.
- PRD/evidence notes are updated with commands run and remaining follow-ons.

## Out of Scope

- No new playback engine implementation.
- No TV/casting feature expansion.
- No Admin-only player workflow.
- No schema migration unless a blocker proves it is required and planner
  approves the shared scope.

## Technical Notes

- Likely Web files: `web/src/features/media/video-player.tsx`,
  `web/src/features/media/media-detail.tsx`,
  `web/src/api/public/media-data-source.ts`, and route tests under
  `web/src/test`.
- Coordinate with the playback runtime lane before changing server playback
  session semantics.
- If public contract changes are required, stop and return to planner
  coordination before widening scope.
