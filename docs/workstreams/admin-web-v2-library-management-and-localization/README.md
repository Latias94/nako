# Admin Web V2 Library Management And Localization

Status: Closed
Last updated: 2026-05-25

This workstream turns the read-only Admin Web V2 Media Libraries route into the
first durable library-management workflow and establishes the first UI
localization boundary for Admin Web.

Authoritative docs:

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [PARITY_GAP_SPLIT.md](PARITY_GAP_SPLIT.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [HANDOFF.md](HANDOFF.md)
- [CLOSEOUT.md](CLOSEOUT.md)

## Why This Lane Exists

`/libraries` now proves route-first rendering, but it does not yet let an
operator manage a Media Library the way Jellyfin/Plex-style web consoles make
library setup, metadata language, source inventory, scans, NFO actions, and
library health visible.

This lane keeps Admin Web administration-first. It adds library management and
localization foundations without turning Admin Web into the flagship playback
client.
