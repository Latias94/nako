# Playback Capability Profile And Rendition Planning

Status: Completed
Last updated: 2026-05-28

This fearless refactor lane deepens Nako's playback planner after the
source-aware transcode and playback runtime boundary lanes. It removes the
remaining shallow playback-profile adapter and makes playback output shape a
typed Rendition Plan owned by `nako-playback`.

The first slice is intentionally behavior-preserving: browser, Nako renderer,
and Chromecast-like adapter flows should keep the same Public/Admin wire
behavior and redaction semantics while code depends on a clearer planner
boundary.

Closed on 2026-05-28 after replacing the old execution-shaped decision payload
with `PlaybackRenditionPlan`, moving transcode profile generation onto
`PlaybackTargetProfile`, deleting `PlaybackProfile`, and passing focused
planner/API/server playback gates.
