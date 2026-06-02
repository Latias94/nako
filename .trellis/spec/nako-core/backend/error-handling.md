# Error Handling

`nako-core` owns the shared `NakoError` type. Use it to express domain and
repository-contract failures without pulling in adapter-specific error types.

## Required Patterns

- Invalid request fields or impossible domain inputs return
  `NakoError::InvalidInput`.
- Stale updates, already-applied decisions, cancellation conflicts, and invalid
  state transitions return `NakoError::Conflict`.
- Missing records return `NakoError::NotFound { entity, id }` when the caller
  asked for a specific entity.
- Unknown persisted enum strings or scores return `NakoError::Database`; see
  `JobKind::parse`, `JobStatus::parse`, and `JobPriority::from_score`.
- Provider or external runtime failures should not be represented in core with
  raw HTTP, SQL, or process errors. Convert them at the adapter boundary.

## Validation Matrix

| Condition | Error |
|-----------|-------|
| Empty required locator, URI, ID-like value, or zero max attempts | `InvalidInput` |
| Retry source job does not match loaded job | `InvalidInput` |
| Retrying a non-failed job | `Conflict` |
| Unknown stored enum value | `Database` |
| Expected record missing during a targeted operation | `NotFound` |

## Wrong vs Correct

### Wrong

```rust
return Err(sqlx_error.into());
```

### Correct

```rust
return Err(NakoError::Database {
    message: format!("unknown job status stored in database: {value}"),
});
```

`nako-core` should expose project error meaning, not adapter error types.

## Evidence

- `crates/nako-core/src/error.rs`
- `crates/nako-core/src/job.rs`
- `crates/nako-metadata/src/candidate_review.rs`
