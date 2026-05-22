# DWI-030 App-Service Intake And Managed Import Handoff

Date: 2026-05-22
Task: DWI-030
Status: DONE

## Summary

Added the server app-service seam for acquisition intake candidates. The seam
records and lists redacted candidate diagnostics and accepts candidates into
Managed Import artifacts without applying promotion, creating Media Sources, or
writing library files.

## Implementation

- Added `AcquisitionIntakeAppService` under `crates/taru-server/src/app`.
- Wired the service into `TaruApp` and `TaruAppServices` composition.
- Added redacted `AcquisitionIntakeCandidateDiagnostic` and acceptance
  diagnostics.
- `record_candidate` validates the target library, trims required fields,
  idempotently upserts by source key through the repository, and exposes only
  redacted/summarized evidence.
- `accept_candidate` validates candidate state and can:
  - link a requested existing Managed Import artifact;
  - reuse an existing same-source Managed Import artifact;
  - create a new `Proposed` Managed Import artifact through
    `ManagedImportAppService`.
- Acceptance links the candidate to the artifact and marks it accepted, but does
  not create promotion applies, Media Sources, or library file writes.

## TDD Notes

- Initial `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`
  failed because there were no acquisition-intake app tests.
- After adding tests, same-source candidate acceptance failed on the Managed
  Import artifact uniqueness constraint. This proved missing artifact reuse.
- Implemented same-source artifact reuse and explicit existing-artifact linking,
  then expanded tests to cover both paths.

## Verification

- `cargo nextest run -p taru-server acquisition_intake --no-fail-fast` — pass,
  3 passed, 229 skipped.
- `cargo nextest run -p taru-server managed_import --no-fail-fast` — pass, 18
  passed, 214 skipped.
- `cargo nextest run -p taru-db acquisition_intake --no-fail-fast` — pass, 1
  passed, 123 skipped.
- `cargo check -p taru-server --tests` — pass.
- `cargo fmt --all -- --check` — pass.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/taru-client-protocol` — no output.

## Next

Continue with DWI-040: watch-folder discovery through storage/VFS list/stat
boundaries, writing idempotent intake records without trusting raw host paths.
