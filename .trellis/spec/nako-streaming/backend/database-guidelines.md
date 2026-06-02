# Database Guidelines

`nako-streaming` has no database layer. It plans ranges from caller-provided
object length, file name, and optional range headers.

## Required Patterns

- Accept object length and header text from callers.
- Return response plan data that higher layers can combine with storage access.
- Keep source lookup, authorization, and playback-session persistence outside
  this crate.

## Forbidden Patterns

- Do not import repository traits, SQL, or database adapters.
- Do not lookup media source records in direct response planning.
- Do not persist playback sessions or stream metrics here.
- Do not infer storage locator state from file names.

## Tests Required

- Pure unit tests should cover range parsing and response planning.
- Cross-layer tests should live in server/playback crates when database-backed
  source lookup is involved.
