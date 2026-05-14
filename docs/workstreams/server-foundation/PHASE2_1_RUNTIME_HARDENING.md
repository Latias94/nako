# Phase 2.1: Runtime Hardening and API Discipline

## Status

Implemented in the current workspace. Phase 2.1 hardens the Phase 2 runtime
foundation before metadata, NFO, playback, addons, automation, and remote
storage work begins.

## Scope

Phase 2.1 adds:

- persisted job input payloads
- explicit job retry and cancellation policy documentation
- offset pagination for current list endpoints
- stable API response, error, job, and pagination envelopes
- local development setup documentation
- test strategy documentation
- project licensing policy and reference-code boundaries

## Job Input Payloads

Jobs now store `input_json`. The payload records durable user intent, such as:

```json
{
  "library_id": "018f0000-0000-7000-8000-000000000001",
  "force": false
}
```

The payload is exposed through the HTTP job envelope as `input`. Future retry
APIs should create a new job row using the previous job's input rather than
mutating the old failed job back to `queued`.

## Pagination

The following routes accept `limit` and `offset`:

```text
GET /libraries
GET /libraries/{library_id}/sources
GET /items
```

Defaults:

- `limit = 50`
- `offset = 0`
- max `limit = 500`

Responses include a `page` object with `limit`, `offset`, and `returned`.

## API Discipline

The current API envelope rules are documented in:

- `docs/api/HTTP_API.md`

The job lifecycle policy is documented in:

- `docs/adr/0006-persist-job-inputs-and-explicit-retry-policy.md`

## Development Workflow

Local setup and testing strategy are documented in:

- `docs/development/LOCAL_SETUP.md`
- `docs/development/TEST_STRATEGY.md`

Licensing and GPL reference-code boundaries are documented in:

- `docs/legal/LICENSING.md`

## Verification

Automated gates:

```powershell
cargo fmt --all -- --check
cargo check --workspace
cargo nextest run --workspace
```

Key test coverage:

- job input persistence
- job success and failure lifecycle states
- job input returned from HTTP
- paginated source and item routes
- invalid pagination mapped to `400 invalid_input`

## Out of Scope

- retry API implementation
- persisted cancellation state
- total-count pagination
- OpenAPI generation
- metadata provider implementation
- NFO import/export implementation
- playback, streaming, or transcode implementation
