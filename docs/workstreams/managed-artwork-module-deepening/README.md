# Managed Artwork Module Deepening

Status: Completed
Last updated: 2026-05-19

## Purpose

Managed Artwork has grown through several correct vertical slices: candidate
acceptance, ingest, artifact storage, Selected Artwork publication, public image
serving, variants, gallery management, lifecycle cleanup, remediation, and
runtime controls. The product seams are now proven, but the implementation is
too concentrated in a few shallow modules.

This lane deepens the Managed Artwork modules without changing product
semantics. The goal is better locality around Artwork Candidates, Managed
Artwork Artifacts, Selected Artwork, image variants, lifecycle diagnostics, and
remediation while preserving the existing redaction and authority rules.

## Goals

- Split concentrated app-layer implementation into deeper private modules with
  small caller interfaces.
- Keep Managed Artwork storage authority internal to Nako.
- Preserve public/Admin redaction for raw source URLs, storage URIs, local
  paths, cache URIs, and content hashes.
- Make image variant serving, artifact store inventory, ingest processing, and
  lifecycle/remediation logic easier to test at their seams.
- Keep repository and API module seams aligned with `nako-core`, `nako-db`,
  `nako-api`, and `nako-server` ownership.
- Remove obsolete pass-through helpers once deeper modules make them
  unnecessary.

## Non-Goals

- Provider search, ranking, or provider payload expansion.
- Public Client gallery browsing.
- Persisted thumbnail cache eviction.
- Missing-artifact repair or re-ingest.
- New durable retry, cancellation, backoff, or lease semantics.
- Addon direct fetch/cache/publication side effects.
- Returning raw source URLs, storage URIs, local paths, cache URIs, or content
  hashes in public or Admin responses.

## Authoritative Docs

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
- `WORKSTREAM.json`

## Closeout

`MAMD-020` extracted Selected Artwork image variant serving into a private app
Module. `MAMD-030` extracted local Managed Artwork Artifact storage and
inventory into a private artifact store Module. `MAMD-040` extracted fetch,
validation, artifact write, prepared artifact construction, and safe failure
summary creation into an ingest pipeline Module while preserving durable job
commit ordering. `MAMD-050` split the SQLite Managed Artwork repository adapter
into concern-local modules. `MAMD-060` moved Managed Artwork Admin DTOs and
redaction tests into a concern-local API module. `MAMD-070` verified and closed
the lane without splitting additional follow-ons.

## Operating Notes

Use low-concurrency Rust validation for this lane:

```powershell
$env:CARGO_TARGET_DIR='G:\nako-cargo-target'
$env:CARGO_BUILD_JOBS='2'
$env:NEXTEST_TEST_THREADS='1'
```
