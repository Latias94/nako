# Current HLS Seek Command State

## Observed local code

* `crates/nako-transcode/src/artifact.rs`
  * `HlsPlaybackGeneration` carries `start_position_ms`.
  * Non-default generations participate in request variant identity through
    `hls-playback-generation:v1;start_ms=<value>`.
* `crates/nako-transcode/src/ffmpeg/hls/input.rs`
  * Emits `-ss <timestamp>` before `-i` when generation is not default.
* `crates/nako-transcode/src/ffmpeg/hls/encoders.rs`
  * Emits `-force_key_frames expr:gte(t,n_forced*<segment_time>)` for
    non-default generation.
* `crates/nako-transcode/src/ffmpeg/hls/muxer.rs`
  * Emits `-avoid_negative_ts make_zero` and `-hls_flags independent_segments`
    for non-default generation.
* `crates/nako-transcode/src/ffmpeg/hls/seek.rs`
  * Contains formatting and helper functions, but not a shared seek plan object.
* `crates/nako-transcode/src/lib.rs`
  * Existing exact argv test covers single-variant seek args.

## Bounded implementation seam

This task can stay inside `nako-transcode` by introducing a request-derived HLS
seek command plan and passing it through the HLS command-part builders. The
server and playback planner already pass `HlsPlaybackGeneration`; this slice does
not need to change route parsing or session lifecycle behavior.

## Risks and constraints

* Seeking is timestamp-sensitive, so this slice must preserve existing command
  behavior rather than changing fast/accurate seek policy.
* Single-variant and adaptive HLS should share the same seek-plan facts.
* Exact argv tests should catch ordering drift, especially `-ss` before `-i`.
