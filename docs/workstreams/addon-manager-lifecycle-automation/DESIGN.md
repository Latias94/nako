# Addon Manager Lifecycle Automation

Status: Completed
Last updated: 2026-05-23

## Why This Lane Exists

Nako's released alpha now proves the manual addon path:

- a published server image can run;
- `nako-addon-protocol` and `nako-addon-client` are public crates;
- `nako-metadata-scraper@0.1.0-alpha.1` can be installed and smoke-tested;
- the first official companion addon can register, health-check, and serve one
  hosted metadata resource call through Nako.

That proves the manual operator path, not the manager path. This lane delivered
the first manager-owned registry/plan slot without collapsing marketplace,
package signing, process/container supervision, or Native Plugin ABI into the
first slice.

## Problem

Operators still have to start and manage addon sidecars manually. Before this
lane, Nako could describe, register, validate, and diagnose addons, but it did
not own a registry/plan loop for addon lifecycle intent. This lane added that
first plan surface while leaving sidecar process ownership external.

## Target State

This lane closed after Nako became able to:

- represent a managed addon source through a redaction-safe registry/plan
  surface;
- represent operator-confirmed install/update/remove intent as a Nako-owned
  plan;
- surface addon permissions, token summaries, grants, and Addon Health Check
  visibility through Nako-owned surfaces;
- keep Addon Install Guide behavior first-class for the operator;
- keep marketplace hosting, package signing, process/container supervision,
  and provider breadth as separate follow-ons;
- keep the Addon Protocol contract and the official addon smoke stable while
  manager features evolve.

## Scope

- Addon Manager discovery and lifecycle plan modeling.
- Operator-confirmed install/update/remove intent capture.
- Addon permissions, token rotation, and health-check visibility surfaces.
- Minimal UI/API contract changes needed to surface managed addon state and
  install-guide behavior.
- Validation and documentation for the first manager-owned addon slice.

## Non-Goals

- Marketplace hosting or distribution policy.
- Package signing trust roots.
- Direct container or process supervision.
- Native Plugin ABI or in-process addon execution.
- Broad provider breadth beyond the official companion addon path.
- OAuth-first addon auth redesign.
- Public client API changes unrelated to addon management.

## Shipped Direction

Treat the manager as a Nako-owned control plane, not as a source of hidden
process magic. The manager should own:

- addon source registry and source resolution;
- install/update/remove intent capture and version selection policy;
- addon permissions and token rotation;
- operator-visible health and install-guide state;
- safe rollback boundaries for the manager-owned plan surface.

The sidecar should still own its provider logic and protocol envelope. The
manager should not smuggle admin credentials into addons, require Docker socket
authority, or convert the sidecar boundary into an in-process plugin ABI.

Process/container supervision may become a later follow-on lane if Nako ever
decides to own it, but it is not part of this completed slice.

## Related Docs

- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- `docs/workstreams/official-addon-e2e-alpha2/`
- `docs/workstreams/addon-runtime-and-distribution/`
- `docs/workstreams/admin-addon-operations-mvp/`
