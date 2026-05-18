# Managed Artwork Ingest Selection Handoff

Status: Proposed
Last updated: 2026-05-19

## Current State

This lane is newly opened from AMAA-040 closeout. No managed artwork ingest
runtime behavior has been implemented here yet.

AMAA-030 shipped internal Addon Artwork Candidate proposals. Those candidates
may contain remote source URLs, but they are not public client artwork and do
not create `ImageAsset` rows, cache artifacts, thumbnails, or selected artwork.

## Active Task

- Task ID: MAIS-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-vfs`, `docs`
- Validation: `rg -n "ArtworkCandidate|ImageAsset|ArtworkTask|cache_uri|source_uri|thumbnail|staging|managed artwork|selected" crates docs`; `git diff --check`
- Status: READY
- Review: choose the first candidate acceptance target and define managed
  fetch/cache/selection/redaction boundaries before writing runtime behavior
- Evidence: update `EVIDENCE_AND_GATES.md` with audit notes and first-target
  decision

## Blockers

- None known.

## Next Recommended Action

- Run MAIS-020. Audit whether `ArtworkTask` can represent candidate
  fetch/validate/cache, where managed artwork bytes should live, how cache URI
  assignment should work, and whether the first slice should publish selected
  public `ImageAsset` rows or only create unselected managed artifacts.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
- Do not expose candidate `source_uri`, Source Locators, filesystem paths,
  remote storage handles, or cache internals in Public Client DTOs.
- Keep artwork sidecar export in `addon-library-file-write-policy`.
