# Quality Guidelines

Media probing must preserve technical facts without turning provider quirks into
library policy.

## Required Patterns

- Keep `MediaProbe` backend-agnostic and mockable through a trait.
- Keep `FfprobeMediaProbe::new` configurable so tests and deployments can choose
  the executable path.
- Use `stdin(Stdio::null())`, piped stdout, and piped stderr for external probe
  commands.
- Parse ffprobe duration seconds into milliseconds with checked arithmetic.
- Preserve known ffprobe stream kinds and use `MediaStreamKind::Other` for
  unknown values.
- Preserve video technical facts: profile, level, codec tag, pixel format,
  bits-per-sample, frame rates, field order, rotation, color, HDR, and
  disposition.
- Preserve audio technical facts: sample rate, channels, channel layout, bit
  depth, duration, bit rate, and language.
- Preserve subtitle stream kind, language, and forced/default disposition.

## Forbidden Patterns

- Do not infer catalog identity, movie/series type, or playback eligibility in
  this crate.
- Do not silently drop a newly consumed ffprobe field without a focused test.
- Do not require remote sources to be directly readable by ffprobe; use caller
  staging and `local_path_hint`.
- Do not copy external project parsing logic; write Nako-specific mappings
  against Nako core records.

## Tests Required

- JSON mapping from representative ffprobe output into `MediaProbeResult`.
- HDR detection from color transfer and side data.
- Rotation from tags and side data.
- Disposition flag mapping.
- Rational frame rate parsing, including invalid and zero values when behavior
  changes.
- Missing local path hint behavior.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-media-probe --no-fail-fast`
- Relevant cross-crate:
  `cargo check -p nako-media-probe -p nako-core -p nako-vfs --tests`

## Review Checklist

- Are all consumed provider fields asserted by tests?
- Are optional malformed values handled as absence?
- Is storage/probe orchestration still outside this crate?
- Are provider errors usable by library failure classification?
