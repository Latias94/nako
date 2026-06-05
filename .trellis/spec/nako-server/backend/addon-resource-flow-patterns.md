# Addon Resource Flow Patterns

Use this spec before changing server-side Addon flows that turn read-only Addon
discovery into host-owned selection, planning, materialization, or writes.

## Scenario: Host-Owned Addon Resource Flow

### 1. Scope / Trigger

Trigger this spec when server work touches any of:

- Addon resource search selections.
- Addon subtitle selected references, import plans, or import apply.
- External acquisition selected-link, intake-candidate, or materialization
  handoff.
- New Addon resource types that need Admin-visible candidate selection followed
  by host-owned side effects.

This is a `nako-server` app/HTTP boundary pattern. Do not move host-owned
selection sessions, Admin apply plans, grant storage, VFS writes, or durable
job policy into `nako-addon-protocol`.

### 2. Signatures

Existing signatures that define the pattern:

- `AdminAddonResourceSearchResponse` exposes `search_id`, redacted result
  summaries, provider diagnostics, and optional `safe_error_code`.
- `AdminAddonResourceSearchLinkSummary` exposes `selection_id`,
  `link_type`, safe display source, `source_ref_redacted`, and safe flags.
- `AdminAddonSubtitleSelectedReference` exposes addon, manifest, search, and
  selection identity without exposing subtitle delivery material.
- `AdminSubtitleImportPlan` exposes host-derived idempotency key, status,
  reasons, target summary, sidecar plan, conflict policy, and backup policy.
- `AddonExternalAcquisitionMaterializationRequest` accepts only protocol target
  refs plus job/declaration/runner/idempotency/audit facts; the server validates
  the running host task before materializing raw link data.

### 3. Contracts

- Discovery Addon calls are read-only unless a separate action scope and host
  handoff exists.
- Admin callers receive opaque `search_id` and `selection_id` values, not raw
  URLs, passwords, addon secrets, local paths, or materialization refs.
- Selection sessions are short-lived host memory. They must carry `addon_id`,
  `manifest_id`, `created_at_ms`, `expires_at_ms`, and typed selections.
- Apply plans are host-derived and idempotency-keyed. Resource-specific modules
  keep domain validation such as subtitle language/format checks.
- Apply/materialization results report host decisions and redacted facts, not
  raw sidecar/provider material.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Missing selection session or selection id | Return not-found for the resource-specific selected entity. |
| Addon id or manifest id mismatch | Reject before handoff. |
| Read-only discovery tries to imply a side effect | Require a separate action/link-check/materialization flow. |
| Apply request idempotency key differs from plan | Reject as invalid input. |
| External acquisition materialization is not for a running aligned task | Reject as invalid materialization request. |
| Unsupported materialized link type | Reject without exposing raw source ref. |
| Addon/client failure | Map to stable `safe_error_code` and redacted facts. |

### 5. Good/Base/Bad Cases

- Good: Resource Search stores a selected link in a host session, Admin selects
  by `selection_id`, and Nako records an acquisition intake candidate with
  redacted diagnostics.
- Good: Subtitle Search stores a selected candidate, Nako derives a sidecar file
  plan, writes through Library File Write/VFS, and refreshes subtitle facts.
- Good: External acquisition action receives a selected-link or intake-candidate
  ref, then calls Nako runtime materialization to receive raw material only
  after task/token/idempotency/audit validation.
- Base: Read-only discovery succeeds but no side effect happens until the Admin
  caller selects, plans, or applies.
- Bad: Browser resubmits a raw link/password from a search result.
- Bad: Addon Protocol DTOs grow fields for Nako Admin session TTL or apply-plan
  policy.
- Bad: Product responses include raw source URIs, local paths, bearer tokens,
  materialization refs, or idempotency keys.

### 6. Tests Required

When changing these flows, keep or add tests that assert:

- Response shapes still expose `search_id`/`selection_id` or selected refs
  without raw source material.
- Selection lookup rejects missing, expired, wrong-addon, or wrong-manifest
  references.
- Apply plans are deterministic and idempotency-keyed.
- Apply/materialization responses do not contain raw paths, raw URLs, bearer
  tokens, addon tokens, candidate IDs that grant raw access, materialization
  refs, or idempotency keys.
- Resource-specific behavior remains in the resource module after shared helper
  extraction.

### 7. Wrong vs Correct

#### Wrong

```text
Addon search result -> Admin response includes raw URL -> browser posts raw URL
back to Nako -> server starts downloader/import.
```

This gives a read-only Addon or browser request implicit side-effect authority.

#### Correct

```text
Addon search result -> Nako stores a host-owned selected reference -> Admin
posts selection_id -> Nako derives plan/materialization/write through host
policy -> response contains only redacted diagnostics.
```

This preserves Nako ownership of grants, audit, idempotency, storage, durable
work, and diagnostics.
