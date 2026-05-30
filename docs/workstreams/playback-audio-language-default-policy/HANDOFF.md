# Playback Audio Language Default Policy - Handoff

Status: Completed
Last updated: 2026-05-29

## Current State

The lane is closed. It follows the closed HLS audio sidecar and
selected-main-audio cleanup lanes. Nako now has request-scoped audio
language/default policy so HLS audio groups can mark the right rendition as
`DEFAULT=YES` without relying only on an explicit stream index or first-audio
fallback.

PALD-020 is complete. `nako-playback` owns request-scoped preferred audio
language selection, and server HLS setup uses the playback decision's selected
transcode track selection.

PALD-030 is complete. The public HLS playlist route accepts
`preferred_audio_language` as a comma-separated ordered language list, OpenAPI
and generated TypeScript/Kotlin SDKs expose it, and route tests assert language
defaulting, explicit stream override, and normalized request identity reuse.

PALD-040 is complete. Fresh playback, HLS, API, formatting, JSON, and diff
gates passed. Architecture and workstream docs now mark this first slice as
shipped, and all larger preference/audio/subtitle/runtime items are deferred to
follow-on lanes.

## Active Task

None. Open a new workstream for persisted user preferences, subtitle language
policy, codec-aware audio sidecars, downmix/normalization, player UI controls,
LL-HLS, DASH, DRM, or offline sync.

## Decisions Since Opening

- First slice is request-scoped, not persisted user preference settings.
- Explicit source stream selection must override language preference.
- Language matching starts from normalized media probe stream language tags.
- HLS audio rendition defaults are the first visible integration point.
- Do not add UI controls, subtitle language policy, codec-aware audio, downmix,
  LL-HLS, DASH, DRM, or offline sync in this lane.
- PALD-020 intentionally did not add HTTP/browser query parsing; PALD-030 owns
  that public request surface decision.
- PALD-030 chose `preferred_audio_language` as a singular HLS query parameter
  whose value is a comma-separated ordered language list.
- HLS request identity normalizes preferred language values, so equivalent
  ordered inputs such as `JPN,eng,jpn` and `jpn,eng` reuse the same transcode
  session.

## Blockers

- None.

## Next Recommended Action

- Commit the verified closeout docs.
- Open follow-ons only when product scope needs persisted user preferences,
  subtitle language policy, codec-aware audio sidecars, downmix/normalization,
  player UI controls, LL-HLS, DASH, DRM, or offline sync.
