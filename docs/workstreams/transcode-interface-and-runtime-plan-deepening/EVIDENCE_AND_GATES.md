# Transcode Interface And Runtime Plan Deepening - Evidence And Gates

Status: Active
Last updated: 2026-05-31

## Required Gates

```text
python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-transcode remux --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused transcode planning gates before the server HLS gate. Broaden only
when the implementation touches shared playback runtime behavior.

## Evidence Ledger

### TIRP-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/DESIGN.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/TODO.md`
- `docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/architecture/LANES.md`
- `python -m json.tool docs/workstreams/transcode-interface-and-runtime-plan-deepening/WORKSTREAM.json`
  - 2026-05-31: Passed.
- `git diff --check -- docs/workstreams/transcode-interface-and-runtime-plan-deepening docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/architecture/LANES.md docs/workstreams/README.md`
  - 2026-05-31: Passed with only Windows line-ending warnings.

Notes:

- The first executable task is HLS runtime plan Interface deepening.
- HDR tone mapping, broad hardware capability matrices, HLS lifecycle
  consolidation, and resource admission unification are outside this first
  workstream unless the planner splits a task explicitly.

## Residual Risks

- The first implementation may reveal that a small server adapter still needs
  to translate playback-owned values into transcode-owned values. Keep that
  adapter thin; do not make `nako-transcode` depend on `nako-playback` without
  planner review.
- Tightening `pub use` can break tests or downstream internal callers. Ratchet
  exports only after the higher-level Interface exists.
- HLS lifecycle and resource admission remain separate shallow areas; do not
  solve them opportunistically in `TIRP-020`.
