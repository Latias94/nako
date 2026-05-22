# AI Assisted Library Ops Design

Status: Active
Last updated: 2026-05-22

## Why This Lane Exists

Taru already has an external automation foundation and durable Automation
Artifacts, but the product boundary is still too generic for AI-assisted media
library operations. Generated outputs can be helpful for title matching,
metadata cleanup, summaries, recommendations, and intake triage, yet the risk
is high: a plausible AI answer can overwrite trusted metadata, leak paths or
provider secrets, bypass NFO sidecar policy, or create an unreviewable catalog
mutation.

The first-principles boundary is therefore not "run a model"; it is "turn an
untrusted generated proposal into a reviewable Taru artifact that can only
become canonical through an explicit acceptance workflow."

## Relevant Authority

- Glossary and policy:
  - `CONTEXT.md` — **Generated Artifact** and **Acceptance Workflow**
- ADRs:
  - `docs/adr/0004-ai-as-external-automation-first.md`
  - `docs/adr/0007-metadata-merge-policy-and-local-authority.md`
  - `docs/adr/0008-nfo-as-local-metadata-boundary.md`
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0027-admin-api-boundary-for-web-console.md`
- Completed boundaries:
  - `docs/workstreams/addons-automation`
  - `docs/workstreams/metadata-provider-breadth`
  - `docs/workstreams/link-apply-and-import-promotion`
  - `docs/workstreams/nfo-sidecar-promotion-apply`
  - `docs/workstreams/downloads-watch-folder-intake`
  - `docs/workstreams/network-access-boundary`
- Related code:
  - `crates/taru-core/src/automation.rs`
  - `crates/taru-automation/src/lib.rs`
  - `crates/taru-server/src/app/automation.rs`
  - `crates/taru-server/src/http/automation.rs`
  - `crates/taru-api/src/extension.rs`
  - `crates/taru-api/src/admin.rs`

## Problem

The existing automation layer can enqueue external automation jobs and persist
proposed artifacts. It does not yet answer product-critical questions:

- which generated artifacts are safe to show to operators;
- which artifact kinds can target a Media Item, Media Source, Library, or intake
  candidate;
- how confidence, explanation, provenance, and stale target checks are modeled;
- how an accepted title-match or metadata-cleanup proposal maps into existing
  metadata authority / NFO / promotion apply workflows;
- how Admin diagnostics expose proposal queues without prompt payloads, provider
  secrets, raw source locators, local paths, or generated raw text that might
  contain sensitive data;
- how to prove AI can help without creating autonomous writes.

## Target State

When this lane closes:

- Taru has a stable Generated Artifact proposal/readiness vocabulary for
  AI-assisted title matching, metadata cleanup, summaries, and recommendations.
- Existing Automation Artifacts are wrapped or deepened into an operator-facing
  proposal queue with redacted Admin diagnostics.
- Generated Artifact acceptance is explicit, idempotent, auditable, and never
  implies autonomous canonical metadata, sidecar, or library-file mutation.
- Accepted metadata/title proposals route through existing Taru authority
  boundaries rather than introducing direct AI writes.
- Provider prompts, secrets, raw local paths, source locators, raw generated
  payloads, and internal credentials remain out of Public Client API and Admin
  diagnostics unless deliberately summarized/redacted.
- Public Client API and `taru-client-protocol` remain unchanged unless a
  dedicated client-contract lane is opened.
- Local model runtime, embeddings/vector DB, and Addon distribution remain
  split follow-ons.

## In Scope

- AI-assisted library ops workstream docs and task ledger.
- Generated Artifact proposal categories, status/readiness, provenance,
  confidence, target, and stale-target checks.
- Admin-only generated artifact proposal diagnostics and review/readiness
  surfaces.
- Acceptance planning for title-match and metadata-cleanup proposals through
  existing metadata authority and NFO/apply boundaries.
- Tests proving generated artifacts do not mutate canonical metadata or library
  files without explicit acceptance.
- Redaction tests for prompts, provider secrets, raw generated payloads, local
  paths, source locators, and private provider responses.

## Out Of Scope

- Local LLM/model runtime, embedding pipeline, vector DB, GPU scheduling, or
  model download/cache management.
- Direct autonomous writes to Canonical Metadata, NFO sidecars, Managed Import
  artifacts, Media Sources, or library files.
- Addon runtime/distribution or manifest marketplace UX.
- Protocol downloader adapters, background watch scheduling, or remote network
  exposure.
- Public Client API/SDK changes.
- Provider-specific OpenAI/Anthropic/local sidecar implementation unless split
  as a provider adapter lane after the proposal/acceptance boundary is stable.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| AI should enter through external automation first, not a local runtime. | High | ADR 0004 and `taru-automation` foundation | If local model execution becomes urgent, split a runtime lane after generated artifact semantics are stable. |
| Generated Artifact acceptance must reuse existing metadata/import/NFO authority. | High | `CONTEXT.md`, metadata/NFO/apply workstream closeouts | If an artifact needs a new apply target, define it as a separate accepted side-effect lane. |
| Admin-only diagnostics are the first operator surface. | High | ADR 0027 and prior Admin read models | If Public Client display is needed, open a client-contract lane with protocol gates. |
| Raw prompts and raw model outputs can contain sensitive data. | High | Providers may receive paths, titles, external IDs, or user-provided text | Store bounded/redacted summaries for Admin review; keep raw payloads internal or absent unless policy says otherwise. |

## Architecture Direction

Keep AI as an external proposal source and Taru as the authority boundary:

```text
taru-core / taru-db
  Own stable Generated Artifact proposal records, target identity,
  provenance, confidence, status, stale-target evidence, and repository
  contracts.

taru-automation
  Own provider job execution and conversion from external outcome into
  proposed artifacts. It must not apply canonical mutations.

taru-server::app
  Own proposal queue/readiness, acceptance planning, idempotent accept/reject,
  and routing accepted changes into existing metadata/import/NFO apply
  workflows.

taru-api::admin / taru-server::http::admin
  Own Admin-only proposal diagnostics and review contracts. Public Client API
  stays untouched.
```

## AILO-020 Backend Proposal Queue Baseline

AILO-020 established the backend read model for Generated Artifact proposals
without introducing a parallel AI artifact store. Existing Automation Artifacts
remain the durable source of truth. The new proposal projection adds:

- target identity: Library, Media Item, Media Source, or blocked untargeted
  artifact;
- provenance: provider id/name, job id, capability, redacted prompt and
  idempotency fingerprints, attempt count, artifact creation time;
- payload summary: JSON validity, shape, byte count, payload fingerprint,
  object/array counts, textual/explanation booleans, confidence in milli units;
- readiness: ready, blocked, or stale with explicit reasons for invalid JSON,
  already accepted/rejected artifacts, missing provider/job/target records,
  target mismatch, or job-input mismatch.

The projection intentionally omits raw prompt JSON, raw generated payload text,
provider secrets, Source Locators, local paths, source fingerprints, and
canonical metadata write operations. Acceptance is still a later workflow;
AILO-020 only makes proposals reviewable and stale-checkable.

## AILO-030 Admin Diagnostics Baseline

AILO-030 exposed the AILO-020 proposal queue through Admin-only diagnostics and
typed Admin Web support. The route
`/admin/v1/automation/generated-artifacts/proposals` returns DTOs owned by
`taru-api::admin`, not by the Public Client protocol. The DTO mirrors only the
safe proposal summary:

- proposal id, kind, capability, artifact status, and target ids;
- provider/job provenance with idempotency and prompt fingerprints;
- payload validity, shape, byte count, fingerprint, counts, explanation/text
  booleans, and confidence;
- readiness status/actionability/reasons and timestamps.

The Admin surface intentionally does not expose raw prompt JSON, raw generated
artifact JSON, provider secrets, Source Locators, local paths, source
fingerprints, provider raw responses, or any acceptance/apply mutation. The
generated Admin TypeScript contract and Admin Web client/data-source/mocks are
kept in sync with the Rust generator, while `taru-client-protocol` remains
unchanged. Acceptance remains AILO-040.

## AILO-040 Acceptance Planning Baseline

AILO-040 added an explicit review boundary for Generated Artifacts without
granting generated output autonomous write authority. The first accepted
operation is `metadata_cleanup` / `metadata_suggestion`:

- `review-plan` computes whether a proposal can be accepted or rejected using
  the AILO-020 readiness model.
- `accept` for metadata-cleanup proposals records the Automation Artifact as
  accepted, but the returned boundary states that canonical metadata, sidecars,
  and library files are not written and that a later metadata authority/apply
  workflow is required.
- `reject` records a rejected proposal as a no-mutation action, even when a
  proposal is stale.
- stale, missing-target, unsupported-kind, or otherwise non-ready proposals
  cannot be accepted.
- Admin review responses expose action/reason/boundary summaries and omit raw
  prompts, raw generated payloads, provider secrets, Source Locators, local
  paths, and source fingerprints.

This is deliberately a planning/review milestone, not an apply milestone. It
prevents "AI accepted" from meaning "AI wrote Canonical Metadata." A future lane
may deepen accepted metadata-cleanup artifacts into metadata authority changes
or NFO sidecar apply requests, but that must reuse the existing authority/apply
contracts and add its own evidence.

## Closeout Condition

This lane can close when:

- generated artifact proposal/readiness semantics are explicit and tested;
- Admin diagnostics expose safe proposal summaries and review state;
- acceptance planning proves no autonomous canonical metadata, sidecar, or file
  writes;
- at least one concrete AI-assisted operation, likely title-match or
  metadata-cleanup proposal review, is routed through existing authority
  boundaries;
- redaction gates cover prompts, provider secrets, raw generated output, local
  paths, source locators, and provider payloads;
- provider-specific adapters, local model runtime, vector search, Addon
  distribution, and Public Client API changes are split or deferred.
