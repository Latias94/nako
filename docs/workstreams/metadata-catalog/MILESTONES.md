# Metadata-Catalog Milestones

## M27.0: Metadata-Catalog Domain Baseline

Status: proposed.

Outcome: Taru has a documented, implementation-ready domain model for growing
from movie-first metadata into series, episode, anime, collection, artwork,
duplicate-source, and NFO-safe catalog behavior.

Scope: design baseline only. M27.0 should not add schema migrations, provider
features, runtime behavior, or public API changes. It should make the next
implementation slices unambiguous.

Deliverables:

- Audit current `taru-core`, `taru-db`, `taru-catalog`, `taru-metadata`,
  `taru-nfo`, and HTTP DTO surfaces against `CONTEXT.md`.
- Decide the first stable **Media Item** hierarchy for movie, series, season,
  episode, collection, and extra-like items.
- Decide how **Media Source** links to **Media Item** when one item has
  multiple playable sources.
- Decide the persistence shape for **Source Duplicate Relationship** without
  automatic source or item merging.
- Define **Metadata Source Priority** and field-lock behavior for local edits,
  NFO, TMDB, Douban, Bangumi, and future addon contributions.
- Define **NFO Round Trip** requirements for import/export without destructive
  rewrites.
- Define the first **Managed Artwork**, **Artwork Candidate**, and **Selected
  Artwork** route/storage contract.
- Move relevant TODO items out of `server-foundation` into this workstream.
- Record any architecture decision that materially changes schema, provider
  boundaries, or public API DTO shape.

Exit criteria:

- `CONTEXT.md` uses stable terms for the selected model
- M27.1 and M27.2 implementation boundaries are explicit
- current code gaps are recorded with enough detail for implementation
- required ADRs are created or linked
- `git diff --check`

## M27.1: Catalog Schema and Repository Slice

Status: planned after M27.0.

Outcome: the database and repository seams can persist the selected
metadata-catalog model.

Deliverables:

- Add or refactor durable records for selected item hierarchy and source links.
- Add migration coverage for duplicate-source relationships if selected in
  M27.0.
- Add focused repository tests before changing provider behavior.
- Keep old movie MVP behavior working through compatibility queries or a
  deliberate migration plan.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused `cargo nextest run -p taru-db`
- focused `cargo nextest run -p taru-core`
- `git diff --check`

## M27.2: Provider and NFO Expansion Slice

Status: planned after M27.1.

Outcome: built-in providers and NFO import/export can populate the expanded
catalog model without bypassing local authority rules.

Deliverables:

- Add TMDB series, season, and episode mapping.
- Add Douban provider MVP when the domain model can store its identifiers and
  localized metadata safely.
- Add Bangumi provider MVP when anime profiles can map to ordinary movie or
  episode item kinds instead of a separate hard media kind.
- Preserve local edits and NFO authority through **Metadata Source Priority**.
- Add NFO round-trip tests for preserved unknown content.

Exit criteria:

- `cargo fmt --all -- --check`
- `cargo check --workspace --tests`
- focused provider and NFO nextest runs
- `git diff --check`
