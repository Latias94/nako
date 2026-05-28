# Playback Capability Profile And Rendition Planning TODO

Status: Completed
Last updated: 2026-05-28

## Task Ledger

### PCPR-010 - Open workstream and lock refactor brief

Status: Complete
Owner: codex
Depends on: none

Scope:

- Create durable workstream docs.
- Record the deletion, boundary, testing, and risk plan.
- Link the lane from the workstream index.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-capability-profile-and-rendition-planning/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-capability-profile-and-rendition-planning docs/workstreams/README.md
```

### PCPR-020 - Replace execution shape with PlaybackRenditionPlan

Status: Complete
Owner: codex
Depends on: PCPR-010

Scope:

- Rename the selected output boundary from `PlaybackExecutionPlan` to
  `PlaybackRenditionPlan`.
- Move direct play, remux, transcode, and denied payloads under
  `PlaybackDecision.rendition`.
- Delete duplicate top-level `direct_play`, `transcode_plan`, and
  `transcode_requirement` fields.
- Preserve Public Client DTO behavior and redaction.

Likely files:

- `crates/nako-playback/src/lib.rs`
- `crates/nako-api/src/public_client.rs`
- `crates/nako-server/src/app/playback/selection.rs`
- `crates/nako-server/src/app/playback/mod.rs`

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast
```

### PCPR-030 - Delete shallow PlaybackProfile adapter

Status: Complete
Owner: codex
Depends on: PCPR-020

Scope:

- Move transcode profile helpers onto `PlaybackTargetProfile`.
- Remove `PlaybackProfile` from public exports and server tests.
- Use target-profile identity for remux/HLS request identities.

Validation:

```text
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
```

### PCPR-040 - Review, verify, and close lane

Status: Complete
Owner: codex
Depends on: PCPR-030

Scope:

- Run fresh focused gates and non-test checks.
- Update evidence, handoff, milestones, and closeout.
- Mark workstream completed only after verification passes.

Validation:

```text
python3 -m json.tool docs/workstreams/playback-capability-profile-and-rendition-planning/WORKSTREAM.json
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-api playback_decision_dto_hides_internal_selection_plan --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```
