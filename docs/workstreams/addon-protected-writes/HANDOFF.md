# Addon Protected Writes Handoff

Status: Active
Last updated: 2026-05-18

## Current State

This lane has been split from the completed Addon Token Grants Side Effects
workstream. No protected-write apply code has been changed yet. The existing
system can authenticate Addon Tokens, enforce accepted permissions and
Library-Scoped Addon Grants, persist Addon Side Effect intake, enforce
idempotency, and return redacted intake summaries.

APW-020 audited the current protected-write seams and selected Canonical
Metadata as the first concrete apply target. The audit found one important
precondition: `validation_status = accepted` means the intake was authorized
and recorded, not that a domain write was applied. APW-030 must therefore add
explicit apply outcome state and Addon metadata source attribution before it
applies even the smallest metadata payload.

## Active Task

- Task ID: APW-030
- Owner: codex
- Files: `crates/taru-core`, `crates/taru-db`, `crates/taru-server`,
  `crates/taru-api`, `crates/taru-metadata`, `crates/taru-catalog`, `docs/api`
- Validation: `cargo check -p taru-core -p taru-db -p taru-api -p taru-server -p taru-metadata -p taru-catalog --tests`; focused `cargo nextest run -p taru-server addon_side_effect --no-fail-fast`; `cargo nextest run -p taru-db addon --no-fail-fast`; relevant metadata/catalog tests; `cargo fmt --all -- --check`; `git diff --check`
- Status: READY
- Review: ensure HTTP handlers do not own metadata merge logic, and response
  DTOs do not expose raw payload/provenance, Source Locators, filesystem paths,
  provider bodies, token hashes, or raw Addon Tokens
- Evidence: APW-030 code/tests/docs plus APW-020 audit notes in
  `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Concrete protected writes are a separate scope boundary from Addon Token,
  accepted grant, addon-principal, and intake proof.
- The first task is an audit, not immediate code, because metadata, artwork,
  subtitle, NFO, storage/VFS, and catalog seams have different owners.
- Canonical Metadata is the selected first apply slice because Taru already has
  `MetadataMergePolicy`, metadata persistence commits, and catalog hydration
  seams.
- Add a distinct side-effect apply outcome model before applying writes. Do not
  overload `AddonSideEffectValidationStatus`.
- Add or choose an explicit Addon metadata attribution/source model before
  writing field locks, provider mappings, catalog graph source facts, or event
  payloads.
- Addon Sidecars must not receive admin tokens, raw Source Locators, filesystem
  paths, database access, or remote storage handles.
- Public Client API and generated SDK surfaces should continue excluding
  `/addon/v1/*` protected write routes.
- Artwork and Library File Write remain heavier follow-ons: artwork currently
  has assets/tasks but no Managed Artwork fetch/cache apply service, while
  NFO/VFS has mature atomic write and backup policy but requires path
  derivation and backup semantics that are too broad for the first apply slice.

## Blockers

- None known.

## APW-020 Findings

- Addon intake: `crates/taru-core/src/addon.rs`,
  `crates/taru-db/migrations/0022_addon_side_effects.sql`,
  `crates/taru-server/src/app/addons.rs`, and
  `crates/taru-api/src/extension.rs` persist validation-only side effects.
- Metadata/catalog: `MetadataMergePolicy`,
  `MetadataRepository::commit_metadata_refresh`, and
  `CatalogRepository::commit_item_projection` are the strongest reusable seams.
- Source attribution gap: `MetadataSource` currently has `Local`, `Nfo`,
  `User`, and `Provider`, but no Addon source. DB codec fallback maps unknown
  sources to `Provider(Other(_))`, which is not correct for Addon writes.
- Event gap: `DomainEventKind` has `ItemMetadataRefreshed`, but no addon
  metadata-applied event. APW-030 can reuse a safe metadata event only if the
  payload clearly identifies addon apply without leaking raw payload.
- Artwork: `ImageAsset` and `ArtworkTask` exist, but Managed Artwork import,
  fetch/cache ownership, and artifact storage policy are not as ready as
  metadata.
- NFO/VFS: NFO export already uses `StorageWriteRequest::atomic_replace` with
  backup policy, but addon-driven Library File Write needs path derivation,
  sidecar target policy, and backup/report redaction before implementation.

## Next Recommended Action

- Run APW-030. Start by adding side-effect apply outcome and Addon metadata
  source attribution, then implement the smallest `metadata_write` apply path
  through metadata merge and catalog hydration.
