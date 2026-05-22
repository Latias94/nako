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
  Handoff: DONE. Playback lane completed through PTOH-060 and returned to
  parent re-score.

## M7 — Post-Playback Ops Lane Scoring

- [x] PRPH-110 [owner=planner] [deps=PTOH-060] [scope=docs/workstreams/post-rpd-product-hardening,docs/workstreams/playback-transcode-ops-hardening,docs/workstreams/README.md]
  Goal: Re-score downloads/watch-folder, network, AI, and addon runtime after
  Playback/Transcode Ops Hardening closeout.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, HANDOFF.md,
  and workstream index agree on the next executable lane.
  Evidence: `DESIGN.md` Post-PTOH Closeout Re-Score and
  `docs/workstreams/playback-transcode-ops-hardening/EVIDENCE_AND_GATES.md`.
  Handoff: Downloads/watch-folder intake is selected as the next mainline
  action because metadata authority, local file writes, staged import,
  accepted promotion apply, NFO sidecar apply, and playback supportability are
  now proven.

## M8 — Downloads / Watch-Folder Intake Lane Open

- [x] PRPH-120 [owner=planner] [deps=PRPH-110] [scope=docs/workstreams/downloads-watch-folder-intake,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open `downloads-watch-folder-intake` as a dedicated workstream for
  staged artifact acquisition intake, watch-folder candidate discovery, redacted
  diagnostics, and explicit handoff into existing promotion/apply workflows.
  Validation: Child workstream has design, TODO, milestones, gates, workstream
  metadata, and handoff docs; parent umbrella and index point to the first
  executable task.
  Evidence: `docs/workstreams/downloads-watch-folder-intake/DESIGN.md`.
  Handoff: DONE. Execute DWI-020 durable intake candidate domain without adding
  protocol-specific downloader behavior or direct library writes.

## M9 — Post-Downloads Intake Lane Scoring

- [x] PRPH-130 [owner=planner] [deps=DWI-060] [scope=docs/workstreams/post-rpd-product-hardening,docs/workstreams/downloads-watch-folder-intake,docs/workstreams/README.md]
  Goal: Re-score network access, AI-assisted library ops, Addon
  runtime/distribution, protocol downloader integrations, background watch
  scheduling, and Admin UI polish after Downloads / Watch-Folder Intake
  closeout.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, HANDOFF.md,
  and workstream index agree on the next executable lane.
  Evidence: `DESIGN.md` Post-DWI Closeout Re-Score and
  `docs/workstreams/downloads-watch-folder-intake/EVIDENCE_AND_GATES.md`.
  Handoff: DONE. Open `network-access-boundary` as the next mainline lane,
  scoped first to endpoint/proxy/tunnel policy and remote readiness without
  built-in NAT traversal runtime.

## M10 — Network Access Boundary Lane Open

- [x] PRPH-140 [owner=planner] [deps=PRPH-130] [scope=docs/workstreams/network-access-boundary,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open `network-access-boundary` as a dedicated workstream for remote
  endpoint policy, trusted proxy/header behavior, tunnel-provider readiness,
  origin constraints, and Admin-only diagnostics.
  Validation: Child workstream has design, TODO, milestones, gates,
  workstream metadata, and handoff docs; parent umbrella and index point to the
  first executable network task.
  Evidence: `docs/workstreams/network-access-boundary/DESIGN.md`.
  Handoff: DONE. Execute NAB-020 network policy domain/config validation
  without adding built-in NAT traversal runtime, identity/RBAC, downloader
  protocols, AI writes, Addon runtime, or Public Client API churn.

## M11 — Post-Network Lane Scoring

- [x] PRPH-150 [owner=planner] [deps=NAB-050] [scope=docs/workstreams/post-rpd-product-hardening,docs/workstreams/network-access-boundary,docs/workstreams/README.md]
  Goal: Re-score AI-assisted library ops, Addon runtime/distribution,
  protocol downloader integrations, concrete tunnel runtime, endpoint
  discovery, and identity/RBAC after Network Access Boundary closeout.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, HANDOFF.md,
  and workstream index agree on the next executable lane.
  Evidence: `DESIGN.md` Post-NAB Closeout Re-Score and
  `docs/workstreams/network-access-boundary/EVIDENCE_AND_GATES.md`.
  Handoff: DONE. Open `ai-assisted-library-ops` as the next mainline lane,
  scoped first to Generated Artifact proposal/readiness and acceptance planning
  without local model runtime or autonomous writes.

## M12 — AI Assisted Library Ops Lane Open

- [x] PRPH-160 [owner=planner] [deps=PRPH-150] [scope=docs/workstreams/ai-assisted-library-ops,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Open `ai-assisted-library-ops` as a dedicated workstream for Generated
  Artifact proposals, redacted diagnostics, and explicit acceptance planning.
  Validation: Child workstream has design, TODO, milestones, gates, workstream
  metadata, and handoff docs; parent umbrella and index point to the first
  executable AI task.
  Evidence: `docs/workstreams/ai-assisted-library-ops/DESIGN.md`.
  Handoff: DONE. AI lane completed through AILO-050 and returned to parent
  re-score.

## M13 — Post-AI Lane Scoring And Addon Runtime Open

- [x] PRPH-170 [owner=planner] [deps=AILO-050] [scope=docs/workstreams/post-rpd-product-hardening,docs/workstreams/ai-assisted-library-ops,docs/workstreams/addon-runtime-and-distribution,docs/workstreams/README.md]
  Goal: Re-score Addon runtime/distribution, protocol downloader integrations,
  concrete tunnel runtime, endpoint discovery, local AI runtime/vector search,
  and Public Client display after AI Assisted Library Ops closeout, then open
  the selected Addon lane.
  Validation: DESIGN.md lane table, WORKSTREAM.json continue policy, HANDOFF.md,
  child workstream docs, and workstream index agree on the next executable lane.
  Evidence: `DESIGN.md` Post-AILO Closeout Re-Score and
  `docs/workstreams/addon-runtime-and-distribution/DESIGN.md`.
  Handoff: DONE. ARD-020 completed package / install descriptor and redacted
  install-guide boundary, and ARD-030 completed Admin-only runtime readiness
  diagnostics. Execute `addon-runtime-and-distribution` ARD-040 declared
  task/event routing plans without adding Addon Manager automation, package
  signing, process supervision, Native Plugin ABI, direct library writes,
  Public Client API churn, hidden schedulers, or `taru-client-protocol`
  changes.
