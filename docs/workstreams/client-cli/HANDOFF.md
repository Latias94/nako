# Client CLI Entrypoint Handoff

Status: Completed
Last updated: 2026-05-17

## Current State

M37 is closed. `crates/taru-client-cli` is an Apache-2.0 public client CLI on
top of `taru-client`, with focused tests and workspace validation complete.

## Active Task

- None.

## Decisions Since Last Update

- The first concrete client should be a Rust CLI because it validates the
  public SDK/license boundary before Flutter, Web, or publishing work.
- The CLI is separate from `taru-server`'s existing operator commands.
- The CLI must not depend on AGPL server/internal Taru crates.
- `GET /health` remains unauthenticated; authenticated command tests use
  `search`.
- Streaming commands print request facts and redact bearer token values.

## Blockers

- None.

## Next Recommended Action

- Choose M38 around full Rust SDK streaming body abstraction, TypeScript SDK
  streaming/package parity, or Flutter/Dart SDK foundation.
