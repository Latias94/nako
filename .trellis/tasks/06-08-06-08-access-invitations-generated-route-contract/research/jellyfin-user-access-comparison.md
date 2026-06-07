# Jellyfin User Access Comparison

## Question

What operator workflow guidance can Nako take from Jellyfin's user access
surface without copying implementation?

## Findings

- Jellyfin centralizes account creation, password changes, deletion, and policy
  updates in user-facing controller/service boundaries such as
  `UserController` and `UserManager`.
- Jellyfin guards administrative policy changes with elevated/admin checks and
  includes protection around the last administrator account.
- Jellyfin's model is direct user creation and password/policy mutation. Nako's
  current model is intentionally invitation-first: an Admin creates an
  invitation, the invitee redeems it, and the raw invitation token is only
  returned at creation time.
- Nako already has server tests that prove invitation redemption is one-time and
  Admin list responses do not include raw tokens or token hashes.

## Nako Interpretation

- Do not broaden this slice into Jellyfin-style full account administration.
- Make the existing Nako invitation lifecycle visible as an operator workflow
  because it is already server-backed and safety-tested.
- Keep the Admin Web projection narrow:
  - safe invitation metadata in list rows,
  - one-time token only in the create mutation result,
  - explicit confirmation for revoke,
  - no password, session, or policy mutation UX.

## Files Inspected

- `repo-ref/jellyfin/Jellyfin.Api/Controllers/UserController.cs`
- `repo-ref/jellyfin/Jellyfin.Server.Implementations/Users/UserManager.cs`
- `repo-ref/jellyfin/MediaBrowser.Model/Users/UserPolicy.cs`
- `crates/nako-api/src/admin/access.rs`
- `crates/nako-server/src/http/admin.rs`
- `crates/nako-server/src/http/tests/system.rs`
