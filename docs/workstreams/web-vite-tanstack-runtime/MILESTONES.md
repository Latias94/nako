# Web Vite TanStack Runtime - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope Freeze

Status: done.

Exit criteria:

- Next-to-Vite target state is explicit.
- Non-goals exclude product feature wiring and visual redesign.
- First Vite proof task is chosen.

## M1 - Vite Runtime Proof

Status: done.

Exit criteria:

- Vite entry files exist.
- Root layout concerns no longer depend on Next metadata/layout.
- `npm --prefix web run check` and `npm --prefix web run build` pass.

## M2 - Next Runtime Deletion

Status: done.

Exit criteria:

- Next app wrappers/config/type files are deleted.
- `package.json` and lockfile no longer include Next runtime dependencies.
- Release source has no `next` or `next/*` imports.
- `/tv` remains routable through TanStack Router.

## M3 - Browser And Desktop Static Proof

Status: done.

Exit criteria:

- Vite build output has bundle evidence.
- Static route fallback is smoke-tested.
- Tauri consumes `dist` and builds without a Node sidecar.

## M4 - Closeout

Exit criteria:

- Final gates are recorded.
- Workstream docs match shipped behavior.
- Follow-ons are split or deferred.
