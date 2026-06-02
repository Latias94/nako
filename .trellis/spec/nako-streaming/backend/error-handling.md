# Error Handling

Streaming planners should convert invalid range input into explicit response
plans instead of throwing transport errors.

## Required Patterns

- Treat malformed range headers as unsatisfiable range plans.
- Treat ranges beyond object length as unsatisfiable range plans.
- Use full-object response plans when range header is absent.
- Keep zero-length object behavior explicit in tests.
- Return deterministic headers for each plan.

## Forbidden Patterns

- Do not panic on malformed range syntax.
- Do not return HTTP framework errors from pure planning code.
- Do not silently clamp out-of-bounds starts into valid ranges.
- Do not assume object length is nonzero without testing that path.

## Examples

- `bytes=0-99` on a 1000-byte object returns partial content.
- `bytes=9999-10000` on a 1000-byte object returns range-not-satisfiable.
- Missing range header returns full content planning.

## Review Checklist

- Is the `Content-Range` value correct?
- Are malformed and out-of-bounds inputs separated in tests?
- Does the caller still own actual byte transport?
