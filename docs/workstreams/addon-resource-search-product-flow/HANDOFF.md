# Addon Resource Search Product Flow - Handoff

Status: active. RSPF-010 and RSPF-020 are complete; RSPF-030 is next.

## Current State

- `addon-resource-search-protocol` is closed and delivered protocol/client
  diagnostics/intake handoff.
- This lane is for Nako product API flow only.
- Diagnostic resource search remains separate and should not return raw result
  payloads.

## Next Task

RSPF-030: implement server-side product search sessions and safe result
shaping.

Expected behavior:

- Use typed resource-search addon calls.
- Return display-safe result/link summaries with opaque selection IDs.
- Keep raw links inside a transient host-owned selection session.
- Do not expose raw link URLs, passwords, context payloads, or provider
  exception text.

## Useful Gate

```bash
cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not let the browser submit raw selected links.
- Keep diagnostic and product routes separate.
- Keep official addon migration in `nako-official-addons` follow-on work.
