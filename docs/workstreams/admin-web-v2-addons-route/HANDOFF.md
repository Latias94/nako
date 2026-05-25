# Admin Web V2 Addons Route - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The lane is closed. `/addons` is now a route-first read-only V2 page backed by
safe Addon operation summaries and URL-owned status filtering. Addon operations
remain available in `/legacy` only for mutation and credential-producing
workflows that were intentionally deferred.

## Active Task

- Task ID: none
- Owner: codex
- Files:
  - `apps/admin-web/src/App.tsx`
  - `apps/admin-web/src/adminApi/dataSource.ts`
  - `apps/admin-web/src/features/addons/`
  - route/data-source tests
- Validation: complete; see `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: complete, no blocking findings
- Evidence: complete

## Decisions Since Last Update

- Make `/addons` a read-only route-first page first.
- Use generated `AdminAddonsQuery` for status filter.
- Defer all Addon mutations and credential-producing workflows.
- Use `AddonsRouteSummary` as the safe route read model, excluding base URLs,
  hosted page URLs, paths, env var names, snippets, commands, raw manifests,
  and raw token values.

## Blockers

- None known.

## Next Recommended Action

Split follow-on lanes for Addon mutation workflows if needed:

- registration/onboarding;
- token issue, rotation, and revoke;
- Addon Permission grant replacement;
- health-check and diagnostic actions;
- install-guide snippet presentation or export.
