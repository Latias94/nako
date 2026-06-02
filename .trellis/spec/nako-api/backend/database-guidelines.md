# Database Guidelines

`nako-api` has no database ownership.

## Rules

- DTOs may mirror domain concepts, but they must not expose database row shape
  as the public contract.
- Do not import `nako-db`, `sqlx`, or repository traits into this crate.
- Persistence field additions start in `nako-core`/`nako-db`; API exposure is a
  separate contract decision.
- Public/Admin list DTOs should represent bounded, paginated API behavior rather
  than raw table scans.

## Review Checklist

- Is this field safe for the intended API audience?
- Is the field a stable wire contract rather than an adapter detail?
- Does the Admin/Public split still hold?
- Does `nako-server` own the mapping from domain record to DTO?
