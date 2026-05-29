# Playback Audio Language Default Policy - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

The lane is open. It follows the closed HLS audio sidecar and selected-main-audio
cleanup lanes. Nako now needs request-scoped audio language/default policy so
HLS audio groups can mark the right rendition as `DEFAULT=YES` without relying
only on an explicit stream index or first-audio fallback.

PALD-020 is complete. `nako-playback` now owns request-scoped preferred audio
language selection, and server HLS setup uses the playback decision's selected
transcode track selection.

## Active Task

- Task ID: PALD-030
- Owner: codex
- Files:
  - `crates/nako-server/src/app/playback`
  - `crates/nako-server/src/http/playback.rs`
  - `crates/nako-api` only if public DTO/query contracts change
- Validation:
  - `cargo nextest run -p nako-server hls --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
  - `cargo nextest run -p nako-api --no-fail-fast` if public contracts change
- Status: PENDING
- Review: pending
- Evidence: `docs/workstreams/playback-audio-language-default-policy/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- First slice is request-scoped, not persisted user preference settings.
- Explicit source stream selection must override language preference.
- Language matching starts from normalized media probe stream language tags.
- HLS audio rendition defaults are the first visible integration point.
- Do not add UI controls, subtitle language policy, codec-aware audio, downmix,
  LL-HLS, DASH, DRM, or offline sync in this lane.
- PALD-020 intentionally did not add HTTP/browser query parsing; PALD-030 owns
  that public request surface decision.

## Blockers

- None.

## Next Recommended Action

- Run PALD-030 through `run-workstream-task` or TDD.
- Decide the HLS request query shape for preferred audio languages, then assert
  the selected audio rendition is the only generated `DEFAULT=YES`.
