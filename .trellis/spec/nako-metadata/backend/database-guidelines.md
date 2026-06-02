# Database Guidelines

`nako-metadata` talks to persistence through repository traits only.

## Contract Rules

- Use `nako-core` repository traits such as `MetadataCandidateReviewRepository`,
  `ProviderMappingRepository`, `LibraryItemRepository`, and
  `MetadataRepository`.
- Keep service structs generic over repository traits. Example:
  `MetadataCandidateReviewApplicationService<R>`.
- Return domain summaries from metadata services; let `nako-server` map them to
  Admin/Public DTOs.
- Do not perform raw SQL, migrations, or row mapping in this crate.
- When a metadata workflow needs a new persisted field, update `nako-core`
  records and `nako-db` adapters in the same Trellis task or split an explicit
  schema task first.

## Good/Base/Bad Cases

- Good: a service loads a Candidate Review through a repository trait, builds a
  typed plan, then calls repository methods to apply status or Provider Mapping
  changes.
- Base: provider fetch returns a candidate graph without any database write.
- Bad: provider adapter writes provider subjects directly to SQLite or returns
  a database row as metadata output.

## Evidence

- `crates/nako-metadata/src/candidate_review.rs`
- `crates/nako-metadata/src/strategy.rs`
- `crates/nako-core/src/repository/metadata.rs`
