# playback db write pressure and wal policy

## Goal

Prove that playback-related writes do not create unacceptable SQLite lock
contention on the self-hosted default path, and make the WAL / busy-timeout /
connection-pool policy explicit enough to keep that behavior stable.

## What I already know

* On-disk SQLite already uses WAL, a 10s busy timeout, foreign keys, and up to
  8 pooled connections in `crates/nako-db/src/sqlite/runtime.rs`.
* In-memory SQLite stays on a single connection.
* Playback writes already exist in heartbeat/session state, transcode runtime
  metrics, job leases, and cleanup flows.
* `docs/architecture/STATE_ACCESS.md` and `docs/architecture/PLAYBACK.md`
  already call out playback write pressure as a follow-on lane.

## Assumptions

* The first slice should keep the schema stable.
* The first slice should not add a new queueing system or durable job layer.
* If the current write cadence is already acceptable, the task should codify
  and test that behavior instead of inventing extra indirection.

## Requirements

* Make the SQLite runtime policy for playback-relevant workloads explicit and
  testable.
* Add focused pressure coverage for concurrent playback heartbeat and session
  writes.
* Cover the transcode runtime metric write path if it participates in the
  same lock contention profile.
* Keep PostgreSQL parity intact for any shared repository behavior that changes.
* Update architecture and operations docs to state the operator expectation for
  SQLite write pressure.

## Acceptance Criteria

* [ ] The SQLite runtime policy is covered by tests for WAL, busy timeout, and
      connection limits.
* [ ] Focused tests exercise concurrent playback write activity without flaky
      lock contention.
* [ ] Any write-path throttling or coalescing added for this slice is covered
      by a regression test.
* [ ] Architecture docs explain the playback write-pressure expectation and the
      SQLite runtime policy.
* [ ] The relevant `nako-db` and `nako-server` test gates pass.

## Definition of Done

* Tests added or updated.
* `cargo fmt --all` passes for touched Rust code.
* Focused `cargo nextest` gates pass.
* Docs updated where the operational expectation changed.

## Technical Approach

Start from the existing SQLite runtime policy in `nako-db`, then pressure-test
the actual playback write paths that run under self-hosted load. Keep the work
small enough that it can prove the current policy or justify a narrower write
reduction in the hot path, without widening the architecture.

## Out of Scope

* New playback queueing or background write worker architecture.
* Schema redesign.
* Remote worker support.
* API surface changes unrelated to write pressure.

## Technical Notes

* `crates/nako-db/src/sqlite/runtime.rs`
* `crates/nako-db/src/sqlite/playback.rs`
* `crates/nako-db/src/postgres/playback_runtime.rs`
* `crates/nako-server/src/app/playback/mod.rs`
* `crates/nako-server/src/app/playback/hls.rs`
* `docs/architecture/STATE_ACCESS.md`
* `docs/architecture/PLAYBACK.md`
* `docs/architecture/OPERATIONS_RELEASE.md`
