# Core Architecture Deepening Design

Status: Completed
Last updated: 2026-05-18

## Problem

Taru has moved in the right direction: the workspace is split by domain, the
server root is mostly a composition point, metadata merge authority is in
`taru-core`, catalog graph/search projection commit is atomic, and the runtime
supervisor owns background work.

The remaining friction is concentrated in several shallow interfaces:

- NFO import still exposes ordered multi-record writes to the caller.
- Library indexing records Media Item, Media Source, Source State, Library Item
  State, Local Inference Evidence, Search Projection, and ingestion-failure
  resolution through separate calls.
- Several application services still carry broad `SqliteStore` knowledge where
  a focused workflow port would give more locality.
- HLS transcode reuse still uses request identity that is too weak for future
  profile decisions.
- Hardware acceleration diagnostics can list FFmpeg encoders without proving
  runtime capability.

These are not isolated bugs. They are architecture seams that are too shallow
for the next stage of provider expansion, NFO preservation, Addon Sidecar
protected writes, AI-assisted enrichment, remote storage, and playback clients.

## Target State

The target architecture is a modular monolith with deep workflow interfaces:

- A caller submits one domain commit request for a durable unit of work.
- The repository adapter owns SQL ordering, transactions, rollback, and stored
  representation details.
- Application services orchestrate workflows but do not learn partial-write
  ordering.
- Playback and transcode request identity is explicit, stable, and includes the
  profile facts that affect reuse.
- Runtime diagnostics expose trustworthy capability evidence without leaking
  local paths, secrets, or unsafe process details.
- Replaced shallow helpers are deleted rather than kept as parallel paths.

Closeout result: this target state is satisfied for the scoped slices. NFO
import and Library source indexing now have first-party durable commit units;
the touched application services use named workflow ports; playback/transcode
reuse identity is profile-shaped; admin playback diagnostics expose safe
hardware evidence and smoke-probe status; addon follow-ons are aligned to reuse
first-party commit boundaries; and the stale request-key/test anchors found by
the deletion sweep were removed.

## Scope

This lane covers the following vertical slices:

- NFO import atomic commit.
- Library scan source commit.
- Application-service workflow port narrowing for the touched areas.
- Playback/Transcode Profile identity.
- Hardware acceleration capability diagnostics.
- Cross-lane alignment with Addon Sidecar protected-write work.
- Deletion and closeout gates.

## Non-Goals

- Provider breadth for TMDB, Douban, Bangumi, or AI inference.
- Full Source Variant schema, optimized versions, or adaptive bitrate playback.
- Addon token/grant lifecycle, concrete artwork intake, or Library File Write
  implementation owned by existing addon lanes.
- Splitting Taru into multiple deployable services.
- Copying Jellyfin or Plex source, schema, migrations, tests, or comments.

## Architecture Direction

### NFO Import Commit

NFO import should stop exposing the order of Media Item update, Metadata Field
Lock write, hierarchy confirmation, and follow-on catalog/search refresh to the
caller. The replacement module should present one interface for applying an NFO
Sidecar import result and let the adapter own transaction and rollback details.

The first implementation should include rollback tests for failure after the
Media Item write and failure after lock writes. If a catalog/search refresh is
not part of the same transaction, the design must explicitly state the
eventual-consistency seam and the failure recovery path.

### Library Scan Source Commit

Indexing one discovered Media Source should commit the durable source-of-truth
state together: Media Item, Media Source, Source State, Library Item State,
Local Inference Evidence, Search Projection, and Scan failure resolution. The
current `record_scanned_media_source` direction is good but not deep enough
because important adjacent writes remain in the caller.

The target interface should hide write ordering from `taru-library` and make
failure tests prove that stale search or stale failure state cannot survive a
failed source commit.

### Application-Service Workflow Ports

Do not mechanically split every repository trait. A seam is worth extracting
when it hides real workflow complexity or lets tests exercise the important
behavior without booting SQLite. The first candidates are the areas touched by
NFO import and library indexing; metadata and playback should follow only where
the slice proves broad-store knowledge is still leaking.

### Playback And Transcode Profile Identity

The persisted request key for HLS cannot remain a single constant once client
capability, quality target, audio track, subtitle track, container, codec,
hardware policy, or remote-staging policy affects output reuse. This lane should
define stable profile identity before adding more playback breadth.

The work should preserve current behavior for existing single-variant playback
while replacing the request-key construction with a profile-shaped model.

### Hardware Capability Diagnostics

FFmpeg encoder listing is useful evidence, but it does not prove that VAAPI,
NVENC, or Quick Sync can run in the deployed environment. This lane should add
safe diagnostic vocabulary and smoke-probe hooks where practical, with CI tests
using fakes and local hardware smoke tests documented as operator checks.

### Addon Alignment

Addon token, grant, metadata write, artwork, subtitle, NFO, and Library File
Write behavior remain owned by the dedicated addon workstreams. This lane should
only align shared commit seams so Addon Sidecar writes use the same durable
modules as first-party workflows.

## Deletion Rules

- A task cannot close while both old and new write paths are production
  reachable unless the next task owns a documented deletion gate.
- Test-only helpers may remain if they make the public interface easier to test;
  production pass-through helpers should be removed.
- Compatibility shims require a named migration reason and an expiry task.

## ADR Impact

This lane should reference existing ADRs first. Open a new ADR only if a slice
changes public API shape, storage/schema behavior, resource/concurrency policy,
or addon trust semantics in a way not covered by the current records.
