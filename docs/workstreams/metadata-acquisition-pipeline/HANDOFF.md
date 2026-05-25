# Metadata Acquisition Pipeline Handoff

Active lane opened on 2026-05-25.

## Current State

Nako already scans libraries, imports NFO sidecars, can enqueue scan-triggered
Addon Bulk Metadata Scrape task runs, and has a protected Addon Side Effect
path for `metadata_write`.

The core product/architecture loop is now implemented: scan-time metadata
acquisition has a focused service, scan-triggered Addon scrape can opt into
explicit writeback payloads, and the writeback path is proven through Nako's
Addon Side Effect route.

## Next Steps

1. Complete MAP-050 by re-running the local/NAS scan and playback smoke against
   the refactored code.
2. Decide whether an official addon sidecar process is available for a
   post-commit writeback smoke; the in-process sidecar already proves the Nako
   route and merge boundary.
3. Complete MAP-060 after evidence is fresh.

## Guardrails

- Do not make Addon Task output an implicit metadata mutation source.
- Do not bypass Addon Token, Grant, target, idempotency, or merge validation.
- Keep writeback default false.
- Preserve existing NFO import behavior.
