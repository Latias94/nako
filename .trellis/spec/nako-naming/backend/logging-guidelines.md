# Logging Guidelines

`nako-naming` currently logs nothing. Pure parsing should stay silent by
default.

## Required Patterns

- Prefer caller-level diagnostics when library ingestion wants to explain local
  inference decisions.
- If parser diagnostics are added, expose structured facts such as parser
  version, evidence source, and kind hint.
- Keep raw paths and file names redacted or caller-controlled in logs.
- Keep tests deterministic and independent from tracing state.

## Suggested Fields

| Field | Purpose |
|-------|---------|
| `parser_version` | Identifies heuristic version |
| `kind_hint` | Movie, episode, unknown, or other hint |
| `confidence_milli` | Confidence score emitted by parser |
| `evidence_source` | Source of the parse evidence |

## Forbidden Patterns

- Do not log every parsed file by default.
- Do not log full local paths, source locators, or user directory names without
  an explicit caller policy.
- Do not initialize tracing or global logging from this crate.
- Do not make logging affect parser output.

## Review Checklist

- Is the diagnostic needed at parser level, or can `nako-library` log the
  inference decision with more context?
- Does the log avoid high-cardinality raw path values?
- Does the parser remain pure and deterministic?
