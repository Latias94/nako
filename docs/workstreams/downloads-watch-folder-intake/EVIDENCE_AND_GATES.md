# Downloads / Watch-Folder Intake — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo nextest run -p taru-db acquisition_intake --no-fail-fast
cargo nextest run -p taru-server acquisition_intake --no-fail-fast
cargo nextest run -p taru-api admin_contract --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo fmt --all -- --check
npm run check # from apps/admin-web, after Admin contract/client changes
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

For planning-only changes, validate JSON and diff hygiene:

```powershell
python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0002-internal-vfs-before-os-mounting.md`
- `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
- `docs/adr/0008-nfo-as-local-metadata-boundary.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0017-playback-streaming-and-remote-hardening-boundaries.md`
- `docs/adr/0021-video-first-media-server-domain-model.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/managed-import-staging/DESIGN.md`
- `docs/workstreams/link-apply-and-import-promotion/DESIGN.md`
- `docs/workstreams/nfo-sidecar-promotion-apply/DESIGN.md`
- `docs/workstreams/playback-transcode-ops-hardening/DESIGN.md`
- `crates/taru-core/src/managed_import.rs`
- `crates/taru-core/src/repository/managed_import.rs`
- `crates/taru-server/src/app/managed_import.rs`
- `crates/taru-vfs/src/lib.rs`
- `crates/taru-api/src/admin.rs`
- `crates/taru-api/src/admin_contract.rs`
- `crates/taru-server/src/http/admin.rs`
- `crates/taru-server/src/http/query.rs`
- `crates/taru-server/src/http/tests/system.rs`
- `apps/admin-web/src/adminApi`
- `apps/admin-web/src/App.tsx`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | DWI-010 | `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Scope is acquisition intake/watch-folder discovery only; first implementation task is durable intake candidate domain. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | DWI-020 red gate | `cargo nextest run -p taru-db acquisition_intake --no-fail-fast` | Expected fail. New backend-neutral contract could not compile because `AcquisitionIntakeRepository` was not implemented for SQLite/PostgreSQL/facade yet. |
| 2026-05-22 | DWI-020 implementation gate | `cargo nextest run -p taru-db acquisition_intake --no-fail-fast`; `cargo check -p taru-db --tests`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. SQLite acquisition intake contract passed: 1 passed, 123 skipped. `cargo check -p taru-db --tests` passed. Formatting passed. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. DWI-020 added core intake candidate records, repository traits, SQLite/PostgreSQL migrations/adapters, facade dispatch, backend capabilities, and contract coverage without Media Source or Library File Write behavior. |
| 2026-05-22 | DWI-030 red gates | `cargo nextest run -p taru-server acquisition_intake --no-fail-fast` | Expected fail twice during TDD. First run had no acquisition-intake app tests (`error: no tests to run`). After adding app tests, same-source candidate acceptance failed with a Managed Import unique constraint, proving the app service did not yet reuse an existing artifact for the same source. |
| 2026-05-22 | DWI-030 implementation gate | `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`; `cargo nextest run -p taru-server managed_import --no-fail-fast`; `cargo nextest run -p taru-db acquisition_intake --no-fail-fast`; `cargo check -p taru-server --tests`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Acquisition intake app tests passed: 3 passed, 229 skipped. Managed Import focused regression passed: 18 passed, 214 skipped. DB intake contract passed: 1 passed, 123 skipped. Server test check and formatting passed. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. DWI-030 added `AcquisitionIntakeAppService`, TaruApp composition wiring, redacted candidate diagnostics, idempotent record/list behavior, explicit existing-artifact linking, same-source artifact reuse, new artifact creation, and tests proving no promotion apply, Media Source creation, or library file mutation. |
| 2026-05-22 | DWI-040 red gate | `cargo nextest run -p taru-server acquisition_intake_watch_folder --no-fail-fast` | Expected fail. The watch-folder discovery test could not compile because `DiscoverWatchFolderCandidatesRequest` and `discover_watch_folder_candidates` did not exist. |
| 2026-05-22 | DWI-040 implementation gate | `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`; `cargo nextest run -p taru-vfs --no-fail-fast`; `cargo nextest run -p taru-db acquisition_intake --no-fail-fast`; `cargo check -p taru-server --tests`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Acquisition intake app tests passed: 4 passed, 229 skipped. VFS tests passed: 45 passed. DB intake contract passed: 1 passed, 123 skipped. Server test check and formatting passed. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. DWI-040 added watch-folder discovery via configured storage/VFS list/stat, ready/incomplete/unsupported candidate classification, idempotent intake record writes, redacted discovery diagnostics, and tests proving no Managed Import artifact creation, Media Source creation, promotion apply, or library file mutation. |
| 2026-05-22 | DWI-050 red gates | `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `cargo nextest run -p taru-server admin_v1_acquisition_intake --no-fail-fast` | Expected fail during TDD. The Admin contract gate failed until the generated Admin web TypeScript contract was synchronized. The HTTP gate first failed before the Admin route/DTO wiring was complete, then exposed a root URI parsing redaction gap that was fixed with a safe parse error. |
| 2026-05-22 | DWI-050 implementation gate | `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `cargo nextest run -p taru-api admin_acquisition --no-fail-fast`; `cargo nextest run -p taru-server admin_v1_acquisition_intake --no-fail-fast`; `cargo nextest run -p taru-server http::tests::system --no-fail-fast`; `cargo nextest run -p taru-server acquisition_intake --no-fail-fast`; `npm run check` from `apps/admin-web`; `npm test` from `apps/admin-web`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Admin contract tests passed: 5 passed, 45 skipped. Admin acquisition DTO tests passed: 1 passed, 49 skipped. Admin acquisition HTTP tests passed: 2 passed, 233 skipped. System HTTP tests passed: 19 passed, 216 skipped. Acquisition intake focused tests passed: 6 passed, 229 skipped. Admin web typecheck passed and Admin web tests passed: 3 files, 10 tests. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. DWI-050 added Admin-only intake candidate list diagnostics, watch-folder discovery command route, generated Admin TypeScript contract sync, typed Admin web client/mocks/data-source/UI support, bearer-auth coverage, and redaction coverage for raw paths, raw root URI parse failures, secret query strings, source locators, downloader internals, and Public Client API separation. |

## Redaction Checklist

Every implementation task must prove Admin or operator-facing diagnostics do not
expose:

- raw host filesystem paths;
- raw Source Locators unless explicitly scoped and redacted as Admin-only
  evidence;
- downloader credentials, cookies, tokens, or authorization headers;
- secret query strings from operator-submitted URLs;
- private environment variable values;
- raw provider/addon/downloader response bodies;
- unbounded directory listings.

## Notes

Do not use this lane to smuggle downloader protocols into core Taru. Protocol
adapters, Addon external fetches, and AI-generated artifacts should submit
candidates or Taru-Managed Artifacts into this boundary after it is proven.
