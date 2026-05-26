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

Recommended default direction: short-lived playback tickets, unless BPAT-010
proves a better transport.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

First executable task: BPAT-010.
