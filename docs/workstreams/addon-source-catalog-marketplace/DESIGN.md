# Addon Source Catalog And Marketplace

Status: Completed
Last updated: 2026-05-24

## Why This Lane Exists

The Addon Manager lifecycle lane is complete. Nako can now expose a
manager-owned registry/plan slot with explicit operator-confirmed lifecycle
intent, but it still lacks a clear source catalog and marketplace discovery
boundary for how addon sources are listed, resolved, and browsed before an
operator chooses a plan.

## Problem

Operators needed a discoverable place to find addon sources and resolve
installable addon descriptors. The manager plan slot can confirm intent, but it
did not define how addon sources become catalog entries or marketplace
metadata.

## Target State

This lane closed after Nako became able to:

- list the built-in official addon source through a redaction-safe catalog
  surface;
- browse the official metadata scraper as an installable catalog entry;
- resolve an `AddonInstallDescriptor` and protocol install guide from that
  entry;
- expose marketplace-style discovery metadata without taking over package
  signing, update execution, provider breadth, or process supervision;
- keep provider breadth, trust roots, and execution ownership as separate
  follow-ons;
- preserve the Addon Protocol, Addon Task runtime, and manager-plan contracts
  while discovery grows.

## Scope

- Addon source catalog modeling.
- Marketplace-style discovery and listing metadata.
- Source resolution for installable addon descriptors.
- Minimal Admin/API surfaces needed to browse or select source entries.
- Documentation and validation for the first discovery slice.

## Non-Goals

- Package signing trust roots.
- Direct container or process supervision.
- Broad provider breadth beyond the first source set.
- Downloading packages, executing update/rollback policy, or starting sidecars.
- Authenticated outbound task dispatch credential storage.
- Official-addon task-path smoke coverage.
- Native Plugin ABI or in-process addon execution.
- Changes to the operator-confirmed manager-plan intent contract.

## Architecture Direction

Treat the catalog as a discovery and resolution control plane, not as a
distribution authority. The catalog should own:

- source listing and metadata lookup;
- descriptor resolution and candidate selection;
- redaction-safe browse and install entrypoints;
- stable handoff into the existing manager-plan surface.

The catalog should not smuggle in signing policy, sidecar lifecycle ownership,
or hidden provider execution. Marketplace behavior is the discovery surface,
not a new runtime authority.

## Implemented First Slice

The first catalog slice is intentionally read-only and built in:

- `GET /admin/v1/addons/catalog/sources` lists `nako-official` as the built-in
  official source.
- `GET /admin/v1/addons/catalog/entries` lists the official metadata scraper
  catalog entry.
- `GET /admin/v1/addons/catalog/entries/{entry_id}/resolve` resolves an
  `AddonInstallDescriptor` plus protocol install guide for the selected entry.

The resolved entry is an install candidate, not an Addon registration. It does
not create `AddonRegistrationRecord`, Addon Routing Plans, Addon Task jobs,
manager lifecycle intent, package downloads, signing decisions, or sidecar
processes. Operators still register, enable, grant, and manage the sidecar
through the existing Admin Addon and manager-plan surfaces.

The built-in source deliberately reports:

- `provides_package_signing = false`;
- `provides_process_supervision = false`;
- `provides_provider_breadth = false`;
- `package_signing_verified = false`.

That makes the boundary explicit in the public Admin shape instead of relying
on prose alone.

## Reference Patterns

- Jellyfin uses multiple official and third-party plugin repositories, with a
  catalog that can browse installed repositories and install from repository
  manifests. That suggests Nako should support more than one source and keep
  repository metadata separate from per-addon entries.
- Home Assistant lets users add a repository URL to the store, and each
  repository contains one or more add-ons with a root repository manifest. That
  suggests Nako should treat source identity as first-class and keep source-level
  metadata distinct from addon manifests.
- Visual Studio Code extensions use a manifest with `publisher`, `name`,
  `version`, and `engines.vscode`, plus a pre-release channel. That suggests
  Nako should keep addon version separate from host compatibility and channel
  policy.
- Obsidian plugins use `version` and `minAppVersion`, with `versions.json`
  handling host-version-specific selection. That suggests Nako should model
  host compatibility explicitly rather than inferring it from addon version
  alone.

Nako's implication: source catalog entries should carry repo-level metadata,
per-addon descriptor metadata, and explicit compatibility/channel policy while
remaining separate from package signing or process ownership.

## Closeout

This lane completes the Addon core runtime and architecture mainline. The
remaining addon work is product and ecosystem breadth:

- package signing and trust-root policy;
- provider breadth beyond the first official companion addon;
- rollback/update execution beyond the current manager-plan intent surface;
- authenticated outbound task dispatch credential storage for `Bearer` and
  `SharedSecret` sidecars;
- official-addon task-path smoke coverage once an official addon declares a
  task;
- direct process/container supervision, if Nako decides to own sidecar
  execution.

## Related Docs

- `docs/workstreams/addon-manager-lifecycle-automation/`
- `docs/workstreams/official-addon-e2e-alpha2/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
