# Managed Artwork Fetch Artifact Storage

Status: Completed
Last updated: 2026-05-19

## Why This Lane Exists

`managed-artwork-ingest-selection` created the internal acceptance boundary but
intentionally stopped before remote fetch, content validation, byte storage, and
public artwork publication. A queued ingest job is useful only when Nako can
later consume it under bounded first-party resource policy and produce an
internal artifact that is safe to publish in later lanes.

## Relevant Authority

- ADRs:
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Existing docs:
  - `docs/workstreams/managed-artwork-ingest-selection/`
  - `docs/workstreams/addon-managed-artwork-artifacts/`
  - `docs/api/HTTP_API.md`
- Existing code:
  - `crates/nako-core/src/media/artwork.rs`
  - `crates/nako-core/src/job.rs`
  - `crates/nako-db/migrations/0026_managed_artwork_ingest.sql`
  - `crates/nako-db/src/artwork.rs`
  - `crates/nako-server/src/app/artwork.rs`

## Problem

Accepted artwork candidates currently create a queued ingest and durable job,
but no first-party worker consumes the job, fetches bytes, validates image
content, or writes `managed_artwork_artifacts`. Without this boundary, later
public image serving or selected artwork publication would still need to choose
between leaking unvalidated addon URLs or reintroducing fetch/storage ordering
inside HTTP handlers.

## Target State

- A queued `managed_artwork_ingest` job can be claimed and processed by a
  Nako-owned runtime boundary.
- The worker loads the candidate source internally; raw source URLs remain out
  of public, addon, and ordinary job summaries.
- Remote fetch uses explicit budgets: resource class, timeout, retry, maximum
  byte length, content-type allowlist, and cancellation behavior.
- Validation records only safe failure codes such as `fetch_timeout`,
  `unsupported_media_type`, `too_large`, `invalid_image`, or
  `storage_failed`.
- Successful processing writes artifact bytes to internal storage and commits a
  `managed_artwork_artifacts` row plus `managed_artwork_ingests.status =
  stored` atomically enough that readers never observe a stored ingest without
  artifact metadata.
- Public `ImageAsset`, selected artwork, thumbnails, and public cache/image
  references remain out of scope.

## In Scope

- Audit current job runtime, storage/VFS, HTTP client, image validation, and
  repository seams for managed artwork fetch.
- Define the first internal artifact byte storage policy.
- Add repository methods for managed ingest status transitions and artifact
  commit.
- Add a worker or first-party app service that processes one queued managed
  artwork ingest under redaction and resource controls.
- Add focused tests for success, invalid content, redacted failure reporting,
  and no public `ImageAsset` publication.

## Out Of Scope

- Public Client image serving and DTO redesign.
- Thumbnail generation and resize task execution.
- Selected artwork commit and catalog/search projection refresh.
- Addon Manager lifecycle automation.
- Arbitrary image generation, editing, or AI image processing.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing `jobs` can identify managed artwork ingest work by `JobKind::ManagedArtworkIngest` and `resource_class = "artwork.ingest"`. | High | MAIS-030 implementation and tests | If not, introduce a narrower internal queue table or job claim adapter before processing bytes. |
| `managed_artwork_artifacts.storage_uri` should remain internal and must not become a public cache URI. | High | Public `ImageAsset` currently exposes `cache_uri` | If public serving needs a URI, add a redacted image reference contract in a later lane. |
| The first artifact store can be local/internal before remote storage backends are supported. | Medium | Existing VFS/cache/staging seams are not durable artwork authority | If storage policy must be VFS-backed immediately, MAFA-020 must split storage abstraction before worker implementation. |
| Image validation can start with byte size, media type, dimensions, and decodability without thumbnail output. | Medium | This lane excludes public thumbnails | If validation needs decoded image metadata from a new dependency, add the dependency behind a small validation port. |

## Architecture Direction

Keep the ownership chain explicit:

1. Admin acceptance creates `managed_artwork_ingests` and a durable job.
2. The managed artwork worker claims the job, loads the candidate internally,
   fetches bytes under resource policy, validates the image, writes artifact
   bytes, and commits artifact metadata.
3. Later lanes can publish an artifact as Public Client artwork only through a
   separate redacted image-serving and selected-artwork commit boundary.

Do not reuse `ArtworkTask` for the first fetch worker unless it is refactored
away from `ImageAssetId`; candidate fetch happens before public `ImageAsset`
creation. Do not place fetch or storage ordering inside HTTP handlers or Addon
Side Effect handling.

### MAFA-020 Audit Decision

The first implementation target should be a dedicated managed artwork runtime
boundary, not a reuse of library scan, staging, VFS cache, or public
`ImageAsset` paths.

Job runtime decision:

- Keep `JobKind::ManagedArtworkIngest` and `resource_class = "artwork.ingest"`.
- Add managed-artwork-specific repository methods instead of relying only on
  generic `start_job`. Existing `DurableJobRuntime` can persist success/failure
  for a known job ID, but the next slice needs an atomic claim boundary that
  finds one queued `managed_artwork_ingest`, verifies its job/candidate state,
  marks the job running, and moves the ingest to `fetching`.
- Keep job `input_json`, `summary_json`, and `error` redacted. Summaries should
  include IDs, final status, byte counters, dimensions, media type, and safe
  failure codes only.

Storage decision:

- Introduce a first-party internal artifact storage port owned by
  `nako-server` application code. The first backing store should be a local
  Nako-managed artifact root, not a library root and not VFS cache/staging.
- Add explicit config for artwork artifact storage and fetch policy rather
  than overloading `remux_staging_root`, `staging.max_bytes`, or playback remote
  stream/stage budgets.
- Store bytes under a content-addressed or artifact-id-addressed internal path
  below the configured artwork artifact root. Persist only an opaque internal
  `managed-artwork://...` reference in `managed_artwork_artifacts.storage_uri`;
  do not persist raw absolute paths as artifact authority.
- Byte/file and database writes cannot be fully atomic across the filesystem
  and SQLite. The worker should write and fsync a temporary file, atomically
  promote it to the internal artifact path, verify hash/length, then commit the
  artifact row and ingest status in one DB transaction. If DB commit fails,
  delete the promoted file best-effort; orphan cleanup can be split later. This
  ordering avoids a `stored` ingest pointing to bytes that were never written.

Fetch and validation decision:

- Fetch only `ArtworkCandidateSourceKind::RemoteUrl` with an HTTP(S) scheme,
  even though proposal intake already validates it.
- Use a small artwork fetch policy: timeout, max attempts, max bytes,
  concurrency, user agent, and optional proxy. The first implementation can
  reuse `reqwest` directly behind an artwork fetcher port; do not reuse
  metadata provider JSON runtime because artwork fetch is byte-stream and
  content-validation oriented.
- Stream with a hard byte cap before buffering. Reject content that exceeds the
  configured cap even if `Content-Length` is absent or dishonest.
- Start with static image media types that can be decoded reliably
  (`image/jpeg`, `image/png`, `image/webp`). Record dimensions and byte length
  from decoded content, not only from candidate hints.
- Add a small validation port so the image decoder dependency remains behind a
  narrow interface. Validation failures must map to safe codes such as
  `unsupported_media_type`, `too_large`, `invalid_image`, or
  `dimension_limit_exceeded`.

Rejected reuse:

- VFS cache is a remote storage fact cache, not durable artwork authority.
- Staging manifests are cleanup/lease-oriented probe and FFmpeg input state;
  adding artwork as another staging purpose would still not make it selected
  or public artwork authority.
- `LocalFsBackend` is library-root oriented and text-write shaped today. It is
  useful as reference for path safety and atomic replace mechanics, but managed
  artwork should use a purpose-built artifact storage port.
- Current public `ImageAsset` remains out of scope because its DTOs expose
  `source_uri` and `cache_uri`.

## Closeout Condition

This lane can close when:

- one queued managed artwork ingest can be processed into an internal managed
  artifact or a safe failure state,
- artifact bytes and artifact metadata cannot diverge under normal success and
  failure paths,
- redaction tests prove source URLs, storage paths, cache URIs, and raw
  validation details are not exposed through public/addon/job response seams,
- targeted Rust gates pass,
- and public serving, thumbnails, and selected artwork publication are split or
  explicitly deferred.

Closeout result: met on 2026-05-19. The lane ships internal artifact authority
only. Public image serving, thumbnails, selected artwork publication, durable
retry/requeue, cancellation, and orphan cleanup are deferred or should be split
into narrower follow-on work.
