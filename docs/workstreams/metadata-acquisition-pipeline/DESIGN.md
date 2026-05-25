# Metadata Acquisition Pipeline Design

Status: implemented 2026-05-25.

## Context

Media Library scans currently run indexing, probing, then a hard-coded
scan-time metadata section inside `LibraryScanAppService`. That section knows
about NFO import and Addon Bulk Metadata Scrape directly. This shape worked for
the first NFO and Addon proof, but it is already too narrow for libraries that
need configurable metadata acquisition from NFO, embedded tags, sidecars,
providers, or Addons.

The existing protected write boundary remains correct:

- NFO import writes through the NFO workflow and merge policy.
- Addon writes must go through Addon Side Effects with scoped Addon Tokens and
  Library-Scoped Addon Grants.
- Addon Task output is sidecar-owned diagnostic/result data and must not become
  an implicit database mutation path.

## Target Shape

Scan orchestration should delegate metadata work to a dedicated
`MetadataScanAcquisitionService`. That service derives ordered phases from the
Media Library `MetadataProfile` and executes only the phases enabled by the
profile.

Initial phases:

- `local_nfo_import`: existing NFO import behavior, including cancellation.
- `addon_bulk_scrape`: host-dispatched Addon TaskRun creation for official
  `bulk-metadata-scrape` declarations.

`addon_bulk_scrape` has two modes:

- Suggestion-only: default. Payload omits `writeback`, and the sidecar returns
  task output without write authority.
- Explicit metadata writeback: opt-in. Payload includes a `writeback` object for
  each source. A sidecar may then call `/addon/v1/side-effects`, where Nako
  validates token, grant, target, idempotency, merge policy, and catalog/search
  commit.

Future phases such as embedded metadata, non-NFO sidecars, and provider refresh
should plug into the same service without changing library scan orchestration.

## Shipped Behavior

`LibraryScanAppService` now delegates scan-time metadata work to
`MetadataScanAcquisitionService`. `MetadataScanPolicy` exposes disabled-by-default
`addon_writeback`; when both `addon_scrape` and `addon_writeback` are enabled,
scan-triggered official bulk scrape payloads include a per-source `writeback`
object for a Media Source target.

The closed-loop proof uses a sidecar task endpoint that calls Nako's
`/addon/v1/side-effects` route with a scoped Addon Token and Library-Scoped
Metadata Write grant. The Media Item is updated through the existing merge
policy and catalog/search commit path.

## Invariants

- Default scan behavior must remain backward compatible.
- `scan.enabled = false` disables all scan-time metadata phases.
- `scan.addon_scrape = false` prevents Addon task creation even when writeback
  is enabled.
- `scan.addon_writeback = true` only asks the Addon to submit a side effect; it
  does not grant permission by itself.
- Side-effect permission and target validation remain Nako-owned.

## Validation

- Focused core tests for plan derivation and policy defaults.
- Server app tests for default suggestion-only Addon payloads.
- Server app test for opt-in writeback payload shape.
- End-to-end server app test where an Addon task path submits metadata_write via
  the Addon runtime route and the Media Item metadata reflects the merge.
- Focused `cargo nextest` packages before broadening as risk requires.
