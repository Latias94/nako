# Selected Artwork Unpublish Delete Policy Task Ledger

Status: Completed
Last updated: 2026-05-19

## M0 - Scope And Contract

- [x] SAUD-010 [owner=codex] [deps=none] [scope=docs/workstreams/selected-artwork-unpublish-delete-policy,docs/workstreams/README.md]
  Goal: Open the Selected Artwork unpublish/delete policy lane with explicit
  lifecycle terminology, route direction, redaction rules, non-goals, and
  validation gates.
  Validation: Workstream docs exist and agree; `WORKSTREAM.json` parses.
  Evidence: `DESIGN.md`, `EVIDENCE_AND_GATES.md`.
  Handoff: Continue with the item/kind-scoped Admin unpublish command.

## M1 - Unpublish Command

- [x] SAUD-020 [owner=codex] [deps=SAUD-010] [scope=crates/nako-core,crates/nako-db,crates/nako-api,crates/nako-server,docs/api]
  Goal: Implement `DELETE /admin/v1/items/{item_id}/artwork/{kind}/selection`
  so Admin users can remove a Selected Artwork publication slot without
  deleting the linked artifact or file.
  Validation: focused API/server/db tests plus relevant cargo check.
  Review: route must be item/kind-scoped, idempotent for existing empty slots,
  and must not call artifact cleanup or physical deletion.
  Evidence: response returns `changed = true` with the redacted previous
  selection when a slot existed, `changed = false` when no slot existed, and no
  forbidden locator/hash/path fields.
  Handoff: Implemented as `DELETE
  /admin/v1/items/{item_id}/artwork/{kind}/selection`; continue with lifecycle
  proof that unpublish changes cleanup eligibility but not artifact retention.

## M2 - Public And Lifecycle Regression

- [x] SAUD-030 [owner=codex] [deps=SAUD-020] [scope=crates/nako-db,crates/nako-server,docs/api]
  Goal: Prove the public and lifecycle consequences of unpublish: item image
  lists no longer expose the slot, `/images/{old_selected_id}` returns `404`,
  and the artifact remains stored until explicit lifecycle cleanup.
  Validation: focused server/db tests and redaction inventory.
  Review: tests should distinguish public visibility from artifact retention.
  Evidence: old public image references are not silently redirected to artifact
  IDs; lifecycle diagnostics can report the artifact as cleanup-eligible only
  because no Selected Artwork rows reference it.
  Handoff: Public item images omit unpublished slots, old `GET`/`HEAD
  /images/{old_selected_id}` return `404`, and artifacts remain retained until
  explicit lifecycle cleanup.

## M3 - Closeout

- [x] SAUD-040 [owner=codex] [deps=SAUD-030] [scope=workspace,docs]
  Goal: Close the lane with fresh validation evidence, HTTP docs, and explicit
  follow-ons.
  Validation: focused nextest gates; relevant cargo check; `cargo fmt --all
  -- --check`; `git diff --check`; redaction inventory.
  Evidence: `EVIDENCE_AND_GATES.md` and `HANDOFF.md`.
  Handoff: Lane closed. Keep artifact deletion, repair/re-ingest, Public
  gallery browsing, variant cache eviction, and ingest retry/cancel as separate
  lanes.
