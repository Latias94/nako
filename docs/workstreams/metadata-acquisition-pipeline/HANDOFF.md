# Metadata Acquisition Pipeline Handoff

Lane closed on 2026-05-25.

## Current State

Nako already scans libraries, imports NFO sidecars, can enqueue scan-triggered
Addon Bulk Metadata Scrape task runs, and has a protected Addon Side Effect
path for `metadata_write`.

The core product/architecture loop is implemented: scan-time metadata
acquisition has a focused service, scan-triggered Addon scrape can opt into
explicit writeback payloads, and the writeback path is proven through Nako's
Addon Side Effect route.

## Completed Proof

- Focused tests prove scan acquisition plan derivation, public DTO/OpenAPI/SDK
  shape, default suggestion-only Addon scrape, explicit writeback payload, and
  side-effect metadata merge.
- Real-directory smoke passed for `H:\Super\Videos`.
- Real-directory smoke passed for NAS SMB subdirectory
  `\\frankorz-nas\home\www\Data\Video\Super\JAV_output\安位カヲル`.

## Follow-Ons

1. Full `\\frankorz-nas\home\www\Data\Video\Super\JAV_output` root scan is
   intentionally deferred because it is a large batch validation, not a smoke.
2. Official sidecar process writeback smoke can be added when the operator wants
   an external-process proof beyond the in-process sidecar/Nako HTTP route test.
3. UI/Admin controls for scan metadata source ordering remain product follow-on
   work.

## Guardrails

- Do not make Addon Task output an implicit metadata mutation source.
- Do not bypass Addon Token, Grant, target, idempotency, or merge validation.
- Keep writeback default false.
- Preserve existing NFO import behavior.
