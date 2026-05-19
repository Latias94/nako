# Core Architecture Deepening

Status: Completed
Last updated: 2026-05-18

This workstream owns the architecture-first refactor that follows the
2026-05-18 Taru architecture review. The goal is not a minimal patch set. A
slice is complete only when the target seam is deep enough, the old shallow path
is removed, and tests prove the new invariants.

Closeout: CAD-010 through CAD-090 are complete. Workspace check, workspace
nextest, formatting, and diff gates passed on 2026-05-18.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`

## Goals

- Move multi-record Media Item, NFO Sidecar, Catalog Item Graph, Search
  Projection, Source State, Local Inference Evidence, and failure-resolution
  writes behind explicit commit interfaces.
- Make `taru-server` application services depend on focused workflow ports
  where broad `SqliteStore` knowledge currently leaks across seams.
- Replace weak playback and transcode request identity with stable profile
  identity before more Source Variant, audio, subtitle, hardware, or quality
  decisions depend on it.
- Deepen hardware acceleration diagnostics from encoder-list availability
  toward runtime capability evidence that operators can trust.
- Align with existing Addon Sidecar protected-write workstreams without
  duplicating addon token, grant, artwork, or Library File Write scope.
- Delete obsolete helpers, compatibility shortcuts, duplicated write paths, and
  shallow adapters after replacement invariants are covered.

## Non-Goals

- No new metadata provider breadth for TMDB, Douban, Bangumi, or future AI
  providers unless the slice is specifically about the commit seam they use.
- No in-process plugin ABI or Jellyfin plugin compatibility.
- No adaptive bitrate ladder, optimized-version workflow, or client UI work.
- No new storage backend or network traversal feature implementation.
- No broad schema redesign unless a focused slice proves the current schema
  cannot support the correct commit interface.

## Refactor Policy

Prefer the clean target architecture over preserving MVP shortcuts. Temporary
duplication is allowed only inside an active task and must be removed before the
task closes unless `TODO.md` records the next owner and deletion gate.

Do not keep old and new orchestration paths alive for convenience. The deletion
of replaced code is part of the implementation definition of done.

## Related Workstreams

- `metadata-catalog-commit-atomicity`: completed graph/search commit baseline.
- `metadata-merge-policy-unification`: completed Canonical Metadata merge
  authority unification.
- `repository-seam-deepening`: completed catalog hydration workflow-port
  extraction.
- `playback-source-selection-deepening`: completed first playback selection
  model; this lane deepens request/profile identity.
- `transcode-runtime`: completed playback/transcode runtime foundation; this
  lane deepens capability evidence and profile-driven reuse.
- `addon-protected-writes`, `addon-managed-artwork-artifacts`, and
  `addon-library-file-write-policy`: own concrete Addon Sidecar write behavior.
