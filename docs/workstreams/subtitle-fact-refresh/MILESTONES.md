# Subtitle Fact Refresh Milestones

## M1 - Fact Model

- Imported subtitles appear as media probe subtitle streams.
- The fact includes language, format, role/disposition, and sidecar origin.
- Status: complete.

## M2 - Apply Integration

- Subtitle import apply refreshes the fact after write or already-applied
  detection.
- Existing probe streams are preserved.
- Repeated apply does not create duplicate sidecar streams.
- Status: complete.

## M3 - Visibility

- Public media stream DTOs expose origin and disposition.
- Responses remain free of paths, raw subtitle content, URLs, and backup URIs.
- Status: complete.

## M4 - Closeout

- Focused API/server tests pass.
- Remaining playback execution work is split.
- Status: complete.
