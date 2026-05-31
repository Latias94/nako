# PTJCH-210 - HLS Artifact Authority

Status: Done
Merged commit: `8ff30ecd`
Date: 2026-05-31

## Existing Authority Flow

```text
HLS playback request
  -> TranscodePipelinePlanner builds HlsRuntimePlan
  -> HlsRequestVariantPlan records adaptive ladder, media renditions, and generation
  -> TranscodeRequestIdentity persists request_variant in request_key
  -> staging policy creates a fresh HlsArtifactManifest for new sessions
  -> persisted sessions rebuild HlsArtifactSpec from request_key
  -> HlsArtifactSpec + primary playlist path reconstruct HlsArtifactManifest
  -> playlist rewrite, segment serving, and cleanup call manifest.artifact_for_name
```

The manifest is the artifact authority. Server runtime code owns lifecycle and
readiness, but it does not infer HLS artifact membership from directory
contents or request-key substrings.

## Summary

- Kept request variant identity and artifact path formats unchanged.
- Tightened `HlsArtifactManifest::artifact_for_name` so main segments,
  adaptive variant segments, audio sidecar segments, and subtitle sidecar
  segments must match the manifest's sequence pattern instead of passing by
  extension or broad prefix.
- Preserved playlist and init-file authority as explicit manifest artifacts.
- Returned `hls_artifact` not found for legal names outside the manifest
  allow-list while keeping invalid names as validation errors.
- Added `nako-transcode` tests for full request variant identity round-trip,
  manifest reconstruction from persisted request identity, and the serveable
  allow-list for playlist, media group playlist, segment, init file, audio
  sidecar, and subtitle sidecar artifacts.

## Validation

```text
cargo nextest run -p nako-transcode hls --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Result: passed on 2026-05-31. `git diff --check` reported only LF/CRLF
working-copy normalization warnings and no whitespace errors.

## Remaining Risk

- PTJCH-220 still owns session lifecycle, admission, reuse, supersede, cancel,
  failure classification, and diagnostics. This task intentionally did not
  change those runtime behaviors.
- One earlier `nako-transcode` HLS run hit a progressive-readiness timing
  failure in `hls_runner_can_publish_output_while_process_is_running`; the
  focused rerun and final full gate passed.
