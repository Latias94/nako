# Phase 7.5 Multi-Library Backends

## Summary

M7.5 replaces the single configured library assumption with one clean
`[[libraries]]` model. Each configured library can be local or can carry its
own WebDAV backend configuration.

## Implemented

- `NakoServerConfig` now has `libraries: Vec<LocalLibraryConfig>` as the only
  library configuration field.
- Startup persists every configured library, so `GET /libraries` and app
  workflows see more than the first configured library.
- Storage backend creation is library-aware. Scan/probe/NFO use the requested
  library ID, while playback and FFmpeg staging resolve the persisted source
  back to its configured library before opening a backend.
- `MediaSource` now carries `library_id`, so app services and API responses
  have direct source-to-library identity without paged source scans.
- WebDAV remains configured with secret references such as `password_env`;
  plaintext secrets are not stored in source locators or library roots.
- Tests cover TOML parsing, required `[[libraries]]` shape, multiple configured
  libraries, mixed local/WebDAV libraries, and remote direct-play backend
  resolution.

## Configuration Shape

Single-library and multi-library deployments both use `[[libraries]]`:

```toml
[[libraries]]
id = "018f0000-0000-7000-8000-000000000001"
name = "Movies"
root = "F:/Media/Movies"
preset = "movies"

[[libraries]]
id = "018f0000-0000-7000-8000-000000000002"
name = "Remote Anime"
root = "F:/unused"
preset = "anime"

[libraries.webdav]
root = "webdav:///Anime"
base_url = "https://nas.example.test/dav"
username = "media"
password_env = "NAKO_WEBDAV_PASSWORD"
timeout_ms = 30000
max_attempts = 2
```

## Known Limitations

- Multiple local libraries still use `local:///` source locators; callers must
  treat `source.library_id` as the disambiguating identity.
- WebDAV is still the only remote backend in this slice.

## Validation

- `cargo nextest run -p nako-server config_supports_multiple_libraries multi_library_config_registers_libraries_and_resolves_source_backend webdav_preview_config_builds_scanner_backend direct_play_holds_remote_stream_budget_until_body_is_dropped`
- `cargo check -p nako-server --tests`
