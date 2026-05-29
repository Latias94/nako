# Playback Audio Language Default Policy - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

The lane is open. It follows the closed HLS audio sidecar and selected-main-audio
cleanup lanes. Nako now has request-scoped audio language/default policy so HLS
audio groups can mark the right rendition as `DEFAULT=YES` without relying only
on an explicit stream index or first-audio fallback.

PALD-020 is complete. `nako-playback` owns request-scoped preferred audio
language selection, and server HLS setup uses the playback decision's selected
transcode track selection.

PALD-030 is complete. The public HLS playlist route accepts
`preferred_audio_language` as a comma-separated ordered language list, OpenAPI
and generated TypeScript/Kotlin SDKs expose it, and route tests assert language
defaulting, explicit stream override, and normalized request identity reuse.

## Active Task

- Task ID: PALD-040
- Owner: planner
- Files:
  - `docs/workstreams/playback-audio-language-default-policy`
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Validation:
  - `cargo nextest run -p nako-playback audio --no-fail-fast`
  - `cargo nextest run -p nako-server hls --no-fail-fast`
  - `cargo nextest run -p nako-server playback --no-fail-fast`
  - `cargo nextest run -p nako-api --no-fail-fast`
  - `cargo fmt --all -- --check`
  - `python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`
  - `git diff --check`
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
- PALD-030 chose `preferred_audio_language` as a singular HLS query parameter
  whose value is a comma-separated ordered language list.
- HLS request identity normalizes preferred language values, so equivalent
  ordered inputs such as `JPN,eng,jpn` and `jpn,eng` reuse the same transcode
  session.

## Blockers

- None.

## Next Recommended Action

- Run PALD-040 closeout through `review-workstream`,
  `verify-rust-workstream`, and `close-workstream`.
- Decide whether to close this lane after fresh gates or split follow-ons for
  persisted user preferences, subtitle language policy, codec-aware audio,
  downmix/normalization, UI controls, LL-HLS/DASH/DRM, or offline sync.
