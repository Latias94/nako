# Managed Artwork Fetch Artifact Storage Milestones

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

Outcome: the fetch/artifact follow-on is split from MAIS with clear authority
and gates.

Exit criteria:

- Problem, target state, non-goals, and closeout condition are explicit.
- MAIS closeout points to this lane as the next recommended action.
- Workstream index links the new lane.

Primary evidence:

- `docs/workstreams/managed-artwork-fetch-artifact-storage/DESIGN.md`
- `docs/workstreams/managed-artwork-ingest-selection/HANDOFF.md`

## M1 - Fetch And Storage Seam Audit

Outcome: job runtime, fetch policy, validation, storage, and artifact commit
seams are classified before worker implementation.

Exit criteria:

- `managed_artwork_ingest` job claim/update semantics are understood.
- The first artifact byte storage policy is chosen.
- Redacted failure and job summary requirements are explicit.
- Public image serving, thumbnails, and selection remain split.

Result:

- Dedicated managed artwork claim/commit methods are required because generic
  job start does not provide a safe claim-next boundary for queued artwork
  ingests.
- The first artifact byte store should be a Nako-owned local internal artifact
  root exposed to the database only as opaque `managed-artwork://...`
  references.
- VFS cache, staging manifests, and public `ImageAsset` are rejected as first
  artifact authority.

Primary gates:

- `rg -n "ManagedArtworkIngest|managed_artwork_ingest|managed_artwork_artifacts|JobKind::ManagedArtworkIngest|artwork.ingest|storage_uri|ImageAsset|cache_uri|source_uri|thumbnail" crates docs`
- `git diff --check`

## M2 - First Fetch/Artifact Slice

Outcome: one queued managed artwork ingest can produce an internal artifact or
safe failure state.

Exit criteria:

- The worker or service consumes queued ingest work outside HTTP handlers.
- Fetch and validation use explicit resource limits.
- Artifact bytes and `managed_artwork_artifacts` metadata are committed through
  one clear ownership boundary.
- No public `ImageAsset` row or selected artwork is created.

Primary gates:

- focused managed artwork worker tests
- `cargo check -p nako-core -p nako-db -p nako-api -p nako-server -p nako-vfs --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

Result:

- `POST /admin/v1/artwork/ingests/process-next` can claim one queued managed
  artwork ingest, fetch HTTP(S) bytes under artwork-specific policy, validate a
  static JPEG/PNG/WebP image, write bytes to an internal local artifact root,
  and commit `managed_artwork_artifacts` metadata with an opaque
  `managed-artwork://...` storage reference.
- Managed artwork artifact summaries and job summaries expose only safe IDs,
  status, media type, byte length, dimensions, and content hash; raw source
  URLs, storage URIs, filesystem paths, cache URIs, addon tokens, public
  `ImageAsset`, thumbnails, and selected artwork remain out of scope.
- Repository methods now provide a managed-artwork-specific claim/commit/fail
  boundary instead of relying on generic job start/succeed/fail calls.

## M3 - Failure, Retry, And Redaction Hardening

Outcome: failed fetch/validation/storage attempts produce safe, actionable,
redacted diagnostics.

Exit criteria:

- Failure codes are bounded and stable enough for Admin API use.
- Job summaries and errors do not expose raw URLs, paths, cache/storage URIs,
  tokens, or decoder internals.
- Retry and cancellation semantics are documented or explicitly deferred.

Primary gates:

- focused failure/redaction tests
- `cargo nextest run -p nako-server artwork --no-fail-fast`
- `cargo nextest run -p nako-db artwork --no-fail-fast`

Result:

- Managed artwork process failures now use an internal bounded failure-code
  enum before being serialized as safe strings.
- Failed process-next responses include safe job summaries with `status` and
  `failure_code`, and job errors are the same safe code.
- Redaction tests cover unsupported media type and invalid image bodies,
  proving failed responses do not expose raw source URLs, response bodies,
  addon tokens, `source_uri`, `cache_uri`, `storage_uri`, public
  `ImageAsset`, or decoder internals.
- Retry remains limited to the configured in-process fetch attempts. Durable
  requeue/cancellation APIs are not introduced in this lane and should be
  split explicitly if needed.

## M4 - Closeout Or Split

Outcome: internal managed artwork artifact storage is complete enough to close,
or public-serving/thumbnail/selection breadth is split.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped behavior.
- Public image serving, thumbnails, selected artwork publication, and catalog
  projection refresh are completed, deferred, or split.

Result:

- Workstream closed on 2026-05-19 after fresh closeout gates.
- Follow-ons should be opened separately for public image serving, thumbnails,
  selected artwork publication/catalog projection refresh, durable retry/requeue,
  cancellation, and orphan artifact cleanup.
