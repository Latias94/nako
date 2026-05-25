# Addon Notification Provider Adapters

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

Addon Notification Bridge proved that Nako can register an official
notification sidecar and deliver scheduled `library.scanned` events to an ACK
path. It intentionally did not choose Telegram, Discord, Home Assistant, email,
generic webhooks, or another real provider.

Provider breadth has different risks than the host/scheduler proof: each
provider introduces credentials, network policy, message formatting, provider
rate limits, retry semantics, operator configuration, and redaction surfaces.
Those concerns need their own lane instead of being folded into the bridge
proof after the host contract is already validated.

## Relevant Authority

- ADRs:
  - `docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `docs/adr/0014-durable-event-outbox-for-webhooks-and-automation.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `docs/workstreams/addon-notification-bridge/`
  - `docs/workstreams/addon-event-scheduler-and-replay/`
  - `docs/guides/ADDON_AUTHOR_GUIDE.md`
  - `F:\SourceCodes\Rust\nako-official-addons\addons\notification-bridge\README.md`

## Problem

The official notification bridge can receive events but does not yet notify a
real external sink. Picking a provider casually would hard-code credential,
template, retry, and network assumptions before the product boundary is clear.

## Target State

When this workstream closes:

- the first provider adapter, `http_webhook`, is implemented deliberately as an
  outbound HTTP webhook sink;
- provider credentials are configured and stored by the sidecar/operator, not
  by Nako core;
- message templates and provider payloads are sidecar-owned and redaction-safe;
- Nako still only schedules Addon Events and records delivery attempts;
- official addon smoke tests prove the provider adapter without needing real
  secrets in CI;
- operator docs explain configuration, health diagnostics, failure modes, and
  retry ownership.

## In Scope

- First-provider selection criteria. ANP-010 selected `http_webhook`.
- Sidecar configuration schema and secret reference documentation for the
  chosen provider.
- One narrow provider adapter, if selected.
- Redaction-safe provider diagnostics.
- Official addon tests and smoke fixtures for the chosen provider.
- Nako host tests only if the provider requires a manifest or protocol contract
  change.

## Out Of Scope

- Notification provider credentials or templates in Nako core.
- A provider matrix in the first implementation slice.
- Nako-managed sidecar process/container lifecycle.
- Marketplace hosting, package signing, or Docker socket control.
- New Addon Event scheduler semantics.
- Changing Addon Protocol unless provider work exposes a contract gap.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Provider adapters belong inside `nako-notification-bridge`. | High | ADR 0003, ADR 0015, ADR 0034, ANB-030. | Split into a suite-level sidecar only if deployment shape requires it. |
| The first provider should be narrow and testable without real CI secrets. | High | ANB-020 and ANB-030 used ACK fixtures before breadth. | Live-provider-only validation would make CI brittle and leak-prone. |
| `http_webhook` is the right first provider target. | Medium | It can be tested against a local fixture server, requires no platform account, and can bridge to many external automation sinks. | Reopen ANP-010 before implementation if product requirements demand a platform-specific first adapter. |
| Nako core does not need provider-specific notification concepts. | High | ANB boundaries and Addon Event scheduler closeout. | Reopen protocol design only if the sidecar cannot express required behavior. |
| Provider retry beyond Nako delivery retry is sidecar-owned. | Medium | Current scheduler retries sidecar delivery only. | Add provider attempt diagnostics in sidecar rather than host retry semantics. |

## ANP-010 Provider Selection

Selected provider: `http_webhook`.

The first provider adapter should be an outbound HTTP webhook sink owned by the
notification bridge sidecar. This is deliberately narrower than a provider
matrix and more testable than a platform-specific adapter that requires live
Telegram, Discord, Home Assistant, email, or SMTP credentials.

Selection criteria:

- can be fully validated with a local fixture server and no live CI secrets;
- keeps the target URL, optional shared secret, headers, and outbound network
  behavior in the sidecar/operator boundary;
- does not require a Nako core or Addon Protocol change;
- is useful as a bridge to Discord/Slack/Home Assistant/custom automation
  through provider-owned webhook endpoints;
- has simple retry semantics for the first slice.

Credential boundary:

- webhook target URL is secret-adjacent and must not be logged in full;
- optional shared secret/header value is sidecar-owned and must never be echoed;
- Nako stores neither provider URL nor provider secret.

Template boundary:

- the first send path should use a fixed redaction-safe JSON summary, not a
  user-defined template language;
- the payload should include event identifiers and safe event facts, not raw
  event payload values by default.

Retry boundary:

- the first adapter should not add a sidecar background queue;
- retryable provider HTTP failures can make the sidecar return a retryable safe
  failure to Nako so the existing Addon Event delivery retry is reused;
- richer provider attempt history or background retry should be split if needed.

## ANP-020 Configuration Contract

The `http_webhook` provider configuration is sidecar-owned. Nako core should
not store the webhook URL, shared secret, provider payload template, or provider
retry state.

Runtime settings are read from `nako-notification-bridge` environment
variables:

- `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED`;
- `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL`;
- `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET`;
- `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SECRET_HEADER_NAME`;
- `NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_TIMEOUT_MS`.

The manifest intentionally keeps `secret_reference_fields` empty for this
slice. Operator secret references are deployment concerns, such as Docker
Compose environment substitution or Kubernetes Secret-backed environment
variables, not Nako-managed provider secrets.

Safe diagnostics expose only:

- provider id: `http_webhook`;
- enabled/configured booleans;
- target URL configured/valid booleans;
- shared secret configured boolean;
- custom secret header name configured boolean;
- provider status: `disabled`, `missing_target_url`, `invalid_target_url`, or
  `configured`;
- send path status: disabled.

Diagnostics must not echo webhook URLs, header names, shared secrets, raw event
payload values, or future rendered message bodies.

## ANP-030 Send Path

The first `http_webhook` send path is implemented behind the existing
`library.scanned` event route in `nako-notification-bridge`.

Send behavior:

- default configuration remains ACK-only because `http_webhook` is disabled by
  default;
- when `http_webhook` is enabled and the target URL is valid, the sidecar posts
  one fixed JSON summary to the configured webhook URL;
- the outbound payload includes schema id, event identifiers, safe event facts,
  attempt number, and sorted payload keys only;
- the outbound payload does not include raw event payload values or a rendered
  message body;
- an optional shared secret is sent in the configured header and is never
  echoed in health, diagnostics, event ACK output, or safe failure bodies.

Failure and retry behavior:

- provider HTTP `2xx` responses are treated as sent;
- provider HTTP `408`, `429`, and `5xx` responses are mapped to a safe
  retryable sidecar failure (`503`) so Nako's existing Addon Event delivery
  retry can run;
- transport failures are mapped to the same safe retryable sidecar failure
  without including the target URL or provider error text;
- provider HTTP `4xx` responses other than `408` and `429` are mapped to a
  non-retryable safe sidecar failure (`424`);
- no sidecar background queue or provider attempt history was added in this
  slice.

## Architecture Direction

Keep the host/addon boundary from ANB. Nako emits domain events, evaluates
event filters, schedules delivery to the Addon sidecar, and records
redaction-safe attempts. The notification bridge sidecar owns provider-specific
credentials, templates, HTTP/API calls, provider retry and rate-limit behavior,
and safe diagnostics.

The first implementation should be one vertical `http_webhook` slice. It should
not start a provider matrix. If the webhook sink exposes host/protocol or
security requirements that exceed this lane, split that concern rather than
adding provider-specific logic to Nako core.

## Closeout Condition

This lane can close when:

- a first provider is either implemented and validated or explicitly split;
- provider ownership remains sidecar-only;
- official addon tests and smoke gates pass;
- host gates pass if any host contract changes;
- docs explain operator configuration and redaction boundaries;
- remaining provider breadth is named as follow-on work.

## Closeout Result

ANP-050 closed this lane after ANP-010 selected `http_webhook`, ANP-020 added
sidecar-owned configuration and diagnostics, ANP-030 implemented the
fixture-backed send path, and ANP-040 completed integration/docs verification.

The target state is met:

- `http_webhook` is implemented as a narrow outbound HTTP webhook sink;
- provider URL, shared secret, message shaping, provider calls, and
  provider-specific retry remain sidecar-owned;
- Nako core remains provider-agnostic and owns only Addon Event scheduling and
  delivery-attempt records;
- tests and default smoke pass without live CI secrets;
- docs explain operator configuration, diagnostics, failure modes, and retry
  ownership.

Named follow-ons:

- `addon-notification-platform-adapters`: Discord, Telegram, email, Home
  Assistant, or other platform-specific adapters.
- `addon-notification-template-controls`: operator-controlled message templates
  and per-event payload shaping beyond the fixed JSON summary.
- `addon-notification-provider-attempt-history`: sidecar-owned provider attempt
  history, richer diagnostics, and optional background retry queue.
- `addon-notification-provider-live-smoke`: opt-in live provider drift checks
  outside default CI secrets.
