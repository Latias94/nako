# Addon Protected Writes

Status: Completed
Last updated: 2026-05-18

## Why This Lane Exists

`addon-token-grants-side-effects` proved the trust boundary for Addon Tokens,
accepted Addon Permissions, Library-Scoped Addon Grants, addon-principal
runtime routes, and a redacted Addon Side Effect intake record. That lane
intentionally stopped before applying canonical metadata, artwork, subtitle,
NFO, or library sidecar writes.

This lane owned the next boundary: how an accepted Addon Side Effect becomes a
Nako-owned protected write without giving an Addon Sidecar admin authority,
database access, raw Source Locators, or filesystem handles.

Closeout result: APW proved the apply model with a bounded Canonical Metadata
`metadata_write` slice. Artwork/artifact and subtitle/NFO/Library File Write
breadth is split to dedicated follow-on lanes because those areas require
separate storage, fetch, backup, and redaction policy.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/workstreams/addon-token-grants-side-effects/`
- `docs/workstreams/metadata-merge-policy-unification/`
- `docs/workstreams/metadata-catalog-commit-atomicity/`
- `docs/workstreams/nfo-round-trip-preservation/`
- `docs/workstreams/nfo-storage-write-policy/`
- `docs/workstreams/nfo-sidecar-backup-policy/`
- `docs/workstreams/public-client-source-locator-redaction/`
- `docs/api/HTTP_API.md`
- `crates/nako-core/src/addon.rs`
- `crates/nako-core/src/repository/addon.rs`
- `crates/nako-db/src/addons.rs`
- `crates/nako-server/src/app/addons.rs`
- `crates/nako-server/src/http/addons.rs`

## Problem

Nako can now accept and audit an Addon Side Effect, but the accepted record is
not yet an applied domain change. If concrete handlers grow one by one without
a shared apply model, Nako risks reintroducing the shortcuts the intake lane was
created to prevent:

- metadata handlers could bypass Canonical Metadata merge policy;
- artwork handlers could hotlink provider URLs instead of producing Managed
  Artwork;
- subtitle and NFO handlers could become direct path writes instead of Library
  File Write operations;
- idempotency and audit could stop at intake while apply failures become
  invisible;
- catalog/search projections could drift if addon writes do not reuse existing
  commit boundaries;
- Public Client or Admin API DTOs could accidentally expose raw Source Locators,
  payload snapshots, provider bodies, or library filesystem paths.

## Target State

- Accepted Addon Side Effects move through an explicit effect-specific apply
  boundary owned by Nako.
- Effect handlers reuse the same addon-principal, accepted permission,
  concrete Media Library, target resolution, idempotency, audit, and redaction
  rules proven by ATGSE.
- Canonical Metadata writes pass through Nako's metadata merge and catalog
  consistency seams.
- Managed Artwork, subtitle, NFO, and other Library File Write behavior uses
  Nako storage/VFS and sidecar write policy rather than addon-provided paths.
- Apply results are inspectable through safe summaries without returning raw
  token material, token hashes, payload/provenance JSON, Source Locators,
  filesystem paths, or raw provider bodies.
- Remaining protected-write breadth is split into narrower follow-on lanes.

## In Scope

- Audit existing metadata, artwork, subtitle, NFO, storage/VFS, catalog, and
  Addon Side Effect seams before changing apply behavior.
- Decide the minimum apply-state model needed beyond intake validation status.
- Implement the first concrete protected-write slice for Canonical Metadata if
  the audit confirms the target seam is ready.
- Define how Managed Artwork and Nako-Managed Artifact outputs enter Nako from
  an Addon Sidecar.
- Define how subtitle, NFO, and other Library File Write requests are
  authorized, normalized, backed up, written, and safely reported.
- Update HTTP/API docs and workstream evidence for shipped behavior.

## Out Of Scope

- Redesigning Addon Token issuance, rotation, revocation, or accepted-grant
  storage from the ATGSE lane.
- Addon Manager discovery, install, update, process supervision, marketplace,
  signing, or log collection.
- OAuth, device flow, or remote multi-tenant addon authorization.
- Jellyfin Plugin Compatibility or an in-process Native Plugin ABI.
- Public Client API write routes.
- A complete field-level metadata permission matrix in the first slice.
- Replacing existing metadata, NFO, storage/VFS, or catalog architecture when a
  narrower adapter can preserve the existing ownership boundary.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The ATGSE token/grant/principal/intake contract is the correct prerequisite for protected writes. | High | ADR 0020 and ATGSE-050 evidence. | Reopen ATGSE before applying writes. |
| Canonical Metadata is the safest first concrete write slice. | Medium | Existing metadata merge and catalog atomicity lanes provide stronger domain seams than artwork/subtitle breadth. | Start with a smaller artifact or NFO slice if APW-020 finds metadata apply is still too coupled. |
| Intake validation status should not be overloaded as write-application state. | Medium | Accepted intake currently means "authorized and recorded", not "domain mutation applied". | Add a distinct apply status/result model or split a follow-on before implementation. |
| Library File Write behavior must reuse Nako storage/VFS policy. | High | `CONTEXT.md` and NFO write lanes reject direct addon path writes. | Protected file writes remain deferred until the policy seam is explicit. |
| Addon payloads can stay internal/audit-only while responses expose safe summaries. | High | ATGSE redaction tests and Public Client Source Locator redaction. | Add a separate admin diagnostics contract before exposing payload detail. |

## Architecture Direction

Keep protected writes as a three-stage Nako workflow:

1. Runtime intake authenticates the Addon Token, resolves the Addon principal,
   enforces accepted Addon Permission plus concrete Media Library scope, stores
   the Addon Side Effect record, and returns a redacted summary.
2. Effect-specific validation normalizes the payload into a Nako domain command
   such as Canonical Metadata update, Managed Artwork candidate/import, subtitle
   import, NFO Export, or Library File Write.
3. Domain apply calls the owning Nako service/repository boundary and records a
   safe apply result that can be replayed by idempotency key.

The HTTP route may perform the first synchronous apply slice if it remains
small, but the domain write must still live behind application-service seams.
If apply work needs long-running fetches, file writes, thumbnailing, backups, or
catalog rebuilds, create a queued Addon Task or durable job path instead of
blocking the runtime request.

Prefer adding narrow adapters around existing metadata, NFO, storage/VFS, and
catalog services over embedding write logic inside `nako-server/src/http`.
Public Client API inventory and SDK generation must continue excluding
`/addon/v1/*` protected write routes.

## Closeout Condition

This lane closed when:

- APW-020 records the current protected-write seam audit and first apply target;
- at least one concrete protected-write apply path is implemented or explicitly
  split with evidence-backed reasoning;
- Addon Side Effect idempotency and audit cover both intake and apply outcome
  for shipped write behavior;
- metadata/artwork/subtitle/NFO/Library File Write breadth is either completed,
  deferred, or split into narrower follow-ons;
- docs describe protected writes without admin tokens, raw storage authority,
  raw Source Locators, or direct addon file writes;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
