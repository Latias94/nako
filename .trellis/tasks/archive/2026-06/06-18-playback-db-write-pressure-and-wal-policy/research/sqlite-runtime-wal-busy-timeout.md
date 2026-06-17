# Research: SQLite runtime policy for playback-related workloads

- Query: Nako `nako-db` current SQLite runtime policy for playback-related workloads, including WAL, busy timeout, pool behavior, related architecture/ADR guidance, and contract-test coverage.
- Scope: mixed
- Date: 2026-06-18

## Findings

- `nako-db` already hard-codes a clear SQLite runtime split: on-disk databases use WAL, a 10s busy timeout, foreign keys, and up to 8 pooled connections; in-memory databases use a single connection and no WAL/synchronous tuning. See `crates/nako-db/src/sqlite/runtime.rs:18-19`, `crates/nako-db/src/sqlite/runtime.rs:27-39`, `crates/nako-db/src/sqlite/runtime.rs:44-47`, `crates/nako-db/src/sqlite/runtime.rs:57-89`.
- The runtime is selected by URL shape. `connect()` treats `:memory:` and `mode=memory` as in-memory; everything else gets the on-disk policy. The facade also exposes `DatabaseConnectOptions::sqlite_with_runtime(...)` for explicit runtime policy injection. See `crates/nako-db/src/sqlite/runtime.rs:57-79`, `crates/nako-db/src/backend.rs:48-83`, `crates/nako-db/src/facade.rs:205-223`.
- The runtime test already proves the important on-disk PRAGMAs: `journal_mode = wal`, `busy_timeout = 10000`, and `foreign_keys = 1`. The in-memory test proves the single-connection policy. See `crates/nako-db/src/sqlite/runtime.rs:112-153`.
- PostgreSQL parity is explicit but separate. `PostgresStore` uses its own pool limit (`POSTGRES_MAX_CONNECTIONS = 5`) and migration lifecycle, so the SQLite policy is not shared or inferred from Postgres. See `crates/nako-db/src/postgres.rs:32-33`, `crates/nako-db/src/postgres.rs:84-120`, `crates/nako-db/src/postgres.rs:154-158`.
- Playback write paths are real and frequent. In `nako-db`, playback sessions update `last_heartbeat_at_ms` and `updated_at_ms` on every heartbeat, and transcode sessions rewrite `runtime_metrics_json` on metrics updates. In the server layer, playback heartbeats are surfaced through `record_playback_session_heartbeat(...)`, and transcode metrics are updated from the playback flow. See `crates/nako-db/src/sqlite/playback.rs:289-319`, `crates/nako-db/src/sqlite/playback.rs:541-559`, `crates/nako-db/src/postgres/playback_runtime.rs:335-365`, `crates/nako-db/src/postgres/playback_runtime.rs:571-589`, `crates/nako-server/src/app/playback/mod.rs:1107-1143`, `crates/nako-server/src/app/playback/hls.rs:432`.
- The architecture docs already identify this as a follow-on lane. `STATE_ACCESS.md` names `playback-db-write-pressure-and-wal-policy` and explicitly scopes WAL/busy-timeout policy, connection pool sizing, heartbeat write coalescing, transcode metric update frequency, and concurrent playback/scan write pressure. `PLAYBACK.md` says playback-specific pressure tests should verify WAL behavior, busy timeouts, pool sizing, and transaction scope. See `docs/architecture/STATE_ACCESS.md:43-54`, `docs/architecture/PLAYBACK.md:351-353`.
- ADR 0030 records the current persistence contract: SQLite runtime policy enables foreign keys, WAL for on-disk DBs, a busy timeout, and a one-connection in-memory test pool; SQL dialect differences stay in backend adapters; contract tests are the behavioral authority. See `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md` and `docs/adr/0029-postgresql-ready-persistence-boundary.md:68`.
- The `nako-db` spec agrees with that boundary: SQLite is the default runtime target, PostgreSQL parity is adapter-shaped, migrations must be registered in both backends, and repository behavior should be covered by contract tests with fresh-store and migrated-store cases. See `.trellis/spec/nako-db/backend/index.md:4,12-17,25-33,39`, `.trellis/spec/nako-db/backend/database-guidelines.md:27-37,46-71,97-102`, `.trellis/spec/nako-db/backend/quality-guidelines.md:9-10`.
- Existing contract coverage is strong for playback state semantics, but it is not a direct SQLite write-pressure test. Playback runtime contracts already cover principal-scoped playback state, playback profile preference CRUD, named playback profile defaulting, playlist/list projections, transcode session lifecycle, playback session lifecycle/admission, and related invariants. See `crates/nako-db/src/contract_tests.rs:421-453`, `crates/nako-db/src/contract_tests.rs:5232-5480`, `crates/nako-db/src/contract_tests.rs:7497-7973`, `crates/nako-db/src/contract_tests.rs:12968-13090`.
- There are some pressure-related contracts elsewhere in `nako-db` (for example job queue pressure summaries), but they do not prove playback heartbeat/session writes under concurrent SQLite lock pressure. See `crates/nako-db/src/contract_tests.rs:1400-1413`.

## Files Found

- `crates/nako-db/src/sqlite/runtime.rs` - SQLite runtime policy, connection setup, and runtime tests.
- `crates/nako-db/src/backend.rs` - public database connect options and backend kind selection.
- `crates/nako-db/src/facade.rs` - facade dispatch between SQLite and Postgres backends.
- `crates/nako-db/src/sqlite/migrations.rs` - SQLite migration registration and migration-shape tests.
- `crates/nako-db/src/postgres.rs` - PostgreSQL pool policy and migration lifecycle.
- `crates/nako-db/src/sqlite/playback.rs` - playback session and transcode runtime write paths.
- `crates/nako-db/src/postgres/playback_runtime.rs` - PostgreSQL playback runtime write paths.
- `crates/nako-db/src/contract_tests.rs` - backend-neutral repository contracts, including playback runtime and session lifecycle.
- `crates/nako-server/src/app/playback/mod.rs` - server playback heartbeat entry points.
- `crates/nako-server/src/app/playback/hls.rs` - transcode metric update call site.
- `docs/architecture/STATE_ACCESS.md` - active lane and scope for this work.
- `docs/architecture/PLAYBACK.md` - playback pressure-risk note.
- `docs/adr/0029-postgresql-ready-persistence-boundary.md` - persistence boundary and contract-test authority.
- `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md` - SQLite runtime policy baseline.
- `.trellis/spec/nako-db/backend/index.md` - crate-level backend guidance.
- `.trellis/spec/nako-db/backend/database-guidelines.md` - migration and SQL rules.
- `.trellis/spec/nako-db/backend/quality-guidelines.md` - contract-test and migration gate rules.

## Code Patterns

- SQLite runtime policy is encoded in `SqliteRuntimeOptions::on_disk()` and `SqliteRuntimeOptions::in_memory()`, then applied through `SqliteConnectOptions` with `foreign_keys(true)`, `busy_timeout(...)`, and optional WAL/synchronous tuning. See `crates/nako-db/src/sqlite/runtime.rs:18-47`.
- In-memory detection is URL-pattern based, not a separate config flag. See `crates/nako-db/src/sqlite/runtime.rs:57-79`, `crates/nako-db/src/sqlite/runtime.rs:101-102`.
- The facade lets callers pass explicit SQLite runtime policy when needed, but the common path still defaults to the current hard-coded split. See `crates/nako-db/src/backend.rs:48-83`, `crates/nako-db/src/facade.rs:205-223`.
- Playback heartbeats are implemented as row updates that touch both state and timestamp fields in one write. See `crates/nako-db/src/sqlite/playback.rs:289-319` and `crates/nako-db/src/postgres/playback_runtime.rs:335-365`.
- Transcode runtime metrics are serialized as JSON and written back as a single update. See `crates/nako-db/src/sqlite/playback.rs:541-559` and `crates/nako-db/src/postgres/playback_runtime.rs:571-589`.
- Playback runtime contract tests already validate behavior across SQLite/Postgres for session state, profile preferences, named profiles, playlist semantics, transcode sessions, and session admission. See `crates/nako-db/src/contract_tests.rs:5232-5480`, `crates/nako-db/src/contract_tests.rs:7497-7973`, `crates/nako-db/src/contract_tests.rs:12968-13090`.

## External References

- None used. This research stayed inside the repository and project docs.

## Related Specs

- `.trellis/spec/nako-db/backend/index.md`
- `.trellis/spec/nako-db/backend/database-guidelines.md`
- `.trellis/spec/nako-db/backend/quality-guidelines.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/PLAYBACK.md`
- `docs/adr/0029-postgresql-ready-persistence-boundary.md`
- `docs/adr/0030-postgresql-ready-sql-dialect-and-migration-policy.md`

## Caveats / Not Found

- I did not find a dedicated concurrent SQLite lock-contention or write-pressure test for playback heartbeats/transcode metrics.
- Existing contract tests prove repository behavior and migration correctness, but not the specific runtime contention profile under concurrent playback writes.
- The current policy looks sufficient as a baseline for self-hosted default SQLite usage: WAL + 10s busy timeout + bounded pool + single-connection in-memory tests is a coherent default, and the docs already classify pressure hardening as a follow-on. The task should still tighten further if its goal is to prove or tune real concurrent playback write pressure, because the current evidence does not yet show that the existing defaults are enough under load.
