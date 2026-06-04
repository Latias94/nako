# VFS Cache Selected Target Refresh Audit

## Current State

- `StorageDiagnosticsAppService` already exposes:
  - latest repair diagnostic;
  - latest action plan;
  - target inventory from unresolved failures;
  - target-scoped preview by opaque HMAC `target_ref`;
  - latest-failure `refresh_cache` execution.
- `target_ref` is generated from failure URI, scheme, operation,
  `failed_at_ms`, `failure_count`, and stored authority using a process-local
  HMAC secret. It is deterministic inside the active process and opaque to
  clients.
- Target preview currently downgrades refreshable diagnostics to `plan_only`
  with `target_scoped_execution_unavailable`. That matched the previous
  read-only boundary.
- VFS refresh execution already uses `backend.refresh_cache(uri, operation)` on
  the resolved storage backend and relies on `vfs_cache_failure_resolved_by_cache`
  to hide failures once a newer cache entry exists.
- Backend resolution already preserves authority:
  - attributed failures must match library id, scheme, and backend key;
  - unattributed failures must match exactly one configured backend root;
  - ambiguous or mismatched targets fail before backend calls.

## Boundary Decision

The selected-target action should reuse the latest refresh implementation shape
but change target selection from "latest unresolved failure" to "failure whose
opaque target ref matches the request". No database schema change is needed
because unresolved status is inferred from cache freshness, consistent with
existing preview/list behavior.

## Required Code Areas

- `crates/nako-server/src/app/storage.rs`
  - share refresh execution across latest and selected-target flows;
  - add target-scoped refresh resolution;
  - update target preview plan to advertise the new route.
- `crates/nako-server/src/http/admin.rs`
  - add POST target refresh route and handler;
  - map target-scoped executable route metadata.
- `crates/nako-api/src/admin_contract.rs`
  - add route inventory entry and generated contract body if route shape changes.
- `apps/admin-web/src/adminApi/generated/contract.ts`
  and `web/src/api/admin/generated/contract.ts`
  - regenerate from `nako-api`.
- `crates/nako-server/src/app/tests/storage.rs`
  and `crates/nako-server/src/http/tests/system.rs`
  - prove selected target execution, redaction, stale/unknown safety, and admin
    access.

## Risks

- Route paths must not embed a concrete target ref in generated contracts; use a
  templated `{target_ref}` path.
- Error bodies must keep using `target_ref` as the not-found id rather than
  echoing the supplied ref.
- Preview plan should become executable only for target-scoped refresh. Latest
  action plan must continue to point at the existing latest refresh route.
- If target refresh is implemented by reusing latest refresh directly, it could
  accidentally refresh the wrong failure. The app service needs a helper that
  accepts the matched `VfsCacheFailure`.
