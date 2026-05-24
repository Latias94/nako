# Addon Ecosystem Foundation

Status: Active
Last updated: 2026-05-25

This workstream turns the Addon sidecar direction into a stronger ecosystem
foundation before Nako adds broad official addon features.

Authoritative docs:

- [Design](DESIGN.md)
- [TODO](TODO.md)
- [Milestones](MILESTONES.md)
- [Evidence and Gates](EVIDENCE_AND_GATES.md)
- [Handoff](HANDOFF.md)

The lane records the decision that official addons should use fine-grained
permissions with coarse-grained deployment through Addon Packages and Addon
Suites. It then hardens the most load-bearing seams before feature breadth:

- Addon Task request fingerprinting;
- official addon catalog and descriptor drift prevention;
- Addon Event Subscription delivery;
- the first official event-driven addon proof;
- future Addon Suite, notification, watch-state sync, MCP, Arr-stack, DLNA,
  WebDAV, UPnP, and Network Tunnel Provider ordering.

Non-goals:

- no Native Plugin ABI;
- no Jellyfin Plugin Compatibility;
- no Nako-owned Docker socket, systemd, Kubernetes, SSH, or host-agent
  supervision in this lane;
- no built-in NAT traversal runtime in Nako core;
- no direct AI mutation of canonical state;
- no direct addon database or raw library-path writes.
