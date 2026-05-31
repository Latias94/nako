# Playback Compatibility Matrix Hardening - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/playback-compatibility-matrix-hardening/WORKSTREAM.json
cargo nextest run -p nako-playback compatibility --no-fail-fast
cargo nextest run -p nako-playback hdr audio --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused playback tests first. Broaden only if the implementation changes
shared playback planner behavior.

## Evidence Ledger

### PCMH-010 - Scope and evidence freeze

Status: Done

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/LANES.md`
- `docs/workstreams/README.md`

Notes:

- This lane is safe to run beside HDR `HTP-030` because it is scoped to
  `nako-playback`.
- Any need to edit `nako-transcode`, `nako-server`, API DTOs, or web/player
  behavior must return to planner coordination.

## Residual Risks

- The matrix will prove representative compatibility behavior, not every
  future device profile database row.
- The lane intentionally does not add executable FFmpeg tone mapping or audio
  filter behavior.
