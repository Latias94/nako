# Managed Artwork Thumbnail Variants Task Ledger

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Contract

- [x] MATV-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-thumbnail-variants,docs/workstreams/README.md]
  Goal: Open the thumbnail variants lane with explicit public/Admin contract,
  non-goals, and validation gates.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with bounded on-demand variant serving.

## M1 - Bounded On-Demand Variant Serving

- [x] MATV-020 [owner=codex] [deps=MATV-010] [scope=crates/nako-api,crates/nako-server,crates/nako-client,sdk/typescript,docs/api]
  Goal: Add bounded `width`/`height` query parameters for `GET/HEAD
  /images/{image_id}`, derive variants on demand, keep original serving
  compatible, and switch public validators away from artifact content hashes.
  Validation: focused API/server image variant tests plus relevant cargo check.
  Evidence: resized bytes are smaller and keep aspect ratio; `HEAD` returns
  matching variant headers; invalid dimensions fail redacted; content hashes are
  absent from DTOs and headers.
  Result: DONE. `GET/HEAD /images/{image_id}` accepts optional `width` and
  `height`; original serving remains compatible; variants are derived on
  demand with configured artwork limits; Public/Admin image references do not
  expose artifact content hashes; HTTP ETags are opaque presentation
  validators. Persisted variant caching remains a follow-on.

## M2 - Validation And Closeout

- [x] MATV-030 [owner=codex] [deps=MATV-020] [scope=workspace,docs]
  Goal: Close the lane with fresh validation evidence and documented follow-ons.
  Validation: `cargo fmt --all -- --check`; focused nextest gates; relevant
  workspace `cargo check`; `git diff --check`.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Result: DONE. Fresh closeout evidence is recorded in
  `EVIDENCE_AND_GATES.md`; `HANDOFF.md` lists residual follow-ons.
