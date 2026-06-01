# Douban Subject Kind Precision - Handoff

Status: Active
Last updated: 2026-06-02

## Current State

The lane is opened from the closed MPDP follow-on split after Bangumi provider
depth closeout. Current Douban code searches and fetches movie subjects, but
capabilities overclaim Series/Season/Episode support before endpoint-backed
behavior exists.

## Active Task

- Task ID: `DSKP-020`
- Owner: codex
- Files: `crates/nako-metadata/src/providers/douban.rs`,
  `crates/nako-metadata/src/tests.rs`, and this workstream
- Validation: focused `nako-metadata` Douban / candidate graph gates, plus
  `cargo fmt --all -- --check`
- Status: READY
- Evidence: `docs/workstreams/douban-subject-kind-precision/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- Open a new focused lane instead of reopening
  `metadata-provider-depth-and-precision`.
- Start by making Douban capability claims truthful before any future TV or
  episode breadth.
- Keep durable candidate review, schema changes, Admin/Web confirmation, graph
  preview, and child Provider Mapping writes out of this lane.

## Blockers

- None for `DSKP-020`.

## Next Recommended Action

- Run `DSKP-020`: narrow Douban capabilities and add regression coverage for
  unsupported Series/Season/Episode behavior while preserving current movie
  search/fetch and refresh behavior.
