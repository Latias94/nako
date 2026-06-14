# Media Web Auto Resume Playback Position

## Goal

Make the Media Web watch route actually resume video playback at the saved User Playback State position when it is safe to do so. The previous slice made resume state visible and linked Continue Watching to the watch route; this slice should complete the player behavior by seeking the browser video after metadata is available.

## What I already know

- `/media` Continue Watching now links to `/media/watch/$itemId?source_id=<saved source>`.
- `/media/watch/$itemId` already displays resume context from `getUserPlaybackState`.
- `MediaWatchPage.tsx` owns the browser ticket, playback candidate selection, video element handlers, progress writes, pause flush, and ended watched-state writes.
- `MediaVideoElement` is the narrowest component that can safely react to `loadedmetadata` and assign `video.currentTime`.
- Existing progress writes are deliberately gated behind `play` / `playing`, so a metadata-only seek must not create progress writes.
- Admin Web must keep ticket tokens, stream URLs, fingerprints, and raw source internals out of rendered text.

## Requirements

- When the saved playback state belongs to the selected source, seek the video to `resume_position_ms` after video metadata is loaded.
- Seek only once per playback candidate/retry cycle so React rerenders do not repeatedly override user-controlled position.
- Do not auto-seek when the saved `source_id` differs from the selected source.
- Do not auto-seek when `resume_position_ms` is missing, zero, negative, or at/near the media duration.
- Keep progress updates gated on playback start; automatic metadata seek must not call `updateUserPlaybackProgress`.
- Provide a visible, redaction-safe "Start over" control when a resumable position exists for the current source.
- If the user selects Start over before metadata is loaded, do not auto-seek later for that candidate.

## Acceptance Criteria

- [ ] A watch route opened with matching saved source auto-seeks the video to the saved resume position on `loadedmetadata`.
- [ ] A watch route opened with a different source does not auto-seek.
- [ ] The auto-seek happens only once for the same candidate/retry key.
- [ ] Clicking Start over sets the current video position to `0` and prevents a later auto-resume for that candidate.
- [ ] Metadata-only auto-seek does not call playback progress mutations.
- [ ] Existing ticket redaction, progress throttling, pause flush, and ended watched-state tests still pass.

## Definition of Done

- Focused Media Web tests cover auto-resume, source mismatch, and Start over behavior.
- `npm run check --prefix apps/admin-web` passes.
- Focused Media Web tests pass.
- Full Admin Web tests and production build pass unless a clear unrelated flake is documented.
- Trellis task context validates.

## Technical Approach

- Compute a safe resume position in `MediaWatch` from `playbackState.value?.state`, the selected source, and the fallback duration.
- Pass that resume position into `MediaBrowserPlayer`, then into `MediaVideoElement`.
- Add an `onLoadedMetadata` handler that applies a bounded seconds value to `video.currentTime`.
- Track the candidate key/retry key inside `MediaVideoElement` to prevent repeated seeks for the same rendered video.
- Add a Start over button alongside the existing player facts. It will set `video.currentTime = 0` when the element is mounted and suppress future auto-resume for the active candidate.
- Keep the behavior frontend-only and reuse existing data-source state; no API or SDK changes.

## Decision (ADR-lite)

**Context**: The saved playback state now reaches the Watch route, but the browser video still starts at the beginning. Browser media duration is only reliable after metadata loads, and setting `currentTime` during render or before metadata can be unreliable.

**Decision**: Implement auto-resume at the `MediaVideoElement` boundary using the `loadedmetadata` event, with explicit one-shot state per candidate and a Start over override.

**Consequences**: This keeps the behavior close to the DOM event that makes seeking safe and avoids contaminating data hooks with player element state. HLS adapter behavior remains best-effort through the same video element; deeper HLS resume/session semantics can be handled later.

## Out of Scope

- Backend playback session resume support.
- Persisting "start over" as a watched/progress mutation.
- Cross-device conflict resolution or heartbeat buffering.
- HLS segment-level resume tuning beyond HTMLVideoElement seek.
- Public product frontend redesign.

## Technical Notes

- `apps/admin-web/src/surfaces/media/MediaWatchPage.tsx` is the primary implementation file.
- `apps/admin-web/src/surfaces/media/mediaSurface.test.tsx` already has Watch route, browser ticket, progress, pause, and ended tests.
- `apps/admin-web/src/surfaces/media/MediaItemShared.tsx` provides current playback state and selected source.
- Relevant specs:
  - `.trellis/spec/admin-web/frontend/routes-forms-and-tests.md`
  - `.trellis/spec/guides/code-reuse-thinking-guide.md`
  - `.trellis/spec/guides/cross-layer-thinking-guide.md`
