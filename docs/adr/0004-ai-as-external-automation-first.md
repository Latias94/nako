# 0004: Treat AI as External Automation First

## Status

Proposed

## Context

AI features can improve edge-case user experience, such as recommendations,
metadata cleanup, summaries, title matching, notification text, or workflow
automation. However, local models, vector databases, embedding pipelines, and
GPU scheduling would greatly increase the foundation scope.

## Decision

Treat AI as external automation first. Nako should provide provider
configuration, secret handling, job execution, audit logs, and explicit user
approval paths. Providers can be OpenAI-compatible gateways, custom HTTP
services, or recommendation APIs.

AI-generated results should be stored as suggestions or generated artifacts
unless a user or policy explicitly accepts them into canonical metadata.

## Consequences

- Useful automation can ship earlier.
- Users can bring their own API keys or gateway providers.
- Nako avoids local model and vector-search complexity in the MVP.
- Provider cost, latency, and privacy must be visible in configuration and logs.
- The automation layer should be designed for retries, rate limits, and
  cancellation from the start.

## Alternatives Considered

- Local model runtime: more private, but too large for the server foundation.
- Vector search first: useful later, but not required for recommendation and
  metadata automation MVPs.
- Hard-coded provider integration: faster initially, but poor extensibility and
  difficult to support across regions.

## Related Workstreams

- `docs/workstreams/server-foundation/`
