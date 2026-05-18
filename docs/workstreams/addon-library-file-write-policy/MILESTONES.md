# Addon Library File Write Policy Milestones

Status: Proposed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Outcome: addon-initiated Library File Write behavior is split from APW with
clear authority and gates.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- APW closeout points to this lane.
- Workstream index links the new lane.

Primary evidence:

- `docs/workstreams/addon-library-file-write-policy/DESIGN.md`
- `docs/workstreams/addon-protected-writes/HANDOFF.md`

## M1 - File Write Seam Audit

Outcome: NFO, subtitle, VFS, backup, and Addon Side Effect seams are classified
before accepting file-write payloads.

Exit criteria:

- NFO Round Trip, NFO export, storage/VFS write modes, backup retention, and
  Addon Side Effect apply outcome boundaries are inventoried.
- The first file-write target is selected with risk notes.
- ADR amendment need is accepted, rejected, or split.

Primary gates:

- `rg -n "Library File Write|subtitle|NFO|nfo|StorageWriteRequest|StorageWriteReport|StorageBackupPolicy|atomic_replace|backup|sidecar" crates docs`
- `git diff --check`

## M2 - First File Write Apply Slice

Outcome: one accepted addon file-write side effect can safely write through
Taru-owned NFO/storage/VFS seams.

Exit criteria:

- The payload is normalized into a bounded Taru Library File Write command.
- Target derivation avoids raw paths and Source Locator leakage.
- Backup/write report behavior is redacted and tested.
- Idempotency replay after write outcome is safe.

Primary gates:

- focused NFO/storage/addon tests
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-nfo -p taru-vfs --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 - Closeout Or Split

Outcome: the first file-write path is complete enough to close, or subtitle,
NFO, and arbitrary sidecar breadth is split.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped behavior.
- Remaining subtitle/NFO/sidecar breadth is completed, deferred, or split.
