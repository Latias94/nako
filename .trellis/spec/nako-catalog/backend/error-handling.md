# Error Handling

Catalog hydration errors should explain which requested read model could not be
assembled without hiding repository failures.

## Required Patterns

- Use `NakoError::NotFound` for missing explicitly requested graph roots.
- Propagate repository errors with their existing `NakoResult` context.
- Keep optional relationship absence distinct from missing graph roots.
- Validate input IDs before running broad hydration work when practical.

## Forbidden Patterns

- Do not convert repository failures into empty graphs.
- Do not return partial graph replacements as successful complete replacements.
- Do not use provider labels or external IDs as primary lookup success signals.
- Do not panic on absent optional relationship lists.

## Examples

- Item graph hydration should fail when the item itself is absent.
- A known item without tags should hydrate successfully with an empty tag list.
- Provider subjects should be included only when their mappings are accepted.

## Review Checklist

- Is the failed root identified?
- Are optional and required relationships separated?
- Could a caller accidentally publish a partial graph as complete?
