# Managed Artwork Ingest Selection

Status: Completed
Last updated: 2026-05-19

This workstream owned the follow-on after
`addon-managed-artwork-artifacts`. AMAA shipped the safe Addon Artwork
Candidate proposal boundary. This lane selected and shipped the first
candidate acceptance boundary: an Admin API command queues Taru-owned managed
artwork ingest state without fetching remote bytes or publishing public
artwork.

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

- Define and ship the first Taru-owned candidate acceptance path.
- Keep remote artwork fetches out of Addon Side Effect handling.
- Record a durable managed artwork ingest row and job with redacted input.
- Prevent candidate source URLs, cache URIs, paths, and unvalidated hotlinks
  from becoming Public Client artwork.
- Split remote fetch, content validation, artifact bytes, image serving,
  thumbnailing, and selected artwork publication into narrower follow-ons.

## Non-Goals

- No direct Addon filesystem, database, or storage access.
- No Addon Manager install/update lifecycle.
- No arbitrary image-generation pipeline.
- No artwork sidecar file export; that remains Library File Write scope.
- No public write API until admin/review workflow semantics are explicit.
