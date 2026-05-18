# Addon Protected Writes Handoff

Status: Completed
Last updated: 2026-05-18

## Current State

This lane is closed. Taru now has the first concrete Addon protected write:
accepted `metadata_write` Addon Side Effects can apply a bounded Canonical
Metadata patch through Taru-owned metadata and catalog/search seams.

The shipped model separates Addon Side Effect intake validation from domain
apply outcome. `validation_status = accepted` means addon principal,
permission, library, and target validation passed. `apply_status = applied`
means a domain write committed. Replays by idempotency key return the stored
apply outcome and do not reapply.

APW also added first-class `MetadataSource::Addon(addon_id)` attribution rather
than mapping addon writes to fake provider provenance.

## Active Task

- Task ID: APW-060
- Owner: planner
- Files: `docs/workstreams/addon-protected-writes`,
  `docs/workstreams/addon-managed-artwork-artifacts`,
  `docs/workstreams/addon-library-file-write-policy`, `docs/workstreams/README.md`,
  `docs/api/HTTP_API.md`
- Validation: `cargo fmt --all -- --check`; `git diff --check`; focused addon,
  DB, catalog, and cross-crate checks recorded in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Review: closeout review found and fixed a catalog provenance issue before
  closeout
- Evidence: APW-030 code/tests/docs, APW-060 closeout evidence, and follow-on
  workstream docs

## Decisions Since Last Update

- Close APW after proving the protected-write apply model with Canonical
  Metadata instead of keeping artwork, subtitle, NFO, and sidecar-file breadth
  in one lane.
- Split `artwork_write`, Artwork Candidate, Managed Artwork, and
  Taru-Managed Artifact storage to
  `docs/workstreams/addon-managed-artwork-artifacts/`.
- Split subtitle, NFO, and sidecar-asset Library File Write policy to
  `docs/workstreams/addon-library-file-write-policy/`.
- Keep Addon Sidecars away from admin tokens, raw Source Locators, filesystem
  paths, database access, remote storage handles, and raw provider bodies.
- Keep Public Client API and generated SDK surfaces excluding `/addon/v1/*`
  protected-write routes.
- Do not write metadata field locks for Addon writes in this slice. Field locks
  are overwrite protection, not provenance. Future field-level Addon
  provenance should use a dedicated provenance/history model.
- Do not fabricate provider subjects, provider mappings, or provider raw
  responses for Addon writes.
- Reject unknown `metadata_write` payload fields so Addon Sidecars cannot
  believe broader writes succeeded.

## APW-030 Outcome

- `crates/taru-db/migrations/0023_addon_side_effect_apply_outcome.sql` adds
  `apply_status`, `apply_error_code`, `applied_item_id`, `applied_source`, and
  `applied_at` to Addon Side Effect records.
- `crates/taru-core/src/media/metadata.rs` defines
  `MetadataSource::Addon(AddonId)`. `crates/taru-db/src/codec.rs` persists it
  as `source = addon`, `source_key = <addon_id>`.
- `crates/taru-server/src/app/addons.rs` keeps HTTP thin by authenticating,
  validating, and recording intake before application-service helpers normalize
  and apply `metadata_write`.
- `metadata_write` supports title-like fields, overview, release date, runtime,
  tagline, genres, and tags.
- Scalar metadata patches refresh search without rewriting existing catalog
  graph label sources.
- Genre/tag patches replace only the touched label sets with Addon source
  attribution, preserving unrelated provider/NFO catalog graph provenance.
- Response DTOs include safe apply outcome fields but omit raw payload,
  provenance, Source Locators, filesystem paths, provider bodies, token hashes,
  and raw Addon Tokens.

## Blockers

- None for this lane.

## Residual Risks

- Media item update plus catalog/search refresh is still not one database
  transaction. This follows the existing metadata/NFO workflow shape; a future
  prepared-catalog unit of work remains separate architecture scope.
- Addon-specific domain events and field-level metadata provenance are not
  implemented. Add them only if a product workflow needs queryable provenance
  beyond side-effect audit records.
- `metadata_write` intentionally remains a minimal Canonical Metadata patch.
  Wider metadata fields should be added as narrow tasks with explicit merge,
  catalog, and redaction tests.

## Next Recommended Action

- If the next user-visible value is cover/poster/backdrop import, continue with
  `docs/workstreams/addon-managed-artwork-artifacts/` task AMAA-010.
- If the next value is subtitles, NFO export, or sidecar assets, continue with
  `docs/workstreams/addon-library-file-write-policy/` task ALFW-010.
