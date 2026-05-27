# Playback Capability Profile Planner - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Focused Gates

Run after planner-only changes:

```powershell
cargo nextest run -p nako-playback playback --no-fail-fast
```

Run after server adapter changes:

```powershell
cargo nextest run -p nako-server playback --no-fail-fast
```

Run before commit or closeout:

```powershell
cargo nextest run -p nako-playback --no-fail-fast
cargo nextest run -p nako-server playback --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

## Evidence Log

| Date | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-27 | Workstream opened | Pending | ADR and docs created before code changes. |
| 2026-05-27 | `cargo nextest run -p nako-playback --no-fail-fast` | Passed | 17 planner/profile tests passed after profile-driven planner and `capability.rs` split. |
| 2026-05-27 | `cargo nextest run -p nako-client-protocol --no-fail-fast` | Passed | 12 protocol tests passed after adding safe decision report DTOs. |
| 2026-05-27 | `cargo nextest run -p nako-api --no-fail-fast` | Passed | 61 API/OpenAPI/SDK tests passed after refreshing Kotlin and TypeScript SDK outputs. |
| 2026-05-27 | `cargo nextest run -p nako-server playback_decision_and_direct_stream_routes_work playback_decision_returns_safe_target_and_policy_denial --no-fail-fast` | Passed | HTTP playback decision report contract and policy denial report verified. |
| 2026-05-27 | `cargo check -p nako-client`, `cargo check -p nako-client-core`, `cargo check -p nako-client-uniffi` | Passed | Public client consumers compile with the new decision report DTO. |
| 2026-05-27 | `cargo nextest run -p nako-playback --no-fail-fast` | Passed | Final gate: 17 planner/profile tests passed after formatting. |
| 2026-05-27 | `cargo nextest run -p nako-server playback --no-fail-fast` | Passed | Final gate: 81 playback tests passed. |
| 2026-05-27 | `cargo fmt --all -- --check` | Passed | Final formatting gate passed. |
| 2026-05-27 | `git diff --check` | Passed | Final whitespace gate passed; Git emitted CRLF normalization warnings only. |

## Manual Review Anchors

- Planner profile model: `crates/nako-playback/src/lib.rs`
- Capability evaluation model: `crates/nako-playback/src/capability.rs`
- Server playback adapter: `crates/nako-server/src/app/playback/`
- Public DTO mapping: `crates/nako-client-protocol/`, `crates/nako-api/`
- ADR: `docs/adr/0044-playback-capability-profile-planner.md`

## Non-Gates

The following are intentionally not required for this lane:

- full workspace test run;
- frontend browser verification;
- hardware smoke tests on a real GPU;
- adaptive HLS playback verification;
- DLNA or Chromecast device playback verification.
