# MVP Release Shape

Status: Closed
Last updated: 2026-06-01

## Why This Lane Exists

Nako has strong architecture lanes and many completed feature workstreams, but
the current planning shape is closer to a capability tree than a release cut.
Without a clear MVP boundary, future work can keep expanding playback,
metadata, addon, storage, client, and operations breadth without producing one
coherent first release.

## Relevant Authority

- `CONTEXT.md`
- `PRODUCT.md`
- `docs/ARCHITECTURE.md`
- `docs/ROADMAP.md`
- `docs/GOALS.md`
- `docs/architecture/LANES.md`
- `docs/architecture/PLAYBACK.md`
- `docs/architecture/STORAGE_VFS.md`
- `docs/architecture/LIBRARY_PIPELINE.md`
- `docs/architecture/STATE_ACCESS.md`
- `docs/architecture/CONTROL_PLANE.md`
- `docs/architecture/OPERATIONS_RELEASE.md`
- `docs/architecture/REALTIME_SYNC.md`
- ADR 0003, 0005, 0016, 0017, 0021, 0023, 0024, 0026, 0027, 0028, 0036,
  0038, 0045, 0052, and 0053.

## Problem

The product target is Jellyfin/Plex-class, self-hosted, extensible media
serving. The first release cannot include the full target. It needs a
deliberate release cut that chooses:

- which already-built capabilities are required for the first usable path;
- which active workstreams must finish before MVP closeout;
- which advanced capabilities are explicitly post-MVP;
- which validation gates prove the MVP is actually usable.

## Target State

When this workstream closes:

- Nako has one accepted MVP statement.
- P0/P1/P2 scope is recorded and linked to architecture lanes.
- Active workstreams are either aligned to the MVP or explicitly deferred.
- A gap matrix maps release blockers to existing or new workstreams.
- Release gates cover install, scan, metadata, playback, Admin diagnostics,
  addon sidecar foundation, network guidance, and redaction.
- Follow-on work is split without being allowed to block MVP.

## In Scope

- Product-level MVP definition.
- Release cut and non-goals.
- Source coverage audit across architecture maps and active workstreams.
- Lane routing for MVP blockers.
- Validation ladder for release convergence.
- Updates to roadmap, goals, lane map, and workstream links.

## Out Of Scope

- Implementing missing runtime, API, or UI behavior.
- Rewriting architecture decisions already accepted by ADRs.
- Changing public API contracts.
- Adding schema migrations.
- Changing `nako-official-addons`.
- Copying behavior, code, tests, schemas, assets, or generated files from
  Jellyfin or other reference repositories.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| MVP should be video-first, not video-only. | High | `CONTEXT.md`; ADR 0021; `PRODUCT.md` | Broader media domains would need a separate release decision. |
| Single-Admin Mode is acceptable for MVP. | High | `CONTEXT.md`; CSAPA evidence; access ADRs | If multi-user sharing is required, account UX becomes a blocker. |
| Browser/web playback can be the MVP client path. | Medium | Media Web and browser playback workstreams | If browser playback is insufficient, desktop/native playback moves into P0. |
| Addon sidecars should prove the extension boundary, not lifecycle management. | High | ADR 0003, 0020, 0053; addon workstreams | If users require managed sidecars, Addon Manager becomes a post-MVP campaign. |
| Remote access starts as endpoint config and operator cookbook. | High | `CONTEXT.md`; ADR 0053; operations docs | Built-in tunnel would require a new product/ADR decision. |

## Architecture Direction

The MVP is a release convergence overlay above existing lanes. It does not
replace playback, storage, control-plane, client, or operations ownership.

The `mvp-release-convergence` lane owns release-scope documents and routing
decisions. Implementation remains in the existing capability lanes:

- playback/transcode owns playback and FFmpeg behavior;
- storage/VFS owns source identity and storage health;
- library/metadata/control-plane owns metadata authority and generated
  artifact apply workflows;
- web-product and client-surfaces own user-facing surfaces and desktop/mobile
  splits;
- operations-release owns install, release gates, and deployment guidance;
- addons-automation owns sidecar contracts and official addon coordination.

## Closeout Condition

This lane can close when:

- `MVP.md`, `RELEASE_CUT.md`, and `GAP_MATRIX.md` have been reviewed against
  current code/docs/workstreams;
- MVP blockers are routed to active or newly opened workstreams;
- validation gates are documented;
- docs links are updated;
- remaining non-MVP work is explicitly deferred.

## Closeout Summary

Closed on 2026-06-01 after `MRS-050` integrated the two P0 campaign slices and
ran the release-candidate validation ladder. Gate 0 through Gate 5 and Gate 7
pass on `main`; Gate 6 is skipped by MVP scope because this candidate does not
claim an official Addon Sidecar proof. Remaining release execution, actual
artifact publication, one-command gate wrapping, and official addon alpha smoke
should open focused follow-on workstreams when they become required.
