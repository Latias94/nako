# Directory Structure

`nako-automation` currently keeps provider, job, and artifact orchestration in
`src/lib.rs`. Split by provider contracts and job orchestration only when the
crate grows.

## Current Layout

- `AutomationProvider`: external automation provider trait.
- Provider config and capability structs.
- `AutomationJobService`: enqueue and run-once orchestration.
- Job request/result/outcome structs.
- Artifact creation and safe error helpers.

## Module Split Rules

- Keep provider trait and provider request/result types together.
- Move durable job orchestration into a private module before adding more job
  types.
- Keep artifact mapping separate from provider client adapters.
- Keep external API clients outside this crate unless they implement the
  provider trait cleanly.

## Naming Rules

- Use `Automation*Job*` for durable job workflow types.
- Use `AutomationProvider*` for external provider contracts.
- Use `AutomationArtifact` terminology for stored provider outcomes.

## Anti-Patterns

- Do not create provider-specific modules without a concrete provider adapter.
- Do not mix automation provider results with metadata acceptance code.
- Do not add server route handlers here.
