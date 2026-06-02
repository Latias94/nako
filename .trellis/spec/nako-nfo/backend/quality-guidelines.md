# Quality Guidelines

NFO changes must protect round-trip behavior and keep import/export workflows
observable before mutation.

## Required Patterns

- Add round-trip tests for parsed and rendered XML.
- Preserve unknown fields and comments when using preserving render paths.
- Keep preview decisions pure and non-mutating.
- Derive sidecar URI from source locator consistently.
- Track content fingerprints in summaries and decisions.

## Forbidden Patterns

- Do not drop unknown XML fields during preservation.
- Do not mutate VFS or repositories during preview.
- Do not couple codec parsing to metadata provider adapters.
- Do not rely on OS filesystem paths instead of `StorageUri`.

## Tests Required

- Codec parse/render tests for valid and invalid XML.
- Preserving render tests for unknown fields, comments, and conflicts.
- Import policy tests for skip/update/create/fail.
- Export policy tests for existing and missing sidecars.
- Workflow tests with fake `StorageBackend`.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-nfo --no-fail-fast`
- VFS contract:
  `cargo check -p nako-nfo -p nako-vfs --tests`
