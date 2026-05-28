# Subtitle Complete Chain TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And ADR

- [x] SCC-010 [owner=codex] [deps=none] [scope=docs/adr,docs/workstreams/subtitle-complete-chain]
  Goal: Open the durable subtitle complete chain lane and record the host-owned
  subtitle import ADR.
  Validation: `git diff --check`.
  Review: ADR must reject direct addon file writes and name Library File Write
  as the future persistence boundary.
  Evidence: `DESIGN.md`; `docs/adr/0051-host-owned-subtitle-import-chain.md`.
  Handoff: DONE 2026-05-28. Continue with SCC-020.

## M1 - Shared Subtitle Protocol Contract

- [ ] SCC-020 [owner=codex] [deps=SCC-010] [scope=crates/nako-addon-protocol,crates/nako-official-addon-catalog,crates/nako-server]
  Goal: Move subtitle search request/response/candidate/delivery/status types
  and schema constants into `nako-addon-protocol`, and keep official catalog
  facts on the shared schema constants.
  Validation: `cargo nextest run -p nako-addon-protocol subtitle --no-fail-fast`;
  `cargo check -p nako-addon-protocol -p nako-official-addon-catalog -p nako-server --tests`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Review: Protocol types must remain provider-neutral and must not include
  target filesystem paths or write-policy fields.
  Evidence: `crates/nako-addon-protocol/src/lib.rs`.
  Handoff: Mark DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

## M2 - Official Provider Migration

- [ ] SCC-030 [owner=codex] [deps=SCC-020] [scope=F:\SourceCodes\Rust\nako-official-addons\crates\nako-subtitle-provider,F:\SourceCodes\Rust\nako-official-addons\addons\subtitle-provider]
  Goal: Make the official subtitle provider use the shared protocol subtitle
  types and constants instead of private duplicate wire structs.
  Validation: `cargo nextest run -p nako-subtitle-provider --no-fail-fast`;
  `cargo check -p nako-subtitle-provider --tests`; `cargo fmt --all -- --check`;
  `git diff --check`.
  Review: Provider remains read-only and fixture-backed.
  Evidence: `crates/nako-subtitle-provider/src/subtitles.rs`.
  Handoff: Mark DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

## M3 - Host Follow-On Contract

- [ ] SCC-040 [owner=planner] [deps=SCC-020] [scope=docs/workstreams/subtitle-complete-chain]
  Goal: Record the future Nako host stages for candidate selection, import
  planning, Library File Write apply, refresh, and playback visibility.
  Validation: `git diff --check`.
  Review: The follow-on must not imply this lane writes subtitle files.
  Evidence: `FOLLOW_ONS.md`.
  Handoff: Mark DONE once follow-ons are explicit.

## M4 - Closeout

- [ ] SCC-050 [owner=codex] [deps=SCC-020,SCC-030,SCC-040] [scope=docs/workstreams/subtitle-complete-chain]
  Goal: Run fresh gates, update evidence, and close or split remaining work.
  Validation: final package gates pass or blockers are concrete.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, `HANDOFF.md`.
  Handoff: Summarize remaining host subtitle import priorities.
