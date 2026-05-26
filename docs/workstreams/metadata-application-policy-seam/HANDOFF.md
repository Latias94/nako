# Metadata Application Policy Seam - Handoff

Status: Completed
Last updated: 2026-05-26

## Current State

The workstream is complete. Nako now has a host-owned `MetadataApplication`
Module in `nako-server`, and Addon `metadata_write` is a thin Adapter over that
Module.

## Active Task

- Task ID: none
- Status: closed

## Decisions Since Last Update

- First `MetadataApplication` lives in server app because it needs repository
  access and catalog projection.
- Pure merge authority remains in `nako-core::MetadataMergePolicy`.
- Addon Sidecars remain fact submitters; they do not choose host merge policy.
- Official Addon adapter cleanup and scan Addon bulk continuation are follow-ons.
- Provider refresh and hierarchy confirmation stay in `nako-metadata` for now;
  only pure application-decision types should move to `nako-core` if cross-crate
  reuse becomes necessary.

## Shipped Behavior

- Addon writeback honors library-profile `MetadataRefreshMode::MissingOnly`.
- User-locked fields are protected from Addon writes.
- Same-source Addon locks allow that Addon to refresh its own fields.
- Addon-sourced metadata writes still update catalog graph/search projection.
- Scan-triggered Addon writeback uses host policy via the Side Effect runtime.
- Apply reports are structured and do not echo the raw metadata payload.

## Blockers

- None.

## Follow-Ons

- Open the official Addon follow-up adapter cleanup lane in
  `F:\SourceCodes\Rust\nako-official-addons` if sidecar writeback code still
  contains host-policy-looking decisions.
- Open scan Addon bulk continuation only after the host policy seam remains
  stable; persist `next_cursor` / resume state through task scheduling instead
  of adding an in-scan while loop.
- Consider a pure `nako-core` metadata application decision type only if
  provider refresh, hierarchy confirmation, NFO, and Addon paths need a common
  command/result without coupling `nako-metadata` to `nako-server`.
