# Addon Managed Artwork Artifacts Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is closed. `artwork_write` now has a first bounded runtime apply
path: a MediaItem-targeted Addon Artwork Candidate proposal. The addon supplies
candidate intent, image kind, and an HTTP(S) remote URL source. Taru records an
internal candidate and returns only redacted apply outcome facts.

The audit found that current public `ImageAsset` rows expose `source_uri` and
`cache_uri` through catalog DTOs. AMAA-030 therefore deliberately does not
write public selected artwork, public `ImageAsset` rows, managed cache
artifacts, thumbnails, or sidecar files. Later slices must add explicit
Taru-owned fetch/cache/artifact policy before candidates become public artwork.

## Closeout Task

- Task ID: AMAA-040
- Owner: planner
- Files: `docs/workstreams/addon-managed-artwork-artifacts`, `docs/api`,
  `docs/workstreams/managed-artwork-ingest-selection`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: DONE
- Review: AMAA-030 has no blocking findings. Remaining managed artwork
  fetch/cache, selected artwork, thumbnails, and Taru-managed artifact intake
  breadth is deferred to a focused follow-on instead of widening this completed
  first slice.
- Evidence: `EVIDENCE_AND_GATES.md` contains implementation, validation, and
  closeout evidence

## Blockers

- None known.

## Decisions Since Last Update

- Close AMAA after proving one safe `artwork_write` apply path instead of
  keeping all artwork ingest, cache, thumbnail, selected-artwork, and artifact
  storage breadth in one lane.
- Keep Addon Artwork Candidates internal. Public Client artwork remains backed
  by `ImageAsset` and must not be populated from unverified addon URLs until
  Taru-owned fetch/cache/content validation exists.
- Split Candidate acceptance, remote fetch, image validation, cache URI
  assignment, thumbnail generation, selected artwork, and public `ImageAsset`
  publication to `managed-artwork-ingest-selection`.
- Keep artwork sidecar export in `addon-library-file-write-policy`, not AMAA.

## Residual Risks

- Candidate source URLs are stored internally for later fetch but are not
  public DTOs. Future admin/review routes must preserve redaction or explicitly
  mark URL access as privileged.
- Candidate dedupe currently keys by addon, library, item, image kind, source
  kind, and source URI. If future candidates can normalize CDN variants or
  provider artwork IDs, add a canonical source identity before merging them.
- Candidate acceptance still needs bounded network fetch, content-type and
  image validation, size limits, cache/storage policy, and failure diagnostics.

## Next Recommended Action

- Continue with `docs/workstreams/managed-artwork-ingest-selection/` if the
  next user-visible plugin value is accepting poster/backdrop/logo/banner/
  thumbnail candidates into Taru-managed public artwork.
- Keep sidecar export in `addon-library-file-write-policy`. The new managed
  artwork lane owns fetch/cache/artifact policy and public artwork redaction.
- CAD-070 alignment still applies: if a later artwork slice needs
  catalog-visible multi-row persistence, reuse or introduce a first-party
  artwork/catalog commit boundary instead of embedding ordering logic in the
  Addon handler.
