# Playback Subtitle Language Default Policy

Status: Active
Last updated: 2026-05-30

This workstream owns the first request-scoped subtitle language/default
selection policy after subtitle sidecar serving, HLS subtitle renditions, and
audio language defaults shipped.

The first slice should be request-scoped and HLS-visible: explicit subtitle
stream selection still wins, but when a user/client supplies preferred subtitle
languages, Nako should choose a matching source subtitle stream and mark the
matching HLS subtitle rendition as default. Persisted user profile settings,
player UI controls, subtitle burn-in, ASS/SSA shaping, OCR, addon late-subtitle
readiness, and DASH/LL-HLS behavior are follow-ons.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
