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
- 2026-05-27: PTP-040 promoted Public Client playback protocol types without
  adopting Jellyfin/DLNA profile breadth. `ClientPlaybackDecision.reason` is now
  `ClientPlaybackDecisionReason` instead of free text, playback sessions reuse
  `ClientPlaybackCapabilitiesDto`, and the API adapter maps internal planner
  reasons to safe protocol wire values. TypeScript and Kotlin SDK outputs were
  regenerated.
  Verified:
  - `cargo nextest run -p nako-client-protocol public --no-fail-fast` passed:
    10 passed, 0 skipped.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
    passed: 20 passed, 39 skipped.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 66
    passed, 282 skipped.
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 35 passed,
    0 skipped.
  - `cargo fmt --all -- --check` passed.
- 2026-05-27: PTP-050 replaced the shallow HLS hardware field with
  `TranscodeExecutionPolicy`. HLS profile identity and FFmpeg HLS request
  planning now carry a decode/filter/encode `TranscodeAccelerationPlan`,
  fallback evidence, output constraints, and subtitle strategy. The FFmpeg
  adapter still owns FFmpeg encoder/filter strings, applies HLS bitrate
  constraints, and rejects unimplemented subtitle strategies instead of hiding
  policy gaps. Public Client transcode plans no longer expose service-side
  hardware selection; TypeScript and Kotlin SDK outputs were regenerated.
  Verified:
  - `cargo check -p nako-transcode -p nako-playback -p nako-client-protocol -p nako-api -p nako-server`
    passed with pre-existing `nako-server` warnings.
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 36 passed,
    0 skipped.
  - `cargo nextest run -p nako-playback --no-fail-fast` passed: 7 passed,
    0 skipped.
  - `cargo nextest run -p nako-server -E 'test(hls_source_uses_selected_cpu_acceleration_when_gpu_falls_back) | test(hls_source_request_identity_separates_selected_acceleration_profiles)' --no-fail-fast`
    passed: 2 passed, 346 skipped.
  - `cargo nextest run -p nako-client-protocol public --no-fail-fast` passed:
    10 passed, 0 skipped.
  - `cargo nextest run -p nako-api -E 'test(public_openapi) | test(sdk) | test(admin_contract)' --no-fail-fast`
    passed: 20 passed, 39 skipped.
  - `cargo nextest run -p nako-server playback --no-fail-fast` passed: 66
    passed, 282 skipped.
  - `cargo fmt --all -- --check` passed.
  - `python -m json.tool docs/workstreams/playback-transcode-policy-deepening/WORKSTREAM.json`
    passed.
  - `git diff --check` passed with Git line-ending warnings only.
- 2026-05-27: PTP-060 added the playback runtime inventory and engine adapter
  seam. `TranscodeRuntimeInventory` summarizes FFmpeg CLI runtime capability
  status without raw host paths or commands, and FFmpeg remux/HLS runners now
  implement `TranscodeEngineAdapter` with typed start outcomes and progress
  snapshots. Server playback orchestration calls the engine adapter rather than
  route-shaped runner APIs, and Admin runtime diagnostics consume the inventory
  summary.
  Verified:
  - `cargo nextest run -p nako-transcode --no-fail-fast` passed: 39 passed,
    0 skipped.
  - `cargo nextest run -p nako-server -E 'test(playback) | test(admin_v1_playback_runtime)' --no-fail-fast`
    passed: 66 passed, 282 skipped.
