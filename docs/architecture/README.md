# Architecture Deep Dives

Last updated: 2026-05-29

This directory contains agent-oriented architecture maps that break the
top-level `docs/ARCHITECTURE.md` system map into functional progress trackers.

Use these documents to choose Trellis tasks, assign parallel agents, and connect
implementation tasks back to ADRs and completed evidence.

Keep `docs/ARCHITECTURE.md` short. Put capability-level status, risks, and
workstream evidence or Trellis follow-on notes in the deep dives below.

## Maps

- [Architecture lanes](LANES.md): terminal/worktree ownership registry for
  parallel Codex development by capability area.
- [Architecture workstream links](WORKSTREAM_LINKS.md): capability area to
  legacy workstream evidence and proposed lane index.
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
- [Control plane architecture](CONTROL_PLANE.md): addon lifecycle,
  observability, durable jobs, remote access, API scale, and cache-contract
  map.

## Linkage Policy

- Capability rows should link concrete ADRs and workstreams when evidence
  exists.
- Future lanes should use a `proposed:<slug>` label until a Trellis task is
  opened.
- New work should open a Trellis task and link relevant ADR, architecture, spec,
  and legacy workstream evidence from the task PRD or research notes.
- Do not duplicate task evidence in architecture docs. Link the Trellis task or
  legacy workstream evidence.
