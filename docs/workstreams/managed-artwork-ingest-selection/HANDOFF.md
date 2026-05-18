# Managed Artwork Ingest Selection Handoff

Status: Active
Last updated: 2026-05-19

## Current State

MAIS-020 is complete. No managed artwork ingest runtime behavior has been
implemented here yet.

AMAA-030 shipped internal Addon Artwork Candidate proposals. Those candidates
may contain remote source URLs, but they are not public client artwork and do
not create `ImageAsset` rows, cache artifacts, thumbnails, or selected artwork.

The selected first implementation target is a queued candidate-ingest boundary
that creates internal Managed Artwork state. It must not create selected public
`ImageAsset` rows during candidate acceptance.

## Active Task

- Task ID: MAIS-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-vfs`, `docs/api`, `docs`
- Validation: focused managed artwork tests; `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-vfs --tests`; `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Review: implement a Taru-owned queued candidate-ingest path with redacted job
  input/summary and internal Managed Artwork state before public publication
- Evidence: update `EVIDENCE_AND_GATES.md`, API docs, code, and tests

## Blockers

- None known.

## Next Recommended Action

- Run MAIS-030. Add the internal managed artwork ingest/job/artifact boundary
  first. A reasonable implementation shape is:
  `ManagedArtworkService::accept_candidate(candidate_id, policy)` validates the
  candidate and queues a managed artwork ingest job; the worker records
  Taru-managed artifact state after fetch/content validation.
- Add or amend `JobKind` and resource class for managed artwork ingest if the
  generic jobs table is used. The durable job input and summary must contain
  redacted Taru IDs and counters only, because `/jobs/{job_id}` exposes parsed
  input and summary today.
- Keep `ArtworkTask` for post-publication image work or later thumbnail/resize
  tasks unless it is explicitly refactored away from `ImageAssetId`.
- Do not publish `ImageAsset` until a managed artifact exists and the public
  image reference/redaction contract is explicit.
- Do not put remote fetch/cache/thumbnailing in the Addon Side Effect handler.
- Do not expose candidate `source_uri`, Source Locators, filesystem paths,
  remote storage handles, raw validation failures, `cache_uri`, or cache
  internals in Public Client DTOs, Addon responses, Admin list responses, job
  input, or job summary.
- Keep artwork sidecar export in `addon-library-file-write-policy`.
