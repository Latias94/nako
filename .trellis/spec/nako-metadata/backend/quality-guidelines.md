# Quality Guidelines

Metadata work must preserve local authority and review governance.

## Required Patterns

- Plan before apply. Candidate Review and Generated Artifact paths should expose
  a readable plan before mutating canonical state.
- Preserve provider-neutral Media Item Hierarchy. Provider-specific subjects map
  through Provider Mapping and Hierarchy Confirmation; they do not create a
  provider-owned item model.
- Keep root Provider Mapping application separate from related hierarchy
  application unless the task explicitly covers both.
- Keep provider capability claims honest. If a provider does not support a
  subject kind or endpoint, reject before HTTP and test that behavior.
- Use `Metadata Source Priority`, local/NFO authority, and field-lock language
  from `CONTEXT.md`.

## Forbidden Patterns

- Do not let provider refresh overwrite confirmed canonical metadata without
  merge policy or an acceptance path.
- Do not treat tags as item identity or provider subjects as Media Items.
- Do not add hidden background metadata jobs outside durable job/runtime
  boundaries.
- Do not expose raw provider cache, secrets, or unredacted URLs through Admin
  or Public API summaries.

## Tests Required

- Unit tests for provider mapping and capability rejection.
- Service tests for Candidate Review status transitions, stale operations,
  application plans, and related hierarchy application.
- Cross-crate tests in `nako-server` when Admin/API routes call the metadata
  service.

## Gate Selection

- Focused metadata:
  `cargo nextest run -p nako-metadata <filter> --no-fail-fast`
- Cross-crate metadata/API/server:
  `cargo check -p nako-core -p nako-metadata -p nako-api -p nako-server --tests`

## Review Checklist

- Does the code use Nako terms from `CONTEXT.md`?
- Is mutation separated from plan/preview?
- Are provider-specific assumptions isolated under `providers/` or `mapping/`?
- Are stale operations and repeated applies deterministic?

## Scenario: Provider Capability Endpoint Precision

### 1. Scope / Trigger

- Trigger: widening a metadata provider capability to a new `MediaKind` or
  `ProviderSubjectKind`, especially when the provider reuses one endpoint for
  multiple subject kinds.

### 2. Signatures

- `MetadataProvider::capabilities()`
- `MetadataProvider::search(MetadataLookup)`
- `MetadataProvider::fetch(MetadataFetchRequest)`
- Provider-specific response structs under `providers/*.rs`

### 3. Contracts

- Capability claims must be backed by executable endpoint behavior.
- If one endpoint returns mixed subject kinds, validate the provider response
  discriminator before mapping to a Nako kind. For Douban, `subtype: "tv"`
  backs `MediaKind::Series`; it does not imply Season/Episode support.
- Unsupported media kinds must fail before HTTP unless the task proves an
  endpoint contract for that kind.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|-----------|-------------------|
| Search/fetch kind has no endpoint support | Return `NakoError::Unsupported` before HTTP. |
| Shared endpoint returns wrong subtype | Reject the result instead of mapping it to the requested Nako kind. |
| New subject-level support is added | Update capability diagnostics and provider tests. |
| New hierarchy/episode support is claimed | Add graph tests proving related subjects and relationships. |

### 5. Good / Base / Bad Cases

- Good: Douban Series support filters search results to `subtype: "tv"` and
  keeps the graph root-only.
- Base: Movie support accepts existing movie fixtures and stays backward
  compatible with missing subtype values when older fixtures omit the field.
- Bad: advertising Season/Episode support because a provider has a TV subject
  endpoint without proving episode endpoint semantics.

### 6. Tests Required

- Capability diagnostics test for supported and unsupported kinds.
- Provider search/fetch test proving the endpoint path, discriminator, metadata
  mapping, subject kind, and graph shape.
- Unsupported-kind test proving rejected kinds do not touch HTTP.

### 7. Wrong vs Correct

#### Wrong

```rust
supported_media_kinds: vec![MediaKind::Movie, MediaKind::Series, MediaKind::Episode]
```

#### Correct

```rust
supported_media_kinds: vec![MediaKind::Movie, MediaKind::Series, MediaKind::Unknown]
```

Only list the kinds whose endpoint contract and graph behavior are covered by
tests.
