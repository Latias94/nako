# Nako Documentation

This directory tracks product architecture, implementation goals, engineering
policy, and workstream-level design notes for Nako.

## Current Focus

- Current goal map: [GOALS.md](GOALS.md)
- Product roadmap: [ROADMAP.md](ROADMAP.md)
- Active workstream:
  [Fearless architecture deepening](workstreams/fearless-architecture-deepening/README.md)
- Proposed follow-on workstream:
  [Managed Artwork PostgreSQL parity](workstreams/managed-artwork-postgresql-parity/README.md)
- Latest completed workstream:
  [PostgreSQL production readiness](workstreams/postgresql-production-readiness/README.md)
- Previous completed workstream:
  [Future-ready architecture refactor](workstreams/future-ready-architecture-refactor/README.md)
- Previous completed workstream:
  [Admin catalog governance read model](workstreams/admin-catalog-governance-read-model/README.md)
- Previous completed workstream: [Rust client SDK](workstreams/rust-client-sdk/README.md)
- Previous completed workstream: [TypeScript SDK package](workstreams/typescript-sdk-package/README.md)
- Previous completed workstream: [SDK client scaffold](workstreams/sdk-client-scaffold/README.md)
- Previous completed workstream: [OpenAPI client contract](workstreams/openapi-client-contract/README.md)
- Previous completed workstream: [access boundary auth](workstreams/access-boundary-auth/README.md)
- Previous completed workstream: [public API contract](workstreams/public-api-contract/README.md)
- Previous completed workstream: [public client API](workstreams/public-client-api/README.md)
- Previous completed workstream: [crate boundary hardening](workstreams/crate-boundary-hardening/README.md)
- Previous completed workstream: [metadata catalog](workstreams/metadata-catalog/README.md)
- Previous completed workstream: [transcode runtime](workstreams/transcode-runtime/README.md)
- Previous completed workstream: [server architecture hardening](workstreams/server-architecture-hardening/README.md)
- Previous completed workstream: [playback streaming](workstreams/playback-streaming/README.md)
- Storage and VFS archive: [storage and VFS](workstreams/storage-vfs/README.md)
- Foundation archive: [server foundation](workstreams/server-foundation/README.md)
- Refactoring policy: [development/REFACTORING_POLICY.md](development/REFACTORING_POLICY.md)

## Core Documents

- [Architecture map](ARCHITECTURE.md): current system map, media-server north
  star, progress matrix, and next architectural pressure points.
- [Playback architecture](architecture/PLAYBACK.md): video playback capability
  map, workstream/ADR authority, next lanes, and risk register.
- [ADR index](adr/README.md): durable architecture decisions and their status.
- [HTTP API](api/HTTP_API.md): current server API contract.
- [Addon author guide](guides/ADDON_AUTHOR_GUIDE.md): Nako HTTP addon manifest
  and resource contract.
- [Webhook receiver guide](guides/WEBHOOK_RECEIVER_GUIDE.md): webhook
  endpoint setup, signatures, and retry inspection.
- [Automation provider guide](guides/AUTOMATION_PROVIDER_GUIDE.md): external
  automation provider configuration and artifact policy.
- [Local setup](development/LOCAL_SETUP.md): local development workflow.
- [Self-hosted deployment](deployment/SELF_HOSTED.md): SQLite/PostgreSQL
  deployment examples and operator configuration guidance.
- [Backup/restore/upgrade](deployment/BACKUP_RESTORE_UPGRADE.md): self-hosted
  state classification, backup, restore, and migration runbook.
- [Test strategy](development/TEST_STRATEGY.md): validation gates and coverage
  expectations.
- [Licensing](legal/LICENSING.md): license policy and reference-code boundary.
- [Workstreams](workstreams/README.md): long-running implementation areas.

## How To Update Docs

When a goal is completed:

- update [GOALS.md](GOALS.md) with the result and evidence;
- update the relevant workstream milestone and TODO files;
- add or revise ADRs when an implementation decision changes architecture;
- add phase notes for non-trivial milestones;
- keep validation evidence close to the completed milestone.

When a change crosses crate boundaries, changes public API shape, or alters
resource/concurrency policy, update the roadmap or ADRs before considering the
work complete.
