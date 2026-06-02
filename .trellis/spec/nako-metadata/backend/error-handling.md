# Error Handling

Metadata workflows should make operator-visible state transitions explicit.

## Required Patterns

- Missing targeted reviews or items return `NakoError::NotFound` with a stable
  entity name and ID.
- Stale review operations use `expected_updated_at_ms` and return
  `NakoError::Conflict` when the review changed under the caller.
- Expired, already accepted, already rejected, or already applied reviews are
  conflicts unless the existing service contract returns an unchanged summary.
- Unsupported source or missing root subject should produce a typed application
  plan with `Skip`, not a blind write failure.
- Provider HTTP/runtime failures should be captured as provider diagnostics or
  attempt records; do not leak credentials or raw response payloads in public
  errors.

## Validation Matrix

| Condition | Behavior |
|-----------|----------|
| Review ID not found | `NotFound { entity: "metadata_candidate_review", ... }` |
| Request item ID differs from review item ID | `Conflict` or stale-operation rejection |
| Review expired before decision | mark expired, then return `Conflict` |
| Review not accepted for application | application plan action `Skip` |
| Existing accepted Provider Mapping | application plan action `Noop` |
| Provider lacks endpoint support | reject before HTTP or return provider diagnostic |

## Wrong vs Correct

### Wrong

```rust
// Applies provider mapping even though the review is still pending.
repository.upsert_provider_mapping(mapping).await?;
```

### Correct

```rust
let plan = build_candidate_review_application_plan(repository, &review).await?;
if plan.action == MetadataCandidateReviewApplicationAction::Apply {
    // Apply through the service boundary.
}
```

## Evidence

- `crates/nako-metadata/src/candidate_review.rs`
- `crates/nako-metadata/src/providers/douban.rs`
- `crates/nako-metadata/src/runtime.rs`
