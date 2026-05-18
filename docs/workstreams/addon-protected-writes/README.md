# Addon Protected Writes

Status: Active
Last updated: 2026-05-18

This workstream owns the concrete write-application follow-on split from
`addon-token-grants-side-effects`. The previous lane proved Addon Token,
accepted grant, addon-principal, and Addon Side Effect intake boundaries. This
lane decides how accepted side effects become Taru-owned Canonical Metadata,
Managed Artwork, subtitle, NFO, and Library File Write changes.

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

- Turn accepted Addon Side Effect intake records into concrete Taru-mediated
  protected writes.
- Keep Addon Sidecars behind Addon Tokens, accepted Addon Permissions, and
  Library-Scoped Addon Grants.
- Route metadata writes through Taru's Canonical Metadata authority instead of
  direct provider or database mutation.
- Route artwork, subtitle, NFO, and other library sidecar outputs through
  Managed Artwork, NFO Round Trip, storage/VFS, and Library File Write
  policies.
- Preserve idempotency, audit, safe response redaction, and catalog/search
  consistency as effect-specific handlers are added.

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
