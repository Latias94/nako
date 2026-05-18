# Android Client Foundation

Status: Proposed
Last updated: 2026-05-18

This workstream tracks the Android-first implementation lane for Taru's native
client architecture. It turns ADR 0026 into a concrete Android foundation while
preserving iOS as a peer future target.

Authoritative docs:

- [Design](DESIGN.md)
- [UX context](UX_CONTEXT.md)
- [Client interface direction](CLIENT_INTERFACE_DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

## Current Direction

Android is the first implementation target, not the product strategy. Taru's
flagship client architecture remains native iOS and native Android shells with
a shared Rust client core where practical.

The first Android client should prioritize playback and the smallest useful
media-library browsing loop:

- connect to a Taru server;
- authenticate with the Public Client API;
- browse Media Libraries and Media Items;
- show Managed Artwork when the public route exists;
- request Playback Source Selection through playback decision APIs;
- play direct, remux, or HLS outputs through Media3 ExoPlayer;
- expose basic playback/session errors in user-facing language.

Server administration, metadata editing, NFO workflows, addon management,
webhook/automation configuration, and advanced transcode policy management are
out of the first Android client slice.
