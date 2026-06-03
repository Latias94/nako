# FFmpeg Subtitles Filter Notes

Source:

* FFmpeg filter documentation: https://www.ffmpeg.org/ffmpeg-filters.html

## Key Findings

* The `subtitles` filter accepts a subtitle source file and can render that
  subtitle track into the video path.
* The filter exposes a `filename` option for the subtitle source.
* The filter exposes `si` to pick the subtitle stream index.
* The filter exposes `original_size`, which is useful when subtitle rendering
  should preserve the source composition size while the output video is scaled.
* The filter also exposes styling-related options such as `force_style` and
  character encoding selection.

## Implication For Nako

* A typed HLS burn-in request can be built from:
  * the input media path,
  * the selected subtitle stream index,
  * and source video dimensions when available.
* We do not need a new FFmpeg adapter to prove the concept; the CLI filter
  surface is already expressive enough for a later burn-in slice.

## Caution

* Text-subtitle burn-in and image-subtitle handling are not the same problem.
  The filter options above are enough for a text-subtitle planning slice, but
  unsupported subtitle codecs still need explicit validation.
