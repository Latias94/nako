# Current FFmpeg HLS Burn-In Builder

## Observed local code

* `crates/nako-transcode/src/ffmpeg/hls.rs`
  * `validate_hls_subtitle_strategy` accepts `None`, `OmitSelected`, and
    `SidecarSelected`.
  * `BurnInSelected` is still rejected as an unsupported HLS subtitle strategy.
  * Primary output command parts already route through
    `filters::hls_filter_graph_args(...)`.
* `crates/nako-transcode/src/ffmpeg/hls/filters.rs`
  * Existing video filter planning is `-vf` based.
  * HDR-to-SDR software tone mapping returns a comma-joined filter graph.
  * VAAPI currently emits a hardware upload `-vf` path when no software color
    filter is required.
  * There is no subtitle burn-in filter helper yet.
* `crates/nako-transcode/src/ffmpeg/hls/input.rs`
  * Primary HLS output maps video and selected/default audio only.
  * Subtitle streams are not mapped into the primary output today.
* `crates/nako-transcode/src/ffmpeg/hls/sidecars.rs`
  * `SidecarSelected` emits WebVTT segmented sidecar outputs.
  * `BurnInSelected` is unreachable because validation rejects it first.

## Prior research carried forward

The prior task recorded FFmpeg `subtitles` filter facts:

* `subtitles` can render subtitles into the video path.
* It accepts an input filename and `si` subtitle-stream ordinal selector.
* `original_size` can be used later when source dimensions need explicit
  subtitle layout preservation.

## Bounded implementation seam

This task can stay within `nako-transcode` by:

* validating `BurnInSelected` has a selected subtitle stream;
* deriving the FFmpeg `si` ordinal from media probe subtitle stream order;
* validating the selected burn-in stream has embedded text-subtitle facts before
  FFmpeg execution planning;
* rejecting subtitle media-rendition artifacts for burn-in;
* composing a primary `-vf` filter that includes
  `subtitles=<input>:si=<subtitle-ordinal>`;
* leaving playback strategy selection and server HLS orchestration unchanged.

## Risks and constraints

* The `subtitles` filter uses a filename expression; path escaping must be
  deterministic in tests and not hand-concatenated into shell commands.
* Nako source stream indexes are global ffprobe indexes, while FFmpeg `si` is
  an ordinal among subtitle streams. Command planning must not confuse them.
* Hardware filter paths and software subtitle rendering may conflict. The first
  slice should reject non-software filter acceleration for burn-in unless there
  is an existing safe pattern to compose it.
* Image subtitles and external subtitle staging remain unsupported in this slice.
  Text subtitle codec eligibility is intentionally conservative and can be
  expanded later with fixture-backed evidence.
