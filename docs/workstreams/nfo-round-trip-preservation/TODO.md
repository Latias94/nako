# NFO Round Trip Preservation Task Ledger

Status: Completed
Last updated: 2026-05-17

## M0 - Scope And Evidence Freeze

- [x] NRT-010 [owner=codex] [deps=none] [scope=docs/workstreams/nfo-round-trip-preservation,docs/GOALS.md]
  Goal: Open M47 with preservation scope, non-goals, conflict model, and
  validation gates.
  Validation: Workstream docs exist and agree.
  Evidence: `docs/workstreams/nfo-round-trip-preservation/DESIGN.md`.
  Handoff: Continue with the codec preservation model.

## M1 - Codec Preservation Model

- [x] NRT-020 [owner=codex] [deps=NRT-010] [scope=crates/taru-nfo/src/codec.rs,crates/taru-nfo/src/lib.rs]
  Goal: Add a test-visible preservation-aware movie NFO update path and report
  type that updates Taru-owned fields while retaining unknown XML elements.
  Validation: `cargo check -p taru-nfo --tests`;
  `cargo nextest run -p taru-nfo movie_nfo_preservation --no-fail-fast`.
  passed.
  Evidence: codec tests show unknown fields preserved, owned fields updated,
  and duplicate owned fields reported.
  Handoff: Wire forced export to use the update path when a sidecar exists.

## M2 - Export Workflow Wiring

- [x] NRT-030 [owner=codex] [deps=NRT-020] [scope=crates/taru-nfo/src/export.rs,crates/taru-nfo/src/lib.rs]
  Goal: During forced export over an existing sidecar, read existing XML and
  write preservation-aware output instead of regenerating the whole document.
  Validation: `cargo check -p taru-nfo --tests`;
  `cargo nextest run -p taru-nfo nfo_service_preserves_existing_sidecar_unknown_fields_when_forced --no-fail-fast`.
  passed.
  Evidence: service test proves existing unknown sidecar fields survive forced
  export while Taru-owned fields update. Import-then-forced-export round trip
  test also proves unknown sidecar fields survive after import.
  Handoff: Run focused and workspace closeout gates.

## M3 - Validation And Closeout

- [x] NRT-040 [owner=codex] [deps=NRT-030] [scope=workspace,docs]
  Goal: Close M47 with focused and workspace validation, evidence updates, and
  follow-on notes.
  Validation: `cargo fmt --all -- --check`; `cargo check -p taru-nfo --tests`;
  `cargo nextest run -p taru-nfo --no-fail-fast`; `cargo check --workspace --tests`;
  `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  passed.
  Evidence: `EVIDENCE_AND_GATES.md` and `docs/GOALS.md`.
  Handoff: Recommend the next goal from VFS/storage write policy, NFO nested
  preservation, or API diagnostics only after M47 evidence is complete.
