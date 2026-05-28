# Subtitle Fact Refresh Task Ledger

## SFR-010 - Workstream Boundary

- [x] SFR-010 [owner=codex] [deps=none] [scope=docs/workstreams/subtitle-fact-refresh]
  Goal: Open the workstream and select `media_probe` as the sidecar subtitle
  fact read model.
  Validation: documentation review.

## SFR-020 - Stream DTO Contract

- [x] SFR-020 [owner=codex] [deps=SFR-010] [scope=crates/nako-core,crates/nako-client-protocol,crates/nako-api]
  Goal: Add media stream origin/disposition to public DTOs so sidecar subtitle
  facts are visible without exposing paths.
  Validation: `cargo nextest run -p nako-api media_stream --no-fail-fast`.

## SFR-030 - Apply Refresh

- [x] SFR-030 [owner=codex] [deps=SFR-020] [scope=crates/nako-server/src/app/addons.rs,crates/nako-server/src/http/tests/addons.rs]
  Goal: Refresh imported subtitle sidecar facts after import apply and keep
  repeated apply idempotent.
  Validation: `cargo nextest run -p nako-server addon_subtitle_import --no-fail-fast`.

## SFR-040 - Closeout

- [x] SFR-040 [owner=codex] [deps=SFR-020,SFR-030] [scope=docs/workstreams/subtitle-fact-refresh]
  Goal: Record gates and commit the bounded slice.
  Validation: `cargo check -p nako-api -p nako-server --tests`;
  `cargo fmt --all -- --check`; `git diff --check`.
