# Casting Renderer Runtime

Status: Planned
Last updated: 2026-05-27

This workstream implements casting after playback policy and renderer target
semantics are in place. It treats casting as renderer sessions plus protocol
adapters, starting with Nako-to-Nako casting before Chromecast, DLNA, or AirPlay.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`
- `docs/adr/0040-casting-as-renderer-session-adapter.md`

First executable task after `playback-policy-and-renderer-targets` closes:
`CAST-020`, characterization and readiness tests for Playback Session,
browser-ticket, Public Client session heartbeat, and remote-control gaps.
