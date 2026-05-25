# Metadata Profile Configuration Authority - Design

Status: Completed
Last updated: 2026-05-25

## Problem

`MetadataProfile` is now editable through Admin API, but configured Media
Libraries are reconciled from TOML during startup. Today reconciliation builds a
complete desired `Library` from config and upserts it, which can silently erase
a profile saved through Admin API.

That is a product correctness problem: an operator can save a Metadata Profile,
see the next scan use it, then lose the setting after restart if the library is
also listed in config.

## Relevant Authority

- `CONTEXT.md`
- `docs/adr/0010-library-presets-are-configuration-templates.md`
- `docs/workstreams/multi-library-hardening`
- `docs/workstreams/admin-library-metadata-profile-configuration`
- `crates/nako-server/src/app/library_reconciliation.rs`
- `crates/nako-server/src/config.rs`
- `crates/nako-core/src/media/library.rs`

## Target State

Nako distinguishes three Metadata Profile sources:

- `preset`: generated from the library preset template.
- `configured`: explicitly supplied through `metadata.library_profiles` in TOML.
- `admin`: persisted through Admin API.

Startup reconciliation keeps config authoritative for library identity, name,
root, backend, preset, domain, scan defaults, and naming defaults, but it does
not erase an `admin` Metadata Profile when TOML does not explicitly provide
`metadata.library_profiles.<library_id>`.

When TOML explicitly provides a profile for a library, that profile is
authoritative for startup reconciliation and should replace the persisted
profile source with `configured`.

## In Scope

- Add source tracking to `LibraryOptions` with backward-compatible serde
  defaults.
- Mark TOML profile overrides as `configured`.
- Mark Admin API profile updates as `admin`.
- Update configured-library reconciliation to merge persisted Admin profiles
  when no TOML profile override exists.
- Add focused startup/Admin tests proving restart persistence and explicit TOML
  override behavior.
- Update workstream evidence and handoff.

## Out Of Scope

- Writing updated profiles to TOML.
- New database columns or migrations.
- Public Client DTO changes.
- Admin Web controls or warning copy.
- Addon capability-aware controls.
- Full NAS root scan.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `LibraryOptions` is persisted as JSON, so adding a serde-defaulted field is migration-free. | High | SQLite/PostgreSQL persist `options_json`. | A schema or codec migration would be required. |
| Public Client DTOs should not expose internal source tracking. | High | Public DTO mapping is explicit. | Public protocol and generated SDKs would need versioned shape review. |
| Config should still own library identity/root/preset at startup. | High | `multi-library-hardening` states config is desired-state input. | Reconciliation would need a broader source-of-truth redesign. |
| Explicit TOML profile override should remain authoritative. | High | `library-metadata-scan-policy` introduced `metadata.library_profiles`. | Operators could not enforce config-managed profile policy. |

## Architecture Direction

Keep the first slice inside the existing Library boundary:

- `nako-core::LibraryOptions` owns the source marker because it travels with the
  persisted options JSON.
- `nako-server::config` constructs configured libraries with `preset` or
  `configured` profile source.
- `nako-server::app::LibraryAppService` marks Admin updates as `admin`.
- `nako-server::app::ConfiguredLibraryReconciliationService` merges persisted
  `admin` profile state into the desired configured library only when the
  desired profile source is `preset`.

This avoids a new table while making the authority decision deterministic and
test-visible.

## Closeout Condition

This lane can close when:

- Admin-updated Metadata Profile survives app restart for a configured library
  without explicit TOML profile override;
- explicit TOML `metadata.library_profiles` still overrides persisted Admin
  profile on restart;
- Public Client DTO shape remains unchanged;
- focused nextest gates, formatting, and whitespace checks pass; and
- Admin Web V2 and field-specific patch UX are left as follow-ons.

Closeout result: met on 2026-05-25. Admin-owned profiles now survive restart
when TOML only supplies preset defaults, while explicit TOML profile overrides
remain authoritative.
