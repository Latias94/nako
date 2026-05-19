# Managed Artwork Artifact Store Drift Inventory Task Ledger

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Split

- [x] MASDI-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-artifact-store-drift-inventory,docs/workstreams/managed-artwork-artifact-lifecycle-cleanup,docs/workstreams/README.md]
  Goal: Split artifact-store drift inventory from lifecycle cleanup and record
  the read-only diagnostics scope.
  Validation: Workstream docs exist and agree; lifecycle cleanup points to this
  follow-on.
  Evidence: `DESIGN.md`, `WORKSTREAM.json`, lifecycle cleanup handoff.
  Handoff: Continue with the first Admin diagnostics slice.

## M1 - Read-Only Drift Diagnostics

- [x] MASDI-020 [owner=codex] [deps=MASDI-010] [scope=crates/taru-api,crates/taru-server,docs/api]
  Goal: Add a redacted Admin storage drift diagnostics route that checks
  DB-backed active artifact files and bounded artifact-root stray files without
  deleting or repairing anything.
  Validation: focused API/server drift tests plus relevant cargo check.
  Evidence: missing DB-backed artifacts are reported; stray files are counted
  and classified; responses do not leak storage handles, paths, filenames,
  source/cache URLs, provider query strings, addon tokens, or content hashes.
  Handoff: Completed. `GET /admin/v1/artwork/artifacts/storage-drift` returns
  bounded read-only diagnostics with missing DB-backed artifact rows, stray file
  classifications, file scan truncation status, and redacted safe facts only.

## M2 - Validation And Closeout

- [x] MASDI-030 [owner=codex] [deps=MASDI-020] [scope=workspace,docs]
  Goal: Close or split the lane with fresh validation evidence and documented
  follow-ons.
  Validation: `cargo fmt --all -- --check`; focused nextest gates; relevant
  workspace `cargo check`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Completed. Split repair, deletion, and re-ingest remediation into
  future lanes; this lane remains diagnostics-only.
