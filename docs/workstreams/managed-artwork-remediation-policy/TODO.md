# Managed Artwork Remediation Policy Task Ledger

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Policy

- [x] MARP-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-remediation-policy,docs/workstreams/README.md]
  Goal: Open the remediation policy lane and define actionable versus advisory
  drift states.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with dry-run remediation plan and confirmed stray cleanup.

## M1 - Remediation Plan And Confirmed Stray Cleanup

- [x] MARP-020 [owner=codex] [deps=MARP-010] [scope=crates/nako-api,crates/nako-server,docs/api]
  Goal: Add a redacted Admin remediation plan and a confirmed command that
  deletes only cleanable untracked artifact files.
  Validation: focused API/server remediation tests plus relevant cargo check.
  Evidence: missing DB-backed artifacts are advisory only; untracked parseable
  files are cleanable; active artifact files are protected by re-check; reports
  remain redacted.
  Handoff: Completed. `GET /admin/v1/artwork/artifacts/remediation-plan`
  provides dry-run policy output, and
  `POST /admin/v1/artwork/artifacts/remediate-stray-files?confirm=true`
  deletes only cleanable untracked artifact files after active DB re-check.

## M2 - Validation And Closeout

- [x] MARP-030 [owner=codex] [deps=MARP-020] [scope=workspace,docs]
  Goal: Close the lane with fresh validation evidence and documented follow-ons.
  Validation: `cargo fmt --all -- --check`; focused nextest gates; relevant
  workspace `cargo check`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Completed. Continue with repair/re-ingest policy only after this
  confirmed stray cleanup boundary remains stable.
