# Metadata Catalog Commit Atomicity

Status: Proposed
Last updated: 2026-05-18

This workstream deepens the commit path that turns provider or local metadata
refresh results into durable Media Item state, Catalog Item Graph records, and
Search Projection records.

The immediate goal is to remove the current partial-write window where catalog
graph replacement can succeed while search projection fails. Later slices can
pull more of the metadata refresh commit into the same explicit unit of work.

## First Slice

Make catalog hydration commit graph replacement and search projection through
one repository method so the SQLite adapter can persist both in one transaction.

## Follow-On Slices

- Fold provider raw response, provider mapping acceptance, and source-library
  confirmation into an explicit metadata refresh commit unit.
- Add failure-path tests that prove stale search projection cannot survive a
  failed catalog commit.
- Revisit NFO import and hierarchy confirmation after the metadata commit
  unit is deep enough.
