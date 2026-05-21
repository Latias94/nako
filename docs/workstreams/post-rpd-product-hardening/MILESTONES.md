# Post-RPD Product Hardening — Milestones

Status: Active
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

Exit criteria:

- Post-RPD roadmap order is explicit.
- Dependencies and non-goals are explicit.
- First execution lane is chosen.

Primary evidence:

- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `docs/workstreams/post-rpd-product-hardening/TODO.md`

## M1 — First Execution Lane Opened

Exit criteria:

- `metadata-provider-breadth` exists as a child workstream.
- Child workstream has independently validatable task slices.
- This umbrella does not contain hidden implementation work.

Primary evidence:

- `docs/workstreams/metadata-provider-breadth/DESIGN.md`
- `docs/workstreams/metadata-provider-breadth/TODO.md`

## M2 — Next Lane Decision

Exit criteria:

- Metadata provider breadth closeout evidence is reviewed.
- NFO/link, playback/transcode, and managed import staging are re-scored.
- Next execution lane is selected and either opened immediately or handed off
  as the next concrete action.

Primary evidence:

- `docs/workstreams/metadata-provider-breadth/EVIDENCE_AND_GATES.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`

## M3 — Umbrella Closeout Or Refresh

Exit criteria:

- Active child lanes own their implementation details.
- Remaining lanes are either opened, re-scored, or intentionally deferred.
- Workstream index reflects the current active product lane.

## M4 — Post-LAIP Lane Scoring

Exit criteria:

- [x] LAIP closeout evidence is reviewed.
- [x] NFO sidecar apply, playback/transcode ops, network, AI, addon runtime,
  and downloads/watch-folder are re-scored.
- [x] Next execution lane is selected without opening a duplicate workstream.

Primary evidence:

- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/HANDOFF.md`
