# Library Watcher Planned-Write Suppression Evidence

Date: 2026-06-03

## Selected Slice

Implemented planned-write suppression first. The slice adds a process-local
watch-folder suppression registry owned by `nako-server`, then routes
watch-folder discovery and runtime ticks through it before stable-candidate
intake records are updated.

## Behavior

* Nako-owned write workflows can begin a suppression scope with library id,
  `StorageUri` scope, safe owner, safe reason, TTL, and completion behavior.
* Suppression matching is scheme-aware and source-scope based. A scope matches
  the exact URI and descendants, not raw host paths.
* Suppressed watch-folder entries are not recorded as intake candidates, do not
  advance stable observation evidence, and do not enqueue library scans.
* Completion removes the suppression and reports whether reconciliation was
  requested by completion semantics.
* Admin discovery diagnostics expose only redaction-safe suppression facts:
  library id, scheme, redacted scope reference, safe owner, safe reason,
  expiry, completion, and aggregate suppressed count.
* Admin generated TypeScript contracts now include the named
  `AdminWatchFolderSuppression` DTO.
* `StorageBackendKind::WebDav` is explicitly serialized as `webdav` so the
  Admin contract matches existing frontend backend-kind usage.

## Non-Goals Preserved

* No persistence or schema migration.
* No broad degraded watcher state, overflow handling, permission dashboard, or
  backend capability matrix.
* No raw paths, raw source locators, credentials, fingerprints, etags, backend
  URLs, or raw provider/backend errors in diagnostics.
* No Jellyfin source code copied or ported.

## Validation

Passed:

* `cargo fmt --all`
* `cargo check -p nako-server -p nako-api --tests`
* `cargo nextest run -p nako-server watch_folder --no-fail-fast`
* `cargo nextest run -p nako-api watch_folder --no-fail-fast`
* `cargo nextest run -p nako-api admin_contract --no-fail-fast`
* `cargo nextest run -p nako-api admin_storage --no-fail-fast`
* `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output apps/admin-web/src/adminApi/generated/contract.ts`
* `cargo run -q -p nako-api --example emit-admin-typescript-contract -- --output web/src/api/admin/generated/contract.ts`
* `cargo fmt --all -- --check`
* `npm run check --prefix apps/admin-web`
* `git diff --check`
