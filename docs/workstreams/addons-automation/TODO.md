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

- [ ] Draft manifest JSON schema and protocol versioning rules.
- [ ] Define addon resource request/response envelopes.
- [ ] Define addon scopes and explicit enablement model.
- [ ] Add addon registration and manifest validation.
- [ ] Add bounded addon HTTP caller.
- [ ] Add invalid manifest and scope denial tests.
- [ ] Add reference addon fixture.

## Documentation

- [ ] Add addon author guide.
- [ ] Add webhook receiver guide.
- [ ] Add automation provider configuration guide.
- [ ] Update HTTP API docs when M5 routes are implemented.
- [ ] Document M5 known limitations before stabilization.
