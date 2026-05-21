# Metadata Provider Breadth — TODO

Status: Completed
Last updated: 2026-05-21

Task IDs use the `MPB` prefix.

## M0 — Scope And Evidence Freeze

- [x] MPB-010 [owner=planner] [deps=none] [scope=docs/workstreams/metadata-provider-breadth]
  Goal: Open the metadata provider breadth lane and freeze first-slice scope.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: `docs/workstreams/metadata-provider-breadth/DESIGN.md`
  Handoff: Execute MPB-020 first.

## M1 — Provider Capability Diagnostics

- [x] MPB-020 [owner=codex] [deps=MPB-010] [scope=crates/taru-metadata,crates/taru-api,crates/taru-server]
  Goal: Add a diagnostics-safe capability model for TMDB, Douban, and Bangumi and expose it on metadata provider diagnostics.
  Validation: `cargo nextest run -p taru-metadata registry --no-fail-fast`; `cargo nextest run -p taru-server metadata_diagnostics --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Capabilities must not expose resolved tokens, header values, proxy URLs, local paths, or raw provider payloads.
  Evidence: capability types in `taru-metadata`, diagnostics DTO in `taru-api`, `/metadata/providers` route test.
  Handoff: DONE. Continued with matching policy.

## M2 — Matching Policy And Ambiguity Vocabulary

- [x] MPB-030 [owner=codex] [deps=MPB-020] [scope=crates/taru-metadata]
  Goal: Introduce an explicit candidate match decision model for accepted, needs-confirmation, and rejected provider candidates.
  Validation: `cargo nextest run -p taru-metadata matching --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Keep thresholds explainable and deterministic; do not add AI or probabilistic opaque ranking.
  Evidence: matching policy tests for exact title/year, strong match, weak match, and conflicting candidates.
  Handoff: DONE. Continued with refresh strategy integration.

## M3 — Non-Destructive Ambiguous Refresh

- [x] MPB-040 [owner=codex] [deps=MPB-030] [scope=crates/taru-metadata,crates/taru-server]
  Goal: Make ambiguous search-based provider selection non-destructive while preserving safe external-ID refresh behavior.
  Validation: `cargo nextest run -p taru-metadata refresh --no-fail-fast`; targeted server metadata tests if route behavior changes; `git diff --check`.
  Review: Existing external-ID refresh tests must remain compatible; ambiguous search must not commit canonical metadata.
  Evidence: tests showing external-ID refresh still commits and ambiguous search records an explainable non-success outcome.
  Handoff: DONE. Durable review queue was split; first non-destructive candidate review boundary continued in MPB-050.

## M4 — Cross-Provider Conflict Review Boundary

- [x] MPB-050 [owner=codex] [deps=MPB-040] [scope=crates/taru-metadata,crates/taru-api,crates/taru-server,docs]
  Goal: Provide the first reviewable cross-provider conflict boundary without building a full Admin UI.
  Validation: focused metadata/server tests for conflict output; docs updated; `git diff --check`.
  Review: If durable review queues require schema work, split it instead of expanding this task.
  Evidence: service/API or diagnostics output that lists conflicting provider candidates and leaves canonical state untouched.
  Handoff: DONE. Continue with docs and closeout.

## M5 — Docs And Closeout

- [x] MPB-060 [owner=planner] [deps=MPB-050] [scope=docs/workstreams/metadata-provider-breadth,docs/workstreams/post-rpd-product-hardening,docs/workstreams/README.md]
  Goal: Document shipped capability/matching/conflict behavior, update roadmap evidence, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; focused metadata/server gates; `git diff --check`.
  Review: Run review-workstream and verify-rust-workstream before closeout.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md, closeout journal.
  Handoff: DONE. Re-score NFO/link authority and playback/transcode hardening in the umbrella.
