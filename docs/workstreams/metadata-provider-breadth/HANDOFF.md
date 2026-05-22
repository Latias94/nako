# Metadata Provider Breadth — Handoff

Status: Completed
Last updated: 2026-05-21

## Current State

The Wave 1 metadata provider breadth lane is implemented and verified. Nako now
has diagnostics-safe provider capabilities, deterministic candidate matching,
non-destructive ambiguous search refresh, and a first cross-provider candidate
review boundary.

## Completed Tasks

- MPB-010: workstream opened and scope frozen.
- MPB-020: provider capabilities exposed from `nako-metadata` through
  `/metadata/providers` without secrets.
- MPB-030: explicit candidate match decisions and reasons added.
- MPB-040: ambiguous search-based refresh now stops before fetch/cache/commit
  when confirmation is required; external-ID refresh remains compatible.
- MPB-050: `/items/{item_id}/metadata/candidates` exposes reviewable
  cross-provider decisions while leaving canonical metadata, raw responses, and
  provider mappings untouched.
- MPB-060: docs, gates, and follow-on split notes refreshed.

## Verification Summary

Fresh gates recorded in `EVIDENCE_AND_GATES.md`:

- `cargo nextest run -p nako-api --no-fail-fast`
- `cargo nextest run -p nako-metadata --no-fail-fast`
- `cargo nextest run -p nako-server metadata --no-fail-fast`
- `cargo check --workspace --tests`
- `cargo fmt --all -- --check`
- `git diff --check`

## Decisions Since Last Update

- Capability reporting stays in-memory/diagnostics-safe; no schema migration is
  needed for provider capabilities.
- Candidate matching remains deterministic and threshold-based; no AI or opaque
  probabilistic ranking was introduced.
- Search refresh may only auto-commit accepted candidates. Needs-confirmation
  and rejected candidates are recorded as non-success attempts and do not fetch
  provider payloads.
- Cross-provider candidate review is an API/service boundary, not a durable
  review queue. Durable candidate persistence and Admin UI confirmation should
  be a follow-on if needed.

## Follow-Ons

- Durable candidate review queue and manual accept/reject UI if operators need
  asynchronous confirmation.
- Lower-case or multi-provider query parsing ergonomics for
  `/items/{item_id}/metadata/candidates`; current route supports the effective
  profile plus one optional provider override.
- Provider-specific capability precision can deepen as TMDB/Douban/Bangumi
  feature breadth grows.

## Blockers

- None.

## Next Recommended Action

- Commit this lane, then use the post-RPD umbrella to open `nfo-link-authority`
  as the next execution workstream. Playback/transcode ops hardening can run in
  parallel only if its write scope stays disjoint.
