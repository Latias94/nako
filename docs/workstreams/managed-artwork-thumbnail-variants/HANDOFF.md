# Managed Artwork Thumbnail Variants Handoff

Status: Completed
Last updated: 2026-05-19

## Current State

This lane is closed. `MATV-010`, `MATV-020`, and `MATV-030` are complete.

The lane shipped bounded, redacted image variant serving for Selected Artwork:

- optional `width` and `height` query parameters for `GET/HEAD
  /images/{image_id}`;
- zero and over-limit dimensions are rejected with redacted invalid-input
  errors;
- variants are derived on demand while preserving aspect ratio and avoiding
  upscaling;
- original route behavior remains compatible when no variant is requested;
- content-hash ETags were replaced with opaque presentation validators;
- OpenAPI, the generated TypeScript SDK, the Rust client request builders, and
  `docs/api/HTTP_API.md` describe the variant query contract.

## Closeout

- Task IDs: MATV-020, MATV-030
- Owner: codex
- Validation: fresh closeout gates recorded in `EVIDENCE_AND_GATES.md`
- Status: DONE
- Evidence: `EVIDENCE_AND_GATES.md`

## Blockers

None known.

## Follow-Ons Outside This Lane

- Persisted thumbnail/variant cache and eviction.
- `managed-artwork-gallery-candidate-management`
- `managed-artwork-ingest-runtime-controls`
- Missing-artifact repair/re-ingest.

Keep the redaction invariant in all follow-ons: no `storage_uri`, source URL,
`cache_uri`, local path, `managed-artwork://...`, artifact content hash, or
addon/provider token material in Public Client/Admin DTOs or image headers.
