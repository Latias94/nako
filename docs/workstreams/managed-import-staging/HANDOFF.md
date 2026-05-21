# Managed Import Staging — Handoff

Status: Active
Last updated: 2026-05-21

## Current State

The lane is open as the next post-RPD mainline after `nfo-link-authority`.
Existing VFS staging manifests provide byte-cache lifecycle primitives, but they
are not enough for product import semantics. The first implementation slice is
therefore durable Managed Import artifact domain/schema work.

## Active Task

- Task ID: MIS-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`
- Validation: `cargo nextest run -p taru-db managed_import --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Evidence: MIS-010 planning docs are complete

## Decisions

- Do not implement torrent/Usenet/downloader protocols in this lane's first
  slice.
- Do not write, copy, move, link, or delete files in media library roots during
  staging or preview.
- Do not overload `StagingManifestRecord` as the operator-facing import artifact
  model.
- Managed Import records may reference VFS staging manifests, but they own
  product import intent, target library, diagnostics, and acceptance state.
- Promotion apply remains separate until rollback, cleanup, audit, and operator
  confirmation are proven.

## Blockers

- None for MIS-020.

## Next Recommended Action

- Execute MIS-020 with TDD: add core domain records/repository trait and DB
  contract tests for durable Managed Import artifacts.