# Subtitle Fact Refresh Design

## Intent

After subtitle import apply writes a sidecar, Nako must expose a durable,
redaction-safe subtitle fact for the media source. Playback planning and public
catalog views already consume media probe streams, so this lane stores imported
sidecar subtitles as `MediaStreamKind::Subtitle` facts with `origin=sidecar`.

## In Scope

- Refresh media probe streams after a successful or already-applied subtitle
  import apply.
- Preserve existing audio/video probe streams.
- Add a sidecar subtitle stream if missing.
- Update the existing sidecar subtitle stream instead of duplicating it on
  repeated apply.
- Expose stream origin and disposition through public media stream DTOs.
- Keep local sidecar locator, raw content, and backup URI out of DTOs.

## Out Of Scope

- Serving sidecar subtitle bytes to players.
- HLS subtitle renditions.
- Embedded subtitle extraction.
- Separate subtitle table or path inventory.
- Cloud-drive transfer.

## Boundary

`media_probe` remains the read model for playback-facing technical facts. The
sidecar file write remains owned by Subtitle Import Apply / Library File Write;
this lane only refreshes the media-source fact after that mutation succeeds.

## Validation

- `cargo nextest run -p nako-api media_stream --no-fail-fast`
- `cargo nextest run -p nako-server addon_subtitle_import --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
