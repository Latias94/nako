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
