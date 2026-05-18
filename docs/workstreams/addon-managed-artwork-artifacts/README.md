# Addon Managed Artwork Artifacts

Status: Active
Last updated: 2026-05-19

This workstream owns the `artwork_write` follow-on split from
`addon-protected-writes`. APW proved Addon Side Effect intake plus apply outcome
with Canonical Metadata. This lane decides how addon-submitted artwork enters
Taru as Artwork Candidates, Managed Artwork, or Taru-Managed Artifacts without
hotlinking unsafe provider URLs or exposing library paths.

Current state after AMAA-030: the first implementation slice is an Addon
Artwork Candidate proposal, not immediate Managed Artwork selection and not
direct public `ImageAsset` insertion. Taru captures addon artwork intent in a
first-party candidate boundary so raw addon URLs, paths, payloads, and future
cache internals do not become public client artwork.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../addon-protected-writes/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Define the first Taru-owned `artwork_write` apply path.
- Represent addon artwork as Artwork Candidates, Managed Artwork, or
  Taru-Managed Artifacts instead of raw provider hotlinks.
- Preserve Addon Token, accepted permission, Media Library grant, idempotency,
  audit, and redaction behavior from APW.
- Define external fetch ownership, artifact storage, cache/thumbnail policy,
  resource budgets, and safe diagnostics before implementation.

## Non-Goals

- No Addon Manager install or process lifecycle work.
- No direct Addon access to Source Locators, filesystem paths, remote storage
  handles, or database credentials.
- No Public Client write API expansion.
- No image-processing pipeline rewrite unless the audit proves the current
  artwork/task seams are insufficient.
