# Subtitle Import Apply Task Ledger

## SIA-010 - Workstream Boundary

- [x] SIA-010 [owner=codex] [deps=none] [scope=docs/workstreams/subtitle-import-apply]
  Goal: Open the apply workstream and lock the host-owned mutation boundary.
  Validation: documentation review.
  Handoff: Opened 2026-05-28.

## SIA-020 - API Contract

- [x] SIA-020 [owner=codex] [deps=SIA-010] [scope=crates/nako-api/src/extension.rs,crates/nako-api/src/admin_contract.rs,apps/admin-web/src/adminApi/generated/contract.ts,web/src/api/admin/generated/contract.ts]
  Goal: Add import-apply request/response DTOs and generated TypeScript
  contract.
  Validation: `cargo nextest run -p nako-api subtitle_import_apply --no-fail-fast`;
  `cargo nextest run -p nako-api admin_contract --no-fail-fast`.
  Handoff: DONE 2026-05-28. Added redaction-safe apply request/response
  DTOs, route contract, and generated TypeScript copies.

## SIA-030 - Host Apply Runtime

- [x] SIA-030 [owner=codex] [deps=SIA-020] [scope=crates/nako-server/src/app/addons.rs,crates/nako-server/src/app/addons/library_file_write.rs,crates/nako-server/src/http/addons.rs,crates/nako-server/src/http/tests/addons.rs]
  Goal: Recompute a ready import plan, resolve subtitle content, validate it,
  and write the sidecar through Library File Write.
  Validation: `cargo nextest run -p nako-server addon_subtitle_import_apply --no-fail-fast`.
  Handoff: DONE 2026-05-28. Implemented inline/download-url apply,
  idempotent same-content handling, create-missing conflict, replace-existing
  backup, and redaction-safe reports.

## SIA-040 - Verification And Closeout

- [x] SIA-040 [owner=codex] [deps=SIA-020,SIA-030] [scope=docs/workstreams/subtitle-import-apply]
  Goal: Record gates, update handoff, and commit the bounded slice.
  Validation: `cargo check -p nako-api -p nako-server --tests`;
  `cargo fmt --all -- --check`; `git diff --check`.
  Handoff: DONE 2026-05-28. Final gates passed; subtitle fact refresh,
  artifact-ref resolver, and downloader policy remain follow-ons.
