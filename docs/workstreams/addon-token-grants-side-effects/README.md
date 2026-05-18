# Addon Token Grants Side Effects

## Status

Completed.

This workstream owns the follow-up needed before Addons can perform protected
writes through Taru. It turns the Post-M5 Addon Token, Library-Scoped Addon
Grant, and Addon Side Effect concerns into an executable architecture lane.

This lane closed on 2026-05-18 after the Addon Token lifecycle, accepted grant
model, addon-principal runtime checks, and first Addon Side Effect intake proof
were implemented and verified. Concrete metadata, artwork, subtitle, NFO, and
Library File Write application behavior is split to
`../addon-protected-writes/`.

Top-level tracking:

- [Addon token/grant/side-effect design](DESIGN.md)
- [Task ledger](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)
- [ADR 0015: capability-scoped HTTP addons and automation providers](../../adr/0015-capability-scoped-http-addons-and-automation-providers.md)
- [ADR 0020: Jellyfin-like sidecar addons with scoped Taru API access](../../adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md)

## Goals

- Define Addon Token issuance, revocation, and Addon Token Rotation.
- Model accepted Addon Permissions and Library-Scoped Addon Grants separately
  from manifest-requested scopes.
- Introduce a Taru-owned Addon Side Effect intake boundary for metadata,
  artwork, subtitle, and Library File Write behavior.
- Keep Addon Sidecars powerful enough for Jellyfin-like workflows without
  granting admin tokens, database access, or raw filesystem authority.
- Record audit, idempotency, and resource-boundary expectations before enabling
  addon-initiated protected writes.

## Non-Goals

- No OAuth-first authorization flow.
- No in-process Native Plugin ABI.
- No Jellyfin Plugin Compatibility.
- No Addon Manager lifecycle work such as discovery, download, install,
  process supervision, or marketplace behavior.
- No direct Addon access to database credentials, admin tokens, raw Source
  Locators, or library filesystem paths.
- No implementation of concrete metadata/artwork/subtitle write APIs in this
  lane; those apply handlers now belong to `../addon-protected-writes/`.
