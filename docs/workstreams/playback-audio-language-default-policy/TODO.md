# Playback Audio Language Default Policy - TODO

Status: Active
Last updated: 2026-05-29

## Task Ledger

### PALD-010 - Open lane and freeze first-slice policy

Status: Completed
Owner: codex
Depends on: none

Scope:

- `docs/workstreams/playback-audio-language-default-policy`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`
- `docs/workstreams/README.md`

Goal:

- Create the durable workstream.
- Freeze the first slice to request-scoped audio language/default selection.
- Keep persisted user settings, UI controls, downmix/normalization,
  codec-aware sidecars, and LL-HLS/DASH/DRM out of this lane.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-audio-language-default-policy docs/architecture/PLAYBACK.md docs/architecture/WORKSTREAM_LINKS.md docs/workstreams/README.md
```

Evidence:

- `docs/workstreams/playback-audio-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-audio-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: This lane is open and linked from playback architecture indexes.
- The first executable task is PALD-020.

### PALD-020 - Model request-scoped audio language preference

Status: Completed
Owner: codex
Depends on: PALD-010

Scope:

- `crates/nako-playback`
- `crates/nako-server/src/app/playback/selection.rs`
- focused playback policy tests

Goal:

- Extend playback preference vocabulary with ordered preferred audio languages.
- Implement deterministic audio selection precedence: explicit stream,
  preferred language match, fallback.
- Keep the selection result explainable and independent of HLS playlist
  authoring.

Validation:

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- `crates/nako-playback`
- `crates/nako-server/src/app/playback/selection.rs`
- `docs/workstreams/playback-audio-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- DONE: Playback owns explicit-stream, preferred-language, and fallback audio
  selection.
- DONE: HLS source setup now uses the playback decision's selected transcode
  track selection instead of recomputing from request facts.
- PALD-030 should make the behavior wire-visible through HLS request parsing
  and playlist/default-rendition assertions.

### PALD-030 - Surface policy through HLS audio rendition defaults

Status: Pending
Owner: codex
Depends on: PALD-020

Scope:

- `crates/nako-server/src/app/playback`
- `crates/nako-server/src/http/playback.rs` if request query parsing is needed
- `crates/nako-api` only if public DTO/query contracts change

Goal:

- Use the selected policy audio stream as the HLS audio rendition default.
- Preserve explicit `requested_audio_stream` precedence.
- Add or update browser/public playback request parsing if the first slice
  needs a wire-level preferred-language input.
- Preserve request identity and HLS route behavior.

Validation:

```text
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-api --no-fail-fast
```

Review:

- Use `review-workstream` before accepting completion.

Evidence:

- HLS audio default tests.
- Public/browser request parsing tests if contracts change.
- `docs/workstreams/playback-audio-language-default-policy/EVIDENCE_AND_GATES.md`

Handoff:

- PALD-020 already propagates internal selected audio policy into HLS source
  facts. Split persisted user settings or UI controls instead of adding them
  here.

### PALD-040 - Verify, document, and close or split follow-ons

Status: Pending
Owner: planner
Depends on: PALD-030

Scope:

- `docs/workstreams/playback-audio-language-default-policy`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/WORKSTREAM_LINKS.md`

Goal:

- Run fresh focused gates.
- Record final evidence and residual risks.
- Close the lane or split follow-ons for persisted preferences, subtitle
  language policy, codec-aware audio, downmix/normalization, or UI controls.

Validation:

```text
cargo nextest run -p nako-playback audio --no-fail-fast
cargo nextest run -p nako-server hls --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Review:

- Use `review-workstream` and `verify-rust-workstream` before closeout.

Evidence:

- `docs/workstreams/playback-audio-language-default-policy/EVIDENCE_AND_GATES.md`
- `docs/workstreams/playback-audio-language-default-policy/HANDOFF.md`
- `docs/workstreams/playback-audio-language-default-policy/WORKSTREAM.json`

Handoff:

- Update `WORKSTREAM.json` status and continue policy.
