# Post-RPD Product Hardening — Milestones

Status: Active
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Exit criteria:

- Post-RPD roadmap order is explicit.
- Dependencies and non-goals are explicit.
- First execution lane is chosen.

Primary evidence:

- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `docs/workstreams/post-rpd-product-hardening/TODO.md`

## M1 — First Execution Lane Opened

Exit criteria:

- `metadata-provider-breadth` exists as a child workstream.
- Child workstream has independently validatable task slices.
- This umbrella does not contain hidden implementation work.

Primary evidence:

- `docs/workstreams/metadata-provider-breadth/DESIGN.md`
- `docs/workstreams/metadata-provider-breadth/TODO.md`

## M2 — Next Lane Decision

Exit criteria:

- Metadata provider breadth closeout evidence is reviewed.
- NFO/link, playback/transcode, and managed import staging are re-scored.
- Next execution lane is selected and either opened immediately or handed off
  as the next concrete action.

Primary evidence:

- `docs/workstreams/metadata-provider-breadth/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`

## M3 — Umbrella Closeout Or Refresh

Exit criteria:

- Active child lanes own their implementation details.
- Remaining lanes are either opened, re-scored, or intentionally deferred.
- Workstream index reflects the current active product lane.

## M4 — Post-LAIP Lane Scoring

Exit criteria:

- [x] LAIP closeout evidence is reviewed.
- [x] NFO sidecar apply, playback/transcode ops, network, AI, addon runtime,
  and downloads/watch-folder are re-scored.
- [x] Next execution lane is selected without opening a duplicate workstream.

Primary evidence:

- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/HANDOFF.md`

## M5 — Post-NSPA Lane Scoring

Exit criteria:

- [x] NFO Sidecar Promotion Apply closeout evidence is reviewed.
- [x] Playback/transcode ops, downloads/watch-folder, network, AI, and addon
  runtime are re-scored.
- [x] Next execution lane is selected without mixing library mutation scope.

Primary evidence:

- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/EVIDENCE_AND_GATES.md`

## M6 — Playback/Transcode Ops Lane Open

Exit criteria:

- [x] `playback-transcode-ops-hardening` exists as a child workstream.
- [x] Child docs define runtime/diagnostic scope, non-goals, tasks, gates, and
  handoff.
- [x] Parent umbrella and workstream index point to PTOH-020 as the next
  executable task.

Primary evidence:

- `docs/workstreams/playback-transcode-ops-hardening/DESIGN.md`
- `docs/workstreams/playback-transcode-ops-hardening/TODO.md`

## M7 — Post-Playback Ops Lane Scoring

Exit criteria:

- [x] Playback/Transcode Ops Hardening closeout evidence is reviewed.
- [x] Downloads/watch-folder, network, AI, and addon runtime are re-scored.
- [x] The next mainline lane is selected without mixing downloader protocols,
  network traversal, AI writes, or Addon runtime behavior into playback
  supportability.

Primary evidence:

- `docs/workstreams/playback-transcode-ops-hardening/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`

## M8 — Downloads / Watch-Folder Intake Lane Open

Status: completed on 2026-05-22.

Exit criteria:

- [x] `downloads-watch-folder-intake` exists as a child workstream.
- [x] Child docs define acquisition-intake scope, non-goals, tasks, gates, and
  handoff.
- [x] Parent umbrella and workstream index point to the first executable intake
  task.

Primary evidence:

- `docs/workstreams/downloads-watch-folder-intake/DESIGN.md`
- `docs/workstreams/downloads-watch-folder-intake/TODO.md`

## M9 — Post-Downloads Intake Lane Scoring

Status: completed on 2026-05-22.

Exit criteria:

- [x] Downloads / Watch-Folder Intake closeout evidence is reviewed.
- [x] Network access, AI-assisted library ops, Addon runtime/distribution,
  protocol downloader integrations, background watch scheduling, and Admin UI
  polish are re-scored.
- [x] The next mainline lane is selected without mixing remote access, AI
  writes, Addon runtime, or downloader protocols into the completed intake
  boundary.

Primary evidence:

- `docs/workstreams/downloads-watch-folder-intake/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`

## M10 — Network Access Boundary Lane Open

Status: completed on 2026-05-22.

Exit criteria:

- [x] `network-access-boundary` exists as a child workstream.
- [x] Child docs define remote endpoint/proxy/tunnel policy scope, non-goals,
  tasks, gates, and handoff.
- [x] Parent umbrella and workstream index point to NAB-020 as the next
  executable task.

Primary evidence:

- `docs/workstreams/network-access-boundary/DESIGN.md`
- `docs/workstreams/network-access-boundary/TODO.md`
