# Identity And Library Access Contract

Status: Complete
Last updated: 2026-05-26

This workstream owns the first real post-Single-Admin identity model for Nako.
It defines and implements local users, coarse roles, Library Access, bootstrap
administrator behavior, and the database schema baseline needed before Media
Web or Admin Web account controls can ship.

The lane intentionally starts with persistence and contract shape. Admin Web
account CRUD, Media Web login UI, invitations, OAuth/OIDC, LDAP, passkeys, and
parental controls remain follow-ons until the backend authority exists.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`

This lane is complete. Follow-on work should be split into focused lanes for
Admin Web account UI, Media Web login/account switching, invitation onboarding,
and Management Context Links.
