# Architecture Decision Records

This directory tracks architecture decisions for Nako.

## Format

Each ADR should include:

- Status: proposed, accepted, rejected, superseded
- Context
- Decision
- Consequences
- Alternatives considered
- Related workstreams

Implemented ADRs should be promoted from `proposed` to `accepted` during
documentation hygiene. Do not batch status changes unless the implementation
evidence has been reviewed.

## Index

- [0001: Use a Modular Monolith Rust Workspace](0001-modular-monolith-rust-workspace.md)
- [0002: Build an Internal VFS Before OS Mounting](0002-internal-vfs-before-os-mounting.md)
- [0003: Prefer HTTP Addons Before In-Process Plugins](0003-http-addons-before-in-process-plugins.md)
- [0004: Treat AI as External Automation First](0004-ai-as-external-automation-first.md)
- [0005: Use Bounded Async Pipelines and Resource Budgets](0005-bounded-async-pipelines-and-resource-budgets.md)
- [0006: Persist Job Inputs and Use Explicit Retry Policy](0006-persist-job-inputs-and-explicit-retry-policy.md)
- [0007: Define Metadata Merge Policy and Local Authority](0007-metadata-merge-policy-and-local-authority.md)
- [0008: Treat NFO as a Local Metadata Boundary](0008-nfo-as-local-metadata-boundary.md)
- [0009: Resolve Provider Secrets from Environment References](0009-resolve-provider-secrets-from-environment.md)
- [0010: Treat Library Presets as Configuration Templates](0010-library-presets-are-configuration-templates.md)
- [0011: Normalize Catalog Graph and Project Search Documents](0011-normalized-catalog-graph-and-search-projection.md)
- [0012: Persist Scan State and Source Tombstones](0012-durable-scan-state-and-source-tombstones.md)
- [0013: Use Bounded Artwork Task Resource Classes](0013-bounded-artwork-task-resource-classes.md)
- [0014: Use a Durable Event Outbox for Webhooks and Automation](0014-durable-event-outbox-for-webhooks-and-automation.md)
- [0015: Use Capability-Scoped HTTP Addons and Automation Providers](0015-capability-scoped-http-addons-and-automation-providers.md)
- [0016: Define Remote Storage and VFS Cache Boundaries](0016-remote-storage-and-vfs-cache-boundary.md)
- [0017: Define Playback Streaming and Remote Hardening Boundaries](0017-playback-streaming-and-remote-hardening-boundaries.md)
- [0018: Use a Shared Metadata Provider Runtime and Diagnostics Boundary](0018-metadata-provider-runtime-and-diagnostics.md)
- [0019: Use a Thin Server Composition Root and Explicit Runtime Supervisors](0019-server-architecture-hardening-boundaries.md)
- [0020: Use Jellyfin-Like Sidecar Addons with Scoped Nako API Access](0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md)
- [0021: Use a Video-First Media Server Domain Model](0021-video-first-media-server-domain-model.md)
- [0022: Keep Public Protocol Crates Permissive While Server Crates Remain AGPL](0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md)
- [0023: Stabilize Public API Versions and Error Envelopes](0023-public-api-versioning-and-error-envelope-contract.md)
- [0024: Add an Inbound Token Authentication Boundary](0024-inbound-token-authentication-boundary.md)
- [0025: Generate Public Client OpenAPI From Protocol-Owned Wire Types](0025-openapi-public-client-sdk-contract.md)
- [0026: Use Native Client Shells With a Shared Rust Client Core](0026-native-client-shells-with-shared-rust-client-core.md)
- [0027: Define a Versioned Admin API Boundary for the Web Console](0027-admin-api-boundary-for-web-console.md)
- [0028: Resolve User Playback State Through a Stable Principal](0028-user-playback-state-principal-and-public-contract.md)
- [0029: Use a PostgreSQL-Ready Persistence Boundary](0029-postgresql-ready-persistence-boundary.md)
- [0030: Define PostgreSQL-Ready SQL Dialect And Migration Policy](0030-postgresql-ready-sql-dialect-and-migration-policy.md)
- [0031: Sequence Generated Client SDK Before Mobile Rust FFI](0031-android-client-sdk-before-mobile-rust-ffi.md)
- [0032: Pull Shared Rust Client Core Forward Behind App-Supplied Transport](0032-shared-rust-client-core-app-supplied-transport.md)
- [0033: Version Addon Protocol Compatibility Separately From Addon and Crate Releases](0033-version-addon-protocol-independently-from-addon-and-crate-releases.md)
- [0034: Package Addon Capabilities Into Sidecar Suites](0034-package-addon-capabilities-into-sidecar-suites.md)
- [0035: Addon Native Metadata Writeback](0035-addon-native-metadata-writeback.md)
- [0036: Use Short-Lived Playback Tickets for Browser Media Transport](0036-short-lived-browser-playback-tickets.md)
- [0037: Add Local Credential and Session Authentication](0037-local-credential-and-session-auth.md)
- [0038: Deepen Playback Planning and Transcode Policy Seams](0038-playback-planning-and-transcode-policy-seams.md)
- [0039: Keep Playback Policy and Renderer Targets Explicit](0039-playback-policy-and-renderer-target-boundary.md)
- [0040: Model Casting as Renderer Sessions and Protocol Adapters](0040-casting-as-renderer-session-adapter.md)
- [0041: Separate Renderer Cast-Safe Transport Tickets From Browser Tickets](0041-renderer-cast-safe-transport-tickets.md)
- [0042: Use Sidecar Renderer Adapters For External Casting Protocols](0042-sidecar-renderer-adapters-for-external-casting-protocols.md)
- [0043: Ship Chromecast First As An Official Renderer Adapter Sidecar](0043-ship-chromecast-first-as-official-renderer-adapter.md)
