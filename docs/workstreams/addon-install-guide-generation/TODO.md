# Addon Install Guide Generation TODO

Status: Completed
Last updated: 2026-05-22

## AIG.0 Scope And Contract Baseline

- [x] AIG-010 [owner=codex] [deps=none] [scope=docs/workstreams/addon-install-guide-generation, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Open the workstream, freeze the Addon Install Guide boundary, and set
  the Codex goal for this productization lane.
  Validation: Workstream docs agree with `CONTEXT.md` language and explicitly
  separate Addon Install Guide from Addon Manager.
  Evidence: this workstream.
  Handoff: Continue with AIG-020.

## AIG.1 Server-Owned Guide

- [x] AIG-020 [owner=codex] [deps=AIG-010] [scope=crates/nako-api/src/extension.rs, crates/nako-api/src/admin_contract.rs, crates/nako-server/src/app/addons.rs, crates/nako-server/src/http/addons.rs, crates/nako-server/src/http/tests/addons.rs, apps/admin-web/src/adminApi/generated/contract.ts]
  Goal: Add `GET /admin/v1/addons/{addon_id}/install-guide` as a redaction-safe
  server-owned Admin read model with generated Admin Web contract coverage.
  Validation: `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`, `cargo nextest run -p nako-api admin_contract --no-fail-fast`, and `cargo nextest run -p nako-server install_guide --no-fail-fast`.
  Review: Route must not leak resolved secrets, raw tokens, local paths, Source
  Locators, storage URIs, or process-control promises.
  Evidence: Rust tests and generated contract diff.
  Handoff: Continue with AIG-030.

## AIG.2 Admin Web Preview

- [x] AIG-030 [owner=codex] [deps=AIG-020] [scope=apps/admin-web/src/adminApi, apps/admin-web/src/App.tsx, apps/admin-web/src/App.test.tsx]
  Goal: Render the Addon Install Guide in Admin Web through the existing
  data-source seam with safe mock fallback.
  Validation: `npm test` and `npm run build` from `apps/admin-web`.
  Review: UI must present snippets as inert previews and must repeat that Nako
  does not manage Addon Sidecar lifecycle.
  Evidence: Admin Web tests and build output.
  Handoff: Continue with AIG-040.

## AIG.3 Documentation And Closeout

- [x] AIG-040 [owner=codex] [deps=AIG-030] [scope=docs/api/HTTP_API.md, docs/guides/ADDON_AUTHOR_GUIDE.md, docs/workstreams/addon-install-guide-generation, docs/GOALS.md, docs/ROADMAP.md, docs/workstreams/README.md]
  Goal: Document the shipped guide route and close the workstream with fresh
  evidence.
  Validation: `cargo fmt --all -- --check`, focused Rust gates, Admin Web
  gates, and `git diff --check`.
  Review: Split Addon Manager automation as a follow-on if requested, not as
  hidden scope in this lane.
  Evidence: `EVIDENCE_AND_GATES.md`, `MILESTONES.md`, `HANDOFF.md`, and close
  journal.
