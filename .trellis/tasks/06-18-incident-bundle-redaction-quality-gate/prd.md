# Incident Bundle Redaction Quality Gate

## Goal

Restore the `nako-api` contract test gate for the Admin incident bundle without
weakening the redaction contract. The current failure is a false positive: the
test scans the serialized JSON string for the substring `credential`, which
also matches the valid redaction summary field `credentials_redacted`.

## Requirements

- Keep the Incident Bundle Admin API wire shape unchanged.
- Preserve the contract that the incident bundle does not expose raw
  credentials, tokens, paths, locators, backend URLs, query strings, durable job
  payloads, provider payloads, or FFmpeg command lines.
- Replace broad whole-body substring checks only where they create false
  positives against legitimate redaction status fields.
- Keep assertions strong enough to catch unsafe field names and unsafe sample
  values in the support artifact.
- Do not modify unrelated dirty files:
  `crates/nako-api/src/admin/managed_artwork.rs` and
  `crates/nako-reference-addon/src/lib.rs`.

## Acceptance Criteria

- [ ] `admin::incident_bundle::tests::incident_bundle_response_serializes_support_artifact_without_sensitive_families`
      passes and still proves sensitive values are absent.
- [ ] `cargo nextest run -p nako-api admin::incident_bundle::tests::incident_bundle_response_serializes_support_artifact_without_sensitive_families --no-fail-fast`
      passes.
- [ ] `cargo nextest run -p nako-api --no-fail-fast` passes, or any remaining
      failure is verified as unrelated and documented.
- [ ] `cargo fmt --all -- --check` and `git diff --check` pass.
- [ ] Only task-related files are committed.

## Technical Approach

- Keep the test fixture representative of the incident bundle contract.
- Add a small test helper that walks `serde_json::Value` and rejects forbidden
  sensitive terms in object keys and string values, while allowing the explicit
  redaction summary boolean fields that intentionally say what was redacted.
- Prefer exact test helper logic over changing DTO names or deleting the
  `credentials_redacted` flag.

## Out of Scope

- New incident bundle fields or route behavior.
- Admin Web contract regeneration.
- Server route changes.
- Managed artwork API work.
