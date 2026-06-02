# Error Handling

Library intake should classify failures into durable ingestion failure records
so operators can distinguish retryable storage/provider errors from structural
problems.

## Required Patterns

- Convert scan/probe backend errors through `failure.rs` helpers before
  persisting ingestion failures.
- Record scan failures with `IngestionFailurePhase::Scan`.
- Record probe failures with the probe phase and source/locator context.
- Resolve prior ingestion failures when a directory or source observation
  succeeds.
- Sort failure summaries deterministically before returning them.
- Do not panic on unsupported entries, stale cache, or missing probe facts.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| VFS stat/list fails during scan | Add `LibraryScanFailure` and continue |
| Directory observation succeeds | Upsert directory snapshot and resolve scan failure |
| Source missing from current scan | Tombstone through `ScanRepository` |
| Probe fails | Persist ingestion failure and include `LibraryProbeFailure` |
| Existing probe and `force == false` | Skip source |

## Wrong vs Correct

### Wrong

```rust
return Err(err);
```

### Correct

```rust
failures.push(scan_failure(uri, "directory", err));
```

Scanning should continue and preserve failure evidence when a bounded entry
fails.

## Evidence

- `crates/nako-library/src/failure.rs`
- `crates/nako-library/src/scan.rs`
- `crates/nako-library/src/probe.rs`
