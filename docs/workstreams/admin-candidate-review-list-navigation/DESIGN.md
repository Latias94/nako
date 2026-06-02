# Admin Candidate Review List Navigation - Design

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

`admin-web-provider-depth-governance` shipped the durable Candidate Review
detail and apply flow, but the Web route currently requires a known
`review_id`. Operators still need an Admin/Web way to discover durable Candidate
Reviews for a Media Item and then navigate into the existing governance route.

The backend already has `list_metadata_candidate_reviews_for_item`, so the next
correct slice is an Admin API/Web navigation surface, not a schema or hierarchy
application lane.

## Target State

When this lane closes:

1. Admin API exposes an item-scoped, paginated, redaction-safe Candidate Review
   list.
2. List entries summarize review identity, status, source, root metadata
   summary, related evidence counts, and accepted-review application action
   without raw provider payloads.
3. Web Admin can show Candidate Reviews for an item and route the operator to
   the existing Candidate Review detail/apply page.
4. Generated Admin TypeScript contracts and Web data-source mapping stay in
   sync with Rust DTOs.
5. Global review queues, batch governance, and related hierarchy application
   remain follow-ons.

## In Scope

- Admin DTOs and route inventory for item-scoped Candidate Review lists.
- Server app/HTTP read path using the existing Candidate Review repository
  list method.
- Generated Admin TypeScript contract sync.
- Web Admin data-source and route-state support for item-scoped list/navigation.
- Redaction, pagination, and no-write tests.

## Out Of Scope

- Public Client API.
- Related Provider Subject, child Provider Mapping, or Media Item hierarchy
  application.
- Global Candidate Review queue/search across all items.
- Batch accept/apply or provider governance bulk actions.
- Raw provider payload, token, header, proxy URL, local path, or image URL
  exposure.
- Schema migrations unless implementation proves the existing repository seam is
  insufficient.

## Architecture Direction

### Item-Scoped First

An item-scoped list is enough to connect current Admin item context to the
existing Candidate Review detail/apply route. A global review queue has
different filtering, pagination, and operator workflow requirements and should
not be hidden in this lane.

### Summary, Not Detail Duplication

The list should not duplicate the full Candidate Review graph. It should expose
summary facts needed for triage and navigation, then link to
`GET /admin/v1/metadata/candidate-reviews/{review_id}` for full evidence.

### Read-Only Before Web Mutation Reuse

The first executable task is read-only. Web should route into the existing
detail/apply route rather than adding another apply mutation entrypoint.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Media Item, Metadata Candidate Review, Provider Subject, Provider Mapping, Admin API, and Public Client API terms. |
| Metadata ADRs | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md`; `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md`; `docs/adr/0021-video-first-media-server-domain-model.md` | Keeps provider evidence separate from Canonical Metadata and prevents raw provider/secret leakage. |
| Prior governance closeout | Covered | `docs/workstreams/admin-web-provider-depth-governance/CLOSEOUT.md` | Confirms direct detail/apply exists and review discovery is the residual product gap. |
| Repository seam | Covered | `crates/nako-core/src/repository/metadata.rs`; `crates/nako-db/src/sqlite/metadata_candidate_review.rs` | Existing item-scoped list method makes a no-schema first slice realistic. |
| Architecture lane map | Covered | `docs/architecture/LANES.md`; `docs/architecture/LIBRARY_PIPELINE.md` | Routes this as the active library-metadata-control-plane follow-on. |

## First Executable Task

`ACRN-020` adds the Admin API item-scoped Candidate Review list. It must be
read-only and redaction-safe.
