# OpenDAL Adapter Decision Research

Date: 2026-06-07

## Sources Checked

- Apache OpenDAL homepage: https://opendal.apache.org/
- Apache OpenDAL Rust API docs: https://opendal.apache.org/docs/rust/opendal/
- Apache OpenDAL `Operator` docs:
  https://opendal.apache.org/docs/rust/opendal/struct.Operator.html
- Apache OpenDAL changelog for 0.57.0:
  https://docs.rs/crate/opendal/latest/source/CHANGELOG.md
- Local crate metadata:
  `cargo info opendal`

## Current OpenDAL Facts

- Current `opendal` version from `cargo info opendal`: `0.57.0`.
- License from `cargo info opendal`: `Apache-2.0`.
- Rust version from `cargo info opendal`: `1.85`.
- Default features from `cargo info opendal` include auto service
  registration, `reqwest-rustls-tls`, Tokio executor support, and logging,
  retry, timeout, and concurrent-limit layers.
- Apache OpenDAL presents a single `Operator` API over object storage,
  filesystems, cloud SaaS, databases, protocols, and key-value services.
- The Rust docs describe the `opendal` crate as a facade over public APIs from
  `opendal-core` plus optional services and layers.
- `Operator::stat` retrieves metadata for a path.
- `Operator::read` reads a whole file into memory. `read_with` / `read_options`
  support range options, and `reader` is the documented lazy/streaming path for
  large files.
- `Operator::list` is prefix-based: the queried path itself may be returned,
  missing parent directories can still succeed if deeper prefixed objects
  exist, and large listings should prefer `lister`.
- The 0.57.0 changelog contains breaking API changes and service/layer crate
  split work. Treat minor-version upgrades as real adapter-maintenance work.

## Nako Boundary Facts

- `nako-vfs` already defines:
  - `StorageUri`
  - `StorageCapabilities`
  - `StorageBackend`
  - `ByteRange`
  - `ReadRange` / `ReadStream`
  - cache repair diagnostics
  - deterministic staging path derivation
- `LocalFsBackend` owns path authority, local path hints, transactional writes,
  link planning, cleanup, restore, and local staging.
- `WebDavBackend` owns WebDAV URL construction, credential rejection in URLs and
  URIs, redaction-safe HTTP errors, explicit range headers, read-only writes,
  and deterministic staging.
- `CachedStorageBackend` and server storage services own cache repair,
  durable repair jobs, and storage health policy outside individual adapters.
- `StorageBackend::list` currently means list a directory, not prefix search.
- `StorageBackend::stream_range` is the preferred full-object path for large
  source hashing and direct playback flows that do not require local path hints.

## Feasible Approaches

### Approach A: Keep bespoke adapters only

Continue writing each backend directly against `StorageBackend`.

Pros:

- Maximum control over Nako URI, error, staging, and redaction semantics.
- No new dependency surface.
- Existing local and WebDAV behavior remains easy to reason about.

Cons:

- Each new backend repeats auth, retry, range, listing, metadata, and
  provider-specific behavior.
- S3-compatible, SMB/SFTP, cloud drive, and object-store breadth will be slow.
- More hand-written adapters means more chances for inconsistent capability and
  error mapping.

### Approach B: Use OpenDAL behind `StorageBackend` (recommended)

Add an optional `OpenDalStorageBackend` implementation later. It adapts a
configured OpenDAL `Operator` into Nako's `StorageBackend` trait while preserving
Nako-owned URI, capability, error, cache, health, redaction, and staging rules.

Pros:

- Reuses OpenDAL's backend and layer ecosystem for future breadth.
- Keeps Nako domain semantics as the public and cross-crate contract.
- Allows a small proof adapter before production S3/WebDAV configuration.
- Apache-2.0 is compatible with Nako's AGPL server distribution posture.

Cons:

- Adds a non-trivial dependency surface when implemented.
- Needs careful feature selection to avoid pulling every backend.
- OpenDAL prefix-list semantics must be narrowed to Nako directory-list
  semantics.
- OpenDAL error kinds and metadata must be mapped into Nako-safe classes.

### Approach C: Replace `StorageBackend` with OpenDAL `Operator`

Make application code consume OpenDAL directly.

Pros:

- Minimal adapter code.
- Direct access to OpenDAL services and layers.

Cons:

- Loses Nako's current storage authority boundary.
- Risks leaking provider paths, URLs, credentials, or generic errors into
  catalog, API, job, and Admin surfaces.
- Makes source identity, cache repair authority, storage health, deterministic
  staging, and write/link policy dependent on a generic storage abstraction.
- Would force broad cross-crate API churn for little near-term product value.

## Recommended First Proof Slice

Add a non-production OpenDAL proof adapter behind a compile-time feature or
internal module, using a low-risk service such as memory or filesystem. The
proof should show:

- `StorageUri` to OpenDAL path mapping rejects traversal and credentials.
- `stat` maps mode, length, etag/metadata, and capabilities correctly.
- `list` filters prefix results into Nako's direct-directory semantics.
- `read_range` and `stream_range` reuse `ByteRange` validation and do not force
  whole-object reads for large streaming paths.
- OpenDAL errors map to `StorageErrorKind` / `StorageFailureClass` without raw
  backend details.
- No cache repair, storage health, source fingerprint, or staging authority is
  moved into OpenDAL layers.

## Decision Summary

Accept OpenDAL as a future optional adapter foundation, not as Nako's storage
domain model. The next implementation should prove semantic mapping before any
production backend, dependency-wide rollout, or existing WebDAV replacement.
