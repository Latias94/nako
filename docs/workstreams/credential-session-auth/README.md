# Credential Session Auth

Status: Complete
Last updated: 2026-05-26

This workstream owns Nako's first local credential and session authority after
the completed User, Role, and Library Access contract. It adds the backend
capability for administrators to provision local passwords, for users to log in
through the Public Client API, and for issued sessions to participate in the
existing inbound Bearer authentication boundary.

The lane was backend-only by default. Admin Web account forms, Media Web login
screens, desktop shell integration, native mobile account UX, invitations,
OAuth/OIDC, LDAP, and passkeys remain follow-ons until this authority is stable.

Authoritative docs:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `HANDOFF.md`

This lane is complete. It shipped local credential storage, durable user
sessions, Admin API password provisioning, Public Client login/current-account
/logout, Bearer session principal resolution, and refreshed generated client
contracts. Follow-on work should be split into focused lanes for Admin Web
account UI, Media Web login/account switching, cookie transport, invitation
onboarding, account recovery, SSO, and Management Context Links.
