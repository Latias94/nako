# NFO Link Authority — TODO

Status: Complete
Last updated: 2026-05-21

Task IDs use the `LNA` prefix.

## M0 — Lane Open

- [x] LNA-010 [owner=planner] [deps=post-rpd PRPH-030] [scope=docs/workstreams/nfo-link-authority]
  Goal: Open the NFO/link authority lane with scope, non-goals, gates, and
  first executable slice.
  Validation: workstream docs agree and `WORKSTREAM.json` is valid JSON.
  Evidence: `DESIGN.md`, `TODO.md`, `WORKSTREAM.json`.
  Handoff: Execute LNA-020.

## M1 — VFS Link Dry-Run Contract

- [x] LNA-020 [owner=codex] [deps=LNA-010] [scope=crates/nako-vfs]
  Goal: Add a non-destructive storage link planning contract and local backend
  diagnostics for hard/soft link eligibility.
  Validation: `cargo nextest run -p nako-vfs link --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Evidence: `crates/nako-vfs/src/lib.rs`, `crates/nako-vfs/src/local.rs`.
  Handoff: Use the dry-run model to design source duplicate link evidence in LNA-030.

## M2 — Source Duplicate Link Evidence

- [x] LNA-030 [owner=codex] [deps=LNA-020] [scope=nako-core,nako-db,nako-server]
  Goal: Surface link/duplicate evidence through `SourceDuplicateRelationship`
  diagnostics without merging Media Sources or Media Items.
  Validation: source duplicate repository/app tests prove suggested
  filesystem-link evidence and no item merge.
  Evidence: `CatalogAppService::record_filesystem_link_duplicate_suggestion`
  and catalog app tests.
  Handoff: Continue to NFO authority preview in LNA-040; defer Admin API
  read model until diagnostics need operator review.

## M3 — NFO Authority Preview

- [x] LNA-040 [owner=codex] [deps=LNA-020] [scope=crates/nako-nfo]
  Goal: Expose a non-mutating NFO export/import authority preview that explains
  create, skip, forced-preserving update, backup requirement, and policy
  rejection decisions before write execution.
  Validation: focused `nako-nfo` tests prove preview does not write sidecars
  and matches export policy.
  Evidence: `NfoAuthorityPreview*` model, `NfoService::preview_authority`,
  `NfoAppService::preview_library_nfo_authority`, and focused NFO tests.
  Handoff: Decide link apply split in LNA-050; feed managed import staging and
  Admin UX after closeout.

## M4 — Link Apply Split Decision

- [x] LNA-050 [owner=planner] [deps=LNA-030,LNA-040] [scope=docs/workstreams/nfo-link-authority]
  Goal: Decide whether actual symlink/hardlink creation belongs in this lane
  or a follow-on after managed import staging opens.
  Validation: DESIGN/HANDOFF record apply, rollback, backup, and audit
  requirements.
  Evidence: DESIGN split decision and handoff follow-ons.
  Handoff: Defer actual link apply to managed-import-staging/link-apply
  follow-on.

## M5 — Closeout

- [x] LNA-060 [owner=planner] [deps=LNA-050] [scope=docs/workstreams/nfo-link-authority]
  Goal: Close or split remaining NFO/link authority work.
  Validation: evidence gates are fresh; follow-ons are explicit.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, commit history.
  Handoff: Return to `post-rpd-product-hardening` for next lane scoring.
