# Metadata Application Cross-Path Audit - Design

Status: Complete
Last updated: 2026-05-26

## Question

After adding the server-side `MetadataApplication` Module for Addon writeback,
should provider refresh and hierarchy confirmation also call that Module?

## Finding

No code move is warranted now.

`MetadataApplication` currently belongs in `nako-server` because it owns
repository writes and catalog projection. `nako-metadata` already has its own
ports for provider refresh and hierarchy confirmation, and it cannot depend on
server app code without creating the wrong dependency direction.

The cross-path shared policy is already in `nako-core::MetadataMergePolicy`:

- provider refresh uses `from_locks_and_mode`, protecting all user/provider
  locks while applying the library refresh mode;
- hierarchy confirmation uses `for_source_refresh_mode`, allowing the same
  source to refresh its own locked fields while protecting other sources;
- Addon writeback now uses the server `MetadataApplication`, which delegates to
  the same core merge policy and then projects catalog changes.

## Boundary

Keep the current split:

- `nako-core`: pure merge/lock decision rules.
- `nako-metadata`: provider fetch, match, hierarchy confirmation, provider
  mapping ports.
- `nako-server`: orchestration that needs database adapters and catalog
  projection.

Only extract a new pure command/result type into `nako-core` if provider
refresh, hierarchy confirmation, NFO import, and Addon writes need to share the
same application-report contract without repository access.

## Non-Goals

- No dependency from `nako-metadata` to `nako-server`.
- No forced unification that hides provider matching or hierarchy-specific
  structure validation.
- No schema changes.
