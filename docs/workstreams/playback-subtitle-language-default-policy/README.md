# Playback Subtitle Language Default Policy

Status: Completed
Last updated: 2026-05-30

This workstream shipped the first request-scoped subtitle language/default
selection policy after subtitle sidecar serving, HLS subtitle renditions, and
audio language defaults shipped.

The shipped slice is request-scoped and HLS-visible: explicit subtitle stream
selection still wins, and when a user/client supplies preferred subtitle
languages, Nako chooses a matching source subtitle stream and marks the matching
HLS subtitle rendition as default. Persisted user profile settings, player UI
controls, subtitle burn-in, ASS/SSA shaping, OCR, addon late-subtitle readiness,
and DASH/LL-HLS behavior are follow-ons.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `CLOSEOUT.md`
- `HANDOFF.md`
