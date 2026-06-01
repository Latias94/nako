# Generated Artifact Provider Mapping Breadth - Design

Status: Active
Last updated: 2026-06-01

## Problem

Accepted metadata Generated Artifacts can now be applied one item at a time or
in a guarded bulk batch, but the apply path only interprets neutral Canonical
Metadata fields such as title, overview, runtime, genres, and tags.

That leaves a valuable class of addon/AI evidence unused:

- an addon may know that a Nako item maps to a TMDB, Douban, Bangumi, IMDb, or
  other Provider Subject;
- existing Provider Subject and Provider Mapping repositories can already
  represent that identity;
- Admin Catalog Governance can review Provider Mappings, but Generated Artifact
  apply cannot yet turn accepted generated evidence into a host-owned mapping.

The risk is that provider identity can become a hidden side effect. If a review
acceptance directly upserts Provider Mappings, addons would effectively mutate
catalog identity without the Metadata Authority step that GAMA/GABMA created.

## Target State

Nako supports provider mapping proposals inside accepted metadata Generated
Artifacts while preserving the existing authority model:

1. addon/runtime submits a metadata Generated Artifact with optional provider
   subject proposal data in its JSON payload;
2. review acceptance continues to stage the artifact only;
3. the Admin metadata apply plan parses supported provider subject proposals and
   returns a redacted, read-only Provider Mapping plan next to field plans;
4. final metadata apply revalidates target freshness, applies unlocked
   Canonical Metadata fields, and upserts accepted Provider Mappings through
   host-owned repository contracts;
5. apply outcomes remain idempotent and durable, so replay never duplicates
   Provider Subjects or Provider Mappings;
6. bulk apply inherits provider mapping counters and per-item outcomes through
   the existing one-artifact apply execution path;
7. Web Admin displays provider mapping effects before confirmation and renders
   result facts without exposing raw prompts, raw provider payloads, Source
   Locators, paths, tokens, or secrets.

## Scope

- Extend Generated Artifact metadata suggestion parsing to recognize supported
  provider subject proposal shapes.
- Add read-only Provider Mapping plan entries and counters to the existing
  metadata apply plan contract.
- Preserve field-lock behavior for Canonical Metadata fields and apply separate
  provider mapping rules for Provider Subjects.
- Upsert Provider Subjects and Provider Mappings through existing repository
  traits, extending atomic outcome persistence if needed.
- Keep final apply idempotent by matching existing Provider Subjects and
  item/subject mappings before inserting new rows.
- Synchronize Admin TypeScript contracts and Web Admin plan/result rendering.
- Add focused Rust/Web tests, docs evidence, and PostgreSQL parity evidence
  when repository transaction behavior changes.

## Non-Goals

- No Generated Artifact review acceptance mutation.
- No provider search, scraping, refresh-depth expansion, or hierarchy repair.
- No automatic provider mapping acceptance from unreviewed addons.
- No Public Client API changes.
- No change to Admin Catalog Governance review semantics except reading the
  Provider Mappings this lane may create.
- No raw provider payload, prompt, idempotency key, Source Locator, host path,
  token, or secret exposure in Admin/Web DTOs.
- No broad apply repair/search workflow; that remains
  `proposed:generated-artifact-apply-operations-repair`.
- No Admin settings restoration.

## Architecture Direction

### Provider Mapping Is A Metadata Authority Effect

Generated Artifact review acceptance marks the artifact accepted. It must not
upsert Provider Subjects, Provider Mappings, Canonical Metadata, sidecars, or
library files.

Provider Mapping mutation is allowed only during final metadata apply, after
the same target freshness checks used by Canonical Metadata apply.

### Plan First, Then Mutate

`GAPM-020` must be read-only. It may parse artifact payload JSON, inspect
existing Provider Subjects and Provider Mappings, and return redacted plan
facts, but it must not write any rows.

The plan should make these cases visible:

- valid new mapping proposal;
- existing accepted mapping for the same subject;
- existing candidate/rejected mapping for the same subject;
- unsupported provider or subject kind;
- missing provider subject key;
- low or invalid confidence;
- target missing or stale;
- no applicable metadata fields and no applicable provider mappings.

### Payload Shape Is Host-Interpreted

The first implementation should avoid a provider-owned free-form schema. The
host should recognize a small explicit shape, for example:

```json
{
  "title": "Example",
  "provider_subjects": [
    {
      "provider": "tmdb",
      "subject_kind": "movie",
      "subject_key": "123",
      "title": "Example",
      "release_year": 2024,
      "locale": "en-US",
      "confidence_milli": 930
    }
  ]
}
```

The worker may choose the exact internal DTO names, but the contract must map
to Nako terms: Provider Subject, Provider Mapping, Provider Subject Kind,
subject key, and confidence. Unknown shapes should be ignored or blocked with
explicit reasons, not passed through raw.

### Known Providers First

Start with typed Nako `ExternalProvider` values that already exist in the
domain model. `ExternalProvider::Other` and provider-specific aliases can be
added later only if the worker can preserve stable parsing, redaction, and
operator comprehension without turning provider payloads into raw catalog
state.

### Outcome Persistence Must Be Atomic Enough

If final apply changes both Canonical Metadata and Provider Mappings, the
outcome should not claim success when only one side commits. Prefer extending
the existing generated-artifact metadata apply outcome transaction to include
Provider Subject/Mapping commits if current repository boundaries are not
atomic enough.

### Bulk Apply Inherits Semantics

Bulk apply should not get a second provider mapping implementation. Once the
one-artifact plan/apply path handles provider mapping proposals, bulk planning
and execution should surface mapping counters and results by reusing that path.

## Source Coverage

| Source | State | Evidence | Impact |
| --- | --- | --- | --- |
| User direction and follow-ons | Covered | GAMA/GABMA closeouts and current planner prompt | Opens provider mapping breadth first. |
| Domain glossary | Covered | `CONTEXT.md` | Uses Provider Subject, Provider Mapping, Generated Artifact, Metadata Authority. |
| Architecture lane maps | Covered | `docs/architecture/LANES.md`, `LIBRARY_PIPELINE.md`, `CONTROL_PLANE.md`, `WORKSTREAM_LINKS.md` | Routes work to `library-metadata-control-plane`. |
| Metadata merge/local authority ADR | Covered | `docs/adr/0007-metadata-merge-policy-and-local-authority.md` | Keeps field locks and local authority intact. |
| Current Generated Artifact apply code | Covered | `crates/nako-server/src/app/automation.rs`, `crates/nako-core/src/automation.rs` | Shows current parser only plans Canonical Metadata fields. |
| Provider Mapping repository model | Covered | `crates/nako-core/src/media/provider.rs`, `crates/nako-core/src/repository/metadata.rs`, `crates/nako-metadata/src/confirmation.rs` | Existing subject/mapping primitives should be reused. |
| Program helper scripts | Missing but non-blocking | `scripts/` lacks `workstream_inventory.py`, `program_status.py`, `validate_orchestration_state.py` | Manual doc/code reconciliation used instead. |

## Stop Conditions

Return to planner coordination before continuing if implementation requires:

- changing addon protocol contracts or the official addon repository;
- accepting provider mappings during review acceptance;
- broad provider search/depth, hierarchy repair, or mapping conflict policy;
- public API or client-facing provider identity changes;
- exposing raw payloads, prompts, paths, Source Locators, tokens, or secrets;
- schema changes outside generated artifact outcome/provider mapping ownership;
- raising Web bundle budgets instead of narrowing the UI slice;
- dirty unrelated files that affect the same scopes.

## First Executable Task

`GAPM-020` completed the read-only Provider Mapping plan support in the
existing Generated Artifact metadata apply plan.

It proves parsing, redaction, current-state comparison, and no-mutation behavior
before any Provider Mapping persistence is added.

Next task: `GAPM-030`, which must add durable/idempotent Provider Subject and
Provider Mapping apply without changing review acceptance semantics.
