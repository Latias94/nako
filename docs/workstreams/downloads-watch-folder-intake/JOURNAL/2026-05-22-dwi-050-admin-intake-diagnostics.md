# DWI-050 Admin Intake Diagnostics

Date: 2026-05-22
Task: DWI-050
Status: DONE

## Summary

Added an Admin-only read/control surface for acquisition intake diagnostics.
The slice exposes redacted candidate diagnostics and a watch-folder discovery
command through Admin API v1, synchronizes the generated Admin TypeScript
contract, and wires typed Admin web client/data-source/mock/UI support.

## Implementation

- Added Admin DTOs for acquisition intake candidate diagnostics and
  watch-folder discovery responses.
- Added Admin route constants and generated TypeScript contract entries:
  - `/admin/v1/acquisition/intake/candidates`
  - `/admin/v1/acquisition/intake/watch-folder-discovery`
- Added HTTP query parsing for library/state/source-kind/artifact filters.
- Added Admin HTTP routes that call `AcquisitionIntakeAppService` and convert
  app diagnostics into explicit Admin DTOs.
- Added safe `root_uri` parsing so invalid operator-submitted paths are not
  echoed in error responses.
- Updated Admin web typed client, mocks, data source, generated contract, and
  console surface for acquisition intake candidates.

## Boundary Notes

- Public Client API and `taru-client-protocol` were not changed.
- The Admin route exposes redacted source references and source-key
  fingerprints, not raw source URIs, source keys, display names, intended
  locators, diagnostics JSON, raw watch-folder roots, or downloader internals.
- The discovery command records intake candidates only. It does not create
  Managed Import artifacts, Media Sources, promotion applies, or library file
  writes.

## TDD Notes

- Red gate: `cargo nextest run -p taru-api admin_contract --no-fail-fast`
  failed until `apps/admin-web/src/adminApi/generated/contract.ts` was
  regenerated.
- HTTP acquisition-intake tests first proved missing Admin route/DTO wiring,
  then exposed an invalid `root_uri` redaction gap.

## Verification

- `cargo nextest run -p taru-api admin_contract --no-fail-fast` — pass, 5
  passed, 45 skipped.
- `cargo nextest run -p taru-api admin_acquisition --no-fail-fast` — pass, 1
  passed, 49 skipped.
- `cargo nextest run -p taru-server admin_v1_acquisition_intake --no-fail-fast`
  — pass, 2 passed, 233 skipped.
- `cargo nextest run -p taru-server http::tests::system --no-fail-fast` —
  pass, 19 passed, 216 skipped.
- `cargo nextest run -p taru-server acquisition_intake --no-fail-fast` —
  pass, 6 passed, 229 skipped.
- `npm run check` from `apps/admin-web` — pass.
- `npm test` from `apps/admin-web` — pass, 3 test files, 10 tests.
- `cargo fmt --all -- --check` — pass.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/taru-client-protocol` — no output.

## Next

Continue with DWI-060 closeout and follow-on split.
