# Logging Guidelines

NFO diagnostics should explain sidecar decisions without logging full XML
documents or sensitive paths.

## Required Patterns

- Prefer structured fields for source ID, sidecar URI, decision, fingerprint
  presence, and conflict count.
- Log policy decisions at the workflow boundary, not inside low-level XML
  parsing loops.
- Keep XML content out of logs.
- Redact or avoid full storage credentials embedded in URIs.

## Forbidden Patterns

- Do not log raw NFO XML.
- Do not log provider secrets or storage credentials.
- Do not emit per-node parser logs during normal operation.
- Do not replace explicit decision summaries with vague log messages.

## Useful Fields

- `nfo.source_id`
- `nfo.sidecar_uri`
- `nfo.decision`
- `nfo.content_fingerprint`
- `nfo.conflict_count`
