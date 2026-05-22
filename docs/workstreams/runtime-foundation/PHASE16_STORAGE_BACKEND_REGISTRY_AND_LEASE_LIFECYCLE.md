# Phase 16: Storage Backend Registry And Lease Lifecycle

## Objective

M16 turns storage backend construction, remote staging, and staged-input leases
into explicit runtime boundaries. Nako is still pre-release, so the phase may
remove older ad hoc paths rather than preserve compatibility with incomplete
MVP behavior.

## Deliverables

- `NakoApp` owns a `StorageBackendRegistry`.
- The registry caches one `LibraryStorageBackend` per configured `library_id`.
- Scan, probe, direct play, remux/HLS input staging, and NFO import/export
  resolve storage through the same library-aware registry boundary.
- Staging uses manifest-backed reservation, staging, completion, failure,
  expiration, deletion, and lease transitions.
- Active staging leases protect cleanup and are released explicitly after
  playback or by a drop-time fallback if a future is cancelled.
- Storage backend diagnostics expose process-local registry and health state
  through explicit API DTOs.

## Runtime Shape

`StorageBackendRegistry` is process-local. It builds storage backend wrappers
from server configuration, caches them by `library_id`, and attaches per-library
remote stream/stage semaphores and health counters. WebDAV backends remain
wrapped by the VFS cache layer; local backends remain rooted at their configured
filesystem root.

Diagnostics use sanitized DTOs and intentionally avoid returning local root
paths, WebDAV base URLs, usernames, passwords, or resolved secret values. The
diagnostic state is not distributed and does not claim multi-process health.

## Staging Lifecycle

The intended staging state machine is:

```text
reserved -> staging -> ready -> leased -> ready -> expired -> deleted
reserved -> staging -> failed -> expired -> deleted
```

`active_leases` is the cleanup guard. Cleanup candidates must have no active
leases and must be expired. Reservation and budget accounting happen in the
database before bytes are materialized, and the manifest uses `reserved` before
the backend download starts. Concurrent staging attempts cannot independently
overrun the configured budget in a single process.

## Validation

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- `cargo nextest run --workspace`
- `git diff --check`

Targeted tests:

- storage backend registry instance reuse;
- storage backend health counter updates;
- `/storage/backends` sanitized diagnostics;
- staging reservation, completion, failure, and budget behavior;
- cleanup preservation for active leases;
- cleanup removal for expired pending reservations;
- lease ready/leased/ready transitions;
- dropped lease fallback release.
