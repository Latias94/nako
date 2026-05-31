# Remote Storage Health And Circuit Breaker

Status: Closed
Last updated: 2026-05-31

## Why This Lane Exists

Remote and mounted storage can be slow, stale, or temporarily unavailable.
Nako already has first-slice redaction-safe failure classification and bounded
process-local backoff, but the product still lacks one durable health contract
that scan, probe, playback staging, and Admin diagnostics can share.

Without that contract, each runtime path is tempted to invent its own retry
state. That makes operator behavior hard to explain and makes future remote
libraries depend on accidental per-feature policy.

## Relevant Authority

- ADRs:
  - `docs/adr/0016-remote-storage-and-vfs-cache-boundary.md`
  - `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
  - `docs/adr/0053-application-control-plane-boundary.md`
- Existing docs:
  - `CONTEXT.md`
  - `docs/architecture/STORAGE_VFS.md`
  - `docs/architecture/WORKSTREAM_LINKS.md`
  - `docs/architecture/LANES.md`
- Related workstreams:
  - `docs/workstreams/storage-vfs-resilience-and-source-identity/`

## Problem

Storage health is currently observable in narrow runtime diagnostics, but not
modeled as a durable backend state with repository parity, circuit-breaker
decision inputs, operator reset, and consistent redaction rules.

## Target State

When this workstream closes:

- **Storage Backend Health** is a durable repository contract with SQLite and
  PostgreSQL parity.
- repeated backend failures can open a **Storage Circuit Breaker** with bounded
  and explainable policy.
- scan, probe, and playback staging can consult the same health decision
  surface without owning storage policy.
- Admin diagnostics can show redaction-safe health, last failure class,
  recovery state, and operator reset results.

## In Scope

- durable health records and repository contract tests;
- backend health summary/query/update APIs in the storage-facing domain;
- runtime policy adapter for storage/VFS callers;
- Admin diagnostics and reset action after the durable contract exists;
- workstream evidence, focused tests, and architecture documentation updates.

## Out Of Scope

- cache repair previews or stale-cache remediation;
- source fingerprint partial/full hash escalation;
- PostgreSQL runtime harness work unrelated to storage health;
- changing media-library provider semantics;
- broad playback artifact I/O scheduling or transcode resource scheduling.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Storage health belongs behind repository traits before server runtime policy. | High | `nako-core` and `nako-db` already own durable repository contracts. | Runtime paths would duplicate health state or become SQLite-only. |
| Circuit-breaker state should be backend-scoped, not source-scoped. | Medium | ADR 0016 and storage architecture treat backend failures as shared product behavior. | Some per-source failures may need a later narrower suppression model. |
| Operator reset is useful only after health state is durable. | High | Admin diagnostics already expose storage state; reset without persistence would be misleading. | Reset route would become a one-off runtime helper instead of a control-plane action. |

## Architecture Direction

Model storage health as a domain/repository contract first. SQLite and
PostgreSQL adapters should prove the state shape before server runtime paths
consume it. The runtime policy layer can then classify failures into health
updates and ask whether new work should proceed, back off, or surface a
temporarily unavailable storage error.

Admin routes should read from the same contract. They may expose reset/clear
actions, but not raw paths, locators, filenames, credentials, command lines, or
provider-specific secrets.

## Closeout Condition

This lane can close when:

- durable health records and repository parity pass focused tests;
- runtime storage policy uses the health contract for bounded work admission;
- Admin diagnostics and reset behavior are redaction-safe and tested;
- `STORAGE_VFS.md`, `WORKSTREAM_LINKS.md`, and this workstream evidence match
  shipped behavior;
- follow-ons for cache repair, hash escalation, or PostgreSQL runtime harnesses
  are split or explicitly deferred.
