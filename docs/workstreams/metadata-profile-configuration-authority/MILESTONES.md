# Metadata Profile Configuration Authority - Milestones

Status: Completed
Last updated: 2026-05-25

## M0 - Authority Model Freeze

Outcome: The source-of-truth model for Metadata Profile updates is documented.

Exit criteria:

- Source states are named.
- TOML, preset, and Admin semantics are explicit.
- First validation slice is narrow and testable.

## M1 - Restart Persistence Proof

Outcome: Admin-updated Metadata Profiles survive restart unless TOML explicitly
owns the profile.

Exit criteria:

- `LibraryOptions` can persist profile source state without a migration.
- Admin profile updates are marked as Admin-owned.
- Startup reconciliation preserves Admin-owned profile state when desired config
  only provides preset defaults.
- Explicit TOML `metadata.library_profiles` remains authoritative.
- Focused startup/Admin tests pass.

## M2 - Evidence And Handoff

Outcome: The lane has fresh verification evidence and follow-ons are clear.

Exit criteria:

- Focused nextest gates pass.
- `cargo fmt --all -- --check` passes or a precise unrelated blocker is
  recorded.
- `git diff --check` passes.
- TODO, evidence, and handoff are current.

Result: Completed 2026-05-25. Fresh focused nextest gates, formatting, and
whitespace checks passed; follow-ons are documented.
