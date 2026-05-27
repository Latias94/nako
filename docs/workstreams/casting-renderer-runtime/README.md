# Casting Renderer Runtime

Status: Closed
Last updated: 2026-05-27

This workstream implements casting now that playback policy and renderer target
semantics are in place. It treats casting as renderer sessions plus protocol
adapters, starting with Nako-to-Nako casting before Chromecast, DLNA, or AirPlay.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `ADAPTER_FOLLOW_ONS.md`
- `CLOSEOUT.md`
- `WORKSTREAM.json`
- `docs/adr/0040-casting-as-renderer-session-adapter.md`

Outcome: Nako-to-Nako casting is implemented through Renderer Sessions,
Renderer Commands, policy-checked play command flow, Public Client renderer
routes, and redaction-safe Admin renderer diagnostics. Chromecast, DLNA,
AirPlay, and non-direct cast-safe renderer transport are split to follow-ons in
`ADAPTER_FOLLOW_ONS.md`.
