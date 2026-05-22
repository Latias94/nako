# 0015: Use Capability-Scoped HTTP Addons and Automation Providers

## Status

Accepted

## Status Note

Accepted by implementation evidence from the completed Addons and Automation
workstream and follow-on Addon Token Grants Side Effects workstream. Addon
manifest resource scopes and accepted Addon Permissions remain distinct
Interfaces; the Addon Architecture Deepening workstream owns further
clarification without weakening the default-deny model.

## Context

Nako needs extension points for metadata, recommendations, automation, webhook
targets, and future stream or catalog integrations. These integrations can
observe private library data, call external APIs, spend provider credits, and
return content that may be wrong or untrusted.

ADR 0003 already chooses HTTP addons before in-process plugins. ADR 0004 keeps
AI-like features as external automation first. M5 needs the execution contract:
what addons declare, how providers are enabled, what data they can access, and
how returned results become user-visible without silently mutating canonical
state.

## Decision

Use capability-scoped HTTP addons and external automation providers.

Addon manifests should be explicit JSON documents served by the addon and
stored by Nako when an addon is enabled. The manifest should include:

- addon ID, name, version, protocol version, and base URL;
- resource declarations such as `metadata`, `image`, `subtitle`, `catalog`,
  `recommendation`, `automation`, and future `stream`;
- declared input and output schemas or schema names;
- required scopes, for example item metadata read, catalog read, suggestion
  write, webhook event read, or stream URL read;
- authentication mode;
- timeout, retry, and rate-limit hints;
- declared content safety and data-retention notes.

Nako must deny all addon and automation access by default. Users or future admin
policy must explicitly enable a provider/addon and grant scopes.

Automation providers should use configuration records that reference secrets by
environment variable or a future secret store. Job inputs, summaries, logs,
outbox events, and addon manifests must not store plaintext API keys.

External results should be stored as suggestions, generated artifacts, or job
summaries unless a user action or explicit policy accepts them into canonical
metadata. This applies to recommendations, generated summaries, metadata
cleanup, title matching, and other AI-like workflows.

HTTP handlers and domain services should not call addons directly. Calls should
go through bounded workers with resource classes:

- `network.webhook`;
- `network.addon`;
- `automation.external_api`.

Every call must have a timeout, retry policy, idempotency key when side effects
are possible, safe error mapping, and structured logs that omit secrets.

Addon and automation requests should use Nako IDs, public API URLs, and small
metadata snapshots. They must not expose raw local filesystem paths unless a
future explicit local-trust mode is designed.

## Consequences

- Extension behavior is auditable and safer for self-hosted users.
- JavaScript/TypeScript SDKs can be added later without embedding a JS runtime.
- Addons can fail independently from the Nako server process.
- The manifest schema needs compatibility rules as resources evolve.
- Users may need to run sidecar services for custom local addons.

## Alternatives Considered

- Native plugin ABI: rejected by ADR 0003 due to crash isolation, versioning,
  and sandboxing risks.
- Embedded JavaScript runtime: deferred because it adds sandboxing and package
  management complexity before the HTTP protocol is proven.
- Hard-code one automation provider: rejected because users may prefer
  different OpenAI-compatible gateways, recommendation services, or local
  sidecars.
- Let automation mutate canonical metadata directly: rejected because generated
  output needs provenance and user/policy acceptance.

## Related Workstreams

- `docs/workstreams/addons-automation/`
