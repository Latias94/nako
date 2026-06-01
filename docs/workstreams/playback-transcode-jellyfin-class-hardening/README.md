# Playback Transcode Jellyfin-Class Hardening

Status: Closed
Last updated: 2026-06-01
Lane: `playback-transcode`
Current task: none

This workstream freezes the playback/transcode Interfaces, owned scopes,
shared scopes, validation gates, and first parallel worker prompts needed to
deepen Nako toward Jellyfin/Plex-class playback and transcode behavior without
collapsing the current typed architecture boundaries.

It is an architecture coordination lane first. `PTJCH-010`, `PTJCH-020`, the
first parallel Rust implementation batch, `PTJCH-210`, `PTJCH-220`,
`PTJCH-310`, and `PTJCH-390` are complete. HLS artifact I/O pressure is split
to the existing `proposed:hls-artifact-io-pressure-enforcement` follow-on
rather than remaining inside this workstream.

Authoritative files:

- `DESIGN.md` - seam map, scope, stop conditions, and target state.
- `TODO.md` - task ledger and parallel lane sequencing.
- `MILESTONES.md` - milestone checkpoints and closeout criteria.
- `EVIDENCE_AND_GATES.md` - required evidence and validation commands.
- `CONTEXT.jsonl` - documents and reference material to read before work.
- `WORKER_PROMPTS.md` - first-batch prompts for parallel Codex terminals.
- `WORKSTREAM.json` - machine-readable status and lane metadata.
- `HANDOFF.md` - continuation notes for the next Codex terminal.
- `CLOSEOUT.md` - closeout decision, gates, follow-ons, and residual risks.
