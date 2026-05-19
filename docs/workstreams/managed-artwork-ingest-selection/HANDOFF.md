# Managed Artwork Ingest Selection Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is closed. The first managed artwork ingest runtime boundary is now
implemented as a queued Admin API acceptance path and committed as
`de72467 feat(artwork): queue managed candidate ingest`.

AMAA-030 shipped internal Addon Artwork Candidate proposals. Those candidates
may contain remote source URLs, but they are not public client artwork and do
not create `ImageAsset` rows, cache artifacts, thumbnails, or selected artwork.

The selected first implementation target is a queued candidate-ingest boundary
that creates internal Managed Artwork state. The shipped slice creates a
durable ingest record and `managed_artwork_ingest` job, but still does not
fetch remote bytes, cache artifacts, thumbnail, create selected public
`ImageAsset` rows, or publish client-visible artwork.

## Closeout Task

- Task ID: MAIS-040
- Owner: planner
- Files: `docs/workstreams/managed-artwork-ingest-selection`, `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: DONE
- Review: MAIS-030 has no blocking findings. It creates internal managed
  ingest state and a durable redacted job, but deliberately keeps remote
  fetch, artifact bytes, thumbnails, selected artwork, and public publication
  out of this lane.
- Evidence: `EVIDENCE_AND_GATES.md` contains implementation, validation, and
  closeout evidence.

## Blockers

- None known.

## Decisions Since Last Update

- Close MAIS after proving the queued candidate-ingest boundary instead of
  widening this lane into fetch, artifact byte storage, image serving,
  thumbnail, and selected-publication work.
- Keep `managed_artwork_ingests` as internal authority for candidate
  acceptance. Public `ImageAsset` publication remains blocked until a managed
  artifact exists and public image references are redacted.
- Keep `ArtworkTask` for post-publication image work or later thumbnail/resize
  tasks unless a future lane explicitly refactors it away from `ImageAssetId`.
- Keep artwork sidecar export in `addon-library-file-write-policy`.

## Residual Risks

- Candidate source URLs remain stored internally for future fetch workers.
  Future Admin list/detail routes must preserve redaction or explicitly model
  privileged URL access.
- `managed_artwork_artifacts` exists as schema shape only; no worker stores
  bytes into it yet.
- Public Client artwork still exposes `source_uri` and `cache_uri` through
  existing `ImageAsset` DTOs. Do not publish addon candidate-derived artwork
  there until the public image-serving contract changes.
- The queued `managed_artwork_ingest` job is durable but no worker consumes it
  yet.

## Next Recommended Action

- Open a focused follow-on for managed artwork remote fetch/content validation
  and artifact byte storage into `managed_artwork_artifacts`.
- After artifact bytes exist, split image-serving/redacted public artwork
  references before selected artwork publication.
- Keep thumbnail/resize workers separate from initial fetch/storage unless the
  worker boundary and resource budgets stay small.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
- Do not expose candidate `source_uri`, Source Locators, filesystem paths,
  remote storage handles, raw validation failures, `cache_uri`, or cache
  internals in Public Client DTOs, Addon responses, Admin list responses, job
  input, or job summary.
- Keep artwork sidecar export in `addon-library-file-write-policy`.
