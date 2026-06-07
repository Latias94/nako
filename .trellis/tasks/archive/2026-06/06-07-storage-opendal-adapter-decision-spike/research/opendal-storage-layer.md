# OpenDAL Storage Layer Evaluation

Date: 2026-06-07

## Question

Should Nako adopt `opendal` as its storage layer?

## Sources

- crates.io / cargo index: `opendal = "0.57.0"` from `cargo search opendal --limit 1`
- OpenDAL Rust service docs: https://opendal.apache.org/docs/rust/opendal/services/
- OpenDAL WebDAV service docs: https://opendal.apache.org/docs/rust/opendal/services/struct.Webdav.html
- OpenDAL filesystem service docs: https://opendal.apache.org/docs/rust/opendal/services/struct.Fs.html
- OpenDAL S3 service docs: https://opendal.apache.org/docs/rust/opendal/services/struct.S3.html
- OpenDAL retry layer docs: https://docs.rs/opendal/latest/opendal/layers/struct.RetryLayer.html
- OpenDAL timeout layer docs: https://docs.rs/opendal/latest/opendal/layers/struct.TimeoutLayer.html
- OpenDAL feature list: https://docs.rs/crate/opendal/latest/features

## Findings

OpenDAL is a credible Rust storage abstraction for backend access. Its current
service set covers local filesystem, WebDAV, S3-compatible storage, and other
backend targets. The feature set includes retry, timeout, logging, concurrent
limit, tracing/metrics, throttle, and capability checks.

The WebDAV service advertises broad read/write capabilities, including stat,
read, write, delete, list, copy, and rename. Nako's WebDAV product surface is
deliberately narrower and must preserve its own capability policy.

Retry and timeout layers are useful but non-trivial. OpenDAL's docs make it
clear that layer ordering matters; Nako still needs its own runtime/resource
policy and redaction policy instead of assuming OpenDAL's layers are the
product boundary.

## Nako Fit

OpenDAL maps well to an adapter behind Nako's `StorageBackend` interface.

OpenDAL does not replace Nako's domain-owned storage model:

- `StorageUri`
- Source Locator redaction
- Source Fingerprint evidence
- library-scoped backend authority
- storage backend health and circuit breaker records
- VFS cache repair authority
- deterministic staging paths
- Admin-safe diagnostics
- playback/probe/scan resource policy

## Local Code Inspection

`crates/nako-vfs/src/lib.rs` shows that `StorageBackend` is already a
domain-level contract, not just an object-store facade. It includes:

- scheme ownership through `StorageUri`;
- object metadata with Nako capabilities, cache status, etag, and fingerprint;
- byte-range reads and streaming reads;
- deterministic staging reports;
- local write, atomic replace, backup, restore, cleanup, apply, hardlink, and
  symlink planning semantics;
- default unsupported reports for backends that do not support mutation.

`LocalFsBackend` currently owns path authority, local path escape prevention,
direct and atomic writes, backup retention, restore, cleanup, link planning,
and local staging by returning an existing path hint.

`WebDavBackend` currently owns credential redaction, endpoint validation,
manual retry/timeout policy, PROPFIND parsing, range reads, streaming reads,
deterministic staging, and deliberately read-only write behavior.

This means OpenDAL could reduce backend-specific HTTP/filesystem operation
code, but only after Nako defines an adapter layer that maps OpenDAL behavior
back into Nako's stricter product contract.

## Recommended Decision

Do not add OpenDAL as an immediate production dependency.

Open a bounded adapter decision spike first:

1. Compare current `LocalFsBackend` and `WebDavBackend` behavior against
   OpenDAL `Operator` capabilities.
2. Define how OpenDAL errors map to Nako storage failure classes without
   leaking raw paths, URLs, headers, credentials, etags, or provider errors.
3. Define capability narrowing so OpenDAL write/delete/copy support cannot
   accidentally widen Nako's M1/M2 product contract.
4. Prove range-read and streaming behavior against a test-only adapter or a
   feature-gated backend before touching production config.
5. Decide whether future S3-compatible support is worth the dependency.

## Suggested Follow-On

Candidate task name:

`storage-opendal-adapter-first-slice`

Exit states:

- Reject: keep hand-written backends until storage breadth justifies a shared
  operator layer.
- Defer: revisit when S3/SFTP/object storage becomes a committed product target.
- Adopt narrowly: add OpenDAL behind `StorageBackend` with feature flags and
  redaction/capability/range-read tests.

