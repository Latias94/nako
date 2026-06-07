# 0055: Use Apache OpenDAL As An Optional Storage Adapter Foundation

## Status

Proposed.

## Context

Nako's storage/VFS model is intentionally host-owned. ADR 0002 requires an
internal VFS before OS mounting, and ADR 0016 keeps remote storage behavior,
cache state, staging, credentials, and source identity out of provider-specific
application code.

The current `nako-vfs` boundary already defines:

- `StorageUri` as storage identity;
- `StorageCapabilities` as product-visible backend facts;
- `ByteRange` as the centralized range validation contract;
- `StorageBackend` as the adapter trait;
- `ReadRange` and `ReadStream` for bounded byte access;
- redaction-safe cache repair diagnostics;
- deterministic staging paths;
- local path authority for `LocalFsBackend`;
- WebDAV-specific credential, range, retry, and staging behavior in
  `WebDavBackend`.

Broader storage support will eventually need S3-compatible object stores,
additional WebDAV variants, SFTP/SMB-like remotes, and cloud-drive style
backends. Hand-writing every backend would preserve control but creates
repeated auth, listing, range, retry, timeout, metadata, and error mapping work.

Apache OpenDAL 0.57.0 provides a Rust `Operator` API, optional services and
layers, stat/read/range/list primitives, retry/timeout/logging/concurrency
layers, and Apache-2.0 licensing. It is a good candidate for reducing adapter
implementation breadth, but it is a generic storage library rather than Nako's
storage domain model.

One important mismatch is listing semantics. OpenDAL `Operator::list` is
prefix-based: a missing parent can still list deeper prefixed entries, and the
queried path itself may be returned. Nako `StorageBackend::list` currently means
listing a directory after storage authority validation.

## Decision

Nako may use Apache OpenDAL as an optional implementation foundation behind
`StorageBackend`.

OpenDAL must not replace Nako-owned storage concepts or cross-crate contracts:

- `StorageUri`;
- Source Locator redaction;
- `StorageCapabilities`;
- `ByteRange`;
- `StorageBackend`;
- VFS cache and repair authority;
- storage health/circuit-breaker records;
- source fingerprint semantics;
- deterministic staging;
- local path and library file write authority;
- Admin/Public API DTOs.

Any OpenDAL-backed adapter must map OpenDAL paths, metadata, modes,
capabilities, range reads, listing results, and errors into Nako types before
data crosses the `nako-vfs` boundary.

The first implementation must be a proof adapter, not a production backend
rollout. It should use a low-risk OpenDAL service such as memory or filesystem,
stay behind an explicit feature or internal module, and prove:

- `StorageUri` to OpenDAL path mapping rejects traversal, credentials, and
  authority leaks;
- `stat` maps object kind, length, modified time, etag/fingerprint facts, and
  capabilities;
- `list` narrows OpenDAL prefix behavior into Nako direct-directory behavior;
- `read_range` and `stream_range` preserve `ByteRange` validation and avoid
  whole-object reads on streaming paths;
- OpenDAL errors map to `StorageErrorKind` and redaction-safe failure classes;
- cache repair, storage health, source fingerprint, and staging ownership stay
  outside OpenDAL layers.

## Architecture Sketch

```text
Library config / future backend config
  -> Nako backend factory
  -> OpenDAL service builder + selected layers
  -> OpenDalStorageBackend
  -> StorageBackend trait
  -> scan / probe / playback / cache repair / Admin diagnostics
```

OpenDAL is an implementation detail inside the adapter box. Callers still see
Nako storage records and diagnostics.

## Success Metrics

| Metric | Target | Measurement |
| --- | --- | --- |
| Boundary stability | No public `StorageBackend` or `StorageUri` replacement | Diff review and crate API review |
| Semantic parity | Proof adapter passes focused stat/list/range/error tests | `cargo nextest run -p nako-vfs opendal --no-fail-fast` in the follow-on |
| Redaction safety | No raw backend URL, credential, local path, or provider error leaves adapter tests | Focused redaction assertions |
| Dependency control | OpenDAL dependency is optional and feature-scoped when introduced | `Cargo.toml` feature review and `cargo tree` |
| Listing correctness | Prefix results are narrowed to direct directory entries | Adapter tests with missing parent and nested objects |

## Risks And Mitigations

| Risk | Severity | Likelihood | Mitigation |
| --- | --- | --- | --- |
| Generic `Operator` semantics dilute Nako storage authority | High | Medium | Keep `StorageBackend` as the only cross-crate contract |
| Prefix listing returns too much or masks missing directories | High | Medium | Add direct-directory filtering and focused tests |
| Dependency/features pull unnecessary services | Medium | Medium | Disable default features when feasible and enable only needed services/layers |
| OpenDAL minor releases introduce adapter churn | Medium | Medium | Pin version through workspace deps and treat upgrades as explicit tasks |
| Error messages leak backend details | High | Medium | Map OpenDAL errors into Nako safe error kinds/messages before exposure |
| Layers duplicate Nako retry/health/cache policy | Medium | Medium | Use layers only for per-request mechanics; keep durable policy in Nako |

## Consequences

- Future backend breadth can reuse OpenDAL services without exposing OpenDAL as
  Nako's public storage model.
- The first implementation must spend effort on semantic mapping rather than
  production backend configuration.
- Existing `LocalFsBackend` and `WebDavBackend` remain the compatibility
  baseline until a proof adapter demonstrates parity.
- OpenDAL dependency introduction remains deferred to a separate code task.
- Storage health, cache repair, source identity, staging, and Admin diagnostics
  remain Nako-owned.

## Alternatives Considered

### Keep bespoke adapters only

Rejected as the default long-term strategy. It maximizes control and avoids new
dependencies, but each backend repeats range, metadata, auth, retry, timeout,
listing, and error behavior. This slows S3-compatible and cloud-drive backend
breadth and increases drift risk across adapters.

### Replace `StorageBackend` with OpenDAL `Operator`

Rejected. This would push generic storage semantics across Nako's scan, probe,
playback, cache, source identity, Admin, and job boundaries. It would weaken
Nako-owned URI, redaction, staging, capability, health, and repair contracts for
little near-term product value.

### Use `object_store` with `object_store_opendal`

Deferred. `object_store_opendal` is useful for ecosystems that already consume
the `object_store` trait, but Nako already has a richer media-storage boundary
with local path hints, deterministic staging, link/write policy, cache repair,
and source identity semantics. Introducing both `object_store` and OpenDAL would
add another abstraction layer before proving value.

## Related Workstreams

- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
- `docs/architecture/STORAGE_VFS.md`
- `.trellis/tasks/archive/2026-06/06-07-06-07-storage-vfs-opendal-adapter-decision-spike/`
