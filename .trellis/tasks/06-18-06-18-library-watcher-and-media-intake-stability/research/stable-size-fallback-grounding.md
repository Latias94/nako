# Stable-Size Fallback Grounding

Date: 2026-06-18

## Current State

- `crates/nako-library/src/intake.rs` already models stable intake candidates, but `StableIntakeObservationFacts::has_complete_stability_evidence()` requires both `has_size` and `has_change_marker`.
- `crates/nako-server/src/app/acquisition_intake.rs` already passes `has_size` from `metadata.len` and only treats change markers as `modified_at`, `etag`, or `fingerprint`.
- `crates/nako-server/src/app/watch_folder_runtime.rs` already hands stable candidates to scan admission through the existing supervised runtime path.
- `docs/architecture/LIBRARY_PIPELINE.md` still lists `stable-size detection` and `copy-in-progress handling` as remaining watcher/debounce follow-ons.
- The legacy watch-folder source-key cleanup slice is already archived, so it should not be reopened as the first step in this task.

## Implication

Repeated same-size observations are currently stuck in `Inspecting` when a candidate lacks richer change markers. That means large files, slow copies, and remote storage can remain conservative longer than necessary even when the stable size signal is already available.

## First-Slice Direction

The first useful slice is to deepen the existing stable-candidate primitive so repeated same-size observations can graduate to `Stable` without new schema, new daemon, or new executor work.

