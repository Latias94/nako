# Subtitle Complete Chain Follow-Ons

Status: Complete
Last updated: 2026-05-28

## Host Candidate Selection

Nako should expose a host-owned selected subtitle candidate reference after a
provider search. The selected reference must point to a provider candidate and
captured safe metadata, not raw provider credentials or local target paths.

## Import Planning

Nako should derive a `SubtitleImportPlan` from media item/source identity,
language, format, sidecar role, conflict policy, and backup policy. The plan can
be previewed before any file write.

## Library File Write Apply

Subtitle sidecar persistence should reuse Library File Write and VFS semantics:
atomic replace, backup, idempotency, redacted report fields, and safe error
codes. Addons must not provide absolute paths, Source Locators, remote storage
handles, or backup URIs.

## Refresh And Playback Visibility

After apply, Nako should refresh subtitle facts for the media source and expose
them to playback planning. Playback subtitle execution, HLS subtitle renditions,
and burn-in remain separate playback/transcode work.

## Provider Breadth

OpenSubtitles-like providers, embedded subtitle extraction, and local ASR can be
implemented after the shared protocol contract is stable. Provider breadth does
not change the host-owned write boundary.
