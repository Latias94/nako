# Browser Playback Auth Transport

Status: Active
Last updated: 2026-05-26

This workstream owns the secure browser playback transport needed before Media
Web can render a real browser media element.

It follows `media-web-client-foundation`, which deliberately shipped a safe
watch shell without minting stream URLs. This lane decides and implements the
transport that lets browser playback respect Library Access, playback-session
policy, range requests, and User Playback State without exposing privileged
permanent URLs.

Accepted direction after BPAT-010: short-lived browser playback tickets, as
recorded in ADR 0036.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

Next executable task: BPAT-020.
