# Addon Managed Artwork Artifacts Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

This lane is newly split from APW-060. No artwork runtime behavior has been
implemented here yet. APW proved Addon Side Effect intake and apply outcome
semantics with `metadata_write`; this lane must audit artwork/artifact seams
before accepting `artwork_write`.

## Active Task

- Task ID: AMAA-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-vfs`, `docs`
- Validation: audit `rg`, then `git diff --check`
- Status: READY
- Review: decide whether first apply target is Artwork Candidate, Managed
  Artwork import, or Taru-Managed Artifact intake
- Evidence: write audit notes to `EVIDENCE_AND_GATES.md`

## Blockers

- None known.

## Next Recommended Action

- Run AMAA-020. Do not implement `artwork_write` until external fetch ownership,
  artifact storage, cache/thumbnail policy, resource budgets, and redacted
  diagnostics are explicit.
