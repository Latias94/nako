# Playback Subtitle Language Default Policy - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open. It follows the closed subtitle serving, HLS subtitle
rendition, HLS master rendition authoring, and playback audio language default
policy workstreams. Nako now needs request-scoped subtitle language/default
policy so HLS subtitle groups can mark the right rendition as default without
relying only on an explicit stream index or first-subtitle fallback.

## Active Task

- Task ID: PSLD-020
- Owner: codex
- Files:
  - `crates/nako-playback`
  - `crates/nako-server/src/app/playback`
  - focused playback policy tests
- Validation:
  - `cargo nextest run -p nako-playback subtitle --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
- Status: PENDING
- Review: pending
- Evidence: `docs/workstreams/playback-subtitle-language-default-policy/EVIDENCE_AND_GATES.md`

## Decisions Since Opening

- First slice is request-scoped, not persisted user preference settings.
- Explicit source subtitle stream selection must override language preference.
- Language matching starts from normalized media probe stream language tags.
- HLS subtitle rendition defaults are the first visible integration point.
- Do not add UI controls, subtitle OCR, image-subtitle burn-in, ASS/SSA style
  shaping, addon late-subtitle readiness, LL-HLS, DASH, DRM, or offline sync in
  this lane.
- PALD proved the same boundary for audio; PSLD should reuse the architectural
  shape without forcing a generic abstraction before subtitle behavior proves
  it.

## Blockers

- None.

## Next Recommended Action

- Run PSLD-020 through `run-workstream-task` or TDD.
- Model request-scoped preferred subtitle languages in playback policy, then
  assert explicit stream precedence, preferred-language match, fallback, and
  request identity normalization before touching HLS playlist behavior.
