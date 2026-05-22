# Addon Runtime And Distribution — Evidence And Gates

Status: Complete
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-db addon --no-fail-fast
cargo nextest run -p nako-server addons --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo fmt --all -- --check
npm run check # from apps/admin-web, after Admin contract/client changes
git diff --check
git diff --name-only -- crates/nako-client-protocol
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
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client`
- `crates/nako-core/src/addon.rs`
- `crates/nako-server/src/http/addons.rs`
- `crates/nako-api/src/admin.rs`
- `apps/admin-web/src/adminApi`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | ARD-010 | `docs/workstreams/addon-runtime-and-distribution/DESIGN.md`; `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Scope is Addon Sidecar package/install/runtime readiness first. Addon Manager automation, marketplace, package signing, process supervision, Native Plugin ABI, direct library writes, Public Client API changes, and `nako-client-protocol` changes are out of scope. `git diff --check` emitted only repository CRLF conversion warnings for the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | ARD-020 protocol slice | `cargo nextest run -p nako-addon-protocol install_descriptor --no-fail-fast`; `cargo nextest run -p nako-addon-protocol --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Added package/install descriptor protocol DTOs, one-of runtime reference validation, local-path/credential rejection without echoing rejected values, Secret Reference binding validation that accepts explicit reference schemes, and redacted install guide generation. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated `sdk/kotlin` working-tree change. ARD-020 is not complete yet: Admin DTO/server preview route tests remain. |
| 2026-05-22 | ARD-020 Admin preview | `cargo nextest run -p nako-server admin_addon_install_guide_preview --no-fail-fast`; `cargo nextest run -p nako-server register_addon --no-fail-fast`; `cargo nextest run -p nako-server addons --no-fail-fast`; `cargo nextest run -p nako-api --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Added Admin request/response DTOs, app validation, and `POST /admin/v1/addons/install-guide-preview`. HTTP tests prove preview success redacts raw secret values, admin/Add-on token strings, and local paths while exposing safe runtime/manifest/Secret Reference facts; invalid local runtime references, invalid manifest paths, and raw secret bindings return safe `400` messages without echoing rejected values. `nako-client-protocol` diff is empty. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | ARD-030 runtime readiness | `cargo check -p nako-api -p nako-server --tests`; `cargo nextest run -p nako-server admin_addon_runtime_readiness --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo nextest run -p nako-server addons --no-fail-fast`; `npm run check`; `npm test -- src/adminApi/client.test.ts`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Added Admin-only `POST /admin/v1/addons/{addon_id}/runtime-readiness`, DTOs, generated Admin Web contract/client support, and route tests. Diagnostics classify ready sidecars, degraded sidecars, missing grants, missing Secret Reference configuration, network policy blockers, protocol mismatch, manifest mismatch, and unsafe sidecar responses using typed statuses and safe error codes. Tests prove no admin token, Addon Token, raw network error, raw sidecar payload, credential-bearing URL, localhost URL, or secret field/value is echoed. `nako-client-protocol` diff is empty. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | ARD-040 declared routing | `cargo check -p nako-api -p nako-server --tests`; `cargo nextest run -p nako-db addon --no-fail-fast`; `cargo nextest run -p nako-server admin_addon_routing_plans --no-fail-fast`; `cargo nextest run -p nako-server addons --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `npm run check`; `npm test -- src/adminApi/client.test.ts`; `cargo fmt --all -- --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Added durable `addon_routing_plans` records/migrations for SQLite and PostgreSQL, `JobKind::AddonTask`, routing-plan domain records, Admin `POST /admin/v1/addons/{addon_id}/routing-plans`, generated Admin Web contract/client support, and HTTP/DB contract tests. Plans are executable only when the addon is enabled, required grants are present, and event kinds are supported; disabled addons, missing grants, and unsupported event kinds are deferred with safe reason codes. Tests prove idempotent manifest-declaration replacement, stale-plan removal, no hidden scheduler/outbox side effects, redacted manifest diagnostics, and no `nako-client-protocol` diff. |
| 2026-05-22 | ARD-050 artifact/intake handoff | `cargo check -p nako-api -p nako-server --tests`; `cargo nextest run -p nako-server addon_generated_artifact_handoff --no-fail-fast`; `cargo nextest run -p nako-server addon_acquisition_candidate_handoff --no-fail-fast`; `cargo nextest run -p nako-server addon_handoff --no-fail-fast`; `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Added Addon Token routes `POST /addon/v1/generated-artifacts` and `POST /addon/v1/acquisition/intake/candidates`, public extension DTOs, app-service handoff logic, and focused HTTP/app tests. Generated Artifact submissions are scoped by Addon grants, revalidate library/item/source ownership, upsert an Addon-backed Automation Provider without overwriting prior capabilities, enqueue an Addon Task job, and create Proposed AILO automation artifacts with replay/conflict idempotency. Acquisition candidates enter DWI `AddonProposed` intake records with redacted source diagnostics. Tests prove replay behavior, stale-target rejection, missing-scope rejection, no raw prompt/payload/token/source leaks, no Canonical Metadata or NFO sidecar mutation, no Media Source creation, no Managed Import artifact creation, no promotion/apply authority, and no `nako-client-protocol` diff. Broader `cargo nextest run -p nako-server addons --no-fail-fast` and retry with `-j 1` were attempted but blocked by Windows socket error 10055 after the ARD-050 focused tests passed, so they are recorded as environment-limited attempts rather than pass evidence. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | ARD-060 closeout | `python -m json.tool docs/workstreams/addon-runtime-and-distribution/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol`; `cargo fmt --all -- --check` | Pass. Addon Runtime And Distribution is closed with package/install descriptor validation, redacted install-guide preview, Admin-only runtime readiness diagnostics, declared task/event routing plans, and AILO/DWI handoff complete. Addon Manager discovery/install/update, marketplace hosting, package signing trust roots, process/container supervision, logs/rollback, Native Plugin ABI, downloader protocol adapters, local AI/model runtime, Public Client surfaces, direct library writes, hidden schedulers, and `nako-client-protocol` changes are split follow-ons. The broader `nako-server addons` gate remains documented as environment-limited by Windows socket error 10055; closeout relies on the focused ARD-050 gates plus JSON/diff/protocol/format hygiene. |

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
