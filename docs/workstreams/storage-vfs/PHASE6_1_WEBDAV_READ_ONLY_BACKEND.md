# Phase 6.1: WebDAV Read-Only VFS Backend

Status: completed.

## Goal

Add a first read-only WebDAV storage backend behind `nako-vfs` so Nako can
stat, list, and validate range-readable remote objects without plaintext
credentials in storage locators.

## Completed Shape

- Added `WebDavBackendConfig` with:
  - HTTP/HTTPS `base_url`;
  - optional `username`;
  - optional `password_env` secret reference;
  - bounded `timeout_ms`;
  - bounded `max_attempts`.
- Added `WebDavSecretResolver` and `EnvWebDavSecretResolver`.
- Added `WebDavBackend` implementing `StorageBackend`.
- Implemented WebDAV `PROPFIND` parsing for file and collection metadata.
- Implemented `stat`, `list`, and `open_range`.
- Kept the backend read-only; `write_string` returns unsupported.
- Mapped WebDAV hrefs back to `webdav:///...` locators while stripping the
  configured server base path.
- Rejected credentials embedded in WebDAV base URLs or storage URIs.

## Current Behavior

`stat` and `list` use WebDAV `PROPFIND` with depth `0` and `1`. Object metadata
includes:

- `kind`;
- length where available;
- modified timestamp;
- etag;
- fingerprint derived from etag or size/modified time;
- remote storage capabilities.

`open_range` validates a requested range against known object length when
available and returns a `VirtualFile` without `local_path_hint`. This proves the
backend does not pretend remote sources have local paths. Actual byte streaming
is deferred to M6.4.

## Credential Policy

The first backend accepts a password through `password_env`. The resolved
secret is used only to build HTTP Basic auth headers at runtime. It is not
stored in `StorageUri`, source locators, jobs, scan state, or metadata.

## Non-Goals

- No directory/stat cache yet.
- No remote probe staging yet.
- No direct remote byte streaming in HTTP playback routes yet.
- No remote write/delete support.
- No S3-compatible backend.

## Validation

Coverage:

- `nako-vfs` tests cover WebDAV stat, list, and range-open behavior with a
  mocked local WebDAV server.
- Tests verify `open_range` does not produce a local path hint.
- Tests verify secret references are resolved at runtime and do not appear in
  source locators.
- Tests verify credentials embedded in base URLs or `webdav://` locators are
  rejected.
- `nako-library` tests verify a WebDAV directory can be scanned through
  `VfsLibraryScanner`.
- Tests verify the WebDAV backend is read-only in M6.1.

Validation commands:

```text
cargo fmt --all -- --check
cargo test -p nako-vfs
cargo test -p nako-library vfs_scanner_discovers_webdav_media_without_credentials_in_locator
cargo check --workspace
```
