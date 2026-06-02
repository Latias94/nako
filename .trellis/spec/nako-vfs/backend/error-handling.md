# Error Handling

VFS errors should classify storage failures so callers can make bounded
decisions instead of guessing from raw strings.

## Required Patterns

- Invalid `StorageUri` input returns `NakoError::InvalidInput`.
- Backend failures should map to `StorageErrorKind` and
  `StorageFailureClass` when possible.
- Cache repair diagnostics should use `VfsCacheRepairDiagnostic` and
  `VfsCacheRepairClassification`.
- Redact cache failure messages before exposing them. Use safe operator actions
  instead of raw backend error dumps.
- Permission and security failures should become operator-action-required, not
  infinite retry loops.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| URI missing `://` or empty scheme | `NakoError::InvalidInput` |
| Fresh cache with no failure | `Healthy` |
| Stale fallback object | `RepairableStaleFallback` |
| Timeout/unavailable/rate limited/partial read | `RetryableRefreshFailure` |
| Permission or security failure | `OperatorActionRequired` |
| Unknown failure class | `UnknownFailure` |

## Wrong vs Correct

### Wrong

```rust
Err(NakoError::Provider {
    provider: "webdav".to_owned(),
    message: format!("{raw_url_with_secret}: {err}"),
})
```

### Correct

```rust
Err(NakoError::Provider {
    provider: "webdav".to_owned(),
    message: "webdav storage request failed".to_owned(),
})
```

Expose details through redaction-safe diagnostics only.

## Evidence

- `crates/nako-vfs/src/lib.rs`
- `crates/nako-core/src/error.rs`
- `crates/nako-core/src/vfs_cache.rs`
