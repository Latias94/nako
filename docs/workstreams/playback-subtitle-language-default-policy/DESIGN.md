# Playback Subtitle Language Default Policy

Status: Active
Last updated: 2026-05-30

## Why This Lane Exists

Nako can serve sidecar subtitles, generate selected WebVTT HLS subtitle
renditions, advertise those renditions in HLS master playlists, and choose HLS
audio defaults from request-scoped language preference. The matching subtitle
gap is default selection.

Today playback can honor an explicit subtitle stream request, but there is no
request-scoped subtitle language/default policy. Multi-subtitle sources
therefore fall back to explicit stream or existing selected/first behavior,
which is too weak for anime, foreign films, commentary subtitles, forced
subtitles, and multi-language libraries.

## Relevant Authority

- ADRs:
  - `docs/adr/0052-hls-runtime-and-media-engine-boundary.md`
  - `docs/adr/0044-playback-capability-profile-planner.md`
  - `docs/adr/0023-public-api-versioning-and-error-envelope-contract.md`
- Architecture maps:
  - `docs/architecture/PLAYBACK.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
- Related workstreams:
  - `docs/workstreams/playback-subtitle-serving/`
  - `docs/workstreams/hls-media-renditions-runtime/`
  - `docs/workstreams/hls-master-renditions-authoring/`
  - `docs/workstreams/playback-audio-language-default-policy/`

## Problem

Subtitle selection currently has one strong input: an explicit source stream
index. Without a request-scoped language/default policy:

- HLS `TYPE=SUBTITLES` default behavior can be correct only when the caller
  explicitly requested a stream or the first subtitle stream happens to be
  right.
- Playback planning cannot explain why a subtitle language was selected.
- Public/browser playback routes cannot pass a request-scoped subtitle
  language preference without inventing per-route behavior.
- Future persisted user preferences, player UI, DASH packaging, and offline
  sync would each need to rediscover the same policy.

## Target State

When this lane closes:

- Playback has a typed request-scoped subtitle preference model that can carry
  ordered preferred subtitle languages.
- Explicit `requested_subtitle_stream` continues to win over language
  preference.
- If no explicit stream is requested, playback chooses the first source
  subtitle stream matching the ordered preferred language list.
- If no preferred language matches, playback falls back to existing subtitle
  default behavior.
- HLS subtitle sidecar default flags use the selected policy stream, so at most
  one generated subtitle rendition is marked default when subtitle renditions
  exist.
- Public/browser playback request parsing and API/SDK docs are updated only if
  the first slice needs a wire-level preferred-language input.
- Tests prove explicit stream precedence, language matching, fallback behavior,
  HLS `TYPE=SUBTITLES` default flags, request identity stability, and route
  compatibility.

## In Scope

- Request-scoped subtitle language preference vocabulary.
- Selection policy for explicit stream, ordered preferred languages, and
  fallback.
- Integration with HLS subtitle rendition default selection.
- Public/browser playback route parsing if needed for request-scoped subtitle
  preferences.
- Focused playback/HLS/API tests and docs.

## Out Of Scope

- Persisted per-user default subtitle settings.
- Admin Web or player UI preference controls.
- Audio language policy changes.
- Subtitle OCR, image-subtitle burn-in, ASS/SSA shaping, or style preservation.
- Addon late-subtitle readiness windows.
- LL-HLS, DASH/CMAF, DRM/key delivery, or offline sync.
- Provider metadata language heuristics beyond media probe stream language tags.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| A request-scoped preference is the right first slice before persisted user settings. | High | Audio language policy shipped with the same boundary and avoided premature settings schema. | Split persisted user preference schema/API into a follow-on. |
| Subtitle language matching can start from normalized media stream `language` tags. | Medium | Probe facts already expose stream languages used by HLS subtitle rendition identity. | Add a normalization task or defer ambiguous tags. |
| Explicit subtitle stream index must override language preference. | High | Current explicit stream semantics represent direct user/client intent. | Add a compatibility task before changing precedence. |
| HLS subtitle rendition defaults are the first visible integration point. | High | HLS subtitle sidecar artifacts and master playlist authoring are already shipped. | Re-scope to planner-only if HLS integration is premature. |

## Architecture Direction

Subtitle defaulting should live in playback vocabulary, not in playlist string
rewriting. `nako-playback` should own the policy inputs and selected subtitle
result. `nako-server` should adapt HTTP/browser request facts into that
preference context and use the chosen stream when building HLS subtitle
renditions. `nako-transcode` should continue to receive explicit stream facts
and artifact identity; it should not decide user language preference.

Target flow:

```text
Playback request
  -> PlaybackPreferenceContext
  -> subtitle selection policy
  -> selected subtitle stream index
  -> HlsSubtitleRendition default flag
  -> request identity + HLS master playlist
```

## Closeout Condition

This lane can close when:

- request-scoped subtitle language preference is typed and tested;
- explicit stream selection, preferred-language selection, and fallback are
  deterministic;
- HLS subtitle rendition defaults follow the selected policy stream;
- focused playback/HLS/API gates pass with fresh evidence;
- persisted user settings, UI controls, burn-in/OCR/ASS shaping, addon
  readiness, and LL-HLS/DASH/DRM are either split or explicitly deferred.
