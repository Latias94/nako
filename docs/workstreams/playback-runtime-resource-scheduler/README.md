# Playback Runtime Resource Scheduler

Status: Active
Last updated: 2026-05-29

This workstream owns the next playback runtime refactor after HLS progressive
runtime made long-running HLS sessions serve-visible before FFmpeg exits.

Nako already has several resource controls: transcode CPU/GPU limits in
`nako-transcode`, remux concurrency, remote stream/stage permits, runtime
supervision, and Admin diagnostics. The missing boundary is a host-owned
playback runtime admission model that can reason about these budgets together
before starting or reusing direct, remux, HLS, and future remote-worker
playback workloads.

This lane keeps the first implementation single-node and FFmpeg CLI-first. It
does not introduce a distributed queue, LL-HLS, DASH, DRM, or OS-level cgroups.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

