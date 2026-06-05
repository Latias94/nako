# Admin Source Fingerprint Hash Trigger First Slice

## Goal

Add an Admin-only command that lets an operator enqueue one source fingerprint
hash job for a known Media Source, using the shipped internal source hash app
service and existing disk-scan scheduler execution path.

## Requirements

- Add a versioned Admin API command for source fingerprint hash enqueue.
- Accept only redaction-safe trigger fields:
  - `library_id`
  - `source_id`
  - hash mode: `full` or `partial`
  - `partial_prefix_bytes` only for partial mode
  - optional job priority
- Delegate to
  `SourceFingerprintHashAppService::enqueue_source_fingerprint_hash`.
- Return an existing redacted Admin job DTO shape, not durable job input JSON or
  hash evidence.
- Register the route in server Admin routes and generated Admin contract output.
- Keep the existing disk-scan scheduler as the executor for queued jobs.
- Reject cross-library source IDs, missing sources, invalid locators, invalid
  partial prefix, and non-admin callers.
- Keep all error bodies redaction-safe.

## Acceptance Criteria

- [ ] Admin can enqueue a full source fingerprint hash job for a source.
- [ ] Admin can enqueue a partial source fingerprint hash job with an explicit
      prefix byte count.
- [ ] The job persists `JobKind::SourceFingerprintHash`,
      `disk.scan.source_fingerprint_hash`, library binding, source binding, and
      safe input JSON.
- [ ] The response exposes job identity/status facts only and does not expose
      input JSON, Source Locator, raw hash, fingerprint, path, etag, backend URL,
      or credentials.
- [ ] Non-admin access is rejected.
- [ ] Missing source, cross-library source, invalid locator, and invalid partial
      prefix failures do not leak unsafe source details.
- [ ] Admin contract generation/tests cover the new route and DTOs.

## Definition Of Done

- Focused server/app/HTTP tests cover success and rejection paths.
- `nako-api` Admin contract tests pass when DTO/route inventory changes.
- Focused Rust gates pass.
- `git diff --check` passes.
- Changes are committed with a Conventional Commit message.

## Out Of Scope

- No automatic scan-originated enqueue.
- No source hash retry/requeue command.
- No source hash evidence detail route.
- No duplicate relationship mutation or reconciliation plan/apply.
- No source-hash-specific runtime loop.
- No schema migration.
- No Public Client API route.

## Technical Approach

- Add Admin DTOs in `nako-api` under the Admin operations/source hash surface.
- Add HTTP mapping in `nako-server::http::admin`.
- Convert Admin mode/priority into existing server/core types at the HTTP/app
  boundary.
- Reuse `AdminJobListItem` or an equivalent redacted job response wrapper.
- Add generated Admin TypeScript contract coverage from `nako-api`.
- Add focused server route tests using existing Admin auth helpers and source
  hash app fixtures where possible.

## Decision (ADR-lite)

**Context**: Source hash queue, execution, scheduler integration, evidence
persistence, overview diagnostics, and Jobs drill-down filters are already
shipped. The missing next product boundary is an explicit safe trigger.

**Decision**: Implement Admin manual enqueue first. Do not make scan commit or
hash completion perform hidden follow-up work.

**Consequences**: Operators can trigger source hash work with existing durable
job visibility. Automatic source duplicate reconciliation remains deferred
until idempotency, plan/apply, and rollback semantics are specified.

## Technical Notes

- Parent wave:
  `.trellis/tasks/06-06-06-06-fearless-refactor-development-wave/`
- Predecessor research:
  `.trellis/tasks/archive/2026-06/06-05-source-hash-triggering-reconciliation-policy/research/source-hash-triggering-reconciliation-policy.md`
- Source hash server spec:
  `.trellis/spec/nako-server/backend/source-fingerprint-hash-policy.md`
