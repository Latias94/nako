# Phase 7.4.1: NFO Storage Boundary

Status: completed for M7.

## Goal

Make NFO import/export use the same VFS backend boundary as scan, probe, and
playback instead of hard-coding `LocalFsBackend` from the configured local
root.

## Implemented Shape

- `run_nfo_import` now builds the backend with
  `storage_backend_for_library_root`.
- `run_nfo_export` now builds the backend with
  `storage_backend_for_library_root`.
- NFO export checks the root backend metadata and requires
  `StorageCapabilities::WRITABLE` before calling `NfoService`.
- WebDAV import can read remote `.nfo` sidecars through `nako-vfs`.
- WebDAV export fails before writing because the current WebDAV backend is
  intentionally read-only.

## Validation

- `cargo nextest run -p nako-server nfo_import_uses_configured_webdav_backend nfo_export_rejects_read_only_webdav_backend nfo_import_job_imports_sidecar_and_persists_summary nfo_routes_queue_background_jobs`

## Remaining Gaps

- Remote NFO export remains blocked for WebDAV because the current WebDAV
  backend is read-only.
