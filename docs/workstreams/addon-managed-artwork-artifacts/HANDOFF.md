# Addon Managed Artwork Artifacts Handoff

Status: Active
Last updated: 2026-05-19

## Current State

AMAA-020 is complete. No artwork runtime behavior has been implemented here
yet, but the first apply target is selected: a MediaItem-targeted Addon Artwork
Candidate proposal.

The audit found that current public `ImageAsset` rows expose `source_uri` and
`cache_uri` through catalog DTOs. The first `artwork_write` path must therefore
not directly write public selected artwork or unmanaged addon hotlinks. It
should record a first-party candidate that can later be fetched, cached,
selected, or rejected through explicit Taru-owned artwork/artifact policy.

## Active Task

- Task ID: AMAA-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-vfs`, `docs`
- Validation: focused artwork/addon tests; `cargo check -p taru-core -p
  taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all
  -- --check`; `git diff --check`
- Status: READY
- Review: implement the candidate proposal boundary without exposing raw
  payloads, filesystem paths, Source Locators, remote storage handles, cache
  URIs, or unverified addon URLs in public client artwork
- Evidence: update `EVIDENCE_AND_GATES.md` with code/test/API evidence

## Blockers

- None known.

## Next Recommended Action

- Run AMAA-030. Add a typed `artwork_write` payload for MediaItem-targeted
  Artwork Candidate proposals. The addon supplies candidate intent and remote
  URL source metadata; Taru records the candidate and exposes only redacted
  outcome facts.
- Reject filesystem paths, Source Locators, remote storage handles, raw image
  bytes, data URIs, `cache_uri`, `selected`, and sidecar export fields.
- Do not write public `ImageAsset` rows or selected artwork in AMAA-030 unless
  the task is explicitly split to include managed artifact fetch/cache and
  public DTO redaction.
- CAD-070 alignment: if artwork application needs catalog-visible multi-row
  persistence, reuse or introduce a first-party artwork/catalog commit boundary.
  Do not put ordering logic in the Addon handler, and route sidecar-file export
  to `addon-library-file-write-policy`.
