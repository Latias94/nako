# External Casting Adapter Protocol Selection

Status: Active
Last updated: 2026-05-27

## Decision

`ECAB-050` selects Chromecast as the first real external casting protocol.

The implementation boundary is:

- `nako`: addon protocol resource/scope, host renderer adapter bridge,
  policy/session/ticket authority, Admin diagnostics, and tests.
- `nako-official-addons`: official Chromecast renderer adapter sidecar,
  protocol dependency, discovery, receiver launch, command translation, smoke
  scripts, and sidecar docs.

The first real sidecar should use `oxicast` if it builds cleanly in the
official addon workspace. It is async, Tokio-native, supports mDNS discovery,
can connect by IP, launch the Default Media Receiver, load media URLs, and send
playback controls. The `serve` feature must remain disabled because Nako, not
the adapter, owns media URL exposure.

## Reference Findings

### Jellyfin

Jellyfin keeps Cast receiver app configuration as a small system-level model
and separately carries a much deeper DLNA profile and stream-building model.
The useful architectural lesson for Nako is not to copy implementation details,
but to preserve these separations:

- Cast receiver app selection/configuration is a small control-plane concern.
- DLNA renderer support is profile-heavy and compatibility-heavy.
- Stream planning has to account for container, codec, subtitle, bitrate,
  remux, and transcode constraints before the control protocol is useful.

Relevant local reference files:

- `repo-ref/jellyfin/MediaBrowser.Model/System/CastReceiverApplication.cs`
- `repo-ref/jellyfin/Jellyfin.Server/Migrations/Routines/20250420160000_AddDefaultCastReceivers.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Dlna/DeviceProfile.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Dlna/StreamBuilder.cs`

### Chromecast Rust Options

`cargo search` and `cargo info` on 2026-05-27 showed these candidates:

| Candidate | Current signal | Fit |
| --- | --- | --- |
| `oxicast` `0.0.3` | MIT/Apache-2.0, Rust 1.85, async Tokio API, mDNS discovery feature, Default Media Receiver/media controls, examples and mock-device tests. | Best first sidecar dependency if it builds in the official addon workspace. |
| `cast-sender` `0.3.0` | MIT, async CASTV2 implementation with lower-level sender/control pieces. | Good fallback if `oxicast` proves too young or too opinionated. |
| `rust_cast` `0.21.0` | MIT, established Google Cast library, but less aligned with the current async sidecar shape. | Fallback, not first choice. |
| `chromecast` `0.18.2` | Related/forked package around the `rust_cast` lineage. | Not selected; weaker boundary signal than `oxicast`. |

### DLNA Rust Options

DLNA currently looks like a composition problem rather than a single mature
renderer-adapter crate:

| Candidate | Current signal | Fit |
| --- | --- | --- |
| `ssdp-client` `2.1.0` | MIT/Apache-2.0 SSDP discovery/subscription client. | Useful building block for discovery. |
| `ssdp` `0.7.0` | MIT/Apache-2.0 SSDP discovery abstraction. | Older building block. |
| `upnp-client` `0.1.11` | MIT simple UPnP client tagged for DLNA. | Possible control-plane helper, but not enough for mature renderer support. |
| `moosicbox_upnp` `0.3.0` | MPL-2.0 MoosicBox-specific UPnP player package. | Too product-specific and license-shaped for first Nako adapter. |
| `vuio` `0.0.22` | MIT/Apache-2.0 DLNA/UPnP media server. | Server-oriented, not the first renderer-control boundary. |

DLNA should follow after Nako defines a first-class device-profile model for
external renderers. That profile work should be driven by Jellyfin-like
capability needs, but written as original Nako domain code.

## First Real Slice

`ECAB-060` should do two things:

1. In `nako`, add a first-class renderer-adapter addon resource/scope and
   typed protocol payloads for readiness, discovery, and command dispatch.
2. In `nako-official-addons`, open an official Chromecast renderer adapter
   workstream and land a sidecar skeleton that declares the renderer-adapter
   resource, validates command payloads, maps host command envelopes to a
   Chromecast command plan, and keeps live LAN/hardware execution behind
   optional smoke gates.

The first slice does not need physical Chromecast hardware in CI. Required CI
evidence is manifest validation, request/response contract tests, redaction
tests, and command-mapping tests. A live smoke script can be optional and
manually enabled.

## Deferred Protocols

- DLNA: split into a separate renderer-profile and UPnP control workstream.
- AirPlay: split after pairing/auth and platform discovery constraints are
  explicit.
- Frontend casting picker: split after host protocol diagnostics and at least
  one official adapter are stable.

## Sources

- `cargo search chromecast`, `cargo info oxicast`, `cargo info cast-sender`,
  `cargo info rust_cast`, `cargo info chromecast`.
- `cargo search ssdp`, `cargo search upnp`, `cargo info ssdp-client`,
  `cargo info ssdp`, `cargo info upnp-client`, `cargo info moosicbox_upnp`,
  `cargo info vuio`.
- `https://crates.io/crates/oxicast`
- `https://crates.io/crates/cast-sender`
- `https://crates.io/crates/rust_cast`
- `https://crates.io/crates/ssdp-client`
- `https://crates.io/crates/upnp-client`
- `https://developers.google.com/cast/docs/media`
