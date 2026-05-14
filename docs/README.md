# Taru Documentation

This directory tracks product architecture, implementation goals, engineering
policy, and workstream-level design notes for Taru.

## Current Focus

- Current goal map: [GOALS.md](GOALS.md)
- Product roadmap: [ROADMAP.md](ROADMAP.md)
- Active workstream: [server foundation](workstreams/server-foundation/README.md)
- Refactoring policy: [development/REFACTORING_POLICY.md](development/REFACTORING_POLICY.md)

## Core Documents

- [ADR index](adr/README.md): durable architecture decisions and their status.
- [HTTP API](api/HTTP_API.md): current server API contract.
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
