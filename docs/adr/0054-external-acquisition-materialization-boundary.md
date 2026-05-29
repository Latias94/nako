# 0054: External Acquisition Materialization Boundary

## Status

Proposed.

## Context

ADR 0050 keeps acquisition resource search read-only and requires external
runner actions to consume host-owned opaque references instead of
browser-submitted raw URLs or passwords.

The official external acquisition runner contract now has a safe action task
shape: `enqueue` accepts `selected_link_ref` or `intake_candidate_ref`, while
status and control operations use `runner_job_ref`. That is enough for a
fixture runner, but not enough for Transmission, qBittorrent, aria2, or an HTTP
downloader. A production sidecar eventually needs the selected magnet, ed2k, or
web link material.

Putting that material back into browser requests or task input would undo the
host-owned boundary. It would make retries, idempotency, audit, and redaction
ambiguous, and would increase the chance of leaking provider URLs, extraction
passwords, access codes, local paths, or tokens through persisted JSON.

## Decision

Nako exposes external acquisition materialization as a host-owned addon runtime
capability, not as browser API input and not as extra raw fields in
`AddonExternalAcquisitionActionRequest`.

The runtime route is:

- `POST /addon/v1/acquisition/materialize`

The request and response schemas are:

- `nako.addon.external_acquisition_materialization.request.v1`
- `nako.addon.external_acquisition_materialization.response.v1`

Materialization requests must bind the runtime call to the already-approved
action context:

- task job identity;
- action declaration identity;
- target reference;
- runner profile id;
- idempotency key;
- operation;
- audit reference;
- purpose, initially `external_acquisition_enqueue`.

Only `enqueue` can materialize `selected_link_ref` or `intake_candidate_ref`.
`cancel`, `pause`, `resume`, and `query_status` must use `runner_job_ref` and
must not materialize acquisition link material.

Materialization responses may contain short-lived runner-consumable link
material over the addon runtime channel. They must not be copied into persisted
task input, task output, browser-visible admin responses, catalog responses, or
diagnostic summaries. Debug representations must redact target refs,
idempotency keys, audit refs, materialization refs, link URIs, and passwords.

## Consequences

- Production downloader adapters can resolve approved references without
  widening browser authority.
- Nako core remains responsible for policy, TTL, audit, idempotency, and source
  resolution.
- Sidecars can stay runner-specific: they request materialization, enqueue the
  external runner, then report only redaction-safe status and facts.
- The protocol surface grows before any Transmission adapter can be safely
  implemented.
- Server work must validate mismatched job, declaration, operation, profile,
  target, idempotency, audit, and expiry cases before this contract is treated
  as production-ready.

## Alternatives Considered

- **Add raw URL/password fields to the action task input:** rejected because task
  input is persisted and browser-influenced. It would violate ADR 0050.
- **Let the sidecar resolve selected-link references itself:** rejected because
  selected-link and intake-candidate storage are host-owned policy surfaces.
- **Use generic side effects or automation runtime calls:** rejected because the
  acquisition runner needs narrower operation, audit, idempotency, and redaction
  rules than generic automation.
- **Delay the contract until the Transmission adapter is written:** rejected
  because adapter implementation would hard-code unsafe shortcuts before the
  boundary is testable.

## Related Workstreams

- `../nako-official-addons/docs/workstreams/official-external-acquisition-runner/`
- `../nako-official-addons/docs/workstreams/official-external-acquisition-materialization/`
- `docs/workstreams/web-admin-acquisition-intake/`
