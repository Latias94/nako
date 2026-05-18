# Addon Protected Writes Handoff

Status: Active
Last updated: 2026-05-18

## Current State

This lane has been split from the completed Addon Token Grants Side Effects
workstream. No protected-write apply code has been changed yet. The existing
system can authenticate Addon Tokens, enforce accepted permissions and
Library-Scoped Addon Grants, persist Addon Side Effect intake, enforce
idempotency, and return redacted intake summaries.

The remaining work is to decide and implement how accepted side effects become
concrete Taru-owned Canonical Metadata, Managed Artwork, subtitle, NFO, or
Library File Write changes.

## Active Task

- Task ID: APW-020
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-metadata`, `crates/taru-catalog`,
  `crates/taru-nfo`, `crates/taru-vfs`, `docs`
- Validation: `rg -n "side_effect|Addon Side Effect|metadata_write|artwork_write|subtitle_write|Canonical Metadata|Managed Artwork|Library File Write|NFO|subtitle|Source Locator" crates docs`; `git diff --check`
- Status: READY
- Review: classify ADR impact and scope before implementation
- Evidence: APW-020 audit notes in `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Concrete protected writes are a separate scope boundary from Addon Token,
  accepted grant, addon-principal, and intake proof.
- The first task is an audit, not immediate code, because metadata, artwork,
  subtitle, NFO, storage/VFS, and catalog seams have different owners.
- Canonical Metadata is the assumed first apply slice, but APW-020 may choose a
  narrower target if the existing metadata apply seam is not ready.
- Addon Sidecars must not receive admin tokens, raw Source Locators, filesystem
  paths, database access, or remote storage handles.
- Public Client API and generated SDK surfaces should continue excluding
  `/addon/v1/*` protected write routes.

## Blockers

- None known.

## Next Recommended Action

- Run APW-020 to audit current write seams and select the first concrete
  protected-write apply target.
