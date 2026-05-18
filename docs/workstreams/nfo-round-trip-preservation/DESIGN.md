# NFO Round Trip Preservation Design

Status: Completed
Last updated: 2026-05-17

## Problem

NFO is Taru's local metadata boundary. ADR 0008 intentionally started with a
minimal movie codec, and ADR 0007 says local/user authority must not be silently
overwritten by provider refresh. Export now violates the spirit of that boundary
when `force` is used against an existing NFO: `MovieNfoCodec::render` generates
a fresh `<movie>` document from only Taru-known fields.

That behavior is acceptable for creating a missing sidecar, but it is unsafe for
updating an existing file. Existing NFO files often contain extra tags, comments,
tool-specific fields, ratings, IDs, artwork variants, or hand-written notes.
Before Taru adds stronger library file write policy and link management, it
needs a preservation model that makes NFO updates explainable and bounded.

## Target State

- The codec exposes a preservation-aware update path for existing NFO XML.
- Taru-owned movie fields are rendered from `CanonicalMetadata`.
- Unknown XML fields survive an update instead of being dropped.
- Duplicate or conflicting Taru-owned fields are surfaced in a report.
- Export workflow reads an existing sidecar during forced update and uses the
  preservation-aware path instead of whole-document rewrite.
- Creating a new sidecar still uses deterministic Taru-owned rendering.
- Import behavior remains compatible with the current minimal movie codec.

## In Scope

- `crates/taru-nfo/src/codec.rs`
- `crates/taru-nfo/src/export.rs`
- focused `taru-nfo` tests around preservation, conflict reporting, and export
  over an existing sidecar
- workstream and goal documentation

## Out Of Scope

- No full Jellyfin, Kodi, Plex, or Emby NFO compatibility matrix.
- No public HTTP API, OpenAPI, SDK, or protocol changes.
- No database schema or repository trait changes.
- No provider breadth, metadata merge-policy redesign, or catalog graph change.
- No VFS soft-link, hard-link, atomic-write, backup, or storage policy work.
- No broad episode/series export expansion beyond preserving existing XML when
  the movie exporter is used.

## Architecture Direction

Add a small preservation model inside `taru-nfo` rather than making XML a core
domain type:

```text
NfoPreservationReport
  preserved_unknown_fields
  updated_owned_fields
  conflicts

NfoFieldConflict
  field
  existing_value
  replacement_value
  reason
```

The first implementation should be pragmatic and intentionally limited:

- Treat top-level movie child elements known to Taru as Taru-owned fields.
- Update Taru-owned fields from `NfoDocument`.
- Preserve unknown top-level child elements and comments where the XML parser
  makes that practical.
- Normalize output formatting after update; byte-for-byte formatting
  preservation is not a goal for this slice.
- Report duplicate Taru-owned top-level fields as conflicts before replacing
  them with Taru's canonical value.

The important boundary is semantic preservation, not exact textual preservation.
Future work can add richer XML span preservation or compatibility profiles if
real library samples require it.

## Taru-Owned Movie Fields

The first preservation set follows the current minimal codec:

- `title`
- `originaltitle`
- `sorttitle`
- `plot`
- `releasedate`
- `runtime`
- `tagline`
- `genre`
- `tag`
- `actor`
- `director`
- `writer`
- `poster`
- `fanart`
- `thumb`

Known aliases accepted during parse, such as `aired`, `premiered`, and `year`,
should be considered potential conflicts during update when they coexist with
`releasedate` semantics.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing NFO preservation matters most when forced export updates an existing sidecar. | High | Current export skips existing sidecars unless `force` is true. | If users need non-force merge behavior, add a separate write policy goal. |
| Preserving unknown top-level fields is enough for the first safe model. | Medium | Current codec only owns top-level movie fields and nested `actor`/`fanart`. | If nested unknown preservation becomes necessary, extend the preservation model with XML fragments before link policy work. |
| Exact whitespace and attribute ordering are not required in M47. | Medium | The existing renderer already normalizes XML. | If users depend on byte-for-byte stability, introduce an XML patch/span model as a follow-on. |
| Conflict reporting can stay internal/test-visible for this slice. | High | `NfoExportSummary` currently exposes only item-level failures. | If UI needs conflict diagnostics, add public DTOs in a later API goal. |

## Closeout Condition

This lane can close when:

- forced export over an existing movie NFO preserves unknown XML elements;
- Taru-owned fields are updated deterministically from current metadata;
- duplicate/conflicting owned fields are reported in codec tests;
- export workflow uses preservation-aware update for existing sidecars;
- current import and new-sidecar export tests still pass;
- focused and workspace validation gates pass.

## Completion Notes

M47 is complete. `MovieNfoCodec` now exposes a preservation-aware render path
that keeps unknown top-level movie XML elements, comments, and processing
instructions by using parser source ranges. Taru-owned movie fields are
re-rendered from `NfoDocument`; duplicate owned fields and release-date aliases
are reported through `NfoPreservationReport`.

Forced export over an existing sidecar reads the existing XML and uses the
preservation path. Missing sidecar creation still uses deterministic fresh
rendering. Conflict diagnostics remain internal/test-visible in this slice.
