# Directory Structure

`nako-naming` currently fits in `src/lib.rs`: public parser records, a trait,
one default parser, private helpers, and focused unit tests.

## Current Layout

- `DEFAULT_PARSER_VERSION`: stable version marker for current heuristics.
- `ParsedMediaKind`: parser hint categories such as movie, episode, and
  unknown.
- `NameEvidenceSource`: evidence origin labels.
- `ParsedName`: parser output with title, year, season/episode, confidence,
  evidence, and parser version.
- `NameParser`: trait used by callers that need injectable parsing.
- `DefaultNameParser`: zero-sized default parser delegating to `parse_path`.
- Private helpers for file name/stem extraction, separator cleanup, token
  parsing, year parsing, and title normalization.
- Unit tests for current heuristics.

## Module Rules

- Keep public output records and trait at the crate root.
- Keep parser helpers private unless multiple crates need the exact helper.
- Split parser strategies into modules only when adding a second parser or
  locale-specific strategy.
- Keep metadata, catalog, provider, and repository code outside this crate.

## Naming Rules

- Use `Parsed*` names for parser outputs and hint enums.
- Use `Name*` names for parser traits and evidence concepts.
- Increment `DEFAULT_PARSER_VERSION` when heuristic changes would affect stored
  or compared inference evidence.
- Keep evidence values as original file names when the parser used file-name
  evidence.

## Anti-Patterns

- Do not expose every private helper as public API.
- Do not hide provider or catalog lookup inside parser methods.
- Do not add global parser configuration or mutable process state.
- Do not return bare tuples when the information belongs in `ParsedName`.
