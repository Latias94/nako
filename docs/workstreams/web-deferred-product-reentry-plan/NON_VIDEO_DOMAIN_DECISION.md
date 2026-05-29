# Web Deferred Product Reentry Plan - Non-Video Domain Decision

Status: Deferred
Last updated: 2026-05-28

## WDRP-060 Decision

Decision: do not open `non-video-media-domain-baseline` yet.

Photos, music, and podcasts remain intentionally deferred from the live `web/`
runtime. They should not reenter as UI-first surfaces, imported v0 mocks, or
generic catalog variants until a concrete non-video domain baseline is pulled
forward.

## Rationale

ADR-0021 accepts a video-first implementation scope while keeping the domain
model open to audio, image, document, mixed, and online media. It also says
domain-specific metadata for music, podcasts, photos, books, or online catalogs
should wait for the owning media-domain workstream unless a current video-first
feature requires it.

Current evidence:

- Nako already has broad `Media Domain` and `Library Preset` vocabulary.
- The live backend and new `web/` shell are currently video-first.
- Photos need image metadata, thumbnail/variant policy, EXIF/date handling,
  album/event semantics, and permission behavior.
- Music needs album/artist/track identity, embedded tag ingestion, MusicBrainz
  style provider mapping, audio playback queue semantics, and track progress
  policy.
- Podcasts need feed/subscription identity, episode acquisition/download
  policy, per-episode progress, retention, and refresh scheduling.
- None of those domains has a current accepted product slice that justifies
  opening the baseline ahead of video/Admin/public-client follow-ons.

## Reentry Triggers

Open `non-video-media-domain-baseline` only when at least one trigger is true:

| Trigger | Required evidence |
| --- | --- |
| Photo support is pulled forward | Accepted photo library product target, EXIF/local metadata needs, thumbnail/variant serving policy, and Public Client browse requirements. |
| Music support is pulled forward | Accepted audio domain target, album/artist/track identity, embedded tag/provider strategy, queue/playback behavior, and scan metadata requirements. |
| Podcast support is pulled forward | Accepted feed/subscription target, episode identity, acquisition/download policy, refresh scheduling, and progress-state requirements. |
| Mixed-library behavior blocks video work | A concrete video-first workflow fails because current Media Domain / Library Preset boundaries are insufficient. |
| Public Client API needs non-video contracts | A client cannot honestly render a required non-video flow without new DTOs, facets, sort keys, or playback/viewer semantics. |

## Sequence Recommendation

1. Keep video-first and Admin/public-client follow-ons ahead of non-video UI.
2. If non-video is pulled forward, start with one domain, not all three.
3. Prefer photos first only if image library/product needs are concrete.
4. Prefer music before podcasts if audio playback and tag/provider work become
   the main product driver.
5. Start podcasts after acquisition/feed/download policy is explicit; podcasts
   are not just music with RSS.

## Frontend Rule

Do not restore `/media/photos`, `/media/music`, or `/media/podcasts` as live
routes until the baseline workstream accepts the relevant domain model and
Public Client route contracts. The current `web/` shell may keep readiness
copy or deferred navigation only if it is explicit and non-interactive.
