# Addon Protected Writes

Status: Completed
Last updated: 2026-05-18

This workstream owned the concrete write-application follow-on split from
`addon-token-grants-side-effects`. The previous lane proved Addon Token,
accepted grant, addon-principal, and Addon Side Effect intake boundaries. This
lane proved how accepted side effects become Nako-owned Canonical Metadata
changes, then split Managed Artwork, subtitle, NFO, and Library File Write
breadth into narrower follow-ons.

Authoritative docs:

- `DESIGN.md`
- `MILESTONES.md`
- `TODO.md`
- `EVIDENCE_AND_GATES.md`
- `WORKSTREAM.json`
- `HANDOFF.md`
- `../addon-token-grants-side-effects/`
- `../../adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`

## Goals

- Turn accepted Addon Side Effect intake records into concrete Nako-mediated
  protected writes.
- Keep Addon Sidecars behind Addon Tokens, accepted Addon Permissions, and
  Library-Scoped Addon Grants.
- Route metadata writes through Nako's Canonical Metadata authority instead of
  direct provider or database mutation.
- Route artwork, subtitle, NFO, and other library sidecar outputs through
  Managed Artwork, NFO Round Trip, storage/VFS, and Library File Write
  policies.
- Preserve idempotency, audit, safe response redaction, and catalog/search
  consistency as effect-specific handlers are added.

## Closeout

APW is complete after the bounded `metadata_write` apply slice. The shipped
model separates intake validation from apply outcome, records safe apply
summaries, persists first-class Addon metadata attribution, applies Canonical
Metadata through merge policy, and refreshes catalog/search without leaking raw
payloads, Source Locators, filesystem paths, provider bodies, token hashes, or
raw Addon Tokens.

Follow-ons:

- `../addon-managed-artwork-artifacts/` for `artwork_write`, Artwork
  Candidates, Managed Artwork, and Nako-Managed Artifact storage.
- `../addon-library-file-write-policy/` for subtitle, NFO, and Library File
  Write behavior.

## Non-Goals

- No new Addon Token lifecycle or grant model unless APW-020 proves the ATGSE
  contract is insufficient.
- No OAuth-first authorization, Addon Manager lifecycle automation, Native
  Plugin ABI, or Jellyfin Plugin Compatibility.
- No direct Addon access to database credentials, admin bearer tokens, raw
  Source Locators, filesystem paths, or remote storage handles.
- No field-level permission system in the first protected-write slice.
- No Public Client API expansion; protected writes remain Addon/Admin/internal
  surfaces until a separate client contract says otherwise.
