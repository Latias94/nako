# Playback Subtitle Language Default Policy - Closeout

Date: 2026-05-30
Status: Completed

## Final Status

PSLD-010 through PSLD-040 are complete. The lane is closed.

Nako now has a request-scoped subtitle language/default policy for playback:
explicit subtitle stream selection wins, ordered preferred subtitle languages
select the first matching source subtitle stream, fallback remains
deterministic, and HLS subtitle rendition authoring marks the selected policy
stream as the only generated `DEFAULT=YES` subtitle rendition.

The public HLS playlist route exposes this as `preferred_subtitle_language`.
OpenAPI, generated TypeScript/Kotlin SDKs, and HTTP API docs describe the query.

## Fresh Gates

- `cargo nextest run -p nako-playback subtitle --no-fail-fast` - 4 passed, 23
  skipped.
- `cargo nextest run -p nako-server hls --no-fail-fast` - 58 passed, 429
  skipped.
- `cargo nextest run -p nako-server playback --no-fail-fast` - 137 passed, 350
  skipped.
- `cargo nextest run -p nako-api --no-fail-fast` - 70 passed.
- `cargo fmt --all -- --check`.
- `python3 -m json.tool docs/workstreams/playback-subtitle-language-default-policy/WORKSTREAM.json`.
- `git diff --check`.

## Follow-Ons

- Persisted per-user subtitle language/default settings.
- Player/Admin UI controls for subtitle language defaults.
- Subtitle OCR, image-subtitle burn-in, ASS/SSA shaping, and style preservation.
- Addon late-subtitle readiness windows.
- LL-HLS, DASH/CMAF, DRM/key delivery, and offline sync.
- Richer language metadata normalization beyond media probe stream language
  tags.

## Residual Risk

Language matching intentionally starts from normalized media probe stream
language tags. Ambiguous, missing, forced/commentary-specific, or
provider-derived language metadata should be handled in a future
preference/profile lane rather than expanding this first request-scoped slice.
