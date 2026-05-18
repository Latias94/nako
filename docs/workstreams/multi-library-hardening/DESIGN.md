# Multi-Library Hardening

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

Taru's M8 baseline fixed the first class of multi-library bugs: source locators
are only unique inside a Media Library, and CLI operations must choose one
library or explicitly scan all configured libraries. The next risk is authority:
configured libraries and persisted Library rows can drift, and several server
paths still need callers to understand whether configuration or database state
is the source of truth.

This lane is routed from ARF-004 in the 2026-05-18 architecture review.

## Relevant Authority

- `CONTEXT.md`
- `docs/workstreams/multi-library-hardening/PHASE8_0_CORRECTNESS_BASELINE.md`
- `docs/workstreams/server-architecture-hardening/`
- `docs/workstreams/repository-seam-deepening/`
- `docs/adr/0019-server-architecture-hardening-boundaries.md`

## Problem

The current code already treats **Media Source** identity as
`(library_id, locator)`, but Library records are still partly configuration and
partly database state. Startup code, scan jobs, NFO jobs, metadata maintenance,
storage backend diagnostics, and Public/Admin APIs all need a consistent answer
to these questions:

- which configured libraries are persisted at startup;
- whether a persisted Library missing from config is disabled, deleted, or
  retained for audit;
- whether config changes update names, roots, presets, and local metadata
  policies in place;
- which layer owns validation for duplicate IDs, duplicate roots, and
  unsupported backend roots;
- how job and event records preserve library identity after configuration
  changes.

Without one reconciliation boundary, each workflow can accidentally implement
its own version of Media Library authority.

## Target State

- Startup has one explicit Media Library reconciliation workflow.
- The database has the authoritative Library rows used by workflows after
  startup, while configuration remains the administrator's desired state input.
- Reconciliation behavior is documented for added, updated, missing, duplicate,
  and invalid libraries.
- Library-scoped workflows look up the reconciled Library state instead of
  re-parsing broad server configuration.
- Tests cover multi-library startup, scan, NFO, metadata, and storage
  diagnostics boundaries without relying on one-library shortcuts.

## In Scope

- Promote this historical lane into standard workstream docs.
- Define config-to-database Library reconciliation semantics.
- Add or adjust repository/service boundaries needed for startup
  reconciliation.
- Remove remaining one-library fallback helpers when their replacement is
  covered.
- Add tests for duplicate configured library IDs, missing persisted libraries,
  root/preset updates, and workflow use of reconciled Library rows.

## Out Of Scope

- Public Client Source Locator redaction; that is
  `public-client-source-locator-redaction`.
- Addon permission grants and side effects.
- New storage backends.
- Library Access, RBAC, or multi-user sharing.
- Source duplicate relationship or source variant merging.
- Public wire-shape changes except when needed to preserve existing library
  IDs and safe diagnostics.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Configuration should remain the desired-state input for libraries at startup. | High | `taru-server::config` owns configured libraries today. | Need an admin mutation API before the database can be the only authority. |
| Workflows should use reconciled database Library rows after startup. | High | Repository and job records already store `library_id`. | Workflows continue to duplicate config lookup and drift. |
| Missing configured libraries should not silently delete persisted data. | High | Jobs, events, sources, and metadata reference Library IDs. | Need an explicit retirement model before deletion. |
| Public locator redaction is related but separate. | High | Public DTOs expose `locator`, while M8 identity work is internal correctness. | Mixing both would combine contract and startup-reconciliation risk. |

## Architecture Direction

Introduce a startup reconciliation boundary rather than spreading config checks
through application services. `taru-server` should compose configuration and
call a focused service; `taru-db` should persist the resulting Library rows; and
workflow services should read Library state through existing or narrowed
repositories.

The first executable task should be characterization. Before deleting helpers,
tests should make the current behavior visible for configured libraries,
persisted libraries, and ambiguous one-library shortcuts.

## Closeout Condition

This lane can close when:

- reconciliation semantics are implemented and documented;
- configured and persisted Library state agree after startup;
- workflow code no longer needs broad config access for ordinary
  library-scoped operations;
- targeted Rust gates and `git diff --check` pass;
- and follow-ons for Library Access or admin mutation APIs are split.
