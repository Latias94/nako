# DWI-020 — Durable Intake Candidate Domain

Date: 2026-05-22

## Summary

Implemented the first acquisition intake persistence slice.

The slice adds:

- `AcquisitionIntakeCandidateId`;
- `AcquisitionIntakeSourceKind`;
- `AcquisitionIntakeCandidateState`;
- candidate list filters, new-record, and record DTOs in `nako-core`;
- `AcquisitionIntakeRepository`;
- SQLite and PostgreSQL schema/adapters;
- `NakoDatabase` facade dispatch and backend capability flag;
- backend-neutral contract coverage.

## Boundary Decisions

- Candidate records are not Media Sources.
- Repository-level Managed Import artifact linking only records the accepted
  handoff; app-service artifact creation/reuse semantics belong to DWI-030.
- No promotion apply, NFO sidecar mutation, watch-folder scan runtime, Admin
  HTTP route, downloader protocol, network traversal, AI, Addon runtime, or
  Public Client API behavior was added.

## Verification

- `cargo nextest run -p nako-db acquisition_intake --no-fail-fast` — pass, 1
  SQLite contract test; PostgreSQL paired contract is ignored unless
  `NAKO_TEST_POSTGRES_URL` is provided.
- `cargo check -p nako-db --tests` — pass.
- `cargo fmt --all -- --check` — pass.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/nako-client-protocol` — no output.

## Next

Continue with DWI-030: app-service intake and Managed Import handoff.
