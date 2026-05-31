# Playback Compatibility Matrix Hardening - Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This workstream is closed. `PCMH-020` was reviewed, verified, merged into
`main`, and accepted as the playback-only compatibility matrix slice.

## Follow-ons

Open separate workstreams for:

- full device profile compatibility matrices;
- persisted preferences or client/player controls;
- Public Client API DTO compatibility reporting;
- transcode execution policy or FFmpeg command-plan matrices;
- server HLS composition gaps found by future compatibility work.

Required context for follow-ons:

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

- Public Client API DTOs, device profile databases, persisted preferences, or
  web/player behavior become necessary;
- a follow-on needs edits outside `crates/nako-playback`.

## Report Format

End with one of:

- DONE
- DONE_WITH_CONCERNS
- BLOCKED
- NEEDS_CONTEXT

Include changed files, test matrix coverage, validation evidence, and any
follow-on gaps found.
