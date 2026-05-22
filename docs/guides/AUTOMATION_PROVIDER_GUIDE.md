# Automation Provider Guide

Nako automation providers are external HTTP services that produce proposed
artifacts for recommendation, metadata cleanup, summary generation, or title
matching.

## Configure A Provider

Use `POST /automation/providers`:

```json
{
  "id": null,
  "name": "gateway",
  "base_url": "https://example.test/automation",
  "secret_env": "NAKO_AUTOMATION_SECRET",
  "capabilities": ["summary", "recommendation"],
  "timeout_ms": 30000,
  "max_attempts": 2,
  "status": "enabled"
}
```

Secrets are referenced by environment variable name and resolved at runtime.
Resolved secret values must not appear in job input, summaries, logs, or
artifacts.

## Enqueue Jobs

Use `POST /automation/jobs`:

```json
{
  "provider_id": "018f0000-0000-7000-8000-000000000001",
  "capability": "summary",
  "library_id": null,
  "item_id": null,
  "source_id": null,
  "prompt": {
    "title": "The Matrix"
  },
  "idempotency_key": "summary:matrix"
}
```

Jobs run through the same persisted job lifecycle as scan, NFO, and metadata
work. Provider failures are mapped into safe errors and retry state.

## Artifact Policy

Generated results are stored as automation artifacts with status `proposed`.
They do not mutate canonical metadata in M5. A future explicit acceptance
policy must promote proposed artifacts before they become canonical state.

## Current Limits

M5 does not include a concrete OpenAI-compatible provider, local model runtime,
embedding pipeline, vector database, or automatic outbox-triggered automation
scheduler.
