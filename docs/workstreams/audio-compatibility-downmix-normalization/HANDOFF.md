# Audio Compatibility Downmix Normalization - Handoff

Status: Completed
Last updated: 2026-05-31

## Current State

The lane is closed and linked from playback architecture indexes. `ACDN-010`
through `ACDN-050` are complete.

Playback owns audio output requirement values for channel support, downmix
intent, normalization intent, and audio-specific compatibility reasons.
`TranscodeRequirement` carries the playback-owned audio output requirement, and
transcode/server playback now propagate actionable audio output requirements
into HLS execution policy, pipeline planning, and profile identity.

Compatible audio source facts are collapsed to the empty transcode audio output
requirement at the server adaptation boundary. This keeps ordinary HLS request
keys stable when no downmix or normalization is requested.

FFmpeg HLS command planning now turns non-empty transcode audio output policy
into deterministic audio filters. Downmix emits an explicit
`aformat=channel_layouts=...` filter from the policy target channel count, and
normalization emits `loudnorm=I=-16:TP=-1.5:LRA=11`. When both are requested,
downmix is ordered before normalization in a single `-af` chain. Audio sidecar
outputs receive the same filter chain so selected-main-audio cleanup does not
drop the compatibility requirement when audio leaves the main HLS output.

## Active Task

None. Open follow-on workstreams for persisted preferences, client UI controls,
device profile databases, dialogue clarity, subtitle burn-in, or HDR tone
mapping.

Closed implementation scope:

- `crates/nako-transcode/src/ffmpeg.rs`
- `crates/nako-transcode/src/lib.rs`
- `crates/nako-transcode/src/tests*`

Final validation:

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
python -m json.tool docs/workstreams/audio-compatibility-downmix-normalization/WORKSTREAM.json
git diff --check
```

## Stop Conditions

None for this closed lane. New audio, HDR, subtitle, or device-profile work
should open or reuse a planner-approved workstream.

## ACDN-030 Evidence

```text
cargo nextest run -p nako-transcode audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

ACDN-030 transcode audio, server HLS, formatting, and whitespace gates passed
on 2026-05-30. Planner verification also passed after stabilization. Before
stabilization, the full server HLS gate failed with 59/61
tests passing and two existing running-playlist timeout tests failing under the
full HLS filter:

- `app::tests::playback::hls_playlist_playback_returns_when_playlist_is_ready_before_runner_finishes`
- `http::tests::playback::hls_playlist_route_returns_while_transcode_session_is_running`

Both tests passed when run individually on 2026-05-30. See
`docs/workstreams/audio-compatibility-downmix-normalization/EVIDENCE_AND_GATES.md`.

## ACDN-040 Evidence

```text
cargo nextest run -p nako-transcode hls audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

ACDN-040 transcode HLS/audio, server HLS, formatting, and whitespace gates
passed on 2026-05-31. See
`docs/workstreams/audio-compatibility-downmix-normalization/EVIDENCE_AND_GATES.md`.

This task only changed FFmpeg HLS command planning and command-plan tests. It
did not edit HDR tone mapping, subtitle burn-in, web player, public DTO, or
server playback code.

## Report Format

This handoff is closed. Use `docs/workstreams/audio-compatibility-downmix-normalization/EVIDENCE_AND_GATES.md`
for final evidence anchors.
