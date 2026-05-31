# Transcode Capability Inventory Matrix - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/transcode-capability-inventory-matrix/WORKSTREAM.json
cargo nextest run -p nako-transcode hardware --no-fail-fast
cargo nextest run -p nako-transcode probe --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused transcode capability gates only. Broaden to HLS/server gates only
if the worker reports an approved scope expansion, which should normally block
this workstream and return to planner coordination.

## Evidence Ledger

### TCIM-010 - Scope and evidence freeze

Status: Done

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Notes:

- This lane is safe to run beside HDR `HTP-030` only while it stays in
  `hardware.rs` / `probe.rs` inventory and report seams.
- Pipeline selection, FFmpeg command planning, server routes, API DTOs, and
  release packaging are explicit follow-ons.

## Residual Risks

- The lane improves capability observability, not actual playback format
  breadth.
- Host and Docker driver smoke evidence remains a release/operations follow-on.
