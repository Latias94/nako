# Server Runtime Deepening Milestones

Status: Completed
Last updated: 2026-05-17

## M38.0 Scope And Baseline

Outcome: M38 has an explicit startup/runtime workstream and excludes client,
playback, NFO, and broad repository work.

## M38.1 Startup Workflow

Outcome: startup side effects move behind `ServerStartupWorkflow` and produce a
test-visible report.

Exit evidence:

- `NakoApp::new_with_store` composes app services and calls startup workflow.
- Startup report tests cover configured libraries, stale transcode recovery,
  staging cleanup, raw-cache cleanup, and lifecycle task registration where
  relevant.

## M38.2 Durable Job Runtime Helper

Outcome: runtime supervision has a job-specific helper and diagnostics.

Exit evidence:

- Library scan, metadata refresh, and metadata maintenance background jobs use
  the helper.
- Runtime diagnostics expose job successes and failures separately from plain
  runtime task panics.
- Existing job persisted state behavior remains compatible.

## M38.3 Closeout

Outcome: docs and gates prove the startup/runtime deepening pass.

Exit evidence:

- Workstream docs, GOALS, ROADMAP, and workstream index record M38 closeout.
- Focused server startup/runtime/metadata gates pass.
- Workspace check and nextest gates pass.
