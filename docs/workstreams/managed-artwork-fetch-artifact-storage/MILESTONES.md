# Managed Artwork Fetch Artifact Storage Milestones

Status: Active
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
- The first artifact byte store should be a Taru-owned local internal artifact
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
- `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`
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
- `cargo nextest run -p taru-server artwork --no-fail-fast`
- `cargo nextest run -p taru-db artwork --no-fail-fast`

## M4 - Closeout Or Split

Outcome: internal managed artwork artifact storage is complete enough to close,
or public-serving/thumbnail/selection breadth is split.

Exit criteria:

- Fresh command evidence is recorded.
- HTTP/API docs reflect shipped behavior.
- Public image serving, thumbnails, selected artwork publication, and catalog
  projection refresh are completed, deferred, or split.
