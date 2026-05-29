# Web Admin Acquisition Intake - Mutation Boundary Decision

Status: Deferred to follow-on
Decided: 2026-05-29
Task: WAAI-040

## Decision

Do not implement watch-folder discovery mutation controls in this lane.

`/admin/acquisition/intake` remains a read-only Admin route. The watch-folder
discovery mutation must split into a focused follow-on lane, tentatively named
`web-admin-watch-folder-discovery-mutation`, before any UI button, form, or
data-source mutation method is added.

## Contract Inventory

The generated Admin API already exposes the route:

```text
POST /admin/v1/acquisition/intake/watch-folder-discovery
```

Request:

```text
target_library_id
root_uri
max_depth
```

Response:

```text
target_library_id
root_scheme
root_ref_redacted
ready_candidates
blocked_candidates
incomplete_candidates
unsupported_candidates
recorded_candidates
failures[].ref_redacted
failures[].safe_message
writes_library
managed_import_artifacts_created
promotion_apply
```

The response is intentionally explicit that the operation does not perform
promotion apply or direct library writes. It may still record candidates and
create Managed Import artifacts, so it is a mutation with durable side effects.

## Why It Splits

- The current lane's target state is a read-only route-first diagnostic page.
- Discovery can traverse storage/VFS roots and may be slow, partial, or noisy.
- The UI needs a confirmation step because it records intake candidates.
- Idempotency expectations must be visible before operators repeat a discovery.
- Permission and audit semantics must be named before a mutation appears in the
  Admin route.
- Failure output must remain redacted and must not expose raw local paths,
  credentials, source locators, downloader internals, or request bodies.
- The route must not imply Managed Import promotion, apply, or direct library
  writes.

## Follow-On Requirements

A future guarded mutation lane must define and test:

- Admin permission requirement and disabled state when unavailable.
- Confirmation copy that states the operation records intake candidates but does
  not apply them to a media library.
- Idempotency behavior for repeated `target_library_id`, `root_uri`, and
  `max_depth` submissions.
- Redacted result display for `root_ref_redacted` and `failures`.
- Clear result counters for ready, blocked, incomplete, unsupported, recorded,
  and Managed Import artifact creation.
- Loading, partial failure, retry, and empty-result states.
- Route or local state behavior after the mutation finishes.
- Data-source contract tests for request serialization and response mapping.
- Route tests that prove no raw path, credential, prompt, source locator, or
  downloader-internal text is rendered.

## Current Lane Scope

WAAI closes as the read-only Acquisition Intake page lane. It may link to the
future discovery mutation follow-on, but it must not add mutation controls under
WAAI.
