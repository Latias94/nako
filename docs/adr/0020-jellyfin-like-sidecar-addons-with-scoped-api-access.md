# 0020: Use Jellyfin-Like Sidecar Addons with Scoped Taru API Access

## Status

Proposed.

## Context

Taru should offer an extensibility experience comparable to Jellyfin plugins:
metadata providers, bulk scraping, subtitles, artwork, playback resource
suggestions, settings, diagnostics, tasks, event automation, and future
user-facing entry points. Copying Jellyfin's in-process .NET plugin model would
not fit Taru's Rust server architecture, would not provide Jellyfin plugin API
compatibility, and would weaken crash isolation, trust boundaries, and
cross-language addon authoring.

Earlier ADRs chose HTTP addons before in-process plugins and capability-scoped
automation providers. The remaining decision is how much power addons should
have. A suggestion-only model is too weak for the Jellyfin-like experience Taru
wants; a native plugin ABI is too risky for the current product stage.

## Decision

Taru will use sidecar addons that implement the Taru Addon Protocol. An addon
declares its protocol version, resources, entry points, event subscriptions,
configuration schema, requested coarse permissions, and optional library-scoped
grants in its manifest. Taru validates the manifest and stores the accepted
registration.

Addons may perform strong side effects, including metadata writes, artwork or
subtitle updates, task execution, and other authorized changes, but those
effects must pass through Taru-owned APIs. Addon sidecars authenticate with
revocable, rotatable addon tokens scoped to the addon registration, accepted
permissions, and library grants. Addons must not receive admin tokens, database
credentials, raw filesystem authority, or unmediated storage access.

Taru owns task lifecycle, progress, cancellation, audit, event delivery,
artifact intake, library file writes, storage/VFS boundaries, playback runtime
sessions, resource budgets, and API-safe error behavior. Addons own their
external fetches and execution logic, but Taru-managed artifacts and library
file changes enter Taru through explicit APIs. Addons may expose hosted pages
for advanced settings or diagnostics, but Taru does not treat those pages as
trusted admin UI and does not pass admin credentials to them.

The first implementation phase supports manually registered sidecar addons.
Automatic discovery, download, installation, update, process supervision, and
marketplace behavior belong to a later addon manager design.

## Consequences

- Taru can provide a Jellyfin-like extension experience without adopting
  Jellyfin plugin API compatibility or an in-process native plugin ABI.
- Addons can be powerful enough for bulk scraping and operational workflows
  while still being constrained by Taru permissions, library scope, audit, and
  resource boundaries.
- Addon authors can use any language that can implement the HTTP protocol.
- Taru needs stable addon-token issuance, revocation, permission checks,
  library-scope enforcement, event delivery, task reporting, and diagnostics.
- Some workflows will require addon authors to run a sidecar service manually
  until an addon manager exists.

## Alternatives Considered

- In-process native plugins: rejected because Rust has no stable native plugin
  ABI for this use case, and crash isolation, sandboxing, versioning, and trust
  boundaries would be harder to maintain.
- Jellyfin plugin API compatibility: rejected because it would force Taru to
  emulate Jellyfin's internal .NET object model rather than build around Taru's
  own media, storage, task, permission, and playback boundaries.
- Suggestion-only addons: rejected because it is too weak for Jellyfin-like
  plugin workflows such as trusted bulk metadata scraping, subtitle updates,
  and operational tasks.
- OAuth-first addon authorization: deferred because self-hosted sidecars can
  start with revocable, rotatable addon tokens; OAuth can be revisited if Taru
  later supports remote multi-tenant addon services.
- Built-in addon manager first: deferred because package signing, process
  supervision, update rollback, logs, port allocation, and marketplace policy
  are separate lifecycle concerns from the initial addon protocol.

## Related Workstreams

- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/workstreams/addons-automation/`
