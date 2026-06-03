# Jellyfin Transcode Parity Notes

## Summary

Jellyfin concentrates a lot of transcode complexity in a large `EncodingHelper`
and a `TranscodeManager`, while keeping the playback request DTO and device
profile separate. Nako already has a more decomposed shape, so the main
question is not whether to rewrite the transcode stack, but which remaining
seams still need to be deepened.

## Comparable Patterns

1. `MediaBrowser.Model.Dlna.DeviceProfile` describes what a client can direct
   play or transcode.
2. `Jellyfin.Api.Models.MediaInfoDtos.PlaybackInfoDto` carries the playback
   request and embeds the device profile.
3. `MediaBrowser.Controller.MediaEncoding.EncodingHelper` centralizes encoder
   choice, hwaccel selection, filter graphs, bitrate, seeking, and command
   fragments.
4. `MediaBrowser.MediaEncoding.Transcoding.TranscodeManager` owns process
   launch, logging, progress, cancellation, and permission checks.

## Mapping To Nako

- `crates/nako-playback/src/lib.rs` already owns playback decision making.
- `crates/nako-server/src/app/playback/selection.rs` maps a playback decision
  into transcode inputs.
- `crates/nako-transcode/src/pipeline.rs` owns pipeline readiness and hardware
  fallback.
- `crates/nako-transcode/src/profile.rs` owns transcode identity and profile
  validation.
- `crates/nako-transcode/src/ffmpeg/hls.rs` and `remux.rs` own command
  assembly.
- `crates/nako-server/src/app/playback/hls.rs` still stitches admission,
  persistence, and execution together in one service.

## Conclusion

No wholesale fearless rewrite is justified yet. The highest-leverage move is to
deepen the remaining orchestration seam in `nako-server` and keep the
transcode adapter boundaries strict.

## Deepening Candidates

1. `crates/nako-server/src/app/playback/hls.rs` and
   `crates/nako-server/src/app/playback/mod.rs`
   - Problem: HLS source and playlist startup mixed request preparation,
     resource admission, supersede coordination, input staging, background
     execution, and playlist readiness waiting inside the broad playback app
     service.
   - Solution: move HLS lifecycle orchestration into a dedicated
     `hls_flow` module and leave `PlaybackAppService` as a thin delegator.
   - Benefit: future HLS feature work has one server-side lifecycle locality
     without pulling playback planning or FFmpeg command assembly into the app
     service.
2. `crates/nako-transcode/src/profile.rs` and
   `crates/nako-transcode/src/pipeline.rs`
   - Problem: Jellyfin-grade feature parity will add more request facts
     around profiles, fallback, hardware readiness, audio, color, and subtitle
     handling; these must not regress into stringly request keys or ad hoc
     server-side branching.
   - Solution: keep deepening typed identity, readiness, and fallback values
     inside `nako-transcode`.
   - Benefit: profile changes stay testable as transcode contract changes,
     while server code consumes typed runtime plans.
3. `crates/nako-transcode/src/ffmpeg/hls.rs`
   - Problem: HLS parity work will likely add subtitle, HDR, audio rendition,
     and ladder complexity.
   - Solution: deepen FFmpeg HLS command builders around typed request parts
     instead of adding caller-side argument fragments.
   - Benefit: argv behavior remains exact-testable and artifact publication
     remains manifest-driven.

## Implemented Slice

The first candidate was implemented by adding
`crates/nako-server/src/app/playback/hls_flow.rs`.

The refactor is behavior-preserving:

- `PlaybackAppService::hls_source_with_policy` delegates to `hls_flow`.
- `PlaybackAppService::hls_playlist_with_policy` delegates to `hls_flow`.
- HLS source context, playlist-ready context, supersede admission, input
  release, background start, and playlist readiness waiting now live in
  `hls_flow`.
- `nako-playback` remains the pure decision source.
- `nako-transcode` remains the pipeline and FFmpeg planning source.

Verification:

- `cargo check -p nako-server`
- `cargo check -p nako-server --tests`
- `cargo check -p nako-playback -p nako-transcode -p nako-server --tests`
- `cargo nextest run -p nako-server hls_source --no-fail-fast`
- `cargo nextest run -p nako-server hls_playlist --no-fail-fast`
- `cargo fmt --all -- --check`
