# Metadata Candidate Durable Review

Status: Active
Last updated: 2026-06-02

## Why This Lane Exists

Provider depth work deliberately kept Candidate Graph previews in memory. TMDB
and Bangumi can expose related Provider Subjects, but metadata refresh persists
only the root Provider Subject and accepted root Provider Mapping.

That boundary is correct for automatic refresh. It is not enough for an
operator review workflow: Admin/Web cannot confirm, reject, retain, expire, or
explain related preview nodes until the backend owns a durable, redaction-safe
candidate review contract.

This lane creates that contract before UI work.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Media Item, Candidate Graph, and Canonical Metadata terms. |
| Metadata merge ADR | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md` | Keeps Canonical Metadata and raw provider cache separate from review state. |
| Provider diagnostics ADR | Covered | `docs/adr/0018-metadata-provider-runtime-and-diagnostics.md` | Keeps secrets and raw provider responses out of diagnostics/review records. |
| Video-first domain ADR | Covered | `docs/adr/0021-video-first-media-server-domain-model.md` | Keeps provider-specific subjects mapped through Nako identity. |
| Provider depth closeouts | Covered | `docs/workstreams/tmdb-season-episode-graph-depth/CLOSEOUT.md`; `docs/workstreams/bangumi-relations-and-episode-depth/CLOSEOUT.md`; `docs/workstreams/douban-subject-kind-precision/CLOSEOUT.md` | Establishes endpoint-backed graph preview and root-only persistence boundaries. |
| Existing code boundary | Covered | `crates/nako-core/src/media/candidate.rs`; `crates/nako-core/src/media/provider.rs`; `crates/nako-core/src/repository/metadata.rs`; `crates/nako-metadata/src/confirmation.rs` | Candidate Graph exists; Provider Mapping has Candidate/Accepted/Rejected states; hierarchy confirmation can accept explicit mappings. |

## Target State

When this lane closes:

1. Nako has a provider-neutral Metadata Candidate Review contract.
2. Review records can represent root and related Provider Subjects without
   making them accepted Provider Mappings.
3. Redaction rules prevent raw provider payloads and deployment secrets from
   entering candidate review evidence.
4. Idempotent accept/reject semantics are defined before Admin/Web mutation.
5. Admin/Web provider depth governance remains a separate follow-on.

## In Scope

- Redaction-safe Candidate Graph -> review plan modeling.
- Explicit distinction between preview evidence, candidate review records, and
  accepted Provider Mappings.
- Future durable repository/schema shape after the pure contract is proven.
- Idempotent accept/reject backend semantics before UI.

## Out Of Scope

- Admin API, Public Client API, or Web routes in the first slice.
- Generated Artifact apply outcome table reuse.
- Automatic Media Item hierarchy creation.
- Accepted Provider Mapping writes from preview graph nodes.
- Raw provider response retention beyond the existing raw cache boundary.
- Provider-specific shortcuts for TMDB, Bangumi, or Douban.

## Architecture Direction

### Review Plan First

Start with a pure, testable review plan contract. It should translate
`MetadataCandidateGraph` into stable review facts:

- root subject summary;
- related subject summaries;
- relationship summaries;
- candidate metadata summary suitable for operator review;
- redaction-safe source and provider facts.

The contract must not require a database migration to prove shape and safety.

### Durable Snapshot State

`MCDR-030` adds durable repository/schema support for review snapshots:

- `MetadataCandidateReviewRecord` owns item identity, candidate source,
  `source_key`, status, expiry, timestamps, and the redaction-safe
  `MetadataCandidateReviewPlan`.
- `metadata_candidate_reviews` stores `plan_json`; raw provider response bodies
  stay in the existing raw cache boundary.
- snapshots are idempotent by `item_id`, source, and `source_key`;
- SQLite and PostgreSQL adapters expose the same
  `MetadataCandidateReviewRepository` contract.

The durable layer does not reuse Generated Artifact apply outcomes as a generic
candidate queue.

### Mutation Stays Explicit

`MCDR-040` adds backend-only decision semantics for the review record itself:

- accepting or rejecting the same review decision is idempotent;
- conflicting decisions fail with a safe conflict;
- `item_id` and `expected_updated_at_ms` guards prevent stale review decisions;
- expired pending reviews are marked `Expired`;
- decision transitions do not write Provider Mapping rows.

Accepting a review may eventually create or update `ProviderMappingStatus`
records, but only through a named application service split from the review
status transition. Automatic refresh must continue to avoid child Provider
Mapping writes.

## First Executable Task

`MCDR-020` defined a redaction-safe Metadata Candidate Review plan contract from
`MetadataCandidateGraph`. `MCDR-030` made that plan durable without changing
Provider Mapping behavior.

`MCDR-040` answered:

- how accept/reject transitions remain idempotent;
- how stale or expired review decisions are prevented from mutating current
  Media Item state;
- accepted reviews do not create or update `ProviderMappingStatus` records in
  this lane. That mutation should be a named follow-on service.
