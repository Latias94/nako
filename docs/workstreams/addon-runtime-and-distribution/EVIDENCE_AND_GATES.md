# Addon Runtime And Distribution — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo nextest run -p taru-addon-protocol --no-fail-fast
cargo nextest run -p taru-addon-client --no-fail-fast
cargo nextest run -p taru-db addon --no-fail-fast
cargo nextest run -p taru-server addons --no-fail-fast
cargo nextest run -p taru-api admin_contract --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo fmt --all -- --check
npm run check # from apps/admin-web, after Admin contract/client changes
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

For planning-only changes, validate JSON and diff hygiene:

```powershell
python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0003-http-addons-before-in-process-plugins.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `docs/adr/0022-keep-public-protocol-crates-permissive-while-server-crates-remain-agpl.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/addon-architecture-deepening`
- `docs/workstreams/admin-addon-operations-mvp`
- `docs/workstreams/ai-assisted-library-ops`
- `docs/workstreams/downloads-watch-folder-intake`
- `crates/taru-addon-protocol/src/lib.rs`
- `crates/taru-addon-client`
- `crates/taru-core/src/addon.rs`
- `crates/taru-server/src/http/addons.rs`
- `crates/taru-api/src/admin.rs`
- `apps/admin-web/src/adminApi`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | ARD-010 | `docs/workstreams/addon-runtime-and-distribution/DESIGN.md`; `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Scope is Addon Sidecar package/install/runtime readiness first. Addon Manager automation, marketplace, package signing, process supervision, Native Plugin ABI, direct library writes, Public Client API changes, and `taru-client-protocol` changes are out of scope. `git diff --check` emitted only repository CRLF conversion warnings for the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | ARD-020 protocol slice | `cargo nextest run -p taru-addon-protocol install_descriptor --no-fail-fast`; `cargo nextest run -p taru-addon-protocol --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Added package/install descriptor protocol DTOs, one-of runtime reference validation, local-path/credential rejection without echoing rejected values, Secret Reference binding validation that accepts explicit reference schemes, and redacted install guide generation. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated `sdk/kotlin` working-tree change. ARD-020 is not complete yet: Admin DTO/server preview route tests remain. |

## Redaction Checklist

Every implementation task must prove Admin/operator diagnostics do not expose:

- administrator bearer tokens or Addon Token values;
- resolved Secret Reference values or provider credentials;
- raw local filesystem paths, Source Locators, storage/cache URIs, or host
  paths;
- credential-bearing image URLs, compose snippets, environment files, or command
  lines;
- raw sidecar response payloads, raw network errors, or stack traces;
- unbounded package metadata or manifest payloads;
- downloader/client credentials, tunnel/network secrets, or private relay URLs.

## Notes

Do not use this lane to ship an Addon Manager, marketplace, package signing
trust root, process/container supervisor, Native Plugin ABI, or Jellyfin Plugin
Compatibility. Those are follow-ons after sidecar package/runtime readiness is
proven.
