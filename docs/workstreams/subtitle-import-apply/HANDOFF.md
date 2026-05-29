# Subtitle Import Apply Handoff

## Status

Complete.

## Current Scope

Implemented host-owned subtitle import apply after selected-reference and
import-plan preview. The lane writes subtitle sidecars only through Nako's
Library File Write / VFS boundary.

## Completed

- Added Admin import-apply route and generated contract.
- Reused import-plan fields plus `plan_idempotency_key`.
- Recomputed the plan before mutation and rejected stale keys.
- Supported inline and download-url subtitle content.
- Rejected artifact-ref delivery until a host artifact resolver exists.
- Validated subtitle size, UTF-8, NUL bytes, and basic SRT/VTT markers.
- Derived the sidecar target from the stored media source locator.
- Implemented same-content idempotency.
- Enforced create-missing conflict and replace-existing backup behavior.
- Kept responses free of URLs, raw subtitle text, local locators, and backup
  URIs.

## Do Not Do

- Do not let addons provide target paths.
- Do not expose source URLs, subtitle content, local locators, or backup URIs in
  Admin responses.
- Do not refresh playback subtitle facts in this lane.
- Do not implement cloud-drive transfer.

## Follow-Ons

- Subtitle fact refresh and playback visibility.
- Artifact-ref subtitle resolver.
- Pluggable downloader policy if direct HTTP download is not enough.
