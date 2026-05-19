# Managed Artwork Gallery Candidate Management Task Ledger

Status: Active
Last updated: 2026-05-19

## M0 - Scope And Contract

- [x] MAGC-010 [owner=codex] [deps=none] [scope=docs/workstreams/managed-artwork-gallery-candidate-management,docs/workstreams/README.md]
  Goal: Open the gallery/candidate management lane with explicit terminology,
  redaction policy, route direction, non-goals, and validation gates.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with a redacted item-scoped Admin gallery read model.

## M1 - Admin Gallery Read Model

- [x] MAGC-020 [owner=codex] [deps=MAGC-010] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server,docs/api]
  Goal: Implement the first item-scoped Admin artwork gallery read model that
  shows current Selected Artwork, eligible Managed Artwork Artifacts, and safe
  Artwork Candidate summaries without exposing locators or hashes.
  Validation: focused API/server/db tests plus relevant cargo check.
  Review: response shape must use explicit Admin DTOs and must not reuse raw
  persistence records.
  Evidence: route response includes candidate/artifact/selection IDs, image
  kind, dimensions, statuses, selected flags, and public image refs where
  applicable; forbidden storage/source/cache/hash/path/token values are absent.
  Handoff: `GET /admin/v1/items/{item_id}/artwork` is the first read model.
  Continue with selection management; decide whether to preserve the existing
  artifact publish route as the command or add an item/kind-scoped command for
  stronger operator intent.

## M2 - Selection Management

- [ ] MAGC-030 [owner=codex] [deps=MAGC-020] [scope=crates/taru-core,crates/taru-db,crates/taru-api,crates/taru-server,docs/api]
  Goal: Add a safe management action for selecting/replacing an item's Selected
  Artwork from an eligible Managed Artwork Artifact while preserving existing
  public image references and redaction.
  Validation: focused API/server/db tests and public image route regression.
  Review: replacing selection must be explicit, idempotent where possible, and
  must not delete artifacts or files.
  Evidence: selected artwork changes only for the target item/kind; previous
  artifacts remain lifecycle-managed; Public Client item images return the new
  selected public image reference.
  Handoff: Split unpublish/delete behavior if it needs separate retention
  policy.

## M3 - Closeout

- [ ] MAGC-040 [owner=codex] [deps=MAGC-030] [scope=workspace,docs]
  Goal: Close the lane with fresh validation evidence, documented route
  contract, and explicit follow-ons.
  Validation: focused nextest gates; relevant cargo check; `cargo fmt --all
  -- --check`; `git diff --check`; redaction inventory.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Split Public gallery browsing, unpublish behavior, persisted variant
  cache/eviction, retry/cancel, and repair/re-ingest.
