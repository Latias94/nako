# Admin Web Addon Onboarding Handoff

Status: Completed
Last updated: 2026-05-22

## Current State

The workstream is complete. The Admin Web now has a paste-and-preview Addon
manifest onboarding panel that registers Addons as disabled and hands
operators to Install Guide / sidecar start / Health Check next steps.

## Next Task

Recommended follow-on:

- Addon token/grant onboarding UX, with one-time raw token display, rotation,
  revocation, accepted grant editing, and explicit redaction tests; or
- URL-based manifest discovery only after SSRF/trust policy is designed.

## Boundaries To Preserve

- Registration is not installation.
- Health Check verifies sidecar reachability after registration.
- Nako does not manage Docker, systemd, Kubernetes, SSH, host agents, package
  installation, process lifecycle, logs, upgrades, or removal.
- URL-based manifest fetch is out of scope.
