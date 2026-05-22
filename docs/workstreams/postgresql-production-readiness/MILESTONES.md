# PostgreSQL Production Readiness — Milestones

Status: Completed
Last updated: 2026-05-20

## M0 — Scope, Matrix, And Evidence Baseline

Status: completed.

Exit criteria:

- M62 workstream docs exist and agree.
- Current M61 proof state is recorded.
- Contract-test matrix is prioritized.
- First executable task is chosen.

Primary evidence:

- `DESIGN.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`

## M1 — Lifecycle, Backend Selection, And Verification Harness

Status: completed.

Exit criteria:

- Production code can represent SQLite and PostgreSQL backend kind explicitly.
  Completed by PGR-020.
- `NakoDatabase` can select the active backend without exposing concrete
  adapters to server code.
  Completed by PGR-020 and promoted during PGR-120: PostgreSQL now enters a
  real runtime connection path and the facade dispatches through an internal
  backend adapter trait instead of a SQLite-only field.
- The backend contract-test harness can run multiple contract families against
  SQLite always-on and PostgreSQL opt-in.
  Completed structurally by PGR-030 for lifecycle and job-lease families; new
  families can now be added through paired contract cases. PGR-040 and PGR-050
  have reused the harness for library/media and scan-commit families. PGR-060
  has reused the same harness for metadata/catalog commit contracts. PGR-120
  added runtime-promotion and VFS/Staging families, and the full PostgreSQL
  ignored contract gate now fails fast without `NAKO_TEST_POSTGRES_URL`.
- Local PostgreSQL test setup and opt-in commands are documented in
  `EVIDENCE_AND_GATES.md` and `WORKSTREAM.json`.

Primary gates:

- `cargo check -p nako-db --tests`
- `cargo check -p nako-server --tests`
- focused `cargo nextest run -p nako-db contract --no-fail-fast`
- `git diff --check`

## M2 — Core Repository And Workflow Contracts

Status: completed.

Exit criteria:

- Media Library, Media Item, Media Source, scan commit, metadata/catalog
  commit, and search projection behavior are proven by backend-neutral
  contracts.
  Media Library, Media Item, Media Source, and Library Item State identity
  contracts completed by PGR-040. Library scan commit, Source State, Local
  Inference Evidence, Media Technical Facts, ingestion failure resolution, and
  Search Projection side-effect contracts completed by PGR-050. Metadata
  refresh, Provider Mapping, provider raw response/attempt, NFO import, full
  Catalog Item Graph replacement, Search Projection, and rollback contracts
  completed by PGR-060.
- PostgreSQL migrations cover all tables needed by those contracts.
  PostgreSQL migrations now cover the PGR-040 library/media tables and the
  PGR-050 scan/probe/source-state/search/evidence/ingestion tables. PGR-060
  added metadata/provider mapping/provider attempt and Catalog Item Graph
  tables for people, credits, genres, tags, collections, studios, and image
  assets. PGR-120 also corrected provider raw responses to text storage where
  byte-for-byte raw payload round trips are part of the contract.
- SQLite remains the reference always-on backend.

Primary gates:

- `cargo check -p nako-db --tests`
- `cargo check -p nako-library --tests`
- `cargo check -p nako-metadata --tests`
- `cargo check -p nako-catalog --tests`
- focused nextest for contract families
- optional PostgreSQL contract run when `NAKO_TEST_POSTGRES_URL` is available

## M3 — Runtime State And Operational Contracts

Status: completed.

Exit criteria:

- User Playback State and Transcode Session lifecycle are backend-neutral.
  Completed by PGR-070 with `PlaybackRuntime` contracts and PostgreSQL
  repository parity.
- Event outbox, webhooks, Addons, and Automation Provider state are either
  PostgreSQL-ready or explicitly disabled/split for PostgreSQL runtime.
  Completed by PGR-080 with `EventAddonAutomation` contracts and PostgreSQL
  repository parity.
- VFS cache and staging manifest state are PostgreSQL-ready for the supported
  playback/storage startup scope.
  Completed during PGR-120 with `VfsStaging` contracts, PostgreSQL migration
  parity, repository parity, and PostgreSQL `vfs_cache` capability promotion.
- Managed Artwork parity is either implemented or split with named expiry
  gates.
  Completed by PGR-090 as the named follow-on
  `docs/workstreams/managed-artwork-postgresql-parity/`; runtime support must
  stay explicitly disabled or diagnostic-gated until that follow-on proves
  parity.

Primary gates:

- `cargo check -p nako-db --tests`
- `cargo check -p nako-server --tests`
- focused nextest for runtime state families
- `git diff --check`

## M4 — Runtime Diagnostics, Assumption Cleanup, And Closeout

Status: completed.

Exit criteria:

- Safe Admin/config diagnostics expose backend kind and migration state without
  credentials or raw database details.
  Completed by PGR-100 with a sanitized database diagnostics block, startup
  migration status, active/backend configured kind reporting, URL-scheme-only
  reporting, and Admin DTO/contract/docs redaction tests.
- SQLite-only assumptions above adapter seams are deleted or documented as
  follow-ons.
  PGR-110 removed facade-level implicit SQLite constructors and deleted the
  remaining facade-test imports of SQLite row codecs/direct pool inspection.
  Remaining SQLite SQL dialect, row codec, PRAGMA, and migration assumptions
  are isolated under `nako-db::sqlite` or SQLite-owned tests. Remaining
  `sqlite::memory:` values above adapters are test fixture data, not production
  backend-selection logic.
- Final SQLite workspace gates pass.
- PostgreSQL opt-in gates pass with `NAKO_TEST_POSTGRES_URL` set to a local
  test PostgreSQL URL.
- Goal/workstream/roadmap docs reflect the shipped PostgreSQL scope.

Primary gates:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace --no-fail-fast`
- PostgreSQL contract nextest with `NAKO_TEST_POSTGRES_URL`, when available
- `git diff --check`
