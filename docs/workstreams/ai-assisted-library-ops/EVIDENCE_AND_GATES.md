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
| 2026-05-22 | AILO-020 | `cargo nextest run -p taru-db generated_artifact --no-fail-fast` | Pass: 2 tests. Proves Generated Artifact proposal queue returns stable target/provenance/payload/readiness summaries, redacts raw prompts/raw output/source fingerprints from proposal JSON, and marks mismatched source/item evidence stale. |
| 2026-05-22 | AILO-020 | `cargo nextest run -p taru-db automation --no-fail-fast` | Pass: 4 tests. Existing automation provider/artifact contract remains green after proposal repository extension. |
| 2026-05-22 | AILO-020 | `cargo nextest run -p taru-automation --no-fail-fast` | Pass: 3 tests. Automation runner still persists proposed artifacts, rejects canonical mutation, retries provider failures, and now exposes ready generated artifact proposal summaries without raw prompt/output/secret leakage. |
| 2026-05-22 | AILO-020 | `cargo nextest run -p taru-server automation --no-fail-fast` | Pass: 2 tests. App service lists generated artifact proposals without canonical metadata mutation or raw payload/source/secret leakage; existing automation HTTP job/provider test remains green. |
| 2026-05-22 | AILO-020 | `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. `git diff --check` emitted only CRLF conversion warnings, including the unrelated existing `sdk/kotlin` working-tree change. Public Client protocol untouched. |

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
