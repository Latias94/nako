# Error Handling

Playback planning should usually return a typed decision report instead of
throwing an error.

## Required Patterns

- Incompatible playback should produce `PlaybackMode::Denied`,
  `PlaybackDenial`, and `PlaybackDecisionReport` reasons where possible.
- Capability failures should use `PlaybackCompatibilityCondition` values such
  as `ContainerUnsupported`, `VideoCodecUnsupported`, `AudioChannelsUnsupported`,
  `VideoHdrUnsupported`, `SubtitleDeliveryUnsupported`, or `PolicyDenied`.
- Missing media technical facts should be represented as
  `MediaTechnicalFactsMissing` or a specific compatibility condition, not a
  panic.
- Runtime failures after planning belong to `nako-server`/`nako-transcode`.

## Validation Matrix

| Condition | Planner behavior |
|-----------|------------------|
| Policy denies all modes | `Denied` with `PolicyDenied` |
| Direct Play disabled by client | Direct Play unsupported, consider Remux/Transcode |
| Container unknown | Direct Play/Remux unsupported by container condition |
| Requested HLS output | Transcode path with HLS output requirement |
| HDR unsupported | color pipeline/tone mapping requirement or unsupported condition |

## Wrong vs Correct

### Wrong

```rust
panic!("client does not support this codec");
```

### Correct

```rust
PlaybackCapabilityEvaluation::unsupported(vec![
    PlaybackCompatibilityCondition::VideoCodecUnsupported,
])
```

## Evidence

- `crates/nako-playback/src/capability.rs`
- `crates/nako-playback/src/lib.rs`
