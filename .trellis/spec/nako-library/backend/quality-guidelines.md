# Quality Guidelines

Library workflow changes must preserve deterministic, bounded intake behavior.

## Required Patterns

- Keep scan traversal bounded by `LibraryScannerOptions::max_depth`.
- Keep probe concurrency bounded by `LibraryProbeOptions::max_concurrent_probes`.
- Sort discovered media sources and returned failures deterministically.
- Propagate stale-cache evidence from VFS metadata/listing into summaries.
- Use source fingerprint evidence as duplicate evidence, not source identity.
- Expose source fingerprint escalation decisions on in-memory source observation
  plans only. The decision can recommend partial/full hashing later, but scan
  planning must not execute hashing, merge sources, change repository schema, or
  add API fields in the same slice.
- Keep source fingerprint hash execution isolated in `source_hash.rs`. Partial
  hash execution reads a configured prefix range and returns
  `BackendFingerprint` evidence; full hash execution uses streaming reads and
  returns `ContentHash` evidence. Returned evidence must not expose raw hashes,
  paths, locators, etags, backend URLs, or credentials.
- Persist local inference evidence so provisional hierarchy decisions are
  explainable.
- Require repeated unchanged intake observation evidence before a watcher
  candidate becomes stable. If the observation key changes, stability must
  reset for the next evaluation.

## Forbidden Patterns

- Do not collapse Media Sources across locators automatically.
- Do not let local inference overwrite confirmed canonical metadata during
  rescan.
- Do not turn recursive scan into unbounded filesystem or remote listing work.
- Do not hide watcher debounce state inside runtime schedulers or storage
  admission helpers when the slice is only preparing stable-candidate intake
  evidence.
- Do not hide storage/probe failures as generic skipped counts.
- Do not bypass VFS for local filesystem paths.

## Tests Required

- Scan tests for supported extension filtering, recursion, ordering, stale cache
  propagation, and source fingerprint evidence.
- Intake tests for first-observation inspect state, repeated identical
  observations becoming stable, and changed-observation stability reset.
- Ingestion tests for insert/update/tombstone disposition.
- Probe tests for skip/force/failure persistence and bounded concurrency.
- Local inference tests for provisional hierarchy and evidence.

## Gate Selection

- Focused library:
  `cargo nextest run -p nako-library <filter> --no-fail-fast`
- Cross-crate intake:
  `cargo check -p nako-library -p nako-vfs -p nako-db --tests`

## Scenario: Source Fingerprint Escalation Plans

### 1. Scope / Trigger

- Trigger: source observation planning has fingerprint evidence plus zero or
  more reconciliation candidates and needs a redaction-safe recommendation for
  future hash escalation.
- Scope: `nako-core` owns the pure policy; `nako-library` attaches the decision
  to the in-memory source observation persistence plan.

### 2. Signatures

- Core:
  `SourceFingerprintEvidence::escalation_decision(existing_locator: bool, candidate_count: usize) -> SourceFingerprintEscalationDecision`.
- Library: `SourceObservationPersistencePlan::fingerprint_escalation` has type
  `SourceFingerprintEscalationDecision`.

### 3. Contracts

- Actions are `none`, `partial_hash`, or `full_hash`.
- Reasons are redaction-safe enums such as existing locator, strong evidence, no
  ambiguous candidate, confirm one weak candidate, disambiguate multiple
  candidates, and refresh stale ambiguous evidence.
- Decision fields may include evidence kind, confidence, stale state, and
  candidate count.
- Decision fields must not include raw source locators, paths, etags,
  fingerprints, backend URLs, storage credentials, or provider payloads.
- The plan decision is advisory only. It does not persist new fields, call VFS
  reads, schedule jobs, change Admin/Public API contracts, or change source
  identity behavior.

### 4. Validation & Error Matrix

- Existing locator -> `none` / existing locator.
- Strong non-stale evidence -> `none` / strong evidence.
- No reconciliation candidates -> `none` / no ambiguous candidate.
- One weak non-stale candidate -> `partial_hash` / confirm single weak
  candidate.
- Multiple weak non-stale candidates -> `full_hash` / disambiguate multiple
  candidates.
- Any stale ambiguous candidate set -> `full_hash` / refresh stale ambiguous
  evidence.

### 5. Good / Base / Bad Cases

- Good: a new locator with one weak duplicate candidate records a partial-hash
  recommendation and still creates a duplicate suggestion.
- Base: an existing locator update records no escalation and keeps update
  disposition unchanged.
- Bad: weak source fingerprint evidence automatically merges media sources or
  starts hashing during scan planning.

### 6. Tests Required

- Core unit tests for every action class: no escalation, partial hash, full
  hash.
- Library ingestion tests that assert the decision is present while
  disposition, source IDs, and duplicate relationship counts remain unchanged.
- Focused gate:
  `cargo nextest run -p nako-library source_observation_plan_recommends --no-fail-fast`.

### 7. Wrong vs Correct

#### Wrong

Use weak evidence as source identity or run a file hash from source commit
planning.

#### Correct

Keep the source commit behavior unchanged and expose only a typed advisory
decision for later hash scheduling or operator diagnostics.

## Scenario: Source Fingerprint Hash Execution Kernel

### 1. Scope / Trigger

- Trigger: a future scan/operator workflow decides to execute a prior source
  fingerprint escalation recommendation for one source URI.
- Scope: `nako-library::source_hash` computes redaction-safe hash evidence
  through VFS only.

### 2. Signatures

- `SourceFingerprintHashExecutor<B>::execute(SourceFingerprintHashRequest) ->
  SourceFingerprintHashReport` where `B: StorageBackend`.
- Modes are explicit: `Partial { prefix_bytes }` or `Full`.

### 3. Contracts

- Partial mode must call `StorageBackend::read_range` with a prefix
  `ByteRange` and return `SourceFingerprintEvidenceKind::BackendFingerprint`.
- Full mode must call `StorageBackend::stream_range(uri, None)` and return
  `SourceFingerprintEvidenceKind::ContentHash`.
- Reports may include safe execution facts such as mode and bytes hashed.
- Reports must not include raw SHA-256 digests, source locators, local paths,
  etags, backend URLs, credentials, or raw backend error bodies.
- This kernel does not persist evidence, enqueue work, add Admin/Public API
  fields, mutate duplicate relationships, or automatically merge Media Sources.

### 4. Validation & Error Matrix

- Partial prefix is zero -> `NakoError::InvalidInput`.
- Backend range read unsupported or failed -> propagate the backend
  `NakoError`.
- Backend stream unsupported or failed -> propagate the backend `NakoError`.
- Stream chunk failure -> propagate the chunk `NakoError`.

### 5. Good / Base / Bad Cases

- Good: a future operator-triggered full hash streams bytes and receives strong
  redaction-safe `ContentHash` evidence.
- Base: a single weak candidate can be confirmed with a bounded partial prefix
  read that remains weaker `BackendFingerprint` evidence.
- Bad: source observation planning starts hashing during scan commit or uses a
  raw digest as source identity.

### 6. Tests Required

- Partial range selection and `BackendFingerprint` evidence.
- Full streaming path with no `read_range(None)` fallback.
- Raw hash/path/locator redaction assertions.
- Unsupported and failing backend propagation.
- Focused gate:
  `cargo nextest run -p nako-library source_hash --no-fail-fast`.

## Review Checklist

- Is the workflow bounded?
- Do repeated observations need explicit stable evidence before intake?
- Are source identity and duplicate evidence kept separate?
- Are failures durable and redaction-safe?
- Does confirmed canonical state remain protected?
