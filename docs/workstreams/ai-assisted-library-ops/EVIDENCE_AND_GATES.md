# AI Assisted Library Ops — Evidence And Gates

Status: Active
Last updated: 2026-05-22

## Expected Gates

Use focused gates for each task, then broaden before closeout.

```powershell
cargo nextest run -p taru-db automation --no-fail-fast
cargo nextest run -p taru-automation --no-fail-fast
cargo nextest run -p taru-api admin_contract --no-fail-fast
cargo nextest run -p taru-server http::tests::system --no-fail-fast
cargo fmt --all -- --check
npm run check # from apps/admin-web, after Admin contract/client changes
git diff --check
git diff --name-only -- crates/taru-client-protocol
```

For planning-only changes, validate JSON and diff hygiene:

```powershell
python -m json.tool docs/workstreams/ai-assisted-library-ops/WORKSTREAM.json
python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `CONTEXT.md`
- `docs/adr/0004-ai-as-external-automation-first.md`
- `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
- `docs/adr/0027-admin-api-boundary-for-web-console.md`
- `docs/workstreams/addons-automation`
- `docs/workstreams/post-rpd-product-hardening/DESIGN.md`
- `crates/taru-core/src/automation.rs`
- `crates/taru-automation/src/lib.rs`
- `crates/taru-server/src/app/automation.rs`
- `crates/taru-api/src/admin.rs`
- `apps/admin-web/src/adminApi`

## Evidence Log

| Date | Task | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-22 | AILO-010 | `docs/workstreams/ai-assisted-library-ops/DESIGN.md`; `python -m json.tool docs/workstreams/ai-assisted-library-ops/WORKSTREAM.json`; `python -m json.tool docs/workstreams/post-rpd-product-hardening/WORKSTREAM.json`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. Scope is Generated Artifact proposal/acceptance first. Local model runtime, embeddings/vector DB, provider-specific adapters, Addon distribution, protocol downloaders, autonomous writes, and Public Client API changes are out of scope. `git diff --check` emitted only repository CRLF conversion warnings for the unrelated `sdk/kotlin` working-tree change. |

## Redaction Checklist

Every implementation task must prove Admin/operator diagnostics do not expose:

- provider API keys, bearer tokens, or resolved secrets;
- raw prompts or prompt templates containing private library data;
- raw generated output or provider raw responses unless explicitly summarized;
- raw Source Locators, local paths, storage/cache URIs, or host paths;
- downloader/client credentials or tunnel/network secrets;
- unbounded provider payloads, traces, logs, or stack errors.

## Notes

Do not use this lane to ship a local model runtime, vector database, provider
marketplace, or Addon distribution. Those are follow-ons after generated
artifact proposal/acceptance semantics are proven.
