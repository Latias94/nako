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
