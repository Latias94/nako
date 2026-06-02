# Quality Guidelines

Naming changes should be deterministic, explainable, and covered by examples
because the output may feed later ingestion decisions.

## Required Patterns

- Keep `parse_path` side-effect free.
- Keep `DefaultNameParser` as a lightweight trait adapter around `parse_path`.
- Normalize dots and underscores to spaces before token parsing.
- Trim bracket and punctuation wrappers when parsing years and episode tokens.
- Preserve the original file name as `evidence_value` for file-name evidence.
- Keep parser version stable unless output semantics change.
- Update confidence tests when heuristic confidence changes.

## Forbidden Patterns

- Do not use regex-heavy or allocation-heavy parsing without evidence it is
  needed.
- Do not add provider matching, fuzzy catalog lookup, or hierarchy confirmation.
- Do not use system locale, current date, network calls, filesystem metadata,
  or environment variables to influence output.
- Do not classify weak evidence as a movie just because a file has a media
  extension.

## Tests Required

- Movie title plus year.
- Episode `SxxEyy` pattern.
- Episode `NxM` pattern.
- Dot and underscore separator normalization.
- Leading slash path handling.
- Weak evidence fallback to unknown.
- Parser version, evidence source, evidence value, and confidence values.

## Gate Selection

- Focused:
  `cargo nextest run -p nako-naming --no-fail-fast`
- Caller compile:
  `cargo check -p nako-naming -p nako-library --tests`

## Review Checklist

- Would the same input produce the same output on every platform?
- Are confidence and parser version implications documented in tests?
- Are hints clearly separate from confirmed metadata identities?
- Does the parser stay dependency-light with serde as the only runtime
  dependency?
