# Nako Renderer Cast-Safe Transport

Status: Active
Last updated: 2026-05-27

## Purpose

This workstream implements the first non-direct renderer media transport for
Nako remote clients. It keeps the Public Client renderer control plane on
bearer-authenticated routes while adding renderer-scoped media URLs for direct,
remux, and HLS playback decisions.

The lane exists because the completed casting runtime can queue direct-play
commands, but it intentionally rejects renderer remux/HLS decisions. Future
Chromecast, DLNA, and AirPlay adapters need the same host-owned transport
primitive, so Nako should solve it once before protocol-specific work starts.

## Boundaries

In scope:

- renderer/cast-safe transport ticket semantics;
- Nako remote-client media transport envelope for renderer commands;
- remux/HLS renderer playback through existing Playback Session and transcode
  runtime boundaries;
- redaction-safe Public and Admin contract updates;
- tests that prevent browser tickets, bearer tokens, Source Locators, local
  paths, and Transcode Session IDs from becoming renderer media credentials.

Out of scope:

- Chromecast, DLNA, AirPlay, SyncPlay, queues, subtitles, receiver app
  configuration, and external discovery;
- frontend playback UI wiring;
- long-term persistent ticket storage beyond the server-owned contract needed
  for this lane;
- copying protocol behavior or source from reference repositories.

## First Executable Task

Start with `NRCT-020` characterization. Prove the current direct-only renderer
behavior, the current Nako renderer transport-auth registration boundary, and
the reason browser playback tickets cannot be reused as renderer cast tickets.

## Authoritative Docs

- [DESIGN.md](DESIGN.md)
- [TODO.md](TODO.md)
- [MILESTONES.md](MILESTONES.md)
- [EVIDENCE_AND_GATES.md](EVIDENCE_AND_GATES.md)
- [HANDOFF.md](HANDOFF.md)
- [WORKSTREAM.json](WORKSTREAM.json)
- [ADR 0041](../../adr/0041-renderer-cast-safe-transport-tickets.md)
