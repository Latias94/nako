# Addons and Automation TODO

## M5.0 Design Baseline

- [x] Create addons and automation workstream.
- [x] Add durable event outbox ADR.
- [x] Add capability-scoped addon/provider ADR.
- [x] Define M5 milestone split and validation strategy.
- [x] Update roadmap, goal map, and workstream index.

## Event Outbox

- [x] Define event ID, event type, subject, idempotency key, payload, and status
      domain model.
- [x] Add SQLite event outbox migration.
- [x] Add repository trait and SQLite implementation.
- [x] Add event write points for scan completion.
- [x] Add event write points for metadata refresh and NFO jobs.
- [x] Add event write points for playback session completion where useful.
- [x] Add tests for outbox persistence and idempotency.
- [x] Add tests that event payloads do not contain plaintext secrets or raw
      local paths.

## Webhooks

- [x] Define webhook endpoint configuration and secret references.
- [x] Define webhook event envelope and versioning.
- [x] Define webhook signature format.
- [x] Add delivery attempt persistence.
- [x] Add bounded webhook worker with timeout, retry, and backoff.
- [x] Add safe error mapping and delivery inspection API/CLI.
- [x] Add mocked webhook delivery tests.

## Automation

- [x] Define automation provider configuration and capabilities.
- [x] Define automation job kinds for recommendation, summary, metadata cleanup,
      and title matching.
- [x] Define suggestion/artifact persistence model.
- [x] Add bounded external provider runner.
- [x] Add explicit acceptance policy before generated results mutate canonical
      metadata.
- [x] Add mocked provider tests without real API keys.

## Addon Protocol

- [x] Draft manifest JSON schema and protocol versioning rules.
- [x] Define addon resource request/response envelopes.
- [x] Define addon scopes and explicit enablement model.
- [x] Add addon registration and manifest validation.
- [x] Add bounded addon HTTP caller.
- [x] Add invalid manifest and scope denial tests.
- [x] Add reference addon fixture.

## Documentation

- [x] Add addon author guide.
- [x] Add webhook receiver guide.
- [x] Add automation provider configuration guide.
- [x] Update HTTP API docs when M5 routes are implemented.
- [x] Document M5 known limitations before stabilization.

## Post-M5 Follow-Ups

- [ ] Continue the focused `addon-token-grants-side-effects` workstream for
      Addon Token issuance, rotation, Library-Scoped Addon Grants, and
      Nako-mediated Addon Side Effect intake before allowing addon metadata,
      artwork, subtitle, or Library File Write behavior.
