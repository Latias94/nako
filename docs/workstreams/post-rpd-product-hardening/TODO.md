# Post-RPD Product Hardening — TODO

Status: Active
Last updated: 2026-05-22

Task IDs use the `PRPH` prefix. This is an umbrella roadmap, so implementation
tasks belong in child workstreams.

## M0 — Roadmap Freeze

- [x] PRPH-010 [owner=planner] [deps=none] [scope=docs/workstreams/post-rpd-product-hardening]
  Goal: Freeze the post-RPD lane order, dependency map, and non-goals.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
  Handoff: Open the first execution lane.

## M1 — First Execution Lane

- [x] PRPH-020 [owner=planner] [deps=PRPH-010] [scope=docs/workstreams/metadata-provider-breadth]
  Goal: Open `metadata-provider-breadth` as the first Wave 1 execution workstream.
  Validation: Child workstream has design, TODO, milestones, gates, workstream metadata, and handoff docs.
  Evidence: `docs/workstreams/metadata-provider-breadth/DESIGN.md`
  Handoff: Completed by `metadata-provider-breadth` closeout; continue with PRPH-030 next-lane scoring.

## M2 — Follow-On Lane Scoring

- [x] PRPH-030 [owner=planner] [deps=metadata-provider-breadth closeout] [scope=docs/workstreams/post-rpd-product-hardening]
  Goal: Re-score NFO/link authority, playback/transcode hardening, and managed import staging after metadata breadth lands.
  Validation: DESIGN.md lane table updated with new evidence and next lane decision.
  Evidence: `DESIGN.md` Post-Metadata Re-Score and child closeout evidence in `metadata-provider-breadth/EVIDENCE_AND_GATES.md`.
  Handoff: Open `nfo-link-authority` as the next execution workstream; keep playback/transcode ops as a disjoint sidecar candidate.

## M3 — Umbrella Closeout

- [ ] PRPH-040 [owner=planner] [deps=PRPH-030] [scope=docs/workstreams/post-rpd-product-hardening]
  Goal: Close or refresh the umbrella once the active product lanes are represented by dedicated workstreams.
  Validation: Fresh review of active/deferred lanes and workstream index.
  Evidence: EVIDENCE_AND_GATES.md and HANDOFF.md.
  Handoff: Keep this umbrella active only while it reduces coordination cost.

## M4 — Post-LAIP Lane Scoring

- [x] PRPH-080 [owner=planner] [deps=LAIP-080] [scope=docs/workstreams/post-rpd-product-hardening]
  Goal: Re-score NFO sidecar apply, playback/transcode ops, network, AI, addon
  runtime, and downloads/watch-folder after Managed Import promotion apply
  closeout.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, and
  HANDOFF.md agree on the next executable lane.
  Evidence: Completed in the Post-LAIP Closeout Re-Score section of
  `DESIGN.md`. `nfo-sidecar-promotion-apply` is selected as the next mainline
  lane because it is the remaining local Library File Write and
  metadata-authority mutation boundary before downloads, AI, or Addon file
  writes deepen.
  Handoff: Execute `nfo-sidecar-promotion-apply` NSPA-020.

## M5 — Post-NSPA Lane Scoring

- [x] PRPH-090 [owner=planner] [deps=NSPA-070] [scope=docs/workstreams/post-rpd-product-hardening]
  Goal: Re-score playback/transcode ops, downloads/watch-folder, network, AI,
  and addon runtime after NFO Sidecar Promotion Apply closeout.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, and
  HANDOFF.md agree on the next executable lane.
  Evidence: Completed in the Post-NSPA Closeout Re-Score section of
  `DESIGN.md`. Playback/Transcode Ops Hardening is selected as the next
  mainline lane because local metadata, sidecar, import, file-write, rollback,
  and repair boundaries are now proven. Downloads/watch-folder, network, AI,
  and addon runtime remain downstream or parallel only if they consume existing
  accepted boundaries.
  Handoff: Open `playback-transcode-ops-hardening`.

## M6 — Playback/Transcode Ops Lane Open

- [x] PRPH-100 [owner=planner] [deps=PRPH-090] [scope=docs/workstreams/playback-transcode-ops-hardening,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open `playback-transcode-ops-hardening` as the next mainline
  execution lane without duplicating completed M7/M25/M56 playback runtime
  work.
  Validation: Child workstream has design, TODO, milestones, gates,
  workstream metadata, and handoff docs; parent umbrella and index point to
  PTOH-020 as the next executable task.
  Evidence: `docs/workstreams/playback-transcode-ops-hardening/DESIGN.md`
  Handoff: Execute PTOH-020 playback runtime readiness contract.
