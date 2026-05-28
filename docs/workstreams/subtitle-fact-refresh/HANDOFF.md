# Subtitle Fact Refresh Handoff

## Status

Complete; commit pending at closeout.

## Scope

Refresh media-source subtitle facts after Subtitle Import Apply writes a
sidecar. Use `media_probe` streams as the shared read model for playback and
public catalog visibility.

## Completed

- Added `origin` and `disposition` to public media stream DTOs.
- Marked ffprobe-discovered streams as `embedded`.
- Refreshed imported sidecar subtitles into `media_probe` streams with
  `origin=sidecar`.
- Returned a redaction-safe `refreshed_fact` from Admin subtitle import apply.
- Regenerated Admin TypeScript contracts for both generated contract targets.
- Verified focused API/server tests and closeout checks.

## Do Not Do

- Do not expose sidecar paths or backup URIs.
- Do not serve subtitle bytes in this lane.
- Do not implement HLS subtitle renditions.
- Do not add cloud-drive transfer.

## Follow-Ons

- Playback subtitle byte serving.
- HLS subtitle rendition planning.
- Embedded subtitle extraction and reconciliation with sidecar facts.
