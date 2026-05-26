# Playback Transcode Policy Deepening - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Planned Gates

Run focused gates first, then broaden when a slice touches shared playback,
protocol, or storage behavior.

### Documentation

```powershell
python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json
git diff --check -- docs/workstreams/playback-transcode-policy-deepening docs/workstreams/README.md docs/adr
```

### Playback Characterization

```powershell
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
```

### Public/Admin Contract

```powershell
cargo nextest run -p nako-client-protocol public --no-fail-fast
cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast
```

### Runtime And Storage

```powershell
cargo nextest run -p nako-transcode --no-fail-fast
cargo nextest run -p nako-server -E 'test(playback) | test(admin_v1_playback_runtime)' --no-fail-fast
cargo nextest run -p nako-db --no-fail-fast
```

### Closeout

```powershell
cargo fmt --all -- --check
cargo nextest run -p nako-server playback --no-fail-fast
cargo nextest run -p nako-transcode --no-fail-fast
git diff --check
python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json
```

## Evidence Log

- 2026-05-27: PTP-010 opened the lane and recorded ADR 0038 after reviewing
  Jellyfin playback/transcode feature pressure from Device Profile,
  PlaybackInfo/StreamInfo, Transcode Reasons, Encoding Options, Transcode
  Manager, and session progress/cleanup concepts.
  Verified:
  - `python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json`
    passed.
  - `git diff --check -- docs/workstreams/playback-transcode-policy-deepening docs/workstreams/README.md docs/adr`
    passed with Git line-ending warnings only for tracked index files.
- 2026-05-27: PTP-020 characterized the critical Playback Session invariant
  before planner refactoring: direct play creates a durable Playback Session
  with mode `direct` and client capability evidence, but creates no Transcode
  Session artifact. Existing remux/HLS/browser-ticket/redaction/hardware
  fallback coverage was refreshed.
  Verified:
  - `cargo nextest run -p nako-server direct_stream_route_records_playback_session_without_transcode_artifact --no-fail-fast`
    passed: 1 passed, 347 skipped.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 66
    passed, 282 skipped.
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 35 passed,
    0 skipped.
- 2026-05-27: PTP-030 extracted playback planning from `nako-streaming` into
  `nako-playback`. The new crate owns planner/profile/capability/decision
  records and typed decision reasons; `nako-streaming` now stays focused on
  direct byte-range response planning. Server playback app code calls
  `PlaybackPlanner`; public DTO adapters convert internal typed reasons into
  safe client strings until PTP-040 promotes stable protocol reason shapes.
  Verified:
  - `cargo check -p nako-playback -p nako-streaming` passed.
  - `cargo check -p nako-api -p nako-server` passed with pre-existing warnings.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 7 passed,
    0 skipped.
  - `cargo nextest run -p nako-streaming --no-fail-fast` passed: 3 passed,
    0 skipped.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 66
    passed, 282 skipped.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
    passed: 19 passed, 39 skipped.
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 35 passed,
    0 skipped.
  - `cargo fmt --all -- --check` passed.
  - `git diff --check` passed with Git line-ending warnings only.
