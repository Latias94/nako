# nako-naming Backend Guidelines

`nako-naming` provides deterministic filename/path parsing helpers for local
inference. It turns path evidence into a `ParsedName` hint; it does not confirm
catalog identity or metadata hierarchy.

## Pre-Development Checklist

- Read [Directory Structure](./directory-structure.md) before adding parser
  traits, parser versions, helper functions, or modules.
- Read [Database Guidelines](./database-guidelines.md) before introducing any
  persistence, cache, or catalog lookup.
- Read [Error Handling](./error-handling.md) before changing fallback behavior,
  confidence, or malformed path handling.
- Read [Quality Guidelines](./quality-guidelines.md) before changing parsing
  heuristics, confidence values, parser versioning, or tests.
- Read [Logging Guidelines](./logging-guidelines.md) before adding parser
  diagnostics.

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Parser trait, output records, and helper layout | Filled from code |
| [Database Guidelines](./database-guidelines.md) | No-persistence naming boundary | Filled from code |
| [Error Handling](./error-handling.md) | Total parser behavior and fallback hints | Filled from code |
| [Quality Guidelines](./quality-guidelines.md) | Deterministic heuristics and parser-version discipline | Filled from code |
| [Logging Guidelines](./logging-guidelines.md) | Minimal, redaction-aware diagnostics | Filled from code |

## Authority / Evidence

- `crates/nako-naming/src/lib.rs`
- `crates/nako-naming/Cargo.toml`
- `crates/nako-library/src/local_inference/*`
- `CONTEXT.md`

## Boundaries

- Own path/name parsing hints, evidence source labels, confidence values, and
  parser version identifiers.
- Keep metadata confirmation, provider lookup, hierarchy matching, and catalog
  writes outside this crate.
- Keep parser output serializable for ingestion evidence.
- Keep parsing deterministic and side-effect free.

## Executable Contract Summary

1. Scope / Trigger: path parser heuristics, parser output records, confidence
   scoring, evidence sources, or parser version updates belong here.
2. Signatures: `NameParser::parse_path`, `DefaultNameParser`, `parse_path`,
   `ParsedName`, `ParsedMediaKind`, and `NameEvidenceSource`.
3. Contracts: current parser recognizes `S01E02` and `2x03` episode tokens,
   movie years from 1888 through 2100, dot/underscore separators, and weak
   unknown fallback.
4. Validation & Error Matrix: parsing is total and returns `ParsedName`; weak
   evidence returns `ParsedMediaKind::Unknown` with low confidence.
5. Good/Base/Bad Cases: good output carries evidence value and parser version;
   bad output guesses catalog identity without external confirmation.
6. Tests Required: movie year, SxE, NxM, separator normalization, leading slash,
   weak unknown, confidence, evidence source, and parser version tests.
7. Wrong vs Correct: do not query metadata providers or repositories in the
   parser; emit a hint for library/local inference to evaluate.

## Validation

- Focused:
  `cargo nextest run -p nako-naming --no-fail-fast`
- Library inference contract:
  `cargo check -p nako-naming -p nako-library --tests`
