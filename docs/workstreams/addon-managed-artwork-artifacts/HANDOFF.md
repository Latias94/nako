# Addon Managed Artwork Artifacts Handoff

Status: Active
Last updated: 2026-05-19

## Current State

AMAA-030 is complete. `artwork_write` now has a first bounded runtime apply
path: a MediaItem-targeted Addon Artwork Candidate proposal. The addon supplies
candidate intent, image kind, and an HTTP(S) remote URL source. Taru records an
internal candidate and returns only redacted apply outcome facts.

The audit found that current public `ImageAsset` rows expose `source_uri` and
`cache_uri` through catalog DTOs. AMAA-030 therefore deliberately does not
write public selected artwork, public `ImageAsset` rows, managed cache
artifacts, thumbnails, or sidecar files. Later slices must add explicit
Taru-owned fetch/cache/artifact policy before candidates become public artwork.

## Active Task

- Task ID: AMAA-040
- Owner: planner
- Files: `docs/workstreams/addon-managed-artwork-artifacts`, `docs/api`
- Validation: verify-rust-workstream records fresh final gate evidence
- Status: READY
- Review: decide whether this lane should close now or split follow-ons for
  managed artwork fetch/cache, selected artwork, thumbnails, Taru-managed
  artifact intake, or artwork sidecar export
- Evidence: AMAA-030 code/tests/API evidence is recorded in
  `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run AMAA-040. Close this lane if candidate proposal is enough for now, or
  split follow-ons if the next product value is candidate review, managed
  fetch/cache, selected artwork, thumbnails, Taru-managed artifact intake, or
  artwork sidecar export.
- Keep sidecar export in `addon-library-file-write-policy`. AMAA owns artwork
  candidates, managed artwork, artifact/cache policy, and public artwork
  redaction.
- CAD-070 alignment still applies: if a later artwork slice needs
  catalog-visible multi-row persistence, reuse or introduce a first-party
  artwork/catalog commit boundary instead of embedding ordering logic in the
  Addon handler.
