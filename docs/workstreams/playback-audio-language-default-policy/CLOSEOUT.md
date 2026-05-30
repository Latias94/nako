# Playback Audio Language Default Policy - Closeout

Date: 2026-05-29
Status: Completed

## Final Status

PALD-010 through PALD-040 are complete. The lane is closed.

Nako now has a request-scoped audio language/default policy for playback:
explicit audio stream selection wins, ordered preferred audio languages select
the first matching source audio stream, fallback remains deterministic, and HLS
audio rendition authoring marks the selected policy stream as the only
`DEFAULT=YES` audio rendition.

The public HLS playlist route exposes this as `preferred_audio_language`.
OpenAPI, generated TypeScript/Kotlin SDKs, and HTTP API docs describe the query.

## Fresh Gates

- `cargo nextest run -p nako-playback audio --no-fail-fast` - 4 passed, 19
  skipped.
- `cargo nextest run -p nako-server hls --no-fail-fast` - 56 passed, 422
  skipped.
- `cargo nextest run -p nako-server playback --no-fail-fast` - 135 passed, 343
  skipped.
- `cargo nextest run -p nako-api --no-fail-fast` - 69 passed.
- `cargo fmt --all -- --check`.
- `python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`.
- `git diff --check`.

## Follow-Ons

- Persisted per-user audio language/default settings.
- Player/Admin UI controls for audio language defaults.
- Subtitle language/default policy.
- Codec-aware audio sidecar copy/transcode selection.
- Audio downmix, normalization, dynamic range, dialogue clarity, and night-mode
  policy.
- LL-HLS, DASH/CMAF, DRM/key delivery, and offline sync.

## Residual Risk

Language matching intentionally starts from normalized media probe stream
language tags. Ambiguous, missing, or provider-derived language metadata should
be handled in a future preference/profile lane rather than expanding this first
request-scoped slice.
