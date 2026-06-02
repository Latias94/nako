# Error Handling

Probe errors should be typed through `nako-core::NakoError` so library
ingestion can classify and persist failures at the orchestration layer.

## Required Patterns

- Return `NakoError::Unsupported` when the adapter cannot handle the source,
  currently when ffprobe lacks a `local_path_hint`.
- Map process spawn failures to `NakoError::Provider` with provider `ffprobe`.
- Map non-zero ffprobe exit status to `NakoError::Provider` and include trimmed
  stderr as the provider message.
- Map JSON decode failures to `NakoError::Provider` with a parse-specific
  message.
- Treat malformed optional fields as absent rather than fatal when the field is
  not required.
- Use checked math when converting seconds to milliseconds.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| Missing local path hint | `NakoError::Unsupported` |
| ffprobe executable cannot start | `NakoError::Provider { provider: "ffprobe" }` |
| ffprobe exits non-zero | Provider error with trimmed stderr |
| ffprobe JSON cannot decode | Provider error with parse message |
| Unknown stream kind | `MediaStreamKind::Other(value)` |
| Invalid numeric optional field | Field becomes `None` |
| Zero rational numerator or denominator | Frame rate becomes `None` |

## Forbidden Patterns

- Do not panic or unwrap while parsing provider output.
- Do not promote optional ffprobe metadata parse failures into fatal errors.
- Do not discard provider identity from errors.
- Do not expose local path hints in high-level library failure summaries unless
  the caller explicitly chooses that policy.

## Wrong vs Correct

### Wrong

```rust
let value = stream.sample_rate.unwrap().parse().unwrap();
```

### Correct

```rust
let value = parse_u32(stream.sample_rate.as_deref());
```

## Evidence

- `crates/nako-media-probe/src/lib.rs`
