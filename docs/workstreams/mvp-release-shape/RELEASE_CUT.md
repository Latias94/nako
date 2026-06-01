# MVP Release Cut

Status: Initial cut pending verification
Last updated: 2026-06-01

## Release Identity

Working name: `Nako MVP`

Release posture:

- self-hosted;
- video-first;
- single-admin;
- local-first storage;
- browser/web client path;
- FFmpeg CLI first;
- out-of-process Addon Sidecars;
- documented third-party remote access.

## Included In The Cut

| Capability | Release position |
| --- | --- |
| SQLite default persistence | Include. PostgreSQL-ready boundaries can remain documented and tested where already covered. |
| Local media library scan | Include. This is the core first-run path. |
| Existing VFS remote support | Include only when already stable; do not add a new backend for MVP. |
| Metadata authority | Include local inference, NFO, and one provider-backed path. Use review/authority language where writes are ambiguous. |
| Browser/web browse path | Include enough list/detail/search/playback entry to make the server usable. |
| Direct Play / Remux / HLS Transcode | Include with CPU fallback and safe operator diagnostics. |
| Hardware acceleration | Include as policy, capability report, and best-effort supported paths; do not make every vendor smoke matrix a release blocker. |
| Admin diagnostics | Include storage, scan/job, playback, FFmpeg, addon, and config readiness summaries where already implemented. |
| Addon Sidecar foundation | Include manual registration, health, grants/tokens, resource calls, and diagnostics. |
| Remote access | Include configuration and cookbook; exclude built-in tunnel ownership. |
| Release gates | Include focused gates and at least one container/local startup path. |

## Excluded From The Cut

- Native plugin ABI or Jellyfin plugin compatibility.
- Addon Manager process/package lifecycle.
- First-party NAT traversal or relay.
- Production mobile, TV, or desktop-native clients.
- Full provider breadth.
- Remote worker fleet.
- Offline sync.
- LL-HLS/CMAF/DASH/DRM.
- Full large-library cache contract unless a P0 route currently breaks.

## Known Acceptable Limitations

- Single-Admin Mode can ship if User, Role, and Library Access concepts remain
  intact.
- Browser playback can be the first client path if limitations are documented.
- Hardware acceleration can be diagnostic/best-effort rather than guaranteed on
  every host.
- WebDAV or other remote storage can be documented as supported preview/stable
  according to existing evidence, but MVP does not require new backend breadth.
- Addons are externally run by operators; Nako does not install or supervise
  them.

## Release Blocker Rule

Treat a gap as a blocker only when it prevents the required MVP user journey or
would make the first release unsafe, misleading, or undiagnosable.
