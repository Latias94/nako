# Admin Web Addon Onboarding

Status: Completed
Opened: 2026-05-22
Closed: 2026-05-22
Owner: codex

This workstream productizes the first-run Addon Sidecar onboarding flow in
Admin Web without changing Taru's lifecycle boundary. Taru registers and
validates Addon manifests, generates operator guidance, and verifies health;
operators still own Docker, systemd, package installation, process start/stop,
updates, logs, and removal outside Taru.
