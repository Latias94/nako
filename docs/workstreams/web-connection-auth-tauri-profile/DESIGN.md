# Web Connection Auth Tauri Profile

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

Nako Web can render in browser and Tauri, but setup/account still behave like
fixture UI. A self-hosted media app needs a durable connection profile, auth
state, server capability checks, and safe local persistence before more live
Admin/Media workflows are wired.

## Target State

- Setup can configure server URL and validate server reachability.
- Browser mode and Tauri mode share connection semantics while keeping native
  persistence behind a Tauri adapter.
- Account/session state is explicit and safe to clear.
- Public Client and Admin data sources read from the connection state instead
  of ad hoc localStorage keys.
- Secrets are not leaked to shared UI, logs, URLs, or route state.

## In Scope

- Connection state module and tests.
- Setup/account UI wiring.
- Tauri profile bridge for non-secret profile data and safe session handling.
- Error, offline, and invalid-server states.

## Out Of Scope

- Designing server-side auth protocols.
- Native playback.
- Multi-server sync.

## Closeout Condition

This lane can close when setup/account can manage a real connection profile,
data sources consume it, and browser/Tauri gates pass without leaking secrets.
