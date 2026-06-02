# Directory Structure

`nako-metadata` owns provider-neutral metadata workflow logic and provider
adapters. It does not own database adapters, HTTP routes, Admin DTOs, or
catalog projection storage.

## Current Layout

```text
crates/nako-metadata/src/
├── lib.rs                 # public export surface
├── providers/             # TMDB, Bangumi, Douban provider adapters
├── mapping/               # provider payload to Nako domain mapping
├── runtime.rs             # HTTP JSON runtime and provider diagnostics
├── registry.rs            # provider registration diagnostics
├── strategy.rs            # metadata refresh strategy and port traits
├── candidate_review.rs    # durable review planning, decide, apply services
├── confirmation.rs        # hierarchy confirmation service
├── matching.rs            # candidate conflict review/match policy
└── tests/                 # fixtures and focused metadata tests
```

## Module Rules

- Put provider-specific HTTP and payload behavior under `providers/<name>.rs`.
- Put provider payload to Nako domain conversion under `mapping/<name>.rs`.
- Keep refresh orchestration in `strategy.rs` through port traits such as
  `MetadataAttemptPort` and `MetadataRefreshPort`.
- Keep Candidate Review governance in `candidate_review.rs`: build plan,
  decide, apply root mapping, and apply related hierarchy are separate steps.
- Keep hierarchy confirmation in `confirmation.rs` instead of replacing Media
  Items during provider refresh.
- Re-export public services and records from `lib.rs`.

## Forbidden Placement

- Do not import `nako-db` or `sqlx` here. Use repository traits from
  `nako-core`.
- Do not add Admin route DTOs here. Put wire contracts in `nako-api` and
  handler mapping in `nako-server`.
- Do not make provider payloads canonical metadata by default. Use merge policy,
  Candidate Review, Provider Mapping, and Hierarchy Confirmation boundaries.
- Do not copy reference-provider code from `repo-ref/`; use it only to study
  behavior.

## Examples

- `candidate_review.rs`: plan/apply separation and stale review validation.
- `strategy.rs`: refresh orchestration through port traits.
- `mapping/tmdb.rs`: provider-specific mapping isolated from core records.
- `runtime.rs`: provider HTTP runtime boundary.
