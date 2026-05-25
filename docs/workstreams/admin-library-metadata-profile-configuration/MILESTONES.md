# Admin Library Metadata Profile Configuration - Milestones

Status: Completed
Last updated: 2026-05-25

## M0 - Scope Freeze

Outcome: A narrow Admin API configuration lane exists and does not duplicate
closed scan/acquisition workstreams.

Exit criteria:

- Problem and target state are documented.
- Non-goals are explicit.
- First task has a focused validation plan.

## M1 - Admin Read/Update Proof

Outcome: Admin API can read and replace a Media Library's effective
`MetadataProfile`.

Exit criteria:

- `GET /admin/v1/libraries/{library_id}/metadata-profile` returns the current
  profile.
- `PUT /admin/v1/libraries/{library_id}/metadata-profile` persists a complete
  profile replacement.
- A follow-up read returns the updated profile.
- A scan after update derives behavior from the updated profile.
- Unknown library IDs return the existing library not-found error shape.

## M2 - Contract And Evidence

Outcome: API consumers have an updated Admin TypeScript contract and current
verification evidence.

Exit criteria:

- Admin contract route constants and types include the profile endpoints.
- Generated `apps/admin-web/src/adminApi/generated/contract.ts` matches the
  generator.
- Focused nextest gates pass.
- Follow-ons are split or documented.

Result: Completed 2026-05-25. Admin contract generation is current, focused
nextest gates pass, formatting and whitespace checks pass, and follow-ons are
documented in the handoff.
