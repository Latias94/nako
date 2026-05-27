# External Casting Adapter Boundary

Status: Active
Last updated: 2026-05-27

## Purpose

This workstream turns Nako's host-owned renderer transport primitive into a
safe path for external casting protocols such as Chromecast, DLNA, and AirPlay.

The first goal is not to copy a full protocol stack into `nako-server`. The
first goal is to define and prove the adapter boundary: Nako owns policy,
Renderer Sessions, Playback Sessions, cast-safe transport tickets, and
redaction; protocol adapters own discovery and device-specific control.

## Boundaries

In scope:

- external protocol renderer adapter contract;
- synthetic adapter proof before real LAN protocol work;
- Admin diagnostics that distinguish ready Nako transport from planned external
  adapters;
- first real protocol implementation selection after the host boundary is
  proven.

Out of scope:

- moving Chromecast, DLNA, or AirPlay discovery into playback planning;
- giving adapters bearer tokens, Source Locators, local paths, or Transcode
  Session IDs as credentials;
- frontend casting picker UI;
- mobile native receiver/client work;
- SyncPlay, queues, subtitles, and watch-party semantics.

## First Executable Task

Start with `ECAB-020`: characterize the current external adapter boundary.
Prove external protocol renderer registration is rejected, Admin diagnostics
keep Chromecast/DLNA/AirPlay planned, and no diagnostic surface leaks transport
credentials.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [PROTOCOL_SELECTION.md](PROTOCOL_SELECTION.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [ADR 0042](../../adr/0042-sidecar-renderer-adapters-for-external-casting-protocols.md)
- [ADR 0043](../../adr/0043-ship-chromecast-first-as-official-renderer-adapter.md)
