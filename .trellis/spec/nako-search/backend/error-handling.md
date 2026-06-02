# Error Handling

Search evaluation should be total for valid inputs and conservative for empty or
malformed query state.

## Required Patterns

- Treat empty query text as browse-style matching with filters and pagination.
- Use normalized text matching for title, aliases, body, and facets.
- Ignore malformed facet labels by failing conversion before evaluation when the
  caller builds documents.
- Clamp pagination rather than letting callers request unbounded result sets.

## Forbidden Patterns

- Do not panic on empty aliases, body text, or facets.
- Do not return database or transport errors from pure evaluation.
- Do not silently change projection versions.
- Do not throw away all results because one optional facet is absent.

## Examples

- A document that fails a required facet filter should not be scored.
- A document with no body text can still match on title or alias.
- Results with equal score must sort by item ID for deterministic output.

## Review Checklist

- Is evaluation deterministic?
- Does pagination stay bounded?
- Are empty fields handled without special-case panics?
