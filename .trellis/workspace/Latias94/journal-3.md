# Journal - Latias94 (Part 3)

> Continuation from `journal-2.md` (archived at ~2000 lines)
> Started: 2026-06-09

---



## Session 99: Move playback ticket source access into app service

**Date**: 2026-06-09
**Task**: Move playback ticket source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved browser playback ticket source Play access into PlaybackAppService validation, added app and HTTP regressions, documented the access boundary, and archived the Trellis task.

### Main Changes

- Moved HLS playlist source `Play` access into `PlaybackAppService` for new
  HLS playback and playback-session reuse.
- Added `hls_segment_playback` so HLS segment serving enforces source `Play`
  access before manifest-backed segment planning.
- Simplified `resolve_source_playback_context` into an auth/ticket-only
  resolver and removed the temporary route source-access flag.
- Added app and HTTP regressions for browse-only HLS access and HLS browser
  ticket revocation at playlist and segment use.
- Documented the HLS Playback Access Boundary in the nako-server HTTP spec and
  archived the Trellis task.

### Git Commits

| Hash | Message |
|------|---------|
| `e8d1b1ec` | (see git log) |
| `6c81ac57` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 100: Move playback decision source access into app service

**Date**: 2026-06-09
**Task**: Move playback decision source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved public playback decision source Play access into PlaybackAppService, added app and HTTP regressions, documented the decision access boundary, and archived the Trellis task.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `e5b7005b` | (see git log) |
| `21cc7bf2` | (see git log) |

### Testing

- [OK] (Add test results)

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 101: Move direct playback source access into app service

**Date**: 2026-06-09
**Task**: Move direct playback source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved Direct Play source Play access into PlaybackAppService for GET/HEAD and ticket-backed use, added app and HTTP regressions, documented the direct access boundary, and archived the Trellis task.

### Main Changes

- Moved Direct Play GET/HEAD source `Play` access from route-local checks into `PlaybackAppService`.
- Added current source `Play` access rechecks for Direct browser-ticket-backed session stream/preflight use.
- Preserved Remux/HLS route-local access during this slice through an explicit resolver transition flag.
- Added app-service and HTTP regressions for browse-only principals and revoked Direct browser tickets.
- Documented the Direct Playback Access Boundary in the nako-server backend HTTP API spec.

### Git Commits

| Hash | Message |
|------|---------|
| `cfd793e2` | (see git log) |
| `8f9fc314` | (see git log) |

### Testing

- [OK] `cargo fmt --all`
- [OK] `cargo nextest run -p nako-server direct_stream --no-fail-fast`
- [OK] `cargo nextest run -p nako-server direct_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`
- [OK] `cargo nextest run -p nako-server browser_playback_ticket_rejects_browse_only_access_and_revocation_at_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server playback_routes_require_play_library_access --no-fail-fast`
- [OK] `cargo check -p nako-server --tests`
- [OK] `git diff --check`
- [OK] `python .\.trellis\scripts\task.py validate 06-09-direct-playback-source-access-app`

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 102: Move remux playback source access into app service

**Date**: 2026-06-09
**Task**: Move remux playback source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved Remux source Play access into PlaybackAppService for GET/HEAD and ticket-backed use, added app and HTTP regressions, documented the Remux access boundary, and archived the Trellis task.

### Main Changes

- Moved Remux GET/HEAD source `Play` access from route-local checks into `PlaybackAppService`.
- Added current source `Play` access rechecks for Remux browser-ticket-backed session stream use.
- Preserved HLS/subtitle/renderer transport route-local access for dedicated follow-up slices.
- Added app-service and HTTP regressions for browse-only principals and revoked Remux browser tickets.
- Documented the Remux Playback Access Boundary in the nako-server backend HTTP API spec.

### Git Commits

| Hash | Message |
|------|---------|
| `64ea0754` | (see git log) |
| `baad19f8` | (see git log) |

### Testing

- [OK] `cargo fmt --all`
- [OK] `cargo nextest run -p nako-server remux_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`
- [OK] `cargo nextest run -p nako-server playback_routes_require_play_library_access --no-fail-fast`
- [OK] `cargo nextest run -p nako-server remux_browser_playback_ticket_rejects_revocation_at_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server remux_stream --no-fail-fast`
- [OK] `cargo nextest run -p nako-server remux --no-fail-fast`
- [OK] `cargo check -p nako-server --tests`
- [OK] `git diff --check`
- [OK] `python .\.trellis\scripts\task.py validate 06-09-remux-playback-source-access-app`

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 103: Move hls playback source access into app service

**Date**: 2026-06-09
**Task**: Move hls playback source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved HLS source Play access into PlaybackAppService for playlist and segment use, added app and HTTP regressions, documented the HLS access boundary, removed the source resolver access flag, and archived the Trellis task.

### Main Changes

(Add details)

### Git Commits

| Hash | Message |
|------|---------|
| `2ee94772` | refactor(server): move hls playback access into app service |
| `4a09fd60` | chore(task): archive hls playback access task |

### Testing

- [OK] `cargo fmt --all`
- [OK] `cargo nextest run -p nako-server hls_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`
- [OK] `cargo nextest run -p nako-server playback_routes_require_play_library_access --no-fail-fast`
- [OK] `cargo nextest run -p nako-server hls_browser_playback_ticket_rejects_revocation_at_playlist_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server hls_browser_playback_ticket_rejects_revocation_at_segment_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
- [OK] `cargo nextest run -p nako-server hls_segment --no-fail-fast`
- [OK] `cargo check -p nako-server --tests`
- [OK] `git diff --check`
- [OK] `python .\.trellis\scripts\task.py validate 06-09-hls-playback-source-access-app`

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 104: Move subtitle playback source access into app service

**Date**: 2026-06-10
**Task**: Move subtitle playback source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved subtitle source Play access into PlaybackAppService, made subtitle principal resolution auth/ticket-only, added app and HTTP regressions for browse-only and ticket revocation use, documented the subtitle access boundary, and archived the Trellis task.

### Main Changes

- Moved sidecar subtitle source `Play` access into `PlaybackAppService::subtitle_playback`.
- Simplified subtitle HTTP principal resolution to auth/ticket identity only.
- Added app-service and HTTP regressions for Browse-only denial and subtitle ticket revocation at use time.
- Documented the subtitle playback access boundary in the nako-server HTTP API spec.

### Git Commits

| Hash | Message |
|------|---------|
| `bd855bae` | refactor(server): move subtitle playback access into app service |
| `8c7f3141` | chore(task): archive subtitle playback access task |

### Testing

- [OK] `cargo fmt --all`
- [OK] `cargo nextest run -p nako-server subtitle_playback_rejects_browse_only_access_before_policy_details --no-fail-fast`
- [OK] `cargo nextest run -p nako-server subtitle_route_requires_play_library_access --no-fail-fast`
- [OK] `cargo nextest run -p nako-server subtitle_browser_playback_ticket_rejects_revocation_at_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server subtitle --no-fail-fast`
- [OK] `cargo nextest run -p nako-server browser_playback_ticket_streams_sidecar_subtitle_without_bearer --no-fail-fast`
- [OK] `cargo check -p nako-server --tests`
- [OK] `git diff --check`
- [OK] `python .\.trellis\scripts\task.py validate 06-09-subtitle-playback-source-access-app`

### Status

[OK] **Completed**

### Next Steps

- None - task complete


## Session 105: Move renderer transport source access into app service

**Date**: 2026-06-10
**Task**: Move renderer transport source access into app service
**Package**: nako-server
**Branch**: `main`

### Summary

Moved renderer transport ticket use source Play access out of the HTTP resolver and into existing PlaybackAppService session-use methods, removed the dead route-local source access helper, added Direct/Remux/HLS renderer transport revocation regressions, documented the renderer transport access boundary, and archived the Trellis task.

### Main Changes

- Made renderer transport principal resolution auth/ticket-only for playback use routes.
- Delegated current source `Play` rechecks to existing Direct, Remux, and HLS playback app-service session-use methods.
- Removed the now-unused HTTP route-local source access helper.
- Added renderer transport revocation regressions for Direct, Remux, and HLS playlist use.
- Documented the renderer transport access boundary in the nako-server HTTP API spec.

### Git Commits

| Hash | Message |
|------|---------|
| `c197bdee` | refactor(server): move renderer transport access into app service |
| `82b38668` | chore(task): archive renderer transport access task |

### Testing

- [OK] `cargo fmt --all`
- [OK] `cargo nextest run -p nako-server renderer_transport_direct_rejects_revoked_source_play_access_at_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server renderer_transport_remux_rejects_revoked_source_play_access_at_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server renderer_transport_hls_rejects_revoked_source_play_access_at_playlist_use --no-fail-fast`
- [OK] `cargo nextest run -p nako-server renderer_play_command_with_cast_ticket --no-fail-fast`
- [OK] `cargo nextest run -p nako-server synthetic_external_adapter_play_command_receives_cast_safe_transport_envelope --no-fail-fast`
- [OK] `cargo check -p nako-server --tests`
- [OK] `git diff --check`
- [OK] `python .\.trellis\scripts\task.py validate 06-10-renderer-transport-source-access-app`

### Status

[OK] **Completed**

### Next Steps

- None - task complete
