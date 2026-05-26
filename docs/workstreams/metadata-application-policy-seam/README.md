# Metadata Application Policy Seam

Status: Completed
Last updated: 2026-05-26

This workstream deepens Nako's host-owned Canonical Metadata application path.
Addon Sidecars, providers, NFO import, and hierarchy confirmation can all
produce Canonical Metadata-shaped facts, but Nako must own the final
application: field locks, merge mode, local authority, catalog projection,
persistence, and apply reporting.

The first execution slice is backend-only and host-only. Official Addon
follow-up adapter cleanup and scan Addon bulk continuation are explicit
follow-ons.

Closeout: this lane is complete. Nako now routes Addon metadata writeback
through a server-side `MetadataApplication` Module that owns field-lock lookup,
source-aware merge policy, catalog projection, and a safe apply report. Addon
`metadata_write` remains responsible for protocol validation, patch mapping,
target resolution, and delegation.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`
