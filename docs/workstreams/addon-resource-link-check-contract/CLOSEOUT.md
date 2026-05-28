# Addon Resource Link Check Contract - Closeout

Status: complete
Closed: 2026-05-28

## Outcome

Nako now has a first-class read-only addon contract for resource link checking.

Delivered:

- `AddonResource::ResourceLinkCheck` with wire value `resource_link_check`.
- `AddonScope::AcquisitionLinkCheckRead` with wire value
  `acquisition_link_check_read`.
- Request schema `nako.addon.resource_link_check.request.v1`.
- Response schema `nako.addon.resource_link_check.response.v1`.
- Typed `AddonResourceLinkCheckRequest`.
- Typed `AddonResourceLinkCheckResponse`.
- Stable status vocabulary: reachable, unavailable, password_needed,
  unsupported, rate_limited, error, unknown.
- `call_addon_resource_link_check` and
  `call_addon_resource_link_check_with_outcome` helpers.
- Tests proving manifest/scope validation, schema validation, payload
  validation, envelope resource selection, and debug redaction.

## Review

Workstream compliance: no blocking findings.

Code quality: no blocking findings.

The response DTO intentionally exposes only safe facts: link type, status,
checked time, password-needed fact, retry facts, safe message, and safe key/value
facts. It does not include raw URL or password fields.

## Gates

Passed on 2026-05-28:

```bash
cargo nextest run -p nako-addon-protocol resource_link_check --no-fail-fast
cargo nextest run -p nako-addon-client resource_link_check --no-fail-fast
cargo nextest run -p nako-addon-protocol -p nako-addon-client resource_link_check --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client --tests
git diff --check
```

`git diff --check` reported Windows line-ending warnings only.

## Follow-Ons

- Add a server/product route that consumes opaque `search_id`/`selection_id` or
  another host-owned selected-link reference.
- Add an official checker addon only after provider scope and network behavior
  are clear.
- Keep Admin UI out of this lane.
- Keep downloader/external runner, cloud-drive transfer, and password/code
  persistence separate from link checking.

## Residual Risk

The contract is additive and not yet wired into product routes. That is
intentional; the next lane should design how host-owned selected links become
link-check requests without exposing raw URLs/passwords to browser clients.
