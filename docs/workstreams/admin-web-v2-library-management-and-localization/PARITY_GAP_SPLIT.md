# Admin Web V2 Parity Gap Split

Status: Accepted
Last updated: 2026-05-25

This document records AWL-050. It re-scores the remaining Jellyfin/Plex-style
admin-console expectations after the library-management and localization
foundation landed.

This is not a compatibility matrix. Nako should use its own domain language and
should stay administration-first. "Parity" means the operator can perform the
same class of management work safely, not that Nako copies another product's
screens, information architecture, or plugin model.

## Current Admin Web V2 Baseline

Completed or live-capable V2 surfaces:

- `/overview`: route-owned operational summary.
- `/jobs`: Admin job list/filter read model.
- `/libraries`: redacted Media Library list.
- `/libraries/:libraryId`: library detail, Metadata Profile full replacement,
  Source inventory bridge summary, and confirmed scan/NFO commands.
- `/catalog/governance`: read-only governance queue.
- `/acquisition/intake`: route-owned intake candidate view.
- `/automation/generated-artifacts`: read-only Generated Artifact proposals.
- `/playback/sessions`: read-only playback session list/filter.
- `/storage/staging`: read-only staging/cache diagnostics.
- `/addons`: read-only Addon operations summary.
- `/settings`: read-only redacted system configuration diagnostics.
- Shell plus library-management copy use the first Admin Web i18n boundary.

The major product risk is no longer "can Admin Web show a V2 page?" The risk is
that follow-on work mixes unrelated authority models: public client browse,
admin settings mutation, User/Role/Library Access, artifact repair, and broad
i18n should not land in one lane.

## Gap Scores

| Area | Current state | Gap score | Split decision |
| --- | --- | --- | --- |
| Media browsing and Item detail | Public browse/item/credits/images/search routes exist, but Admin Web has only governance queues and library detail. | High. Operators cannot inspect a Media Item's sources, Canonical Metadata, artwork, NFO state, Provider Mapping, Local Inference, or playback decision evidence from V2. | Open a dedicated media-browsing/item-detail governance lane first. Keep it read-heavy and administration-supporting, not watch-first. |
| Settings editing and network operations | `/settings` and network diagnostics are read-only through sanitized system config. | High. Runtime config mutation authority is not accepted, and network access checks need self-hosted ownership language. | Split a settings/network mutation design lane. Do not add form writes until the config authority and rollback/audit model are explicit. |
| Users, Roles, and Library Access | Domain language exists and Single-Admin Mode is allowed, but Admin Web has no user or access routes. | High. Library Access constrains playback, sharing, Addon grants, and future per-user state. | Split a user/access lane that starts with read model and Single-Admin Mode transition semantics before broader RBAC UI. |
| Artwork, Generated Artifact review, and Catalog repair | Generated Artifact proposals and catalog governance are read-only; Managed Artwork has backend history, but Admin Web lacks item-scoped artwork/candidate management. | High. Operators need review, accept/reject, apply/repair, rollback/audit, and redaction-safe diff views. | Split governance repair into a vertical lane with detail pages and confirmed mutations. Keep Generated Artifact, Artwork, and Catalog repair decisions explicit instead of adding generic "fix" buttons. |
| Addon operation mutations | `/addons` is read-only, while Admin API already supports registration, status changes, health checks, resource diagnostics, tokens, grants, and install guides. | Medium-high. Backend capability exists, but credential-producing and grant replacement UX need confirmation and redaction rules. | Split Addon operations mutation lane if Addon management is the next priority after media detail. |
| Playback support detail | Session list/runtime diagnostics exist; support evidence route exists, but V2 lacks a session/source detail support workflow. | Medium. Admin Web should diagnose playback, not become the playback client. | Split a playback support detail lane after media detail or settings if operators need deeper troubleshooting. |
| Jobs controls | Job list/filter exists; job detail is known-ID only and retry/cancel semantics depend on durable runtime policy. | Medium. Cancellation exists at the job-runtime level but UI policy is not complete for every job kind. | Split job detail/control lane only after route constants and per-kind cancel/retry semantics are accepted. |
| Full-site i18n | Shell and library-management copy are localized; other routes still have hard-coded English. | Medium. The pattern exists, but migration needs route-by-route tests so API values and diagnostic facts stay stable. | Split an i18n expansion lane after the next workflow lane stabilizes, or fold route-local message IDs into each new route's scope. |

## Recommended Next Lane

Open `admin-web-v2-media-browsing-and-item-detail-governance`.

Reasoning:

- It is the largest remaining Jellyfin/Plex-style expectation that still fits
  Admin Web's governance role.
- It uses existing public client read routes through explicit bridges, which is
  lower risk than inventing settings mutation authority.
- It gives later repair lanes a place to anchor item-scoped actions, artwork
  candidates, provider evidence, NFO state, and playback support evidence.

Suggested first slice:

- Add `/catalog` as a route-owned browse/search entry with safe fallback.
- Add `/items/:itemId` as an administration-supporting item detail route.
- Show Media Item facts, Media Sources, Source inventory context, Canonical
  Metadata summary, artwork/Generated Artifact readiness, NFO/provider evidence
  placeholders, and playback support links.
- Keep playback controls, watch-state, user favorites, settings writes, and
  repair/apply mutations out of the first slice.
- Preserve redaction rules for Source Locators, local paths, provider raw
  payloads, artifact storage handles, and playback output paths.

## Follow-On Lane Candidates

Open these as separate workstreams only when selected for implementation:

1. `admin-web-v2-media-browsing-and-item-detail-governance`
   - Route-owned `/catalog` and `/items/:itemId` for governance-oriented browse
     and item inspection.
   - First gate: route/data-source tests plus desktop/mobile browser smoke.

2. `admin-web-v2-settings-and-network-mutation-authority`
   - Decide whether runtime settings are mutable, staged, restart-required, or
     config-file-owned.
   - Include network access diagnostics, reverse proxy/tunnel ownership, and
     redaction-safe write/audit semantics.

3. `admin-web-v2-users-roles-library-access`
   - Preserve Single-Admin Mode while adding User, Role, and Library Access
     read/update semantics.
   - Model Library Access as a first-class constraint for playback, Addon
     grants, and future per-user state.

4. `admin-web-v2-governance-repair-actions`
   - Add detail/review/apply workflows for Catalog repair, Generated Artifact
     review, and item-scoped Artwork decisions.
   - Require dry-run, confirmation, audit, idempotency, and rollback or
     remediation semantics before destructive changes.

5. `admin-web-v2-addon-operations-mutations`
   - Productize registration, enable/disable, health checks, diagnostics,
     token lifecycle, grant replacement, and install guide presentation.
   - Keep credential-producing flows one-time, redacted, and confirmation-led.

6. `admin-web-v2-playback-support-detail`
   - Add source/session support detail and request-preview diagnostics.
   - Keep Admin Web diagnostic-only; playback remains a client-app workflow.

7. `admin-web-v2-i18n-expansion`
   - Move remaining V2 route copy into message catalogs.
   - Test both locales while keeping API enum/query values and diagnostic
     comparison strings unchanged.

## Closeout Decision For This Lane

This library-management lane closed in AWL-060. It delivered the
library detail workflow, Metadata Profile editing, scan/NFO commands, source
inventory bridge decision, and i18n foundation. Continuing with media browse,
settings mutation, users/access, or repair actions inside this lane would make
the task ledger too broad to verify cleanly.
