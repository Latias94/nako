# Nako Rebrand Baseline

Status: Active
Last updated: 2026-05-22

## Why This Lane Exists

The product is moving from its previous working name to Nako before public
launch, open-source release, or external user adoption. Because there are no
compatibility constraints, this lane should perform a clean rename rather than
preserve legacy aliases that would become permanent architecture debt.

## Relevant Authority

- Existing docs:
  - `CONTEXT.md`
  - `README.md`
  - `assets/brand/README.md`
  - `docs/workstreams/nako-brand-identity/README.md`
- Related workstreams:
  - `docs/workstreams/nako-brand-identity/`
  - `docs/workstreams/addon-architecture-deepening/`
  - `docs/workstreams/admin-web-addon-credential-grant-onboarding/`
  - `docs/workstreams/android-client-foundation/`

## Problem

The repository still exposes the previous working name across product copy,
crate names, Rust module imports, CLI binaries, deployment files, SDKs, Android
package names, Admin Web assets, addon protocol names, and workstream docs. If
left in place, that name will continue to leak into generated contracts,
install instructions, official addon dependencies, and client package surfaces.

## Target State

Nako is the only active product name and code namespace in the repository:

- user-facing copy says Nako,
- crate packages and Rust imports use `nako-*` / `nako_*`,
- CLI, Docker, compose, systemd, and config examples use Nako names,
- Admin Web and Android visible branding use the Nako icon and name,
- SDK packages and generated contract examples use Nako naming,
- addon protocol/client/reference crates use Nako naming,
- old-name grep gates show no unintended residue outside explicit historical
  notes or third-party references approved by this lane.

## In Scope

- Rename tracked source directories and files that contain the old product
  name.
- Rename Rust crate packages and update all intra-workspace dependencies.
- Rename Rust identifiers, imports, binaries, and generated SDK identifiers
  where they use the old product name.
- Rename Android package namespaces to `dev.nako`.
- Rename Admin Web public assets and visible brand copy.
- Rename deploy, container, compose, config, scripts, and documentation
  examples.
- Update official addon repository path dependencies after the protocol crate
  rename.
- Update workstream docs so current documentation speaks in Nako terms.

## Out Of Scope

- Rewriting Git history.
- Preserving compatibility shims, aliases, deprecated binary names, or old
  environment variable names.
- Renaming remote repositories before the source tree is internally consistent.
- Copying implementation from reference repositories.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| No external users or published packages depend on the previous working name. | High | Product owner confirmed no users, not open source, not launched. | Compatibility aliases may be needed, but this lane intentionally does not add them. |
| The selected Nako source icon is `assets/brand/nako-app-icon-1024.png`. | High | `docs/workstreams/nako-brand-identity/README.md` records the canonical asset. | Platform icon generation may need a follow-up. |
| Workspace crates can be renamed together. | Medium | The Rust workspace uses local path dependencies and has no published crates. | Cargo metadata or generated SDK references may need targeted repair. |
| Android namespace can move to `dev.nako`. | Medium | No external app release exists. | Gradle, manifest, and tests may need broad updates. |

## Architecture Direction

Use one decisive namespace migration instead of layering aliases:

- product identity: `Nako`,
- Rust package names: `nako-*`,
- Rust crate identifiers: `nako_*`,
- environment/config prefixes: `NAKO_`,
- Android/Kotlin package roots: `dev.nako`,
- deploy/service/container names: `nako-*`.

This keeps generated contracts, SDKs, addon examples, and operations docs
aligned with the eventual public identity. The old name should not remain in
runtime contracts unless it is a deliberate third-party historical reference.

## Closeout Condition

This lane can close when:

- all active source, package, config, docs, and deploy names use Nako,
- official addon path dependencies are updated,
- repository grep gates have only reviewed residual old-name matches,
- Rust, Admin Web, and feasible Android validation gates pass or have explicit
  blocker notes,
- and follow-on platform icon generation or repository remote renames are split
  out if not completed here.
