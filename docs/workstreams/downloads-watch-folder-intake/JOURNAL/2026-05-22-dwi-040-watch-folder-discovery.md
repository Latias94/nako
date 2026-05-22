# DWI-040 Watch-Folder Discovery

Date: 2026-05-22
Task: DWI-040
Status: DONE

## Summary

Added watch-folder discovery to the acquisition intake app-service boundary.
Discovery uses configured storage/VFS `stat` and `list` operations, classifies
candidate entries, and writes only acquisition-intake records.

## Implementation

- Added `DiscoverWatchFolderCandidatesRequest` and
  `WatchFolderDiscoveryDiagnostic`.
- Wired `AcquisitionIntakeAppService` to receive the existing
  `StorageBackendRegistry` from `NakoAppServices`.
- `discover_watch_folder_candidates` resolves the target library and root URI,
  obtains the configured library storage backend, traverses via VFS list/stat,
  classifies entries as:
  - `Ready` for supported media extensions;
  - `Blocked` / `incomplete` for partial download extensions;
  - `Blocked` / `unsupported` for unsupported file extensions.
- Discovery writes candidates through `record_candidate`, preserving idempotent
  source-key behavior and redacted diagnostics.
- Discovery does not create Managed Import artifacts, promotion applies, Media
  Sources, or Library File Writes.

## TDD Notes

- Red gate: `cargo nextest run -p nako-server acquisition_intake_watch_folder
  --no-fail-fast` failed because the request type and app-service method did
  not exist.
- Added the watch-folder fixture test and implementation, then verified the full
  acquisition-intake app gate.

## Verification

- `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` — pass,
  4 passed, 229 skipped.
- `cargo nextest run -p nako-vfs --no-fail-fast` — pass, 45 passed.
- `cargo nextest run -p nako-db acquisition_intake --no-fail-fast` — pass, 1
  passed, 123 skipped.
- `cargo check -p nako-server --tests` — pass.
- `cargo fmt --all -- --check` — pass after formatting.
- `git diff --check` — pass with repository CRLF conversion warnings only.
- `git diff --name-only -- crates/nako-client-protocol` — no output.

## Next

Continue with DWI-050: Admin-only intake diagnostics/read model and typed Admin
web contract/client support, with Public Client API and `nako-client-protocol`
unchanged.
