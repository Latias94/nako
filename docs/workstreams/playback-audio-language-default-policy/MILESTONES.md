# Playback Audio Language Default Policy - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

Status: Completed

Exit criteria:

- Problem and target state are explicit.
- Related HLS audio workstreams are linked.
- Non-goals are explicit.
- First executable task is chosen.

Primary evidence:

- `docs/workstreams/playback-audio-language-default-policy/DESIGN.md`
- `docs/workstreams/playback-audio-language-default-policy/TODO.md`

## M1 - Preference Vocabulary And Selection Policy

Status: Completed

Exit criteria:

- Request-scoped preferred audio languages are typed.
- Explicit stream selection wins over language preference.
- Language match and fallback behavior are deterministic and tested.

Primary gates:

- `cargo nextest run -p nako-playback audio --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`

## M2 - HLS Default Rendition Integration

Status: Completed

Exit criteria:

- HLS audio sidecar default flags use the selected policy audio stream.
- Request parsing or API shape changes, if any, are tested and documented.
- Existing HLS route/session behavior remains stable.

Primary gates:

- `cargo nextest run -p nako-server hls --no-fail-fast`
- `cargo nextest run -p nako-server playback --no-fail-fast`
- `cargo nextest run -p nako-api --no-fail-fast`

## M3 - Closeout

Status: Pending

Exit criteria:

- Fresh focused gate evidence is recorded.
- Architecture docs reflect the shipped policy slice.
- Follow-ons are split or explicitly deferred.
- `WORKSTREAM.json` status is updated.
