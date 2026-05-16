# Taru Documentation

This directory tracks product architecture, implementation goals, engineering
policy, and workstream-level design notes for Taru.

## Current Focus

- Current goal map: [GOALS.md](GOALS.md)
- Product roadmap: [ROADMAP.md](ROADMAP.md)
- Current planned workstream: [crate boundary hardening](workstreams/crate-boundary-hardening/README.md)
- Latest completed workstream: [metadata catalog](workstreams/metadata-catalog/README.md)
- Previous completed workstream: [transcode runtime](workstreams/transcode-runtime/README.md)
- Previous completed workstream: [server architecture hardening](workstreams/server-architecture-hardening/README.md)
- Previous completed workstream: [playback streaming](workstreams/playback-streaming/README.md)
- Storage and VFS archive: [storage and VFS](workstreams/storage-vfs/README.md)
- Foundation archive: [server foundation](workstreams/server-foundation/README.md)
- Refactoring policy: [development/REFACTORING_POLICY.md](development/REFACTORING_POLICY.md)

## Core Documents

- [ADR index](adr/README.md): durable architecture decisions and their status.
- [HTTP API](api/HTTP_API.md): current server API contract.
- [Addon author guide](guides/ADDON_AUTHOR_GUIDE.md): Taru HTTP addon manifest
  and resource contract.
- [Webhook receiver guide](guides/WEBHOOK_RECEIVER_GUIDE.md): webhook
  endpoint setup, signatures, and retry inspection.
- [Automation provider guide](guides/AUTOMATION_PROVIDER_GUIDE.md): external
  automation provider configuration and artifact policy.
- [Local setup](development/LOCAL_SETUP.md): local development workflow.
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
