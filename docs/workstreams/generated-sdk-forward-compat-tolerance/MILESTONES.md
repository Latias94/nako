# Generated SDK Forward Compatibility Tolerance — Milestones

Status: Complete
Last updated: 2026-05-21

## M0 — Contract Freeze

Outcome: A concrete tolerant string-enum/API-version representation is chosen
before generator changes begin.

Exit criteria:

- Generated Kotlin enum surfaces and Android adapter usages are inventoried.
- The chosen representation preserves known constants and unknown raw values.
- Workstream docs and gates agree.

Tasks: `SDKFC-010`

Status: Complete.

## M1 — Kotlin SDK Tolerance

Outcome: The generated Kotlin SDK decodes unknown public string values without
failing and keeps known values ergonomic.

Exit criteria:

- `nako-api` generator emits the chosen tolerant representation.
- Checked-in `sdk/kotlin` output is regenerated.
- Kotlin SDK tests cover known and unknown values.
- Generator sync and Kotlin package tests pass.

Tasks: `SDKFC-020`

Status: Complete.

## M2 — Android Consumption Proof

Outcome: Android consumes tolerant generated wire values through app-owned
adapters and produces safe diagnostics or presentation fallbacks.

Exit criteria:

- Connection health/API-version handling uses generated `HealthResponse` while
  still reporting unsupported future versions.
- Unknown public playback/transcode values do not become generic JSON
  invalid-response failures.
- Compose UI state, Media3 runtime, product copy, and redaction remain
  Android-owned.

Tasks: `SDKFC-030`

Status: Complete.

## M3 — Regression And Closeout

Outcome: Cross-SDK and Android gates are fresh; residual questions are split.

Exit criteria:

- Full gate set is recorded in `EVIDENCE_AND_GATES.md`.
- Leakage checks remain green.
- TypeScript compatibility stance is documented.
- `CLOSEOUT.md` exists.

Tasks: `SDKFC-040`, `SDKFC-090`

Status: Complete.
