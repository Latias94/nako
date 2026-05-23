# Addon Source Catalog And Marketplace - Handoff

Status: Completed
Last updated: 2026-05-24

## Current State

The Addon core runtime and architecture mainline is complete. Nako now exposes
the first read-only source catalog / marketplace discovery surface for the
built-in official addon source.

Implemented pieces:

- Admin DTOs for source catalog sources, entries, and resolved install
  candidates.
- `GET /admin/v1/addons/catalog/sources`.
- `GET /admin/v1/addons/catalog/entries`.
- `GET /admin/v1/addons/catalog/entries/{entry_id}/resolve`.
- Built-in `nako-official` source with the official metadata scraper entry.
- Resolution to `AddonInstallDescriptor` plus protocol install guide.
- Explicit lifecycle flags proving Nako does not manage packages, processes, or
  containers in this slice.
- Focused Admin route test proving catalog browse/resolve does not create Addon
  registrations, routing plans, jobs, manager intent, package-signing facts, or
  process-control output.

## Next Task

Open a product/ecosystem follow-on only when that scope is ready.

Recommended follow-ons:

- package signing and trust-root policy;
- provider breadth beyond the first official companion addon;
- rollback/update execution beyond the current manager-plan intent surface;
- authenticated outbound task dispatch credential storage for `Bearer` and
  `SharedSecret` sidecars;
- official-addon task-path smoke coverage once an official addon exposes a task
  declaration;
- process/container supervision, if Nako decides to own sidecar execution.

## Known Risks

- Additional remote or third-party sources need a separate persistence and trust
  model; do not widen the built-in source slice implicitly.
- Package signing and trust roots should not be inferred from `nako-official`
  until that lane defines verification evidence.
- Process/container supervision remains an operator-risk decision and should
  not be added through catalog browse or resolve routes.
