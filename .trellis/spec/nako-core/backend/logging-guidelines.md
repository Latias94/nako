# Logging Guidelines

`nako-core` should stay mostly free of runtime logging.

## Boundary

- Core records may carry redaction-safe diagnostic fields, but runtime log
  emission belongs in adapters and app services.
- Do not add `tracing` spans or logging side effects to core parse helpers,
  records, or repository traits unless an ADR explicitly moves diagnostics into
  the core contract.
- Diagnostic strings in core must be operator-safe: no raw tokens, credentials,
  playback tickets, local filesystem paths, or provider secret payloads.

## Examples

- `storage_health.rs` models storage health state without logging directly.
- `vfs_cache.rs` stores VFS cache failure facts; rendering and emission happen
  in VFS/server surfaces.
