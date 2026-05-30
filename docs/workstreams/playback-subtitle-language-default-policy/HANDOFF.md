# Playback Subtitle Language Default Policy - Handoff

Status: Completed
Last updated: 2026-05-30

## Current State

The lane is closed. It follows the closed subtitle serving, HLS subtitle
rendition, HLS master rendition authoring, and playback audio language default
policy workstreams. Nako now has request-scoped subtitle language/default policy
so HLS subtitle groups can mark the right rendition as default without relying
only on an explicit stream index or first-subtitle fallback.

PSLD-020 is complete. `nako-playback` now owns request-scoped preferred
subtitle language selection, explicit subtitle stream precedence, fallback, and
identity normalization.

PSLD-030 is complete. The public HLS playlist route exposes
`preferred_subtitle_language` as a comma-separated ordered language list,
OpenAPI/SDK/HTTP API docs expose the contract, and HLS route tests prove
language-selected subtitle defaults, explicit `subtitle_stream` precedence, and
normalized request identity reuse.

PSLD-040 is complete. Fresh playback, HLS, API, formatting, JSON, and diff
gates passed. Architecture and workstream docs now mark this first slice as
shipped, and all larger preference/subtitle/runtime items are deferred to
follow-on lanes.

## Active Task

None. Open a new workstream for persisted user preferences, subtitle UI
controls, OCR/burn-in/ASS shaping, addon late-subtitle readiness, LL-HLS, DASH,
DRM, offline sync, or richer language metadata normalization.

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

- Commit the verified closeout.
- Open follow-ons only when product scope needs persisted user preferences,
  subtitle UI controls, OCR/burn-in/ASS shaping, addon readiness, LL-HLS, DASH,
  DRM, offline sync, or richer language metadata normalization.
