# External Casting Adapter Follow-ons

Status: Proposed
Last updated: 2026-05-27

ECAB is closed after the host boundary, synthetic adapter proof, addon protocol
surface, official catalog descriptor, and first official Chromecast sidecar
slice. The remaining work is intentionally split so each lane has a clear
authority boundary.

## F1 - Live Chromecast Control Hardening

Repository: `nako-official-addons`

Scope:

- Add richer live-control result telemetry to `nako-chromecast-renderer`.
- Add bounded device cache and safe failure history.
- Add optional hardware smoke documentation for play/pause/seek/stop/volume.
- Keep LAN addresses out of health/resource responses unless explicitly
  fingerprinted.

Non-goals:

- Host playback policy changes.
- DLNA/AirPlay support.

## F2 - DLNA Renderer Profile And UPnP Control

Repository: `nako` first, then `nako-official-addons`

Scope:

- Design a Nako-native renderer device-profile model for container, codec,
  subtitle, bitrate, remux, and transcode constraints.
- Use Jellyfin as product reference only; write original Nako profile types and
  tests.
- After profiles exist, add a UPnP/DLNA official adapter sidecar.

Non-goals:

- Copying Jellyfin DLNA profile schemas or code.
- Treating SSDP discovery alone as mature DLNA renderer support.

## F3 - AirPlay Feasibility

Repository: likely `nako-official-addons`

Scope:

- Research current pairing/auth/discovery constraints.
- Decide whether AirPlay belongs in the official sidecar set or remains
  experimental.
- Document platform and legal/licensing risks before code.

## F4 - Frontend Casting Picker

Repository: `nako`

Scope:

- Add Media/Admin Web casting picker and management context links once at least
  one official adapter is stable.
- Keep renderer adapter diagnostics reachable from media browsing when scan,
  policy, or device readiness needs operator action.

Non-goals:

- Redesigning the full player UI in this backend-focused lane.

## F5 - Network Trust And Remote Casting Policy

Repository: `nako`

Scope:

- Define local subnet, VPN, remote relay, and public internet casting policy.
- Decide when remote external renderers are allowed.
- Add auditing and admin diagnostics for network trust decisions.

Non-goals:

- Letting adapters bypass Nako policy/session/ticket authority.
