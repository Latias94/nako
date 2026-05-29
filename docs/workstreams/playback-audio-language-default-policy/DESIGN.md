# Playback Audio Language Default Policy

Status: Active
Last updated: 2026-05-29

## Why This Lane Exists

Nako can now publish generated HLS audio groups and avoid duplicating selected
audio in the primary HLS video output. The next user-facing correctness gap is
default audio choice.

Today playback can honor an explicit `requested_audio_stream`, but there is no
language/default policy. Multi-audio sources therefore fall back to requested or
first audio behavior, which is too weak for a Jellyfin/Plex-class media server.
Anime, foreign films, commentary tracks, and multi-dub libraries need explicit
and explainable audio defaulting before LL-HLS/DASH or richer clients inherit
the wrong assumptions.

## Relevant Authority

- ADRs:
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- Architecture maps:
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/hls-alternate-audio-renditions/`
  - `docs/workstreams/hls-audio-sidecar-artifacts/`
  - `docs/workstreams/hls-selected-main-audio-cleanup/`
  - `docs/workstreams/playback-planner-transcode-value-vocabulary/`

## Problem

Audio selection currently has only one strong input: an explicit source stream
index. Without a language/default policy:

- HLS audio sidecar `DEFAULT=YES` can be correct only when the user explicitly
  requested a stream or the first audio stream happens to be right.
- Playback planning cannot explain why a language was selected.
- Public and browser playback routes cannot pass a request-scoped language
  preference without inventing per-route behavior.
- Future persisted user preferences, mobile/TV client settings, DASH packaging,
  and offline sync would each need to rediscover the same policy.

## Target State

When this lane closes:

- Playback has a typed request-scoped audio preference model that can carry
  ordered preferred audio languages.
- Explicit `requested_audio_stream` continues to win over language preference.
- If no explicit stream is requested, playback chooses the first source audio
  stream matching the ordered preferred language list.
- If no preferred language matches, playback falls back to the existing first
  audio/default behavior.
- HLS audio sidecar default flags use the selected policy stream, so exactly
  one generated audio rendition is `DEFAULT=YES` when audio renditions exist.
- Public/browser playback request parsing and DTOs are updated only if the
  first slice needs a wire-level language preference.
- Tests prove explicit stream precedence, language matching, fallback behavior,
  HLS `TYPE=AUDIO` default flags, request identity stability, and route
  compatibility.

## In Scope

- Request-scoped audio language preference vocabulary.
- Selection policy for explicit stream, ordered preferred languages, and
  fallback.
- Integration with HLS audio rendition default selection.
- Public/browser playback route parsing if needed for request-scoped language
  preferences.
- Focused playback/HLS/API tests and docs.

## Out Of Scope

- Persisted per-user default audio settings.
- Admin Web or player UI preference controls.
- Subtitle language/default policy.
- Audio downmix, normalization, codec-copy sidecars, or codec-aware audio
  selection.
- LL-HLS, DASH/CMAF, DRM/key delivery, or offline sync.
- Provider metadata language heuristics beyond media probe stream language tags.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| A request-scoped preference is the right first slice before persisted user settings. | High | Existing `PlaybackPreferenceContext` already carries request-scoped `requested_audio_stream`. | Split persisted user preference schema/API into a follow-on. |
| Language matching can start from normalized media stream `language` tags. | Medium | Probe facts already expose stream languages used by HLS audio rendition identity. | Add a normalization task or defer ambiguous tags. |
| Explicit stream index must override language preference. | High | Current selected audio semantics and user intent. | Add a compatibility task before changing precedence. |
| HLS audio sidecar default flags are the first visible integration point. | High | HLS audio sidecars are now generated and selected-main duplication is closed. | Re-scope to planner-only if HLS integration is premature. |

## Architecture Direction

Audio defaulting should live in playback vocabulary, not in playlist string
rewriting. `nako-playback` should own the policy inputs and selected audio
result. `nako-server` should adapt HTTP/browser request facts into that
preference context and use the chosen stream when building HLS audio
renditions. `nako-transcode` should continue to receive explicit stream facts
and artifact identity; it should not decide user language preference.

Target flow:

```text
Playback request
  -> PlaybackPreferenceContext
  -> audio selection policy
  -> selected audio stream index
  -> HlsAudioRendition default flag
  -> request identity + HLS master playlist
```

## Closeout Condition

This lane can close when:

- request-scoped audio language preference is typed and tested;
- explicit stream selection, preferred-language selection, and fallback are
  deterministic;
- HLS audio rendition defaults follow the selected policy stream;
- focused playback/HLS/API gates pass with fresh evidence;
- persisted user settings, UI controls, subtitle language policy, codec-aware
  audio, and LL-HLS/DASH/DRM are either split or explicitly deferred.
