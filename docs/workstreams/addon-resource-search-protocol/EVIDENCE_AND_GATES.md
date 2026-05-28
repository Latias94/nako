# Addon Resource Search Protocol - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Smallest Current Repro

```bash
cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast
```

Before ARSP-020 lands this is expected to run zero tests or fail because the
contract does not exist yet. After ARSP-020, it proves protocol vocabulary,
serde shape, and manifest validation behavior.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-addon-protocol resource_search --no-fail-fast
cargo nextest run -p nako-addon-client resource_search --no-fail-fast
cargo nextest run -p nako-server addon_resource_search --no-fail-fast
```

### Package Gates

```bash
cargo nextest run -p nako-addon-protocol --no-fail-fast
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-server addon --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-server --tests
git diff --check
```

Run broader `cargo nextest run --workspace --no-fail-fast` only if the lane
touches shared storage or cross-cutting server behavior beyond addon/resource
contracts.

### Review Gate

Run `review-workstream` before accepting completed implementation slices.
Run `verify-rust-workstream` before marking the lane complete.

## Evidence Anchors

- `docs/workstreams/addon-resource-search-protocol/DESIGN.md`
- `docs/workstreams/addon-resource-search-protocol/TODO.md`
- `docs/workstreams/addon-resource-search-protocol/MILESTONES.md`
- `crates/nako-addon-protocol/src/lib.rs`
- `crates/nako-addon-client/src/lib.rs`
- `crates/nako-core/src/acquisition_intake.rs`
- `crates/nako-server/src/app/addons.rs`
- `F:/SourceCodes/Rust/nako-official-addons/docs/workstreams/official-resource-search-architecture-hardening/PROTOCOL_PROPOSAL.md`

## Evidence Log

### 2026-05-28 - ARSP-010

- Opened workstream from the hardened official resource-search addon proposal.
- Froze read-only search scope, non-goals, and acquisition handoff split.
- No code changes yet.

## Notes

Search is not acquisition. The base protocol must not imply candidate writes,
link checks, downloader execution, cloud-drive save, stream URL read, or
playback authority.
