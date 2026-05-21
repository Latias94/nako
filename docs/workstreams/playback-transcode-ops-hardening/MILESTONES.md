# Playback Transcode Ops Hardening — Milestones

Status: Active
Last updated: 2026-05-22

## M0 — Scope And Evidence Freeze

Exit criteria:

- Workstream docs exist and agree.
- Scope is limited to Playback Runtime supportability.
- Completed `transcode-runtime`, `playback-streaming`, and
  `admin-playback-runtime-diagnostics` baselines are referenced instead of
  duplicated.
- Parent `post-rpd-product-hardening` points at this lane.

Primary evidence:

- `docs/workstreams/playback-transcode-ops-hardening/DESIGN.md`
- `docs/workstreams/playback-transcode-ops-hardening/TODO.md`

## M1 — Runtime Readiness Contract

Status: completed on 2026-05-22.

Exit criteria:

- Admin playback runtime diagnostics include a stable readiness classification
  and safe reason categories.
- FFmpeg probe, hardware capability, selected fallback, budgets, staging, and
  remote playback prerequisites are explainable without raw paths.
- Existing public client playback contracts remain unchanged.

Primary evidence:

- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/app/playback`
- `crates/taru-server/src/http/tests/system.rs`

## M2 — Validation And Fallback Reasons

Status: completed on 2026-05-22.

Exit criteria:

- Invalid playback transcode request/profile combinations fail before session
  creation or FFmpeg launch.
- Hardware fallback reasons have stable categories plus operator-readable
  messages.
- Validation ownership remains in `taru-transcode` unless app/storage context
  is explicitly required.

Primary evidence:

- `crates/taru-transcode`
- `crates/taru-streaming`
- focused playback app tests

## M3 — Session Failure Taxonomy

Status: completed on 2026-05-22.

Exit criteria:

- Playback transcode failures map to support-oriented categories across probe,
  plan, staging, budget, runner, timeout, cancellation, and hardware fallback
  boundaries.
- Admin diagnostics and session read models do not leak raw paths, command
  lines, output paths, or raw stderr.
- Persisted category changes, if any, are tested for compatibility.

Primary evidence:

- `crates/taru-core`
- `crates/taru-server/src/app/playback`
- `crates/taru-server/src/http/tests`

## M4 — Support Evidence Read Model

Status: completed on 2026-05-22.

Exit criteria:

- Admin operators can retrieve bounded playback support evidence for a runtime
  or session context.
- Evidence includes useful readiness/session/staging/hardware facts.
- Evidence excludes local paths, Source Locators, FFmpeg paths, command argv,
  raw stderr, secrets, and credentials.
- Public Client API and generated client surfaces are unchanged or explicitly
  split.

Primary evidence:

- `crates/taru-api/src/admin.rs`
- `crates/taru-server/src/http/admin.rs`
- redaction tests

## M5 — Closeout And Parent Re-Score

Exit criteria:

- Final gates pass with fresh evidence.
- Workstream status and completed tasks are updated.
- Parent post-RPD umbrella re-scores downloads/watch-folder, network, AI, and
  addon runtime.
- Follow-ons are split rather than hidden in this lane.

Primary evidence:

- `docs/workstreams/playback-transcode-ops-hardening/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
