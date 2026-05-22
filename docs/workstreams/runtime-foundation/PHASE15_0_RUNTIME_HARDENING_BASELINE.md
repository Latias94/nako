# Phase 15.0: Runtime Hardening Baseline

## Goal

Create the planning boundary for runtime foundation hardening before changing
shared infrastructure code. The phase defines what belongs in M15, which MVP
shortcuts are allowed to be removed, and how implementation work should be
sequenced.

## Baseline Findings

SQLite runtime is still MVP-shaped:

- The SQLite pool currently uses a single connection.
- WAL and busy timeout behavior are not explicitly configured.
- Database concurrency policy is not documented for scan, playback, metadata
  maintenance, automation, webhook delivery, and lifecycle workers.

Migration execution is fragile:

- Migration SQL is currently split with string parsing.
- Complex future migrations with triggers or semicolons inside string literals
  can break this approach.
- Migrations are already applied in transactions, which should be preserved.

Secret redaction is inconsistent:

- TMDB has provider-specific redaction.
- Bangumi and Douban provider configs can carry resolved secrets in types that
  derive `Debug`.
- Provider headers and runtime config need a single redaction policy across
  config, diagnostics, jobs, and provider structs.

Hardware acceleration policy is not fully wired into the server runtime path:

- `nako-transcode` has capability and selection models.
- HLS service construction still primarily uses the requested accelerator.
- Resource budgets should follow the selected accelerator, not only the
  configured request.

## Refactor Rules

- Do not preserve legacy config paths solely for compatibility.
- Prefer replacing ad hoc helpers with focused shared types or modules.
- Delete old code after the new path is fully covered by tests.
- Keep `nako-server` as composition glue; move reusable runtime rules into
  narrower modules or crates.
- Avoid expanding public API exposure to internal domain models for new routes.

## Recommended Sequence

1. Harden SQLite connection and migration execution.
2. Introduce the shared secret redaction boundary.
3. Wire hardware capability selection into playback/transcode runtime.
4. Split runtime modules opportunistically where the touched code is already
   too broad.

## Validation

Each implementation phase should run:

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run --workspace
git diff --check
```

Focused tests should be added near the changed boundary instead of relying only
on broad integration tests.
