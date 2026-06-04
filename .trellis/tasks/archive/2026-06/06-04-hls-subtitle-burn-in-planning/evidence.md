# Evidence: HLS Subtitle Burn-In Planning

## Integrated Commits

* `022a1b55 feat(playback): plan hls subtitle burn-in requirements`
* `7cb5076c fix(playback): block remux for burn-in subtitle requirements`

## Changed Scope

* `crates/nako-playback/src/capability.rs`
* `crates/nako-playback/src/lib.rs`
* `crates/nako-transcode/src/pipeline.rs`
* `docs/architecture/PLAYBACK.md`

## Review

Independent review found one blocking issue and one important issue in the
first worker commit:

* remux could bypass burn-in planning for container-unsupported but
  remux-supported ASS/SSA subtitle selections;
* unknown or blank subtitle codec facts used the legacy sidecar path without an
  explicit test or documentation.

Both findings were fixed. The rereview reported: `No findings; safe to
proceed`.

## Main Merge-Gate Verification

* `cargo check -p nako-playback -p nako-transcode --tests` passed.
* `cargo nextest run -p nako-transcode hls --no-fail-fast` passed: 74 tests.
* `cargo nextest run -p nako-playback --no-fail-fast` passed: 40 tests.
* `cargo fmt --all -- --check` passed.
* `git diff --check` passed.

## Residual Risk

This is still a planning slice. PGS/image subtitle execution, external subtitle
burn-in, hardware-filter burn-in, and broader client subtitle capability policy
remain follow-ons.

