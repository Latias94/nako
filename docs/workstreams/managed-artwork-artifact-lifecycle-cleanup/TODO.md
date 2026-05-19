# Managed Artwork Artifact Lifecycle Cleanup Task Ledger

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Evidence Freeze

- [x] MAALC-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-artifact-lifecycle-cleanup,docs/workstreams/README.md]
  Goal: Open the lifecycle cleanup lane with scope, non-goals, evidence gates,
  and Selected Artwork retention boundary.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with Admin diagnostics and cleanup dry-run.

## M1 - Redacted Admin Lifecycle Dry Run

- [x] MAALC-020 [owner=codex] [deps=MAALC-010] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server,docs/api]
  Goal: Add a redacted Admin read model and dry-run route for Managed Artwork
  Artifact lifecycle state. The response must mark cleanup candidates by
  Selected Artwork reference count and must not delete rows or files.
  Validation: `cargo nextest run -p taru-api managed_artwork_lifecycle --no-fail-fast`;
  `cargo nextest run -p taru-db managed_artwork_lifecycle --no-fail-fast`;
  `cargo nextest run -p taru-server managed_artwork_lifecycle --no-fail-fast`;
  `cargo check -p taru-core -p taru-db -p taru-api -p taru-server --tests`.
  Evidence: tests prove protected artifacts and cleanup candidates are
  classified correctly and lifecycle responses redact storage handles, paths,
  source URLs, cache URIs, and content hashes.
  Handoff: Completed. Admin `GET /admin/v1/artwork/artifacts/lifecycle`
  returns a redacted dry-run view with summary counts, byte estimates,
  Selected Artwork reference counts, and cleanup-candidate flags. It does not
  delete rows or files. Continue with protected cleanup command.

## M2 - Protected Cleanup Command

- [x] MAALC-030 [owner=codex] [deps=MAALC-020] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server]
  Goal: Add an explicit Admin cleanup command that removes only eligible
  unselected artifacts and re-checks eligibility at deletion time.
  Validation: focused DB/server cleanup tests plus redaction tests.
  Evidence: selected artifacts survive; unselected artifacts can be removed;
  deletion reports remain redacted.
  Handoff: Completed. Cleanup uses a logical `deleted_at` repository state
  transition guarded by `NOT EXISTS selected_artworks`, hides deleted artifacts
  from active lookups, best-effort removes local artifact bytes, and reports
  redacted file cleanup counts. Continue by splitting or designing file-store
  drift inventory; do not mix thumbnail/runtime/gallery work into this lane.

## M3 - File Store Drift And Orphan File Strategy

- [ ] MAALC-040 [owner=codex] [deps=MAALC-030] [scope=crates/taru-server/src/app/artwork.rs,docs]
  Goal: Decide and, if still in scope, implement safe artifact-root inventory
  without exposing local paths in Admin responses.
  Validation: focused tests for missing DB-backed files and stray files, if
  implemented.
  Evidence: Admin diagnostics report counts/status codes, not filesystem paths.
  Handoff: Split if storage inventory becomes broader than artwork artifacts.

## M4 - Validation And Closeout

- [ ] MAALC-050 [owner=codex] [deps=MAALC-030] [scope=workspace,docs]
  Goal: Close or split the lane with fresh validation evidence and follow-ons.
  Validation: `cargo fmt --all -- --check`; focused nextest gates; relevant
  workspace `cargo check`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Close only when cleanup is safe or explicitly split with no hidden
  deletion risk.
