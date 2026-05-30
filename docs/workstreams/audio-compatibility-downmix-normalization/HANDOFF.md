# Audio Compatibility Downmix Normalization - Handoff

Status: Active
Last updated: 2026-05-30

## Current State

The lane is open and linked from playback architecture indexes. `ACDN-010`,
`ACDN-020`, and `ACDN-030` are complete.

Playback owns audio output requirement values for channel support, downmix
intent, normalization intent, and audio-specific compatibility reasons.
`TranscodeRequirement` carries the playback-owned audio output requirement, and
transcode/server playback now propagate actionable audio output requirements
into HLS execution policy, pipeline planning, and profile identity.

Compatible audio source facts are collapsed to the empty transcode audio output
requirement at the server adaptation boundary. This keeps ordinary HLS request
keys stable when no downmix or normalization is requested.

## Next Task

Planner can assign `ACDN-040` after accepting this branch. `ACDN-030` is
implemented and the full server HLS gate is now clean.

Diagnostic follow-up on 2026-05-30 showed the server HLS gate failure is not
introduced by ACDN-030: the narrowed `hls_playlist` filter fails under default
nextest parallelism in both the current worktree and a detached baseline
worktree at `b770cac9`, before the ACDN-030 uncommitted changes. The same
narrowed filter passes with `-j 1`, and the two focused running-playlist tests
pass when isolated. The gate was stabilized by widening the focused
running-playlist test timeout and by waiting for the first segment route
readiness in the HTTP test.

Owned scope:

- `crates/nako-transcode/src/policy.rs`
- `crates/nako-transcode/src/pipeline.rs`
- `crates/nako-transcode/src/profile.rs`
- `crates/nako-server/src/app/playback/mod.rs`
- `crates/nako-server/src/app/playback/hls.rs`

Additional HLS gate stabilization touched:

- `crates/nako-server/src/app/tests/playback.rs`
- `crates/nako-server/src/http/tests/playback.rs`

Required validation:

```text
cargo nextest run -p nako-transcode audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- public API DTO or generated contract changes become necessary;
- HDR tone mapping or subtitle burn-in becomes part of the design;
- the audio vocabulary needs a new ADR rather than fitting ADR 0038/0044;
- existing user changes appear in files you need to edit.

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

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, tests run, and evidence anchors.
