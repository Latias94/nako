# Public Client API Contract TODO

Status: Completed
Last updated: 2026-05-17

## M29.0 Scope And Evidence Freeze

- [x] PCA-010 [owner=planner] [deps=none] [scope=docs/workstreams/public-client-api]
  Goal: Freeze M29 problem, target state, non-goals, migration rules, and first proof slice.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/public-client-api/DESIGN.md
  Handoff: Continue with PCA-020 before migrating route behavior.

## M29.1 Public Browse Protocol DTO Slice

- [x] PCA-020 [owner=codex] [deps=PCA-010] [scope=crates/nako-client-protocol, crates/nako-api, crates/nako-server/src/app/{catalog,library}.rs]
  Goal: Move the first stable library/catalog browse DTOs into `nako-client-protocol` and keep `nako-api` as the mapping layer.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, cargo nextest run -p nako-client-protocol --no-fail-fast, cargo nextest run -p nako-api --no-fail-fast, cargo nextest run -p nako-server http::tests::catalog --no-fail-fast, cargo nextest run -p nako-server http::tests::system --no-fail-fast, cargo tree -p nako-client-protocol.
  Evidence: Protocol crate owns library/item/source/probe/search/list/detail response structs; `nako-api` owns mapping functions from server records.
  Handoff: Completed. Diagnostics, jobs, ingestion failures, metadata provider internals, webhook, automation, and addon admin DTOs remain server/API-owned.

## M29.2 Public Playback Decision DTO Slice

- [x] PCA-030 [owner=codex] [deps=PCA-020] [scope=crates/nako-client-protocol, crates/nako-api, crates/nako-server/src/app/playback]
  Goal: Move the public playback decision response shape into `nako-client-protocol` without exposing `nako_streaming::PlaybackDecision`.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, cargo nextest run -p nako-api --no-fail-fast, cargo nextest run -p nako-server http::tests::playback --no-fail-fast, cargo tree -p nako-client-protocol.
  Evidence: Protocol crate owns playback decision wire DTOs; `nako-api` maps streaming/transcode plans into protocol structs.
  Handoff: Completed. Transcode session persistence/admin detail remains in `nako-api`.

## M29.3 Contract Docs And Route Evidence

- [x] PCA-040 [owner=codex] [deps=PCA-030] [scope=docs, crates/nako-server/src/http/tests]
  Goal: Record the public client API contract and prove browse/search/list/detail/playback route JSON still matches it.
  Validation: cargo nextest run -p nako-server http::tests --no-fail-fast, cargo tree -p nako-client-protocol.
  Evidence: EVIDENCE_AND_GATES.md maps routes and DTOs to tests.
  Handoff: Completed. Split API versioning, auth, and client SDK generation into follow-ons if needed.

## M29.4 Closeout

- [x] PCA-050 [owner=codex] [deps=PCA-040] [scope=docs/workstreams/public-client-api]
  Goal: Close M29 with a completion audit against the prompt requirements.
  Validation: cargo fmt --all -- --check, cargo check --workspace --tests, cargo nextest run --workspace --no-fail-fast, cargo tree -p nako-client-protocol, git diff --check.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Completed. Remaining protocol migration and client SDK work are follow-ons.
