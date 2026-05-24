# Cross-Repo Fearless Boundary Alignment - Evidence And Gates

Status: Active
Last updated: 2026-05-24

## Current Evidence

Planning evidence:

- `CONTEXT.md`
- `docs/adr/0001-modular-monolith-rust-workspace.md`
- `docs/adr/0019-server-architecture-hardening-boundaries.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/workstreams/fearless-architecture-deepening/`
- `docs/workstreams/fearless-future-architecture-refactor/`
- `docs/workstreams/addon-architecture-deepening/`
- `../nako-official-addons/docs/workstreams/official-metadata-addon-fearless-refactor/`
- `../nako-official-addons/docs/workstreams/official-metadata-addon-side-effect-writer/`
- `../nako-official-addons/docs/workstreams/official-metadata-addon-provider-relevance-budget/`

Observed risk anchors:

- `crates/nako-db/src/facade.rs`
- `crates/nako-library/src/ingestion.rs`
- `crates/nako-library/src/ingestion/source_commit.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/app/acquisition_intake.rs`
- `crates/nako-server/src/app/job_runtime.rs`
- `crates/nako-metadata/src/strategy.rs`
- `crates/nako-transcode/src/hls.rs`
- `crates/nako-server/src/app/composition.rs`
- `../nako-official-addons/crates/nako-metadata-scraper/src/engine/mod.rs`
- `../nako-official-addons/crates/nako-metadata-scraper/src/nako_runtime.rs`
- `../nako-official-addons/crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `../nako-official-addons/crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `../nako-official-addons/crates/nako-metadata-scraper/src/providers/douban.rs`

Reference-product evidence, architecture only:

- Jellyfin local repo module layout under `repo-ref/jellyfin`.
- Jellyfin provider, local metadata, XBMC/NFO metadata, naming, media encoding,
  and database project boundaries.
- Plex public product behavior only; no implementation assumptions.

## Gate Set

### Planning Gate

Use before implementation:

```powershell
git status --short
```

Run in both:

- `F:/SourceCodes/Rust/nako`
- `F:/SourceCodes/Rust/nako-official-addons`

Record unrelated dirty files in `HANDOFF.md`. Do not restore, delete, format,
or stage unrelated files.

### Server Targeted Gate

Use a focused gate for the touched server package:

```powershell
cargo nextest run -p <package> <test-filter> --no-fail-fast
```

Examples:

```powershell
cargo nextest run -p nako-library ingestion --no-fail-fast
cargo nextest run -p nako-metadata metadata_refresh --no-fail-fast
cargo nextest run -p nako-db <repository-or-contract-filter> --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
```

### Official Addon Targeted Gate

Run from `F:/SourceCodes/Rust/nako-official-addons`:

```powershell
cargo nextest run -p nako-metadata-scraper metadata writeback artwork ranking --no-fail-fast
```

For provider adapter splits, add the provider filter:

```powershell
cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper douban ranking title --no-fail-fast
```

### Formatting Gate

Use when practical and safe for the touched repository:

```powershell
cargo fmt --all -- --check
```

If unrelated dirty files exist, inspect diffs before any formatting command
that may rewrite files. Never revert unrelated formatting changes without user
approval.

### Broader Closeout Gate

Use proportional closeout gates:

```powershell
cargo nextest run --workspace --no-fail-fast
```

If the full workspace is too slow or blocked by unrelated work, record the
reason and run package-level gates for every touched package.

### Whitespace Gate

Prefer path-scoped checks when unrelated dirty files exist:

```powershell
git diff --check -- docs/workstreams/cross-repo-fearless-boundary-alignment
```

For implementation tasks, include only the touched paths unless the worktree is
clean.

### Review Gate

Use `review-workstream` before accepting a completed implementation task.
Record:

- blocking findings;
- missing gates;
- residual risks;
- follow-on splits.

Use `verify-rust-workstream` before marking this lane complete.

## Notes

- Reference repositories under `repo-ref/` are never evidence for copied code.
  They are only evidence for mature boundary shape and product workflow
  patterns.
- Fresh verification is required before marking any implementation task done.
- Cross-repo work should prefer one repository per worker unless a task is
  explicitly about a public contract between the repositories.

## Recent Server Evidence

- 2026-05-24 CRFBA-020: `cargo nextest run -p nako-server acquisition_intake
  --no-fail-fast` passed 6 acquisition-intake focused tests after narrowing
  `crates/nako-server/src/app/acquisition_intake.rs` behind
  `AcquisitionIntakeWorkflowStore`. `cargo check -p nako-server --bin
  nako-server` also passed.
- 2026-05-24 CRFBA-020 follow-up: `cargo nextest run -p nako-server
  durable_job_runtime --no-fail-fast` passed 4 durable-job-runtime tests after
  narrowing `crates/nako-server/src/app/job_runtime.rs` behind
  `DurableJobLeaseStore`. `cargo fmt --all -- --check` also passed.

## Recent Addon Evidence

- 2026-05-24 CRFBA-050: `cargo nextest run -p nako-metadata-scraper metadata
  writeback artwork ranking --no-fail-fast` passed 49 tests after the runtime
  split into `query`, `orchestration`, `response`, `runtime`, `writeback`, and
  `bulk` modules.
- 2026-05-24 CRFBA-060: `cargo nextest run -p nako-metadata-scraper tmdb --no-
  fail-fast` passed 33 TMDB-focused tests after the provider split into
  `client`, `search`, `parser`, `mapper`, `enrichment`, and `test_support`
  modules.
- 2026-05-24 package verification: `cargo nextest run -p nako-metadata-scraper
  --no-fail-fast` passed 141 tests across the package after both addon slices
  were split.
- 2026-05-24 formatting verification: `cargo fmt --all -- --check` passed
  after formatting the touched addon files.
