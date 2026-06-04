# Media Web Playback Polish

## Goal

Improve the practical Media Web playback experience after the first-watch flow
shipped by hardening browser capability handling, retry behavior, and
user-facing playback failure states without changing backend contracts.

## Requirements

* Keep this slice frontend-only unless a blocker proves the existing Admin API
  contract cannot represent the needed state.
* Reuse the existing browser playback capability derivation and ticket flow.
* Make retry behavior deterministic after native HLS, `hls.js`, ticket, or
  media element failures.
* Preserve redaction: do not expose raw playback ticket values, local paths,
  source locators, or backend-internal diagnostics in UI state or tests.
* Keep Media Web layout stable on desktop and mobile.

## Acceptance Criteria

* [ ] Media playback retry state is covered by focused Media Web tests.
* [ ] Browser capability and player fallback state remain deterministic in test
      fixtures.
* [ ] Failed playback surfaces a recoverable state without leaking sensitive
      transport data.
* [ ] Existing first-watch happy-path behavior remains covered.
* [ ] Admin Web check, focused tests, build, and a browser smoke pass.

## Definition Of Done

* `npm run check --prefix apps/admin-web`
* `npm run test --prefix apps/admin-web -- src/surfaces/media/mediaSurface.test.tsx src/surfaces/media/mediaDataSource.test.ts`
* `npm run build --prefix apps/admin-web`
* Local browser smoke for the Media Web watch route when the route is runnable.
* `git diff --check`

## Technical Approach

Start from the existing Media Web playback components and data source. Keep API
and generated contract files out of scope. Prefer small state-machine or helper
extraction only when it reduces duplicated retry/capability handling inside the
Media surface.

## Out Of Scope

* Backend playback decision changes.
* Generated Admin contract changes.
* Public Client API changes.
* Desktop/native player implementation.
* LL-HLS/CMAF or new transcode behavior.

## Technical Notes

* Lane: `web-product`.
* Authorized write scope:
  * `apps/admin-web/src/surfaces/media/**`
  * `apps/admin-web/src/test/**`
  * `apps/admin-web/src/i18n/messages.ts` only if text keys are needed
* Forbidden scope:
  * `crates/**`
  * `apps/admin-web/src/adminApi/generated/**`
  * high-context repo instruction files

