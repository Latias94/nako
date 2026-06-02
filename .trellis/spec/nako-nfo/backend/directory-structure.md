# Directory Structure

Keep NFO concerns split by codec, policy, storage workflow, and summaries.

## Current Layout

- `codec.rs`: `NfoCodec`, parse/render contracts, preservation support.
- `import.rs`: import policy and decision data.
- `export.rs`: export policy and decision data.
- `preview.rs`: non-mutating authority preview decisions.
- `workflow.rs`: `NfoService`, VFS-backed sidecar workflow, URI planning.
- `summary.rs`: import/export result summaries.
- `lib.rs`: public exports and crate-level composition.

## Module Rules

- Put XML parser or renderer changes in `codec.rs`.
- Put decision-only import/export policy in `import.rs` or `export.rs`.
- Put storage-backed operations in `workflow.rs`.
- Keep preview paths non-mutating.
- Keep summary structs stable and explicit.

## Naming Rules

- Use `Nfo*Policy` and `Nfo*Decision` for policy results.
- Use `NfoService` for storage-backed workflow orchestration.
- Use `sidecar` terminology for files derived from media source locators.

## Anti-Patterns

- Do not add direct filesystem modules.
- Do not mix XML rendering with repository mutation logic.
- Do not create provider-specific codec modules unless a real NFO format exists.
