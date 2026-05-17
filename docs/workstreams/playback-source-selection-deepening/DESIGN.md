# Playback Source Selection Deepening Design

Status: Completed
Last updated: 2026-05-17

## Problem

`taru-streaming` currently chooses playback mode from a narrow set of facts:
file-name container, probed audio/video codecs, and a small client capability
shape. `taru-server` still carries much of the playback orchestration around
source lookup, storage access, remux/HLS execution, and session handling.

That is acceptable for the MVP, but it is too shallow for Taru's next client
phase. Once **Client Applications** need subtitles, audio-track choice, HDR
handling, bandwidth hints, remote access endpoints, **Library Access**, and
future **Source Variant** selection, the decision logic would spread across
HTTP handlers, server app code, and transcode runtime code.

The deeper Module should be **Playback Source Selection**: callers ask for a
playback decision for a user/client/source context, and the implementation owns
the selection reasoning and plan intent.

## Current Findings

- M42's `CatalogHydrationPort` lookup concern is resolved at the public
  Interface: lookup/snapshot/commit types are now private to `taru-catalog`.
- `MetadataStrategyExecutor` still deserves a later provider-attempt runtime
  extraction, but it is less directly blocking for native client work.
- `taru-api` still mixes **Public Client API**, **Admin API**, diagnostics, and
  extension DTOs in one root file; that is a strong follow-on after playback
  contracts stop moving.
- NFO Round Trip and typed VFS storage errors remain valid follow-ons, but they
  do not need to block this playback seam.

## Target State

- `taru-streaming` owns a deeper **Playback Source Selection** Interface with a
  request shape that can carry client, source, probe, stream, storage, and
  future profile facts.
- The decision output separates source selection from execution intent:
  direct-play, remux, or **Playback Transcode** plan.
- The first slice preserves existing public wire compatibility where possible.
- `taru-server` becomes thinner: load facts, enforce available access checks,
  ask `taru-streaming` for a decision, then execute the returned plan.
- The model has explicit extension points for subtitles, audio tracks, HDR,
  bitrate, remote endpoints, **Source Variants**, and **Transcode Profiles**
  without implementing all of them in M43.

## In Scope

- `crates/taru-streaming/src/selection.rs` request/decision model.
- `crates/taru-server/src/app/playback/*` call sites and orchestration cleanup
  needed to use the deeper decision model.
- `crates/taru-api` DTO mapping only where compatibility mapping is needed for
  existing playback responses.
- Focused unit and route tests proving current behavior is preserved through
  the deeper seam.
- Workstream and goal documentation.

## Out Of Scope

- No Android, Flutter, Web, or player implementation.
- No full **Source Variant** schema or UI.
- No adaptive bitrate ladder.
- No durable **Optimized Version** workflow.
- No full **Transcode Profile** policy engine.
- No new Public Client API route unless the refactor proves it is required.
- No NFO Round Trip preservation work.
- No typed VFS error classification work.
- No metadata provider breadth or provider-attempt runtime extraction.

## Architecture Direction

The decision should move from this shape:

```text
decide_playback(source, probe, client_capabilities) -> PlaybackDecision
```

to a deeper workflow-shaped Interface:

```text
select playback source(request) -> PlaybackSelectionDecision
```

The request can include current fields and reserved future facts:

- source identity and locator facts;
- media probe and stream facts;
- client playback capabilities;
- storage capabilities and remote/local hints;
- optional access context;
- optional requested audio, subtitle, and quality preferences.

The output should make the execution boundary obvious:

- selected source or source-variant identity;
- selected playback mode;
- direct-play response intent;
- remux intent;
- transcode intent;
- decision diagnostics safe for client display and server logs.

`taru-transcode` should remain the FFmpeg/runtime Module. `taru-streaming`
should decide what needs to happen; it should not run FFmpeg or own session
storage.

## Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing public playback DTOs can remain compatible for this slice. | Medium | Current `ClientPlaybackDecision` already has mode, reason, direct play, and transcode plan fields. | If not, add an explicit API contract task and OpenAPI/SDK update gate. |
| `taru-server` can load enough facts before selection without moving storage registries into `taru-streaming`. | High | Current playback app already has source, probe, storage backend, and config access. | Add a small server-side fact adapter instead of widening streaming crate dependencies. |
| This should precede concrete Android/native playback work. | High | `CONTEXT.md` says **Client Applications** should consume stable **Public Client API** contracts, and playback decisions must respect future capability differences. | If Android planning is purely document-only, M43 can still proceed independently. |

## Closeout Condition

This lane can close when:

- playback selection has a workflow-shaped request and decision model;
- `taru-server` playback app uses that model instead of encoding mode-choice
  rules around HTTP/runtime orchestration;
- current playback behavior remains covered by focused tests;
- compatibility mapping for public playback DTOs is explicit;
- docs record remaining follow-ons for metadata provider runtime, API module
  split, NFO Round Trip, typed VFS errors, and deeper client profiles.

## Closeout Result

M43 is closed. `taru-streaming` now owns a workflow-shaped
`select_playback_source` Interface and returns selected-source facts plus
direct-play, remux, or transcode execution intent. `taru-server` loads source,
probe, client, storage, remux-output, and HLS intent facts, then executes the
returned decision instead of duplicating mode-choice rules. Public playback DTOs
retain the existing wire shape, with a regression test proving internal
selection fields do not enter `taru-client-protocol`.
