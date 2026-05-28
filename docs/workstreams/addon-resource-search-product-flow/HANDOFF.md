# Addon Resource Search Product Flow - Handoff

Status: closed. RSPF-010 through RSPF-060 are complete.

## Current State

- `addon-resource-search-protocol` is closed and delivered protocol/client
  diagnostics/intake handoff.
- This lane is for Nako product API flow only.
- Diagnostic resource search remains separate and should not return raw result
  payloads.
- Product search now returns display-safe result cards backed by a bounded,
  transient host-owned session.
- The browser receives opaque `search_id`/`selection_id` values and never
  submits raw link URLs/passwords back to Nako.
- Explicit selection now creates or replays a `resource_search_selection`
  acquisition intake candidate.

## Next Task

No task remains in this lane. Start a new workstream or issue for one of the
follow-ons below.

Closeout should preserve these follow-ons outside this lane:

- Admin UI for running searches and selecting links.
- `nako-official-addons` manifest/provider migration.
- Link availability checks.
- Downloader execution and external download orchestration.
- Cloud-drive save/transfer.
- Password or extraction-code persistence as secrets.

## Useful Gate

```bash
cargo nextest run -p nako-server addon_resource_search_product --no-fail-fast
cargo nextest run -p nako-server acquisition_intake --no-fail-fast
cargo nextest run -p nako-api admin_contract --no-fail-fast
cargo fmt --all -- --check
cargo check -p nako-addon-protocol -p nako-addon-client -p nako-api -p nako-core -p nako-db -p nako-server --tests
git diff --check
```

See `CLOSEOUT.md` for final gates, review result, and residual risks.

## Watch Points

- Keep `acquisition_search_read` read-only.
- Do not let the browser submit raw selected links.
- Keep diagnostic and product routes separate.
- Keep official addon migration in `nako-official-addons` follow-on work.
