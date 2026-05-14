# Architecture Decision Records

This directory tracks architecture decisions for Taru.

## Format

Each ADR should include:

- Status: proposed, accepted, rejected, superseded
- Context
- Decision
- Consequences
- Alternatives considered
- Related workstreams

## Index

- [0001: Use a Modular Monolith Rust Workspace](0001-modular-monolith-rust-workspace.md)
- [0002: Build an Internal VFS Before OS Mounting](0002-internal-vfs-before-os-mounting.md)
- [0003: Prefer HTTP Addons Before In-Process Plugins](0003-http-addons-before-in-process-plugins.md)
- [0004: Treat AI as External Automation First](0004-ai-as-external-automation-first.md)
- [0005: Use Bounded Async Pipelines and Resource Budgets](0005-bounded-async-pipelines-and-resource-budgets.md)
- [0006: Persist Job Inputs and Use Explicit Retry Policy](0006-persist-job-inputs-and-explicit-retry-policy.md)
- [0007: Define Metadata Merge Policy and Local Authority](0007-metadata-merge-policy-and-local-authority.md)
- [0008: Treat NFO as a Local Metadata Boundary](0008-nfo-as-local-metadata-boundary.md)
- [0009: Resolve Provider Secrets from Environment References](0009-resolve-provider-secrets-from-environment.md)
- [0010: Treat Library Presets as Configuration Templates](0010-library-presets-are-configuration-templates.md)
