# Downloads / Watch-Folder Intake — TODO

Status: Active
Last updated: 2026-05-22

Task IDs use the `DWI` prefix.

## M0 — Scope And Evidence Freeze

- [x] DWI-010 [owner=planner] [deps=post-rpd PRPH-120,PTOH-060] [scope=docs/workstreams/downloads-watch-folder-intake,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open the Downloads / Watch-Folder Intake lane with acquisition-intake
  boundaries, non-goals, first executable slice, gates, and parent routing.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, parent umbrella, and workstream index agree.
  Evidence: `docs/workstreams/downloads-watch-folder-intake/DESIGN.md`.
  Handoff: Continue with DWI-020.

## M1 — Durable Intake Candidate Domain

- [x] DWI-020 [owner=codex] [deps=DWI-010] [scope=crates/taru-core,crates/taru-db]
  Goal: Add acquisition intake candidate IDs, source kinds, states, repository
  traits, SQLite/PostgreSQL migrations, and backend-neutral contract tests for
  idempotent watch-folder/operator candidates.
  Validation: `cargo nextest run -p taru-db acquisition_intake --no-fail-fast`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: `review-workstream` must check that this does not create Media
  Sources, mutate library files, or duplicate Managed Import artifact semantics.
  Evidence: core domain records, repository trait, migrations, DB contract
  tests, and no Public Client API changes.
  Handoff: DONE. Wire app-service intake and Managed Import handoff in DWI-030.

## M2 — App Service Intake And Managed Import Handoff

- [x] DWI-030 [owner=codex] [deps=DWI-020] [scope=crates/taru-server/src/app,crates/taru-server/src/app/tests]
  Goal: Add app-service methods to record/list redacted intake candidates and
  accept a candidate into an existing or new Managed Import artifact without
  promotion apply or library file mutation.
  Validation: `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`;
  focused Managed Import regression tests if shared paths change.
  Review: `review-workstream` must check idempotency, redaction, Managed Import
  handoff semantics, and no direct Media Source creation.
  Evidence: app tests proving duplicate candidate replay, redacted diagnostics,
  Managed Import artifact creation/linking, and no library mutation.
  Handoff: DONE. Add watch-folder discovery in DWI-040.

## M3 — Watch-Folder Discovery

- [x] DWI-040 [owner=codex] [deps=DWI-030] [scope=crates/taru-server/src/app,crates/taru-vfs]
  Goal: Discover watch-folder candidates through storage/VFS list/stat
  primitives, classify ready/incomplete/unsupported candidates, and write
  idempotent intake records without trusting raw host paths.
  Validation: `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`;
  `cargo nextest run -p taru-vfs --no-fail-fast` only if VFS behavior changes.
  Review: `review-workstream` must check storage boundary ownership, path
  normalization/redaction, scan idempotency, and bounded listing behavior.
  Evidence: watch-folder discovery tests with local storage fixtures, incomplete
  candidate blockers, duplicate replay, and no Media Source/promotion apply.
  Handoff: DONE. Add Admin diagnostics/read model in DWI-050.

## M4 — Admin Intake Diagnostics

- [x] DWI-050 [owner=codex] [deps=DWI-040] [scope=crates/taru-api/src/admin.rs,crates/taru-api/src/admin_contract.rs,crates/taru-server/src/http/admin.rs,crates/taru-server/src/http/tests,apps/admin-web/src/adminApi]
  Goal: Expose Admin-only intake candidate diagnostics and typed Admin web
  contract/client support without changing Public Client API or
  `taru-client-protocol`.
  Validation: `cargo nextest run -p taru-api admin_contract --no-fail-fast`;
  `cargo nextest run -p taru-server http::tests::system --no-fail-fast`;
  `npm run check` from `apps/admin-web`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must check Admin API ownership and redaction of
  raw paths, credentials, secret query strings, and downloader internals.
  Evidence: Admin DTO/contract, route tests, admin-web contract sync, and public
  client boundary check.
  Handoff: DONE. Close or split protocol downloader, UI, background scan
  scheduling, and network/AI/Addons follow-ons in DWI-060.

## M5 — Closeout And Follow-On Split

- [ ] DWI-060 [owner=planner] [deps=DWI-050] [scope=docs/workstreams/downloads-watch-folder-intake,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Verify final gates, close or split follow-ons, and re-score network
  access, AI-assisted library ops, Addon runtime/distribution, and protocol
  downloader integrations in the post-RPD umbrella.
  Validation: `verify-rust-workstream` records fresh final evidence; workstream
  JSON and parent umbrella JSON validate with `python -m json.tool`; `git diff
  --check`; `git diff --name-only -- crates/taru-client-protocol`.
  Review: `review-workstream` must have no blocking findings.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and parent umbrella
  re-score notes.
  Handoff: Return to `post-rpd-product-hardening` with the next lane decision.
