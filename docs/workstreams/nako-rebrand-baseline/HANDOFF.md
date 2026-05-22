# Nako Rebrand Baseline — Handoff

Status: Complete
Last updated: 2026-05-22

## Current State

The workstream is open. The product owner explicitly accepted an aggressive
rename because there are no users, no public open-source release, and no
production deployment.

## Current Decision

Do not preserve compatibility aliases for the previous working name. Rename the
active source tree to Nako and keep only reviewed historical or external
residues.

## Completed Scope

The active source tree, package names, generated SDKs, deployment examples,
Admin Web, Android namespace, and official addon local dependency references now
use Nako naming.

## Known Risks

- Android Gradle unit-test tasks are blocked in this local JDK/Gradle setup by
  `Type T not present` before tests execute.
- Repository remote rename remains an external operational task.
