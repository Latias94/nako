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

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | DWI-010 | `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Scope is acquisition intake/watch-folder discovery only; first implementation task is durable intake candidate domain. `git diff --check` emitted only repository CRLF conversion warnings. |
| 2026-05-22 | DWI-020 red gate | `cargo nextest run -p taru-db acquisition_intake --no-fail-fast` | Expected fail. New backend-neutral contract could not compile because `AcquisitionIntakeRepository` was not implemented for SQLite/PostgreSQL/facade yet. |
| 2026-05-22 | DWI-020 implementation gate | `cargo nextest run -p taru-db acquisition_intake --no-fail-fast`; `cargo check -p taru-db --tests`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. SQLite acquisition intake contract passed: 1 passed, 123 skipped. `cargo check -p taru-db --tests` passed. Formatting passed. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. DWI-020 added core intake candidate records, repository traits, SQLite/PostgreSQL migrations/adapters, facade dispatch, backend capabilities, and contract coverage without Media Source or Library File Write behavior. |

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
