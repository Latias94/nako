# Metadata Provider Breadth — Evidence And Gates

Status: Completed
Last updated: 2026-05-21

## Gate Set

### Targeted Iteration Gates

```powershell
cargo nextest run -p nako-metadata registry --no-fail-fast
cargo nextest run -p nako-metadata matching --no-fail-fast
cargo nextest run -p nako-metadata refresh --no-fail-fast
cargo nextest run -p nako-server metadata_diagnostics --no-fail-fast
```

### Package Gates

```powershell
cargo nextest run -p nako-metadata --no-fail-fast
cargo nextest run -p nako-server metadata --no-fail-fast
```

### Broader Closeout Gate

```powershell
cargo fmt --all -- --check
cargo check --workspace --tests
cargo nextest run -p nako-metadata --no-fail-fast
cargo nextest run -p nako-server metadata --no-fail-fast
git diff --check
```

Use a narrower closeout gate only if the workspace gate becomes too slow, and
record the reason.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Run
`verify-rust-workstream` before marking the lane complete.

## Evidence Log

| Date | Scope | Command / Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-21 | MPB-010 planning | Reviewed metadata provider runtime, registry, strategy, diagnostics API, provider tests, `CONTEXT.md`, and relevant ADRs | Pass. First safe slice is provider capability diagnostics. |
| 2026-05-21 | MPB-020 red gate | `cargo nextest run -p nako-metadata registry --no-fail-fast` before capability model | Failed as expected. Registry/provider diagnostics had no capability field yet. |
| 2026-05-21 | MPB-020 targeted | `cargo nextest run -p nako-metadata registry --no-fail-fast` | Pass. Registry diagnostics include provider capabilities. |
| 2026-05-21 | MPB-020 targeted | `cargo nextest run -p nako-server metadata_diagnostics --no-fail-fast` | Pass. `/metadata/providers` exposes capabilities without secrets. |
| 2026-05-21 | MPB-030 targeted | `cargo nextest run -p nako-metadata matching --no-fail-fast` | Pass. Matching policy covers accepted, needs-confirmation, rejected, and high-confidence conflict decisions. |
| 2026-05-21 | MPB-040 red gate | `cargo nextest run -p nako-metadata refresh_search_requires_confirmation --no-fail-fast` before refresh integration | Failed as expected. Ambiguous search candidate still fetched and committed. |
| 2026-05-21 | MPB-040 targeted | `cargo nextest run -p nako-metadata refresh --no-fail-fast` | Pass. External-ID refresh remains compatible; ambiguous search does not fetch, cache raw response, create mapping, or mutate canonical metadata. |
| 2026-05-21 | MPB-050 targeted | `cargo nextest run -p nako-metadata conflict --no-fail-fast` | Pass. Cross-provider candidate review reports manual-confirmation conflicts and all-rejected outcomes. |
| 2026-05-21 | MPB-050 targeted | `cargo nextest run -p nako-server metadata_candidate_review --no-fail-fast` | Pass. `/items/{item_id}/metadata/candidates` returns reviewable provider decisions and leaves canonical state untouched. |
| 2026-05-21 | MPB package | `cargo nextest run -p nako-api --no-fail-fast` | Pass. API DTO additions compile and existing API contract tests remain green. |
| 2026-05-21 | MPB package | `cargo nextest run -p nako-metadata --no-fail-fast` | Pass. 36 metadata tests passed. |
| 2026-05-21 | MPB package | `cargo nextest run -p nako-server metadata --no-fail-fast` | Pass. 21 server metadata tests passed. |
| 2026-05-21 | MPB workspace check | `cargo check --workspace --tests` | Pass. Workspace test targets compile. Cargo briefly waited for the package cache lock, then finished successfully. |
| 2026-05-21 | MPB formatting | `cargo fmt --all -- --check` | Pass. |
| 2026-05-21 | MPB diff hygiene | `git diff --check` | Pass. Git reported CRLF conversion warnings only. |
| 2026-05-21 | MPB closeout review | `review-workstream` closeout audit against DESIGN/TODO/EVIDENCE/HANDOFF and current diff | Pass. No blocking compliance or code-quality findings; durable candidate review queue, query ergonomics, and deeper provider precision remain split follow-ons. |

## Evidence Anchors

- `crates/nako-metadata/src/types.rs`
- `crates/nako-metadata/src/matching.rs`
- `crates/nako-metadata/src/registry.rs`
- `crates/nako-metadata/src/strategy.rs`
- `crates/nako-metadata/src/providers/{tmdb,bangumi,douban}.rs`
- `crates/nako-api/src/metadata_diagnostics.rs`
- `crates/nako-server/src/app/metadata_runtime.rs`
- `crates/nako-server/src/app/metadata.rs`
- `crates/nako-server/src/http/metadata.rs`
- `crates/nako-server/src/http/tests/metadata.rs`

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete.
