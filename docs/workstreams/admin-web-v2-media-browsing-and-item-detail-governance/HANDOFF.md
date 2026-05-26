# Admin Web V2 Media Browsing And Item Detail Governance - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

The lane is closed. MBG-010 through MBG-060 are complete:

- Workstream docs exist for the governance-oriented `/catalog` and
  `/items/:itemId` follow-on.
- The lane inherits the closed
  `admin-web-v2-library-management-and-localization` closeout and parity split.
- The lane explicitly excludes watch-first playback, user state, settings
  mutation, repair/apply actions, and broad i18n expansion.
- `ROUTE_API_READINESS.md` accepts public-read bridges for `/items`, `/search`,
  `/items/{item_id}`, item credits/images, and bounded source probes.
- Metadata diagnostics, per-item Generated Artifacts, Admin artwork decisions,
  Provider Mapping, Local Inference, NFO status, and repair/apply actions remain
  split from the first browse/detail slices.
- `/catalog` is now a route-owned governance browse/search entry using
  explicit public read bridges, URL-owned `q`, `facet`, `limit`, and `offset`,
  deterministic fallback, safe row summaries, and stable links to the reserved
  `/items/:itemId` route.
- `/items/:itemId` now renders a governance detail page with Media Item facts,
  Canonical Metadata summary, safe Media Source filenames, bounded source probe
  summaries, public image readiness, split-workflow readiness placeholders,
  support links, deterministic fallback, and redaction tests.
- Source probes are attempted only for at most three sources returned by the
  live item detail response. If item detail falls back to mock data, the data
  source does not continue probing mock source IDs.
- `FOLLOW_ON_SPLIT.md` re-scores item-scoped repair/action gaps and records
  bounded follow-ons for Generated Artifact review/actions, item artwork
  selection, catalog repair/actions, safe metadata diagnostics, item NFO
  status/actions, and playback support detail.
- The recommended next lane after this workstream closes is
  `admin-web-v2-generated-artifact-review-actions`.

## Active Task

- None in this lane. Open
  `admin-web-v2-generated-artifact-review-actions` as the recommended next
  Admin Web V2 lane.
- Evidence: MBG-060 closeout review, Admin Web check/test/build, browser smoke,
  and `git diff --check` are recorded.

## Decisions Since Opening

- Use `/catalog` for governance-oriented browse/search, not a playback-client
  poster wall.
- Use `/items/:itemId` for item inspection and support evidence, not watch-state
  or playback controls.
- Public Client API reads may be reused only through explicitly named bridge
  methods and safe route-local summaries.
- Mutation workflows such as Catalog repair, Generated Artifact accept/reject,
  Artwork selection, Provider Mapping accept, or NFO apply remain follow-ons.
- MBG-030 uses route-owned search params `q`, `facet`, `limit`, and `offset`;
  default browse uses `/items`, search mode uses `/search`.
- Public catalog list/search DTOs do not expose source or image counts, so
  `/catalog` displays source/image readiness as detail-route information rather
  than fabricating counts. MBG-040 populates those facts from item detail and
  bounded source probe summaries.
- MBG-040 keeps the first item detail slice read-only. It does not render raw
  Source Locators, local paths, raw provider payloads, artifact storage handles,
  playback output paths, playback controls, watch state, or repair/apply
  mutations.
- MBG-050 recommends opening Generated Artifact review/actions first because
  the read-only route and generated Admin API review routes already exist. Item
  artwork selection, catalog repair/actions, metadata diagnostics, NFO
  status/actions, and playback support detail remain separate follow-ons.

## Blockers

- None.

## Next Recommended Action

Open `admin-web-v2-generated-artifact-review-actions`. Start with one proposal,
one safe review plan, one explicit accept/reject confirmation, redacted result
summary, focused tests, and browser smoke.
