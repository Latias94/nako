# Playback Audio Language Default Policy

Status: Active
Last updated: 2026-05-29

This workstream owns the first playback audio language/default selection policy
after HLS audio sidecars and selected-main-audio cleanup shipped.

The first slice should be request-scoped and HLS-visible: explicit audio stream
selection still wins, but when a user/client supplies preferred audio
languages, Nako should choose a matching source audio stream and mark the
matching HLS audio rendition as default. Persisted user profile settings and UI
preferences are follow-ons.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
