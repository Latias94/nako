# Addon Token Grants Side Effects Handoff

Status: Proposed
Last updated: 2026-05-18

## Current State

The focused ARF-006 lane is open. No token, grant, schema, API, or runtime
behavior has been changed yet. The current deliverable is a durable execution
plan and evidence gate set for Addon Token lifecycle, Library-Scoped Addon
Grants, and Taru-mediated Addon Side Effect intake.

## Active Task

- Task ID: ATGSE-020
- Owner: unassigned
- Files: `crates/taru-addon-protocol`, `crates/taru-core/src/addon.rs`,
  `crates/taru-core/src/repository/addon.rs`, `crates/taru-db/src/addons.rs`,
  `crates/taru-db/migrations`, `crates/taru-server/src/app/addons.rs`,
  `crates/taru-server/src/http/addons.rs`, `crates/taru-api/src/extension.rs`,
  `docs`
- Validation: `rg "Addon|addon|scope|token|grant|manifest" crates/taru-addon-protocol crates/taru-core crates/taru-db crates/taru-server crates/taru-api docs`; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: audit must decide whether ADR 0020 needs a narrow amendment before
  code changes
- Evidence: record audit notes in `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Open a new focused workstream instead of continuing to hide ARF-006 as one
  unchecked Post-M5 TODO in `addons-automation`.
- Treat Addon Tokens as addon principals, not admin credentials.
- Keep manifest-requested permissions separate from accepted runtime grants.
- Require Library-Scoped Addon Grants before enabling protected library writes
  unless a global grant is explicitly accepted.
- Route protected mutations through Addon Side Effect intake before concrete
  metadata, artwork, subtitle, or Library File Write handlers expand.

## Blockers

- None known.

## Next Recommended Action

- Run ATGSE-020: audit current addon code and docs, then record the exact
  token/grant/side-effect gaps in `EVIDENCE_AND_GATES.md` before implementing
  migrations or API changes.

