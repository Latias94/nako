# Network Access Boundary — Evidence And Gates

Status: Complete
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo nextest run -p nako-server config --no-fail-fast
cargo nextest run -p nako-server http::tests::system --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo fmt --all -- --check
npm run check # from apps/admin-web, after Admin contract/client changes
git diff --check
git diff --name-only -- crates/nako-client-protocol
```

For planning-only changes, validate JSON and diff hygiene:

```powershell
python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `docs/adr/0024-inbound-token-authentication-boundary.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/deployment/SELF_HOSTED.md`
- `docs/workstreams/access-boundary-auth/DESIGN.md`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `crates/nako-server/src/config.rs`
- `crates/nako-server/src/http`
- `crates/nako-api/src/admin.rs`
- `crates/nako-api/src/admin_contract.rs`
- `apps/admin-web/src/adminApi`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | NAB-010 | `python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check` | Pass. Scope is network access policy/readiness first; built-in NAT traversal runtime, downloader protocols, AI writes, Addon runtime, identity/RBAC, and library mutation are out of scope. `git diff --check` emitted only repository CRLF conversion warnings for the unrelated `sdk/kotlin` working-tree change. |
| 2026-05-22 | NAB-020 red gate | `cargo nextest run -p nako-server config_preflight_rejects_reverse_proxy_policy_without_external_base_url --no-fail-fast` | Expected fail. The test could not compile because `NakoServerConfig` had no `network` policy field and no `NetworkExposureMode`, `NetworkAccessConfig`, or tunnel-provider config types. |
| 2026-05-22 | NAB-020 implementation gate | `cargo nextest run -p nako-server config --no-fail-fast`; `cargo nextest run -p nako-server http::tests::system --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `python -m json.tool docs/workstreams/downloads-watch-folder-intake/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Server config scope passed during NAB-020: 36 passed, 203 skipped. HTTP system regression passed during NAB-020: 19 passed, 220 skipped. Formatting and workstream JSON validation passed. Public Client protocol had no changed files. `git diff --check` emitted only repository CRLF conversion warnings. NAB-020 added `NetworkAccessConfig`, `NetworkExposureMode`, tunnel-provider declarations, config-check validation for reverse-proxy/private-network/tunnel-provider modes, trusted proxy source requirements, explicit browser origin policy, tunnel token-env checks, deployment examples, and redacted diagnostics without starting a tunnel runtime or changing Public Client API. |
| 2026-05-22 | NAB-030 red gate | `cargo nextest run -p nako-server network_boundary_ --no-fail-fast` | Expected fail before implementation. The new HTTP boundary tests proved disallowed origins and forwarded headers were not yet enforced at request time. |
| 2026-05-22 | NAB-030 focused gate | `cargo nextest run -p nako-server network_boundary_ --no-fail-fast` | Pass. 2 passed, 240 skipped in the latest focused run. The HTTP boundary preserves bearer-auth precedence for unauthenticated protected requests, returns redacted forbidden errors for authenticated disallowed origins, allows configured origins and CORS preflight, keeps `/health` public, ignores forwarded host/proto unless proxy headers are enabled and the remote source matches exact-IP or CIDR policy, and rejects malformed multi-hop forwarded host values. |
| 2026-05-22 | NAB-030 implementation gate | `cargo nextest run -p nako-server http::tests::system --no-fail-fast` | Pass. 21 passed, 222 skipped in the latest implementation run. Existing Admin/system route behavior, auth protection, redaction, public health, acquisition diagnostics, playback diagnostics, and new network-boundary tests all pass together. |
| 2026-05-22 | NAB-030 config regression gate | `cargo nextest run -p nako-server config --no-fail-fast` | Pass. 38 passed, 205 skipped in the latest config regression run. Network policy config validation remains redaction-safe and now rejects non-HTTPS reverse-proxy/tunnel public URLs plus path-bearing/credential-bearing browser origins. |
| 2026-05-22 | NAB-040 focused gates | `cargo nextest run -p nako-api admin_network_access_diagnostics_serializes_readiness_without_secret_urls --no-fail-fast`; `cargo nextest run -p nako-server admin_v1_system_config_reports_sanitized_configuration --no-fail-fast` | Pass. API DTO serialization and server Admin system-config diagnostics expose typed network readiness, external endpoint scheme/fingerprint only, trusted proxy/source counts, origin counts, tunnel provider declaration state, and token presence without exposing raw URLs, hostnames, query secrets, forwarded headers, local paths, or credential values. |
| 2026-05-22 | NAB-040 contract/admin-web gate | `cargo nextest run -p nako-api admin_contract --no-fail-fast`; `npm run check` from `apps/admin-web`; `npm run test` from `apps/admin-web` | Pass. Admin TypeScript contract includes `AdminNetworkAccessDiagnostics` inside the Admin-only system-config response, generated Admin Web contract is synchronized, typed Admin Web data mapping renders network readiness, Admin Web tests passed 10/10, and the generated Admin contract still excludes generated fetch runtime and Public Client route inventory. |
| 2026-05-22 | NAB-040 implementation gate | `cargo nextest run -p nako-server http::tests::system --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol`; `python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json` | Pass. HTTP/Admin system regressions passed: 21 passed, 222 skipped. Formatting passed. Public Client protocol boundary check returned no changed files. Workstream and parent umbrella JSON validated. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated Kotlin SDK working-tree file. |
| 2026-05-22 | NAB-050 closeout | `python -m json.tool docs/workstreams/network-access-boundary/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/nako-client-protocol` | Pass. Network Access Boundary is closed with policy/config, HTTP enforcement, and Admin readiness diagnostics complete. Built-in NAT traversal runtime, endpoint discovery, identity/RBAC, protocol downloaders, AI-assisted library ops, and Addon runtime/distribution are split follow-ons. `git diff --check` emitted only repository CRLF conversion warnings, including the unrelated Kotlin SDK working-tree file. |

## Redaction Checklist

Every implementation task must prove Admin/operator diagnostics do not expose:

- bearer tokens or authorization headers;
- tunnel-provider credentials, refresh tokens, cookies, or shared secrets;
- raw forwarded headers or untrusted client-supplied host/proto data;
- secret-bearing URLs or query strings;
- local filesystem paths;
- private environment variable values;
- unbounded internal network inventory.

## Notes

Do not use this lane to ship a built-in NAT traversal runtime. Concrete tunnel
providers, relay services, identity/RBAC, endpoint discovery for public clients,
and sharing semantics are follow-ons unless explicitly opened.
