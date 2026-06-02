# Error Handling

NFO errors should distinguish invalid XML, unsupported preservation, storage
failures, and policy decisions.

## Required Patterns

- Return `NakoError::InvalidInput` for invalid XML and invalid sidecar content.
- Return `NakoError::Unsupported` when preservation is requested but unavailable.
- Propagate VFS read/write errors through `NakoResult`.
- Represent skip/update/create/fail as explicit import/export decisions.
- Report codec conflicts during preserving render instead of silently choosing a
  side.

## Forbidden Patterns

- Do not panic on malformed XML.
- Do not treat unsupported preservation as a successful lossy render.
- Do not hide VFS failures as skipped sidecars.
- Do not merge sidecar data into canonical metadata when policy says preview or
  skip.

## Examples

- Invalid XML input should fail parsing with `InvalidInput`.
- A preserving render with conflicting known and unknown fields should report a
  conflict.
- Missing sidecar files may become a create/export decision, not a codec error.

## Review Checklist

- Is the failure a codec, storage, repository, or policy failure?
- Does the decision remain inspectable by callers?
- Could unknown XML content be lost?
