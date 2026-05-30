# Remote Storage Health And Circuit Breaker - Evidence And Gates

Status: Active
Last updated: 2026-05-30

## Required Gates

```text
python -m json.tool docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json
cargo nextest run -p nako-db storage_backend_health --no-fail-fast
cargo nextest run -p nako-server storage_health --no-fail-fast
cargo nextest run -p nako-server admin_v1_storage --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Run focused package gates first. Broaden only when a task changes shared
storage, playback staging, Admin DTOs, schema migrations, or generated client
contracts.

## Evidence Ledger

### RSHC-010 - Scope and evidence freeze

Status: Done

Evidence:

- `docs/workstreams/remote-storage-health-and-circuit-breaker/DESIGN.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/TODO.md`
- `docs/workstreams/remote-storage-health-and-circuit-breaker/WORKSTREAM.json`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Notes:

- The first executable task is repository parity, not server runtime policy.
- Playback staging and Admin reset work are deliberately sequenced after the
  durable health contract.

## Residual Risks

- Mount-like local paths can still hang below the OS boundary. Circuit-breaker
  state should reduce repeated work admission, not claim to preempt every
  blocking syscall.
- Backend-scoped health may be too coarse for rare source-specific corruption.
  Split a follow-on only after evidence proves source-scoped suppression is
  needed.
- Admin reset can hide an active incident if it is not paired with clear
  diagnostics and updated timestamps.
