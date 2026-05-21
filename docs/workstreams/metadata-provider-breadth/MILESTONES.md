# Metadata Provider Breadth — Milestones

Status: Completed
Last updated: 2026-05-21

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- First slice is capability diagnostics.
- Non-goals prevent UI, AI, download, and NFO/link scope creep.

Primary evidence:

- `docs/workstreams/metadata-provider-breadth/DESIGN.md`
- `docs/workstreams/metadata-provider-breadth/TODO.md`

## M1 — Provider Capability Diagnostics

Exit criteria:

- TMDB, Douban, and Bangumi expose diagnostics-safe capabilities.
- `/metadata/providers` includes capabilities without secrets.
- Registry/provider tests cover the capability model.

Primary gates:

- `cargo nextest run -p taru-metadata registry --no-fail-fast`
- `cargo nextest run -p taru-server metadata_diagnostics --no-fail-fast`

## M2 — Matching Policy

Exit criteria:

- Candidate scores are translated into deterministic match decisions.
- Decisions include explainable reasons.
- Weak/ambiguous candidates can be distinguished from automatic accepts.

Primary gate:

- `cargo nextest run -p taru-metadata matching --no-fail-fast`

## M3 — Refresh Integration

Exit criteria:

- Safe external-ID refresh remains compatible.
- Ambiguous search-based refresh does not silently mutate canonical metadata.
- Attempts or diagnostics explain why confirmation is needed.

Primary gate:

- `cargo nextest run -p taru-metadata refresh --no-fail-fast`

## M4 — Conflict Review Boundary

Exit criteria:

- Cross-provider conflicts are reviewable through a service/API/diagnostic
  boundary.
- Durable review queue/UI scope is either implemented or split.
- Canonical metadata remains untouched until an accepted path commits it.

Primary gates:

- `cargo nextest run -p taru-metadata conflict --no-fail-fast`
- `cargo nextest run -p taru-server metadata_candidate_review --no-fail-fast`

## M5 — Closeout

Exit criteria:

- Workstream evidence is fresh.
- Docs teach the shipped provider capability and matching behavior.
- Follow-ons are split for durable candidate review, UI, NFO/link, managed
  import, AI, or addon metadata if still needed.

Primary gates:

- `cargo nextest run -p taru-api --no-fail-fast`
- `cargo nextest run -p taru-metadata --no-fail-fast`
- `cargo nextest run -p taru-server metadata --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
