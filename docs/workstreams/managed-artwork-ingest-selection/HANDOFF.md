# Managed Artwork Ingest Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAIS-030 is complete. The first managed artwork ingest runtime boundary is now
implemented as a queued Admin API acceptance path.

AMAA-030 shipped internal Addon Artwork Candidate proposals. Those candidates
may contain remote source URLs, but they are not public client artwork and do
not create `ImageAsset` rows, cache artifacts, thumbnails, or selected artwork.

The selected first implementation target is a queued candidate-ingest boundary
that creates internal Managed Artwork state. The shipped slice creates a
durable ingest record and `managed_artwork_ingest` job, but still does not
fetch remote bytes, cache artifacts, thumbnail, create selected public
`ImageAsset` rows, or publish client-visible artwork.

## Active Task

- Task ID: MAIS-040
- Owner: codex
- Files: `docs/workstreams/managed-artwork-ingest-selection`, `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: READY
- Review: decide whether this lane should close as the queued boundary, or
  split remote fetch/artifact bytes, thumbnails, and public publication into
  follow-ons
- Evidence: update `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and closeout
  notes

## Blockers

- None known.

## Next Recommended Action

- Run MAIS-040 using close-workstream or split follow-on lanes. The likely
  follow-ons are remote fetch/content validation into
  `managed_artwork_artifacts`, image-serving/redacted public artwork
  references, thumbnail/resize workers, and selected artwork publication.
- Keep `ArtworkTask` for post-publication image work or later thumbnail/resize
  tasks unless a future lane explicitly refactors it away from `ImageAssetId`.
- Do not publish `ImageAsset` until a managed artifact exists and the public
  image reference/redaction contract is explicit.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
- Do not expose candidate `source_uri`, Source Locators, filesystem paths,
  remote storage handles, raw validation failures, `cache_uri`, or cache
  internals in Public Client DTOs, Addon responses, Admin list responses, job
  input, or job summary.
- Keep artwork sidecar export in `addon-library-file-write-policy`.
