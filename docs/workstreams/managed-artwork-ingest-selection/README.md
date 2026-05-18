# Managed Artwork Ingest Selection

Status: Active
Last updated: 2026-05-19

This workstream owns the follow-on after
`addon-managed-artwork-artifacts`. AMAA shipped the safe Addon Artwork
Candidate proposal boundary. This lane decides how a candidate becomes
Taru-managed artwork: fetched by Taru, validated as an image, stored under a
Taru-owned cache/artifact URI, optionally thumbnailed, selected, and finally
published as safe Public Client artwork.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../addon-managed-artwork-artifacts/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Define the first Taru-owned candidate acceptance path.
- Fetch remote artwork through bounded Taru-owned runtime policy, not through
  Addon handler hotlinks.
- Validate image content, size, type, dimensions, and failure diagnostics
  before public publication.
- Store managed artwork under Taru-owned cache/artifact URIs.
- Publish selected public artwork without exposing raw candidate source URLs or
  future cache internals.

## Non-Goals

- No direct Addon filesystem, database, or storage access.
- No Addon Manager install/update lifecycle.
- No arbitrary image-generation pipeline.
- No artwork sidecar file export; that remains Library File Write scope.
- No public write API until admin/review workflow semantics are explicit.
