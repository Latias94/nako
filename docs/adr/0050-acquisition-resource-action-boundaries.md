# 0050: Acquisition Resource Action Boundaries

## Status

Proposed.

## Context

Nako now has a first-class addon `resource_search` contract and an
Admin-mediated product flow that lets the host call search addons, return
display-safe candidate cards, and record an explicit selected link as a
`resource_search_selection` acquisition intake candidate.

That creates adjacent product pressure:

- check whether links are alive, require a password/code, or point to an
  unsupported service;
- hand a selected magnet, ed2k, HTTP, or cloud-drive link to an external
  downloader;
- save or transfer a cloud-drive resource into a user-owned account;
- persist extraction passwords or access codes long enough for the later
  acquisition step.

Those capabilities are related to search, but they are not search. Folding them
into `acquisition_search_read` would turn a read-only discovery scope into an
implicit external-action authority and make auditing, retries, secrets,
idempotency, and user consent ambiguous.

## Decision

Nako keeps resource search read-only.

`AddonScope::AcquisitionSearchRead` may only authorize bounded candidate
discovery. It must not authorize link probes that touch third-party services
with credentialed state, downloader commands, cloud-drive save/transfer
actions, acquisition writes, or durable password/code persistence.

Follow-on contracts must be separate:

- **Resource link check:** read-only availability and capability probe for a
  selected host-owned link reference. It returns safe facts such as reachable,
  unavailable, password_needed, unsupported, rate_limited, and checked_at. It
  gets its own scope, timeout, retry, and redaction rules.
- **External acquisition runner:** explicit audited action contract for
  qBittorrent, Transmission, aria2, ed2k handlers, HTTP downloaders, or other
  external runners. It consumes a host-owned selected-link reference, not a
  browser-submitted raw URL.
- **Cloud-drive transfer:** explicit write/action contract for provider account
  operations such as save, transfer, or copy. It must be governed by host
  acquisition policy and account secret references.
- **Password/code references:** host-owned selected-link metadata that may store
  "has_password" facts, redacted display hints, or secret references. Provider
  authentication secrets and resource extraction/access codes remain different
  secret classes.

The default official resource-search addon may include fixture search and
generic disabled-by-default external search adapters, such as a PanSou-compatible
HTTP adapter. Site-specific scrapers, downloader integrations, and cloud-drive
write integrations should be third-party or separately packaged official
capabilities unless they share the same trust, license, deployment, and audit
boundary.

## Consequences

- Search results can be displayed and selected without granting write or
  external-action authority.
- Browser clients continue to receive opaque `search_id` and `selection_id`
  values instead of raw URLs or passwords.
- Link checks can be cached and retried independently from search.
- Downloader and cloud-drive actions can have explicit idempotency keys,
  audit events, cancellation, and failure states.
- Password/code handling can become durable without conflating resource access
  codes with addon provider credentials.
- More protocol DTOs and scopes will be required before downloader or
  cloud-drive workflows can ship.

## Alternatives Considered

- **Let search results include executable hooks:** rejected because it lets a
  read-only provider shape later side effects without host-owned audit and
  policy.
- **Use `automation_run` for all follow-ons:** rejected because generic
  automation is too broad for acquisition-specific authority and makes scoped
  addon grants harder to reason about.
- **Let the browser resubmit selected raw links/passwords:** rejected because
  the host already has a safe transient selection model and should remain the
  owner of raw external resource references.
- **Make the official search addon own downloader/cloud-drive actions now:**
  rejected because those integrations have different credentials, dependencies,
  failure modes, and legal/licensing risk than read-only search.

## Related Workstreams

- `docs/workstreams/addon-resource-search-protocol/`
- `docs/workstreams/addon-resource-search-product-flow/`
- `../nako-official-addons/docs/workstreams/official-resource-search-first-class-protocol/`
