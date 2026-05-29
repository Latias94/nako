# Web Feature Boundary Reshape - Evidence And Gates

Status: Complete
Last updated: 2026-05-28

## Gate Set

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build
rg -n "src/api/admin|generated/contract|Admin[A-Za-z]+Response" web/src/features/media web/components/ui web/lib
git diff --check
```

The `rg` boundary check should return no shared/media DTO leaks.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-28 | WFBR-010 | Queued after the frontend six-lane roadmap was accepted. | Queued. |
| 2026-05-28 | WFBR-010 | Activated after WTRC completed with `npm --prefix web run test`, `npm --prefix web run check`, `npm --prefix web run build`, and `git diff --check` passing. | Active. Current task is WFBR-020. |
| 2026-05-28 | WFBR-020 | Moved Media surface and current internal product pages to `web/src/features/media`; route imports now use the feature index; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build`; `rg -n "src/api/admin\|generated/contract\|Admin[A-Za-z]+Response" web/src/features/media web/components/ui web/lib`. | Passed. Boundary grep returned no DTO leak matches. |
| 2026-05-28 | WFBR-030 | Moved Admin surface, admin child pages, and transcode settings to `web/src/features/admin`; route imports now use the admin feature index; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build`; shared/media DTO boundary grep; admin generated DTO grep. | Passed. Shared/media grep returned no Admin DTO leaks, and admin feature does not import generated DTO contract types directly. |
| 2026-05-28 | WFBR-040 | Moved account, notifications, settings, setup, and TV surfaces under `web/src/features/*`; moved router and surface switcher under `web/src/shell`; `npm --prefix web run check`; `npm --prefix web run test`; `npm --prefix web run build`; `rg -n "@/components/nako\|components/nako" web/src web/components web/lib`. | Passed. No `components/nako` imports remain; Vite emitted separate feature chunks for account, notifications, setup, settings, and TV. |
| 2026-05-28 | WFBR-050 | `npm --prefix web run test`; `npm --prefix web run check`; `npm --prefix web run build`; `rg -n "src/api/admin\|generated/contract\|Admin[A-Za-z]+Response" web/src/features/media web/components/ui web/lib`; `git diff --check`. | Passed. Feature-boundary lane closed and WROP activated. |
