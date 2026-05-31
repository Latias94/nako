# Playback Compatibility Matrix Hardening - Handoff

Status: Active
Last updated: 2026-05-31

## Current State

`PCMH-020` is complete and locally verified. The lane now has playback-only
table-driven matrix coverage for representative Direct Play, Remux, and HLS
Transcode compatibility decisions, including HDR remux denial and audio output
requirements.

## Next Task

Assign `PCMH-030` for planner review, fresh verification, and closeout.

Required context:

```text
docs/workstreams/playback-compatibility-matrix-hardening/CONTEXT.jsonl
docs/adr/0038-playback-planning-and-transcode-policy-seams.md
docs/adr/0044-playback-capability-profile-planner.md
docs/workstreams/audio-compatibility-downmix-normalization/CLOSEOUT.md
docs/workstreams/hdr-tone-mapping-pipeline/CLOSEOUT.md
```

Required validation:

```text
cargo nextest run -p nako-playback compatibility --no-fail-fast
cargo nextest run -p nako-playback hdr audio --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Stop Conditions

Return to planner coordination if:

- the task needs edits outside `crates/nako-playback`;
- Public Client API DTOs, device profile databases, persisted preferences, or
  web/player behavior become necessary;
- closeout discovers a transcode, server, or API gap that should become a
  follow-on instead of expanding this lane.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, test matrix coverage, validation evidence, and any
follow-on gaps found.
