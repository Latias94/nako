# Library Metadata Scan Policy - TODO

Status: Closed
Last updated: 2026-05-25

Task IDs use the `LMSP` prefix.

## M0 - Scope And Evidence Freeze

- [x] LMSP-010 [owner=codex] [deps=none] [scope=docs/workstreams/library-metadata-scan-policy]
  Goal: Freeze the scan-time metadata acquisition problem, target state,
  non-goals, Jellyfin reference lesson, and first NFO-only execution slice.
  Validation: workstream docs exist and agree.
  Evidence: `DESIGN.md`, `TODO.md`, `MILESTONES.md`,
  `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Continue with LMSP-020.

## M1 - Metadata Acquisition Plan

- [x] LMSP-020 [owner=codex] [deps=LMSP-010] [scope=crates/nako-core,crates/nako-server/src/config.rs,crates/nako-api]
  Goal: Add a library/profile-level scan-time metadata acquisition model that
  can express local NFO import now and future provider/addon/embedded readers
  later without hard-coding NFO in the scan service.
  Validation: focused config/core/API tests for default presets, TOML override
  behavior, and Public Client profile DTO shape if changed.
  Review: The model must preserve existing preset behavior and keep disabled
  local metadata from running during scan.
  Evidence: focused tests and docs notes.
  Result: DONE 2026-05-25. `MetadataProfile` now exposes a
  `scan_acquisition_plan`; default video presets plan local NFO import, while
  disabled local metadata, missing NFO readers, or disabled scan metadata skip
  it. Server config now supports per-library metadata profile overrides under
  `metadata.library_profiles`.
  Handoff: Continue with LMSP-030 after the plan can decide whether NFO import
  is enabled.

## M2 - Scan-Time NFO Import

- [x] LMSP-030 [owner=codex] [deps=LMSP-020] [scope=crates/nako-server/src/app/jobs.rs,crates/nako-server/src/app/nfo.rs,crates/nako-server/src/app/tests]
  Goal: Make library scan execute the profile-derived metadata acquisition
  plan after index/probe, with NFO Import as the first concrete step.
  Validation: focused server app/HTTP tests proving scan imports NFO when
  enabled, skips it when disabled, and keeps manual `import-nfo` working.
  Review: Reuse NFO app/service boundaries; do not duplicate sidecar discovery,
  XML parsing, merge policy, or catalog/search commit ordering in scan.
  Evidence: test output and updated summaries.
  Result: DONE 2026-05-25. Library scan now runs scan-time NFO Import when
  the Metadata Profile acquisition plan enables it, records an NFO import
  summary in scan output/job summary, and skips the step when scan metadata is
  disabled.
  Handoff: Continue with LMSP-040 real-directory validation.

## M3 - Real Directory Smoke

- [x] LMSP-040 [owner=codex] [deps=LMSP-030] [scope=target/nako-smoke,docs/workstreams/library-metadata-scan-policy]
  Goal: Re-run local and NAS narrow smoke with temporary configs proving scan,
  scan-time metadata acquisition, playback decision, and Range streaming.
  Validation: local `H:\Super\Videos` scan shows NFO metadata applied without a
  separate `import-nfo`; NAS single-directory smoke still direct-plays over SMB.
  Review: Do not scan the full NAS root until progress/cancellation visibility
  is stronger.
  Evidence: command summaries in `EVIDENCE_AND_GATES.md`.
  Result: DONE 2026-05-25. Local `H:\Super\Videos` scan discovered 5 videos,
  probed 5, and automatically imported 3 NFO sidecars. NAS single-directory
  SMB smoke discovered/probed/imported 1 source and direct-played over Range.
  Handoff: Continue with LMSP-050 closeout.

## M4 - Closeout Or Follow-On Split

- [x] LMSP-050 [owner=codex] [deps=LMSP-020,LMSP-030,LMSP-040] [scope=docs/workstreams/library-metadata-scan-policy]
  Goal: Verify final gates, close the NFO scan-time metadata slice, and split
  provider refresh, Addon Bulk Metadata Scrape, embedded readers, image
  discovery, and NAS large-library progress into explicit follow-ons.
  Validation: fresh verification evidence and workstream docs updated.
  Review: No completion claim without current gates and real-directory smoke.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Result: DONE 2026-05-25. The NFO scan-time acquisition slice is closed.
  Follow-ons are deferred for provider refresh, Addon scrape tasks, embedded
  readers, image discovery, richer priority controls, and full NAS root
  progress/cancellation visibility.
