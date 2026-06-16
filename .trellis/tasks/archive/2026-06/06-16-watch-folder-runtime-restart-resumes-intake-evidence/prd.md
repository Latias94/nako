# Watch Folder Runtime Restart Resumes Intake Evidence

## Problem

Watch-folder intake already debounces stable media observations and coalesces
newly ready candidates into the durable library scan queue. The remaining
self-hosted reliability gap is restart continuity: after Nako observes a media
file once and stops, the next process should resume from the persisted
candidate evidence instead of starting the stability window from scratch.

## Requirements

- Prove a first watch-folder tick records an `Inspecting` candidate and enqueues
  no `LibraryScan` job.
- Prove a restarted `NakoApp` using the same store treats the repeated
  observation as newly stable and enqueues exactly one `JobKind::LibraryScan`
  through the existing watch-folder admission path.
- Preserve existing watch-folder redaction and durable-job boundaries.
- Do not introduce a new watcher loop, scan executor, schema migration, or Admin
  API shape.

## Scope Boundaries

- In scope: focused `nako-server` app test coverage for restart continuity.
- Out of scope: addon lifecycle, frontend UI, filesystem event backends,
  queued/running scan reuse across startup, and broad scheduler rewrites.

## Verification

- `cargo nextest run -p nako-server watch_folder_runtime_tick_resumes_inspecting_candidate_after_restart --no-fail-fast`
- `cargo check -p nako-server --tests`
- `cargo fmt --all -- --check`
- `python ./.trellis/scripts/task.py validate .trellis/tasks/06-16-watch-folder-runtime-restart-resumes-intake-evidence`
