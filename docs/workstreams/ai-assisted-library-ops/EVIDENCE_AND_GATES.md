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
| 2026-05-22 | AILO-030 | `cargo nextest run -p taru-api admin_generated_artifact --no-fail-fast` | Pass: 1 test. Proves Admin Generated Artifact proposal DTOs expose target/provenance/payload/readiness summaries and omit raw prompt JSON, raw artifact JSON, source locators, source fingerprints, provider secret fields, local paths, and token-like values. |
| 2026-05-22 | AILO-030 | `cargo nextest run -p taru-api admin_contract --no-fail-fast` | Pass: 5 tests. Proves Admin TypeScript route constants include `generatedArtifactProposals`, generated Admin Web contract is synchronized, forbidden raw/sensitive fields stay out of the contract, Admin routes stay out of Public Client inventory, and public TypeScript SDK excludes Admin routes. |
| 2026-05-22 | AILO-030 | `cargo nextest run -p taru-server http::tests::system --no-fail-fast` | Pass: 22 tests. Proves `/admin/v1/automation/generated-artifacts/proposals` is read-only, Admin-only, paginated, redacted, and does not mutate canonical metadata while existing Admin system diagnostics remain green. |
| 2026-05-22 | AILO-030 | `npm run check` from `apps/admin-web`; `npm test` from `apps/admin-web` | Pass: TypeScript build and 10 Vitest tests. Proves typed Admin Web client/data-source/mocks consume `generatedArtifactProposals` and UI renders Generated Artifact summaries without raw prompt/artifact/path fields. |
| 2026-05-22 | AILO-030 | `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol`; `python -m json.tool docs/workstreams/ai-assisted-library-ops/WORKSTREAM.json` | Pass. `git diff --check` emitted only CRLF conversion warnings, including the unrelated existing `sdk/kotlin` working-tree change. Public Client protocol untouched; workstream JSON valid. |
| 2026-05-22 | AILO-040 | `cargo nextest run -p taru-server automation_app --no-fail-fast` | Pass: 3 tests. Proves metadata-cleanup Generated Artifact accept/reject planning is explicit, idempotent for replay, rejects stale acceptance, allows no-mutation rejection, and does not mutate canonical metadata or Media Source locators. |
| 2026-05-22 | AILO-040 | `cargo nextest run -p taru-api admin_generated_artifact_review --no-fail-fast` | Pass: 1 test. Proves Admin review responses expose acceptance boundary facts rather than raw prompt/artifact payload, local paths, or secret-like fields. |
| 2026-05-22 | AILO-040 | `cargo nextest run -p taru-server admin_v1_generated_artifact_review --no-fail-fast`; `cargo nextest run -p taru-server http::tests::system --no-fail-fast` | Pass: focused route test and 23 system tests. Proves Admin review-plan/review routes return redacted boundaries and accepted metadata-cleanup proposals still do not mutate canonical metadata. |
| 2026-05-22 | AILO-040 | `cargo nextest run -p taru-db automation --no-fail-fast`; `cargo nextest run -p taru-api admin_contract --no-fail-fast`; `npm run check` from `apps/admin-web` | Pass. Automation repository contract remains green after adding artifact lookup; Admin contract includes review-plan/review routes and generated Admin Web contract is synchronized; Admin Web TypeScript compiles. |
| 2026-05-22 | AILO-040 | `cargo fmt --all -- --check`; `git diff --check`; `git diff --name-only -- crates/taru-client-protocol` | Pass. `git diff --check` emitted only CRLF conversion warnings, including the unrelated existing `sdk/kotlin` working-tree change. Public Client protocol untouched. |

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
