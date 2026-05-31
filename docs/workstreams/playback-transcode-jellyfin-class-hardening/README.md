# Playback Transcode Jellyfin-Class Hardening

Status: Active
Last updated: 2026-05-31
Lane: `playback-transcode`
Current task: `PTJCH-220`

This workstream freezes the playback/transcode Interfaces, owned scopes,
shared scopes, validation gates, and first parallel worker prompts needed to
deepen Nako toward Jellyfin/Plex-class playback and transcode behavior without
collapsing the current typed architecture boundaries.

It is an architecture coordination lane first. `PTJCH-010`, `PTJCH-020`, the
first parallel Rust implementation batch, and `PTJCH-210` are complete. The
next work is `PTJCH-220` Playback Runtime ownership.

Authoritative files:

- `DESIGN.md` - seam map, scope, stop conditions, and target state.
- `TODO.md` - task ledger and parallel lane sequencing.
- `MILESTONES.md` - milestone checkpoints and closeout criteria.
- `EVIDENCE_AND_GATES.md` - required evidence and validation commands.
- `CONTEXT.jsonl` - documents and reference material to read before work.
- `WORKER_PROMPTS.md` - first-batch prompts for parallel Codex terminals.
- `WORKSTREAM.json` - machine-readable status and lane metadata.
- `HANDOFF.md` - continuation notes for the next Codex terminal.
