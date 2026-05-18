# Addon Library File Write Policy

Status: Proposed
Last updated: 2026-05-18

## Why This Lane Exists

Subtitle, NFO, and sidecar-asset writes are more dangerous than Canonical
Metadata writes because they cross from database state into Media Library
storage. Taru already has NFO Round Trip, VFS write modes, atomic replace, and
backup policy workstreams. Addon-initiated file writes must reuse those seams
instead of accepting raw addon-provided paths.

## Problem

Accepted Addon Side Effects can now record and apply Canonical Metadata, but
Library File Writes still need explicit rules for:

- deriving target sidecar locations without exposing Source Locators or paths;
- deciding overwrite, atomic replace, and backup behavior;
- preserving NFO Round Trip content where applicable;
- representing subtitle imports versus NFO exports versus arbitrary sidecar
  assets;
- safely reporting write outcomes without leaking local paths or remote handles;
- idempotent replay after file writes.

## Target State

- Addon file-write side effects move through intake and explicit apply outcome
  state.
- Taru derives the target Library File Write from a media target, file role,
  and policy, not from an addon-provided absolute path.
- NFO writes preserve NFO Round Trip and backup policy.
- Subtitle and sidecar writes use storage/VFS write modes with bounded
  diagnostics.
- Responses and audit summaries expose safe codes and IDs, not raw Source
  Locators, filesystem paths, remote storage handles, or full payloads.

## In Scope

- Audit NFO import/export, VFS write, backup, subtitle, and Addon Side Effect
  seams.
- Decide the first Library File Write target: subtitle import, NFO export, or a
  narrower sidecar asset.
- Define target/path derivation and backup policy.
- Define redacted write reports and idempotent replay semantics.
- Update HTTP API docs and workstream evidence for shipped behavior.

## Out Of Scope

- Direct addon filesystem or remote storage access.
- Addon Manager lifecycle automation.
- Public Client write routes.
- Artwork/Managed Artwork behavior.
- General storage backend redesign outside the selected first write path.

## Architecture Direction

Reuse the APW three-stage model:

1. Addon runtime route authenticates, validates permission/library/target,
   persists the side-effect record, and returns redacted summaries.
2. File-write validation derives a Taru Library File Write command from a media
   target, sidecar role, content type, and policy.
3. Domain apply calls NFO/storage/VFS services, records backup/report summary
   safely, and stores an apply outcome.

If the write can block on remote storage, backup pruning, or large payloads,
prefer a queued Addon Task or durable job over a synchronous runtime request.

## Closeout Condition

This lane can close when:

- current subtitle/NFO/storage seams are audited;
- one bounded Library File Write path is implemented or deliberately split;
- target derivation, write mode, backup policy, idempotency, and redaction are
  tested;
- targeted Rust gates, `cargo fmt --all -- --check`, and `git diff --check`
  pass.
