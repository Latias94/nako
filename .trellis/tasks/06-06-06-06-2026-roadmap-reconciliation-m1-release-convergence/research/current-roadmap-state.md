# Current Roadmap State

## Sources Read

- `CONTEXT.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LANES.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/mvp-release-shape/CLOSEOUT.md`

## Findings

- The domain language is mature: Nako distinguishes Media Library, Media
  Source, Source Fingerprint, Source Duplicate Relationship, Media Item,
  Provider Mapping, Addon Sidecar, and Nako-managed artifacts.
- Architecture principles are stable and useful for roadmap decisions:
  Direct Play first, planner before runtime, manifest-backed artifacts,
  FFmpeg CLI first, resource budgets as product behavior, explicit local
  authority, out-of-process addons, and explicit control-plane boundaries.
- The lane map is current enough to plan from. All major lanes are idle, and
  the active queue says no implementation lane is selected.
- The roadmap and goals files preserve extensive historical completion
  evidence, especially around metadata provider governance, storage/source hash,
  playback/transcode, and release shape.
- The reading path is too historical for future planning. A new contributor or
  agent must scan many completed slices before seeing the current product
  direction.
- The previous MVP release shape closeout validated a release ladder but did
  not mean the product roadmap has an obvious next implementation queue.

## Implications

- The next roadmap update should put the current release target first and push
  historical evidence behind concise references.
- M1 should be framed as a coherent product journey, with validation gates
  inherited from release-shape evidence.
- Candidate implementation queues should be lane-owned and narrow. The next
  plan should not reopen closed workstreams or start parallel terminals before
  a focused Trellis task exists.

## Candidate M1 Queue

- Storage/VFS: scan-originated source hash triggering and source duplicate
  relationship reconciliation boundaries.
- Library/Catalog: library intake and catalog browse smoke from a fresh
  library.
- Playback/Transcode: player-facing direct/remux/HLS happy path plus error
  recovery and resource pressure guardrails.
- Web Product: Media Web/Admin operator journey smoke and generated contract
  alignment.
- Operations/Control Plane: one-command release-readiness ladder or documented
  gate runner if product release execution is selected.
