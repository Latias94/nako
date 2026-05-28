# Subtitle Import Apply Design

## Intent

Nako owns subtitle sidecar mutation. Subtitle provider addons may search and
return opaque selected references, but only the host may resolve subtitle
content, validate it, derive the library target, and write a sidecar file.

This lane follows `subtitle-import-plan-preview` and turns a ready import plan
into a controlled Library File Write apply operation.

## In Scope

- Admin subtitle import apply endpoint under the selected-reference route.
- Reuse the import-plan request fields plus the plan idempotency key.
- Recompute the plan server-side before mutation.
- Resolve subtitle content from inline or download-url delivery.
- Reject artifact-ref delivery until a host-owned artifact resolver exists.
- Validate text size, UTF-8, and basic format markers before writing.
- Derive the sidecar target from the cataloged media source locator.
- Write only through Library File Write / VFS storage write behavior.
- Return redaction-safe apply output without URLs, raw content, local paths, or
  backup URIs.

## Out Of Scope

- Cloud-drive transfer or remote save semantics.
- Addon-provided target paths.
- Playback subtitle fact refresh.
- Library scan mutation.
- Third-party downloader plug-in registry.

## Boundary

Addon:

- Provides subtitle search candidates and delivery references.
- Never chooses a library path.
- Never writes library files.

Nako host:

- Revalidates selected reference ownership.
- Recomputes plan idempotency.
- Downloads or reads subtitle content.
- Applies conflict and backup policies.
- Owns VFS writes and redacted reporting.

## API Shape

`POST /admin/v1/addons/{addon_id}/subtitle-search/{search_id}/selections/{selection_id}/import-apply`

The request repeats the import-plan fields and includes
`plan_idempotency_key`. The server rejects stale or mismatched keys.

The response includes selected reference, safe candidate summary, the safe plan,
and a safe apply report.

## Risks

- Download URLs can contain secrets. Responses and errors must not echo them.
- `create_missing` must not silently overwrite existing sidecars.
- Path traversal must be impossible because the sidecar URI is derived from the
  stored media source locator and a sanitized sidecar file name.
- Repeated apply should be idempotent when the target already contains the same
  content.

## Validation

- `cargo nextest run -p nako-api subtitle_import_apply --no-fail-fast`
- `cargo nextest run -p nako-api admin_contract --no-fail-fast`
- `cargo nextest run -p nako-server addon_subtitle_import_apply --no-fail-fast`
- `cargo check -p nako-api -p nako-server --tests`
- `cargo fmt --all -- --check`
- `git diff --check`
