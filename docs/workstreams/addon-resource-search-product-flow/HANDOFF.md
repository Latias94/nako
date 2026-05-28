# Addon Resource Search Product Flow - Handoff

Status: active. RSPF-010 is complete; RSPF-020 is next.

## Current State

- `addon-resource-search-protocol` is closed and delivered protocol/client
  diagnostics/intake handoff.
- This lane is for Nako product API flow only.
- Diagnostic resource search remains separate and should not return raw result
  payloads.

## Next Task

RSPF-020: add Admin API DTOs and route constants for product search and
selection.

Expected behavior:

- Add product DTOs distinct from diagnostic DTOs.
- Include opaque `search_id` and `selection_id` concepts.
- Return display-safe result/link summaries.
- Do not expose raw link URLs, passwords, context payloads, or provider
  exception text.

## Useful Gate

```bash
cargo nextest run -p nako-api admin_contract --no-fail-fast
```

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not let the browser submit raw selected links.
- Keep diagnostic and product routes separate.
- Keep official addon migration in `nako-official-addons` follow-on work.
