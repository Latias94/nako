# Web Admin Acquisition Intake - Closeout

Status: Closed
Closed: 2026-05-29
Task: WAAI-050

## Shipped

The new `web/` shell now exposes `/admin/acquisition/intake` as a read-only
Admin operation route. The route owns query state for `library_id`, `state`,
`source_kind`, `managed_import_artifact_id`, `limit`, and `offset`; maps the
generated Admin contract through `web/src/api/admin`; supports fixture/live
data-source behavior; and renders candidate diagnostics without raw locators,
host paths, credentials, prompt bodies, or downloader internals.

## Deferred

Watch-folder discovery mutation controls are intentionally not part of this
lane. A future guarded mutation lane must define permission, confirmation,
idempotency, loading/failure states, redacted result display, and explicit
no-promotion/no-library-write copy before any UI control is added.

Downloader protocols, public client download UX, mobile/offline policy, and
Managed Import promotion/apply mutations also remain follow-ons.

## Final Evidence

Validation passed on 2026-05-29:

```bash
npm --prefix web run test
npm --prefix web run check
npm --prefix web run build:budget
python -m json.tool docs/workstreams/web-admin-acquisition-intake/WORKSTREAM.json
git diff --check -- docs/workstreams/web-admin-acquisition-intake web/src/features/admin web/src/shell web/src/test web/src/api/admin
```

Browser smoke passed for desktop and mobile viewports:

- Desktop `1280x720`: heading, candidate content, redacted source text, no raw
  path, no prompt body, and no horizontal overflow.
- Mobile `390x844`: heading, candidate content, library filter state, and no
  horizontal overflow.

Screenshots were saved to `target/waai-050-desktop.png` and
`target/waai-050-mobile.png`.

## Review Result

No blocking workstream compliance or code-quality findings remain. The lane is
closed and ready for a new follow-on workstream.
