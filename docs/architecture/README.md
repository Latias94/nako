# Architecture Deep Dives

Last updated: 2026-05-29

This directory contains agent-oriented architecture maps that break the
top-level `docs/ARCHITECTURE.md` system map into functional progress trackers.

Use these documents to choose workstreams, assign parallel agents, and connect
implementation tasks back to ADRs and completed evidence.

## Maps

- [Playback architecture](PLAYBACK.md): video playback capability map,
  workstream/ADR authority, next lanes, and known risk register.
- [Storage and VFS architecture](STORAGE_VFS.md): source locator, remote
  storage, staging, mount resilience, and source identity map.
- [Library and asset pipeline architecture](LIBRARY_PIPELINE.md): scan,
  watcher, probe, metadata, artwork, and addon-assisted intake map.
- [State, database, and access architecture](STATE_ACCESS.md): persistence,
  playback state, identity, access policy, and write-pressure map.
- [Realtime and sync architecture](REALTIME_SYNC.md): realtime client updates,
  event boundaries, and offline sync map.
- [Operations and release architecture](OPERATIONS_RELEASE.md): deployment,
  release gates, diagnostics, backup, and packaging map.
