# Playback Subtitle Serving Milestones

## M1 - Sidecar Resolution

- Addon import write and playback read share sidecar leaf/URI derivation.
- Invalid stream facts fail safely without exposing storage paths.
- Status: Complete.

## M2 - Authorized Serving

- Sidecar subtitles are readable by source and stream index.
- Browse-only access and disabled media playback are rejected.
- Responses set subtitle content type and content length.
- Status: Complete.

## M3 - Browser Ticket

- Subtitle ticket URLs are opaque and scoped to source plus subtitle stream.
- Direct/remux/HLS tickets cannot be reused for subtitle reads.
- Status: Complete.

## M4 - Closeout

- Focused server/protocol tests pass.
- Remaining HLS/embedded subtitle work is split as follow-on.
- Status: Complete after final format and diff gates pass.
