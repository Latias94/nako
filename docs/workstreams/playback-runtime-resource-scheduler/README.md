# Playback Runtime Resource Scheduler

Status: Completed
Last updated: 2026-05-29

This workstream owns the next playback runtime refactor after HLS progressive
runtime made long-running HLS sessions serve-visible before FFmpeg exits.

Nako already has several resource controls: transcode CPU/GPU limits in
`nako-transcode`, remux concurrency, remote stream/stage permits, runtime
supervision, and Admin diagnostics. The missing boundary is a host-owned
playback runtime admission model that can reason about these budgets together
before starting or reusing direct, remux, HLS, and future remote-worker
playback workloads.

This lane shipped the first single-node, FFmpeg CLI-first slice: playback
resource demand is typed in `nako-server`, HLS/remux start paths acquire
host-owned admission permits before process-backed work starts, active session
reuse does not double-acquire those permits, and Admin diagnostics expose
redaction-safe runtime pressure.

It intentionally does not introduce a distributed queue, remote transcode
workers, LL-HLS, DASH, DRM, OS-level cgroups, or per-device capacity tuning.
Those belong in follow-on lanes after this admission boundary has proven the
local host resource model.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `CLOSEOUT.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
