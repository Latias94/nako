# Library Metadata Scan Policy - Design

Status: Closed
Last updated: 2026-05-25

## Problem

Real Media Library validation showed that a scan discovers and probes sources,
but NFO metadata is only applied after a separate `import-nfo` command. That is
surprising for a self-hosted media server: users expect library scanning to
also apply configured local metadata inputs.

The broader requirement is not "always import NFO after scan". Nako needs a
library-level metadata acquisition boundary that can later coordinate local
readers, network providers, Addon metadata scrape tasks, embedded metadata, and
image/artwork discovery according to the library's `MetadataProfile`.

Jellyfin's product model is useful as a reference: libraries expose metadata
and image fetcher choices and local `.nfo` files are treated as a high-priority
local source. Nako should keep the same operator intuition while using Nako
language and boundaries: Media Library, Metadata Profile, Metadata Scrape,
NFO Import, Canonical Metadata, Metadata Source Priority, Addon Task, and
Acceptance Workflow.

## Target State

- A Media Library has an explicit metadata scan policy derived from its
  `MetadataProfile`.
- Library scan can produce one combined, operator-visible output with scan,
  probe, and metadata acquisition summaries.
- The first shipped metadata acquisition step is NFO Import when:
  - `local_metadata_policy` is not `disabled`;
  - `local_readers` contains `nfo`;
  - scan-time metadata acquisition is enabled for the library.
- NFO Import keeps using the existing NFO service, NFO Round Trip, source-aware
  merge policy, catalog/search commit ordering, and durable job cancellation
  behavior.
- Provider refresh, Addon Bulk Metadata Scrape, embedded metadata, sidecar
  readers, and image/artwork discovery become future plan steps, not scan-time
  ad hoc code.

## Shipped State

The NFO-only scan acquisition slice is complete as of 2026-05-25:

- `MetadataProfile` builds a `MetadataScanAcquisitionPlan`.
- Per-library profile overrides live under `metadata.library_profiles`.
- Library scan runs NFO Import after index/probe when the profile enables local
  NFO acquisition.
- Scan output and durable job outcome include the NFO import summary.
- Public API DTOs and generated SDKs expose the scan policy shape.
- Provider refresh, Addon scrape, embedded readers, sidecar breadth, image
  discovery, and full NAS root scanning remain explicit follow-ons.

## Scope

- Add a small profile-derived metadata acquisition plan in Nako-owned code.
- Add library config knobs only where needed to let operators disable
  scan-time metadata acquisition or disable local readers.
- Wire the scan command/job path to run NFO Import after index/probe.
- Keep the existing manual `import-nfo` command and route.
- Add tests proving scan-time NFO import applies metadata and can be disabled
  through library profile configuration.
- Re-run real local and SMB smoke against temporary config.

## Non-Goals

- Do not add new external metadata provider behavior.
- Do not run Addon Sidecars during scan in this slice.
- Do not change Addon Protocol, Addon Task dispatch, or package distribution.
- Do not add schema migrations for persisted metadata plans.
- Do not change NFO XML parsing/export semantics.
- Do not scan the full NAS library before progress/cancellation visibility is
  good enough for a large SMB tree.

## Architecture Direction

`nako-core::MetadataProfile` already carries the key product terms:
`local_readers`, `metadata_providers`, `image_providers`, `refresh_mode`, and
`local_metadata_policy`. This lane should deepen that model by adding a
scan-time acquisition mode and by introducing a server-side plan builder that
turns a profile into executable steps.

The first executable step should call the existing `NfoService`/`NfoAppService`
import path rather than duplicating NFO discovery or metadata merge logic in
the scan service. The scan service may own orchestration and summary shape; the
NFO boundary continues to own sidecar discovery, parsing, merge policy, field
locks, hierarchy confirmation, catalog/search updates, and NFO import events.

Provider and Addon breadth should be added as future plan steps:

- provider metadata refresh: use `MetadataRefreshService` and existing provider
  attempts;
- Addon metadata scrape: use Addon Task lifecycle and Addon Token grants;
- embedded/sidecar readers: add first-party local reader implementations before
  enabling them in the plan;
- artwork discovery: route candidates through Managed Artwork boundaries.

## Reference Notes

Jellyfin's docs describe local `.nfo` metadata files as local metadata stored
near media files and generated/read by the server's NFO plugin. Jellyfin's
library settings also expose per-library metadata/image fetcher choices. Nako
should adopt the operator-facing lesson, not Jellyfin's plugin API or internal
object model.
