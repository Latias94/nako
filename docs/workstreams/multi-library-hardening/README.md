# Multi-Library Hardening

Status: Completed
Last updated: 2026-05-18

This workstream owns Taru's remaining Media Library source-of-truth hardening
after the M8 correctness baseline. M8 made `(library_id, locator)` the natural
source identity and made CLI operations explicit, but startup reconciliation
and persisted Library authority still need a focused execution lane before more
client, addon, and remote-library behavior depends on them.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `PHASE8_0_CORRECTNESS_BASELINE.md`
